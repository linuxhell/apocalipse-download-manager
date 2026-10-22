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

/// The ports to forward: the incoming TCP port while a server (connected or about to log in) or
/// Kad is in use, the Kad UDP port while Kad runs. Nothing when the option is off or nothing is in use.
pub fn wanted_mappings(
    prefs: &Prefs,
    server_in_use: bool,
    tcp_port: Option<u16>,
    kad_udp: Option<u16>,
) -> Vec<Mapping> {
    if !prefs.upnp {
        return Vec::new();
    }
    let mut wanted = Vec::new();
    if let Some(port) = tcp_port.filter(|port| *port != 0 && (server_in_use || kad_udp.is_some())) {
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

/// Forwards the incoming TCP port **before** logging in to an ED2K server.
///
/// The server tests our TCP port while we log in and answers with a Low ID when it cannot connect
/// back. Mapping the port only after the login (as a background task) meant the first connection was
/// always Low ID until the user disconnected and reconnected. This waits for the router (up to
/// `LOGIN_WAIT`), but not when a recent search already found no router, so a network without UPnP
/// does not delay every login.
pub async fn forward_before_login(state: &Ed2kState) {
    const LOGIN_WAIT: Duration = Duration::from_secs(12);
    let prefs = state.net.prefs();
    if !prefs.upnp {
        return;
    }
    let Ok(tcp_port) = state.incoming_port().await else {
        return;
    };
    let kad_udp = state
        .net
        .kad_session()
        .await
        .map(|session| session.status().udp_port);
    let wanted = wanted_mappings(&prefs, true, Some(tcp_port), kad_udp);
    let status = state.net.upnp.status();
    let no_router = status.state == "failed"
        && status
            .error
            .as_deref()
            .is_some_and(|error| error.starts_with("upnp_no_gateway"));
    if no_router {
        let upnp = state.net.upnp.clone();
        tauri::async_runtime::spawn(async move { upnp.apply(wanted).await });
        return;
    }
    let _ = tokio::time::timeout(LOGIN_WAIT, state.net.upnp.apply(wanted)).await;
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
    state.downloads.wake();
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

    /// A router that speaks the real protocol (SSDP discovery, device and service descriptions,
    /// SOAP), answering from loopback. It records the port mappings it was asked for.
    mod fake_router {
        use std::{
            net::SocketAddr,
            sync::{Arc, Mutex},
        };
        use tokio::{
            io::{AsyncReadExt, AsyncWriteExt},
            net::{TcpListener, UdpSocket},
            task::JoinHandle,
        };

        pub struct Router {
            pub ssdp: SocketAddr,
            /// (protocol, port) of every mapping currently on the router.
            pub mappings: Arc<Mutex<Vec<(String, u16)>>>,
            tasks: Vec<JoinHandle<()>>,
        }
        impl Drop for Router {
            fn drop(&mut self) {
                for task in &self.tasks {
                    task.abort();
                }
            }
        }
        const DEVICE: &str = r#"<?xml version="1.0"?><root xmlns="urn:schemas-upnp-org:device-1-0"><specVersion><major>1</major><minor>0</minor></specVersion><device><deviceType>urn:schemas-upnp-org:device:InternetGatewayDevice:1</deviceType><friendlyName>Fake</friendlyName><deviceList><device><deviceType>urn:schemas-upnp-org:device:WANDevice:1</deviceType><friendlyName>WAN</friendlyName><deviceList><device><deviceType>urn:schemas-upnp-org:device:WANConnectionDevice:1</deviceType><friendlyName>Conn</friendlyName><serviceList><service><serviceType>urn:schemas-upnp-org:service:WANIPConnection:1</serviceType><serviceId>urn:upnp-org:serviceId:WANIPConn1</serviceId><controlURL>/ctl</controlURL><eventSubURL>/evt</eventSubURL><SCPDURL>/scpd.xml</SCPDURL></service></serviceList></device></deviceList></device></deviceList></device></root>"#;
        fn arg(name: &str) -> String {
            format!("<argument><name>{name}</name><direction>in</direction></argument>")
        }
        fn scpd() -> String {
            let add: String = [
                "NewRemoteHost",
                "NewExternalPort",
                "NewProtocol",
                "NewInternalPort",
                "NewInternalClient",
                "NewEnabled",
                "NewPortMappingDescription",
                "NewLeaseDuration",
            ]
            .map(arg)
            .concat();
            let delete: String = ["NewRemoteHost", "NewExternalPort", "NewProtocol"]
                .map(arg)
                .concat();
            format!(
                r#"<?xml version="1.0"?><scpd xmlns="urn:schemas-upnp-org:service-1-0"><actionList><action><name>AddPortMapping</name><argumentList>{add}</argumentList></action><action><name>DeletePortMapping</name><argumentList>{delete}</argumentList></action><action><name>GetExternalIPAddress</name><argumentList><argument><name>NewExternalIPAddress</name><direction>out</direction></argument></argumentList></action></actionList></scpd>"#
            )
        }
        fn soap(body: &str) -> String {
            format!(
                r#"<?xml version="1.0"?><s:Envelope xmlns:s="http://schemas.xmlsoap.org/soap/envelope/" s:encodingStyle="http://schemas.xmlsoap.org/soap/encoding/"><s:Body>{body}</s:Body></s:Envelope>"#
            )
        }
        fn tag(body: &str, name: &str) -> String {
            let open = format!("<{name}>");
            let start = body.find(&open).map_or(0, |i| i + open.len());
            let end = body[start..].find('<').map_or(start, |i| start + i);
            body[start..end].to_owned()
        }
        pub async fn start() -> Router {
            let listener = TcpListener::bind(("127.0.0.1", 0)).await.unwrap();
            let http_port = listener.local_addr().unwrap().port();
            let mappings: Arc<Mutex<Vec<(String, u16)>>> = Arc::default();
            let book = mappings.clone();
            let server = tokio::spawn(async move {
                loop {
                    let Ok((mut stream, _)) = listener.accept().await else {
                        return;
                    };
                    let book = book.clone();
                    tokio::spawn(async move {
                        let mut buffer = Vec::new();
                        let mut chunk = [0u8; 4096];
                        let (head, mut body) = loop {
                            let Ok(read) = stream.read(&mut chunk).await else {
                                return;
                            };
                            if read == 0 {
                                return;
                            }
                            buffer.extend_from_slice(&chunk[..read]);
                            if let Some(end) = buffer.windows(4).position(|w| w == b"\r\n\r\n") {
                                break (
                                    String::from_utf8_lossy(&buffer[..end]).into_owned(),
                                    buffer[end + 4..].to_vec(),
                                );
                            }
                        };
                        let length = head
                            .lines()
                            .find_map(|l| {
                                l.to_ascii_lowercase()
                                    .strip_prefix("content-length:")
                                    .map(|v| v.trim().parse::<usize>().unwrap_or(0))
                            })
                            .unwrap_or(0);
                        while body.len() < length {
                            let Ok(read) = stream.read(&mut chunk).await else {
                                return;
                            };
                            if read == 0 {
                                break;
                            }
                            body.extend_from_slice(&chunk[..read]);
                        }
                        let body = String::from_utf8_lossy(&body).into_owned();
                        let first = head.lines().next().unwrap_or_default().to_owned();
                        let reply = if first.starts_with("GET /root.xml") {
                            DEVICE.to_owned()
                        } else if first.starts_with("GET /scpd.xml") {
                            scpd()
                        } else if head.contains("#AddPortMapping") {
                            let entry = (
                                tag(&body, "NewProtocol"),
                                tag(&body, "NewExternalPort").parse().unwrap_or(0),
                            );
                            let mut book = book.lock().unwrap();
                            if !book.contains(&entry) {
                                book.push(entry);
                            }
                            soap(
                                r#"<u:AddPortMappingResponse xmlns:u="urn:schemas-upnp-org:service:WANIPConnection:1"/>"#,
                            )
                        } else if head.contains("#DeletePortMapping") {
                            let protocol = tag(&body, "NewProtocol");
                            let port: u16 = tag(&body, "NewExternalPort").parse().unwrap_or(0);
                            book.lock()
                                .unwrap()
                                .retain(|e| *e != (protocol.clone(), port));
                            soap(
                                r#"<u:DeletePortMappingResponse xmlns:u="urn:schemas-upnp-org:service:WANIPConnection:1"/>"#,
                            )
                        } else if head.contains("#GetExternalIPAddress") {
                            soap(
                                r#"<u:GetExternalIPAddressResponse xmlns:u="urn:schemas-upnp-org:service:WANIPConnection:1"><NewExternalIPAddress>203.0.113.7</NewExternalIPAddress></u:GetExternalIPAddressResponse>"#,
                            )
                        } else {
                            String::new()
                        };
                        let status = if reply.is_empty() {
                            "404 Not Found"
                        } else {
                            "200 OK"
                        };
                        let response = format!(
                            "HTTP/1.1 {status}\r\nContent-Type: text/xml\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{reply}",
                            reply.len()
                        );
                        let _ = stream.write_all(response.as_bytes()).await;
                        let _ = stream.shutdown().await;
                    });
                }
            });
            let socket = UdpSocket::bind(("127.0.0.1", 0)).await.unwrap();
            let ssdp = socket.local_addr().unwrap();
            let responder = tokio::spawn(async move {
                let mut buffer = [0u8; 2048];
                loop {
                    let Ok((_, from)) = socket.recv_from(&mut buffer).await else {
                        return;
                    };
                    let reply = format!(
                        "HTTP/1.1 200 OK\r\nCACHE-CONTROL: max-age=120\r\nLOCATION: http://127.0.0.1:{http_port}/root.xml\r\nST: urn:schemas-upnp-org:device:InternetGatewayDevice:1\r\nUSN: uuid:fake::urn:schemas-upnp-org:device:InternetGatewayDevice:1\r\n\r\n"
                    );
                    let _ = socket.send_to(reply.as_bytes(), from).await;
                }
            });
            Router {
                ssdp,
                mappings,
                tasks: vec![server, responder],
            }
        }
    }

    /// An ED2K state whose UPnP client looks for its router at `ssdp` instead of the network.
    fn state_with_router_at(root: &Path, ssdp: std::net::SocketAddr) -> Ed2kState {
        use nibiru_xtreme::upnp::Timing;
        let mut state = Ed2kState::new(root.join("app/ed2k-kad")).unwrap();
        state.net.upnp = Upnp::with(
            "Apocalipse test",
            Timing {
                lease_secs: 3600,
                renew: Duration::from_secs(3600),
                search: Duration::from_millis(1500),
            },
            Box::new(move || {
                let mut options = igd_next::SearchOptions::default();
                options.bind_addr = "127.0.0.1:0".parse().unwrap();
                options.broadcast_address = ssdp;
                options.timeout = Some(Duration::from_millis(1500));
                options.single_search_timeout = Some(Duration::from_millis(500));
                options.allowed_gateway_ips = vec!["127.0.0.1".parse().unwrap()];
                options
            }),
        );
        state
    }

    #[tokio::test]
    async fn the_tcp_port_is_forwarded_before_the_server_login_so_the_id_is_high() {
        let root = tempfile::tempdir().unwrap();
        let router = fake_router::start().await;
        let state = state_with_router_at(root.path(), router.ssdp);
        // Nothing is forwarded until a login is about to happen.
        assert!(router.mappings.lock().unwrap().is_empty());
        forward_before_login(&state).await;
        // The moment this returns (that is, before the login is attempted) the router already
        // forwards the port the server will test.
        let port = state
            .incoming
            .lock()
            .await
            .as_ref()
            .unwrap()
            .status()
            .tcp_port;
        assert_eq!(
            *router.mappings.lock().unwrap(),
            vec![("TCP".to_owned(), port)]
        );
        assert_eq!(state.net.upnp.status().state, "mapped");
        // Logging in again changes nothing and does not pile up mappings.
        forward_before_login(&state).await;
        assert_eq!(router.mappings.lock().unwrap().len(), 1);
        state.net.upnp.release().await;
        assert!(router.mappings.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn the_option_off_forwards_nothing_and_a_missing_router_does_not_delay_every_login() {
        let root = tempfile::tempdir().unwrap();
        let router = fake_router::start().await;
        let state = state_with_router_at(root.path(), router.ssdp);
        state
            .net
            .save(Prefs {
                upnp: false,
                ..state.net.prefs()
            })
            .unwrap();
        forward_before_login(&state).await;
        assert!(router.mappings.lock().unwrap().is_empty(), "UPnP is off");

        // A router that never answers: the first login waits for the search, the next ones do not.
        let silent = tokio::net::UdpSocket::bind(("127.0.0.1", 0)).await.unwrap();
        let state = state_with_router_at(root.path(), silent.local_addr().unwrap());
        state.net.save(Prefs::default()).unwrap();
        forward_before_login(&state).await;
        let status = state.net.upnp.status();
        assert_eq!(status.state, "failed");
        assert!(status.error.unwrap().starts_with("upnp_no_gateway"));
        let started = std::time::Instant::now();
        forward_before_login(&state).await;
        assert!(
            started.elapsed() < Duration::from_millis(400),
            "a network without UPnP must not delay the login again"
        );
    }
}
