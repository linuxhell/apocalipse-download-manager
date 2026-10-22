//! Private loopback stream: never expose holes in a partially downloaded ED2K file.
use std::{io, path::PathBuf, sync::Arc, time::Duration};
use tokio::{
    io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    time::timeout,
};

pub struct Snapshot {
    pub path: PathBuf,
    pub size: u64,
    pub parts: Vec<bool>,
    pub part_size: u64,
}
type Source = Arc<dyn Fn() -> Option<Snapshot> + Send + Sync>;
const PATIENCE: Duration = Duration::from_secs(90);

/// Only a fresh in-memory verification map can authorize reads of an incomplete file.
async fn read_verified(source: &Source, offset: u64, limit: usize) -> io::Result<Vec<u8>> {
    timeout(PATIENCE, async {
        loop {
            let snapshot = source().ok_or_else(|| io::Error::other("ed2k_task_unavailable"))?;
            if offset >= snapshot.size || snapshot.part_size == 0 {
                return Err(io::Error::other("invalid_preview_offset"));
            }
            let index = (offset / snapshot.part_size) as usize;
            if snapshot.parts.get(index) == Some(&true) {
                let count = (snapshot.size - offset)
                    .min(snapshot.part_size - offset % snapshot.part_size)
                    .min(limit as u64) as usize;
                // Completed transfers atomically rename the staging file. Re-resolve on retry.
                match tokio::fs::symlink_metadata(&snapshot.path).await {
                    Ok(meta) if meta.is_file() && !meta.file_type().is_symlink() => {
                        if let Ok(mut file) = tokio::fs::File::open(&snapshot.path).await {
                            file.seek(io::SeekFrom::Start(offset)).await?;
                            let mut bytes = vec![0; count];
                            file.read_exact(&mut bytes).await?;
                            return Ok(bytes);
                        }
                    }
                    Ok(_) => return Err(io::Error::other("invalid_preview_file")),
                    Err(error) if error.kind() == io::ErrorKind::NotFound => {}
                    Err(error) => return Err(error),
                }
            }
            tokio::time::sleep(Duration::from_millis(200)).await;
        }
    })
    .await
    .map_err(|_| io::Error::new(io::ErrorKind::TimedOut, "waiting_for_verified_ed2k_part"))?
}
fn range(value: &str, length: u64) -> Result<(u64, u64), ()> {
    let (start, end) = value
        .strip_prefix("bytes=")
        .ok_or(())?
        .split_once('-')
        .ok_or(())?;
    let (start, end) = if start.is_empty() {
        let suffix: u64 = end.parse().map_err(|_| ())?;
        if suffix == 0 {
            return Err(());
        }
        (length.saturating_sub(suffix), length.saturating_sub(1))
    } else {
        (
            start.parse().map_err(|_| ())?,
            if end.is_empty() {
                length.saturating_sub(1)
            } else {
                end.parse().map_err(|_| ())?
            },
        )
    };
    if length == 0 || start > end || start >= length {
        return Err(());
    }
    Ok((start, end.min(length - 1)))
}
async fn reject(stream: &mut TcpStream, status: &str) -> io::Result<()> {
    stream
        .write_all(
            format!("HTTP/1.1 {status}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n")
                .as_bytes(),
        )
        .await
}
async fn connection(
    mut stream: TcpStream,
    path: &str,
    host: &str,
    source: Source,
) -> io::Result<()> {
    let head = timeout(Duration::from_secs(10), async {
        let mut bytes = Vec::new();
        loop {
            let byte = stream.read_u8().await?;
            bytes.push(byte);
            if bytes.ends_with(b"\r\n\r\n") {
                return Ok::<_, io::Error>(String::from_utf8_lossy(&bytes).into_owned());
            }
            if bytes.len() > 8192 {
                return Err(io::Error::other("preview_header_too_large"));
            }
        }
    })
    .await
    .map_err(|_| io::Error::new(io::ErrorKind::TimedOut, "preview_header_timeout"))??;
    let mut lines = head.lines();
    let request: Vec<_> = lines.next().unwrap_or("").split_whitespace().collect();
    if request.len() != 3 || request[1] != path {
        return reject(&mut stream, "404 Not Found").await;
    }
    if !matches!(request[0], "GET" | "HEAD") {
        return reject(&mut stream, "405 Method Not Allowed").await;
    }
    let mut host_valid = false;
    let mut requested_range = None;
    for line in lines {
        if let Some((name, value)) = line.split_once(':') {
            match name.to_ascii_lowercase().as_str() {
                "host" => host_valid = value.trim() == host,
                "range" => requested_range = Some(value.trim()),
                "origin" => return reject(&mut stream, "403 Forbidden").await,
                _ => {}
            }
        }
    }
    if !host_valid {
        return reject(&mut stream, "421 Misdirected Request").await;
    }
    let Some(snapshot) = source() else {
        return reject(&mut stream, "404 Not Found").await;
    };
    let length = snapshot.size;
    let (start, end) = if let Some(value) = requested_range {
        match range(value, length) {
            Ok(value) => value,
            Err(()) => {
                stream.write_all(format!("HTTP/1.1 416 Range Not Satisfiable\r\nContent-Range: bytes */{length}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n").as_bytes()).await?;
                return Ok(());
            }
        }
    } else {
        (0, length.saturating_sub(1))
    };
    if length == 0 {
        return reject(&mut stream, "503 Service Unavailable").await;
    }
    let partial = requested_range.is_some();
    let mut header = format!("HTTP/1.1 {}\r\nContent-Type: application/octet-stream\r\nContent-Length: {}\r\nAccept-Ranges: bytes\r\nCache-Control: no-store\r\nConnection: close\r\n", if partial {"206 Partial Content"} else {"200 OK"}, end-start+1);
    if partial {
        header.push_str(&format!("Content-Range: bytes {start}-{end}/{length}\r\n"));
    }
    header.push_str("\r\n");
    stream.write_all(header.as_bytes()).await?;
    if request[0] == "HEAD" {
        return Ok(());
    }
    let mut position = start;
    while position <= end {
        let bytes = read_verified(
            &source,
            position,
            (end - position + 1).min(256 * 1024) as usize,
        )
        .await?;
        timeout(Duration::from_secs(30), stream.write_all(&bytes))
            .await
            .map_err(|_| io::Error::new(io::ErrorKind::TimedOut, "preview_player_stalled"))??;
        position += bytes.len() as u64;
    }
    Ok(())
}
pub async fn start(source: Source, extension: &str) -> io::Result<String> {
    let listener = TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0)).await?;
    let host = format!("127.0.0.1:{}", listener.local_addr()?.port());
    let path = format!("/{}/video.{}", uuid::Uuid::new_v4().simple(), extension);
    let url = format!("http://{host}{path}");
    tokio::spawn(async move {
        let slots = Arc::new(tokio::sync::Semaphore::new(8));
        let lifetime = tokio::time::sleep(Duration::from_secs(6 * 60 * 60));
        tokio::pin!(lifetime);
        loop {
            let accepted =
                tokio::select! { _ = &mut lifetime => return, value = listener.accept() => value };
            let Ok((stream, peer)) = accepted else {
                return;
            };
            if !peer.ip().is_loopback() {
                continue;
            }
            let Ok(permit) = slots.clone().try_acquire_owned() else {
                continue;
            };
            let (source, path, host) = (source.clone(), path.clone(), host.clone());
            tokio::spawn(async move {
                let _permit = permit;
                let _ = connection(stream, &path, &host, source).await;
            });
        }
    });
    Ok(url)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn player_ranges_support_seeking_and_suffixes() {
        assert_eq!(range("bytes=7-99", 10), Ok((7, 9)));
        assert_eq!(range("bytes=-3", 10), Ok((7, 9)));
        assert_eq!(range("bytes=5-", 10), Ok((5, 9)));
        assert!(range("bytes=10-", 10).is_err());
        assert!(range("bytes=0-1,5-6", 10).is_err());
    }
    #[tokio::test]
    async fn waits_for_verification_instead_of_reading_disk_holes() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("video.part");
        std::fs::write(&path, b"goodHOLE").unwrap();
        let ready = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let flag = ready.clone();
        let source: Source = Arc::new(move || {
            Some(Snapshot {
                path: path.clone(),
                size: 8,
                parts: vec![true, flag.load(std::sync::atomic::Ordering::Acquire)],
                part_size: 4,
            })
        });
        assert_eq!(read_verified(&source, 0, 8).await.unwrap(), b"good");
        assert!(
            timeout(Duration::from_millis(30), read_verified(&source, 4, 4))
                .await
                .is_err()
        );
        ready.store(true, std::sync::atomic::Ordering::Release);
        assert_eq!(read_verified(&source, 4, 4).await.unwrap(), b"HOLE");
    }
    #[tokio::test]
    async fn serves_ranges_and_rejects_wrong_token_or_host() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("video.part");
        std::fs::write(&path, b"abcdefgh").unwrap();
        let source: Source = Arc::new(move || {
            Some(Snapshot {
                path: path.clone(),
                size: 8,
                parts: vec![true, true],
                part_size: 4,
            })
        });
        let url = start(source, "mkv").await.unwrap();
        let client = reqwest::Client::new();
        let response = client
            .get(&url)
            .header("Range", "bytes=5-7")
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), 206);
        assert_eq!(response.bytes().await.unwrap().as_ref(), b"fgh");
        assert_eq!(
            client.head(&url).send().await.unwrap().headers()["content-length"],
            "8"
        );
        assert_eq!(
            client
                .get(&url)
                .header("Host", "evil.example")
                .send()
                .await
                .unwrap()
                .status(),
            421
        );
        assert_eq!(
            client
                .get(format!("{url}wrong"))
                .send()
                .await
                .unwrap()
                .status(),
            404
        );
        assert_eq!(
            client
                .get(&url)
                .header("Range", "bytes=8-")
                .send()
                .await
                .unwrap()
                .status(),
            416
        );
    }
}
