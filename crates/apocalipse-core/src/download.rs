use anyhow::{bail, Context, Result};
use futures_util::StreamExt;
use reqwest::{header, Client, StatusCode};
use std::{path::PathBuf, time::Duration};
use tokio::{fs, io::AsyncWriteExt, sync::mpsc};

use crate::validation::{PayloadExpectation, validate_payload};

#[derive(Debug, Clone)]
pub struct DownloadRequest {
    pub url: String,
    pub destination: PathBuf,
    pub overwrite: bool,
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
        let client = Client::builder()
            .connect_timeout(Duration::from_secs(15))
            .timeout(Duration::from_secs(60))
            .redirect(reqwest::redirect::Policy::limited(10))
            .user_agent("ApocalipseDownloadManager/0.1")
            .build()?;
        Ok(Self { client })
    }

    pub async fn download(&self, request: DownloadRequest, events: mpsc::Sender<DownloadEvent>) -> Result<()> {
        if request.destination.exists() && !request.overwrite {
            bail!("destination already exists");
        }
        if let Some(parent) = request.destination.parent() {
            fs::create_dir_all(parent).await?;
        }
        let partial = request.destination.with_extension(format!("{}part", request.destination.extension().and_then(|x| x.to_str()).map(|x| format!("{x}." )).unwrap_or_default()));
        let existing = fs::metadata(&partial).await.map(|m| m.len()).unwrap_or(0);
        let mut builder = self.client.get(&request.url);
        if existing > 0 {
            builder = builder.header(header::RANGE, format!("bytes={existing}-"));
        }
        let response = builder.send().await?.error_for_status()?;
        let resumed = existing > 0 && response.status() == StatusCode::PARTIAL_CONTENT;
        let start = if resumed { existing } else { 0 };
        let total = response.content_length().map(|n| n + start);
        let content_type = response.headers().get(header::CONTENT_TYPE).and_then(|value| value.to_str().ok()).map(str::to_owned);
        let expectation = match request.destination.extension().and_then(|value| value.to_str()).map(str::to_ascii_lowercase).as_deref() {
            Some("zip") => PayloadExpectation::Zip,
            Some(_) => PayloadExpectation::Binary,
            None => PayloadExpectation::Any,
        };
        let _ = events.send(DownloadEvent::Started { resumed_at: start, total }).await;
        let mut file = if resumed {
            fs::OpenOptions::new().append(true).open(&partial).await?
        } else {
            fs::File::create(&partial).await?
        };
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
        drop(file);
        if let Some(expected) = total {
            if received != expected { bail!("incomplete download: received {received} of {expected} bytes"); }
        }
        if request.overwrite && request.destination.exists() {
            fs::remove_file(&request.destination).await?;
        }
        fs::rename(&partial, &request.destination).await?;
        let _ = events.send(DownloadEvent::Completed { bytes: received }).await;
        Ok(())
    }
}
