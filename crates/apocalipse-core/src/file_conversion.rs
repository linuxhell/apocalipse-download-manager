//! Stream-aware local conversion. Sources and existing destinations are never replaced.
use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::HashSet,
    path::{Path, PathBuf},
    process::Stdio,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, BufReader},
    process::Command,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Track {
    pub index: u32,
    pub kind: String,
    pub codec: String,
    pub language: String,
    pub attached_picture: bool,
    pub sample_format: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Inspection {
    pub source: PathBuf,
    pub kind: String,
    pub duration: Option<f64>,
    pub tracks: Vec<Track>,
    pub black_video: bool,
    pub formats: Vec<String>,
}
#[derive(Clone)]
pub struct Tools {
    pub ffmpeg: PathBuf,
    pub ffprobe: PathBuf,
}
fn command(path: &Path) -> Command {
    let mut cmd = Command::new(path);
    cmd.kill_on_drop(true).stdin(Stdio::null());
    #[cfg(windows)]
    cmd.creation_flags(0x08000000);
    cmd
}
fn check_cancel(cancel: &AtomicBool) -> Result<()> {
    if cancel.load(Ordering::Relaxed) {
        bail!("conversion_cancelled");
    }
    Ok(())
}
async fn cancelled(cancel: &AtomicBool) {
    while !cancel.load(Ordering::Relaxed) {
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}
async fn capture(mut cmd: Command, cancel: &AtomicBool) -> Result<Vec<u8>> {
    cmd.stdout(Stdio::piped()).stderr(Stdio::null());
    let mut child = cmd.spawn().context("conversion_tool_unavailable")?;
    let mut reader = child
        .stdout
        .take()
        .context("conversion_pipe_failed")?
        .take(4 * 1024 * 1024 + 1);
    let mut bytes = Vec::new();
    tokio::select! {
        _ = cancelled(cancel) => { let _ = child.kill().await; bail!("conversion_cancelled"); }
        result = reader.read_to_end(&mut bytes) => { result?; }
    }
    if bytes.len() > 4 * 1024 * 1024 {
        let _ = child.kill().await;
        bail!("conversion_probe_too_large");
    }
    let status = tokio::select! {
        _ = cancelled(cancel) => { let _ = child.kill().await; bail!("conversion_cancelled"); }
        status = child.wait() => status?,
    };
    if !status.success() {
        bail!("conversion_probe_failed");
    }
    Ok(bytes)
}
fn parse_probe(source: PathBuf, value: &Value) -> Result<Inspection> {
    let tracks: Vec<_> = value["streams"]
        .as_array()
        .context("conversion_unsupported")?
        .iter()
        .map(|s| Track {
            index: s["index"].as_u64().unwrap_or(0) as u32,
            kind: s["codec_type"].as_str().unwrap_or("").into(),
            codec: s["codec_name"].as_str().unwrap_or("unknown").into(),
            language: s["tags"]["language"].as_str().unwrap_or("").into(),
            sample_format: s["sample_fmt"].as_str().unwrap_or("").into(),
            attached_picture: s["disposition"]["attached_pic"].as_u64() == Some(1),
        })
        .collect();
    let audio = tracks.iter().any(|t| t.kind == "audio");
    let video = tracks
        .iter()
        .any(|t| t.kind == "video" && !t.attached_picture);
    let demux = value["format"]["format_name"].as_str().unwrap_or("");
    // Detect images from the probed container/codec, never from the filename alone.
    let brand = value["format"]["tags"]["major_brand"]
        .as_str()
        .unwrap_or("")
        .trim();
    let animated = matches!(demux, "gif" | "webp_pipe")
        && value["streams"].as_array().is_some_and(|streams| {
            streams.iter().any(|s| {
                s["nb_frames"]
                    .as_str()
                    .and_then(|n| n.parse::<u64>().ok())
                    .is_some_and(|n| n > 1)
            })
        });
    let image = !audio
        && video
        && !animated
        && (demux.ends_with("_pipe")
            || matches!(demux, "image2" | "gif" | "ico" | "avif")
            || matches!(brand, "avif" | "avis" | "heic" | "heix" | "mif1"));
    let kind = if image {
        "image"
    } else if video {
        "video"
    } else if audio {
        "audio"
    } else {
        bail!("conversion_unsupported")
    };
    let duration = value["format"]["duration"]
        .as_str()
        .and_then(|d| d.parse::<f64>().ok())
        .filter(|d| d.is_finite() && *d > 0.0);
    Ok(Inspection {
        source,
        kind: kind.into(),
        duration,
        tracks,
        black_video: false,
        formats: vec![],
    })
}
// Fully decode each video track. Any nonblack frame preserves video options. No sampling,
// downscaling or duration guesses that could discard a later scene or tiny visible object.
async fn all_black(tools: &Tools, source: &Path, index: u32, cancel: &AtomicBool) -> Result<bool> {
    let mut cmd = command(&tools.ffmpeg);
    cmd.args(["-hide_banner", "-v", "error", "-nostdin", "-xerror", "-i"]).arg(source)
        .args(["-map", &format!("0:{index}"), "-an", "-sn", "-dn", "-vf", "format=yuv444p,signalstats,metadata=mode=select:key=lavfi.signalstats.YMAX:value=16.5:function=greater,metadata=mode=print:key=lavfi.signalstats.YMAX:file=-", "-f", "null", "-"])
        .stdout(Stdio::piped()).stderr(Stdio::null());
    let mut child = cmd.spawn().context("conversion_tool_unavailable")?;
    let mut stdout = child.stdout.take().context("conversion_pipe_failed")?;
    let mut byte = [0];
    let read = tokio::select! {
        _ = cancelled(cancel) => { let _ = child.kill().await; bail!("conversion_cancelled"); }
        read = stdout.read(&mut byte) => read?,
    };
    if read > 0 {
        let _ = child.kill().await;
        return Ok(false);
    }
    let status = child.wait().await?;
    // Decode errors cannot prove that a video is empty.
    Ok(status.success())
}
fn available_names(bytes: &[u8]) -> HashSet<String> {
    String::from_utf8_lossy(bytes)
        .lines()
        .filter_map(|line| {
            let mut fields = line.split_whitespace();
            let flags = fields.next()?;
            if flags == "--" || flags.contains('=') {
                return None;
            }
            fields.next().map(str::to_owned)
        })
        .flat_map(|names| names.split(',').map(str::to_owned).collect::<Vec<_>>())
        .collect()
}
fn profiles(
    info: &Inspection,
    encoders: &HashSet<String>,
    muxers: &HashSet<String>,
) -> Vec<String> {
    let mut result = vec![];
    let mut add = |id: &str, encoder: &str, muxer: &str| {
        if encoders.contains(encoder) && muxers.contains(muxer) {
            result.push(id.into());
        }
    };
    if info.kind == "image" {
        for (id, encoder) in [
            ("png", "png"),
            ("webp", "libwebp"),
            ("jpg", "mjpeg"),
            ("tiff", "tiff"),
            ("bmp", "bmp"),
        ] {
            add(id, encoder, "image2");
        }
    } else {
        if info.kind == "video" {
            for (id, encoder, muxer) in [
                ("mp4", "libx264", "mp4"),
                ("mkv", "libx264", "matroska"),
                ("mkv-lossless", "ffv1", "matroska"),
                ("webm", "libvpx-vp9", "webm"),
            ] {
                let audio_encoder = if id == "webm" {
                    "libopus"
                } else if id == "mkv-lossless" {
                    "flac"
                } else {
                    "aac"
                };
                if !info.tracks.iter().any(|t| t.kind == "audio")
                    || encoders.contains(audio_encoder)
                {
                    add(id, encoder, muxer);
                }
            }
        }
        if info.tracks.iter().any(|t| t.kind == "audio") {
            for (id, encoder, muxer) in [
                ("flac", "flac", "flac"),
                ("wav", "pcm_s32le", "wav"),
                ("mp3", "libmp3lame", "mp3"),
                ("m4a", "aac", "ipod"),
                ("opus", "libopus", "opus"),
                ("ogg", "libvorbis", "ogg"),
            ] {
                add(id, encoder, muxer);
            }
        }
    }
    result
}
pub async fn inspect(tools: &Tools, source: &Path, cancel: Arc<AtomicBool>) -> Result<Inspection> {
    check_cancel(&cancel)?;
    let source = tokio::fs::canonicalize(source)
        .await
        .context("conversion_source_missing")?;
    if !source.is_file() {
        bail!("conversion_source_missing");
    }
    let mut cmd = command(&tools.ffprobe);
    cmd.args(["-v", "error", "-show_entries", "format=format_name,duration:stream=index,codec_type,codec_name,sample_fmt,nb_frames:format_tags=major_brand:stream_disposition=attached_pic:stream_tags=language", "-of", "json", "-i"]).arg(&source);
    let mut info = parse_probe(
        source,
        &serde_json::from_slice(&capture(cmd, &cancel).await?)?,
    )?;
    if info.kind == "video" && info.tracks.iter().any(|t| t.kind == "audio") {
        let mut black = true;
        for track in info
            .tracks
            .iter()
            .filter(|t| t.kind == "video" && !t.attached_picture)
        {
            if !all_black(tools, &info.source, track.index, &cancel).await? {
                black = false;
                break;
            }
        }
        info.black_video = black;
        if black {
            info.kind = "audio".into();
        }
    }
    let mut cmd = command(&tools.ffmpeg);
    cmd.args(["-hide_banner", "-encoders"]);
    let encoders = available_names(&capture(cmd, &cancel).await?);
    let mut cmd = command(&tools.ffmpeg);
    cmd.args(["-hide_banner", "-muxers"]);
    let muxers = available_names(&capture(cmd, &cancel).await?);
    info.formats = profiles(&info, &encoders, &muxers);
    if info.formats.is_empty() {
        bail!("conversion_no_encoders");
    }
    Ok(info)
}
pub fn extension(format: &str) -> &str {
    if format == "mkv-lossless" {
        "mkv"
    } else {
        format
    }
}
fn arguments(info: &Inspection, format: &str, audio_track: Option<u32>) -> Result<Vec<String>> {
    if !info.formats.iter().any(|f| f == format) {
        bail!("conversion_unsupported_format");
    }
    let mut args: Vec<String> = vec![];
    let mut add = |items: &[&str]| args.extend(items.iter().map(|s| s.to_string()));
    let audio_output = matches!(format, "flac" | "wav" | "mp3" | "m4a" | "opus" | "ogg");
    if audio_output {
        let track = audio_track
            .and_then(|index| {
                info.tracks
                    .iter()
                    .find(|t| t.index == index && t.kind == "audio")
            })
            .or_else(|| {
                if audio_track.is_none() {
                    info.tracks.iter().find(|t| t.kind == "audio")
                } else {
                    None
                }
            })
            .context("conversion_audio_missing")?;
        add(&["-map", &format!("0:{}", track.index), "-vn", "-sn", "-dn"]);
        add(match format {
            "flac" => &["-c:a", "flac"],
            "wav"
                if track.sample_format.starts_with("flt")
                    || track.sample_format.starts_with("dbl") =>
            {
                &["-c:a", "pcm_f64le"]
            }
            "wav" => &["-c:a", "pcm_s32le"],
            "mp3" => &["-c:a", "libmp3lame", "-q:a", "0"],
            "m4a" => &["-c:a", "aac", "-b:a", "256k"],
            "opus" => &["-c:a", "libopus", "-b:a", "192k"],
            _ => &["-c:a", "libvorbis", "-q:a", "8"],
        });
    } else {
        let track = info
            .tracks
            .iter()
            .find(|t| t.kind == "video" && !t.attached_picture)
            .context("conversion_video_missing")?;
        let map = if info.kind == "image" {
            format!("0:{}", track.index)
        } else {
            "0:V".into()
        };
        add(&["-map", &map, "-sn", "-dn"]);
        if info.kind == "image" {
            add(&["-frames:v", "1", "-update", "1", "-an"]);
            add(match format {
                "png" => &["-c:v", "png"],
                "webp" => &["-c:v", "libwebp", "-lossless", "1"],
                "jpg" => &["-c:v", "mjpeg", "-q:v", "1"],
                "tiff" => &["-c:v", "tiff"],
                _ => &["-c:v", "bmp"],
            });
        } else {
            add(&["-map", "0:a?", "-map_metadata", "0"]);
            add(match format {
                "mkv-lossless" => &["-c:v", "ffv1", "-level", "3", "-c:a", "flac"],
                "webm" => &[
                    "-c:v",
                    "libvpx-vp9",
                    "-crf",
                    "18",
                    "-b:v",
                    "0",
                    "-c:a",
                    "libopus",
                    "-b:a",
                    "192k",
                ],
                _ => &[
                    "-c:v", "libx264", "-preset", "medium", "-crf", "18", "-pix_fmt", "yuv420p",
                    "-c:a", "aac", "-b:a", "256k",
                ],
            });
            if format == "mp4" {
                add(&["-movflags", "+faststart"]);
            }
        }
    }
    if format == "wav" {
        add(&["-rf64", "auto"]);
    }
    Ok(args)
}
pub async fn convert(
    tools: &Tools,
    info: &Inspection,
    format: &str,
    audio_track: Option<u32>,
    directory: &Path,
    cancel: Arc<AtomicBool>,
    progress: impl Fn(f64) + Send,
) -> Result<PathBuf> {
    check_cancel(&cancel)?;
    let args = arguments(info, format, audio_track)?;
    let directory = tokio::fs::canonicalize(directory)
        .await
        .context("conversion_directory_missing")?;
    if !directory.is_dir() {
        bail!("conversion_directory_missing");
    }
    let temporary = tempfile::Builder::new()
        .prefix(".adm-convert-")
        .suffix(&format!(".{}", extension(format)))
        .tempfile_in(&directory)?;
    let mut cmd = command(&tools.ffmpeg);
    cmd.args([
        "-hide_banner",
        "-v",
        "error",
        "-nostdin",
        "-y",
        "-xerror",
        "-progress",
        "pipe:1",
        "-nostats",
        "-i",
    ])
    .arg(&info.source)
    .args(args)
    .arg(temporary.path())
    .stdout(Stdio::piped())
    .stderr(Stdio::piped());
    let mut child = cmd.spawn().context("conversion_tool_unavailable")?;
    let stderr = child.stderr.take().context("conversion_pipe_failed")?;
    let error_reader = tokio::spawn(async move {
        let mut stderr = stderr;
        let mut tail = Vec::new();
        let mut chunk = [0; 4096];
        while let Ok(n) = stderr.read(&mut chunk).await {
            if n == 0 {
                break;
            }
            tail.extend_from_slice(&chunk[..n]);
            if tail.len() > 8192 {
                tail.drain(..tail.len() - 8192);
            }
        }
        String::from_utf8_lossy(&tail).into_owned()
    });
    let mut lines = BufReader::new(child.stdout.take().context("conversion_pipe_failed")?).lines();
    loop {
        let line = tokio::select! {
            _ = cancelled(&cancel) => { let _ = child.kill().await; let _ = error_reader.await; bail!("conversion_cancelled"); }
            line = lines.next_line() => line?,
        };
        let Some(line) = line else { break };
        if let (Some(duration), Some(time)) = (
            info.duration,
            line.strip_prefix("out_time_us=")
                .and_then(|s| s.parse::<f64>().ok()),
        ) {
            progress((time / 1_000_000.0 / duration * 100.0).clamp(0.0, 99.0));
        }
    }
    let status = tokio::select! {
        _ = cancelled(&cancel) => { let _ = child.kill().await; let _ = error_reader.await; bail!("conversion_cancelled"); }
        status = child.wait() => status?,
    };
    let error = error_reader.await.unwrap_or_default();
    if !status.success() {
        bail!("conversion_failed: {}", error.trim());
    }
    check_cancel(&cancel)?;
    if temporary.as_file().metadata()?.len() == 0 {
        bail!("conversion_empty_output");
    }
    temporary.as_file().sync_all()?;
    let stem = info
        .source
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy();
    let mut file = temporary;
    for index in 0..10_000 {
        let suffix = if index == 0 {
            String::new()
        } else {
            format!(" ({index})")
        };
        let destination = directory.join(format!("{stem}.converted{suffix}.{}", extension(format)));
        match file.persist_noclobber(&destination) {
            Ok(_) => {
                progress(100.0);
                return Ok(destination);
            }
            Err(e) if e.error.kind() == std::io::ErrorKind::AlreadyExists => file = e.file,
            Err(e) => return Err(e.error.into()),
        }
    }
    bail!("conversion_name_conflict")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    fn flag() -> Arc<AtomicBool> {
        Arc::new(AtomicBool::new(false))
    }
    #[test]
    fn cover_art_is_not_video_and_misleading_extensions_do_not_decide_type() {
        let audio = parse_probe("song.mp4".into(), &json!({"streams":[{"index":0,"codec_type":"audio","codec_name":"aac"},{"index":1,"codec_type":"video","codec_name":"mjpeg","disposition":{"attached_pic":1}}],"format":{"format_name":"mov,mp4,m4a,3gp,3g2,mj2"}})).unwrap();
        assert_eq!(audio.kind, "audio");
        let picture = parse_probe("picture.mp3".into(), &json!({"streams":[{"index":0,"codec_type":"video","codec_name":"png"}],"format":{"format_name":"png_pipe"}})).unwrap();
        assert_eq!(picture.kind, "image");
    }
    #[tokio::test]
    #[ignore = "requires FFmpeg and FFprobe; executed explicitly in Linux CI"]
    async fn smart_file_conversion_detects_streams_and_preserves_sources() {
        let tools = Tools {
            ffmpeg: "ffmpeg".into(),
            ffprobe: "ffprobe".into(),
        };
        let directory = tempfile::tempdir().unwrap();
        async fn fixture(tools: &Tools, path: &Path, inputs: &[&str], output: &[&str]) {
            assert!(command(&tools.ffmpeg)
                .args(["-hide_banner", "-v", "error", "-y"])
                .args(inputs)
                .args(output)
                .arg(path)
                .status()
                .await
                .unwrap()
                .success());
        }
        let audio = directory.path().join("audio.mp4");
        fixture(
            &tools,
            &audio,
            &["-f", "lavfi", "-i", "sine=frequency=440:duration=0.25"],
            &["-c:a", "aac"],
        )
        .await;
        let info = inspect(&tools, &audio, flag()).await.unwrap();
        assert_eq!(info.kind, "audio");
        assert!(!info.formats.contains(&"mp4".into()));
        let source_bytes = tokio::fs::read(&audio).await.unwrap();
        for format in ["flac", "wav", "mp3", "m4a", "opus", "ogg"] {
            assert!(
                info.formats.contains(&format.to_string()),
                "missing {format}"
            );
            let path = convert(
                &tools,
                &info,
                format,
                None,
                directory.path(),
                flag(),
                |_| {},
            )
            .await
            .unwrap();
            let result = inspect(&tools, &path, flag()).await.unwrap();
            assert_eq!(result.kind, "audio");
        }
        let first = convert(
            &tools,
            &info,
            "flac",
            None,
            directory.path(),
            flag(),
            |_| {},
        )
        .await
        .unwrap();
        let first_bytes = tokio::fs::read(&first).await.unwrap();
        let next = convert(
            &tools,
            &info,
            "flac",
            None,
            directory.path(),
            flag(),
            |_| {},
        )
        .await
        .unwrap();
        assert_ne!(first, next);
        assert_eq!(tokio::fs::read(&first).await.unwrap(), first_bytes);
        assert_eq!(tokio::fs::read(&audio).await.unwrap(), source_bytes);
        let cancel_during_work = flag();
        let stopped = cancel_during_work.clone();
        let cancelled_result = convert(
            &tools,
            &info,
            "flac",
            None,
            directory.path(),
            cancel_during_work,
            move |_| {
                stopped.store(true, Ordering::Relaxed);
            },
        )
        .await;
        assert!(cancelled_result
            .unwrap_err()
            .to_string()
            .contains("cancelled"));

        assert!(
            convert(&tools, &info, "mp4", None, directory.path(), flag(), |_| {})
                .await
                .is_err()
        );
        assert!(convert(
            &tools,
            &info,
            "mp3",
            Some(9),
            directory.path(),
            flag(),
            |_| {}
        )
        .await
        .is_err());
        let cancel = flag();
        cancel.store(true, Ordering::Relaxed);
        assert!(convert(
            &tools,
            &info,
            "flac",
            None,
            directory.path(),
            cancel,
            |_| {}
        )
        .await
        .unwrap_err()
        .to_string()
        .contains("cancelled"));
        let black = directory.path().join("black.mp4");
        fixture(
            &tools,
            &black,
            &[
                "-f",
                "lavfi",
                "-i",
                "color=c=black:s=32x32:d=0.25",
                "-f",
                "lavfi",
                "-i",
                "sine=duration=0.25",
            ],
            &["-c:v", "libx264", "-c:a", "aac", "-shortest"],
        )
        .await;
        let black = inspect(&tools, &black, flag()).await.unwrap();
        assert_eq!(black.kind, "audio");
        assert!(black.black_video);
        assert!(!black.formats.contains(&"mp4".into()));
        let silent = directory.path().join("silent.mp4");
        fixture(
            &tools,
            &silent,
            &["-f", "lavfi", "-i", "color=c=red:s=32x32:d=0.25"],
            &["-c:v", "libx264"],
        )
        .await;
        let silent = inspect(&tools, &silent, flag()).await.unwrap();
        assert_eq!(silent.kind, "video");
        assert!(silent.formats.contains(&"mp4".into()));
        assert!(!silent.formats.contains(&"mp3".into()));
        let mixed = directory.path().join("late-scene.mp4");
        fixture(
            &tools,
            &mixed,
            &[
                "-f",
                "lavfi",
                "-i",
                "color=c=black:s=32x32:d=0.25",
                "-f",
                "lavfi",
                "-i",
                "color=c=red:s=32x32:d=0.25",
                "-f",
                "lavfi",
                "-i",
                "sine=duration=0.5",
            ],
            &[
                "-filter_complex",
                "[0:v][1:v]concat=n=2:v=1:a=0[v]",
                "-map",
                "[v]",
                "-map",
                "2:a",
                "-c:v",
                "libx264",
                "-c:a",
                "aac",
                "-shortest",
            ],
        )
        .await;
        let mixed = inspect(&tools, &mixed, flag()).await.unwrap();
        assert_eq!(mixed.kind, "video");
        assert!(!mixed.black_video);
        assert!(mixed.formats.contains(&"mp3".into()));
        for format in ["mp4", "mkv", "mkv-lossless", "webm"] {
            let path = convert(
                &tools,
                &mixed,
                format,
                None,
                directory.path(),
                flag(),
                |_| {},
            )
            .await
            .unwrap();
            let result = inspect(&tools, &path, flag()).await.unwrap();
            assert_eq!(result.kind, "video");
            assert!(result.tracks.iter().any(|t| t.kind == "audio"));
        }
        let image = directory.path().join("image.png");
        fixture(
            &tools,
            &image,
            &["-f", "lavfi", "-i", "color=c=red:s=32x32"],
            &["-frames:v", "1", "-threads", "1"],
        )
        .await;
        let wrong_extension = directory.path().join("image.mp3");
        tokio::fs::copy(&image, &wrong_extension).await.unwrap();
        let image = inspect(&tools, &wrong_extension, flag()).await.unwrap();
        assert_eq!(image.kind, "image");
        for format in ["png", "webp", "jpg", "tiff", "bmp"] {
            let path = convert(
                &tools,
                &image,
                format,
                None,
                directory.path(),
                flag(),
                |_| {},
            )
            .await
            .unwrap();
            assert_eq!(inspect(&tools, &path, flag()).await.unwrap().kind, "image");
        }
        assert!(!std::fs::read_dir(directory.path()).unwrap().any(|e| e
            .unwrap()
            .file_name()
            .to_string_lossy()
            .starts_with(".adm-convert-")));
    }
}
