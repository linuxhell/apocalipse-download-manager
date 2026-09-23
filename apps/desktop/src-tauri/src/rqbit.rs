use reqwest::Client;
use serde_json::Value;
use std::{
    fs,
    net::TcpListener,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    time::Duration,
};

#[derive(Clone)]
pub struct Endpoint {
    base_url: String,
    username: String,
    password: String,
    client: Client,
}

pub struct Runtime {
    child: Child,
    endpoint: Endpoint,
    port: u16,
}

#[derive(Debug, Clone, Default)]
pub struct AddedTorrent {
    pub id: usize,
    pub info_hash: String,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct TorrentFile {
    pub index: usize,
    pub name: String,
    pub length: u64,
}

#[derive(Debug, Clone, Default)]
pub struct TorrentStatus {
    pub state: String,
    pub finished: bool,
    pub error: Option<String>,
    pub progress_bytes: u64,
    pub total_bytes: u64,
    pub download_bytes_per_second: u64,
    pub upload_bytes_per_second: u64,
    pub eta_seconds: Option<u64>,
    pub live_peers: u64,
}

fn number(value: Option<&Value>) -> u64 {
    value.and_then(Value::as_u64).unwrap_or(0)
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
            return Err("rqbit_not_found".to_owned());
        }
        fs::create_dir_all(runtime_root).map_err(|error| error.to_string())?;
        // Same loopback-port-race retry pattern as aria2::Runtime::spawn: the
        // port can be grabbed by another process between reservation and the
        // child actually binding it, which makes rqbit exit almost
        // immediately. Retry with a fresh port instead of waiting out the
        // full readiness timeout for a process that already died.
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
        let default_download_dir = runtime_root.join("downloads");
        fs::create_dir_all(&default_download_dir).map_err(|error| error.to_string())?;
        let mut command = Command::new(executable);
        command
            .arg("server")
            .arg("start")
            .arg(&default_download_dir)
            // --http-api-listen-addr is a global flag on rqbit's top-level
            // Opts, declared before the "server"/"download"/"share"
            // subcommand, so it cannot be placed after "server start
            // <dir>" on the command line. Set it via its documented env
            // var instead, which the same Opts field also reads.
            .env("RQBIT_HTTP_API_LISTEN_ADDR", format!("127.0.0.1:{port}"))
            .env(
                "RQBIT_HTTP_BASIC_AUTH_USERPASS",
                format!("{username}:{password}"),
            )
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            command.creation_flags(0x08000000);
        }
        let mut child = command.spawn().map_err(|error| error.to_string())?;
        std::thread::sleep(Duration::from_millis(150));
        if let Ok(Some(status)) = child.try_wait() {
            return Err(format!("rqbit_exited_early:{status}"));
        }
        let client = Client::builder()
            .connect_timeout(Duration::from_secs(2))
            .timeout(Duration::from_secs(15))
            .build()
            .map_err(|error| error.to_string())?;
        Ok(Self {
            child,
            endpoint: Endpoint {
                base_url: format!("http://127.0.0.1:{port}"),
                username: username.to_owned(),
                password: password.to_owned(),
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
    fn request(&self, method: reqwest::Method, path: &str) -> reqwest::RequestBuilder {
        self.client
            .request(method, format!("{}{}", self.base_url, path))
            .basic_auth(&self.username, Some(&self.password))
    }

    pub async fn wait_ready(&self) -> Result<(), String> {
        let mut last_error = String::new();
        for _ in 0..80 {
            match self.request(reqwest::Method::GET, "/torrents").send().await {
                Ok(response) if response.status().is_success() => return Ok(()),
                Ok(response) => last_error = format!("http_{}", response.status().as_u16()),
                Err(error) => last_error = error.to_string(),
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        Err(format!("rqbit_start_timeout:{last_error}"))
    }

    pub async fn version(&self) -> Result<String, String> {
        let response = self
            .request(reqwest::Method::GET, "/")
            .send()
            .await
            .map_err(|error| error.to_string())?
            .error_for_status()
            .map_err(|error| error.to_string())?;
        let payload: Value = response.json().await.map_err(|error| error.to_string())?;
        payload
            .get("version")
            .and_then(Value::as_str)
            .map(str::to_owned)
            .ok_or_else(|| "rqbit_version_missing".to_owned())
    }

    pub async fn add_torrent(
        &self,
        source: &str,
        output_folder: &Path,
        only_files: &[usize],
        list_only: bool,
    ) -> Result<AddedTorrent, String> {
        let local = PathBuf::from(source);
        let body = if local.is_file() {
            fs::read(&local).map_err(|error| error.to_string())?
        } else {
            source.as_bytes().to_vec()
        };
        let mut request = self.request(reqwest::Method::POST, "/torrents").query(&[
            (
                "output_folder",
                output_folder.to_string_lossy().into_owned(),
            ),
            ("overwrite", "true".to_owned()),
            ("list_only", list_only.to_string()),
        ]);
        if !only_files.is_empty() {
            let pairs = only_files
                .iter()
                .map(|index| ("only_files", index.to_string()))
                .collect::<Vec<_>>();
            request = request.query(&pairs);
        }
        let response = request
            .header("Content-Type", "application/octet-stream")
            .body(body)
            .send()
            .await
            .map_err(|error| error.to_string())?
            .error_for_status()
            .map_err(|error| error.to_string())?;
        let payload: Value = response.json().await.map_err(|error| error.to_string())?;
        let details = payload.get("details").unwrap_or(&payload);
        Ok(AddedTorrent {
            id: payload
                .get("id")
                .and_then(Value::as_u64)
                .or_else(|| details.get("id").and_then(Value::as_u64))
                .ok_or_else(|| "rqbit_torrent_id_missing".to_owned())? as usize,
            info_hash: details
                .get("info_hash")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_owned(),
            name: details
                .get("name")
                .and_then(Value::as_str)
                .map(str::to_owned),
        })
    }

    pub async fn files(&self, id: usize) -> Result<Vec<TorrentFile>, String> {
        let response = self
            .request(reqwest::Method::GET, &format!("/torrents/{id}"))
            .send()
            .await
            .map_err(|error| error.to_string())?
            .error_for_status()
            .map_err(|error| error.to_string())?;
        let payload: Value = response.json().await.map_err(|error| error.to_string())?;
        Ok(payload
            .get("files")
            .and_then(Value::as_array)
            .map(|files| {
                files
                    .iter()
                    .enumerate()
                    .map(|(index, file)| TorrentFile {
                        index,
                        name: file
                            .get("name")
                            .and_then(Value::as_str)
                            .unwrap_or_default()
                            .to_owned(),
                        length: number(file.get("length")),
                    })
                    .collect()
            })
            .unwrap_or_default())
    }

    pub async fn stats(&self, id: usize) -> Result<TorrentStatus, String> {
        let response = self
            .request(reqwest::Method::GET, &format!("/torrents/{id}/stats/v1"))
            .send()
            .await
            .map_err(|error| error.to_string())?
            .error_for_status()
            .map_err(|error| error.to_string())?;
        let payload: Value = response.json().await.map_err(|error| error.to_string())?;
        let live = payload.get("live").filter(|value| !value.is_null());
        let mbps_to_bytes = |value: Option<&Value>| -> u64 {
            value
                .and_then(|value| value.get("mbps"))
                .and_then(Value::as_f64)
                .map(|mbps| (mbps * 1024.0 * 1024.0).max(0.0) as u64)
                .unwrap_or(0)
        };
        Ok(TorrentStatus {
            state: payload
                .get("state")
                .and_then(Value::as_str)
                .unwrap_or("initializing")
                .to_owned(),
            finished: payload
                .get("finished")
                .and_then(Value::as_bool)
                .unwrap_or(false),
            error: payload
                .get("error")
                .and_then(Value::as_str)
                .map(str::to_owned),
            progress_bytes: number(payload.get("progress_bytes")),
            total_bytes: number(payload.get("total_bytes")),
            download_bytes_per_second: mbps_to_bytes(
                live.and_then(|live| live.get("download_speed")),
            ),
            upload_bytes_per_second: mbps_to_bytes(live.and_then(|live| live.get("upload_speed"))),
            eta_seconds: live
                .and_then(|live| live.get("time_remaining"))
                .filter(|value| !value.is_null())
                .and_then(|value| value.get("duration"))
                .and_then(|value| value.get("secs"))
                .and_then(Value::as_u64),
            live_peers: live
                .and_then(|live| live.get("snapshot"))
                .and_then(|snapshot| snapshot.get("peer_stats"))
                .and_then(|peers| peers.get("live"))
                .and_then(Value::as_u64)
                .unwrap_or(0),
        })
    }

    pub async fn pause(&self, id: usize) -> Result<(), String> {
        self.request(reqwest::Method::POST, &format!("/torrents/{id}/pause"))
            .send()
            .await
            .map_err(|error| error.to_string())?
            .error_for_status()
            .map_err(|error| error.to_string())?;
        Ok(())
    }

    pub async fn resume(&self, id: usize) -> Result<(), String> {
        self.request(reqwest::Method::POST, &format!("/torrents/{id}/start"))
            .send()
            .await
            .map_err(|error| error.to_string())?
            .error_for_status()
            .map_err(|error| error.to_string())?;
        Ok(())
    }

    /// Stops tracking the torrent but keeps its downloaded files on disk.
    pub async fn forget(&self, id: usize) -> Result<(), String> {
        self.request(reqwest::Method::POST, &format!("/torrents/{id}/forget"))
            .send()
            .await
            .map_err(|error| error.to_string())?
            .error_for_status()
            .map_err(|error| error.to_string())?;
        Ok(())
    }
}
