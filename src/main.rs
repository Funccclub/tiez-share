//! TieZ Share - a standalone LAN file-transfer / chat relay.
//!
//! This is the "局域网传输" service of jimuzhe/tiez-clipboard (GPL-3.0), lifted
//! out of the Tauri desktop app so it can run as a plain Linux/AArch64 server
//! (Docker container on an OpenWrt / iStoreOS router).
//!
//! Everything is environment driven; see README.md.

mod handlers;
mod models;
mod store;
mod web_ui;
mod network;
mod maintenance;

use axum::{
    extract::DefaultBodyLimit,
    routing::{delete, get, post},
    Router,
};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;

use models::{AppState, Config, Inner};
use store::{load_history, token_for_path};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Built-in fallback logo (inline SVG: a speech bubble). Override with
/// `TIEZ_LOGO` to use your own image URL or data URI.
const DEFAULT_LOGO: &str = "data:image/svg+xml,%3Csvg%20xmlns='http://www.w3.org/2000/svg'%20viewBox='0%200%2064%2064'%3E%3Crect%20width='64'%20height='64'%20rx='14'%20fill='%23487bdb'/%3E%3Cpath%20d='M18%2022h28a4%204%200%200%201%204%204v14a4%204%200%200%201-4%204H32l-10%208v-8h-4a4%204%200%200%201-4-4V26a4%204%200%200%201%204-4z'%20fill='%23ffffff'/%3E%3C/svg%3E";

const THEMES: [&str; 5] = ["sticky-note", "paper", "mica", "acrylic", "liquid-glass"];
const COLOR_MODES: [&str; 3] = ["system", "light", "dark"];

fn env_str(key: &str, default: &str) -> String {
    std::env::var(key)
        .ok()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| default.to_string())
}

fn env_parsed<T: std::str::FromStr>(key: &str, default: T) -> T {
    std::env::var(key)
        .ok()
        .and_then(|v| v.trim().parse::<T>().ok())
        .unwrap_or(default)
}

fn env_bool(key: &str, default: bool) -> bool {
    match std::env::var(key).ok().map(|v| v.trim().to_lowercase()) {
        Some(v) => matches!(v.as_str(), "1" | "true" | "yes" | "on"),
        None => default,
    }
}

fn load_config() -> Config {
    let data_dir = PathBuf::from(env_str("TIEZ_DATA_DIR", "/data"));
    let history_file = match std::env::var("TIEZ_HISTORY_FILE") {
        Ok(v) if !v.trim().is_empty() => PathBuf::from(v.trim()),
        _ => data_dir.join("history.jsonl"),
    };

    let mut theme = env_str("TIEZ_THEME", "mica");
    if !THEMES.contains(&theme.as_str()) {
        eprintln!(
            "[tiez-share] unknown TIEZ_THEME '{}', falling back to 'mica' (valid: {})",
            theme,
            THEMES.join(", ")
        );
        theme = "mica".to_string();
    }

    let mut color_mode = env_str("TIEZ_COLOR_MODE", "system");
    if !COLOR_MODES.contains(&color_mode.as_str()) {
        eprintln!(
            "[tiez-share] unknown TIEZ_COLOR_MODE '{}', falling back to 'system' (valid: {})",
            color_mode,
            COLOR_MODES.join(", ")
        );
        color_mode = "system".to_string();
    }

    Config {
        bind: env_str("TIEZ_BIND", "0.0.0.0"),
        port: env_parsed("TIEZ_PORT", 12345u16),
        data_dir,
        history_file,
        max_history: env_parsed("TIEZ_MAX_HISTORY", 1000usize).max(1),
        theme,
        color_mode,
        title: env_str("TIEZ_TITLE", "局域网聊天室"),
        logo: env_str("TIEZ_LOGO", DEFAULT_LOGO),
        join_notice: env_bool("TIEZ_JOIN_NOTICE", false),
        max_file_bytes: env_parsed("TIEZ_MAX_FILE_BYTES", 2 * 1024 * 1024 * 1024u64),
        max_storage_bytes: env_parsed("TIEZ_MAX_STORAGE_BYTES", 10 * 1024 * 1024 * 1024u64),
        file_ttl_secs: env_parsed("TIEZ_FILE_TTL_SECS", 7 * 86400u64),
        message_idle_ttl_secs: env_parsed("TIEZ_MESSAGE_IDLE_TTL_SECS", 86400u64),
        upload_ttl_secs: env_parsed("TIEZ_UPLOAD_TTL_SECS", 3600u64).max(1),
        cleanup_interval_secs: env_parsed("TIEZ_CLEANUP_INTERVAL_SECS", 60u64).max(1),
        lan_networks: Arc::new(network::Policy::load().unwrap_or_else(|e| {
            eprintln!("[tiez-share] invalid LAN configuration: {e}");
            std::process::exit(1)
        })),
    }
}

#[tokio::main]
async fn main() {
    let cfg = load_config();

    if let Err(e) = std::fs::create_dir_all(&cfg.data_dir) {
        eprintln!(
            "[tiez-share] FATAL: cannot create data dir {:?}: {}",
            cfg.data_dir, e
        );
        std::process::exit(1);
    }

    // Fail fast on the classic deployment mistake: a bind mount that is
    // read-only, or owned by a uid this process cannot write to.
    let probe = cfg.data_dir.join(".tiez-share-write-test");
    match std::fs::write(&probe, b"ok") {
        Ok(_) => {
            let _ = std::fs::remove_file(&probe);
        }
        Err(e) => {
            eprintln!(
                "[tiez-share] FATAL: data dir {:?} is not writable: {}",
                cfg.data_dir, e
            );
            eprintln!("[tiez-share] hint: check the bind mount and its ownership/permissions");
            std::process::exit(1);
        }
    }

    let history = load_history(&cfg.history_file, cfg.max_history);
    // Avoid reusing ids after idle cleanup empties the persisted history.
    let next_id = history.iter().map(|m| m.id).max().unwrap_or(0)
        .max(store::now_millis().max(0) as u64);
    let history_len = history.len();

    // Rebuild download tokens so links in restored history keep resolving.
    let mut tokens: HashMap<String, String> = HashMap::new();
    for m in &history {
        if let Some(path) = m.file_path.as_ref() {
            if std::path::Path::new(path).is_file() {
                tokens.insert(token_for_path(path), path.clone());
            }
        }
    }

    let (ws_tx, _) = broadcast::channel::<String>(512);
    let state = Arc::new(AppState {
        cfg: cfg.clone(),
        inner: Mutex::new(Inner {
            messages: history,
            next_id,
            devices: HashMap::new(),
            tokens,
            uploads: HashMap::new(),
            managed_files: maintenance::load_index(&cfg.data_dir),
        }),
        ws_tx,
        file_ops: tokio::sync::Mutex::new(()),
        history_ops: Mutex::new(()),
    });
    println!("[tiez-share] permitted LAN networks: {:?} (plus loopback)", cfg.lan_networks);
    maintenance::cleanup_messages(&state);
    tokio::spawn(maintenance::run(state.clone()));

    let app = Router::new()
        .route("/", get(handlers::index))
        .route("/healthz", get(handlers::healthz))
        .route("/ws", get(handlers::ws_handler))
        .route("/poll", get(handlers::poll_messages))
        .route("/messages/{id}/copied", post(handlers::message_copied))
        .route("/upload", post(handlers::upload))
        .route("/upload_chunk", post(handlers::upload_chunk).layer(DefaultBodyLimit::max(2 * 1024 * 1024)))
        .route("/upload-chunk", post(handlers::upload_chunk).layer(DefaultBodyLimit::max(2 * 1024 * 1024)))
        .route("/send_text", post(handlers::handle_text))
        .route("/send-text", post(handlers::handle_text))
        .route("/download/{token}", get(handlers::handle_file_download))
        .route("/files", get(handlers::list_files))
        .route("/files/{token}", delete(handlers::delete_file))
        .with_state(state.clone())
        .layer(DefaultBodyLimit::disable())
        .layer(axum::middleware::from_fn_with_state(state.clone(), network::lan_only));

    let addr: SocketAddr = match cfg.bind.parse::<std::net::IpAddr>().map(|ip| SocketAddr::new(ip, cfg.port)) {
        Ok(addr) => addr,
        Err(_) => {
            eprintln!(
                "[tiez-share] FATAL: invalid bind address '{}:{}'",
                cfg.bind, cfg.port
            );
            std::process::exit(1);
        }
    };

    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(listener) => listener,
        Err(e) => {
            eprintln!("[tiez-share] FATAL: cannot bind {}: {}", addr, e);
            eprintln!("[tiez-share] hint: is another service (or camera/rtsp) already using this port?");
            std::process::exit(1);
        }
    };

    println!("======================================================");
    println!(" TieZ Share v{} - LAN transfer + chat relay", VERSION);
    println!("------------------------------------------------------");
    println!(" 打开方式   http://<路由器LAN地址>:{}", cfg.port);
    println!(" 监听地址   {}", addr);
    println!(" 存储目录   {}", cfg.data_dir.display());
    println!(
        " 历史记录   {} ({} 条)",
        cfg.history_file.display(),
        history_len
    );
    println!(" 界面       theme={} title={}", cfg.theme, cfg.title);
    println!("======================================================");

    if let Err(e) = axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>())
        .with_graceful_shutdown(shutdown_signal())
        .await
    {
        eprintln!("[tiez-share] server error: {}", e);
    }
    println!("[tiez-share] stopped");
}

async fn shutdown_signal() {
    let ctrl_c = async {
        let _ = tokio::signal::ctrl_c().await;
    };

    #[cfg(unix)]
    let terminate = async {
        use tokio::signal::unix::{signal, SignalKind};
        match signal(SignalKind::terminate()) {
            Ok(mut sig) => {
                sig.recv().await;
            }
            Err(_) => std::future::pending::<()>().await,
        }
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
    println!("[tiez-share] shutdown signal received");
}
