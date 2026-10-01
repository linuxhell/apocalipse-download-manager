//! Convert a completed direct audio download without exposing partial output.
use super::*;

pub(super) struct Job {
    pub source: PathBuf,
    destination: PathBuf,
    format: String,
    ffmpeg: PathBuf,
}
pub(super) fn format(selection: Option<&str>) -> Option<&str> {
    selection
        .and_then(|s| s.strip_prefix("audio:"))
        .filter(|format| matches!(*format, "mp3" | "m4a" | "opus" | "flac" | "wav"))
}
pub(super) fn prepare(app: &tauri::AppHandle, id: DownloadId) -> Result<Option<Job>, String> {
    let state = app.state::<AppState>();
    let task = state
        .queue
        .lock()
        .map_err(|e| e.to_string())?
        .iter()
        .find(|task| task.id == id)
        .cloned()
        .ok_or("download_not_found")?;
    let Some(format) = format(task.format_selection.as_deref()) else {
        return Ok(None);
    };
    let settings = state.settings.lock().map_err(|e| e.to_string())?;
    let ffmpeg = configured_tool(
        &settings.ffmpeg_path,
        if cfg!(windows) {
            "ffmpeg.exe"
        } else {
            "ffmpeg"
        },
    );
    let directory = state
        .queue_path
        .parent()
        .unwrap_or(Path::new("."))
        .join("audio-work")
        .join(id.to_string());
    fs::create_dir_all(&directory).map_err(|e| e.to_string())?;
    Ok(Some(Job {
        source: directory.join("source.bin"),
        destination: task.destination,
        format: format.to_owned(),
        ffmpeg,
    }))
}
fn codec(format: &str) -> &'static [&'static str] {
    match format {
        "mp3" => &["-c:a", "libmp3lame", "-q:a", "2"],
        "m4a" => &["-c:a", "aac", "-b:a", "256k"],
        "opus" => &["-c:a", "libopus", "-b:a", "192k"],
        "flac" => &["-c:a", "flac"],
        "wav" => &["-c:a", "pcm_s16le"],
        _ => unreachable!("validated audio format"),
    }
}
pub(super) async fn convert(job: &Job) -> Result<u64, String> {
    let parent = job.destination.parent().ok_or("invalid_destination")?;
    fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    let temporary = tempfile::Builder::new()
        .prefix(".adm-audio-")
        .suffix(&format!(".{}", job.format))
        .tempfile_in(parent)
        .map_err(|e| e.to_string())?;
    let mut command = tokio::process::Command::new(&job.ffmpeg);
    command
        .args(["-hide_banner", "-nostdin", "-y", "-i"])
        .arg(&job.source)
        .args(["-map", "0:a:0", "-vn"])
        .args(codec(&job.format))
        .arg(temporary.path())
        .kill_on_drop(true);
    #[cfg(windows)]
    command.creation_flags(0x08000000);
    let output = command
        .output()
        .await
        .map_err(|e| format!("ffmpeg_audio_start_failed: {e}"))?;
    if !output.status.success() {
        return Err(format!(
            "ffmpeg_audio_conversion_failed: {}",
            String::from_utf8_lossy(&output.stderr)
                .chars()
                .take(2000)
                .collect::<String>()
        ));
    }
    let bytes = temporary
        .as_file()
        .metadata()
        .map_err(|e| e.to_string())?
        .len();
    if bytes == 0 {
        return Err("empty_audio_conversion".into());
    }
    temporary.as_file().sync_all().map_err(|e| e.to_string())?;
    temporary
        .persist_noclobber(&job.destination)
        .map_err(|e| e.to_string())?;
    if let Some(directory) = job.source.parent() {
        let _ = fs::remove_dir_all(directory);
    }
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    #[ignore = "requires FFmpeg; executed explicitly in Linux CI"]
    async fn direct_audio_conversion_produces_requested_format_and_preserves_existing_files() {
        let directory = tempfile::tempdir().unwrap();
        let input = directory.path().join("fixture.wav");
        let status = tokio::process::Command::new("ffmpeg")
            .args([
                "-hide_banner",
                "-nostdin",
                "-y",
                "-f",
                "lavfi",
                "-i",
                "sine=frequency=440:duration=0.1",
            ])
            .arg(&input)
            .status()
            .await
            .unwrap();
        assert!(status.success());
        for format in ["mp3", "m4a", "opus", "flac", "wav"] {
            let work = directory.path().join(format!("work-{format}"));
            fs::create_dir(&work).unwrap();
            let source = work.join("source.bin");
            fs::copy(&input, &source).unwrap();
            let destination = directory.path().join(format!("output.{format}"));
            let job = Job {
                source,
                destination: destination.clone(),
                format: format.into(),
                ffmpeg: "ffmpeg".into(),
            };
            assert!(convert(&job).await.unwrap() > 0);
            assert!(!work.exists());
            let original = fs::read(&destination).unwrap();
            fs::create_dir(&work).unwrap();
            fs::copy(&input, &job.source).unwrap();
            assert!(convert(&job).await.is_err());
            assert_eq!(fs::read(&destination).unwrap(), original);
            let probe = tokio::process::Command::new("ffprobe")
                .args([
                    "-v",
                    "error",
                    "-show_entries",
                    "stream=codec_type",
                    "-of",
                    "csv=p=0",
                ])
                .arg(&destination)
                .output()
                .await
                .unwrap();
            assert!(probe.status.success());
            assert_eq!(String::from_utf8_lossy(&probe.stdout).trim(), "audio");
        }
    }
}
