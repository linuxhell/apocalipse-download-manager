use base64::Engine as _;
use reqwest::Client;
use serde_json::{json, Map, Value};
use std::{
    collections::HashMap,
    fs,
    net::TcpListener,
    path::{Path, PathBuf},
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
    // Number of HTTP/HTTPS webseed URIs aria2 is actively pulling from
    // alongside the BT swarm for this download (0 for a plain HTTP/FTP
    // task, where every file only ever has its own source URI "in use").
    pub web_seeds: u64,
    /// Sum of aria2-reported lengths for selected torrent files.
    pub selected_file_bytes: u64,
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

    /// Downloads only Magnet metadata and persists it as a real .torrent file.
    /// No torrent payload is started here. The saved file is later parsed by ADM
    /// and passed back to aria2 with the user's select-file indexes.
    pub async fn save_magnet_metadata(
        &self,
        magnet: &str,
        torrents_dir: &Path,
        mut on_progress: impl FnMut(u64, &str, u64, u64, u64, u64, Option<&str>),
    ) -> Result<PathBuf, String> {
        fs::create_dir_all(torrents_dir).map_err(|error| error.to_string())?;
        let mut options = Map::new();
        options.insert(
            "dir".into(),
            Value::String(torrents_dir.to_string_lossy().into_owned()),
        );
        options.insert(
            "bt-metadata-only".into(),
            Value::String("true".into()),
        );
        options.insert(
            "bt-save-metadata".into(),
            Value::String("true".into()),
        );
        options.insert("file-allocation".into(), Value::String("none".into()));

        let gid = self
            .call(
                "aria2.addUri",
                vec![json!([magnet]), Value::Object(options)],
            )
            .await?
            .as_str()
            .map(str::to_owned)
            .ok_or_else(|| "aria2_gid_missing".to_owned())?;

        let started = tokio::time::Instant::now();
        let deadline = started + Duration::from_secs(150);
        let mut last_report = started;
        let mut last_status = String::new();
        let (mut peak_connections, mut peak_seeders, mut peak_total, mut peak_completed) =
            (0_u64, 0_u64, 0_u64, 0_u64);

        let result = loop {
            let value = self
                .call(
                    "aria2.tellStatus",
                    vec![
                        Value::String(gid.clone()),
                        json!([
                            "status",
                            "errorMessage",
                            "connections",
                            "numSeeders",
                            "totalLength",
                            "completedLength",
                            "infoHash"
                        ]),
                    ],
                )
                .await?;

            let status = value
                .get("status")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_owned();
            peak_connections = peak_connections.max(number(value.get("connections")));
            peak_seeders = peak_seeders.max(number(value.get("numSeeders")));
            peak_total = peak_total.max(number(value.get("totalLength")));
            peak_completed = peak_completed.max(number(value.get("completedLength")));
            let info_hash = value
                .get("infoHash")
                .and_then(Value::as_str)
                .filter(|hash| !hash.is_empty());

            let now = tokio::time::Instant::now();
            if status != last_status
                || now.duration_since(last_report) >= Duration::from_secs(5)
                || status == "complete"
            {
                last_status = status.clone();
                last_report = now;
                on_progress(
                    now.duration_since(started).as_secs(),
                    &status,
                    peak_connections,
                    peak_seeders,
                    peak_total,
                    peak_completed,
                    info_hash,
                );
            }

            if status == "complete" {
                let Some(info_hash) = info_hash else {
                    break Err(format!(
                        "torrent_metadata_saved_without_info_hash:connections={peak_connections}:seeders={peak_seeders}:total={peak_total}:completed={peak_completed}"
                    ));
                };
                let direct = torrents_dir.join(format!("{}.torrent", info_hash.to_ascii_lowercase()));
                let saved = if direct.is_file() {
                    Some(direct)
                } else {
                    fs::read_dir(torrents_dir)
                        .ok()
                        .into_iter()
                        .flatten()
                        .filter_map(Result::ok)
                        .map(|entry| entry.path())
                        .find(|path| {
                            path.extension()
                                .and_then(|ext| ext.to_str())
                                .is_some_and(|ext| ext.eq_ignore_ascii_case("torrent"))
                                && path
                                    .file_stem()
                                    .and_then(|stem| stem.to_str())
                                    .is_some_and(|stem| stem.eq_ignore_ascii_case(info_hash))
                        })
                };
                if let Some(saved) = saved {
                    break Ok(saved);
                }
                break Err(format!(
                    "torrent_metadata_file_missing:info_hash={info_hash}:connections={peak_connections}:seeders={peak_seeders}:total={peak_total}:completed={peak_completed}"
                ));
            }

            if matches!(status.as_str(), "error" | "removed") {
                break Err(value
                    .get("errorMessage")
                    .and_then(Value::as_str)
                    .filter(|message| !message.is_empty())
                    .unwrap_or("torrent_metadata_unavailable")
                    .to_owned());
            }
            if now >= deadline {
                break Err(format!(
                    "torrent_metadata_unavailable:status={status}:connections={peak_connections}:seeders={peak_seeders}:total={peak_total}:completed={peak_completed}"
                ));
            }
            tokio::time::sleep(Duration::from_millis(250)).await;
        };

        if result.is_err() {
            let _ = self.remove(&gid).await;
        }
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
