use anyhow::{bail, Context, Result};
use std::path::{Path, PathBuf};
use tokio::process::Command;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConversionMode {
    /// Repackages the streams without changing their quality.
    Remux,
    /// Re-encodes video to H.264 and audio to AAC for broad MP4 compatibility.
    Compatible,
}

#[derive(Debug, Clone)]
pub struct TsToMp4Request {
    pub ffmpeg: PathBuf,
    pub input: PathBuf,
    pub output: PathBuf,
    pub mode: ConversionMode,
    pub overwrite: bool,
}

impl TsToMp4Request {
    pub fn validate(&self) -> Result<()> {
        if !self.input.is_file() {
            bail!("input TS file does not exist: {}", self.input.display());
        }
        if !has_extension(&self.input, "ts") {
            bail!("input must use the .ts extension");
        }
        if !has_extension(&self.output, "mp4") {
            bail!("output must use the .mp4 extension");
        }
        if self.input == self.output {
            bail!("input and output paths must be different");
        }
        if self.output.exists() && !self.overwrite {
            bail!("output already exists");
        }
        Ok(())
    }

    pub fn arguments(&self) -> Vec<String> {
        let mut args = vec![
            if self.overwrite { "-y" } else { "-n" }.into(),
            "-hide_banner".into(),
            "-nostdin".into(),
            "-i".into(),
            self.input.to_string_lossy().into_owned(),
        ];
        match self.mode {
            ConversionMode::Remux => args.extend(["-map", "0", "-c", "copy", "-movflags", "+faststart", "-avoid_negative_ts", "make_zero"].map(str::to_owned)),
            ConversionMode::Compatible => args.extend(["-map", "0:v:0?", "-map", "0:a:0?", "-c:v", "libx264", "-preset", "medium", "-crf", "20", "-c:a", "aac", "-b:a", "192k", "-movflags", "+faststart"].map(str::to_owned)),
        }
        args.push(self.output.to_string_lossy().into_owned());
        args
    }
}

pub async fn convert_ts_to_mp4(request: &TsToMp4Request) -> Result<()> {
    request.validate()?;
    if let Some(parent) = request.output.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    let output = Command::new(&request.ffmpeg)
        .args(request.arguments())
        .kill_on_drop(true)
        .output()
        .await
        .with_context(|| format!("could not start FFmpeg at {}", request.ffmpeg.display()))?;
    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        bail!("FFmpeg conversion failed: {}", error.trim());
    }
    if !request.output.is_file() {
        bail!("FFmpeg finished without creating the MP4 file");
    }
    Ok(())
}

fn has_extension(path: &Path, expected: &str) -> bool {
    path.extension().and_then(|value| value.to_str()).is_some_and(|value| value.eq_ignore_ascii_case(expected))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn remux_keeps_streams_and_enables_fast_start() {
        let request = TsToMp4Request { ffmpeg: "ffmpeg".into(), input: "movie.ts".into(), output: "movie.mp4".into(), mode: ConversionMode::Remux, overwrite: false };
        let args = request.arguments();
        assert!(args.windows(2).any(|pair| pair[0] == "-c" && pair[1] == "copy"));
        assert!(args.iter().any(|arg| arg == "+faststart"));
        assert_eq!(args.last().map(String::as_str), Some("movie.mp4"));
    }

    #[test]
    fn compatibility_mode_uses_h264_and_aac() {
        let request = TsToMp4Request { ffmpeg: "ffmpeg".into(), input: "movie.ts".into(), output: "movie.mp4".into(), mode: ConversionMode::Compatible, overwrite: true };
        let args = request.arguments();
        assert!(args.iter().any(|arg| arg == "libx264"));
        assert!(args.iter().any(|arg| arg == "aac"));
        assert_eq!(args.first().map(String::as_str), Some("-y"));
    }
}
