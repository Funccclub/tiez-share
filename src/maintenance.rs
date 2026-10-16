//! Quotas and retention. Only registered uploads are automatically deleted.
use std::{collections::HashMap, path::{Path, PathBuf}, sync::Arc, time::{Duration, SystemTime}};
use crate::{models::AppState, store::{now_millis, rewrite_history}};

const INDEX: &str = ".tiez-managed.json";

pub fn message_expired(msg: &crate::models::Message, now: i64, ttl_secs: u64) -> bool {
    ttl_secs > 0 && now.saturating_sub(msg.last_copied_at.unwrap_or(msg.timestamp)).max(0) as u64
        >= ttl_secs.saturating_mul(1000)
}

/// Expire chat records only. Uploaded files retain their independent file policy.
pub fn cleanup_messages(state: &AppState) {
    let _history = state.history_ops.lock().unwrap_or_else(|e| e.into_inner());
    let now = now_millis();
    let (ids, snapshot) = {
        let mut inner = state.lock();
        let mut ids = Vec::new();
        inner.messages.retain(|msg| {
            if message_expired(msg, now, state.cfg.message_idle_ttl_secs) {
                ids.push(msg.id);
                false
            } else { true }
        });
        if ids.is_empty() { return; }
        (ids, inner.messages.clone())
    };
    rewrite_history(&state.cfg.history_file, &snapshot);
    let _ = state.ws_tx.send(serde_json::json!({
        "type": "messages_removed", "removed_message_ids": ids
    }).to_string());
}

pub fn load_index(dir: &Path) -> HashMap<String, i64> {
    std::fs::read(dir.join(INDEX)).ok().and_then(|v| serde_json::from_slice(&v).ok()).unwrap_or_default()
}

fn save_index(state: &AppState) -> std::io::Result<()> {
    let bytes = serde_json::to_vec(&state.lock().managed_files)?;
    let temp = state.cfg.data_dir.join(".tiez-managed.json.tmp");
    std::fs::write(&temp, bytes)?;
    std::fs::rename(temp, state.cfg.data_dir.join(INDEX))
}

// Never follow a symlink or delete a file outside the configured root.
pub fn safe_file(state: &AppState, path: &Path) -> bool {
    let root = state.cfg.data_dir.canonicalize();
    let parent = path.parent().and_then(|p| p.canonicalize().ok());
    matches!((root, parent), (Ok(root), Some(parent)) if root == parent) &&
        std::fs::symlink_metadata(path).is_ok_and(|m| m.file_type().is_file())
}

pub fn record_file(state: &AppState, path: &Path) -> std::io::Result<()> {
    let key = path.to_string_lossy().to_string();
    state.lock().managed_files.insert(key.clone(), now_millis());
    if let Err(e) = save_index(state) {
        state.lock().managed_files.remove(&key);
        let _ = std::fs::remove_file(path);
        return Err(e);
    }
    Ok(())
}

pub fn forget_file(state: &AppState, path: &str) {
    let _history = state.history_ops.lock().unwrap_or_else(|e| e.into_inner());
    let (ids, snapshot) = {
        let mut inner = state.lock();
        let ids: Vec<_> = inner.messages.iter().filter(|m| m.file_path.as_deref() == Some(path)).map(|m| m.id).collect();
        inner.messages.retain(|m| m.file_path.as_deref() != Some(path));
        inner.tokens.retain(|_, p| p != path);
        inner.managed_files.remove(path);
        (ids, inner.messages.clone())
    };
    rewrite_history(&state.cfg.history_file, &snapshot);
    if let Err(e) = save_index(state) { eprintln!("[tiez-share] cannot save file index: {e}"); }
    let _ = state.ws_tx.send(serde_json::json!({"type":"files_changed", "removed_message_ids":ids}).to_string());
}

fn remove_registered(state: &AppState, path: &str) {
    let path_buf = PathBuf::from(path);
    if safe_file(state, &path_buf) {
        if let Err(e) = std::fs::remove_file(&path_buf) {
            eprintln!("[tiez-share] cleanup failed: {e}");
            return;
        }
    } else if path_buf.exists() { return; }
    forget_file(state, path);
}

pub fn disk_usage(state: &AppState) -> u64 {
    std::fs::read_dir(&state.cfg.data_dir).into_iter().flatten().flatten()
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            // Include partial uploads in quota, exclude service metadata/history.
            if name.starts_with('.') && !name.starts_with(".tmp_tiez_") { return None; }
            if e.path() == state.cfg.history_file { return None; }
            std::fs::symlink_metadata(e.path()).ok().filter(|m| m.file_type().is_file()).map(|m| m.len())
        }).fold(0u64, |total, n| total.saturating_add(n))
}

// Caller holds file_ops, so uploads, manual deletion and cleanup cannot race.
pub fn cleanup(state: &AppState, incoming: u64) -> bool {
    cleanup_messages(state);
    let now = now_millis();
    let expired: Vec<_> = state.lock().uploads.iter().filter(|(_, s)|
        now.saturating_sub(s.updated) as u64 / 1000 >= state.cfg.upload_ttl_secs)
        .map(|(id, _)| id.clone()).collect();
    for id in expired {
        if let Some(s) = state.lock().uploads.remove(&id) {
            if !s.completed && safe_file(state, &s.path) { let _ = std::fs::remove_file(s.path); }
        }
    }
    let active: Vec<_> = state.lock().uploads.values().filter(|s| !s.completed).map(|s| s.path.clone()).collect();
    // Also reclaim abandoned partials left by a process restart.
    if let Ok(entries) = std::fs::read_dir(&state.cfg.data_dir) {
        for e in entries.flatten() {
            if e.file_name().to_string_lossy().starts_with(".tmp_tiez_") && !active.contains(&e.path()) && safe_file(state, &e.path()) {
                let age = e.metadata().and_then(|m| m.modified()).ok().and_then(|t| SystemTime::now().duration_since(t).ok());
                if age.is_some_and(|age| age.as_secs() >= state.cfg.upload_ttl_secs) { let _ = std::fs::remove_file(e.path()); }
            }
        }
    }
    let mut files: Vec<_> = state.lock().managed_files.iter().map(|(p,t)| (p.clone(),*t)).collect();
    files.sort_by_key(|(_,t)| *t);
    for (path, created) in &files {
        if state.cfg.file_ttl_secs > 0 && now.saturating_sub(*created) as u64 / 1000 >= state.cfg.file_ttl_secs {
            remove_registered(state, path);
        }
    }
    let limit = state.cfg.max_storage_bytes;
    if limit == 0 { return true; }
    if incoming > limit { return false; }
    for (path, _) in files {
        if disk_usage(state).saturating_add(incoming) <= limit { return true; }
        let registered = state.lock().managed_files.contains_key(&path);
        if registered { remove_registered(state, &path); }
    }
    disk_usage(state).saturating_add(incoming) <= limit
}

pub async fn run(state: Arc<AppState>) {
    let mut ticker = tokio::time::interval(Duration::from_secs(state.cfg.cleanup_interval_secs));
    loop {
        ticker.tick().await;
        let _guard = state.file_ops.lock().await;
        cleanup(&state, 0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Message;

    #[test]
    fn idle_deadline_uses_last_copy_or_original_timestamp() {
        let mut msg = Message { timestamp: 1000, ..Message::default() };
        assert!(!message_expired(&msg, 1000 + 86_400_000 - 1, 86400));
        assert!(message_expired(&msg, 1000 + 86_400_000, 86400));
        msg.last_copied_at = Some(2000);
        assert!(!message_expired(&msg, 1000 + 86_400_000, 86400));
        assert!(message_expired(&msg, 2000 + 86_400_000, 86400));
        assert!(!message_expired(&msg, 0, 86400));
        assert!(!message_expired(&msg, i64::MAX, 0));
    }

    #[test]
    fn legacy_and_copied_history_survive_serialization() {
        let mut msg: Message = serde_json::from_str(r#"{"id":7,"timestamp":1000,"content":"old"}"#).unwrap();
        assert_eq!(msg.last_copied_at, None);
        assert!(message_expired(&msg, 86_401_000, 86400));
        msg.last_copied_at = Some(86_400_000);
        let restored: Message = serde_json::from_str(&serde_json::to_string(&msg).unwrap()).unwrap();
        assert_eq!(restored.last_copied_at, msg.last_copied_at);
        assert!(!message_expired(&restored, 86_401_000, 86400));
    }
}
