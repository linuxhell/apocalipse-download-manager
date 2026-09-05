use anyhow::{bail, Context, Result};
use futures_util::{stream::FuturesUnordered, StreamExt};
use hickory_resolver::{
    config::{NameServerConfigGroup, ResolverConfig},
    name_server::TokioConnectionProvider,
    TokioResolver,
};
use reqwest::{
    dns::{Addrs, Name, Resolve, Resolving},
    header, Client, RequestBuilder, StatusCode,
};
use sha2::{Digest, Sha256};
use std::{
    net::{IpAddr, SocketAddr},
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicU64, AtomicUsize, Ordering},
        Arc,
    },
    time::Duration,
};
use tokio::{
    fs,
    io::{AsyncReadExt, AsyncWriteExt, BufWriter},
    sync::mpsc,
};

use crate::validation::{validate_payload, PayloadExpectation};

const SEGMENT_CHUNK_SIZE: u64 = 16 * 1024 * 1024;

#[derive(Debug, Clone)]
pub struct DownloadRequest {
    pub url: String,
    pub destination: PathBuf,
    pub overwrite: bool,
    pub connections: usize,
    pub method: String,
    pub body: Option<Vec<u8>>,
    pub headers: Vec<(String, String)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DownloadEvent {
    Started { resumed_at: u64, total: Option<u64>, connections: usize },
    Progress { received: u64, total: Option<u64> },
    Completed { bytes: u64 },
}

#[derive(Clone)]
pub struct DownloadEngine {
    client: Client,
}

#[derive(Clone)]
struct CustomDnsResolver {
    resolver: TokioResolver,
}

impl Resolve for CustomDnsResolver {
    fn resolve(&self, name: Name) -> Resolving {
        let resolver = self.resolver.clone();
        Box::pin(async move {
            let lookup = resolver.lookup_ip(name.as_str()).await?;
            let addrs: Addrs = Box::new(
                lookup
                    .into_iter()
                    .map(|address| SocketAddr::new(address, 0)),
            );
            Ok(addrs)
        })
    }
}

impl DownloadEngine {
    pub fn new() -> Result<Self> {
        Self::with_network(None, None, None, &[])
    }

    pub fn with_proxy(
        proxy_url: Option<&str>,
        username: Option<&str>,
        password: Option<&str>,
    ) -> Result<Self> {
        Self::with_network(proxy_url, username, password, &[])
    }

    pub fn with_network(
        proxy_url: Option<&str>,
        username: Option<&str>,
        password: Option<&str>,
        dns_servers: &[IpAddr],
    ) -> Result<Self> {
        let mut builder = Client::builder()
            .connect_timeout(Duration::from_secs(15))
            .read_timeout(Duration::from_secs(60))
            .redirect(reqwest::redirect::Policy::limited(10))
            .pool_max_idle_per_host(32)
            .tcp_nodelay(true)
            .user_agent(concat!("ApocalipseDownloadManager/", env!("CARGO_PKG_VERSION")));
        if let Some(url) = proxy_url.map(str::trim).filter(|value| !value.is_empty()) {
            let mut proxy = reqwest::Proxy::all(url).context("invalid proxy URL")?;
            if let Some(user) = username.filter(|value| !value.is_empty()) {
                proxy = proxy.basic_auth(user, password.unwrap_or_default());
            }
            builder = builder.proxy(proxy);
        }
        if !dns_servers.is_empty() {
            let name_servers = NameServerConfigGroup::from_ips_clear(dns_servers, 53, true);
            let config = ResolverConfig::from_parts(None, Vec::new(), name_servers);
            let resolver =
                TokioResolver::builder_with_config(config, TokioConnectionProvider::default())
                    .build();
            builder = builder.dns_resolver(Arc::new(CustomDnsResolver { resolver }));
        }
        let client = builder.build()?;
        Ok(Self { client })
    }

    pub async fn download(
        &self,
        request: DownloadRequest,
        events: mpsc::Sender<DownloadEvent>,
    ) -> Result<()> {
        if request.destination.exists() && !request.overwrite {
            bail!("destination already exists");
        }
        if let Some(parent) = request.destination.parent() {
            fs::create_dir_all(parent).await?;
        }
        let can_segment = request.method.eq_ignore_ascii_case("GET") && request.body.is_none();
        let head = if can_segment {
            apply_headers(self.client.head(&request.url), &request.headers)
                .send()
                .await
                .ok()
        } else {
            None
        };
        let total = head.as_ref().and_then(|response| response.content_length());
        let requested = request.connections.clamp(1, 32);
        if can_segment && requested > 1 {
            let probe = apply_headers(self.client.get(&request.url), &request.headers)
                .header(header::RANGE, "bytes=0-0")
                .send()
                .await;
            if let Ok(probe) = probe {
                if probe.status() == StatusCode::PARTIAL_CONTENT {
                    let range_total = probe
                        .headers()
                        .get(header::CONTENT_RANGE)
                        .and_then(|value| value.to_str().ok())
                        .and_then(content_range_total)
                        .or(total);
                    if let Some(total) = range_total {
                        let useful_connections =
                            requested.min(total.div_ceil(4_194_304) as usize);
                        if useful_connections > 1 {
                            return self
                                .download_segmented(request, events, total, useful_connections)
                                .await;
                        }
                    }
                }
            }
        }
        self.download_single(request, events).await
    }

    async fn download_single(
        &self,
        request: DownloadRequest,
        events: mpsc::Sender<DownloadEvent>,
    ) -> Result<()> {
        let partial = partial_path(&request.destination);
        let existing = fs::metadata(&partial)
            .await
            .map(|metadata| metadata.len())
            .unwrap_or(0);
        let method = reqwest::Method::from_bytes(request.method.as_bytes())
            .context("invalid HTTP method")?;
        let mut builder = self.client.request(method, &request.url);
        for (name, value) in &request.headers {
            builder = builder.header(name.as_str(), value.as_str());
        }
        if let Some(body) = &request.body {
            builder = builder.body(body.clone());
        }
        if existing > 0 && request.method.eq_ignore_ascii_case("GET") && request.body.is_none() {
            builder = builder.header(header::RANGE, format!("bytes={existing}-"));
        }
        let response = builder.send().await?.error_for_status()?;
        let resumed = existing > 0 && response.status() == StatusCode::PARTIAL_CONTENT;
        let start = if resumed { existing } else { 0 };
        let total = response.content_length().map(|size| size + start);
        let content_type = response
            .headers()
            .get(header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .map(str::to_owned);
        let expectation = payload_expectation(&request.destination);
        let _ = events
            .send(DownloadEvent::Started {
                resumed_at: start,
                total,
                connections: 1,
            })
            .await;
        let file = if resumed {
            fs::OpenOptions::new().append(true).open(&partial).await?
        } else {
            fs::File::create(&partial).await?
        };
        let mut file = BufWriter::with_capacity(1024 * 1024, file);
        let mut received = start;
        let mut stream = response.bytes_stream();
        let mut inspected = resumed;
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.context("network stream failed")?;
            if !inspected {
                validate_payload(expectation, content_type.as_deref(), &chunk)?;
                inspected = true;
            }
            file.write_all(&chunk).await?;
            received += chunk.len() as u64;
            let _ = events.try_send(DownloadEvent::Progress { received, total });
        }
        file.flush().await?;
        finish_download(&request, &partial, received, total, &events).await
    }

    async fn download_segmented(
        &self,
        request: DownloadRequest,
        events: mpsc::Sender<DownloadEvent>,
        total: u64,
        connections: usize,
    ) -> Result<()> {
        let progress = Arc::new(AtomicU64::new(0));
        let chunk_count = total.div_ceil(SEGMENT_CHUNK_SIZE) as usize;
        let worker_count = connections.min(chunk_count);
        let next_chunk = Arc::new(AtomicUsize::new(0));

        fs::create_dir_all(chunk_directory(&request.destination)).await?;

        for index in 0..chunk_count {
            let start = index as u64 * SEGMENT_CHUNK_SIZE;
            let expected = (total - start).min(SEGMENT_CHUNK_SIZE);
            let chunk = chunk_path(&request.destination, index);
            let legacy = legacy_chunk_path(&request.destination, index);
            if fs::metadata(&chunk).await.is_err()
                && fs::metadata(&legacy).await.is_ok()
                && fs::rename(&legacy, &chunk).await.is_err()
            {
                fs::copy(&legacy, &chunk).await?;
                fs::remove_file(&legacy).await?;
            }
            let existing = fs::metadata(&chunk)
                .await
                .map(|metadata| metadata.len())
                .unwrap_or(0);
            if existing <= expected {
                progress.fetch_add(existing, Ordering::Relaxed);
            } else {
                fs::remove_file(chunk).await?;
            }
        }

        let mut jobs = FuturesUnordered::new();
        for _ in 0..worker_count {
            let client = self.client.clone();
            let url = request.url.clone();
            let headers = request.headers.clone();
            let destination = request.destination.clone();
            let sender = events.clone();
            let shared = progress.clone();
            let cursor = next_chunk.clone();
            jobs.push(async move {
                loop {
                    let index = cursor.fetch_add(1, Ordering::Relaxed);
                    if index >= chunk_count {
                        break;
                    }
                    let start = index as u64 * SEGMENT_CHUNK_SIZE;
                    let end = (start + SEGMENT_CHUNK_SIZE).min(total) - 1;
                    let expected = end - start + 1;
                    let segment = chunk_path(&destination, index);
                    let existing = fs::metadata(&segment)
                        .await
                        .map(|metadata| metadata.len())
                        .unwrap_or(0);
                    if existing == expected {
                        continue;
                    }
                    let response = apply_headers(client.get(&url), &headers)
                        .header(header::RANGE, format!("bytes={}-{}", start + existing, end))
                        .send()
                        .await?;
                    if response.status() != StatusCode::PARTIAL_CONTENT {
                        bail!("server stopped supporting byte ranges");
                    }
                    let file = if existing > 0 {
                        fs::OpenOptions::new().append(true).open(&segment).await?
                    } else {
                        fs::File::create(&segment).await?
                    };
                    let mut file = BufWriter::with_capacity(1024 * 1024, file);
                    let mut downloaded = existing;
                    let mut stream = response.bytes_stream();
                    while let Some(chunk) = stream.next().await {
                        let chunk = chunk.context("segmented network stream failed")?;
                        file.write_all(&chunk).await?;
                        downloaded += chunk.len() as u64;
                        let received = shared.fetch_add(chunk.len() as u64, Ordering::Relaxed)
                            + chunk.len() as u64;
                        let _ = sender.try_send(DownloadEvent::Progress {
                            received,
                            total: Some(total),
                        });
                    }
                    file.flush().await?;
                    if downloaded != expected {
                        bail!("incomplete segment: received {downloaded} of {expected} bytes");
                    }
                }
                Result::<()>::Ok(())
            });
        }
        let resumed = progress.load(Ordering::Relaxed);
        let _ = events
            .send(DownloadEvent::Started {
                resumed_at: resumed,
                total: Some(total),
                connections: worker_count,
            })
            .await;
        while let Some(result) = jobs.next().await {
            result?;
        }
        let partial = partial_path(&request.destination);
        let mut output = fs::File::create(&partial).await?;
        let mut buffer = vec![0_u8; 4 * 1024 * 1024];
        for index in 0..chunk_count {
            let segment = chunk_path(&request.destination, index);
            let mut input = fs::File::open(&segment).await?;
            loop {
                let count = input.read(&mut buffer).await?;
                if count == 0 {
                    break;
                }
                output.write_all(&buffer[..count]).await?;
            }
            fs::remove_file(segment).await?;
        }
        output.flush().await?;
        let _ = cleanup_chunk_artifacts(&request.destination).await;
        finish_download(&request, &partial, total, Some(total), &events).await
    }
}

fn apply_headers(mut builder: RequestBuilder, headers: &[(String, String)]) -> RequestBuilder {
    for (name, value) in headers {
        if ["range", "content-length", "connection", "host"]
            .iter()
            .any(|blocked| name.eq_ignore_ascii_case(blocked))
        {
            continue;
        }
        builder = builder.header(name.as_str(), value.as_str());
    }
    builder
}

fn content_range_total(value: &str) -> Option<u64> {
    value.rsplit_once('/')?.1.trim().parse().ok()
}

fn payload_expectation(destination: &Path) -> PayloadExpectation {
    match destination
        .extension()
        .and_then(|value| value.to_str())
        .map(str::to_ascii_lowercase)
        .as_deref()
    {
        Some("zip") => PayloadExpectation::Zip,
        Some(_) => PayloadExpectation::Binary,
        None => PayloadExpectation::Any,
    }
}

async fn finish_download(
    request: &DownloadRequest,
    partial: &Path,
    received: u64,
    total: Option<u64>,
    events: &mpsc::Sender<DownloadEvent>,
) -> Result<()> {
    if let Some(expected) = total {
        if received != expected {
            bail!("incomplete download: received {received} of {expected} bytes");
        }
    }
    if request.overwrite && request.destination.exists() {
        fs::remove_file(&request.destination).await?;
    }
    fs::rename(partial, &request.destination).await?;
    let _ = events
        .send(DownloadEvent::Completed { bytes: received })
        .await;
    Ok(())
}

pub fn partial_path(destination: &Path) -> PathBuf {
    destination.with_extension(format!(
        "{}part",
        destination
            .extension()
            .and_then(|value| value.to_str())
            .map(|value| format!("{value}."))
            .unwrap_or_default()
    ))
}

pub fn segment_path(destination: &Path, index: usize) -> PathBuf {
    PathBuf::from(format!("{}.part.{index:02}", destination.display()))
}

pub fn chunk_directory(destination: &Path) -> PathBuf {
    let mut hasher = Sha256::new();
    hasher.update(destination.to_string_lossy().as_bytes());
    let identifier = hasher
        .finalize()
        .iter()
        .take(16)
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    destination
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(".apocalipse-parts")
        .join(identifier)
}

fn chunk_path(destination: &Path, index: usize) -> PathBuf {
    chunk_directory(destination).join(format!("{index:06}.part"))
}

fn legacy_chunk_path(destination: &Path, index: usize) -> PathBuf {
    PathBuf::from(format!("{}.part.chunk.{index:06}", destination.display()))
}

pub async fn cleanup_chunk_artifacts(destination: &Path) -> Result<()> {
    let directory = chunk_directory(destination);
    match fs::remove_dir_all(&directory).await {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(error.into()),
    }
    if let Some(root) = directory.parent() {
        let _ = fs::remove_dir(root).await;
    }

    let Some(parent) = destination.parent() else {
        return Ok(());
    };
    let Some(file_name) = destination.file_name().and_then(|value| value.to_str()) else {
        return Ok(());
    };
    let prefix = format!("{file_name}.part.chunk.");
    let mut entries = match fs::read_dir(parent).await {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error.into()),
    };
    while let Some(entry) = entries.next_entry().await? {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        let Some(suffix) = name.strip_prefix(&prefix) else {
            continue;
        };
        if suffix.len() == 6 && suffix.bytes().all(|byte| byte.is_ascii_digit()) {
            match fs::remove_file(entry.path()).await {
                Ok(()) => {}
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => return Err(error.into()),
            }
        }
    }
    for index in 0..32 {
        match fs::remove_file(segment_path(destination, index)).await {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn partial_names_are_deterministic() {
        assert_eq!(
            partial_path(Path::new("video.mp4")),
            PathBuf::from("video.mp4.part")
        );
        assert_eq!(
            segment_path(Path::new("video.mp4"), 7),
            PathBuf::from("video.mp4.part.07")
        );
    }

    #[test]
    fn reads_total_size_from_content_range() {
        assert_eq!(content_range_total("bytes 0-0/5368709120"), Some(5_368_709_120));
        assert_eq!(content_range_total("bytes */4096"), Some(4096));
        assert_eq!(content_range_total("bytes 0-0/*"), None);
    }

    #[test]
    fn adaptive_chunk_names_do_not_collide_with_legacy_segments() {
        assert_eq!(
            chunk_path(Path::new("image.iso"), 42),
            chunk_directory(Path::new("image.iso")).join("000042.part")
        );
        assert_ne!(
            chunk_path(Path::new("image.iso"), 0),
            segment_path(Path::new("image.iso"), 0)
        );
    }

    #[test]
    fn different_destinations_have_different_chunk_directories() {
        assert_ne!(
            chunk_directory(Path::new("first.iso")),
            chunk_directory(Path::new("second.iso"))
        );
    }

    #[tokio::test]
    async fn cleanup_only_removes_chunks_for_the_exact_destination() {
        let root = std::env::temp_dir().join(format!(
            "apocalipse-chunk-cleanup-{}",
            uuid::Uuid::new_v4()
        ));
        std::fs::create_dir_all(&root).unwrap();
        let destination = root.join("image.iso");
        let chunk = legacy_chunk_path(&destination, 1);
        let unrelated = root.join("image.iso.part.chunk.backup");
        std::fs::write(&chunk, b"chunk").unwrap();
        std::fs::write(&unrelated, b"keep").unwrap();

        cleanup_chunk_artifacts(&destination).await.unwrap();

        assert!(!chunk.exists());
        assert!(unrelated.exists());
        std::fs::remove_dir_all(root).unwrap();
    }
}
