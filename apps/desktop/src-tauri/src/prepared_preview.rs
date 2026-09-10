//! Prepare pages and separate tracks before handing an actual media file to a
//! player. A web page URL is never a playable-media success.
use super::{
    configured_tool, diagnostic_log, tiktok_preview, AppState, MediaPreviewRequest, UserSettings,
};
use apocalipse_core::{classify_url, contextual_media_page, DownloadEngine, DownloadKind};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Stdio,
    sync::atomic::{AtomicUsize, Ordering},
    time::Duration,
};
use tauri::{Emitter, Manager};

const MAX_BYTES: u64 = 1024 * 1024 * 1024;
static ACTIVE: AtomicUsize = AtomicUsize::new(0);

pub(super) fn is_candidate(request: &MediaPreviewRequest) -> bool {
    request.audio_url.is_some()
        || request.page_extractor
        || (!tiktok_preview::is_candidate(request)
            && classify_url(&request.url) == Some(DownloadKind::MediaPage))
}
fn is_page(request: &MediaPreviewRequest) -> bool {
    request.page_extractor
        || (request.audio_url.is_none()
            && !tiktok_preview::is_candidate(request)
            && classify_url(&request.url) == Some(DownloadKind::MediaPage))
}
fn validate_selection(request: &MediaPreviewRequest) -> Result<(), String> {
    tiktok_preview::validate(request)?;
    if is_page(request) {
        if request.audio_url.is_some() {
            return Err("preview_page_with_unrelated_audio".into());
        }
        // Reject social home/profile feeds rather than opening/extracting an
        // arbitrary first item. Short shared-video URLs are deliberately allowed.
        let url = url::Url::parse(&request.url).map_err(|_| "invalid_preview_url")?;
        let host = url.host_str().unwrap_or("");
        let short = matches!(host, "fb.watch" | "vm.tiktok.com" | "vt.tiktok.com")
            && !url.path().trim_matches('/').is_empty();
        let specific = contextual_media_page(
            "https://media.invalid/clip.mp4",
            Some(&request.url),
            Some("video"),
            false,
            None,
        )
        .is_some();
        if !specific && !short {
            return Err("preview_specific_video_required".into());
        }
    } else if request.audio_url.is_none() {
        return Err("preview_missing_media_pair".into());
    }
    Ok(())
}
struct Slot;
impl Slot {
    fn acquire() -> Result<Self, String> {
        ACTIVE
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |n| {
                (n < 4).then_some(n + 1)
            })
            .map(|_| Self)
            .map_err(|_| "too_many_media_previews".into())
    }
}
impl Drop for Slot {
    fn drop(&mut self) {
        ACTIVE.fetch_sub(1, Ordering::SeqCst);
    }
}
struct Work(PathBuf);
impl Work {
    fn create(parent: &Path) -> Result<Self, String> {
        fs::create_dir_all(parent).map_err(|_| "preview_directory_failed")?;
        let path = parent.join(uuid::Uuid::new_v4().to_string());
        let mut builder = fs::DirBuilder::new();
        #[cfg(unix)]
        {
            use std::os::unix::fs::DirBuilderExt;
            builder.mode(0o700);
        }
        builder
            .create(&path)
            .map_err(|_| "preview_directory_failed")?;
        Ok(Self(path))
    }
}
impl Drop for Work {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

pub(super) fn open(
    app: &tauri::AppHandle,
    state: &AppState,
    mut request: MediaPreviewRequest,
) -> Result<(), String> {
    validate_selection(&request)?;
    let settings = state
        .settings
        .lock()
        .map_err(|_| "preview_settings_unavailable")?
        .clone();
    if request.user_agent.as_deref().is_none_or(str::is_empty) {
        request.user_agent = settings.user_agent.clone();
    }
    tiktok_preview::validate(&request)?;
    let slot = Slot::acquire()?;
    let work = Work::create(
        &state
            .queue_path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join("previews"),
    )?;
    let app = app.clone();
    let trace = request
        .trace_id
        .clone()
        .filter(|id| uuid::Uuid::parse_str(id).is_ok())
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    diagnostic_log(
        state,
        "INFO",
        "media.preview_preparing",
        &format!(
            "trace={trace} route={} queue_entry=false",
            if is_page(&request) {
                "page_extractor"
            } else {
                "paired_media"
            }
        ),
    );
    let _ = app.emit("media-preview-preparing", ());
    tauri::async_runtime::spawn(async move {
        let result = tokio::time::timeout(
            Duration::from_secs(600),
            prepare(&request, &settings, &work.0),
        )
        .await
        .map_err(|_| "preview_preparation_timeout".to_owned())
        .and_then(|result| result);
        let media = match result {
            Ok(path) => path,
            Err(error) => {
                report_error(&app, &trace, &error);
                return;
            }
        };
        app.state::<AppState>().diagnostics.record(
            "preview.streams_verified",
            "INFO",
            Some(&trace),
            None,
            serde_json::json!({"verified":true,"pictureAndSoundNotObserved":true}),
        );
        // Remove the credential jar before starting an external player.
        let _ = fs::remove_file(work.0.join("cookies.txt"));
        let configured = settings
            .media_player_path
            .clone()
            .unwrap_or_else(|| PathBuf::from(if cfg!(windows) { "vlc.exe" } else { "vlc" }));
        let player = tiktok_preview::effective_player(&configured);
        let mut command = tiktok_preview::player_command(&player, &media.to_string_lossy());
        match command.spawn() {
            Ok(mut child) => {
                app.state::<AppState>().diagnostics.record(
                    "preview.player_process_started",
                    "INFO",
                    Some(&trace),
                    None,
                    serde_json::json!({"pid":child.id(),"playbackConfirmed":false}),
                );
                diagnostic_log(
                    &app.state::<AppState>(),
                    "INFO",
                    "media.preview_file_ready",
                    &format!("trace={trace} pid={} verified=true", child.id()),
                );
                let _ = app.emit("media-preview-ready", ());
                let owns = tiktok_preview::owns_player_process(&player);
                // Keep only our unique temporary directory alive. No queue entry,
                // destination mutation, or deletion of the user's own downloads.
                tauri::async_runtime::spawn_blocking(move || {
                    let status = child.wait();
                    if status.as_ref().is_ok_and(|status| !status.success()) {
                        report_error(&app, &trace, "preview_player_exited");
                    }
                    if !owns && status.as_ref().is_ok_and(|status| status.success()) {
                        tauri::async_runtime::spawn(async move {
                            tokio::time::sleep(Duration::from_secs(7200)).await;
                            drop(work);
                            drop(slot);
                        });
                    } else {
                        drop(work);
                        drop(slot);
                    }
                });
            }
            Err(error) => report_error(
                &app,
                &trace,
                &format!("preview_player_start_failed:{:?}", error.kind()),
            ),
        }
    });
    Ok(())
}
fn report_error(app: &tauri::AppHandle, trace: &str, message: &str) {
    // Never include subprocess stderr: it can contain signed URLs or cookies.
    diagnostic_log(
        &app.state::<AppState>(),
        "ERROR",
        "media.preview_error",
        &format!("trace={trace} {message}"),
    );
    super::show_main_window(app);
    let _ = app.emit("media-preview-error", message);
}
fn network(settings: &UserSettings) -> Result<reqwest::ClientBuilder, String> {
    DownloadEngine::network_client_builder(
        if settings.proxy_enabled {
            settings.proxy_url.as_deref()
        } else {
            None
        },
        if settings.proxy_enabled {
            settings.proxy_username.as_deref()
        } else {
            None
        },
        if settings.proxy_enabled {
            settings.proxy_password.as_deref()
        } else {
            None
        },
        if settings.dns_enabled {
            &settings.dns_servers
        } else {
            &[]
        },
    )
    .map_err(|_| "preview_network_configuration_failed".into())
}
fn process(path: &Path) -> tokio::process::Command {
    let mut command = tokio::process::Command::new(path);
    command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .kill_on_drop(true);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.as_std_mut().creation_flags(0x08000000);
    }
    command
}
async fn run(command: &mut tokio::process::Command, stage: &str) -> Result<(), String> {
    let status = command
        .status()
        .await
        .map_err(|error| format!("preview_{stage}_start_failed:{:?}", error.kind()))?;
    if !status.success() {
        return Err(format!(
            "preview_{stage}_failed:code={}",
            status.code().unwrap_or(-1)
        ));
    }
    Ok(())
}
fn page_command(
    request: &MediaPreviewRequest,
    settings: &UserSettings,
    directory: &Path,
) -> Result<tokio::process::Command, String> {
    let mut command = process(&configured_tool(
        &settings.yt_dlp_path,
        if cfg!(windows) {
            "yt-dlp.exe"
        } else {
            "yt-dlp"
        },
    ));
    command
        .args([
            "--ignore-config",
            "--no-playlist",
            "--no-cache-dir",
            "--no-progress",
            "--quiet",
            "--no-warnings",
            "--socket-timeout",
            "20",
            "--retries",
            "2",
            "--fragment-retries",
            "2",
            "--max-filesize",
            "1G",
            "-f",
            "bestvideo+bestaudio/best",
            "--merge-output-format",
            "mp4",
            "-o",
        ])
        .arg(directory.join("preview.%(ext)s"));
    if let Some(ffmpeg) = &settings.ffmpeg_path {
        command.arg("--ffmpeg-location").arg(ffmpeg);
    }
    let quickjs = configured_tool(
        &settings.qjs_path,
        if cfg!(windows) { "qjs.exe" } else { "qjs" },
    );
    command
        .arg("--js-runtimes")
        .arg(format!("quickjs:{}", quickjs.display()));
    if let Some(cookie) = request.cookie_header.as_deref().filter(|v| !v.is_empty()) {
        let jar = directory.join("cookies.txt");
        // Use a domain-scoped jar, never global Cookie headers across redirects.
        super::write_social_cookie_jar(&jar, &request.url, cookie)?;
        command.arg("--cookies").arg(jar);
    }
    if let Some(agent) = request.user_agent.as_deref() {
        command.arg("--user-agent").arg(agent);
    }
    if let Some(referer) = request.referer.as_deref() {
        command.arg("--referer").arg(referer);
    }
    if settings.proxy_enabled {
        if let Some(proxy) = settings.proxy_url.as_deref() {
            command.arg("--proxy").arg(super::external_proxy_url(
                proxy,
                settings.proxy_username.as_deref(),
                settings.proxy_password.as_deref(),
            ));
        }
    }
    if let Some(credential) = super::website_credential_for_url(settings, &request.url) {
        command
            .arg("--username")
            .arg(&credential.username)
            .arg("--password")
            .arg(&credential.password);
    }
    command.arg("--").arg(&request.url);
    Ok(command)
}
fn completed_file(directory: &Path) -> Result<PathBuf, String> {
    let candidates: Vec<_> = ["mp4", "mkv", "webm", "mov", "m4v"]
        .into_iter()
        .map(|extension| directory.join(format!("preview.{extension}")))
        .filter(|path| path.is_file())
        .collect();
    if candidates.len() != 1 {
        return Err("preview_final_file_missing_or_ambiguous".into());
    }
    let path = &candidates[0];
    let size = fs::metadata(path)
        .map_err(|_| "preview_file_unreadable")?
        .len();
    if size == 0 || size > MAX_BYTES {
        return Err("preview_size_limit".into());
    }
    Ok(path.clone())
}
async fn prepare(
    request: &MediaPreviewRequest,
    settings: &UserSettings,
    directory: &Path,
) -> Result<PathBuf, String> {
    let paired = !is_page(request);
    if paired {
        let mut audio = request.clone();
        audio.url = request
            .audio_url
            .clone()
            .ok_or("preview_missing_media_pair")?;
        audio.cookie_header = request.audio_cookie_header.clone();
        audio.audio_url = None;
        let video_file = directory.join("video.input");
        let audio_file = directory.join("audio.input");
        tokio::try_join!(
            tiktok_preview::download_to(
                request.clone(),
                network(settings)?,
                &video_file,
                MAX_BYTES / 2
            ),
            tiktok_preview::download_to(audio, network(settings)?, &audio_file, MAX_BYTES / 2),
        )?;
        let mut command = process(&configured_tool(
            &settings.ffmpeg_path,
            if cfg!(windows) {
                "ffmpeg.exe"
            } else {
                "ffmpeg"
            },
        ));
        command
            .args(["-nostdin", "-v", "error", "-y", "-i"])
            .arg(&video_file)
            .arg("-i")
            .arg(&audio_file)
            .args([
                "-map",
                "0:v:0",
                "-map",
                "1:a:0",
                "-c",
                "copy",
                "-movflags",
                "+faststart",
            ])
            .arg(directory.join("preview.mp4"));
        run(&mut command, "mux").await?;
        let _ = fs::remove_file(video_file);
        let _ = fs::remove_file(audio_file);
    } else {
        run(
            &mut page_command(request, settings, directory)?,
            "extractor",
        )
        .await?;
    }
    let path = completed_file(directory)?;
    let probe = if let Some(ffmpeg) = settings.ffmpeg_path.as_ref().filter(|p| p.is_absolute()) {
        ffmpeg.with_file_name(if cfg!(windows) {
            "ffprobe.exe"
        } else {
            "ffprobe"
        })
    } else {
        PathBuf::from(if cfg!(windows) {
            "ffprobe.exe"
        } else {
            "ffprobe"
        })
    };
    let mut command = process(&probe);
    command
        .args([
            "-v",
            "error",
            "-show_entries",
            "stream=codec_type",
            "-of",
            "json",
        ])
        .arg(&path);
    let output = tokio::time::timeout(Duration::from_secs(30), command.output())
        .await
        .map_err(|_| "preview_probe_timeout")?
        .map_err(|_| "preview_ffprobe_unavailable")?;
    if !output.status.success() {
        return Err("preview_invalid_media".into());
    }
    verify_streams(&output.stdout, paired, request.media_kind.as_deref())?;
    Ok(path)
}
fn verify_streams(bytes: &[u8], require_audio: bool, kind: Option<&str>) -> Result<(), String> {
    let value: serde_json::Value =
        serde_json::from_slice(bytes).map_err(|_| "preview_invalid_media")?;
    let streams = value["streams"].as_array().ok_or("preview_no_streams")?;
    let has = |kind: &str| {
        streams
            .iter()
            .any(|v| v["codec_type"].as_str() == Some(kind))
    };
    if kind == Some("audio") {
        if !has("audio") {
            return Err("preview_audio_missing".into());
        }
    } else if !has("video") {
        return Err("preview_video_missing".into());
    }
    if require_audio && !has("audio") {
        return Err("preview_audio_missing".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pages_and_paired_media_never_go_directly_to_the_player() {
        for url in [
            "https://www.facebook.com/reel/123",
            "https://www.tiktok.com/@synthetic/video/456",
            "https://www.instagram.com/reel/abc/",
        ] {
            let request = MediaPreviewRequest {
                url: url.into(),
                ..Default::default()
            };
            assert!(is_candidate(&request));
            assert!(validate_selection(&request).is_ok());
        }
        let request = MediaPreviewRequest {
            url: "https://v16.tiktok.com/video.mp4".into(),
            audio_url: Some("https://audio.tiktokcdn.com/track.mp4".into()),
            ..Default::default()
        };
        assert!(is_candidate(&request));
        assert!(!is_page(&request));
    }
    #[test]
    fn generic_feeds_and_unsafe_companion_urls_are_rejected() {
        for url in [
            "https://www.facebook.com/",
            "https://www.tiktok.com/",
            "https://www.instagram.com/",
        ] {
            let request = MediaPreviewRequest {
                url: url.into(),
                page_extractor: true,
                ..Default::default()
            };
            assert!(validate_selection(&request).is_err());
        }
        let request = MediaPreviewRequest {
            url: "https://cdn.example/video.mp4".into(),
            audio_url: Some("file:///secret".into()),
            ..Default::default()
        };
        assert!(validate_selection(&request).is_err());
    }
    #[test]
    fn paired_preview_requires_real_video_and_audio_streams() {
        assert!(verify_streams(
            br#"{"streams":[{"codec_type":"audio"}]}"#,
            true,
            Some("video")
        )
        .is_err());
        assert!(verify_streams(
            br#"{"streams":[{"codec_type":"video"}]}"#,
            true,
            Some("video")
        )
        .is_err());
        assert!(verify_streams(
            br#"{"streams":[{"codec_type":"video"},{"codec_type":"audio"}]}"#,
            true,
            Some("video")
        )
        .is_ok());
        assert!(verify_streams(
            br#"{"streams":[{"codec_type":"video"}]}"#,
            false,
            Some("video")
        )
        .is_ok());
    }
    #[test]
    fn temporary_work_never_selects_cookie_jars_or_partial_files() {
        let work = Work::create(&std::env::temp_dir().join("adm-preview-tests")).unwrap();
        let path = work.0.clone();
        fs::write(path.join("cookies.txt"), b"secret").unwrap();
        fs::write(path.join("preview.mp4.part"), b"partial").unwrap();
        assert!(completed_file(&path).is_err());
        fs::write(path.join("preview.mp4"), b"media").unwrap();
        assert_eq!(completed_file(&path).unwrap(), path.join("preview.mp4"));
        drop(work);
        assert!(!path.exists());
    }
}

#[cfg(all(test, target_os = "linux"))]
mod media_integration {
    use super::*;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    async fn source(bytes: Vec<u8>) -> String {
        let listener = tokio::net::TcpListener::bind(("127.0.0.1", 0))
            .await
            .unwrap();
        let url = format!("http://{}/media.mp4", listener.local_addr().unwrap());
        tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut request = [0; 4096];
            let _ = stream.read(&mut request).await;
            let header = format!("HTTP/1.1 200 OK\r\nContent-Type: video/mp4\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", bytes.len());
            stream.write_all(header.as_bytes()).await.unwrap();
            stream.write_all(&bytes).await.unwrap();
        });
        url
    }
    #[tokio::test]
    #[ignore = "requires FFmpeg/FFprobe; run explicitly by Linux CI"]
    async fn paired_preview_uses_real_mux_and_probe() {
        let work = Work::create(&std::env::temp_dir().join("adm-preview-integration")).unwrap();
        let video = work.0.join("source-video.mp4");
        let audio = work.0.join("source-audio.m4a");
        let mut generate = process(Path::new("/usr/bin/ffmpeg"));
        generate
            .args([
                "-y",
                "-f",
                "lavfi",
                "-i",
                "color=c=red:s=64x64:d=0.2",
                "-an",
                "-c:v",
                "mpeg4",
            ])
            .arg(&video);
        run(&mut generate, "fixture_video").await.unwrap();
        let mut generate = process(Path::new("/usr/bin/ffmpeg"));
        generate
            .args([
                "-y",
                "-f",
                "lavfi",
                "-i",
                "sine=frequency=1000:duration=0.2",
                "-vn",
                "-c:a",
                "aac",
            ])
            .arg(&audio);
        run(&mut generate, "fixture_audio").await.unwrap();
        let request = MediaPreviewRequest {
            url: source(fs::read(&video).unwrap()).await,
            audio_url: Some(source(fs::read(&audio).unwrap()).await),
            media_kind: Some("video".into()),
            ..Default::default()
        };
        let mut settings = UserSettings::default();
        settings.ffmpeg_path = Some("/usr/bin/ffmpeg".into());
        let output = prepare(&request, &settings, &work.0).await.unwrap();
        assert_eq!(output, work.0.join("preview.mp4"));
        assert!(fs::metadata(&output).unwrap().len() > 100);
        assert!(!work.0.join("video.input").exists());
        assert!(!work.0.join("audio.input").exists());
    }
}
