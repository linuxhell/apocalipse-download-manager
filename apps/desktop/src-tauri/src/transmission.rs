use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use reqwest::Client;
use serde_json::{json, Map, Value};
use std::{
    fs,
    net::TcpListener,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::Mutex,
    time::Duration,
};

/// Direct JSON-RPC client for transmission-daemon, matching
/// docs/rpc-spec.md from transmission/transmission: JSON-RPC 2.0 envelope
/// (`{"method","arguments","tag"}` request, `{"result","arguments","tag"}`
/// response), CSRF via `X-Transmission-Session-Id` (re-sent after a 409),
/// and HTTP Basic auth. This talks to the daemon's RPC endpoint directly,
/// the same way aria2.rs talks to aria2c's RPC — transmission-remote is not
/// spawned as a subprocess.
#[derive(Clone)]
pub struct Endpoint {
    base_url: String,
    username: String,
    password: String,
    client: Client,
    session_id: std::sync::Arc<Mutex<String>>,
}

pub struct Runtime {
    child: Child,
    endpoint: Endpoint,
    port: u16,
}

#[derive(Debug, Clone, Default)]
pub struct TorrentStatus {
    pub hash: String,
    pub name: String,
    pub total_size: u64,
    pub left_until_done: u64,
    pub percent_done: f64,
    pub rate_download: u64,
    pub rate_upload: u64,
    pub peers_connected: u32,
    pub eta: i64,
    /// Raw `tr_torrent_activity` value: 0=stopped, 1=check-wait, 2=check,
    /// 3=download-wait, 4=downloading, 5=seed-wait, 6=seeding.
    pub activity: u8,
    pub error: i64,
    pub error_string: String,
}

impl TorrentStatus {
    pub fn downloaded(&self) -> u64 {
        self.total_size.saturating_sub(self.left_until_done)
    }
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
        username: &str,
        password: &str,
    ) -> Result<Self, String> {
        if !executable.is_file() {
            return Err("transmission_not_found".to_owned());
        }
        fs::create_dir_all(runtime_root).map_err(|error| error.to_string())?;
        let attempts = if requested_port.is_some() { 1 } else { 4 };
        let mut last_error = String::new();
        for attempt in 0..attempts {
            match Self::spawn_once(executable, runtime_root, requested_port, username, password) {
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
        username: &str,
        password: &str,
    ) -> Result<Self, String> {
        let port = reserve_loopback_port(requested_port)?;
        let config_dir = runtime_root.join("config");
        let download_dir = runtime_root.join("downloads");
        fs::create_dir_all(&config_dir).map_err(|error| error.to_string())?;
        fs::create_dir_all(&download_dir).map_err(|error| error.to_string())?;
        let log_file = fs::File::create(runtime_root.join("transmission-daemon.log")).ok();
        let stdout_log = log_file
            .as_ref()
            .and_then(|file| file.try_clone().ok())
            .map(Stdio::from)
            .unwrap_or_else(Stdio::null);
        let stderr_log = log_file
            .and_then(|file| file.try_clone().ok())
            .map(Stdio::from)
            .unwrap_or_else(Stdio::null);
        let mut command = Command::new(executable);
        command
            .arg("--foreground")
            .arg("--config-dir")
            .arg(&config_dir)
            .arg("--port")
            .arg(port.to_string())
            .arg("--rpc-bind-address")
            .arg("127.0.0.1")
            .arg("--allowed")
            .arg("127.0.0.1,::1")
            .arg("--auth")
            .arg("--username")
            .arg(username)
            .arg("--password")
            .arg(password)
            .arg("--download-dir")
            .arg(&download_dir)
            .arg("--no-watch-dir")
            .arg("--dht")
            .arg("--lpd")
            .arg("--portmap")
            .stdin(Stdio::null())
            .stdout(stdout_log)
            .stderr(stderr_log);
        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            command.creation_flags(0x08000000);
        }
        let mut child = command.spawn().map_err(|error| error.to_string())?;
        // Same early-exit detection used by aria2/surge: a lost port-reservation
        // race makes the daemon exit almost immediately.
        std::thread::sleep(Duration::from_millis(200));
        if let Ok(Some(status)) = child.try_wait() {
            return Err(format!("transmission_exited_early:{status}"));
        }
        let client = Client::builder()
            .connect_timeout(Duration::from_secs(2))
            .timeout(Duration::from_secs(15))
            .build()
            .map_err(|error| error.to_string())?;
        Ok(Self {
            child,
            endpoint: Endpoint {
                base_url: format!("http://127.0.0.1:{port}/transmission/rpc"),
                username: username.to_owned(),
                password: password.to_owned(),
                client,
                session_id: std::sync::Arc::new(Mutex::new(String::new())),
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

fn status_from_torrent_get(value: &Value) -> Option<TorrentStatus> {
    let object = value.as_object()?;
    let number = |key: &str| object.get(key).and_then(Value::as_f64).unwrap_or(0.0);
    Some(TorrentStatus {
        hash: object.get("hash_string")?.as_str()?.to_owned(),
        name: object
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned(),
        total_size: number("total_size") as u64,
        left_until_done: number("left_until_done") as u64,
        percent_done: number("percent_done"),
        rate_download: number("rate_download") as u64,
        rate_upload: number("rate_upload") as u64,
        peers_connected: number("peers_connected") as u32,
        eta: number("eta") as i64,
        activity: number("status") as u8,
        error: number("error") as i64,
        error_string: object
            .get("error_string")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned(),
    })
}

impl Endpoint {
    async fn call(&self, method: &str, arguments: Value) -> Result<Value, String> {
        let body = json!({
            "method": method,
            "arguments": arguments,
        });
        for attempt in 0..2 {
            let session_id = self.session_id.lock().map_or_else(|_| String::new(), |value| value.clone());
            let response = self
                .client
                .post(&self.base_url)
                .basic_auth(&self.username, Some(&self.password))
                .header("X-Transmission-Session-Id", session_id)
                .json(&body)
                .send()
                .await
                .map_err(|error| error.to_string())?;
            if response.status() == reqwest::StatusCode::CONFLICT {
                if let Some(next_id) = response
                    .headers()
                    .get("X-Transmission-Session-Id")
                    .and_then(|value| value.to_str().ok())
                {
                    if let Ok(mut current) = self.session_id.lock() {
                        *current = next_id.to_owned();
                    }
                }
                if attempt == 0 {
                    continue;
                }
                return Err("transmission_session_id_retry_failed".to_owned());
            }
            let response = response
                .error_for_status()
                .map_err(|error| error.to_string())?;
            let payload: Value = response.json().await.map_err(|error| error.to_string())?;
            if let Some(error) = payload.get("error") {
                let message = error
                    .get("message")
                    .and_then(Value::as_str)
                    .unwrap_or("transmission_rpc_error");
                return Err(format!("transmission_rpc_error:{message}"));
            }
            return Ok(payload
                .get("result")
                .cloned()
                .unwrap_or(Value::Object(Map::new())));
        }
        Err("transmission_call_failed".to_owned())
    }

    pub async fn wait_ready(&self) -> Result<(), String> {
        let mut last_error = String::new();
        for _ in 0..80 {
            match self.call("session_get", json!({"fields": ["version"]})).await {
                Ok(_) => return Ok(()),
                Err(error) => last_error = error,
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        Err(format!("transmission_start_timeout:{last_error}"))
    }

    /// Adds a magnet link or `.torrent` file/URL. `cookie_header`, if given,
    /// is passed through as-is (its `name=value; name2=value2;` shape already
    /// matches the RPC `cookies` field's expected format).
    pub async fn add_download(
        &self,
        source: &str,
        download_dir: &Path,
        cookie_header: Option<&str>,
        sequential: bool,
    ) -> Result<String, String> {
        self.add_download_with_options(source, download_dir, cookie_header, sequential, false)
            .await
    }

    /// Same as `add_download`, but lets the caller add the torrent paused —
    /// used by the file-list preview flow, which only needs metadata, not an
    /// actual download to start.
    pub async fn add_download_with_options(
        &self,
        source: &str,
        download_dir: &Path,
        cookie_header: Option<&str>,
        sequential: bool,
        paused: bool,
    ) -> Result<String, String> {
        let mut arguments = Map::new();
        let local_torrent = PathBuf::from(source);
        if local_torrent.is_file()
            && local_torrent
                .extension()
                .and_then(|value| value.to_str())
                .is_some_and(|value| value.eq_ignore_ascii_case("torrent"))
        {
            let bytes = fs::read(&local_torrent).map_err(|error| error.to_string())?;
            arguments.insert("metainfo".into(), Value::String(BASE64.encode(bytes)));
        } else {
            arguments.insert("filename".into(), Value::String(source.to_owned()));
        }
        arguments.insert(
            "download_dir".into(),
            Value::String(download_dir.to_string_lossy().into_owned()),
        );
        arguments.insert("sequential_download".into(), Value::Bool(sequential));
        arguments.insert("paused".into(), Value::Bool(paused));
        if let Some(cookie) = cookie_header.filter(|value| !value.is_empty()) {
            arguments.insert("cookies".into(), Value::String(cookie.to_owned()));
        }
        let result = self.call("torrent_add", Value::Object(arguments)).await?;
        let torrent = result
            .get("torrent_added")
            .or_else(|| result.get("torrent_duplicate"))
            .ok_or_else(|| "transmission_add_missing_torrent".to_owned())?;
        torrent
            .get("hash_string")
            .and_then(Value::as_str)
            .map(str::to_owned)
            .ok_or_else(|| "transmission_add_missing_hash".to_owned())
    }

    /// File list for a torrent (name + total length per file, in `files`
    /// array order, which is also the file index used elsewhere in the RPC).
    /// Empty until the torrent's metadata (from the magnet's DHT/peer
    /// handshake) has actually arrived.
    pub async fn files(&self, hash: &str) -> Result<Vec<(String, u64)>, String> {
        let result = self
            .call(
                "torrent_get",
                json!({ "ids": [hash], "fields": ["files"] }),
            )
            .await?;
        let files = result
            .get("torrents")
            .and_then(Value::as_array)
            .and_then(|torrents| torrents.first())
            .and_then(|torrent| torrent.get("files"))
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        Ok(files
            .into_iter()
            .filter_map(|file| {
                let name = file.get("name")?.as_str()?.to_owned();
                let length = file.get("length")?.as_u64()?;
                Some((name, length))
            })
            .collect())
    }

    const STATUS_FIELDS: &'static [&'static str] = &[
        "hash_string",
        "name",
        "total_size",
        "left_until_done",
        "percent_done",
        "rate_download",
        "rate_upload",
        "peers_connected",
        "eta",
        "status",
        "error",
        "error_string",
    ];

    pub async fn status(&self, hash: &str) -> Result<TorrentStatus, String> {
        let result = self
            .call(
                "torrent_get",
                json!({ "ids": [hash], "fields": Self::STATUS_FIELDS }),
            )
            .await?;
        result
            .get("torrents")
            .and_then(Value::as_array)
            .and_then(|torrents| torrents.first())
            .and_then(status_from_torrent_get)
            .ok_or_else(|| "transmission_status_missing".to_owned())
    }

    pub async fn pause(&self, hash: &str) -> Result<(), String> {
        self.call("torrent_stop", json!({ "ids": [hash] }))
            .await
            .map(|_| ())
    }

    pub async fn resume(&self, hash: &str) -> Result<(), String> {
        self.call("torrent_start", json!({ "ids": [hash] }))
            .await
            .map(|_| ())
    }

    pub async fn remove(&self, hash: &str, delete_local_data: bool) -> Result<(), String> {
        self.call(
            "torrent_remove",
            json!({ "ids": [hash], "delete_local_data": delete_local_data }),
        )
        .await
        .map(|_| ())
    }
}
