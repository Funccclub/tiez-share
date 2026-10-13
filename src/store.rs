//! Persistence and small string/path helpers.
//!
//! The upstream service kept chat state in memory only; a relay that reboots
//! with the router would otherwise lose every message. History is stored as
//! JSON Lines (append-only, cheap on flash, crash tolerant).

use sha2::{Digest, Sha256};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::models::Message;

pub fn now_millis() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

/// Stable, restart-surviving download token derived from the absolute path.
pub fn token_for_path(path: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(path.as_bytes());
    let digest = hasher.finalize();
    let mut out = String::with_capacity(16);
    for b in digest.iter().take(8) {
        out.push_str(&format!("{:02x}", b));
    }
    out
}

/// Strip any directory component and dangerous characters from a client
/// supplied filename. Upstream joined the raw multipart filename into the
/// storage path, which allowed `../../` traversal; this closes that hole.
pub fn sanitize_filename(name: &str) -> String {
    let base = name
        .rsplit(['/', '\\'])
        .next()
        .unwrap_or(name)
        .trim()
        .trim_start_matches('.');
    let mut cleaned: String = base
        .chars()
        .map(|c| if c.is_control() { '_' } else { c })
        .collect();
    if cleaned.chars().count() > 120 {
        cleaned = cleaned.chars().take(120).collect();
    }
    if cleaned.is_empty() {
        "unnamed".to_string()
    } else {
        cleaned
    }
}

/// Keep client ids to a safe character set (they are echoed back to browsers).
pub fn sanitize_id(id: &str) -> String {
    let cleaned: String = id
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
        .take(64)
        .collect();
    if cleaned.is_empty() {
        "anon".to_string()
    } else {
        cleaned
    }
}

pub fn sanitize_display_name(name: &str) -> String {
    let cleaned: String = name
        .chars()
        .filter(|c| !c.is_control())
        .take(32)
        .collect();
    let cleaned = cleaned.trim().to_string();
    if cleaned.is_empty() {
        "未命名设备".to_string()
    } else {
        cleaned
    }
}

/// Read the tail of the history file. Unparseable or partially written lines
/// are skipped rather than aborting startup.
pub fn load_history(path: &Path, max: usize) -> Vec<Message> {
    let raw = match std::fs::read_to_string(path) {
        Ok(raw) => raw,
        Err(_) => return Vec::new(),
    };
    let mut msgs: Vec<Message> = raw
        .lines()
        .filter(|l| !l.trim().is_empty())
        .filter_map(|l| serde_json::from_str::<Message>(l).ok())
        .collect();
    if msgs.len() > max {
        msgs.drain(0..msgs.len() - max);
    }
    msgs
}

pub fn append_history(path: &Path, msg: &Message) {
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let line = match serde_json::to_string(msg) {
        Ok(line) => line,
        Err(_) => return,
    };
    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
        let _ = writeln!(file, "{}", line);
        let _ = file.flush();
    }
}

/// Compaction: rewrite the file with the current window of messages.
pub fn rewrite_history(path: &Path, msgs: &[Message]) {
    let mut buf = String::new();
    for m in msgs {
        if let Ok(line) = serde_json::to_string(m) {
            buf.push_str(&line);
            buf.push('\n');
        }
    }
    let tmp = path.with_extension("jsonl.tmp");
    if std::fs::write(&tmp, buf).is_ok() {
        let _ = std::fs::rename(&tmp, path);
    }
}

/// Human readable byte count, used for logs only.
pub fn human_bytes(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let mut value = bytes as f64;
    let mut idx = 0;
    while value >= 1024.0 && idx < UNITS.len() - 1 {
        value /= 1024.0;
        idx += 1;
    }
    if idx == 0 {
        format!("{} {}", bytes, UNITS[idx])
    } else {
        format!("{:.1} {}", value, UNITS[idx])
    }
}
