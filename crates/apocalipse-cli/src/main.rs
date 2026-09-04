use anyhow::{bail, Result};
use apocalipse_core::{DownloadEngine, DownloadEvent, DownloadRequest};
use std::path::PathBuf;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 { bail!("usage: apocalipse-cli <url> <destination>"); }
    let (tx, mut rx) = mpsc::channel(64);
    let request = DownloadRequest {
        url: args[1].clone(),
        destination: PathBuf::from(&args[2]),
        overwrite: false,
        connections: 8,
        method: "GET".to_owned(),
        body: None,
        headers: Vec::new(),
    };
    let worker = tokio::spawn(async move { DownloadEngine::new()?.download(request, tx).await });
    while let Some(event) = rx.recv().await {
        match event {
            DownloadEvent::Progress { received, total } => eprintln!("{received}/{}", total.map(|n| n.to_string()).unwrap_or_else(|| "?".into())),
            other => eprintln!("{other:?}"),
        }
    }
    worker.await??;
    Ok(())
}
