use apocalipse_core::{plan_download, Capabilities, Engine};

#[test]
fn facebook_page_handoffs_select_the_media_extractor() {
    for page in [
        "https://www.facebook.com/reel/123456789",
        "https://www.facebook.com/watch/?v=123456789",
        "https://www.facebook.com/share/r/SyntheticId/",
    ] {
        let plan = plan_download(
            page,
            Capabilities {
                yt_dlp: true,
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(plan.primary, Engine::YtDlp);
        assert_eq!(plan.reason, "media_extractor_available");
    }
}

#[test]
fn explicit_cdn_downloads_remain_native_http() {
    let plan = plan_download(
        "https://video.fbcdn.net/synthetic-track.mp4",
        Capabilities {
            yt_dlp: true,
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(plan.primary, Engine::NativeHttp);
    assert_eq!(plan.reason, "direct_http");
}
