//! Network side of the ED2K window: protocol obfuscation, the Kad session and UPnP port mapping.
use super::{save_file, Ed2kState};
use nibiru_xtreme::{
    ed2k::{kad_session::KadSession, obfuscation::Obfuscation},
    upnp::{Mapping, Proto, Upnp},
};
use serde::{Deserialize, Serialize};
use std::{
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::Duration,
};
use tauri::{Manager, State};
use tokio::sync::RwLock;

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(default)]
pub struct Prefs {
    pub obfuscation: Obfuscation,
    /// Ask the router (UPnP) to forward the incoming ports while connected.
    pub upnp: bool,
}

impl Default for Prefs {
    fn default() -> Self {
        Self {
            obfuscation: Obfuscation::Prefer,
            upnp: true,
        }
    }
}

pub struct Network {
    prefs: Mutex<Prefs>,
    prefs_path: PathBuf,
    pub kad: RwLock<Option<Arc<KadSession>>>,
    pub upnp: Arc<Upnp>,
}

impl Network {
    pub fn new(root: &Path) -> Self {
        let prefs_path = root.join("prefs.json");
        let prefs = std::fs::read(&prefs_path)
            .ok()
            .filter(|bytes| bytes.len() <= 64 << 10)
            .and_then(|bytes| serde_json::from_slice(&bytes).ok())
            .unwrap_or_default();
        Self {
            prefs: Mutex::new(prefs),
            prefs_path,
            kad: RwLock::new(None),
            upnp: Upnp::new("Apocalipse ED2K"),
        }
    }

    pub fn prefs(&self) -> Prefs {
        self.prefs.lock().map(|prefs| *prefs).unwrap_or_default()
    }

    fn save(&self, prefs: Prefs) -> Result<(), String> {
        let bytes = serde_json::to_vec_pretty(&prefs).map_err(|e| e.to_string())?;
        save_file(&self.prefs_path, &bytes)?;
        *self.prefs.lock().map_err(|e| e.to_string())? = prefs;
        Ok(())
    }

    pub async fn kad_session(&self) -> Option<Arc<KadSession>> {
        self.kad
            .read()
            .await
            .as_ref()
            .filter(|session| session.status().running)
            .cloned()
    }
}

/// The ports to forward: the incoming TCP port while a server or Kad is in use, the Kad UDP port
/// while Kad runs. Nothing when the option is off or nothing is connected.
pub fn wanted_mappings(
    prefs: &Prefs,
    server_connected: bool,
    tcp_port: Option<u16>,
    kad_udp: Option<u16>,
) -> Vec<Mapping> {
    if !prefs.upnp {
        return Vec::new();
    }
    let mut wanted = Vec::new();
    if let Some(port) =
        tcp_port.filter(|port| *port != 0 && (server_connected || kad_udp.is_some()))
    {
        wanted.push(Mapping {
            proto: Proto::Tcp,
            port,
        });
    }
    if let Some(port) = kad_udp.filter(|port| *port != 0) {
        wanted.push(Mapping {
            proto: Proto::Udp,
            port,
        });
    }
    wanted
}

/// Makes the router's forwarding match what is in use right now. Looking for the router can take
/// seconds, so callers run this in the background.
pub async fn sync_upnp(state: &Ed2kState) {
    let prefs = state.net.prefs();
    let server_connected = state
        .server
        .read()
        .await
        .as_ref()
        .is_some_and(|connection| connection.is_connected());
    let tcp_port = state
        .incoming
        .lock()
        .await
        .as_ref()
        .map(|listener| listener.status())
        .filter(|status| status.running)
        .map(|status| status.tcp_port);
    let kad_udp = state
        .net
        .kad_session()
        .await
        .map(|session| session.status().udp_port);
    state
        .net
        .upnp
        .apply(wanted_mappings(&prefs, server_connected, tcp_port, kad_udp))
        .await;
}

pub fn spawn_sync_upnp(app: tauri::AppHandle) {
    tauri::async_runtime::spawn(async move {
        let state = app.state::<Ed2kState>();
        sync_upnp(&state).await;
    });
}

/// Removes the router mappings when the application exits.
pub fn release_on_exit(app: &tauri::AppHandle) {
    let state = app.state::<Ed2kState>();
    let upnp = state.net.upnp.clone();
    tauri::async_runtime::block_on(async move {
        let _ = tokio::time::timeout(Duration::from_secs(5), upnp.release()).await;
    });
}

pub async fn apply_obfuscation(state: &Ed2kState, mode: Obfuscation) {
    if let Some(listener) = state.incoming.lock().await.as_ref() {
        listener.set_obfuscation(mode);
    }
    if let Some(session) = state.net.kad.read().await.as_ref() {
        session.set_obfuscation(mode != Obfuscation::Disabled);
    }
}

pub async fn overview(state: &Ed2kState) -> serde_json::Value {
    let prefs = state.net.prefs();
    let kad = state
        .net
        .kad_session()
        .await
        .and_then(|session| serde_json::to_value(session.status()).ok());
    serde_json::json!({
        "obfuscation": prefs.obfuscation,
        "upnp_enabled": prefs.upnp,
        "kad": kad,
        "upnp": state.net.upnp.status(),
    })
}

#[tauri::command]
pub async fn ed2k_set_prefs(
    app: tauri::AppHandle,
    state: State<'_, Ed2kState>,
    obfuscation: String,
    upnp: bool,
) -> Result<(), String> {
    state.ready()?;
    let obfuscation: Obfuscation = obfuscation
        .parse()
        .map_err(|_| "invalid_obfuscation".to_owned())?;
    state.net.save(Prefs { obfuscation, upnp })?;
    apply_obfuscation(&state, obfuscation).await;
    spawn_sync_upnp(app);
    Ok(())
}

/// Starts the Kad session on the real incoming TCP port, seeded from the node list.
pub async fn start_kad(state: &Ed2kState) -> Result<(), String> {
    if state.net.kad_session().await.is_some() {
        return Ok(());
    }
    let nodes = state
        .library
        .lock()
        .map_err(|e| e.to_string())?
        .nodes
        .clone();
    if nodes.is_empty() {
        return Err("empty_node_list".into());
    }
    let port = state.incoming_port().await?;
    let session = KadSession::start(
        state.user_hash,
        port,
        state.root.join("kad-contacts.json"),
        nodes,
        false,
    )
    .await?;
    session.set_obfuscation(state.net.prefs().obfuscation != Obfuscation::Disabled);
    *state.net.kad.write().await = Some(session);
    Ok(())
}

pub async fn stop_kad(state: &Ed2kState) {
    if let Some(session) = state.net.kad.write().await.take() {
        session.stop();
    }
}

#[tauri::command]
pub async fn ed2k_kad_connect(
    app: tauri::AppHandle,
    state: State<'_, Ed2kState>,
) -> Result<(), String> {
    state.ready()?;
    start_kad(&state).await?;
    spawn_sync_upnp(app);
    Ok(())
}

#[tauri::command]
pub async fn ed2k_kad_disconnect(
    app: tauri::AppHandle,
    state: State<'_, Ed2kState>,
) -> Result<(), String> {
    state.ready()?;
    stop_kad(&state).await;
    spawn_sync_upnp(app);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_the_ports_in_use_are_forwarded_and_only_when_enabled() {
        let on = Prefs::default();
        let off = Prefs { upnp: false, ..on };
        let tcp = |port| Mapping {
            proto: Proto::Tcp,
            port,
        };
        let udp = |port| Mapping {
            proto: Proto::Udp,
            port,
        };
        assert!(wanted_mappings(&off, true, Some(4662), Some(4672)).is_empty());
        assert!(
            wanted_mappings(&on, false, Some(4662), None).is_empty(),
            "nothing connected"
        );
        assert_eq!(
            wanted_mappings(&on, true, Some(4662), None),
            vec![tcp(4662)]
        );
        assert_eq!(
            wanted_mappings(&on, false, Some(4662), Some(4672)),
            vec![tcp(4662), udp(4672)],
            "Kad alone also needs the incoming TCP port"
        );
        assert_eq!(
            wanted_mappings(&on, true, None, Some(4672)),
            vec![udp(4672)]
        );
        assert!(wanted_mappings(&on, true, Some(0), Some(0)).is_empty());
    }

    #[test]
    fn preferences_default_to_preferred_obfuscation_with_upnp_and_survive_a_restart() {
        let root = tempfile::tempdir().unwrap();
        let first = Network::new(root.path());
        assert_eq!(first.prefs(), Prefs::default());
        assert_eq!(first.prefs().obfuscation, Obfuscation::Prefer);
        first
            .save(Prefs {
                obfuscation: Obfuscation::Require,
                upnp: false,
            })
            .unwrap();
        let second = Network::new(root.path());
        assert_eq!(second.prefs().obfuscation, Obfuscation::Require);
        assert!(!second.prefs().upnp);
        std::fs::write(root.path().join("prefs.json"), b"{ broken").unwrap();
        assert_eq!(
            Network::new(root.path()).prefs(),
            Prefs::default(),
            "a damaged file falls back"
        );
    }

    #[tokio::test]
    async fn kad_starts_on_the_incoming_port_and_stops_cleanly() {
        use nibiru_xtreme::ed2k::metadata::Node;
        let root = tempfile::tempdir().unwrap();
        let state = Ed2kState::new(root.path().join("ed2k-kad")).unwrap();
        // One loopback seed, so the test never contacts a public Kad node.
        state.library.lock().unwrap().nodes = vec![Node {
            id: [7; 16],
            host: std::net::Ipv4Addr::LOCALHOST,
            udp_port: 9,
            tcp_port: 9,
            version: 8,
        }];
        assert!(state.net.kad_session().await.is_none());
        start_kad(&state).await.unwrap();
        let session = state.net.kad_session().await.expect("running");
        let status = session.status();
        let tcp = state
            .incoming
            .lock()
            .await
            .as_ref()
            .unwrap()
            .status()
            .tcp_port;
        assert!(status.running && status.routing && status.udp_port != 0);
        assert_eq!(
            status.tcp_port, tcp,
            "Kad announces the listener's real TCP port"
        );
        // Asking again changes nothing, and the UPnP plan now includes both ports.
        start_kad(&state).await.unwrap();
        let prefs = state.net.prefs();
        let wanted = wanted_mappings(&prefs, false, Some(tcp), Some(status.udp_port));
        assert_eq!(wanted.len(), 2);
        stop_kad(&state).await;
        assert!(state.net.kad_session().await.is_none());
        assert!(wanted_mappings(&prefs, false, Some(tcp), None).is_empty());
    }

    #[tokio::test]
    async fn kad_needs_a_node_list() {
        let root = tempfile::tempdir().unwrap();
        let state = Ed2kState::new(root.path().join("ed2k-kad")).unwrap();
        state.library.lock().unwrap().nodes.clear();
        assert_eq!(start_kad(&state).await.unwrap_err(), "empty_node_list");
    }
}
