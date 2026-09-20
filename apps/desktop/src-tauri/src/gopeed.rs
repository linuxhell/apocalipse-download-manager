use reqwest::{Client, Method};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
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
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeStatus {
    pub status: String,
    #[serde(default)]
    pub used: u64,
    #[serde(default)]
    pub speed: u64,
    #[serde(default)]
    pub downloaded: u64,
    #[serde(default)]
    pub total: u64,
    #[serde(default)]
    pub upload_speed: u64,
    #[serde(default)]
    pub uploaded: u64,
    #[serde(default)]
    pub files: Vec<FileRuntimeStatus>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileRuntimeStatus {
    pub index: usize,
    pub size: u64,
    pub downloaded: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ResolveResult {
    pub id: String,
    pub res: Resource,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Resource {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub size: u64,
    #[serde(default)]
    pub files: Vec<ResourceFile>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ResourceFile {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub path: String,
    #[serde(default)]
    pub size: u64,
}

#[derive(Debug, Clone, Default)]
pub struct StatsSummary {
    pub active_connections: u64,
    pub total_peers: u64,
    pub active_peers: u64,
    pub seeders: u64,
    pub leechers: u64,
}

#[derive(Clone, Default)]
pub struct RequestContext {
    pub method: String,
    pub headers: HashMap<String, String>,
    pub body: String,
}

#[derive(Serialize)]
struct ApiRequest<'a> {
    #[serde(rename = "rawUrl")]
    raw_url: &'a str,
    url: &'a str,
    extra: Value,
}

#[derive(Serialize)]
struct TaskOptions<'a> {
    name: &'a str,
    path: &'a str,
    #[serde(rename = "selectFiles")]
    select_files: &'a [usize],
    extra: Value,
}

#[derive(Deserialize)]
struct Envelope {
    code: i64,
    #[serde(default)]
    msg: String,
    data: Value,
}

fn reserve_loopback_port() -> Result<u16, String> {
    let listener = TcpListener::bind(("127.0.0.1", 0)).map_err(|error| error.to_string())?;
    let port = listener
        .local_addr()
        .map_err(|error| error.to_string())?
        .port();
    drop(listener);
    Ok(port)
}

impl Runtime {
    pub fn spawn(executable: &Path, runtime_root: &Path) -> Result<Self, String> {
        if !executable.is_file() {
            return Err("gopeed_not_found".to_owned());
        }
        let storage = runtime_root.join("storage");
        let temporary = runtime_root.join("temp");
        fs::create_dir_all(&storage).map_err(|error| error.to_string())?;
        fs::create_dir_all(&temporary).map_err(|error| error.to_string())?;
        let port = reserve_loopback_port()?;
        let token = uuid::Uuid::new_v4().simple().to_string();
        let config = runtime_root.join("apocalipse-gopeed.json");
        let mut command = Command::new(executable);
        command
            .arg("--address")
            .arg("127.0.0.1")
            .arg("--port")
            .arg(port.to_string())
            .arg("--api-token")
            .arg(&token)
            .arg("--storage-dir")
            .arg(&storage)
            .arg("--temp-dir")
            .arg(&temporary)
            .arg("--config")
            .arg(&config)
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
                base_url: format!("http://127.0.0.1:{port}"),
                token,
                client,
            },
        })
    }

    pub fn endpoint(&self) -> Endpoint {
        self.endpoint.clone()
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
    async fn request(&self, method: Method, path: &str, body: Option<Value>) -> Result<Value, String> {
        let mut request = self
            .client
            .request(method, format!("{}{}", self.base_url, path))
            .header("X-Api-Token", &self.token);
        if let Some(body) = body {
            request = request.json(&body);
        }
        let response = request
            .send()
            .await
            .map_err(|error| error.to_string())?
            .error_for_status()
            .map_err(|error| error.to_string())?;
        let envelope: Envelope = response.json().await.map_err(|error| error.to_string())?;
        if envelope.code != 0 {
            return Err(if envelope.msg.is_empty() {
                format!("gopeed_api_error:{}", envelope.code)
            } else {
                envelope.msg
            });
        }
        Ok(envelope.data)
    }

    pub async fn wait_ready(&self) -> Result<(), String> {
        let mut last_error = String::new();
        for _ in 0..60 {
            match self.request(Method::GET, "/api/v1/info", None).await {
                Ok(_) => return Ok(()),
                Err(error) => last_error = error,
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        Err(format!("gopeed_start_timeout:{last_error}"))
    }

    pub async fn resolve(
        &self,
        source: &str,
        destination_directory: &str,
        context: &RequestContext,
    ) -> Result<ResolveResult, String> {
        let extra = http_request_extra(context);
        let body = json!({
            "req": ApiRequest {
                raw_url: source,
                url: source,
                extra,
            },
            "opts": {
                "path": destination_directory,
                "selectFiles": []
            }
        });
        let value = self.request(Method::POST, "/api/v1/resolve", Some(body)).await?;
        serde_json::from_value(value).map_err(|error| error.to_string())
    }

    pub async fn create_task(
        &self,
        source: &str,
        destination: &Path,
        connections: usize,
        selected_files: &[usize],
        context: &RequestContext,
        resolved_id: Option<&str>,
        http_download: bool,
    ) -> Result<String, String> {
        let directory = destination
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .to_string_lossy()
            .into_owned();
        let name = if http_download {
            destination
                .file_name()
                .and_then(|value| value.to_str())
                .unwrap_or("download")
        } else {
            ""
        };
        let extra = if http_download {
            json!({ "connections": connections.clamp(1, 32) })
        } else {
            Value::Null
        };
        let opts = TaskOptions {
            name,
            path: &directory,
            select_files: selected_files,
            extra,
        };
        let body = if let Some(rid) = resolved_id {
            json!({ "rid": rid, "opts": opts })
        } else {
            json!({
                "req": ApiRequest {
                    raw_url: source,
                    url: source,
                    extra: http_request_extra(context),
                },
                "opts": opts
            })
        };
        let value = self.request(Method::POST, "/api/v1/tasks", Some(body)).await?;
        value
            .as_str()
            .map(str::to_owned)
            .ok_or_else(|| "gopeed_task_id_missing".to_owned())
    }

    pub async fn status(&self, task_id: &str) -> Result<RuntimeStatus, String> {
        let value = self
            .request(
                Method::GET,
                &format!("/api/v1/tasks/{task_id}/status"),
                None,
            )
            .await?;
        serde_json::from_value(value).map_err(|error| error.to_string())
    }

    pub async fn stats(&self, task_id: &str) -> Result<StatsSummary, String> {
        let value = self
            .request(
                Method::GET,
                &format!("/api/v1/tasks/{task_id}/stats"),
                None,
            )
            .await?;
        let snapshot = value.get("snapshot").unwrap_or(&Value::Null);
        let runtime = value.get("runtime").unwrap_or(&Value::Null);
        let active_connections = snapshot
            .get("connections")
            .and_then(Value::as_array)
            .map(|connections| {
                connections
                    .iter()
                    .filter(|connection| {
                        !connection
                            .get("completed")
                            .and_then(Value::as_bool)
                            .unwrap_or(false)
                            && !connection
                                .get("failed")
                                .and_then(Value::as_bool)
                                .unwrap_or(false)
                    })
                    .count() as u64
            })
            .unwrap_or(0);
        Ok(StatsSummary {
            active_connections,
            total_peers: runtime
                .get("totalPeers")
                .and_then(Value::as_u64)
                .unwrap_or(0),
            active_peers: runtime
                .get("activePeers")
                .and_then(Value::as_u64)
                .unwrap_or(0),
            seeders: runtime
                .get("connectedSeeders")
                .and_then(Value::as_u64)
                .unwrap_or(0),
            leechers: runtime
                .get("connectedLeechers")
                .and_then(Value::as_u64)
                .unwrap_or(0),
        })
    }

    pub async fn pause(&self, task_id: &str) -> Result<(), String> {
        self.request(
            Method::PUT,
            &format!("/api/v1/tasks/{task_id}/pause"),
            Some(json!({})),
        )
        .await
        .map(|_| ())
    }

    pub async fn resume(&self, task_id: &str) -> Result<(), String> {
        self.request(
            Method::PUT,
            &format!("/api/v1/tasks/{task_id}/continue"),
            Some(json!({})),
        )
        .await
        .map(|_| ())
    }

    pub async fn delete(&self, task_id: &str, delete_files: bool) -> Result<(), String> {
        self.request(
            Method::DELETE,
            &format!(
                "/api/v1/tasks/{task_id}?force={}",
                if delete_files { "true" } else { "false" }
            ),
            None,
        )
        .await
        .map(|_| ())
    }
}

fn http_request_extra(context: &RequestContext) -> Value {
    json!({
        "method": if context.method.trim().is_empty() { "GET" } else { context.method.trim() },
        "header": context.headers,
        "body": context.body,
    })
}
