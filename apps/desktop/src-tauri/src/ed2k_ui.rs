//! Native Nibiru ED2K integration. No aMule executable or daemon is used.
pub mod downloads;
pub mod shares;
use nibiru_xtreme::ed2k::{
    self,
    metadata::{Node, Server},
    BootstrapResult, ServerConnection,
};
use serde::{Deserialize, Serialize};
use std::{
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicU64, Ordering},
        Mutex,
    },
};
use tauri::{Manager, State, WebviewUrl, WebviewWindowBuilder};
use tokio::sync::{Mutex as AsyncMutex, RwLock};

#[derive(Clone, Serialize, Deserialize)]
struct Library {
    servers: Vec<Server>,
    nodes: Vec<Node>,
}
pub struct Ed2kState {
    root: PathBuf,
    shares: std::sync::Arc<shares::Shares>,
    downloads: downloads::Downloads,
    startup_error: Option<String>,
    library: Mutex<Library>,
    server: RwLock<Option<ServerConnection>>,
    incoming: AsyncMutex<Option<ed2k::IncomingListener>>,
    generation: AtomicU64,
    kad: RwLock<Option<BootstrapResult>>,
    user_hash: [u8; 16],
}
impl Ed2kState {
    pub fn unavailable(root: PathBuf, error: String) -> Self {
        Self {
            downloads: downloads::Downloads::empty(root.join("downloads.json")),
            shares: shares::Shares::load(&root),
            root,
            startup_error: Some(error),
            library: Mutex::new(Library {
                servers: Vec::new(),
                nodes: Vec::new(),
            }),
            server: RwLock::new(None),
            incoming: AsyncMutex::new(None),
            generation: AtomicU64::new(0),
            kad: RwLock::new(None),
            user_hash: [0; 16],
        }
    }
    async fn incoming_port(&self) -> Result<u16, String> {
        let mut incoming = self.incoming.lock().await;
        if incoming
            .as_ref()
            .is_none_or(|listener| !listener.status().running)
        {
            *incoming = Some(
                ed2k::IncomingListener::bind_with_uploads(
                    self.user_hash,
                    4662,
                    self.shares.registry(),
                    self.shares.uploads(),
                )
                .await
                .map_err(|e| e.to_string())?,
            );
        }
        Ok(incoming
            .as_ref()
            .ok_or("ed2k_incoming_unavailable")?
            .status()
            .tcp_port)
    }
    /// Offers the shared files to the connected server. ED2K has no "withdraw" packet, so files
    /// that were unshared disappear from the server's index when the connection is renewed.
    pub async fn publish_shares(&self) -> Result<usize, String> {
        let Some(connection) = self
            .server
            .read()
            .await
            .as_ref()
            .filter(|c| c.is_connected())
            .cloned()
        else {
            return Ok(0);
        };
        let sent = connection
            .offer_files(&self.shares.registry().offered())
            .await
            .map_err(|e| e.to_string())?;
        self.shares.note_published(sent);
        Ok(sent)
    }
    fn ready(&self) -> Result<(), String> {
        match &self.startup_error {
            Some(error) => Err(error.clone()),
            None => Ok(()),
        }
    }

    pub fn new(root: PathBuf) -> Result<Self, String> {
        std::fs::create_dir_all(&root).map_err(|e| e.to_string())?;
        let mut library: Library =
            serde_json::from_str(include_str!("../../../../assets/ed2k-seeds.json"))
                .map_err(|e| e.to_string())?;
        if let Ok(bytes) = read_metadata(&root.join("server.met")) {
            if let Ok(servers) = ed2k::metadata::parse_servers(&bytes) {
                library.servers = servers;
            }
        }
        if let Ok(bytes) = read_metadata(&root.join("nodes.dat")) {
            if let Ok(nodes) = ed2k::metadata::parse_nodes(&bytes) {
                library.nodes = nodes;
            }
        }
        let identity = root.join("identity.bin");
        let user_hash = match std::fs::read(&identity) {
            Ok(bytes) => bytes
                .try_into()
                .map_err(|_| "invalid_ed2k_identity".to_owned())?,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                let hash = *uuid::Uuid::new_v4().as_bytes();
                save_file(&identity, &hash)?;
                hash
            }
            Err(e) => return Err(e.to_string()),
        };
        Ok(Self {
            downloads: downloads::Downloads::load(root.join("downloads.json"))?,
            shares: shares::Shares::load(&root),
            root,
            startup_error: None,
            library: Mutex::new(library),
            server: RwLock::new(None),
            incoming: AsyncMutex::new(None),
            generation: AtomicU64::new(0),
            kad: RwLock::new(None),
            user_hash,
        })
    }
}
fn read_metadata(path: &Path) -> Result<Vec<u8>, std::io::Error> {
    use std::io::Read;
    let mut bytes = Vec::new();
    std::fs::File::open(path)?
        .take((ed2k::metadata::MAX_METADATA + 1) as u64)
        .read_to_end(&mut bytes)?;
    Ok(bytes)
}
fn save_file(path: &Path, bytes: &[u8]) -> Result<(), String> {
    use std::io::Write;
    let mut temp = tempfile::NamedTempFile::new_in(path.parent().ok_or("invalid_config_path")?)
        .map_err(|e| e.to_string())?;
    temp.write_all(bytes)
        .and_then(|_| temp.as_file().sync_all())
        .map_err(|e| e.to_string())?;
    temp.persist(path).map_err(|e| e.to_string())?;
    Ok(())
}
#[tauri::command]
pub async fn open_ed2k_window(app: tauri::AppHandle) -> Result<(), String> {
    let theme = if super::get_dark_mode(app.state())? {
        tauri::Theme::Dark
    } else {
        tauri::Theme::Light
    };
    if let Some(window) = app.get_webview_window("apocalipse-ed2k-kad") {
        window.set_theme(Some(theme)).map_err(|e| e.to_string())?;
        window.show().map_err(|e| e.to_string())?;
        let _ = window.unminimize();
        window.maximize().map_err(|e| e.to_string())?;
        window.set_focus().map_err(|e| e.to_string())?;
        return Ok(());
    }
    WebviewWindowBuilder::new(
        &app,
        "apocalipse-ed2k-kad",
        WebviewUrl::App("ed2k.html".into()),
    )
    .title("Apocalipse ED2K/Kad")
    .theme(Some(theme))
    .inner_size(1280.0, 820.0)
    .min_inner_size(960.0, 640.0)
    .resizable(true)
    .maximized(true)
    .decorations(true)
    .build()
    .map_err(|e| e.to_string())?;
    Ok(())
}
#[tauri::command]
pub async fn ed2k_overview(state: State<'_, Ed2kState>) -> Result<serde_json::Value, String> {
    state.ready()?;
    let incoming = state
        .incoming
        .lock()
        .await
        .as_ref()
        .map(|listener| listener.status());
    let server = state.server.read().await;
    let active = server.as_ref().filter(|c| c.is_connected());
    let kad = state.kad.read().await;
    let library = state.library.lock().map_err(|e| e.to_string())?;
    Ok(serde_json::json!({
        "servers":library.servers,"node_count":library.nodes.len(),"downloads":state.downloads.list()?,"incoming":incoming,
        "server":active.map(|c|&c.server),"client_id":active.map(|c|c.client_id),"low_id":active.is_some_and(|c|c.is_low_id()),
        "kad_bootstrap":*kad,"server_list_url":ed2k::SERVER_LIST_URL,"node_list_url":ed2k::NODE_LIST_URL,
        "capabilities":{"server_login":true,"server_search":true,"server_sources":true,"kad_bootstrap":true,"kad_routing":false,"peer_download":true,"peer_download_scope":"verified_parts_direct_and_server_callback","server_callback":true,"sharing":true,"upload":true,"upload_queue":true,"publishing":true,"messages":false}
    }))
}
fn install(state: &Ed2kState, kind: &str, bytes: &[u8]) -> Result<usize, String> {
    match kind {
        "servers" => {
            let servers = ed2k::metadata::parse_servers(bytes).map_err(|e| e.to_string())?;
            if servers.is_empty() {
                return Err("empty_server_list".into());
            }
            let count = servers.len();
            save_file(&state.root.join("server.met"), bytes)?;
            state.library.lock().map_err(|e| e.to_string())?.servers = servers;
            Ok(count)
        }
        "nodes" => {
            let nodes = ed2k::metadata::parse_nodes(bytes).map_err(|e| e.to_string())?;
            if nodes.is_empty() {
                return Err("empty_node_list".into());
            }
            let count = nodes.len();
            let mut library = state.library.lock().map_err(|e| e.to_string())?;
            save_file(&state.root.join("nodes.dat"), bytes)?;
            library.nodes = nodes;
            Ok(count)
        }
        _ => Err("invalid_list_kind".into()),
    }
}
#[tauri::command]
pub async fn ed2k_update_list(
    state: State<'_, Ed2kState>,
    kind: String,
    url: String,
) -> Result<usize, String> {
    state.ready()?;
    if kind != "servers" && kind != "nodes" {
        return Err("invalid_list_kind".into());
    }
    let bytes = ed2k::fetch_metadata(&url)
        .await
        .map_err(|e| e.to_string())?;
    install(&state, &kind, &bytes)
}
#[tauri::command]
pub async fn ed2k_import_list(
    state: State<'_, Ed2kState>,
    kind: String,
) -> Result<Option<usize>, String> {
    state.ready()?;
    let extension = match kind.as_str() {
        "servers" => "met",
        "nodes" => "dat",
        _ => return Err("invalid_list_kind".into()),
    };
    let bytes = tauri::async_runtime::spawn_blocking(move || -> Result<Option<Vec<u8>>, String> {
        use std::io::Read;
        let Some(path) = rfd::FileDialog::new()
            .add_filter(extension, &[extension])
            .pick_file()
        else {
            return Ok(None);
        };
        let mut bytes = Vec::new();
        std::fs::File::open(path)
            .map_err(|e| e.to_string())?
            .take((ed2k::metadata::MAX_METADATA + 1) as u64)
            .read_to_end(&mut bytes)
            .map_err(|e| e.to_string())?;
        Ok(Some(bytes))
    })
    .await
    .map_err(|e| e.to_string())??;
    bytes
        .map(|bytes| install(&state, &kind, &bytes))
        .transpose()
}
#[tauri::command]
pub async fn ed2k_connect(
    app: tauri::AppHandle,
    state: State<'_, Ed2kState>,
    server: Option<Server>,
) -> Result<(), String> {
    state.ready()?;
    let revision = state.generation.fetch_add(1, Ordering::AcqRel) + 1;
    if let Some(connection) = state.server.write().await.take() {
        connection.disconnect();
    }
    let candidates = match server {
        Some(s) => vec![s],
        None => state
            .library
            .lock()
            .map_err(|e| e.to_string())?
            .servers
            .iter()
            .take(4)
            .cloned()
            .collect(),
    };
    let mut last = "empty_server_list".to_owned();
    for server in candidates {
        if state.generation.load(Ordering::Acquire) != revision {
            return Err("connection_cancelled".into());
        }
        match ServerConnection::connect(
            server,
            state.user_hash,
            "Apocalipse Nibiru",
            state.incoming_port().await?,
        )
        .await
        {
            Ok(connection) => {
                let mut active = state.server.write().await;
                if state.generation.load(Ordering::Acquire) != revision {
                    connection.disconnect();
                    return Err("connection_cancelled".into());
                }
                if let Some(listener) = state.incoming.lock().await.as_ref() {
                    listener.set_client_id(connection.client_id);
                }
                *active = Some(connection);
                drop(active);
                state.shares.ensure_started(&app);
                spawn_publisher(app.clone(), revision);
                return Ok(());
            }
            Err(error) => last = error.to_string(),
        }
    }
    Err(last)
}
#[tauri::command]
pub async fn ed2k_disconnect(state: State<'_, Ed2kState>) -> Result<(), String> {
    state.ready()?;
    state.generation.fetch_add(1, Ordering::AcqRel);
    if let Some(connection) = state.server.write().await.take() {
        connection.disconnect();
    }
    if let Some(listener) = state.incoming.lock().await.as_ref() {
        listener.set_client_id(0);
    }
    state.shares.clear_published();
    Ok(())
}

/// Offers the shared files now and then again every [`shares::REPUBLISH_EVERY`], until the
/// connection that started this loop is replaced or closed.
fn spawn_publisher(app: tauri::AppHandle, revision: u64) {
    tauri::async_runtime::spawn(async move {
        loop {
            {
                let state = app.state::<Ed2kState>();
                if state.generation.load(Ordering::Acquire) != revision {
                    break;
                }
                let _ = state.publish_shares().await;
            }
            tokio::time::sleep(shares::REPUBLISH_EVERY).await;
        }
    });
}
#[tauri::command]
pub async fn ed2k_search(
    state: State<'_, Ed2kState>,
    query: String,
) -> Result<Vec<ed2k::SearchResult>, String> {
    state.ready()?;
    let connection = state
        .server
        .read()
        .await
        .as_ref()
        .cloned()
        .ok_or("ed2k_not_connected")?;
    connection.search(&query).await.map_err(|e| e.to_string())
}
#[tauri::command]
pub async fn ed2k_find_sources(
    state: State<'_, Ed2kState>,
    hash: String,
) -> Result<Vec<ed2k::FileSource>, String> {
    state.ready()?;
    let connection = state
        .server
        .read()
        .await
        .as_ref()
        .cloned()
        .ok_or("ed2k_not_connected")?;
    connection.sources(&hash).await.map_err(|e| e.to_string())
}
#[tauri::command]
pub async fn ed2k_bootstrap(state: State<'_, Ed2kState>) -> Result<BootstrapResult, String> {
    state.ready()?;
    let nodes = state
        .library
        .lock()
        .map_err(|e| e.to_string())?
        .nodes
        .clone();
    // Rotate candidates between installations rather than all probing the first node.
    let offset = usize::from(state.user_hash[0]) % nodes.len().max(1);
    let mut last = "empty_node_list".to_owned();
    for node in nodes.iter().cycle().skip(offset).take(nodes.len().min(6)) {
        match ed2k::bootstrap(node).await {
            Ok(result) => {
                *state.kad.write().await = Some(result.clone());
                return Ok(result);
            }
            Err(error) => last = error.to_string(),
        }
    }
    Err(last)
}

#[tauri::command]
pub fn ed2k_copy_link(app: tauri::AppHandle, link: String) -> Result<(), String> {
    use tauri_plugin_clipboard_manager::ClipboardExt;
    if !link.starts_with("ed2k://|file|") || link.len() > 4096 || link.chars().any(char::is_control)
    {
        return Err("invalid_ed2k_link".into());
    }
    app.clipboard().write_text(link).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn initial_lists_and_identity_are_available_without_fetching() {
        let root = tempfile::tempdir().unwrap();
        let first = Ed2kState::new(root.path().to_owned()).unwrap();
        assert!(!first.library.lock().unwrap().servers.is_empty());
        assert!(!first.library.lock().unwrap().nodes.is_empty());
        let second = Ed2kState::new(root.path().to_owned()).unwrap();
        assert_eq!(first.user_hash, second.user_hash);
    }
    #[test]
    fn invalid_metadata_preserves_the_last_good_list() {
        let root = tempfile::tempdir().unwrap();
        let state = Ed2kState::new(root.path().to_owned()).unwrap();
        let mut met = vec![0xe0];
        met.extend(1u32.to_le_bytes());
        met.extend([192, 0, 2, 1]);
        met.extend(4661u16.to_le_bytes());
        met.extend(0u32.to_le_bytes());
        assert_eq!(install(&state, "servers", &met).unwrap(), 1);
        assert!(install(&state, "servers", b"not server.met").is_err());
        assert_eq!(std::fs::read(root.path().join("server.met")).unwrap(), met);
        assert_eq!(state.library.lock().unwrap().servers.len(), 1);
    }
    #[test]
    fn feature_startup_failure_does_not_require_the_main_app_to_exit() {
        let state = Ed2kState::unavailable(PathBuf::new(), "invalid_ed2k_identity".into());
        assert_eq!(state.ready().unwrap_err(), "invalid_ed2k_identity");
    }
}
