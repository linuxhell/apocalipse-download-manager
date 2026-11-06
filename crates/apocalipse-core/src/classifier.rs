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

/// A single Vimeo video, including player embeds and unlisted links.
/// Collection pages and CDN resources are not video permalinks.
pub(crate) fn is_vimeo_video_page(url: &Url) -> bool {
    let host = url.host_str().unwrap_or_default();
    let parts: Vec<_> = url.path().trim_matches('/').split('/').collect();
    let id = |s: &str| !s.is_empty() && s.bytes().all(|c| c.is_ascii_digit());
    match host {
        "player.vimeo.com" => matches!(parts.as_slice(), ["video", video] if id(video)),
        "vimeo.com" | "www.vimeo.com" => match parts.as_slice() {
            [video] => id(video),
            [video, hash] => {
                id(video) && !hash.is_empty() && hash.bytes().all(|c| c.is_ascii_alphanumeric())
            }
            ["channels", channel, video] => !channel.is_empty() && id(video),
            ["groups", group, "videos", video] => !group.is_empty() && id(video),
            ["showcase" | "album", collection, "video", video] => id(collection) && id(video),
            _ => false,
        },
        _ => false,
    }
}

/// Resolve the Vimeo player endpoint without discarding an unlisted-video hash.
/// This uses the player extraction path for both inspection and payload download.
pub fn vimeo_player_url(input: &str) -> Option<String> {
    let page = Url::parse(input).ok()?;
    if !matches!(page.scheme(), "http" | "https")
        || !is_vimeo_video_page(&page)
        || !page.username().is_empty()
        || page.password().is_some()
    {
        return None;
    }
    let parts: Vec<_> = page.path().trim_matches('/').split('/').collect();
    let (id, path_hash) = if page.host_str() == Some("player.vimeo.com") {
        (parts[1], None)
    } else if parts.len() == 2 && parts[0].bytes().all(|c| c.is_ascii_digit()) {
        (parts[0], Some(parts[1].to_owned()))
    } else {
        (*parts.last()?, None)
    };
    let hash = page
        .query_pairs()
        .find(|(key, _)| key == "h")
        .map(|(_, value)| value.into_owned())
        .or(path_hash);
    let mut player = Url::parse(&format!("https://player.vimeo.com/video/{id}")).ok()?;
    if let Some(hash) = hash.filter(|hash| !hash.is_empty()) {
        player.query_pairs_mut().append_pair("h", &hash);
    }
    Some(player.into())
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
    } else if path.ends_with(".m3u8") || url.fragment() == Some("adm.m3u8") {
        // The fragment marks manifests the browser extension identified by
        // Content-Type on URLs that carry no ".m3u8" extension.
        Some(DownloadKind::Hls)
    } else if path.ends_with(".meta4") || path.ends_with(".metalink") {
        Some(DownloadKind::Metalink)
    } else if is_vimeo_video_page(&url)
        || url
            .domain()
            .is_some_and(|host| host == "reddit.com" || host.ends_with(".reddit.com"))
            && path
                .split('/')
                .zip(path.split('/').skip(1))
                .any(|(part, next)| part == "comments" && !next.is_empty())
        || matches!(
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
        )
    {
        Some(DownloadKind::MediaPage)
    } else {
        Some(DownloadKind::Http)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vimeo_player_resolution_retains_the_video_and_unlisted_hash() {
        assert_eq!(
            vimeo_player_url("https://vimeo.com/492873660?utm_source=example").as_deref(),
            Some("https://player.vimeo.com/video/492873660")
        );
        for source in [
            "https://vimeo.com/492873660/secret",
            "https://player.vimeo.com/video/492873660?h=secret",
        ] {
            assert_eq!(
                vimeo_player_url(source).as_deref(),
                Some("https://player.vimeo.com/video/492873660?h=secret")
            );
        }
        assert!(vimeo_player_url("https://vimeo.com.evil.test/492873660").is_none());
        assert!(vimeo_player_url("https://vimeo.com/showcase/123").is_none());
    }

    #[test]
    fn vimeo_video_permalinks_are_media_pages_but_cdn_files_are_not() {
        for page in [
            "https://vimeo.com/492873660",
            "https://player.vimeo.com/video/492873660?h=abc",
            "https://vimeo.com/492873660/unlistedhash",
            "https://vimeo.com/channels/documentary/492873660",
            "https://vimeo.com/groups/films/videos/492873660",
            "https://vimeo.com/showcase/123/video/492873660",
        ] {
            assert_eq!(classify_url(page), Some(DownloadKind::MediaPage));
        }
        for other in [
            "https://vimeo.com/",
            "https://vimeo.com/channels/123",
            "https://vimeo.com/showcase/123",
            "https://vimeo.com.evil.test/492873660",
            "https://vod.vimeocdn.com/movie.mp4",
        ] {
            assert_eq!(classify_url(other), Some(DownloadKind::Http));
        }
    }

    #[test]
    fn extensionless_manifest_marked_by_extension_is_hls() {
        assert_eq!(
            classify_url("https://cdn.example.com/video/abc/playlist#adm.m3u8"),
            Some(DownloadKind::Hls)
        );
    }

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
