use url::Url;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DownloadKind {
    Http,
    Magnet,
    Torrent,
    Hls,
    MediaPage,
    Ed2k,
}

pub fn classify_url(input: &str) -> Option<DownloadKind> {
    if input.starts_with("magnet:?") {
        return Some(DownloadKind::Magnet);
    }
    if input.starts_with("ed2k://") {
        return Some(DownloadKind::Ed2k);
    }
    let url = Url::parse(input).ok()?;
    if !matches!(url.scheme(), "http" | "https") {
        return None;
    }
    let path = url.path().to_ascii_lowercase();
    if path.ends_with(".torrent") {
        Some(DownloadKind::Torrent)
    } else if path.ends_with(".m3u8") {
        Some(DownloadKind::Hls)
    } else if matches!(
        url.domain(),
        Some(
            "youtube.com"
                | "www.youtube.com"
                | "youtu.be"
                | "facebook.com"
                | "www.facebook.com"
                | "m.facebook.com"
                | "fb.watch"
        )
    ) {
        Some(DownloadKind::MediaPage)
    } else {
        Some(DownloadKind::Http)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_special_inputs() {
        assert_eq!(classify_url("magnet:?xt=urn:btih:abc"), Some(DownloadKind::Magnet));
        assert_eq!(classify_url("https://cdn.test/live/master.m3u8?token=x"), Some(DownloadKind::Hls));
        assert_eq!(classify_url("https://youtu.be/abc"), Some(DownloadKind::MediaPage));
        assert_eq!(classify_url("https://www.facebook.com/share/example/"), Some(DownloadKind::MediaPage));
        assert_eq!(classify_url("file:///tmp/a"), None);
        assert_eq!(classify_url("ed2k://|file|example.iso|42|abc|/"), Some(DownloadKind::Ed2k));
    }
}
