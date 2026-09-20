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
}

#[derive(Debug, Clone, Default)]
pub struct RuntimeStatus {
    pub status: String,
    pub downloaded: u64,
    pub total: u64,
    pub upload_speed: u64,
    pub uploaded: u64,
    pub speed: u64,
    pub connections: u64,
    pub seeders: u64,
}

#[derive(Debug, Clone, Default)]
pub struct PeerSummary {
    pub peers: u64,
    pub seeders: u64,
    pub leechers: u64,
}

#[derive(Debug, Clone, Default)]
pub struct FileStatus {
    pub index: usize,
    pub path: String,
    pub size: u64,
    pub completed: u64,
    pub selected: bool,
}

fn number(value: Option<&Value>) -> u64 {
    value
        .and_then(|value| value.as_str().and_then(|text| text.parse::<u64>().ok()).or_else(|| value.as_u64()))
        .unwrap_or(0)
}

fn reserve_loopback_port(requested: Option<u16>) -> Result<u16, String> {
    if let Some(port) = requested.filter(|port| *port > 0) {
        let listener = TcpListener::bind(("127.0.0.1", port)).map_err(|error| error.to_string())?;
        drop(listener);
        return Ok(port);
    }
    let listener = TcpListener::bind(("127.0.0.1", 0)).map_err(|error| error.to_string())?;
    let port = listener.local_addr().map_err(|error| error.to_string())?.port();
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
        let port = reserve_loopback_port(requested_port)?;
        let session = runtime_root.join("aria2.session");
        if !session.exists() {
            fs::write(&session, b"").map_err(|error| error.to_string())?;
        }
        let mut command = Command::new(executable);
        command
            .arg("--enable-rpc=true")
            .arg("--rpc-listen-all=false")
            .arg("--rpc-allow-origin-all=false")
            .arg(format!("--rpc-listen-port={port}"))
            .arg(format!("--rpc-secret={secret}"))
            .arg("--continue=true")
            .arg("--file-allocation=none")
            .arg("--auto-file-renaming=false")
            .arg("--allow-overwrite=true")
            .arg("--max-concurrent-downloads=20")
            .arg("--summary-interval=0")
            .arg("--console-log-level=warn")
            .arg("--download-result=hide")
            .arg(format!("--input-file={}", session.display()))
            .arg(format!("--save-session={}", session.display()))
            .arg("--save-session-interval=30")
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            command.creation_flags(0x08000000);
        }
        let child = command.spawn().map_err(|error| error.to_string())?;
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
            let code = error.get("code").and_then(Value::as_i64).unwrap_or_default();
            let message = error.get("message").and_then(Value::as_str).unwrap_or("aria2_rpc_error");
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
        selected_files: &[usize],
        context: &RequestContext,
        http_download: bool,
    ) -> Result<String, String> {
        if !context.body.is_empty() || (!context.method.is_empty() && !context.method.eq_ignore_ascii_case("GET")) {
            return Err("aria2_request_requires_native_http".to_owned());
        }
        let directory = destination.parent().unwrap_or_else(|| Path::new("."));
        let mut options = Map::new();
        options.insert("dir".into(), Value::String(directory.to_string_lossy().into_owned()));
        options.insert("continue".into(), Value::String("true".into()));
        options.insert("file-allocation".into(), Value::String("none".into()));
        if http_download {
            let connections = connections.clamp(1, 32);
            options.insert("max-connection-per-server".into(), Value::String(connections.to_string()));
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
        if !selected_files.is_empty() {
            options.insert(
                "select-file".into(),
                Value::String(
                    selected_files
                        .iter()
                        .map(|index| index.to_string())
                        .collect::<Vec<_>>()
                        .join(","),
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
        if source.to_ascii_lowercase().ends_with(".torrent") {
            options.insert("follow-torrent".into(), Value::String("true".into()));
        }
        let local_torrent = PathBuf::from(source);
        let value = if local_torrent.is_file()
            && local_torrent
                .extension()
                .and_then(|value| value.to_str())
                .is_some_and(|value| value.eq_ignore_ascii_case("torrent"))
        {
            let bytes = fs::read(local_torrent).map_err(|error| error.to_string())?;
            let encoded = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, bytes);
            self.call(
                "aria2.addTorrent",
                vec![Value::String(encoded), Value::Array(Vec::new()), Value::Object(options)],
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

    pub async fn add_metadata_only(&self, source: &str, directory: &Path) -> Result<String, String> {
        let mut options = Map::new();
        options.insert("dir".into(), Value::String(directory.to_string_lossy().into_owned()));
        options.insert("bt-metadata-only".into(), Value::String("true".into()));
        options.insert("bt-save-metadata".into(), Value::String("false".into()));
        options.insert("file-allocation".into(), Value::String("none".into()));
        let value = self
            .call("aria2.addUri", vec![json!([source]), Value::Object(options)])
            .await?;
        value
            .as_str()
            .map(str::to_owned)
            .ok_or_else(|| "aria2_gid_missing".to_owned())
    }

    pub async fn status(&self, gid: &str) -> Result<RuntimeStatus, String> {
        let keys = json!([
            "status", "totalLength", "completedLength", "uploadLength",
            "downloadSpeed", "uploadSpeed", "connections", "numSeeders"
        ]);
        let value = self
            .call("aria2.tellStatus", vec![Value::String(gid.to_owned()), keys])
            .await?;
        Ok(RuntimeStatus {
            status: value.get("status").and_then(Value::as_str).unwrap_or_default().to_owned(),
            total: number(value.get("totalLength")),
            downloaded: number(value.get("completedLength")),
            uploaded: number(value.get("uploadLength")),
            speed: number(value.get("downloadSpeed")),
            upload_speed: number(value.get("uploadSpeed")),
            connections: number(value.get("connections")),
            seeders: number(value.get("numSeeders")),
        })
    }

    pub async fn peers(&self, gid: &str) -> Result<PeerSummary, String> {
        let value = self.call("aria2.getPeers", vec![Value::String(gid.to_owned())]).await?;
        let peers = value.as_array().cloned().unwrap_or_default();
        let seeders = peers
            .iter()
            .filter(|peer| {
                peer.get("seeder")
                    .and_then(|value| value.as_str().map(|text| text == "true").or_else(|| value.as_bool()))
                    .unwrap_or(false)
            })
            .count() as u64;
        Ok(PeerSummary {
            peers: peers.len() as u64,
            seeders,
            leechers: (peers.len() as u64).saturating_sub(seeders),
        })
    }

    pub async fn files(&self, gid: &str) -> Result<Vec<FileStatus>, String> {
        let value = self.call("aria2.getFiles", vec![Value::String(gid.to_owned())]).await?;
        Ok(value
            .as_array()
            .map(|files| {
                files
                    .iter()
                    .filter_map(|file| {
                        let index = file.get("index")?.as_str()?.parse::<usize>().ok()?;
                        Some(FileStatus {
                            index,
                            path: file.get("path").and_then(Value::as_str).unwrap_or_default().to_owned(),
                            size: number(file.get("length")),
                            completed: number(file.get("completedLength")),
                            selected: file
                                .get("selected")
                                .and_then(Value::as_str)
                                .map(|value| value == "true")
                                .unwrap_or(true),
                        })
                    })
                    .collect()
            })
            .unwrap_or_default())
    }

    pub async fn pause(&self, gid: &str) -> Result<(), String> {
        self.call("aria2.pause", vec![Value::String(gid.to_owned())]).await.map(|_| ())
    }

    pub async fn resume(&self, gid: &str) -> Result<(), String> {
        self.call("aria2.unpause", vec![Value::String(gid.to_owned())]).await.map(|_| ())
    }

    pub async fn remove(&self, gid: &str) -> Result<(), String> {
        match self.call("aria2.remove", vec![Value::String(gid.to_owned())]).await {
            Ok(_) => Ok(()),
            Err(_) => self
                .call("aria2.forceRemove", vec![Value::String(gid.to_owned())])
                .await
                .map(|_| ()),
        }
    }

    pub async fn remove_result(&self, gid: &str) -> Result<(), String> {
        self.call("aria2.removeDownloadResult", vec![Value::String(gid.to_owned())])
            .await
            .map(|_| ())
    }
}
