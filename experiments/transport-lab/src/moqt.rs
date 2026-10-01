//! Explicit relay/track capture. Stores transport objects, not a playable video.
use anyhow::{Context, Result, ensure};
use std::{path::Path, time::Duration};
use tokio::io::AsyncWriteExt;
use url::Url;

const MAX_FRAME: u64 = 16 * 1024 * 1024;

async fn archive_track(
    mut track: moq_net::track::Subscriber,
    file: &mut tokio::fs::File,
    max_bytes: u64,
    committed: &mut u64,
    objects: &mut u64,
) -> Result<()> {
    let mut total = *committed;
    while let Some(mut group) = track.recv_group().await? {
        let mut index = 0u64;
        while let Some(mut frame) = group.next_frame().await? {
            ensure!(frame.size <= MAX_FRAME, "MoQ object exceeds frame limit");
            let metadata = serde_json::to_vec(&serde_json::json!({
                "group": group.sequence, "object": index,
                "timestamp": format!("{:?}", frame.timestamp), "bytes": frame.size
            }))?;
            let record_size = 4u64 + metadata.len() as u64 + frame.size;
            ensure!(
                record_size <= max_bytes.saturating_sub(total),
                "archive exceeds size limit"
            );
            file.write_all(&(metadata.len() as u32).to_be_bytes())
                .await?;
            file.write_all(&metadata).await?;
            let mut received = 0u64;
            while let Some(chunk) = frame.read_chunk().await? {
                received += chunk.len() as u64;
                ensure!(received <= frame.size, "oversized MoQ object");
                file.write_all(&chunk).await?;
            }
            ensure!(received == frame.size, "incomplete MoQ object");
            total += record_size;
            *committed = total;
            *objects += 1;
            index += 1;
        }
    }
    Ok(())
}

pub async fn capture(
    relay: Url,
    broadcast_name: &str,
    track_name: &str,
    destination: &Path,
    duration: Duration,
    max_bytes: u64,
) -> Result<u64> {
    crate::init_crypto();
    ensure!(
        relay.scheme() == "https",
        "verified HTTPS/WebTransport relay required"
    );
    ensure!(
        duration > Duration::ZERO && duration <= Duration::from_secs(3600),
        "duration must be 1..3600 seconds"
    );
    ensure!(max_bytes >= 16, "archive limit too small");
    let origin = moq_tokio::origin::spawn();
    let patterns = moq_net::Patterns::from(moq_net::Pattern::subtree(broadcast_name)?);
    let consumer = origin
        .scope("", &patterns)
        .context("invalid broadcast scope")?
        .consume();
    let client = moq_tokio::connect::Config::default().init(Default::default())?;
    let connection = client.with_subscriber(origin).connect(relay);
    let mut file = tokio::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(destination)
        .await?;
    let magic = b"ADM-MOQ-OBJECTS-1\n";
    file.write_all(magic).await?;
    let mut committed = magic.len() as u64;
    let mut objects = 0;
    let work = async {
        let mut announced = consumer.announced();
        let broadcast = loop {
            let update = announced
                .next()
                .await
                .context("relay closed before broadcast announcement")?;
            if update.kind.is_active() && update.prefix.as_str() == broadcast_name {
                break consumer.request_broadcast(&update.prefix).await?;
            }
        };
        let track = broadcast.track(track_name)?.subscribe(None).await?;
        archive_track(track, &mut file, max_bytes, &mut committed, &mut objects).await
    };
    let result = tokio::select! {
        result = tokio::time::timeout(duration, work) => match result {
            Ok(result) => result,
            Err(_) => Ok(()), // Keep only complete objects when recording duration ends.
        },
        closed = connection.closed() => Err(anyhow::anyhow!("relay connection ended: {closed:?}")),
    };
    if result.is_ok() && objects > 0 {
        file.set_len(committed).await?;
        file.sync_all().await?;
        Ok(objects)
    } else {
        drop(file);
        let _ = tokio::fs::remove_file(destination).await;
        result?;
        anyhow::bail!("no complete MoQ objects received")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn archives_a_complete_group_with_object_boundaries() {
        let origin = moq_tokio::origin::spawn();
        let broadcast = origin.create_broadcast("fixture").unwrap();
        let producer = broadcast.create_track("video", None).unwrap();
        let consumer = broadcast.consume().track("video").unwrap();
        let track = consumer.subscribe(None).await.unwrap();
        let mut group = producer
            .create_group(moq_net::group::Info { sequence: 7 })
            .unwrap();
        group
            .write_frame(moq_net::Timestamp::now(), "frame-one")
            .unwrap();
        group
            .write_frame(moq_net::Timestamp::now(), "frame-two")
            .unwrap();
        group.finish().unwrap();
        // Read exactly one group; publisher remains open, so bound the wait.
        let directory = tempfile::tempdir().unwrap();
        let mut file = tokio::fs::File::create(directory.path().join("capture"))
            .await
            .unwrap();
        let mut committed = 0;
        let mut objects = 0;
        let _ = tokio::time::timeout(
            Duration::from_millis(100),
            archive_track(track, &mut file, 4096, &mut committed, &mut objects),
        )
        .await;
        assert_eq!(objects, 2);
        file.sync_all().await.unwrap();
        let output = tokio::fs::read(directory.path().join("capture"))
            .await
            .unwrap();
        assert_eq!(output.len() as u64, committed);
        assert!(output.windows(9).any(|v| v == b"frame-one"));
    }
}
