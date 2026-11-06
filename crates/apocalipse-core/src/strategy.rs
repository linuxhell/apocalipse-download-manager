use crate::{classify_url, DownloadKind};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Engine {
    NativeHttp,
    XtremeRpc,
    YtDlp,
    NativeHls,
    NM3u8dlRe,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Capabilities {
    pub xtreme: bool,
    pub yt_dlp: bool,
    pub n_m3u8dl_re: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StrategyPlan {
    pub primary: Engine,
    pub fallbacks: Vec<Engine>,
    pub reason: &'static str,
}

/// Select a canonical media page when a browser hands off one isolated media
/// track. This is deliberately local and deterministic: it does not probe the
/// network. Manifests are rerouted only for explicitly supported page/CDN pairs;
/// unrelated files and non-GET forms keep their original route.
pub fn contextual_media_page<'a>(
    resource_url: &str,
    page_url: Option<&'a str>,
    media_kind: Option<&str>,
    has_companion_audio: bool,
    request_method: Option<&str>,
) -> Option<&'a str> {
    let resource = url::Url::parse(resource_url).ok()?;
    let page = page_url.and_then(|value| url::Url::parse(value).ok())?;
    let vimeo_capture = crate::classifier::is_vimeo_video_page(&page)
        && resource
            .host_str()
            .is_some_and(|host| host == "vimeocdn.com" || host.ends_with(".vimeocdn.com"));
    let media_capture = media_kind.is_some_and(|kind| {
        kind.eq_ignore_ascii_case("video") || kind.eq_ignore_ascii_case("audio")
    }) || (vimeo_capture
        && classify_url(resource_url) == Some(DownloadKind::Hls)
        && media_kind.is_none_or(|kind| kind.eq_ignore_ascii_case("hls")));
    if has_companion_audio
        || !media_capture
        || request_method.is_some_and(|method| {
            !method.eq_ignore_ascii_case("GET") && !method.eq_ignore_ascii_case("HEAD")
        })
    {
        return None;
    }
    let resource_kind = classify_url(resource_url);
    if !matches!(page.scheme(), "http" | "https")
        || !page.username().is_empty()
        || page.password().is_some()
    {
        return None;
    }
    let host = page.host_str()?.to_ascii_lowercase();
    let host_is = |domain: &str| host == domain || host.ends_with(&format!(".{domain}"));
    // Vimeo can hand over one video-only HLS rendition; use the specific page
    // to resolve the full presentation. Other sites' live manifests stay direct.
    let allowed_kinds: &[DownloadKind] = if host_is("soundcloud.com") || vimeo_capture {
        &[DownloadKind::Http, DownloadKind::Hls]
    } else {
        &[DownloadKind::Http]
    };
    if !resource_kind.is_some_and(|kind| allowed_kinds.contains(&kind)) {
        return None;
    }
    let path = page.path();
    let specific = if host_is("vimeo.com") {
        vimeo_capture
    } else if host_is("youtube.com") {
        path == "/watch" || path.starts_with("/shorts/") || path.starts_with("/live/")
    } else if host_is("youtu.be") {
        path.trim_matches('/')
            .split('/')
            .next()
            .is_some_and(|id| !id.is_empty())
    } else if host_is("facebook.com") || host_is("fb.watch") {
        [
            "/reel/",
            "/reels/",
            "/videos/",
            "/posts/",
            "/share/r/",
            "/share/v/",
        ]
        .iter()
        .any(|prefix| path.to_ascii_lowercase().contains(prefix))
            || (["/watch/", "/watch"]
                .iter()
                .any(|value| path.eq_ignore_ascii_case(value))
                && page
                    .query_pairs()
                    .any(|(name, value)| name == "v" && !value.is_empty()))
            || (["/permalink.php", "/story.php"]
                .iter()
                .any(|value| path.eq_ignore_ascii_case(value))
                && page.query().is_some())
    } else if host_is("instagram.com") {
        ["/reel/", "/reels/", "/p/", "/tv/"]
            .iter()
            .any(|prefix| path.to_ascii_lowercase().starts_with(prefix))
    } else if host_is("tiktok.com") {
        let lower = path.to_ascii_lowercase();
        lower.starts_with("/@") && lower.contains("/video/")
    } else if host_is("soundcloud.com") {
        const RESERVED: &[&str] = &[
            "you",
            "discover",
            "charts",
            "stream",
            "search",
            "upload",
            "creators",
            "pro",
            "backstage",
            "developers",
            "jobs",
            "legal",
            "pages",
            "settings",
            "notifications",
            "messages",
            "signin",
            "signup",
        ];
        let segments: Vec<&str> = path.trim_matches('/').split('/').collect();
        matches!(segments.as_slice(), [artist, track]
            if !artist.is_empty()
                && !track.is_empty()
                && !RESERVED.contains(&artist.to_ascii_lowercase().as_str()))
    } else {
        false
    };
    specific.then_some(page_url?)
}

pub fn plan_download(input: &str, capabilities: Capabilities) -> Option<StrategyPlan> {
    let kind = classify_url(input)?;
    let plan = match kind {
        DownloadKind::Http => StrategyPlan {
            primary: if capabilities.xtreme {
                Engine::XtremeRpc
            } else {
                Engine::NativeHttp
            },
            fallbacks: capabilities
                .xtreme
                .then_some(Engine::NativeHttp)
                .into_iter()
                .collect(),
            reason: if capabilities.xtreme {
                "xtreme_accelerated_http"
            } else {
                "direct_http"
            },
        },
        DownloadKind::MediaPage => StrategyPlan {
            primary: if capabilities.yt_dlp {
                Engine::YtDlp
            } else {
                Engine::NativeHttp
            },
            fallbacks: vec![Engine::NativeHttp],
            reason: if capabilities.yt_dlp {
                "media_extractor_available"
            } else {
                "media_extractor_missing"
            },
        },
        DownloadKind::Metalink => StrategyPlan {
            primary: Engine::NativeHttp,
            fallbacks: Vec::new(),
            reason: "metalink_manifest",
        },
        DownloadKind::Hls => StrategyPlan {
            primary: if capabilities.n_m3u8dl_re {
                Engine::NM3u8dlRe
            } else {
                Engine::NativeHls
            },
            fallbacks: vec![Engine::NativeHls],
            reason: "hls_manifest",
        },
        DownloadKind::Ftp => StrategyPlan {
            primary: Engine::XtremeRpc,
            fallbacks: Vec::new(),
            reason: "ftp_transfer",
        },
        DownloadKind::Torrent | DownloadKind::Magnet => StrategyPlan {
            primary: Engine::XtremeRpc,
            fallbacks: Vec::new(),
            reason: "peer_to_peer",
        },
    };
    Some(plan)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vimeo_video_only_hls_uses_the_full_video_page_and_extractor() {
        let leaf = "https://vod-adaptive-ak.vimeocdn.com/exp=123/v2/playlist/av/id/media.m3u8?pathsig=signature";
        for page in [
            "https://vimeo.com/492873660?utm_source=example",
            "https://player.vimeo.com/video/492873660?h=privatehash",
            "https://vimeo.com/492873660/privatehash",
        ] {
            for kind in [Some("video"), Some("hls"), None] {
                let routed = contextual_media_page(leaf, Some(page), kind, false, Some("GET"));
                assert_eq!(routed, Some(page));
                assert_eq!(classify_url(routed.unwrap()), Some(DownloadKind::MediaPage));
                assert_eq!(
                    plan_download(
                        page,
                        Capabilities {
                            yt_dlp: true,
                            ..Default::default()
                        }
                    )
                    .unwrap()
                    .primary,
                    Engine::YtDlp
                );
            }
        }
    }

    #[test]
    fn vimeo_routing_preserves_unrelated_files_forms_and_paired_tracks() {
        let leaf = "https://vod-adaptive-ak.vimeocdn.com/video/media.m3u8";
        let page = "https://vimeo.com/492873660";
        for bad_page in [
            "https://vimeo.com/",
            "https://vimeo.com/channels/123",
            "https://vimeo.com.evil.test/492873660",
            "https://user:pass@vimeo.com/492873660",
        ] {
            assert_eq!(
                contextual_media_page(leaf, Some(bad_page), Some("video"), false, None),
                None
            );
        }
        for resource in [
            "https://example.test/media.m3u8",
            "https://vimeocdn.com.evil.test/media.m3u8",
        ] {
            assert_eq!(
                contextual_media_page(resource, Some(page), Some("video"), false, None),
                None
            );
        }
        assert_eq!(
            contextual_media_page(leaf, Some(page), Some("video"), false, Some("POST")),
            None
        );
        assert_eq!(
            contextual_media_page(leaf, Some(page), Some("video"), true, None),
            None
        );
        assert_eq!(
            contextual_media_page(leaf, None, Some("video"), false, None),
            None
        );
    }

    #[test]
    fn direct_download_prefers_xtreme_and_keeps_native_as_fallback() {
        let plan = plan_download(
            "https://example.test/file.zip",
            Capabilities {
                xtreme: true,
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(plan.primary, Engine::XtremeRpc);
        assert_eq!(plan.fallbacks, vec![Engine::NativeHttp]);
        assert_eq!(plan.reason, "xtreme_accelerated_http");
    }

    #[test]
    fn youtube_prefers_ytdlp_when_installed() {
        let plan = plan_download(
            "https://youtube.com/watch?v=x",
            Capabilities {
                yt_dlp: true,
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(plan.primary, Engine::YtDlp);
        assert_eq!(plan.reason, "media_extractor_available");
    }

    #[test]
    fn isolated_social_tracks_use_specific_page_context() {
        for page in [
            "https://www.youtube.com/watch?v=x",
            "https://youtu.be/x",
            "https://www.facebook.com/reel/123/",
            "https://www.facebook.com/user/videos/123/",
            "https://www.instagram.com/reel/abc/",
            "https://www.instagram.com/p/abc/",
            "https://www.tiktok.com/@creator/video/123",
        ] {
            assert_eq!(
                contextual_media_page(
                    "https://cdn.example.test/track.mp4",
                    Some(page),
                    Some("video"),
                    false,
                    Some("GET"),
                ),
                Some(page),
            );
        }
    }

    #[test]
    fn soundcloud_hls_capture_routes_to_the_canonical_track_page() {
        // A SoundCloud capture hands over a signed, short-lived
        // playback.media-streaming.soundcloud.cloud/.../playlist.m3u8 manifest.
        // That signature expires quickly, so the download must prefer the
        // canonical track page (yt-dlp re-resolves a fresh signed URL) instead
        // of reusing the stale one, unlike other hosts whose capture is
        // already a durable, direct CDN file.
        assert_eq!(
            contextual_media_page(
                "https://playback.media-streaming.soundcloud.cloud/media/x/aac_96k/playlist.m3u8",
                Some("https://soundcloud.com/artist/track"),
                Some("audio"),
                false,
                Some("GET"),
            ),
            Some("https://soundcloud.com/artist/track"),
        );
        // A SoundCloud user/collection page (not a single artist/track path)
        // is not a specific-enough context to route to.
        assert_eq!(
            contextual_media_page(
                "https://playback.media-streaming.soundcloud.cloud/media/x/aac_96k/playlist.m3u8",
                Some("https://soundcloud.com/you/likes"),
                Some("audio"),
                false,
                Some("GET"),
            ),
            None,
        );
    }

    #[test]
    fn contextual_routing_rejects_unsafe_or_complete_downloads() {
        let resource = "https://cdn.example.test/track.mp4";
        for page in [
            "https://www.facebook.com/",
            "https://www.instagram.com/",
            "https://facebook.com.evil.example/reel/123/",
            "https://user:pass@www.instagram.com/reel/abc/",
            "https://example.test/reel/123/",
        ] {
            assert_eq!(
                contextual_media_page(resource, Some(page), Some("video"), false, Some("GET")),
                None
            );
        }
        for (kind, companion, method) in [
            (Some("video"), true, Some("GET")),
            (Some("image"), false, Some("GET")),
            (Some("video"), false, Some("POST")),
        ] {
            assert_eq!(
                contextual_media_page(
                    resource,
                    Some("https://www.instagram.com/reel/abc/"),
                    kind,
                    companion,
                    method,
                ),
                None,
            );
        }
        assert_eq!(
            contextual_media_page(
                "https://cdn.example.test/live.m3u8",
                Some("https://www.instagram.com/reel/abc/"),
                Some("video"),
                false,
                Some("GET"),
            ),
            None,
        );
    }
}
