use base64::Engine as _;
use reqwest::Client;
use serde_json::{json, Map, Value};
use std::{
    collections::HashMap,
    fs,
    net::TcpListener,
    path::Path,
    process::{Child, Command, Stdio},
    time::Duration,
};

#[derive(Clone)]
pub struct Endpoint {
    base_url: String,
    secret: String,
    client: Client,
}

pub struct Runtime {
    child: Child,
    endpoint: Endpoint,
    port: u16,
}

#[derive(Clone, Default)]
pub struct RequestContext {
    pub method: String,
    pub headers: HashMap<String, String>,
    pub body: String,
}

#[derive(Debug, Clone, Default)]
pub struct RuntimeStatus {
    pub status: String,
    pub downloaded: u64,
    pub total: u64,
    pub upload_speed: u64,
    pub speed: u64,
    pub connections: u64,
    pub error_message: Option<String>,
    // BitTorrent-only: populated when tellStatus includes a "bittorrent"
    // object, absent for plain HTTP/FTP transfers.
    pub seeders: Option<u64>,
    pub followed_by: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct TorrentFile {
    // 1-based, matching aria2's own select-file convention.
    pub index: usize,
    pub path: String,
    pub length: u64,
}

#[derive(Debug, Clone, Default)]
pub struct TorrentMetadata {
    pub name: String,
    pub files: Vec<TorrentFile>,
    pub total_size: u64,
}

fn number(value: Option<&Value>) -> u64 {
    value
        .and_then(|value| {
            value
                .as_str()
                .and_then(|text| text.parse::<u64>().ok())
                .or_else(|| value.as_u64())
        })
        .unwrap_or(0)
}

fn reserve_loopback_port(requested: Option<u16>) -> Result<u16, String> {
    if let Some(port) = requested.filter(|port| *port > 0) {
        let listener = TcpListener::bind(("127.0.0.1", port)).map_err(|error| error.to_string())?;
        drop(listener);
        return Ok(port);
    }
    let listener = TcpListener::bind(("127.0.0.1", 0)).map_err(|error| error.to_string())?;
    let port = listener
        .local_addr()
        .map_err(|error| error.to_string())?
        .port();
    drop(listener);
    Ok(port)
}

impl Runtime {
    pub fn spawn(
        executable: &Path,
        runtime_root: &Path,
        requested_port: Option<u16>,
        secret: &str,
    ) -> Result<Self, String> {
        if !executable.is_file() {
            return Err("aria2_not_found".to_owned());
        }
        fs::create_dir_all(runtime_root).map_err(|error| error.to_string())?;
        // Reserving a loopback port and releasing it before aria2c binds the
        // same one is an inherent TOCTOU race (another process can grab it
        // first). We can't hand the bound socket to an external process, so
        // instead retry with a freshly picked port a few times when nothing
        // pinned the port explicitly and the child dies immediately after
        // spawn, which is the observable symptom of losing that race.
        let attempts = if requested_port.is_some() { 1 } else { 4 };
        let mut last_error = String::new();
        for attempt in 0..attempts {
            match Self::spawn_once(executable, runtime_root, requested_port, secret) {
                Ok(runtime) => return Ok(runtime),
                Err(error) => {
                    last_error = error;
                    if attempt + 1 < attempts {
                        std::thread::sleep(Duration::from_millis(50));
                    }
                }
            }
        }
        Err(last_error)
    }

    fn spawn_once(
        executable: &Path,
        runtime_root: &Path,
        requested_port: Option<u16>,
        secret: &str,
    ) -> Result<Self, String> {
        let port = reserve_loopback_port(requested_port)?;
        let session = runtime_root.join("aria2.session");
        if !session.exists() {
            fs::write(&session, b"").map_err(|error| error.to_string())?;
        }
        let log = runtime_root.join("aria2.log");
        let mut command = Command::new(executable);
        command
            .arg("--enable-rpc=true")
            .arg("--rpc-listen-all=false")
            .arg("--rpc-allow-origin-all=false")
            .arg(format!("--rpc-listen-port={port}"))
            .arg(format!("--rpc-secret={secret}"))
            .arg(format!("--stop-with-process={}", std::process::id()))
            .arg("--continue=true")
            .arg("--file-allocation=none")
            .arg("--auto-file-renaming=false")
            .arg("--allow-overwrite=true")
            .arg("--max-concurrent-downloads=20")
            .arg("--summary-interval=0")
            .arg("--console-log-level=warn")
            .arg("--log-level=notice")
            .arg(format!("--log={}", log.display()))
            .arg("--download-result=hide")
            .arg(format!("--input-file={}", session.display()))
            .arg(format!("--save-session={}", session.display()))
            .arg("--save-session-interval=30")
            // BitTorrent extensions: DHT (IPv4 + IPv6) and Local Peer
            // Discovery find peers without a tracker, Peer Exchange (PEX)
            // trades known peers with already-connected ones (on by
            // default, kept explicit), and the Fast Extension plus UDP
            // tracker support are always compiled into aria2 with no flag
            // needed. bt-min-crypto-level prefers MSE/PSE-encrypted peer
            // connections without refusing plaintext ones (bt-require-crypto
            // stays false for compatibility with older/plain peers).
            .arg("--enable-dht=true")
            .arg("--enable-dht6=true")
            .arg("--enable-peer-exchange=true")
            .arg("--bt-enable-lpd=true")
            .arg("--bt-min-crypto-level=arc4")
            .arg("--bt-require-crypto=false")
            .arg("--follow-torrent=true")
            // This is a download manager, not a seedbox: stop contributing
            // upload bandwidth for a torrent the moment it finishes instead
            // of continuing to seed in the background.
            .arg("--seed-time=0")
            .arg("--seed-ratio=0.0")
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            command.creation_flags(0x08000000);
        }
        let mut child = command.spawn().map_err(|error| error.to_string())?;
        // A port lost to another process between reservation and bind makes
        // aria2c exit almost immediately; catch that here so the caller can
        // retry with a different port instead of waiting out the full RPC
        // readiness timeout for a process that already died.
        std::thread::sleep(Duration::from_millis(150));
        if let Ok(Some(status)) = child.try_wait() {
            return Err(format!("aria2_exited_early:{status}"));
        }
        let client = Client::builder()
            .connect_timeout(Duration::from_secs(2))
            .timeout(Duration::from_secs(15))
            .build()
            .map_err(|error| error.to_string())?;
        Ok(Self {
            child,
            endpoint: Endpoint {
                base_url: format!("http://127.0.0.1:{port}/jsonrpc"),
                secret: secret.to_owned(),
                client,
            },
            port,
        })
    }

    pub fn endpoint(&self) -> Endpoint {
        self.endpoint.clone()
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn pid(&self) -> u32 {
        self.child.id()
    }

    pub fn is_running(&mut self) -> bool {
        matches!(self.child.try_wait(), Ok(None))
    }

    pub fn terminate(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

impl Drop for Runtime {
    fn drop(&mut self) {
        self.terminate();
    }
}

impl Endpoint {
    async fn call(&self, method: &str, mut params: Vec<Value>) -> Result<Value, String> {
        params.insert(0, Value::String(format!("token:{}", self.secret)));
        let response = self
            .client
            .post(&self.base_url)
            .json(&json!({
                "jsonrpc": "2.0",
                "id": "apocalipse",
                "method": method,
                "params": params,
            }))
            .send()
            .await
            .map_err(|error| error.to_string())?
            .error_for_status()
            .map_err(|error| error.to_string())?;
        let payload: Value = response.json().await.map_err(|error| error.to_string())?;
        if let Some(error) = payload.get("error") {
            let code = error
                .get("code")
                .and_then(Value::as_i64)
                .unwrap_or_default();
            let message = error
                .get("message")
                .and_then(Value::as_str)
                .unwrap_or("aria2_rpc_error");
            return Err(format!("aria2_rpc_error:{code}:{message}"));
        }
        payload
            .get("result")
            .cloned()
            .ok_or_else(|| "aria2_rpc_missing_result".to_owned())
    }

    pub async fn wait_ready(&self) -> Result<(), String> {
        let mut last_error = String::new();
        for _ in 0..80 {
            match self.call("aria2.getVersion", Vec::new()).await {
                Ok(_) => return Ok(()),
                Err(error) => last_error = error,
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        Err(format!("aria2_start_timeout:{last_error}"))
    }

    pub async fn version(&self) -> Result<String, String> {
        let value = self.call("aria2.getVersion", Vec::new()).await?;
        Ok(value
            .get("version")
            .and_then(Value::as_str)
            .unwrap_or("unknown")
            .to_owned())
    }

    pub async fn add_download(
        &self,
        source: &str,
        destination: &Path,
        connections: usize,
        context: &RequestContext,
        http_download: bool,
    ) -> Result<String, String> {
        if !context.body.is_empty()
            || (!context.method.is_empty() && !context.method.eq_ignore_ascii_case("GET"))
        {
            return Err("aria2_request_requires_native_http".to_owned());
        }
        let directory = destination.parent().unwrap_or_else(|| Path::new("."));
        let mut options = Map::new();
        options.insert(
            "dir".into(),
            Value::String(directory.to_string_lossy().into_owned()),
        );
        options.insert("continue".into(), Value::String("true".into()));
        options.insert("file-allocation".into(), Value::String("none".into()));
        if http_download {
            let connections = connections.clamp(1, 32);
            options.insert(
                "max-connection-per-server".into(),
                Value::String(connections.to_string()),
            );
            options.insert("split".into(), Value::String(connections.to_string()));
            options.insert("min-split-size".into(), Value::String("1M".into()));
            options.insert(
                "out".into(),
                Value::String(
                    destination
                        .file_name()
                        .and_then(|value| value.to_str())
                        .unwrap_or("download")
                        .to_owned(),
                ),
            );
        }
        if !context.headers.is_empty() {
            options.insert(
                "header".into(),
                Value::Array(
                    context
                        .headers
                        .iter()
                        .map(|(name, value)| Value::String(format!("{name}: {value}")))
                        .collect(),
                ),
            );
        }
        let value = self
            .call(
                "aria2.addUri",
                vec![json!([source]), Value::Object(options)],
            )
            .await?;
        value
            .as_str()
            .map(str::to_owned)
            .ok_or_else(|| "aria2_gid_missing".to_owned())
    }

    pub async fn status(&self, gid: &str) -> Result<RuntimeStatus, String> {
        let keys = json!([
            "status",
            "totalLength",
            "completedLength",
            "uploadLength",
            "downloadSpeed",
            "uploadSpeed",
            "connections",
            "errorMessage",
            "numSeeders",
            "followedBy"
        ]);
        let value = self
            .call(
                "aria2.tellStatus",
                vec![Value::String(gid.to_owned()), keys],
            )
            .await?;
        Ok(RuntimeStatus {
            status: value
                .get("status")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_owned(),
            total: number(value.get("totalLength")),
            downloaded: number(value.get("completedLength")),
            speed: number(value.get("downloadSpeed")),
            upload_speed: number(value.get("uploadSpeed")),
            connections: number(value.get("connections")),
            error_message: value
                .get("errorMessage")
                .and_then(Value::as_str)
                .filter(|value| !value.is_empty())
                .map(str::to_owned),
            seeders: value
                .get("numSeeders")
                .map(|_| number(value.get("numSeeders"))),
            // A magnet link is added as a metadata-only download first (BEP
            // 9); once its small metadata blob finishes, aria2 (with
            // follow-torrent=true, set globally in Runtime::spawn)
            // automatically starts the REAL content download under a new
            // GID, reachable only through this field. The caller must
            // switch to tracking that GID instead of treating the metadata
            // phase's "complete" status as the download being done.
            followed_by: value
                .get("followedBy")
                .and_then(Value::as_array)
                .and_then(|list| list.first())
                .and_then(Value::as_str)
                .map(str::to_owned),
        })
    }

    /// Adds a magnet link or a `.torrent` file (from a local path or its raw
    /// bytes) as an active BitTorrent download. `destination_dir` becomes the
    /// download's own root folder (matching how the rest of the app treats a
    /// Torrent/Magnet task's destination as a directory to clean up as a
    /// whole), and `only_files` is a 1-based file-index selection, matching
    /// aria2's own `select-file` convention (empty means "all files").
    pub async fn add_bittorrent(
        &self,
        source: &str,
        torrent_bytes: Option<&[u8]>,
        destination_dir: &Path,
        only_files: &[usize],
    ) -> Result<String, String> {
        let mut options = Map::new();
        options.insert(
            "dir".into(),
            Value::String(destination_dir.to_string_lossy().into_owned()),
        );
        options.insert("continue".into(), Value::String("true".into()));
        options.insert("file-allocation".into(), Value::String("none".into()));
        // Fetch the first and last couple of MB of every file first so a
        // preview/player can start reading before the rest has arrived.
        options.insert(
            "bt-prioritize-piece".into(),
            Value::String("head=2M,tail=2M".into()),
        );
        if !only_files.is_empty() {
            let selection = only_files
                .iter()
                .map(usize::to_string)
                .collect::<Vec<_>>()
                .join(",");
            options.insert("select-file".into(), Value::String(selection));
        }
        let value = if let Some(bytes) = torrent_bytes {
            let encoded = base64::engine::general_purpose::STANDARD.encode(bytes);
            self.call(
                "aria2.addTorrent",
                vec![
                    Value::String(encoded),
                    Value::Array(Vec::new()),
                    Value::Object(options),
                ],
            )
            .await?
        } else {
            self.call(
                "aria2.addUri",
                vec![json!([source]), Value::Object(options)],
            )
            .await?
        };
        value
            .as_str()
            .map(str::to_owned)
            .ok_or_else(|| "aria2_gid_missing".to_owned())
    }

    /// Resolves a magnet link's metadata (name, file list, sizes) without
    /// downloading any file content, using aria2's own metadata-only mode
    /// (BEP 9) over the BitTorrent network. `workspace` is a scratch
    /// directory the caller owns and should remove afterwards; aria2 needs
    /// one to resolve each returned file's path against, but no file
    /// content is ever written there.
    pub async fn preview_magnet_metadata(
        &self,
        magnet: &str,
        workspace: &Path,
    ) -> Result<TorrentMetadata, String> {
        let mut options = Map::new();
        options.insert(
            "dir".into(),
            Value::String(workspace.to_string_lossy().into_owned()),
        );
        options.insert("bt-metadata-only".into(), Value::String("true".into()));
        options.insert("bt-save-metadata".into(), Value::String("false".into()));
        // "mem" is aria2's documented value for metadata-only mode: it keeps
        // the resolved metadata in memory and marks this GID "complete" once
        // BEP 9 finishes, without spawning a follow-up content download.
        // "false" leaves aria2's internal state machine unable to reach a
        // clean "complete" status for a metadata-only GID, so polling for it
        // never succeeds and this always ran out the clock on the timeout
        // below instead of resolving in the couple of seconds it actually
        // takes once a peer answers.
        options.insert("follow-torrent".into(), Value::String("mem".into()));
        let gid = self
            .call(
                "aria2.addUri",
                vec![json!([magnet]), Value::Object(options)],
            )
            .await?;
        let gid = gid
            .as_str()
            .map(str::to_owned)
            .ok_or_else(|| "aria2_gid_missing".to_owned())?;
        let keys = json!(["status", "errorMessage", "bittorrent", "files"]);
        // Metadata arrives once at least one connected peer answers the
        // BEP 9 metadata exchange; that can take much longer than an
        // ordinary local API call on a slow-to-respond swarm, so this polls
        // for up to 90 seconds instead of a single short-timeout request.
        let deadline = tokio::time::Instant::now() + Duration::from_secs(90);
        let result = loop {
            let value = self
                .call(
                    "aria2.tellStatus",
                    vec![Value::String(gid.clone()), keys.clone()],
                )
                .await?;
            match value.get("status").and_then(Value::as_str) {
                Some("complete") => break Ok(value),
                Some("error") => {
                    let message = value
                        .get("errorMessage")
                        .and_then(Value::as_str)
                        .unwrap_or("aria2_metadata_failed");
                    break Err(message.to_owned());
                }
                _ if tokio::time::Instant::now() >= deadline => {
                    break Err("aria2_metadata_timeout".to_owned());
                }
                _ => tokio::time::sleep(Duration::from_millis(500)).await,
            }
        };
        let _ = self
            .call("aria2.forceRemove", vec![Value::String(gid)])
            .await;
        let value = result?;
        let name = value
            .get("bittorrent")
            .and_then(|bt| bt.get("info"))
            .and_then(|info| info.get("name"))
            .and_then(Value::as_str)
            .unwrap_or("torrent")
            .to_owned();
        let files: Vec<TorrentFile> = value
            .get("files")
            .and_then(Value::as_array)
            .map(|files| {
                files
                    .iter()
                    .enumerate()
                    .map(|(position, file)| {
                        let absolute = file.get("path").and_then(Value::as_str).unwrap_or_default();
                        let relative = Path::new(absolute)
                            .strip_prefix(workspace)
                            .ok()
                            .map(|path| path.to_string_lossy().replace('\\', "/"))
                            .unwrap_or_else(|| absolute.replace('\\', "/"));
                        TorrentFile {
                            index: file
                                .get("index")
                                .and_then(Value::as_str)
                                .and_then(|value| value.parse::<usize>().ok())
                                .unwrap_or(position + 1),
                            path: relative,
                            length: number(file.get("length")),
                        }
                    })
                    .collect()
            })
            .unwrap_or_default();
        let total_size = files.iter().map(|file: &TorrentFile| file.length).sum();
        if files.is_empty() {
            return Err("torrent_metadata_unavailable".to_owned());
        }
        Ok(TorrentMetadata {
            name,
            files,
            total_size,
        })
    }

    pub async fn pause(&self, gid: &str) -> Result<(), String> {
        self.call("aria2.pause", vec![Value::String(gid.to_owned())])
            .await
            .map(|_| ())
    }

    pub async fn resume(&self, gid: &str) -> Result<(), String> {
        self.call("aria2.unpause", vec![Value::String(gid.to_owned())])
            .await
            .map(|_| ())
    }

    pub async fn remove(&self, gid: &str) -> Result<(), String> {
        match self
            .call("aria2.remove", vec![Value::String(gid.to_owned())])
            .await
        {
            Ok(_) => Ok(()),
            Err(_) => self
                .call("aria2.forceRemove", vec![Value::String(gid.to_owned())])
                .await
                .map(|_| ()),
        }
    }

    pub async fn remove_result(&self, gid: &str) -> Result<(), String> {
        self.call(
            "aria2.removeDownloadResult",
            vec![Value::String(gid.to_owned())],
        )
        .await
        .map(|_| ())
    }
}
