use anyhow::{bail, Context, Result};
use futures_util::{stream::FuturesUnordered, StreamExt};
use reqwest::{header, Client, StatusCode};
use std::{
    path::{Path, PathBuf},
    sync::{atomic::{AtomicU64, Ordering}, Arc},
    time::Duration,
};
use tokio::{fs, io::{AsyncReadExt, AsyncWriteExt}, sync::mpsc};

use crate::validation::{validate_payload, PayloadExpectation};

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
    Started { resumed_at: u64, total: Option<u64> },
    Progress { received: u64, total: Option<u64> },
    Completed { bytes: u64 },
}

#[derive(Clone)]
pub struct DownloadEngine {
    client: Client,
}

impl DownloadEngine {
    pub fn new() -> Result<Self> {
        Self::with_proxy(None, None, None)
    }

    pub fn with_proxy(proxy_url: Option<&str>, username: Option<&str>, password: Option<&str>) -> Result<Self> {
        let mut builder = Client::builder()
            .connect_timeout(Duration::from_secs(15))
            .read_timeout(Duration::from_secs(60))
            .redirect(reqwest::redirect::Policy::limited(10))
            .user_agent("ApocalipseDownloadManager/0.1");
        if let Some(url) = proxy_url.map(str::trim).filter(|value| !value.is_empty()) {
            let mut proxy = reqwest::Proxy::all(url).context("invalid proxy URL")?;
            if let Some(user) = username.filter(|value| !value.is_empty()) {
                proxy = proxy.basic_auth(user, password.unwrap_or_default());
            }
            builder = builder.proxy(proxy);
        }
        let client = builder.build()?;
        Ok(Self { client })
    }

    pub async fn download(&self, request: DownloadRequest, events: mpsc::Sender<DownloadEvent>) -> Result<()> {
        if request.destination.exists() && !request.overwrite {
            bail!("destination already exists");
        }
        if let Some(parent) = request.destination.parent() {
            fs::create_dir_all(parent).await?;
        }
        let can_segment = request.method.eq_ignore_ascii_case("GET") && request.body.is_none();
        let head = if can_segment { self.client.head(&request.url).send().await.ok() } else { None };
        let total = head.as_ref().and_then(|response| response.content_length());
        let accepts_ranges = head.as_ref().and_then(|response| response.headers().get(header::ACCEPT_RANGES))
            .and_then(|value| value.to_str().ok()).is_some_and(|value| value.eq_ignore_ascii_case("bytes"));
        let requested = request.connections.clamp(1, 32);
        let useful_connections = total.map(|size| requested.min(size.div_ceil(4_194_304) as usize)).unwrap_or(1);
        if can_segment && accepts_ranges && useful_connections > 1 {
            let probe = self.client.get(&request.url).header(header::RANGE, "bytes=0-0").send().await?;
            if probe.status() == StatusCode::PARTIAL_CONTENT {
                return self.download_segmented(request, events, total.unwrap(), useful_connections).await;
            }
        }
        self.download_single(request, events).await
    }

    async fn download_single(&self, request: DownloadRequest, events: mpsc::Sender<DownloadEvent>) -> Result<()> {
        let partial = partial_path(&request.destination);
        let existing = fs::metadata(&partial).await.map(|metadata| metadata.len()).unwrap_or(0);
        let method = reqwest::Method::from_bytes(request.method.as_bytes()).context("invalid HTTP method")?;
        let mut builder = self.client.request(method, &request.url);
        for (name, value) in &request.headers {
            builder = builder.header(name.as_str(), value.as_str());
        }
        if let Some(body) = &request.body { builder = builder.body(body.clone()); }
        if existing > 0 && request.method.eq_ignore_ascii_case("GET") && request.body.is_none() {
            builder = builder.header(header::RANGE, format!("bytes={existing}-"));
        }
        let response = builder.send().await?.error_for_status()?;
        let resumed = existing > 0 && response.status() == StatusCode::PARTIAL_CONTENT;
        let start = if resumed { existing } else { 0 };
        let total = response.content_length().map(|size| size + start);
        let content_type = response.headers().get(header::CONTENT_TYPE).and_then(|value| value.to_str().ok()).map(str::to_owned);
        let expectation = payload_expectation(&request.destination);
        let _ = events.send(DownloadEvent::Started { resumed_at: start, total }).await;
        let mut file = if resumed { fs::OpenOptions::new().append(true).open(&partial).await? } else { fs::File::create(&partial).await? };
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
            let _ = events.send(DownloadEvent::Progress { received, total }).await;
        }
        file.flush().await?;
        finish_download(&request, &partial, received, total, &events).await
    }

    async fn download_segmented(&self, request: DownloadRequest, events: mpsc::Sender<DownloadEvent>, total: u64, connections: usize) -> Result<()> {
        let progress = Arc::new(AtomicU64::new(0));
        let mut jobs = FuturesUnordered::new();
        for index in 0..connections {
            let start = total * index as u64 / connections as u64;
            let end = total * (index as u64 + 1) / connections as u64 - 1;
            let segment = segment_path(&request.destination, index);
            let existing = fs::metadata(&segment).await.map(|metadata| metadata.len()).unwrap_or(0).min(end - start + 1);
            progress.fetch_add(existing, Ordering::Relaxed);
            if existing == end - start + 1 { continue; }
            let client = self.client.clone();
            let url = request.url.clone();
            let sender = events.clone();
            let shared = progress.clone();
            jobs.push(async move {
                let response = client.get(url).header(header::RANGE, format!("bytes={}-{}", start + existing, end)).send().await?;
                if response.status() != StatusCode::PARTIAL_CONTENT { bail!("server stopped supporting byte ranges"); }
                let mut file = if existing > 0 { fs::OpenOptions::new().append(true).open(&segment).await? } else { fs::File::create(&segment).await? };
                let mut stream = response.bytes_stream();
                while let Some(chunk) = stream.next().await {
                    let chunk = chunk.context("segmented network stream failed")?;
                    file.write_all(&chunk).await?;
                    let received = shared.fetch_add(chunk.len() as u64, Ordering::Relaxed) + chunk.len() as u64;
                    let _ = sender.send(DownloadEvent::Progress { received, total: Some(total) }).await;
                }
                file.flush().await?;
                Result::<()>::Ok(())
            });
        }
        let resumed = progress.load(Ordering::Relaxed);
        let _ = events.send(DownloadEvent::Started { resumed_at: resumed, total: Some(total) }).await;
        while let Some(result) = jobs.next().await { result?; }
        let partial = partial_path(&request.destination);
        let mut output = fs::File::create(&partial).await?;
        let mut buffer = vec![0_u8; 256 * 1024];
        for index in 0..connections {
            let segment = segment_path(&request.destination, index);
            let mut input = fs::File::open(&segment).await?;
            loop {
                let count = input.read(&mut buffer).await?;
                if count == 0 { break; }
                output.write_all(&buffer[..count]).await?;
            }
            fs::remove_file(segment).await?;
        }
        output.flush().await?;
        finish_download(&request, &partial, total, Some(total), &events).await
    }
}

fn payload_expectation(destination: &Path) -> PayloadExpectation {
    match destination.extension().and_then(|value| value.to_str()).map(str::to_ascii_lowercase).as_deref() {
        Some("zip") => PayloadExpectation::Zip,
        Some(_) => PayloadExpectation::Binary,
        None => PayloadExpectation::Any,
    }
}

async fn finish_download(request: &DownloadRequest, partial: &Path, received: u64, total: Option<u64>, events: &mpsc::Sender<DownloadEvent>) -> Result<()> {
    if let Some(expected) = total {
        if received != expected { bail!("incomplete download: received {received} of {expected} bytes"); }
    }
    if request.overwrite && request.destination.exists() { fs::remove_file(&request.destination).await?; }
    fs::rename(partial, &request.destination).await?;
    let _ = events.send(DownloadEvent::Completed { bytes: received }).await;
    Ok(())
}

pub fn partial_path(destination: &Path) -> PathBuf {
    destination.with_extension(format!("{}part", destination.extension().and_then(|value| value.to_str()).map(|value| format!("{value}.")).unwrap_or_default()))
}

pub fn segment_path(destination: &Path, index: usize) -> PathBuf {
    PathBuf::from(format!("{}.part.{index:02}", destination.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn partial_names_are_deterministic() {
        assert_eq!(partial_path(Path::new("video.mp4")), PathBuf::from("video.mp4.part"));
        assert_eq!(segment_path(Path::new("video.mp4"), 7), PathBuf::from("video.mp4.part.07"));
    }
}
