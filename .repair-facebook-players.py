from pathlib import Path
import hashlib


def checked(path, sha):
    p = Path(path)
    data = p.read_bytes()
    actual = hashlib.sha1(b'blob ' + str(len(data)).encode() + b'\0' + data).hexdigest()
    if actual != sha:
        raise RuntimeError('Source changed; refusing to overwrite: ' + path)
    return p, data.decode('utf-8')


def once(text, old, new):
    if text.count(old) != 1:
        raise RuntimeError('Unexpected source; aborting: ' + old[:100])
    return text.replace(old, new, 1)


p, s = checked('browser-extension/content.js', '515d559c2f783038fd47fcd7220fabab8aecc98a')
s = once(s, '''        const liveSource = String(element.currentSrc || element.src || "");''', '''        // The Reel/post permalink represents the complete video. A recent CDN
        // response can be only one DASH track (even when labelled video/mp4).
        // Use the same page-extractor route as the popup, before blobs or CDN URLs.
        const facebookPageUrl = isFacebookVideo
          ? [typeof resolved === "string" ? resolved : resolved?.url, visibleFacebookUrl]
            .find((value) => value && isFacebookMediaUrl(value)) || null
          : null;
        const liveSource = String(element.currentSrc || element.src || "");''')
s = once(s, '(isTikTokPage || (isFacebookVideo && isFacebookMediaUrl(location.href)))', '(isTikTokPage || (isFacebookVideo && !facebookPageUrl && isFacebookMediaUrl(location.href)))')
s = once(s, '        if (liveBlobUrl) {', '        if (liveBlobUrl && !facebookPageUrl) {')
s = once(s, '          ? (browserVideoMedia || ((typeof resolved === "string" && isFacebookMediaUrl(resolved))', '          ? (facebookPageUrl || browserVideoMedia || ((typeof resolved === "string" && isFacebookMediaUrl(resolved))')
s = once(s, '''        const requestUrls = [...new Set([
          ...(resolved?.requestUrls?.length''', '''        // Do not attach unrelated CDN audio to an extractor page task.
        const companionAudioUrl = isFacebookVideo && !facebookPageUrl ? browserAudioMedia : null;
        const requestUrls = facebookPageUrl ? [] : [...new Set([
          ...(resolved?.requestUrls?.length''')
s = once(s, '          pageFallback: Boolean(isFacebookVideo && isFacebookMediaUrl(currentUrl)),', '          pageFallback: Boolean(isFacebookVideo && isFacebookMediaUrl(currentUrl)),\n          facebookPageExtractorPreferred: Boolean(facebookPageUrl),')
s = once(s, 'audioUrl: isFacebookVideo ? browserAudioMedia : null', 'audioUrl: companionAudioUrl')
s = once(s, 'requestUrls: [...requestUrls, ...(browserAudioMedia ? [browserAudioMedia] : [])]', 'requestUrls: [...requestUrls, ...(companionAudioUrl ? [companionAudioUrl] : [])]')
s = once(s, 'title: isFacebookVideo ? facebookDownloadTitle(currentUrl) : document.title', 'title: facebookPageUrl ? titleFor(element) : (isFacebookVideo ? facebookDownloadTitle(currentUrl) : document.title)')
p.write_text(s, encoding='utf-8')

p, s = checked('apps/desktop/src-tauri/src/tiktok_preview.rs', '9c38218b855c5bd3c1f2b525775693e2a121dc1a')
s = once(s, 'pub(super) fn open(app:', '''// Only these directly launched binaries have a process lifetime we own.
// PotPlayer, portable launchers and other players may hand off to an existing
// window and exit successfully while that window still reads the local URL.
fn owns_player_process(player: &Path) -> bool {
    matches!(
        player.file_name().and_then(|name| name.to_str()).unwrap_or("").to_ascii_lowercase().as_str(),
        "vlc" | "vlc.exe" | "mpv" | "mpv.exe"
    )
}
async fn retain_handoff_session(mut keepalive: oneshot::Sender<()>) {
    // The relay still enforces idle/lifetime limits. Release this sender as soon
    // as the receiver closes; do not retain an unbounded background process.
    keepalive.closed().await;
}

pub(super) fn open(app:''')
s = once(s, '    tauri::async_runtime::spawn_blocking(move || {\n        match child.wait() {', '    let owns_process = owns_player_process(&player);\n    tauri::async_runtime::spawn_blocking(move || {\n        let outcome = child.wait();\n        let handed_off = !owns_process && outcome.as_ref().is_ok_and(|status| status.success());\n        match outcome {')
s = once(s, '        let _ = stop.send(());\n    });\n    Ok(())', '''        if handed_off {
            report("media.preview_handoff_retained", "waiting_for_local_transport_idle".into());
            tauri::async_runtime::spawn(retain_handoff_session(stop));
        } else {
            let _ = stop.send(());
        }
    });
    Ok(())''')
p.write_text(s, encoding='utf-8')

p = Path('apps/desktop/src-tauri/src/tiktok_preview/tests.rs')
s = p.read_text(encoding='utf-8')
if 'fn potplayer_arguments_use_local_url_without_vlc_flags' in s:
    raise RuntimeError('Player tests already exist')
s += r'''

#[test]
fn potplayer_arguments_use_local_url_without_vlc_flags() {
    for name in ["PotPlayerMini64.exe", "PotPlayerMini.exe", "PotPlayerPortable.exe"] {
        let player = Path::new(name);
        let target = "http://127.0.0.1:1234/synthetic-token/media";
        let command = player_command(player, target);
        let args = command.get_args().map(|value| value.to_string_lossy().into_owned()).collect::<Vec<_>>();
        assert_eq!(args, vec![target.to_owned()]);
        assert!(!owns_player_process(player));
        assert!(!format!("{command:?}").contains("signature="));
    }
    assert!(owns_player_process(Path::new("vlc.exe")));
    assert!(owns_player_process(Path::new("mpv.exe")));
    assert!(!owns_player_process(Path::new("VLCPortable.exe")));
}

#[tokio::test]
async fn successful_launcher_handoff_keeps_stream_alive_and_expires() {
    let _serial = SERIAL.lock().await;
    let origin = fixture(vec![response("200 OK", "Content-Type: video/mp4\r\n", "movie")]).await;
    let relay = Relay::bind(request(&origin.url), Client::builder().no_proxy()).unwrap();
    let target = relay.target.clone();
    let (sender, receiver) = oneshot::channel();
    let reporter: Reporter = Arc::new(|_, _| {});
    let server = tokio::spawn(relay.run(receiver, reporter, Duration::from_millis(500), Duration::from_secs(5)));
    // Simulate a successful launcher exit before the existing player requests data.
    let keeper = tokio::spawn(retain_handoff_session(sender));
    tokio::time::sleep(Duration::from_millis(20)).await;
    let answer = client().get(&target).send().await.unwrap();
    assert_eq!(answer.status(), StatusCode::OK);
    assert_eq!(answer.text().await.unwrap(), "movie");
    tokio::time::timeout(Duration::from_secs(6), server).await.unwrap().unwrap();
    tokio::time::timeout(Duration::from_secs(1), keeper).await.unwrap().unwrap();
    assert!(client().get(&target).send().await.is_err());
}
'''
p.write_text(s, encoding='utf-8')

p = Path('crates/apocalipse-core/tests/facebook_page_strategy.rs')
if p.exists():
    raise RuntimeError('Facebook strategy tests already exist')
p.parent.mkdir(parents=True, exist_ok=True)
p.write_text('''use apocalipse_core::{plan_download, Capabilities, Engine};

#[test]
fn facebook_page_handoffs_select_the_media_extractor() {
    for page in [
        "https://www.facebook.com/reel/123456789",
        "https://www.facebook.com/watch/?v=123456789",
        "https://www.facebook.com/share/r/SyntheticId/",
    ] {
        let plan = plan_download(page, Capabilities { yt_dlp: true, ..Default::default() }).unwrap();
        assert_eq!(plan.primary, Engine::YtDlp);
        assert_eq!(plan.reason, "media_extractor_available");
    }
}

#[test]
fn explicit_cdn_downloads_remain_native_http() {
    let plan = plan_download("https://video.fbcdn.net/synthetic-track.mp4", Capabilities { yt_dlp: true, ..Default::default() }).unwrap();
    assert_eq!(plan.primary, Engine::NativeHttp);
    assert_eq!(plan.reason, "direct_http");
}
''', encoding='utf-8')
