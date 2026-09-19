use anyhow::{bail, Context, Result};
use apocalipse_core::{DownloadEngine, DownloadEvent, DownloadRequest};
use serde_json::json;
use std::{path::PathBuf, time::Instant};
use tokio::sync::mpsc;

fn usage() -> &'static str {
    "usage:
  apocalipse-cli <url> <destination>
  apocalipse-cli benchmark <url> <destination> [connections] [mirror ...]"
}

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let benchmark = args.get(1).is_some_and(|value| value == "benchmark");
    let (url_index, destination_index) = if benchmark { (2, 3) } else { (1, 2) };
    if args.len() <= destination_index {
        bail!(usage());
    }

    let connections = if benchmark {
        args.get(4)
            .map(|value| value.parse::<usize>())
            .transpose()
            .context("invalid connection count")?
            .unwrap_or(8)
            .clamp(1, 32)
    } else {
        8
    };
    let mirrors = if benchmark && args.len() > 5 {
        args[5..].to_vec()
    } else {
        Vec::new()
    };

    let (tx, mut rx) = mpsc::channel(256);
    let request = DownloadRequest {
        url: args[url_index].clone(),
        destination: PathBuf::from(&args[destination_index]),
        overwrite: false,
        connections,
        method: "GET".to_owned(),
        body: None,
        headers: Vec::new(),
        expected_sha256: None,
        limiters: Vec::new(),
    };

    let engine = DownloadEngine::new()?;
    let started = Instant::now();
    let worker = tokio::spawn(async move {
        let sources = engine.verified_sources(&request, &mirrors).await;
        engine.download_from_sources(request, sources, tx).await
    });

    let mut total = None;
    let mut completed_bytes = 0_u64;
    let mut active_connections = 1_usize;
    let mut peak_bytes_per_second = 0_u64;
    let mut source_count = 1_u64;
    let mut segment_count = 0_u64;
    let mut mirror_fallback_segments = 0_u64;
    let mut transports = std::collections::BTreeMap::<String, u64>::new();

    while let Some(event) = rx.recv().await {
        match event {
            DownloadEvent::Started {
                resumed_at,
                total: event_total,
                connections,
                ..
            } => {
                completed_bytes = resumed_at;
                total = event_total;
                active_connections = connections;
            }
            DownloadEvent::Progress {
                received,
                total: event_total,
            } => {
                completed_bytes = received;
                total = event_total.or(total);
                if !benchmark {
                    eprintln!(
                        "{received}/{}",
                        total.map(|n| n.to_string()).unwrap_or_else(|| "?".into())
                    );
                }
            }
            DownloadEvent::Diagnostic { event, detail } => {
                if event == "http.engine_plan" {
                    source_count = detail["sourceCount"].as_u64().unwrap_or(source_count);
                } else if event == "http.performance_sample" {
                    peak_bytes_per_second = peak_bytes_per_second
                        .max(detail["bytesPerSecond"].as_u64().unwrap_or(0));
                } else if event == "http.segment_completed" {
                    segment_count = segment_count.saturating_add(1);
                    if detail["attempts"].as_u64().unwrap_or(1) > 1 {
                        mirror_fallback_segments = mirror_fallback_segments.saturating_add(1);
                    }
                    if let Some(transport) = detail["transport"].as_str() {
                        *transports.entry(transport.to_owned()).or_default() += 1;
                    }
                }
            }
            DownloadEvent::Completed { bytes } => {
                completed_bytes = bytes;
                total = Some(bytes);
            }
        }
    }

    worker.await??;
    let elapsed = started.elapsed();
    if benchmark {
        let elapsed_ms = elapsed.as_millis() as u64;
        let average_bytes_per_second = if elapsed_ms > 0 {
            completed_bytes.saturating_mul(1000) / elapsed_ms
        } else {
            completed_bytes
        };
        println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "schema": 1,
                "url": args[url_index],
                "destination": args[destination_index],
                "elapsedMs": elapsed_ms,
                "bytes": completed_bytes,
                "totalBytes": total,
                "averageBytesPerSecond": average_bytes_per_second,
                "peakObservedBytesPerSecond": peak_bytes_per_second,
                "connections": active_connections,
                "verifiedSources": source_count,
                "segmentsCompleted": segment_count,
                "mirrorFallbackSegments": mirror_fallback_segments,
                "transports": transports,
            }))?
        );
    }

    Ok(())
}
