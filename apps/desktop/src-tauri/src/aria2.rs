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
    /// Classic aria2 HTTP/HTTPS/FTP proxy URI.
    pub proxy_url: Option<String>,
    pub proxy_username: Option<String>,
    pub proxy_password: Option<String>,
    pub proxy_required: bool,
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
        fs::create_dir_all(runtime_root).map_err(|e| e.to_string())?;
        let attempts = if requested_port.is_some() { 1 } else { 4 };
        let mut last = String::new();
        for attempt in 0..attempts {
            match Self::spawn_once(executable, runtime_root, requested_port, secret) {
                Ok(r) => return Ok(r),
                Err(e) => {
                    last = e;
                    if attempt + 1 < attempts {
                        std::thread::sleep(Duration::from_millis(50));
                    }
                }
            }
        }
        Err(last)
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
            fs::write(&session, b"").map_err(|e| e.to_string())?
        }
        let log = runtime_root.join("aria2.log");
        let mut command = Command::new(executable);
        command.current_dir(runtime_root);
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
            .arg("--log-level=debug")
            .arg(format!("--log={}", log.display()))
            .arg("--download-result=hide")
            .arg(format!("--input-file={}", session.display()))
            .arg(format!("--save-session={}", session.display()))
            .arg("--save-session-interval=30")
            .arg("--enable-dht=true")
            .arg("--enable-dht6=true")
            .arg("--enable-peer-exchange=true")
            .arg("--bt-enable-lpd=true")
            .arg("--bt-min-crypto-level=arc4")
            .arg("--bt-require-crypto=false")
            .arg("--follow-torrent=true")
            .arg("--listen-port=6881-6999")
            .arg("--dht-listen-port=6881-6999")
            .arg("--bt-max-peers=50")
            .arg("--bt-request-peer-speed-limit=50K")
            .arg("--seed-time=0")
            .arg("--seed-ratio=0.0")
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            command.creation_flags(0x08000000);
        }
        let mut child = command.spawn().map_err(|e| e.to_string())?;
        std::thread::sleep(Duration::from_millis(150));
        if let Ok(Some(status)) = child.try_wait() {
            return Err(format!("aria2_exited_early:{status}"));
        }
        let client = Client::builder()
            .connect_timeout(Duration::from_secs(2))
            .timeout(Duration::from_secs(15))
            .build()
            .map_err(|e| e.to_string())?;
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
        download_limit: u64,
    ) -> Result<String, String> {
        if !context.body.is_empty()
            || (!context.method.is_empty() && !context.method.eq_ignore_ascii_case("GET"))
        {
            return Err("aria2_request_requires_native_http".to_owned());
        }
        if context.proxy_required && context.proxy_url.is_none() {
            return Err("aria2_proxy_scheme_unsupported".to_owned());
        }
        let directory = destination.parent().unwrap_or_else(|| Path::new("."));
        let mut options = Map::new();
        options.insert(
            "dir".into(),
            Value::String(directory.to_string_lossy().into_owned()),
        );
        options.insert("continue".into(), Value::String("true".into()));
        options.insert("file-allocation".into(), Value::String("none".into()));
        if download_limit > 0 {
            options.insert(
                "max-download-limit".into(),
                Value::String(download_limit.to_string()),
            );
        }
        if let Some(proxy) = context.proxy_url.as_ref() {
            options.insert("all-proxy".into(), Value::String(proxy.clone()));
            if let Some(u) = context.proxy_username.as_ref().filter(|v| !v.is_empty()) {
                options.insert("all-proxy-user".into(), Value::String(u.clone()));
            }
            if let Some(p) = context.proxy_password.as_ref().filter(|v| !v.is_empty()) {
                options.insert("all-proxy-passwd".into(), Value::String(p.clone()));
            }
        }
        if http_download {
            let c = connections.clamp(1, 16);
            options.insert(
                "max-connection-per-server".into(),
                Value::String(c.to_string()),
            );
            options.insert("split".into(), Value::String(c.to_string()));
            options.insert("min-split-size".into(), Value::String("1M".into()));
            options.insert(
                "out".into(),
                Value::String(
                    destination
                        .file_name()
                        .and_then(|v| v.to_str())
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
                        .map(|(n, v)| Value::String(format!("{n}: {v}")))
                        .collect(),
                ),
            );
        }
        let v = self
            .call(
                "aria2.addUri",
                vec![json!([source]), Value::Object(options)],
            )
            .await?;
        v.as_str()
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
        let v = self
            .call(
                "aria2.tellStatus",
                vec![Value::String(gid.to_owned()), keys],
            )
            .await?;
        Ok(RuntimeStatus {
            status: v
                .get("status")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_owned(),
            total: number(v.get("totalLength")),
            downloaded: number(v.get("completedLength")),
            selected_file_bytes: v
                .get("files")
                .and_then(Value::as_array)
                .map(|fs| {
                    fs.iter()
                        .filter(|f| f.get("selected").and_then(Value::as_str) == Some("true"))
                        .map(|f| number(f.get("length")))
                        .sum()
                })
                .unwrap_or(0),
            speed: number(v.get("downloadSpeed")),
            upload_speed: number(v.get("uploadSpeed")),
            connections: number(v.get("connections")),
            error_message: v
                .get("errorMessage")
                .and_then(Value::as_str)
                .filter(|x| !x.is_empty())
                .map(str::to_owned),
            seeders: v.get("numSeeders").map(|_| number(v.get("numSeeders"))),
            followed_by: v
                .get("followedBy")
                .and_then(Value::as_array)
                .and_then(|x| x.first())
                .and_then(Value::as_str)
                .map(str::to_owned),
            web_seeds: v
                .get("files")
                .and_then(Value::as_array)
                .map(|fs| {
                    fs.iter()
                        .filter_map(|f| f.get("uris"))
                        .filter_map(Value::as_array)
                        .flatten()
                        .filter(|u| {
                            u.get("status").and_then(Value::as_str) == Some("used")
                                && u.get("uri").and_then(Value::as_str).is_some_and(|x| {
                                    x.starts_with("http://") || x.starts_with("https://")
                                })
                        })
                        .count() as u64
                })
                .unwrap_or(0),
        })
    }

    /// Adds a magnet link or a `.torrent` file as an active BitTorrent
    /// download. Magnet file selection is deliberately deferred until
    /// metadata-only exposes the real file list on the same aria2 GID.
    pub async fn add_bittorrent(
        &self,
        source: &str,
        torrent_bytes: Option<&[u8]>,
        destination_dir: &Path,
        only_files: &[usize],
        download_limit: u64,
    ) -> Result<String, String> {
        let mut o = Map::new();
        o.insert(
            "dir".into(),
            Value::String(destination_dir.to_string_lossy().into_owned()),
        );
        o.insert("continue".into(), Value::String("true".into()));
        o.insert("file-allocation".into(), Value::String("none".into()));
        o.insert(
            "bt-prioritize-piece".into(),
            Value::String("head,tail".into()),
        );
        if download_limit > 0 {
            o.insert(
                "max-download-limit".into(),
                Value::String(download_limit.to_string()),
            );
        }
        if !only_files.is_empty() {
            o.insert(
                "select-file".into(),
                Value::String(
                    only_files
                        .iter()
                        .map(usize::to_string)
                        .collect::<Vec<_>>()
                        .join(","),
                ),
            );
        }
        let v = if let Some(bytes) = torrent_bytes {
            let enc = base64::engine::general_purpose::STANDARD.encode(bytes);
            self.call(
                "aria2.addTorrent",
                vec![
                    Value::String(enc),
                    Value::Array(Vec::new()),
                    Value::Object(o),
                ],
            )
            .await?
        } else {
            self.call("aria2.addUri", vec![json!([source]), Value::Object(o)])
                .await?
        };
        v.as_str()
            .map(str::to_owned)
            .ok_or_else(|| "aria2_gid_missing".to_owned())
    }

    /// Resolves a magnet link's metadata (name, file list, sizes) without
    /// downloading file content. aria2 keeps the same GID from BEP 9
    /// metadata discovery through file selection and payload. With
    /// `metadata-only=true` that GID pauses after metadata is validated so
    /// the complete real file list can be inspected without payload transfer.
    pub async fn preview_magnet_metadata(
        &self,
        magnet: &str,
        workspace: &Path,
        mut on_progress: impl FnMut(u64, i64, i64, i64, i64, Option<&str>),
    ) -> Result<TorrentMetadata, String> {
        let mut o = Map::new();
        o.insert(
            "dir".into(),
            Value::String(workspace.to_string_lossy().into_owned()),
        );
        o.insert("bt-metadata-only".into(), Value::String("true".into()));
        o.insert("bt-save-metadata".into(), Value::String("false".into()));
        o.insert("file-allocation".into(), Value::String("none".into()));
        let gid = self
            .call("aria2.addUri", vec![json!([magnet]), Value::Object(o)])
            .await?
            .as_str()
            .map(str::to_owned)
            .ok_or_else(|| "aria2_gid_missing".to_owned())?;
        let start = tokio::time::Instant::now();
        let deadline = start + Duration::from_secs(150);
        let mut last = start;
        let (mut pc, mut ps, mut pt, mut pd) = (0_i64, 0_i64, 0_i64, 0_i64);
        let mut followed: Option<String> = None;
        let result = loop {
            let v = self
                .call(
                    "aria2.tellStatus",
                    vec![
                        Value::String(gid.clone()),
                        json!([
                            "status",
                            "errorMessage",
                            "bittorrent",
                            "connections",
                            "numSeeders",
                            "totalLength",
                            "completedLength",
                            "followedBy"
                        ]),
                    ],
                )
                .await?;
            let n = |k: &str| {
                v.get(k)
                    .and_then(Value::as_str)
                    .and_then(|x| x.parse::<i64>().ok())
                    .unwrap_or(0)
            };
            pc = pc.max(n("connections"));
            ps = ps.max(n("numSeeders"));
            pt = pt.max(n("totalLength"));
            pd = pd.max(n("completedLength"));
            if followed.is_none() {
                followed = v
                    .get("followedBy")
                    .and_then(Value::as_array)
                    .and_then(|x| x.first())
                    .and_then(Value::as_str)
                    .map(str::to_owned);
            }
            let now = tokio::time::Instant::now();
            if now.duration_since(last) >= Duration::from_secs(5) || followed.is_some() {
                last = now;
                on_progress(
                    now.duration_since(start).as_secs(),
                    pc,
                    ps,
                    pt,
                    pd,
                    followed.as_deref(),
                );
            }
            let inspect = followed.as_deref().unwrap_or(&gid);
            if let Ok(fv) = self
                .call("aria2.getFiles", vec![Value::String(inspect.to_owned())])
                .await
            {
                let mut files = Vec::new();
                if let Some(es) = fv.as_array() {
                    for f in es {
                        let p = f.get("path").and_then(Value::as_str).unwrap_or_default();
                        let len = number(f.get("length"));
                        let nm = Path::new(p)
                            .file_name()
                            .and_then(|x| x.to_str())
                            .unwrap_or(p);
                        if len == 0 || nm.eq_ignore_ascii_case("[METADATA]") {
                            continue;
                        }
                        let idx = f
                            .get("index")
                            .and_then(Value::as_str)
                            .and_then(|x| x.parse::<usize>().ok())
                            .unwrap_or(files.len() + 1);
                        files.push(TorrentFile {
                            index: idx,
                            path: p.to_owned(),
                            length: len,
                        });
                    }
                }
                if !files.is_empty() {
                    let total_size = files.iter().map(|f| f.length).sum();
                    let name = v
                        .get("bittorrent")
                        .and_then(|x| x.get("info"))
                        .and_then(|x| x.get("name"))
                        .and_then(Value::as_str)
                        .map(str::to_owned)
                        .or_else(|| {
                            files.first().and_then(|f| {
                                Path::new(&f.path)
                                    .file_name()
                                    .and_then(|x| x.to_str())
                                    .map(str::to_owned)
                            })
                        })
                        .unwrap_or_else(|| "torrent".to_owned());
                    break Ok(TorrentMetadata {
                        name,
                        files,
                        total_size,
                    });
                }
            }
            if matches!(
                v.get("status").and_then(Value::as_str),
                Some("error" | "removed")
            ) {
                break Err(v
                    .get("errorMessage")
                    .and_then(Value::as_str)
                    .unwrap_or("torrent_metadata_unavailable")
                    .to_owned());
            }
            if now >= deadline {
                break Err(format!("torrent_metadata_unavailable:connections={pc}:seeders={ps}:total={pt}:completed={pd}"));
            }
            tokio::time::sleep(Duration::from_millis(250)).await;
        };
        if let Some(child) = followed.as_deref() {
            let _ = self.remove(child).await;
            let _ = self.remove_result(child).await;
        }
        let _ = self.remove(&gid).await;
        let _ = self.remove_result(&gid).await;
        result
    }
    pub async fn set_download_limit(&self, gid: &str, bytes_per_second: u64) -> Result<(), String> {
        let mut options = Map::new();
        options.insert(
            "max-download-limit".into(),
            Value::String(bytes_per_second.to_string()),
        );
        self.call(
            "aria2.changeOption",
            vec![Value::String(gid.to_owned()), Value::Object(options)],
        )
        .await
        .map(|_| ())
    }

    pub async fn set_global_download_limit(&self, bytes_per_second: u64) -> Result<(), String> {
        let mut options = Map::new();
        options.insert(
            "max-overall-download-limit".into(),
            Value::String(bytes_per_second.to_string()),
        );
        self.call("aria2.changeGlobalOption", vec![Value::Object(options)])
            .await
            .map(|_| ())
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
