//! RFC 9842 dcz codec and a deliberately narrow, explicit HTTPS update experiment.
use anyhow::{Result, bail, ensure};
use base64::{Engine, engine::general_purpose::STANDARD};
use sha2::{Digest, Sha256};
use std::io::{Read, Write};
use url::Url;

const MAGIC: [u8; 8] = [0x5e, 0x2a, 0x4d, 0x18, 0x20, 0, 0, 0];
const MAX_DICTIONARY: usize = 16 * 1024 * 1024;

pub fn available_dictionary(dictionary: &[u8]) -> String {
    format!(":{}:", STANDARD.encode(Sha256::digest(dictionary)))
}

pub fn encode(dictionary: &[u8], content: &[u8]) -> Result<Vec<u8>> {
    ensure!(dictionary.len() <= MAX_DICTIONARY, "dictionary too large");
    let mut result = MAGIC.to_vec();
    result.extend_from_slice(&Sha256::digest(dictionary));
    let mut compressor = zstd::bulk::Compressor::with_dictionary(5, dictionary)?;
    result.extend(compressor.compress(content)?);
    Ok(result)
}

/// Streaming output with a hard limit; validates the RFC header before decoding.
pub fn decode(dictionary: &[u8], body: &[u8], max_output: u64, mut out: impl Write) -> Result<u64> {
    ensure!(dictionary.len() <= MAX_DICTIONARY, "dictionary too large");
    ensure!(body.len() >= 40 && body[..8] == MAGIC, "invalid dcz header");
    ensure!(
        body[8..40] == Sha256::digest(dictionary)[..],
        "dictionary hash mismatch"
    );
    let mut decoder = zstd::stream::read::Decoder::with_dictionary(&body[40..], dictionary)?;
    // RFC 9842 permits up to 128 MiB windows. Output remains independently bounded.
    decoder.window_log_max(27)?;
    let mut buffer = [0; 64 * 1024];
    let mut total = 0u64;
    loop {
        let count = decoder.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        total = total
            .checked_add(count as u64)
            .ok_or_else(|| anyhow::anyhow!("output overflow"))?;
        ensure!(total <= max_output, "decoded output exceeds limit");
        out.write_all(&buffer[..count])?;
    }
    Ok(total)
}

async fn bounded_body(mut response: reqwest::Response, max: usize) -> Result<Vec<u8>> {
    if let Some(length) = response.content_length() {
        ensure!(length <= max as u64, "response too large");
    }
    let mut result = Vec::new();
    while let Some(chunk) = response.chunk().await? {
        ensure!(
            chunk.len() <= max.saturating_sub(result.len()),
            "response too large"
        );
        result.extend_from_slice(&chunk);
    }
    Ok(result)
}

/// This first version accepts only an exact-path, id-free raw dictionary directive.
/// Unsupported URLPattern, stale metadata, or redirects fail closed; callers can
/// use their existing ordinary downloader as fallback without dictionary headers.
pub async fn fetch(
    dictionary_url: &Url,
    artifact_url: &Url,
    expected_sha256: &str,
    max_bytes: usize,
) -> Result<Vec<u8>> {
    crate::init_crypto();
    ensure!(
        dictionary_url.scheme() == "https" && artifact_url.scheme() == "https",
        "HTTPS required"
    );
    ensure!(
        dictionary_url.origin() == artifact_url.origin(),
        "dictionary must have the same origin"
    );
    ensure!(
        expected_sha256.len() == 64 && expected_sha256.bytes().all(|b| b.is_ascii_hexdigit()),
        "expected SHA-256 required"
    );
    ensure!(
        max_bytes <= 256 * 1024 * 1024,
        "experiment capped at 256 MiB"
    );
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(std::time::Duration::from_secs(120))
        .build()?;
    let started = std::time::Instant::now();
    let response = client
        .get(dictionary_url.clone())
        .header("Accept-Encoding", "identity")
        .send()
        .await?;
    ensure!(
        response.status() == reqwest::StatusCode::OK,
        "dictionary must return 200 without redirect"
    );
    let directive = response
        .headers()
        .get("use-as-dictionary")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| anyhow::anyhow!("missing Use-As-Dictionary"))?;
    let expected_directive = format!("match=\"{}\"", artifact_url.path());
    ensure!(
        directive == expected_directive || directive == format!("{expected_directive}, type=raw"),
        "unsupported dictionary directive (exact path only)"
    );
    let control = response
        .headers()
        .get("cache-control")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    ensure!(
        !control
            .split(',')
            .any(|part| matches!(part.trim(), "no-store" | "no-cache")),
        "dictionary is not reusable"
    );
    let max_age = control
        .split(',')
        .filter_map(|p| p.trim().strip_prefix("max-age="))
        .find_map(|s| s.parse::<u64>().ok())
        .unwrap_or(0);
    let age = response
        .headers()
        .get("age")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("0")
        .parse::<u64>()?;
    ensure!(max_age > age, "dictionary not fresh");
    let remaining_freshness = max_age - age;
    let dictionary = bounded_body(response, MAX_DICTIONARY).await?;
    ensure!(
        started.elapsed().as_secs() < remaining_freshness,
        "dictionary expired while fetching"
    );
    let response = client
        .get(artifact_url.clone())
        .header("Accept-Encoding", "dcz, identity")
        .header("Available-Dictionary", available_dictionary(&dictionary))
        .send()
        .await?;
    ensure!(
        response.status() == reqwest::StatusCode::OK,
        "artifact must return 200 without redirect"
    );
    let encoding = response
        .headers()
        .get("content-encoding")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("identity")
        .to_owned();
    let body = bounded_body(response, max_bytes).await?;
    let output = match encoding.as_str() {
        "dcz" => {
            let mut decoded = Vec::new();
            decode(&dictionary, &body, max_bytes as u64, &mut decoded)?;
            decoded
        }
        "identity" => body,
        _ => bail!("unsupported Content-Encoding"),
    };
    ensure!(
        format!("{:x}", Sha256::digest(&output)) == expected_sha256.to_ascii_lowercase(),
        "artifact SHA-256 mismatch"
    );
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn round_trip_and_reject_bad_dictionary_and_expansion() {
        let dictionary = b"Apocalipse update previous binary data".repeat(1000);
        let mut next = dictionary.clone();
        next.extend_from_slice(b"new version");
        let encoded = encode(&dictionary, &next).unwrap();
        let mut decoded = Vec::new();
        assert_eq!(
            decode(&dictionary, &encoded, next.len() as u64, &mut decoded).unwrap(),
            next.len() as u64
        );
        assert_eq!(decoded, next);
        assert!(decode(b"wrong", &encoded, next.len() as u64, Vec::new()).is_err());
        assert!(decode(&dictionary, &encoded, 10, Vec::new()).is_err());
        assert!(decode(&dictionary, &encoded[..39], 10, Vec::new()).is_err());
    }
    #[tokio::test]
    async fn forbids_cross_origin_and_cleartext() {
        assert!(
            fetch(
                &Url::parse("https://a.example/dict").unwrap(),
                &Url::parse("https://b.example/app").unwrap(),
                &"a".repeat(64),
                10
            )
            .await
            .is_err()
        );
        assert!(
            fetch(
                &Url::parse("http://a.example/dict").unwrap(),
                &Url::parse("http://a.example/app").unwrap(),
                &"a".repeat(64),
                10
            )
            .await
            .is_err()
        );
    }
}
