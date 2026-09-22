use reqwest::Client;
use serde::Deserialize;
use serde_json::{Map, Value};
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
    token: String,
    client: Client,
}

pub struct Runtime {
    child: Child,
    endpoint: Endpoint,
    port: u16,
}

#[derive(Clone, Default)]
pub struct RequestContext {
    pub headers: HashMap<String, String>,
}

/// Mirrors Surge's `DownloadStatus` API shape exactly
/// (see SurgeDM/Surge internal/types/models.go), not a guess.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct DownloadStatus {
    pub id: String,
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub filename: String,
    #[serde(default)]
    pub total_size: u64,
    #[serde(default)]
    pub downloaded: u64,
    #[serde(default)]
    pub progress: f64,
    #[serde(default)]
    pub speed: f64,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub error: String,
    #[serde(default)]
    pub connections: u32,
}

#[derive(Deserialize)]
struct AddResponse {
    id: String,
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
        token: &str,
    ) -> Result<Self, String> {
        if !executable.is_file() {
            return Err("surge_not_found".to_owned());
        }
        fs::create_dir_all(runtime_root).map_err(|error| error.to_string())?;
        let attempts: u32 = if requested_port.is_some() { 1 } else { 4 };
        let mut last_error = String::new();
        for attempt in 0..attempts {
            match Self::spawn_once(executable, runtime_root, requested_port, token) {
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
        token: &str,
    ) -> Result<Self, String> {
        let port = reserve_loopback_port(requested_port)?;
        let log_file = fs::File::create(runtime_root.join("surge.log")).ok();
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
            .arg("server")
            .arg("--port")
            .arg(port.to_string())
            .arg("--token")
            .arg(token)
            .arg("--output")
            .arg(runtime_root)
            .arg("--no-resume")
            .stdin(Stdio::null())
            .stdout(stdout_log)
            .stderr(stderr_log);
        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            command.creation_flags(0x08000000);
        }
        let mut child = command.spawn().map_err(|error| error.to_string())?;
        // Same early-exit detection as aria2's runtime: a port lost to another
        // process between reservation and bind makes the server exit almost
        // immediately, so retry with a fresh port instead of waiting out the
        // full readiness timeout for a process that already died.
        std::thread::sleep(Duration::from_millis(150));
        if let Ok(Some(status)) = child.try_wait() {
            return Err(format!("surge_exited_early:{status}"));
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
                token: token.to_owned(),
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
    fn authorized(&self, builder: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        builder.bearer_auth(&self.token)
    }

    pub async fn wait_ready(&self) -> Result<(), String> {
        let mut last_error = String::new();
        for _ in 0..80 {
            let request = self.authorized(self.client.get(format!("{}/health", self.base_url)));
            match request.send().await {
                Ok(response) if response.status().is_success() => return Ok(()),
                Ok(response) => last_error = format!("http_{}", response.status().as_u16()),
                Err(error) => last_error = error.to_string(),
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        Err(format!("surge_start_timeout:{last_error}"))
    }

    pub async fn add_download(
        &self,
        source: &str,
        destination: &Path,
        context: &RequestContext,
    ) -> Result<String, String> {
        let directory = destination
            .parent()
            .map(|value| value.to_string_lossy().into_owned())
            .unwrap_or_default();
        let file_name = destination
            .file_name()
            .and_then(|value| value.to_str())
            .map(str::to_owned);
        let mut body = Map::new();
        body.insert("url".into(), Value::String(source.to_owned()));
        body.insert("path".into(), Value::String(directory));
        if let Some(name) = file_name {
            body.insert("filename".into(), Value::String(name));
        }
        body.insert("skip_approval".into(), Value::Bool(true));
        if !context.headers.is_empty() {
            body.insert(
                "headers".into(),
                Value::Object(
                    context
                        .headers
                        .iter()
                        .map(|(name, value)| (name.clone(), Value::String(value.clone())))
                        .collect(),
                ),
            );
        }
        let request = self.authorized(
            self.client
                .post(format!("{}/download", self.base_url))
                .json(&Value::Object(body)),
        );
        let response = request
            .send()
            .await
            .map_err(|error| error.to_string())?
            .error_for_status()
            .map_err(|error| error.to_string())?;
        let parsed: AddResponse = response.json().await.map_err(|error| error.to_string())?;
        Ok(parsed.id)
    }

    pub async fn list(&self) -> Result<Vec<DownloadStatus>, String> {
        let request = self.authorized(self.client.get(format!("{}/list", self.base_url)));
        let response = request
            .send()
            .await
            .map_err(|error| error.to_string())?
            .error_for_status()
            .map_err(|error| error.to_string())?;
        response.json().await.map_err(|error| error.to_string())
    }

    pub async fn status(&self, id: &str) -> Result<DownloadStatus, String> {
        self.list()
            .await?
            .into_iter()
            .find(|item| item.id == id)
            .ok_or_else(|| "surge_download_not_found".to_owned())
    }

    async fn action(&self, path: &str, id: &str) -> Result<(), String> {
        let request = self.authorized(
            self.client
                .post(format!("{}/{path}?id={id}", self.base_url)),
        );
        request
            .send()
            .await
            .map_err(|error| error.to_string())?
            .error_for_status()
            .map_err(|error| error.to_string())?;
        Ok(())
    }

    pub async fn pause(&self, id: &str) -> Result<(), String> {
        self.action("pause", id).await
    }

    pub async fn resume(&self, id: &str) -> Result<(), String> {
        self.action("resume", id).await
    }

    pub async fn remove(&self, id: &str) -> Result<(), String> {
        self.action("delete", id).await
    }
}
