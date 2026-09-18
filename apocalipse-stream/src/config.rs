use serde::{Deserialize, Serialize};
use std::{fs, path::{Path, PathBuf}};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VideoPreset {
    P720,
    P1080,
    P4k,
    Shorts1080,
}

impl VideoPreset {
    pub fn label(self) -> &'static str {
        match self {
            Self::P720 => "720p (1280x720)",
            Self::P1080 => "1080p (1920x1080)",
            Self::P4k => "4K (3840x2160)",
            Self::Shorts1080 => "Shorts 1080x1920 (9:16)",
        }
    }

    pub fn dimensions(self) -> (u32, u32) {
        match self {
            Self::P720 => (1280, 720),
            Self::P1080 => (1920, 1080),
            Self::P4k => (3840, 2160),
            Self::Shorts1080 => (1080, 1920),
        }
    }

    pub fn vertical(self) -> bool {
        matches!(self, Self::Shorts1080)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub obs_path: String,
    pub ws_port: u16,
    pub ws_password: String,
    pub presenter_name: String,
    pub overlay_theme: String,
    pub preset: VideoPreset,
    pub fps: u32,
    pub record_dir: String,
    pub stream_server: String,
    pub stream_key: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            obs_path: String::new(),
            ws_port: 4455,
            ws_password: String::new(),
            presenter_name: "Apocalipse Stream".to_owned(),
            overlay_theme: "Eclipse".to_owned(),
            preset: VideoPreset::P1080,
            fps: 60,
            record_dir: data_dir().join("recordings").display().to_string(),
            stream_server: "rtmps://a.rtmps.youtube.com/live2".to_owned(),
            stream_key: String::new(),
        }
    }
}

pub fn app_dir() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(Path::to_path_buf))
        .unwrap_or_else(|| PathBuf::from("."))
}

pub fn data_dir() -> PathBuf {
    app_dir().join("data")
}

pub fn settings_path() -> PathBuf {
    data_dir().join("settings.json")
}

pub fn load_settings() -> Settings {
    fs::read_to_string(settings_path())
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn save_settings(settings: &Settings) -> anyhow::Result<()> {
    let p = settings_path();
    if let Some(parent) = p.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(p, serde_json::to_vec_pretty(settings)?)?;
    Ok(())
}

pub fn detect_obs_path() -> Option<PathBuf> {
    let base = app_dir();
    [
        base.join("engine/obs/bin/64bit/obs64.exe"),
        base.join("obs/bin/64bit/obs64.exe"),
        base.join("obs-studio/bin/64bit/obs64.exe"),
        base.join("OBS/bin/64bit/obs64.exe"),
    ]
    .into_iter()
    .find(|p| p.exists())
}

pub fn is_valid_obs(path: &str) -> bool {
    let p = Path::new(path);
    p.is_file() && p.file_name().and_then(|x| x.to_str()).map(|x| x.eq_ignore_ascii_case("obs64.exe")).unwrap_or(false)
}
