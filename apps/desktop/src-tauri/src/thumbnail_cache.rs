use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    net::IpAddr,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};
use tokio::fs;
use url::Url;

const MAX_THUMBNAIL_BYTES: usize = 8 * 1024 * 1024;
const MAX_CACHE_BYTES: u64 = 512 * 1024 * 1024;
const MAX_REDIRECTS: usize = 5;
const CACHE_VERSION: u8 = 2;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ThumbnailMetadata {
    version: u8,
    content_hash: String,
    extension: String,
    mime: String,
    bytes: u64,
    fetched_at: u64,
}

#[derive(Debug)]
pub(super) struct ThumbnailResolution {
    pub(super) data_url: String,
    pub(super) cache_hit: bool,
    pub(super) bytes: usize,
    pub(super) content_hash: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ImageKind {
    extension: &'static str,
    mime: &'static str,
}

pub(super) async fn resolve(
    cache_root: &Path,
    client: &reqwest::Client,
    original_url: &str,
) -> Result<Option<ThumbnailResolution>, String> {
    let initial = validate_remote_url(original_url)?;
    let cache_dir = cache_root.join("thumbnails-v2");
    fs::create_dir_all(&cache_dir)
        .await
        .map_err(|error| error.to_string())?;

    let url_key = sha256_hex(original_url.as_bytes());
    if let Some(hit) = read_cached(&cache_dir, &url_key).await? {
        return Ok(Some(hit));
    }

    let (bytes, kind) = fetch_validated(client, initial).await?;
    let content_hash = sha256_hex(&bytes);
    let content_path = cache_dir.join(format!("{content_hash}.{}", kind.extension));

    if fs::metadata(&content_path).await.is_err() {
        let staged = cache_dir.join(format!(".{content_hash}.{}.tmp", kind.extension));
        fs::write(&staged, &bytes)
            .await
            .map_err(|error| error.to_string())?;
        match fs::rename(&staged, &content_path).await {
            Ok(()) => {}
            Err(_) if fs::metadata(&content_path).await.is_ok() => {
                let _ = fs::remove_file(&staged).await;
            }
            Err(error) => {
                let _ = fs::remove_file(&staged).await;
                return Err(error.to_string());
            }
        }
    }

    let metadata = ThumbnailMetadata {
        version: CACHE_VERSION,
        content_hash: content_hash.clone(),
        extension: kind.extension.to_owned(),
        mime: kind.mime.to_owned(),
        bytes: bytes.len() as u64,
        fetched_at: epoch_seconds(),
    };
    write_metadata_atomic(&cache_dir, &url_key, &metadata).await?;
    prune_cache(&cache_dir).await;

    Ok(Some(ThumbnailResolution {
        data_url: data_uri(kind.mime, &bytes),
        cache_hit: false,
        bytes: bytes.len(),
        content_hash,
    }))
}

async fn read_cached(
    cache_dir: &Path,
    url_key: &str,
) -> Result<Option<ThumbnailResolution>, String> {
    let metadata_path = cache_dir.join(format!("{url_key}.json"));
    let raw = match fs::read(&metadata_path).await {
        Ok(raw) => raw,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error.to_string()),
    };
    let metadata: ThumbnailMetadata = match serde_json::from_slice(&raw) {
        Ok(metadata) if metadata.version == CACHE_VERSION => metadata,
        _ => {
            let _ = fs::remove_file(&metadata_path).await;
            return Ok(None);
        }
    };
    if !valid_content_hash(&metadata.content_hash)
        || !valid_extension(&metadata.extension)
        || !metadata.mime.starts_with("image/")
    {
        let _ = fs::remove_file(&metadata_path).await;
        return Ok(None);
    }

    let content_path = cache_dir.join(format!(
        "{}.{}",
        metadata.content_hash, metadata.extension
    ));
    let bytes = match fs::read(&content_path).await {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            let _ = fs::remove_file(&metadata_path).await;
            return Ok(None);
        }
        Err(error) => return Err(error.to_string()),
    };
    let Some(kind) = sniff_image(&bytes) else {
        let _ = fs::remove_file(&content_path).await;
        let _ = fs::remove_file(&metadata_path).await;
        return Ok(None);
    };
    if kind.extension != metadata.extension || kind.mime != metadata.mime {
        let _ = fs::remove_file(&content_path).await;
        let _ = fs::remove_file(&metadata_path).await;
        return Ok(None);
    }
    let actual_hash = sha256_hex(&bytes);
    if actual_hash != metadata.content_hash {
        let _ = fs::remove_file(&content_path).await;
        let _ = fs::remove_file(&metadata_path).await;
        return Ok(None);
    }

    Ok(Some(ThumbnailResolution {
        data_url: data_uri(kind.mime, &bytes),
        cache_hit: true,
        bytes: bytes.len(),
        content_hash: metadata.content_hash,
    }))
}

async fn fetch_validated(
    client: &reqwest::Client,
    mut url: Url,
) -> Result<(Vec<u8>, ImageKind), String> {
    for redirect in 0..=MAX_REDIRECTS {
        let response = client
            .get(url.clone())
            .header(reqwest::header::ACCEPT, "image/avif,image/webp,image/png,image/jpeg,image/gif,image/*;q=0.8")
            .send()
            .await
            .map_err(|error| error.to_string())?;

        if response.status().is_redirection() {
            if redirect == MAX_REDIRECTS {
                return Err("thumbnail_redirect_limit".to_owned());
            }
            let location = response
                .headers()
                .get(reqwest::header::LOCATION)
                .and_then(|value| value.to_str().ok())
                .ok_or_else(|| "thumbnail_redirect_without_location".to_owned())?;
            url = validate_remote_url(
                url.join(location)
                    .map_err(|_| "thumbnail_invalid_redirect".to_owned())?
                    .as_str(),
            )?;
            continue;
        }

        let response = response
            .error_for_status()
            .map_err(|error| error.to_string())?;
        if response
            .content_length()
            .is_some_and(|length| length > MAX_THUMBNAIL_BYTES as u64)
        {
            return Err("thumbnail_too_large".to_owned());
        }

        let mut bytes = Vec::with_capacity(
            response
                .content_length()
                .unwrap_or(128 * 1024)
                .min(MAX_THUMBNAIL_BYTES as u64) as usize,
        );
        let mut stream = response.bytes_stream();
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|error| error.to_string())?;
            if bytes.len().saturating_add(chunk.len()) > MAX_THUMBNAIL_BYTES {
                return Err("thumbnail_too_large".to_owned());
            }
            bytes.extend_from_slice(&chunk);
        }
        let kind = sniff_image(&bytes).ok_or_else(|| "thumbnail_not_an_image".to_owned())?;
        return Ok((bytes, kind));
    }
    Err("thumbnail_redirect_limit".to_owned())
}

fn validate_remote_url(value: &str) -> Result<Url, String> {
    let parsed = Url::parse(value).map_err(|_| "thumbnail_invalid_url".to_owned())?;
    if !matches!(parsed.scheme(), "http" | "https")
        || !parsed.username().is_empty()
        || parsed.password().is_some()
    {
        return Err("thumbnail_invalid_url".to_owned());
    }
    let host = parsed
        .host_str()
        .ok_or_else(|| "thumbnail_invalid_url".to_owned())?
        .trim_end_matches('.')
        .to_ascii_lowercase();
    if host == "localhost"
        || host.ends_with(".localhost")
        || host.ends_with(".local")
        || host.ends_with(".internal")
    {
        return Err("thumbnail_local_target_blocked".to_owned());
    }
    if let Ok(ip) = host.parse::<IpAddr>() {
        if is_non_public_ip(ip) {
            return Err("thumbnail_local_target_blocked".to_owned());
        }
    }
    Ok(parsed)
}

fn is_non_public_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(ip) => {
            ip.is_loopback()
                || ip.is_private()
                || ip.is_link_local()
                || ip.is_broadcast()
                || ip.is_unspecified()
                || ip.is_multicast()
        }
        IpAddr::V6(ip) => {
            let segments = ip.segments();
            let unique_local = (segments[0] & 0xfe00) == 0xfc00;
            let link_local = (segments[0] & 0xffc0) == 0xfe80;
            ip.is_loopback()
                || ip.is_unspecified()
                || ip.is_multicast()
                || unique_local
                || link_local
        }
    }
}

fn sniff_image(bytes: &[u8]) -> Option<ImageKind> {
    if bytes.len() >= 8 && bytes[..8] == [0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a] {
        return Some(ImageKind {
            extension: "png",
            mime: "image/png",
        });
    }
    if bytes.len() >= 3 && bytes[..3] == [0xff, 0xd8, 0xff] {
        return Some(ImageKind {
            extension: "jpg",
            mime: "image/jpeg",
        });
    }
    if bytes.len() >= 6 && (&bytes[..6] == b"GIF87a" || &bytes[..6] == b"GIF89a") {
        return Some(ImageKind {
            extension: "gif",
            mime: "image/gif",
        });
    }
    if bytes.len() >= 12 && &bytes[..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
        return Some(ImageKind {
            extension: "webp",
            mime: "image/webp",
        });
    }
    if bytes.len() >= 12
        && &bytes[4..8] == b"ftyp"
        && (&bytes[8..12] == b"avif" || &bytes[8..12] == b"avis")
    {
        return Some(ImageKind {
            extension: "avif",
            mime: "image/avif",
        });
    }
    if bytes.len() >= 2 && &bytes[..2] == b"BM" {
        return Some(ImageKind {
            extension: "bmp",
            mime: "image/bmp",
        });
    }
    if bytes.len() >= 4 && bytes[..4] == [0x00, 0x00, 0x01, 0x00] {
        return Some(ImageKind {
            extension: "ico",
            mime: "image/x-icon",
        });
    }
    None
}

async fn write_metadata_atomic(
    cache_dir: &Path,
    url_key: &str,
    metadata: &ThumbnailMetadata,
) -> Result<(), String> {
    let final_path = cache_dir.join(format!("{url_key}.json"));
    let staged = cache_dir.join(format!(".{url_key}.json.tmp"));
    let payload = serde_json::to_vec(metadata).map_err(|error| error.to_string())?;
    fs::write(&staged, payload)
        .await
        .map_err(|error| error.to_string())?;
    if fs::metadata(&final_path).await.is_ok() {
        let _ = fs::remove_file(&final_path).await;
    }
    fs::rename(&staged, &final_path)
        .await
        .map_err(|error| error.to_string())
}

async fn prune_cache(cache_dir: &Path) {
    let Ok(mut entries) = fs::read_dir(cache_dir).await else {
        return;
    };
    let mut content = Vec::<(PathBuf, u64, SystemTime)>::new();
    let mut total = 0_u64;
    while let Ok(Some(entry)) = entries.next_entry().await {
        let path = entry.path();
        if path.extension().and_then(|value| value.to_str()) == Some("json") {
            continue;
        }
        let Ok(metadata) = entry.metadata().await else {
            continue;
        };
        if !metadata.is_file() {
            continue;
        }
        let size = metadata.len();
        total = total.saturating_add(size);
        content.push((
            path,
            size,
            metadata.modified().unwrap_or(UNIX_EPOCH),
        ));
    }
    if total <= MAX_CACHE_BYTES {
        return;
    }
    content.sort_by_key(|(_, _, modified)| *modified);
    for (path, size, _) in content {
        if total <= MAX_CACHE_BYTES.saturating_mul(9) / 10 {
            break;
        }
        if fs::remove_file(path).await.is_ok() {
            total = total.saturating_sub(size);
        }
    }
}

fn data_uri(mime: &str, bytes: &[u8]) -> String {
    format!("data:{mime};base64,{}", BASE64.encode(bytes))
}

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn valid_content_hash(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn valid_extension(value: &str) -> bool {
    matches!(value, "jpg" | "png" | "gif" | "webp" | "avif" | "bmp" | "ico")
}

fn epoch_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sniffs_supported_thumbnail_formats_and_rejects_html() {
        assert_eq!(
            sniff_image(&[0xff, 0xd8, 0xff, 0x00]).map(|kind| kind.mime),
            Some("image/jpeg")
        );
        assert_eq!(
            sniff_image(b"\x89PNG\r\n\x1a\nrest").map(|kind| kind.mime),
            Some("image/png")
        );
        assert_eq!(
            sniff_image(b"RIFFxxxxWEBPrest").map(|kind| kind.mime),
            Some("image/webp")
        );
        assert!(sniff_image(b"<html><body>login required</body></html>").is_none());
    }

    #[test]
    fn blocks_obvious_local_thumbnail_targets() {
        for url in [
            "http://127.0.0.1/image.jpg",
            "http://10.0.0.1/image.jpg",
            "http://192.168.1.1/image.jpg",
            "http://[::1]/image.jpg",
            "http://localhost/image.jpg",
            "http://host.local/image.jpg",
        ] {
            assert!(validate_remote_url(url).is_err(), "{url}");
        }
        assert!(validate_remote_url("https://i.ytimg.com/vi/example/hqdefault.jpg").is_ok());
    }

    #[test]
    fn metadata_contains_no_original_thumbnail_url_or_token() {
        let metadata = ThumbnailMetadata {
            version: CACHE_VERSION,
            content_hash: "a".repeat(64),
            extension: "jpg".into(),
            mime: "image/jpeg".into(),
            bytes: 123,
            fetched_at: 1,
        };
        let json = serde_json::to_string(&metadata).unwrap();
        assert!(!json.contains("https://"));
        assert!(!json.contains("token"));
    }

    #[test]
    fn data_uri_uses_sniffed_mime() {
        assert_eq!(data_uri("image/jpeg", b"abc"), "data:image/jpeg;base64,YWJj");
    }
}
