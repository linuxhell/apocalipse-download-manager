use anyhow::{bail, Context, Result};
use reqwest::Url;

const MAX_METALINK_BYTES: usize = 4 * 1024 * 1024;
const MAX_MIRRORS: usize = 32;
const MAX_FILES: usize = 256;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetalinkFile {
    pub name: Option<String>,
    pub size: Option<u64>,
    pub sha256: Option<String>,
    pub urls: Vec<String>,
}

fn xml_unescape(value: &str) -> Result<String> {
    let mut output = String::with_capacity(value.len());
    let mut rest = value;
    while let Some(index) = rest.find('&') {
        output.push_str(&rest[..index]);
        rest = &rest[index..];
        let Some(end) = rest.find(';') else {
            bail!("unterminated XML entity")
        };
        let entity = &rest[1..end];
        match entity {
            "amp" => output.push('&'),
            "lt" => output.push('<'),
            "gt" => output.push('>'),
            "quot" => output.push('"'),
            "apos" => output.push('\''),
            _ if entity.starts_with("#x") => output.push(
                char::from_u32(u32::from_str_radix(&entity[2..], 16)?)
                    .context("invalid XML character")?,
            ),
            _ if entity.starts_with('#') => {
                output.push(char::from_u32(entity[1..].parse()?).context("invalid XML character")?)
            }
            _ => bail!("unsupported XML entity"),
        }
        rest = &rest[end + 1..];
    }
    output.push_str(rest);
    Ok(output)
}

fn attribute(tag: &str, name: &str) -> Option<String> {
    let mut cursor = tag;
    while let Some(index) = cursor.find(name) {
        cursor = &cursor[index + name.len()..];
        let trimmed = cursor.trim_start();
        if !trimmed.starts_with('=') {
            continue;
        }
        let value = trimmed[1..].trim_start();
        let quote = value.chars().next()?;
        if quote != '"' && quote != '\'' {
            continue;
        }
        let body = &value[quote.len_utf8()..];
        let end = body.find(quote)?;
        return xml_unescape(&body[..end]).ok();
    }
    None
}

fn elements<'a>(xml: &'a str, local_name: &str) -> Vec<(&'a str, &'a str)> {
    let mut result = Vec::new();
    let mut cursor = 0;
    while let Some(relative) = xml[cursor..].find('<') {
        let start = cursor + relative;
        let Some(tag_end_relative) = xml[start..].find('>') else {
            break;
        };
        let tag_end = start + tag_end_relative;
        let tag = &xml[start + 1..tag_end];
        let element_name = tag
            .split_ascii_whitespace()
            .next()
            .unwrap_or("")
            .rsplit(':')
            .next()
            .unwrap_or("");
        if element_name == local_name {
            let closing = format!("</{}>", tag.split_ascii_whitespace().next().unwrap_or(""));
            if let Some(close_relative) = xml[tag_end + 1..].find(&closing) {
                let close = tag_end + 1 + close_relative;
                result.push((tag, &xml[tag_end + 1..close]));
                cursor = close + closing.len();
                continue;
            }
        }
        cursor = tag_end + 1;
    }
    result
}

/// Parses the bounded, security-relevant subset of RFC 5854 used by the
/// downloader. DTDs and custom entities are rejected to avoid external entity
/// expansion; only HTTP(S) mirrors are returned.
pub fn parse_metalink(xml: &[u8], base_url: Option<&str>) -> Result<Vec<MetalinkFile>> {
    if xml.len() > MAX_METALINK_BYTES {
        bail!("Metalink document is too large")
    }
    let text = std::str::from_utf8(xml).context("Metalink is not UTF-8")?;
    let lowered = text.to_ascii_lowercase();
    if lowered.contains("<!doctype") || lowered.contains("<!entity") {
        bail!("DTD and entity declarations are not allowed")
    }
    let base = base_url.map(Url::parse).transpose()?;
    let mut files = Vec::new();
    for (file_tag, file_body) in elements(text, "file") {
        let name = attribute(file_tag, "name")
            .map(|value| value.trim().to_owned())
            .filter(|value| {
                !value.is_empty()
                    && !value
                        .chars()
                        .any(|character| matches!(character, '/' | '\\'))
            });
        let size = elements(file_body, "size")
            .first()
            .and_then(|(_, value)| value.trim().parse::<u64>().ok());
        let sha256 = elements(file_body, "hash")
            .into_iter()
            .find(|(tag, _)| {
                attribute(tag, "type").is_some_and(|kind| {
                    matches!(kind.to_ascii_lowercase().as_str(), "sha-256" | "sha256")
                })
            })
            .and_then(|(_, value)| {
                let digest = value.trim().to_ascii_lowercase();
                (digest.len() == 64 && digest.bytes().all(|byte| byte.is_ascii_hexdigit()))
                    .then_some(digest)
            });
        let mut urls = Vec::new();
        for (_, value) in elements(file_body, "url") {
            let decoded = xml_unescape(value.trim())?;
            let parsed = match Url::parse(&decoded) {
                Ok(parsed) => Some(parsed),
                Err(_) => base.as_ref().and_then(|base| base.join(&decoded).ok()),
            };
            let Some(parsed) = parsed else { continue };
            if matches!(parsed.scheme(), "http" | "https")
                && parsed.username().is_empty()
                && parsed.password().is_none()
                && !urls.iter().any(|existing| existing == parsed.as_str())
            {
                urls.push(parsed.to_string());
            }
            if urls.len() >= MAX_MIRRORS {
                break;
            }
        }
        if !urls.is_empty() {
            files.push(MetalinkFile {
                name,
                size,
                sha256,
                urls,
            });
            if files.len() >= MAX_FILES {
                break;
            }
        }
    }
    if files.is_empty() {
        bail!("Metalink contains no usable files")
    }
    Ok(files)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_meta4_mirrors_and_sha256() {
        let document = br#"<?xml version="1.0"?><metalink xmlns="urn:ietf:params:xml:ns:metalink"><file name="image.iso"><size>4096</size><hash type="sha-256">aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa</hash><url>https://a.example/image.iso</url><url>mirror/image.iso?x=1&amp;y=2</url></file></metalink>"#;
        let parsed =
            parse_metalink(document, Some("https://b.example/releases/file.meta4")).unwrap();
        assert_eq!(parsed[0].name.as_deref(), Some("image.iso"));
        assert_eq!(parsed[0].size, Some(4096));
        assert_eq!(parsed[0].urls.len(), 2);
        assert_eq!(
            parsed[0].urls[1],
            "https://b.example/releases/mirror/image.iso?x=1&y=2"
        );
    }

    #[test]
    fn rejects_external_entity_documents() {
        assert!(parse_metalink(
            br#"<!DOCTYPE foo [<!ENTITY xxe SYSTEM "file:///etc/passwd">]><metalink/>"#,
            None
        )
        .is_err());
    }
}
