//! HTTP + WebSocket routes.
//!
//! Ported from `jimuzhe/tiez-clipboard` (`src-tauri/src/services/file_transfer/handlers.rs`),
//! GPL-3.0. Route paths and wire formats are unchanged so the upstream web UI
//! (and the TieZ desktop "mobile transfer" client) keep working against this
//! standalone relay.

use axum::{
    body::Body,
    extract::{
        ws::{Message as WsMessage, WebSocket},
        Multipart, Path as AxumPath, Query, State, WebSocketUpgrade,
    },
    http::{header, HeaderMap, StatusCode},
    response::{Html, IntoResponse, Json},
};
use futures::{SinkExt, StreamExt};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt, SeekFrom};
use tokio_util::io::ReaderStream;

use crate::models::*;
use crate::store::*;
use crate::web_ui::render_index;

const SYSTEM_SENDER_ID: &str = "server";
const SYSTEM_SENDER_NAME: &str = "系统";

/// `GET /` - the embedded chat room UI.
pub async fn index(State(state): State<Arc<AppState>>) -> Html<String> {
    Html(render_index(
        &state.cfg.theme,
        &state.cfg.color_mode,
        &state.cfg.logo,
        &state.cfg.title,
    ))
}

/// `GET /healthz` - container health probe.
pub async fn healthz(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let (online, messages) = {
        let inner = state.lock();
        (inner.devices.len(), inner.messages.len())
    };
    Json(serde_json::json!({
        "status": "ok",
        "version": crate::VERSION,
        "online_devices": online,
        "messages": messages,
        "data_dir": state.cfg.data_dir.to_string_lossy(),
    }))
}

/// `GET /poll?last_id=N` - history since a message id (used on page load).
pub async fn poll_messages(
    Query(params): Query<HashMap<String, String>>,
    State(state): State<Arc<AppState>>,
) -> Json<Vec<Message>> {
    crate::maintenance::cleanup_messages(&state);
    let last_id = params
        .get("last_id")
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(0);
    let msgs = {
        let inner = state.lock();
        inner
            .messages
            .iter()
            .filter(|m| m.id > last_id)
            .cloned()
            .collect::<Vec<Message>>()
    };
    Json(msgs)
}

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

pub async fn handle_socket(socket: WebSocket, state: Arc<AppState>) {
    let (mut sender, mut receiver) = socket.split();
    let mut rx = state.ws_tx.subscribe();
    let mut current_device: Option<String> = None;

    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if sender.send(WsMessage::Text(msg.into())).await.is_err() {
                break;
            }
        }
    });

    let state_inner = state.clone();
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if let WsMessage::Text(text) = msg {
                if let Ok(WsIncoming::Identity {
                    device_id,
                    device_name,
                }) = serde_json::from_str::<WsIncoming>(&text)
                {
                    let id = sanitize_id(&device_id);
                    let name = sanitize_display_name(&device_name);
                    let is_new = {
                        let mut inner = state_inner.lock();
                        let is_new = !inner.devices.contains_key(&id);
                        inner.devices.insert(
                            id.clone(),
                            DeviceInfo {
                                id: id.clone(),
                                name: name.clone(),
                                last_seen: now_millis(),
                            },
                        );
                        is_new
                    };
                    current_device = Some(id);
                    broadcast_devices(&state_inner);
                    if is_new && state_inner.cfg.join_notice {
                        broadcast_notice(&state_inner, &format!("{} 已加入", name));
                    }
                }
            }
        }

        if let Some(id) = current_device {
            let left_name = {
                let mut inner = state_inner.lock();
                inner.devices.remove(&id).map(|d| d.name)
            };
            broadcast_devices(&state_inner);
            if let Some(name) = left_name {
                if state_inner.cfg.join_notice {
                    broadcast_notice(&state_inner, &format!("{} 已离开", name));
                }
            }
        }
    });

    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    }
}

/// Push the current device roster to every connected client.
fn broadcast_devices(state: &AppState) {
    let mut devices: Vec<DeviceInfo> = state.lock().devices.values().cloned().collect();
    devices.sort_by(|a, b| a.name.cmp(&b.name));
    let update = serde_json::json!({ "type": "devices_update", "devices": devices });
    let _ = state.ws_tx.send(update.to_string());
}

/// Ephemeral system line: broadcast to live clients, never persisted and never
/// assigned a real id (so it cannot be mistaken for history).
/// `msg_type` is "notice" so the web UI can render it as a small toast instead
/// of a chat bubble (join/leave announcements are not conversation content).
fn broadcast_notice(state: &AppState, text: &str) {
    let msg = Message {
        id: 0,
        direction: "out".to_string(),
        msg_type: "notice".to_string(),
        content: text.to_string(),
        timestamp: now_millis(),
        last_copied_at: None,
        sender_id: SYSTEM_SENDER_ID.to_string(),
        sender_name: SYSTEM_SENDER_NAME.to_string(),
        file_path: None,
    };
    if let Ok(json) = serde_json::to_string(&msg) {
        let _ = state.ws_tx.send(json);
    }
}

/// `POST /messages/{id}/copied`.
/// Copy activity uses server time and is serialized with retention/history writes.
pub async fn message_copied(
    AxumPath(id): AxumPath<u64>,
    State(state): State<Arc<AppState>>,
) -> StatusCode {
    let _history = state.history_ops.lock().unwrap_or_else(|e| e.into_inner());
    let now = now_millis();
    let snapshot = {
        let mut inner = state.lock();
        let Some(msg) = inner.messages.iter_mut().find(|m| m.id == id) else {
            return StatusCode::NOT_FOUND;
        };
        if crate::maintenance::message_expired(msg, now, state.cfg.message_idle_ttl_secs) {
            return StatusCode::NOT_FOUND;
        }
        msg.last_copied_at = Some(now);
        inner.messages.clone()
    };
    rewrite_history(&state.cfg.history_file, &snapshot);
    StatusCode::NO_CONTENT
}

/// `POST /send_text` and `POST /send-text`.
pub async fn handle_text(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ReceiveText>,
) -> axum::response::Response {
    let content = payload.content.trim().to_string();
    if content.is_empty() {
        return (StatusCode::BAD_REQUEST, "Empty content").into_response();
    }
    let sender_id = sanitize_id(&payload.sender_id);
    let sender_name = sanitize_display_name(&payload.sender_name);

    append_message(&state, "in", "text", &content, &sender_id, &sender_name, None);
    (StatusCode::OK, "Text received").into_response()
}

/// `POST /upload` - single shot multipart upload.
pub async fn upload(
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> axum::response::Response {
    let _guard = state.file_ops.lock().await;
    let temp = state.cfg.data_dir.join(format!(".tmp_tiez_{}", unique_suffix()));
    let result = async {
        let mut sender_id = String::new();
        let mut sender_name = String::new();
        let mut uploaded = None;
        loop {
            let field = tokio::time::timeout(std::time::Duration::from_secs(30), multipart.next_field())
                .await.map_err(|_| (StatusCode::REQUEST_TIMEOUT, "Upload timed out"))?
                .map_err(|_| (StatusCode::BAD_REQUEST, "Incomplete multipart upload"))?;
            let Some(mut field) = field else { break; };
            match field.name().unwrap_or("") {
                "sender_id" | "sender_name" => {
                    let id = field.name() == Some("sender_id");
                    let mut bytes = Vec::new();
                    while let Some(chunk) = field.chunk().await.map_err(|_| (StatusCode::BAD_REQUEST, "Invalid field"))? {
                        if bytes.len() + chunk.len() > 1024 { return Err((StatusCode::BAD_REQUEST, "Field too long")); }
                        bytes.extend_from_slice(&chunk);
                    }
                    let text = String::from_utf8(bytes).map_err(|_| (StatusCode::BAD_REQUEST, "Invalid field"))?;
                    if id { sender_id = text; } else { sender_name = text; }
                }
                "file" => {
                    if uploaded.is_some() { return Err((StatusCode::BAD_REQUEST, "One file per request")); }
                    let name = sanitize_filename(field.file_name().unwrap_or("unknown.bin"));
                    let mime = field.content_type().unwrap_or("application/octet-stream").to_string();
                    let mut file = tokio::fs::OpenOptions::new().write(true).create_new(true).open(&temp).await
                        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Cannot create upload"))?;
                    let mut size = 0u64;
                    loop {
                        let chunk = tokio::time::timeout(std::time::Duration::from_secs(30), field.chunk())
                            .await.map_err(|_| (StatusCode::REQUEST_TIMEOUT, "Upload timed out"))?
                            .map_err(|_| (StatusCode::BAD_REQUEST, "Incomplete file"))?;
                        let Some(chunk) = chunk else { break; };
                        size = size.checked_add(chunk.len() as u64).ok_or((StatusCode::PAYLOAD_TOO_LARGE, "File too large"))?;
                        if state.cfg.max_file_bytes > 0 && size > state.cfg.max_file_bytes {
                            return Err((StatusCode::PAYLOAD_TOO_LARGE, "File too large"));
                        }
                        if !crate::maintenance::cleanup(&state, chunk.len() as u64) {
                            return Err((StatusCode::INSUFFICIENT_STORAGE, "Storage limit reached"));
                        }
                        file.write_all(&chunk).await.map_err(|_| (StatusCode::INSUFFICIENT_STORAGE, "Write failed"))?;
                    }
                    file.sync_all().await.map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Flush failed"))?;
                    uploaded = Some((name, mime));
                }
                _ => return Err((StatusCode::BAD_REQUEST, "Unknown field")),
            }
        }
        let (name, mime) = uploaded.ok_or((StatusCode::BAD_REQUEST, "Missing file"))?;
        let final_path = state.cfg.data_dir.join(format!("{}_{}_{}", now_millis(), unique_suffix(), name));
        tokio::fs::rename(&temp, &final_path).await.map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Finalize failed"))?;
        crate::maintenance::record_file(&state, &final_path).map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "File index failed"))?;
        register_received_file(&state, &final_path, mime, &sender_id, &sender_name);
        Ok::<_, (StatusCode, &'static str)>(())
    }.await;
    let _ = tokio::fs::remove_file(&temp).await;
    match result {
        Ok(()) => (StatusCode::OK, "Upload successful").into_response(),
        Err(error) => error.into_response(),
    }
}

fn unique_suffix() -> String {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    format!("{:x}-{:x}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_nanos(), COUNTER.fetch_add(1, Ordering::Relaxed))
}

/// Sequential, idempotent chunk upload. Only matching retries are accepted.
pub async fn upload_chunk(
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> axum::response::Response {
    let mut metadata: Option<ChunkMetadata> = None;
    let mut data = None;
    loop {
        let field = match tokio::time::timeout(std::time::Duration::from_secs(30), multipart.next_field()).await {
            Ok(Ok(field)) => field,
            Ok(Err(_)) => return (StatusCode::BAD_REQUEST, "Invalid multipart upload").into_response(),
            Err(_) => return (StatusCode::REQUEST_TIMEOUT, "Upload timed out").into_response(),
        };
        let Some(field) = field else { break; };
        let name = field.name().unwrap_or("").to_string();
        let bytes = match tokio::time::timeout(std::time::Duration::from_secs(30), field.bytes()).await {
            Ok(Ok(bytes)) => bytes,
            _ => return (StatusCode::BAD_REQUEST, "Incomplete chunk").into_response(),
        };
        if name == "metadata" && bytes.len() > 16 * 1024 {
            return (StatusCode::PAYLOAD_TOO_LARGE, "Metadata too large").into_response();
        }
        if name == "metadata" && metadata.is_none() {
            metadata = serde_json::from_slice(&bytes).ok();
            if metadata.is_none() { return (StatusCode::BAD_REQUEST, "Invalid metadata").into_response(); }
        } else if (name == "data" || name == "file") && data.is_none() { data = Some(bytes); }
        else { return (StatusCode::BAD_REQUEST, "Unexpected field").into_response(); }
    }
    let (Some(mut meta), Some(data)) = (metadata, data) else {
        return (StatusCode::BAD_REQUEST, "Missing metadata or chunk").into_response();
    };
    if meta.upload_id.is_empty() || meta.upload_id.len() > 64 || sanitize_id(&meta.upload_id) != meta.upload_id ||
        meta.total_chunks == 0 || meta.total_chunks > 65536 || meta.chunk_index >= meta.total_chunks {
        return (StatusCode::BAD_REQUEST, "Invalid upload id or chunk index").into_response();
    }
    if state.cfg.max_file_bytes > 0 && meta.total_size > state.cfg.max_file_bytes {
        return (StatusCode::PAYLOAD_TOO_LARGE, "File too large").into_response();
    }
    if data.len() > 1024 * 1024 {
        return (StatusCode::PAYLOAD_TOO_LARGE, "Chunk exceeds 1 MiB").into_response();
    }
    if state.cfg.max_storage_bytes > 0 && meta.total_size > state.cfg.max_storage_bytes {
        return (StatusCode::INSUFFICIENT_STORAGE, "File exceeds storage quota").into_response();
    }
    use sha2::{Digest, Sha256};
    let hash = format!("{:x}", Sha256::digest(&data));
    if meta.chunk_sha256.as_ref().is_some_and(|expected| !expected.eq_ignore_ascii_case(&hash)) {
        return (StatusCode::UNPROCESSABLE_ENTITY, "Chunk checksum mismatch").into_response();
    }
    let index = meta.chunk_index;
    meta.chunk_index = 0;
    meta.chunk_sha256 = None;
    // Avoid browser/user id collisions without altering the public protocol.
    let key = format!("{}:{}", sanitize_id(&meta.sender_id), meta.upload_id);
    let _guard = state.file_ops.lock().await;
    crate::maintenance::cleanup(&state, 0);
    let existing = state.lock().uploads.remove(&key);
    if existing.is_none() && state.lock().uploads.len() >= 1024 {
        return (StatusCode::TOO_MANY_REQUESTS, "Too many upload sessions").into_response();
    }
    let mut session = match existing {
        Some(session) => session,
        None if index == 0 => UploadSession { meta: meta.clone(),
            path: state.cfg.data_dir.join(format!(".tmp_tiez_{}", unique_suffix())),
            chunks: Vec::new(), bytes: 0, updated: now_millis(), completed: false },
        None => return (StatusCode::CONFLICT, "Upload must start at chunk zero").into_response(),
    };
    let result = async {
        if session.meta != meta { return Err((StatusCode::CONFLICT, "Upload metadata changed")); }
        if index < session.chunks.len() {
            if session.chunks[index] != (data.len(), hash.clone()) {
                return Err((StatusCode::CONFLICT, "Retry content differs"));
            }
            if session.completed && !session.path.is_file() {
                return Err((StatusCode::GONE, "Completed file has expired"));
            }
            session.updated = now_millis();
            return Ok(());
        }
        if session.completed || index != session.chunks.len() {
            return Err((StatusCode::CONFLICT, "Out of order chunk"));
        }
        if session.bytes > 0 && !session.path.is_file() {
            return Err((StatusCode::GONE, "Partial upload is no longer available; start a new upload"));
        }
        let size = session.bytes.checked_add(data.len() as u64).ok_or((StatusCode::PAYLOAD_TOO_LARGE, "File too large"))?;
        let final_chunk = index + 1 == meta.total_chunks;
        if size > meta.total_size || (final_chunk && size != meta.total_size) || (!final_chunk && (data.is_empty() || size >= meta.total_size)) {
            return Err((StatusCode::UNPROCESSABLE_ENTITY, "File size mismatch"));
        }
        if !crate::maintenance::cleanup(&state, data.len() as u64) {
            return Err((StatusCode::INSUFFICIENT_STORAGE, "Storage limit reached"));
        }
        let mut file = tokio::fs::OpenOptions::new().create(true).write(true).truncate(false).open(&session.path).await
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Cannot open partial upload"))?;
        // If the previous write was interrupted, discard any unacknowledged tail.
        file.set_len(session.bytes).await.map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Cannot recover upload"))?;
        file.seek(SeekFrom::Start(session.bytes)).await.map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Seek failed"))?;
        file.write_all(&data).await.map_err(|_| (StatusCode::INSUFFICIENT_STORAGE, "Write failed"))?;
        file.sync_all().await.map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Flush failed"))?;
        drop(file);
        if final_chunk {
            if let Some(expected) = &meta.file_sha256 {
                let mut file = File::open(&session.path).await.map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Checksum read failed"))?;
                let mut hasher = Sha256::new();
                let mut buffer = vec![0; 64 * 1024];
                loop {
                    let n = file.read(&mut buffer).await.map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Checksum read failed"))?;
                    if n == 0 { break; }
                    hasher.update(&buffer[..n]);
                }
                if !expected.eq_ignore_ascii_case(&format!("{:x}", hasher.finalize())) {
                    return Err((StatusCode::UNPROCESSABLE_ENTITY, "File checksum mismatch"));
                }
            }
            let final_path = state.cfg.data_dir.join(format!("{}_{}_{}", now_millis(), unique_suffix(), sanitize_filename(&meta.file_name)));
            tokio::fs::rename(&session.path, &final_path).await.map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Finalize failed"))?;
            if crate::maintenance::record_file(&state, &final_path).is_err() {
                return Err((StatusCode::INTERNAL_SERVER_ERROR, "File index failed"));
            }
            session.path = final_path;
            session.completed = true;
            register_received_file(&state, &session.path, meta.content_type.clone().unwrap_or_else(|| "application/octet-stream".into()), &meta.sender_id, &meta.sender_name);
        }
        session.chunks.push((data.len(), hash));
        session.bytes = size;
        session.updated = now_millis();
        Ok::<_, (StatusCode, &'static str)>(())
    }.await;
    state.lock().uploads.insert(key, session);
    match result { Ok(()) => (StatusCode::OK, "Chunk accepted").into_response(), Err(e) => e.into_response() }
}

/// Record an incoming file into the chat stream (and therefore for every other
/// connected device).
fn register_received_file(
    state: &Arc<AppState>,
    final_path: &std::path::Path,
    content_type: String,
    sender_id: &str,
    sender_name: &str,
) {
    let file_name = final_path
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();
    let lower = file_name.to_lowercase();
    let is_image = content_type.starts_with("image/")
        || [".png", ".jpg", ".jpeg", ".gif", ".webp", ".bmp", ".svg", ".ico"]
            .iter()
            .any(|ext| lower.ends_with(ext));
    let is_video = content_type.starts_with("video/")
        || [".mp4", ".mov", ".mkv", ".avi", ".wmv", ".flv", ".webm"]
            .iter()
            .any(|ext| lower.ends_with(ext));

    let msg_type = if is_image {
        "image"
    } else if is_video {
        "video"
    } else {
        "file"
    };

    let saved_path = final_path.to_string_lossy().to_string();
    if let Ok(meta) = std::fs::metadata(final_path) {
        println!(
            "[tiez-share] received {} ({}) from {}",
            file_name,
            human_bytes(meta.len()),
            sender_name
        );
    }

    append_message(
        state,
        "in",
        msg_type,
        &saved_path,
        &sanitize_id(sender_id),
        &sanitize_display_name(sender_name),
        Some(saved_path.clone()),
    );

    // 通知所有在线客户端刷新左侧文件区
    let _ = state.ws_tx.send(
        serde_json::json!({ "type": "files_changed", "removed_message_ids": [] }).to_string(),
    );
}

/// 存储目录里属于服务自身的数据文件：不能列进左侧文件区，
/// 更不能被「删除文件」删掉（否则会把聊天记录一起弄没）。
fn is_reserved_name(name: &str, cfg: &Config) -> bool {
    if name.starts_with('.') {
        return true;
    }
    let hist = cfg
        .history_file
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();
    let hist_tmp = cfg
        .history_file
        .with_extension("jsonl.tmp")
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();
    name == hist || name == hist_tmp
}

/// `GET /files` - 列出存储目录里已收到的文件，供左侧文件区显示。
pub async fn list_files(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let dir = state.cfg.data_dir.clone();
    let mut entries: Vec<serde_json::Value> = Vec::new();
    let mut total_size: u64 = 0;

    if let Ok(read_dir) = std::fs::read_dir(&dir) {
        for entry in read_dir.flatten() {
            let path = entry.path();
            if !crate::maintenance::safe_file(&state, &path) {
                continue;
            }
            let name = match path.file_name() {
                Some(n) => n.to_string_lossy().to_string(),
                None => continue,
            };
            if is_reserved_name(&name, &state.cfg) {
                continue;
            }
            let meta = match entry.metadata() {
                Ok(m) => m,
                Err(_) => continue,
            };
            let size = meta.len();
            let modified = meta
                .modified()
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_millis() as i64)
                .unwrap_or(0);

            let path_str = path.to_string_lossy().to_string();
            let token = state.register_file(&path_str);
            total_size += size;

            entries.push(serde_json::json!({
                "token": token,
                "name": name,
                "size": size,
                "modified": modified,
                "url": format!("/download/{}?name={}", token, urlencoding::encode(&name)),
            }));
        }
    }

    // 最新上传的排最前面
    entries.sort_by(|a, b| {
        let am = a.get("modified").and_then(|v| v.as_i64()).unwrap_or(0);
        let bm = b.get("modified").and_then(|v| v.as_i64()).unwrap_or(0);
        bm.cmp(&am)
    });

    let count = entries.len();
    Json(serde_json::json!({
        "files": entries,
        "count": count,
        "total_size": total_size,
    }))
}

/// `DELETE /files/{token}` - 删除一个已上传的文件。
///
/// 同时移除聊天记录里引用该文件的消息：否则聊天里会留下点不开的死链接，
/// 文件区和聊天记录也会对不上。
pub async fn delete_file(
    AxumPath(token): AxumPath<String>,
    State(state): State<Arc<AppState>>,
) -> axum::response::Response {
    let _files = state.file_ops.lock().await;
    let path_str = { state.lock().tokens.get(&token).cloned() };
    let path_str = match path_str {
        Some(p) => p,
        None => return (StatusCode::NOT_FOUND, "Unknown file").into_response(),
    };
    let path = std::path::PathBuf::from(&path_str);

    // token 只可能指向登记过的路径，这里再兜一层：确认删的确实在存储目录内
    let inside = match (state.cfg.data_dir.canonicalize(), path.canonicalize()) {
        (Ok(dir), Ok(file)) => file.starts_with(&dir),
        _ => false,
    };
    if !inside {
        return (StatusCode::FORBIDDEN, "Refusing to delete outside data dir").into_response();
    }
    if !path.is_file() {
        state.lock().tokens.remove(&token);
        return (StatusCode::NOT_FOUND, "File not found").into_response();
    }

    if let Err(e) = std::fs::remove_file(&path) {
        eprintln!("[tiez-share] failed to delete {:?}: {}", path, e);
        return (StatusCode::INTERNAL_SERVER_ERROR, "Delete failed").into_response();
    }

    crate::maintenance::forget_file(&state, &path_str);

    (StatusCode::OK, "Deleted").into_response()
}

/// Build the message, persist it, and fan it out to every WebSocket client.
/// File messages get a stable `/download/<token>?name=...` URL, exactly like
/// upstream, so pending-upload reconciliation in the UI keeps working.
pub fn append_message(
    state: &Arc<AppState>,
    direction: &str,
    msg_type: &str,
    content: &str,
    sender_id: &str,
    sender_name: &str,
    file_path: Option<String>,
) -> Message {
    let _history = state.history_ops.lock().unwrap_or_else(|e| e.into_inner());
    let mut final_content = content.to_string();
    if matches!(msg_type, "image" | "video" | "file")
        && !final_content.starts_with("data:")
        && !final_content.starts_with("/download/")
        && !final_content.starts_with("http")
    {
        if let Some(path) = file_path.as_ref() {
            let token = state.register_file(path);
            let filename = std::path::Path::new(path)
                .file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| "file".to_string());
            final_content = format!("/download/{}?name={}", token, urlencoding::encode(&filename));
        }
    }

    let msg = Message {
        id: state.next_message_id(),
        direction: direction.to_string(),
        msg_type: msg_type.to_string(),
        content: final_content,
        timestamp: now_millis(),
        last_copied_at: None,
        sender_id: sender_id.to_string(),
        sender_name: sender_name.to_string(),
        file_path,
    };

    let compact = {
        let mut inner = state.lock();
        inner.messages.push(msg.clone());
        if inner.messages.len() > state.cfg.max_history {
            let excess = inner.messages.len() - state.cfg.max_history;
            inner.messages.drain(0..excess);
            Some(inner.messages.clone())
        } else {
            None
        }
    };

    match compact {
        Some(all) => rewrite_history(&state.cfg.history_file, &all),
        None => append_history(&state.cfg.history_file, &msg),
    }

    if let Ok(json) = serde_json::to_string(&msg) {
        let _ = state.ws_tx.send(json);
    }
    msg
}

/// `GET /download/{token}` - streams a stored file with HTTP range support.
pub async fn handle_file_download(
    AxumPath(token): AxumPath<String>,
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let file_path = { state.lock().tokens.get(&token).cloned() };

    if let Some(path_str) = file_path {
        let path = std::path::PathBuf::from(&path_str);
        if crate::maintenance::safe_file(&state, &path) {
            let filename = path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            let mime = mime_guess::from_path(&path)
                .first_or_octet_stream()
                .to_string();
            let is_image = mime.starts_with("image/");
            let is_video = mime.starts_with("video/");
            let encoded_name = urlencoding::encode(&filename);
            let disposition = if is_image || is_video {
                format!(
                    "inline; filename=\"{}\"; filename*=UTF-8''{}",
                    filename, encoded_name
                )
            } else {
                format!(
                    "attachment; filename=\"{}\"; filename*=UTF-8''{}",
                    filename, encoded_name
                )
            };

            if let Ok(mut file) = File::open(&path).await {
                let metadata = match file.metadata().await {
                    Ok(m) => m,
                    Err(_) => {
                        return (StatusCode::INTERNAL_SERVER_ERROR, "Metadata failed")
                            .into_response()
                    }
                };
                let total_size = metadata.len();
                let range_header = headers.get(header::RANGE).and_then(|h| h.to_str().ok());

                if let Some(range) = range_header {
                    if let Some(r) = range.strip_prefix("bytes=") {
                        let parts: Vec<&str> = r.split('-').collect();
                        if parts.len() == 2 && total_size > 0 {
                            let start = parts[0].parse::<u64>().unwrap_or(0);
                            let end = parts[1].parse::<u64>().unwrap_or(total_size - 1);

                            if start < total_size {
                                let end = if end >= total_size {
                                    total_size - 1
                                } else {
                                    end
                                };
                                let content_length = end - start + 1;

                                if file.seek(SeekFrom::Start(start)).await.is_ok() {
                                    let stream = ReaderStream::with_capacity(
                                        file.take(content_length),
                                        64 * 1024,
                                    );
                                    let body = Body::from_stream(stream);
                                    return (
                                        StatusCode::PARTIAL_CONTENT,
                                        [
                                            (header::CONTENT_TYPE, mime),
                                            (header::CONTENT_DISPOSITION, disposition),
                                            (header::ACCEPT_RANGES, "bytes".to_string()),
                                            (
                                                header::CONTENT_RANGE,
                                                format!("bytes {}-{}/{}", start, end, total_size),
                                            ),
                                            (header::CONTENT_LENGTH, content_length.to_string()),
                                        ],
                                        body,
                                    )
                                        .into_response();
                                }
                            }
                        }
                    }
                }

                let stream = ReaderStream::with_capacity(file, 64 * 1024);
                let body = Body::from_stream(stream);
                return (
                    StatusCode::OK,
                    [
                        (header::CONTENT_TYPE, mime),
                        (header::CONTENT_DISPOSITION, disposition),
                        (header::ACCEPT_RANGES, "bytes".to_string()),
                        (header::CONTENT_LENGTH, total_size.to_string()),
                    ],
                    body,
                )
                    .into_response();
            }
        }
    }

    (StatusCode::NOT_FOUND, "File not found").into_response()
}
