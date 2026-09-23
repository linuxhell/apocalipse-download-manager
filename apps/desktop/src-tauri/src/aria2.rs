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
    // Number of HTTP/HTTPS webseed URIs aria2 is actively pulling from
    // alongside the BT swarm for this download (0 for a plain HTTP/FTP
    // task, where every file only ever has its own source URI "in use").
    pub web_seeds: u64,
    /// Sum of aria2-reported lengths for selected torrent files.
    pub selected_file_bytes: u64,
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
        // Reserving a loopback port and releasing it before aria2next binds the
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
            // "debug" (rather than "notice") makes aria2 write DHT
            // bootstrap, tracker/peer activity, and per-peer BitTorrent
            // wire-protocol events (extended handshake, ut_metadata piece
            // requests/responses) into aria2.log. A user's diagnostic
            // bundle proved "info" wasn't enough: it showed 40 peer
            // connections and a known metadata size (via the ut_metadata
            // handshake) but zero metadata bytes ever received across two
            // full 150s attempts - a stall inside the piece exchange itself
            // that "info"-level logging can't explain. This is the only way
            // to tell "no peers reachable" (network/firewall), "a
            // config/state bug on our side" (e.g. followed into a real
            // download), and "peers connected but none actually serve the
            // metadata piece" (a dead/poisoned swarm) apart from each other.
            .arg("--log-level=debug")
            .arg(format!("--log={}", log.display()))
            .arg("--download-result=hide")
            .arg(format!("--input-file={}", session.display()))
            .arg(format!("--save-session={}", session.display()))
            .arg("--save-session-interval=30")
            // BitTorrent extensions: DHT (aria2-next's libtorrent-rasterbar
            // backend handles IPv4 and IPv6 together under one flag now -
            // the old separate --enable-dht6 is accepted only as a legacy
            // alias that logs a warning and maps onto this same option) and
            // Local Peer Discovery find peers without a tracker, Peer
            // Exchange (PEX) trades known peers with already-connected ones
            // (on by default, kept explicit), and the Fast Extension plus
            // UDP tracker support are always compiled in with no flag
            // needed. bt-encryption=preferred prefers MSE/PSE-encrypted peer
            // connections without refusing plaintext ones (upstream aria2's
            // separate bt-min-crypto-level/bt-require-crypto pair was
            // replaced by this single option).
            .arg("--enable-dht=true")
            .arg("--enable-peer-exchange=true")
            .arg("--bt-enable-lpd=true")
            .arg("--bt-encryption=preferred")
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
        // aria2next exit almost immediately; catch that here so the caller can
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
            "followedBy",
            "files"
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
            selected_file_bytes: value
                .get("files")
                .and_then(Value::as_array)
                .map(|files| {
                    files
                        .iter()
                        .filter(|file| file.get("selected").and_then(Value::as_str) == Some("true"))
                        .map(|file| number(file.get("length")))
                        .sum()
                })
                .unwrap_or(0),
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
            // For a BitTorrent download, each file's "uris" lists the
            // WebSeeding (BEP 19) HTTP/HTTPS sources declared in the
            // torrent itself; "used" means aria2 is actively pulling bytes
            // from it right now, alongside the BT swarm. Meaningless for a
            // plain HTTP/FTP task (its single source URI always shows as
            // "used" here too), so callers only surface this for BT.
            web_seeds: value
                .get("files")
                .and_then(Value::as_array)
                .map(|files| {
                    files
                        .iter()
                        .filter_map(|file| file.get("uris"))
                        .filter_map(Value::as_array)
                        .flatten()
                        .filter(|uri| {
                            uri.get("status").and_then(Value::as_str) == Some("used")
                                && uri.get("uri").and_then(Value::as_str).is_some_and(|value| {
                                    value.starts_with("http://") || value.starts_with("https://")
                                })
                        })
                        .count() as u64
                })
                .unwrap_or(0),
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
        // Fetch the first and last pieces of every file first so a
        // preview/player can start reading before the rest has arrived.
        // aria2-next's libtorrent-rasterbar backend replaced the old
        // byte-range bt-prioritize-piece=head=2M,tail=2M option with this
        // boolean (libtorrent's own "prioritize first/last piece" knob, no
        // custom byte range).
        options.insert(
            "bt-first-last-piece-first".into(),
            Value::String("true".into()),
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
    /// downloading file content. aria2 first fetches the BEP 9 metadata job,
    /// then creates the real BitTorrent content GID. `pause-metadata=true`
    /// makes that follow-up GID start paused, so we can inspect its real file
    /// list and sizes before removing both temporary jobs.
    pub async fn preview_magnet_metadata(
        &self,
        magnet: &str,
        workspace: &Path,
        mut on_progress: impl FnMut(u64, i64, i64, i64, i64, Option<&str>),
    ) -> Result<TorrentMetadata, String> {
        let mut options = Map::new();
        options.insert(
            "dir".into(),
            Value::String(workspace.to_string_lossy().into_owned()),
        );
        // bt-metadata-only=true leaves tellStatus(files) pointing at aria2's
        // synthetic [METADATA] object (the .torrent/BEP 9 blob), not at the
        // files described by the torrent. Let aria2 create its normal content
        // GID instead, but pause that child immediately so no payload bytes
        // are downloaded during this preview.
        options.insert("bt-metadata-only".into(), Value::String("false".into()));
        options.insert("bt-save-metadata".into(), Value::String("false".into()));
        options.insert("pause-metadata".into(), Value::String("true".into()));
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
        let keys = json!([
            "status",
            "errorMessage",
            "bittorrent",
            "files",
            "connections",
            "numSeeders",
            "totalLength",
            "completedLength",
            "followedBy"
        ]);
        // Metadata arrives once at least one connected peer answers the BEP
        // 9 exchange. A cold DHT routing table can need time to bootstrap, so
        // keep the existing 150-second ceiling. Once the metadata GID finishes
        // it should expose a followedBy content GID; pause-metadata keeps that
        // content GID from downloading while we read bittorrent/files from it.
        let deadline = tokio::time::Instant::now() + Duration::from_secs(150);
        let start = tokio::time::Instant::now();
        let mut last_report = start;
        let mut peak_connections: i64 = 0;
        let mut peak_seeders: i64 = 0;
        let mut peak_total_length: i64 = 0;
        let mut peak_completed_length: i64 = 0;
        let mut metadata_completed_at: Option<tokio::time::Instant> = None;
        let mut followed_content_gid: Option<String> = None;
        let result = loop {
            let value = self
                .call(
                    "aria2.tellStatus",
                    vec![Value::String(gid.clone()), keys.clone()],
                )
                .await?;
            let parse_i64 = |key: &str| {
                value
                    .get(key)
                    .and_then(Value::as_str)
                    .and_then(|value| value.parse::<i64>().ok())
                    .unwrap_or(0)
            };
            let connections = parse_i64("connections");
            let seeders = parse_i64("numSeeders");
            let total_length = parse_i64("totalLength");
            let completed_length = parse_i64("completedLength");
            let followed_by = value
                .get("followedBy")
                .and_then(Value::as_array)
                .and_then(|list| list.first())
                .and_then(Value::as_str);
            peak_connections = peak_connections.max(connections);
            peak_seeders = peak_seeders.max(seeders);
            peak_total_length = peak_total_length.max(total_length);
            peak_completed_length = peak_completed_length.max(completed_length);
            let now = tokio::time::Instant::now();
            if now.duration_since(last_report) >= Duration::from_secs(5) || followed_by.is_some() {
                last_report = now;
                on_progress(
                    now.duration_since(start).as_secs(),
                    peak_connections,
                    peak_seeders,
                    peak_total_length,
                    peak_completed_length,
                    followed_by,
                );
            }

            if let Some(next_gid) = followed_by {
                if followed_content_gid.as_deref() != Some(next_gid) {
                    followed_content_gid = Some(next_gid.to_owned());
                }
            }

            if let Some(content_gid) = followed_content_gid.as_deref() {
                let content_status = self
                    .call(
                        "aria2.tellStatus",
                        vec![
                            Value::String(content_gid.to_owned()),
                            json!([
                                "status",
                                "errorMessage",
                                "bittorrent",
                                "files",
                                "totalLength",
                                "completedLength"
                            ]),
                        ],
                    )
                    .await?;
                let has_bittorrent_info = content_status
                    .get("bittorrent")
                    .and_then(|bt| bt.get("info"))
                    .is_some();
                let has_real_files = content_status
                    .get("files")
                    .and_then(Value::as_array)
                    .is_some_and(|files| !files.is_empty());
                if has_bittorrent_info && has_real_files {
                    break Ok(content_status);
                }
                if matches!(
                    content_status.get("status").and_then(Value::as_str),
                    Some("error" | "removed")
                ) {
                    let message = content_status
                        .get("errorMessage")
                        .and_then(Value::as_str)
                        .filter(|value| !value.is_empty())
                        .unwrap_or("aria2_metadata_followup_failed");
                    break Err(message.to_owned());
                }
            }

            match value.get("status").and_then(Value::as_str) {
                Some("complete") => {
                    if metadata_completed_at.is_none() {
                        metadata_completed_at = Some(now);
                    }
                    if followed_content_gid.is_none()
                        && metadata_completed_at
                            .as_ref()
                            .is_some_and(|at| now.duration_since(*at) >= Duration::from_secs(5))
                    {
                        break Err("aria2_metadata_followup_missing".to_owned());
                    }
                }
                Some("error" | "removed") => {
                    let message = value
                        .get("errorMessage")
                        .and_then(Value::as_str)
                        .filter(|value| !value.is_empty())
                        .unwrap_or("aria2_metadata_failed");
                    break Err(message.to_owned());
                }
                _ => {}
            }

            if now >= deadline {
                break Err(format!(
                    "aria2_metadata_timeout:connections={peak_connections}:seeders={peak_seeders}"
                ));
            }
            tokio::time::sleep(Duration::from_millis(500)).await;
        };

        let _ = self
            .call("aria2.forceRemove", vec![Value::String(gid.clone())])
            .await;
        let _ = self
            .call("aria2.removeDownloadResult", vec![Value::String(gid)])
            .await;
        if let Some(content_gid) = followed_content_gid {
            let _ = self
                .call(
                    "aria2.forceRemove",
                    vec![Value::String(content_gid.clone())],
                )
                .await;
            let _ = self
                .call(
                    "aria2.removeDownloadResult",
                    vec![Value::String(content_gid)],
                )
                .await;
        }
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
