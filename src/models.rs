//! Shared data models and application state.
//!
//! Extracted from `jimuzhe/tiez-clipboard` (`src-tauri/src/services/file_transfer/models.rs`),
//! GPL-3.0. The Tauri `AppHandle` coupling was replaced by a plain `Arc<AppState>`.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard};
use tokio::sync::broadcast;

/// One chat/transfer record. The JSON shape is kept byte-compatible with the
/// upstream web UI so the original front-end keeps working unchanged.
#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Message {
    pub id: u64,
    /// "in" = browser -> relay, "out" = relay/system -> browsers.
    pub direction: String,
    /// "text" | "file" | "image" | "video" | "notice"
    /// ("notice" is an ephemeral system announcement like "someone joined",
    /// broadcast live only and never persisted; the web UI renders it as a
    /// small toast instead of a chat bubble.)
    pub msg_type: String,
    pub content: String,
    pub timestamp: i64,
    /// Last successful copy reported by a client; old history falls back to timestamp.
    pub last_copied_at: Option<i64>,
    pub sender_id: String,
    pub sender_name: String,
    pub file_path: Option<String>,
}

impl Default for Message {
    fn default() -> Self {
        Self {
            id: 0,
            direction: "in".to_string(),
            msg_type: "text".to_string(),
            content: String::new(),
            timestamp: 0,
            last_copied_at: None,
            sender_id: String::new(),
            sender_name: String::new(),
            file_path: None,
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(default)]
pub struct DeviceInfo {
    pub id: String,
    pub name: String,
    pub last_seen: i64,
}

impl Default for DeviceInfo {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            last_seen: 0,
        }
    }
}

#[derive(Deserialize)]
#[serde(default)]
pub struct ReceiveText {
    pub content: String,
    pub sender_id: String,
    pub sender_name: String,
}

impl Default for ReceiveText {
    fn default() -> Self {
        Self {
            content: String::new(),
            sender_id: String::new(),
            sender_name: String::new(),
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
#[serde(default)]
pub struct ChunkMetadata {
    pub upload_id: String,
    pub chunk_index: usize,
    pub total_chunks: usize,
    pub file_name: String,
    pub sender_id: String,
    pub sender_name: String,
    pub total_size: u64,
    pub content_type: Option<String>,
    pub chunk_sha256: Option<String>,
    pub file_sha256: Option<String>,
}

impl Default for ChunkMetadata {
    fn default() -> Self {
        Self {
            upload_id: String::new(),
            chunk_index: 0,
            total_chunks: 1,
            file_name: String::new(),
            sender_id: String::new(),
            sender_name: String::new(),
            total_size: 0,
            content_type: None,
            chunk_sha256: None,
            file_sha256: None,
        }
    }
}

/// Runtime configuration, entirely environment driven so the container is
/// configurable from an iStoreOS compose file.
#[derive(Clone)]
pub struct Config {
    pub bind: String,
    pub port: u16,
    /// Where received files are written (bind-mount this to your USB disk).
    pub data_dir: PathBuf,
    /// Append-only chat history (JSON Lines).
    pub history_file: PathBuf,
    /// How many messages to keep in memory and on disk.
    pub max_history: usize,
    /// One of: sticky-note | paper | mica | acrylic
    pub theme: String,
    /// One of: system | light | dark
    pub color_mode: String,
    /// Header title shown in the web UI.
    pub title: String,
    /// Logo as a URL or data URI.
    pub logo: String,
    /// Broadcast join/leave notices to connected clients.
    pub join_notice: bool,
    pub max_file_bytes: u64,
    pub max_storage_bytes: u64,
    pub file_ttl_secs: u64,
    pub message_idle_ttl_secs: u64,
    pub upload_ttl_secs: u64,
    pub cleanup_interval_secs: u64,
    pub lan_networks: std::sync::Arc<crate::network::Policy>,
}

pub struct UploadSession {
    pub meta: ChunkMetadata,
    pub path: PathBuf,
    pub chunks: Vec<(usize, String)>,
    pub bytes: u64,
    pub updated: i64,
    pub completed: bool,
}

/// Everything guarded by one mutex: this is a low-traffic LAN relay.
pub struct Inner {
    pub messages: Vec<Message>,
    pub next_id: u64,
    pub devices: HashMap<String, DeviceInfo>,
    /// download token -> absolute file path. Tokens are a hash of the path, so
    /// links keep working across restarts.
    pub tokens: HashMap<String, String>,
    /// upload_id -> temp file path for chunked uploads.
    pub uploads: HashMap<String, UploadSession>,
    pub managed_files: HashMap<String, i64>,
}

pub struct AppState {
    pub cfg: Config,
    pub inner: Mutex<Inner>,
    pub ws_tx: broadcast::Sender<String>,
    pub file_ops: tokio::sync::Mutex<()>,
    pub history_ops: Mutex<()>,
}

impl AppState {
    /// Poison-proof lock helper (`panic = "abort"` makes poisoning moot, but
    /// this keeps the handlers free of `unwrap()` noise).
    pub fn lock(&self) -> MutexGuard<'_, Inner> {
        self.inner.lock().unwrap_or_else(|e| e.into_inner())
    }

    pub fn next_message_id(&self) -> u64 {
        let mut inner = self.lock();
        inner.next_id += 1;
        inner.next_id
    }

    /// Register a file for download and return its stable token.
    pub fn register_file(&self, path: &str) -> String {
        let token = crate::store::token_for_path(path);
        self.lock().tokens.insert(token.clone(), path.to_string());
        token
    }
}

/// WebSocket frames the client may send.
#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum WsIncoming {
    #[serde(rename = "identity")]
    Identity {
        #[serde(default)]
        device_id: String,
        #[serde(default)]
        device_name: String,
    },
}
