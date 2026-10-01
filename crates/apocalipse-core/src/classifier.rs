use url::Url;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DownloadKind {
    Http,
    Magnet,
    Torrent,
    Hls,
    Metalink,
    MediaPage,
    Ftp,
}

pub fn classify_url(input: &str) -> Option<DownloadKind> {
    if input.starts_with("magnet:?") {
        return Some(DownloadKind::Magnet);
    }
    let local_path = input
        .split(['?', '#'])
        .next()
        .unwrap_or(input)
        .to_ascii_lowercase();
    if !input.contains("://") && local_path.ends_with(".torrent") {
        return Some(DownloadKind::Torrent);
    }
    if !input.contains("://") && local_path.ends_with(".m3u8") {
        return Some(DownloadKind::Hls);
    }
    if !input.contains("://")
        && (local_path.ends_with(".meta4") || local_path.ends_with(".metalink"))
    {
        return Some(DownloadKind::Metalink);
    }
    let url = Url::parse(input).ok()?;
    if matches!(url.scheme(), "ftp" | "sftp") {
        return Some(DownloadKind::Ftp);
    }
    if !matches!(url.scheme(), "http" | "https") {
        return None;
    }
    let path = url.path().to_ascii_lowercase();
    if path.ends_with(".torrent") {
        Some(DownloadKind::Torrent)
    } else if path.ends_with(".m3u8") {
        Some(DownloadKind::Hls)
    } else if path.ends_with(".meta4") || path.ends_with(".metalink") {
        Some(DownloadKind::Metalink)
    } else if url
        .domain()
        .is_some_and(|host| host == "reddit.com" || host.ends_with(".reddit.com"))
        && path
            .split('/')
            .collect::<Vec<_>>()
            .windows(2)
            .any(|parts| parts[0] == "comments" && !parts[1].is_empty())
    {
        Some(DownloadKind::MediaPage)
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
                | "tiktok.com"
                | "www.tiktok.com"
                | "vm.tiktok.com"
                | "vt.tiktok.com"
                | "instagram.com"
                | "www.instagram.com"
                | "soundcloud.com"
                | "www.soundcloud.com"
                | "m.soundcloud.com"
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
    fn reddit_posts_use_extractor_but_images_and_feeds_remain_http() {
        for post in [
            "https://www.reddit.com/r/UFOs/comments/1wlkure/title/",
            "https://old.reddit.com/comments/abc123/",
        ] {
            assert_eq!(classify_url(post), Some(DownloadKind::MediaPage));
        }
        for resource in [
            "https://www.reddit.com/r/UFOs/",
            "https://i.redd.it/image.jpg",
            "https://v.redd.it/id/DASH_720.mp4",
            "https://reddit.com.evil.test/comments/abc123/",
        ] {
            assert_eq!(classify_url(resource), Some(DownloadKind::Http));
        }
    }

    #[test]
    fn classifies_special_inputs() {
        assert_eq!(
            classify_url("magnet:?xt=urn:btih:abc"),
            Some(DownloadKind::Magnet)
        );
        assert_eq!(
            classify_url("https://cdn.test/live/master.m3u8?token=x"),
            Some(DownloadKind::Hls)
        );
        assert_eq!(
            classify_url("https://downloads.example/image.meta4"),
            Some(DownloadKind::Metalink)
        );
        assert_eq!(
            classify_url("C:\\Downloads\\release.metalink"),
            Some(DownloadKind::Metalink)
        );
        assert_eq!(
            classify_url("https://youtu.be/abc"),
            Some(DownloadKind::MediaPage)
        );
        assert_eq!(
            classify_url("https://www.facebook.com/share/example/"),
            Some(DownloadKind::MediaPage)
        );
        assert_eq!(
            classify_url("https://www.tiktok.com/@creator/video/123"),
            Some(DownloadKind::MediaPage)
        );
        assert_eq!(
            classify_url("https://vm.tiktok.com/ZMabc123/"),
            Some(DownloadKind::MediaPage)
        );
        assert_eq!(
            classify_url("https://vt.tiktok.com/ZSabc123/"),
            Some(DownloadKind::MediaPage)
        );
        assert_eq!(
            classify_url("https://www.instagram.com/reel/example/"),
            Some(DownloadKind::MediaPage)
        );
        assert_eq!(
            classify_url("https://soundcloud.com/artist/track"),
            Some(DownloadKind::MediaPage)
        );
        assert_eq!(classify_url("file:///tmp/a"), None);
        assert_eq!(
            classify_url("ftp://example.test/file.iso"),
            Some(DownloadKind::Ftp)
        );
        assert_eq!(
            classify_url("sftp://example.test/file.iso"),
            Some(DownloadKind::Ftp)
        );
        assert_eq!(
            classify_url("C:\\Downloads\\video.m3u8"),
            Some(DownloadKind::Hls)
        );
    }
}
