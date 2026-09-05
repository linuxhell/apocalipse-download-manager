use serde::{Deserialize, Serialize};
use std::{
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};
use uuid::Uuid;

pub type DownloadId = Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DownloadState {
    Queued,
    Inspecting,
    Downloading,
    Paused,
    Verifying,
    Completed,
    Failed { message: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadTask {
    pub id: DownloadId,
    pub source: String,
    pub destination: PathBuf,
    pub state: DownloadState,
    pub received: u64,
    pub total: Option<u64>,
    #[serde(default)]
    pub progress_percent: Option<f64>,
    #[serde(default)]
    pub download_speed: Option<u64>,
    #[serde(default)]
    pub upload_speed: Option<u64>,
    #[serde(default)]
    pub torrent_selection: Vec<usize>,
    #[serde(default)]
    pub torrent_seeders: Option<u64>,
    #[serde(default)]
    pub torrent_leechers: Option<u64>,
    #[serde(default)]
    pub torrent_eta: Option<String>,
    #[serde(default)]
    pub format_selection: Option<String>,
    #[serde(default)]
    pub referer: Option<String>,
    #[serde(default)]
    pub known_duration: Option<f64>,
    #[serde(default)]
    pub mirrors: Vec<String>,
    #[serde(default)]
    pub priority: i8,
    #[serde(default)]
    pub sha256: Option<String>,
    #[serde(default)]
    pub integrity_verified: bool,
    #[serde(default)]
    pub created_at: u64,
    #[serde(default)]
    pub completed_at: Option<u64>,
}

impl DownloadTask {
    pub fn new(source: impl Into<String>, destination: impl Into<PathBuf>) -> Self {
        Self {
            id: Uuid::new_v4(),
            source: source.into(),
            destination: destination.into(),
            state: DownloadState::Queued,
            received: 0,
            total: None,
            progress_percent: None,
            download_speed: None,
            upload_speed: None,
            torrent_selection: Vec::new(),
            torrent_seeders: None,
            torrent_leechers: None,
            torrent_eta: None,
            format_selection: None,
            referer: None,
            known_duration: None,
            mirrors: Vec::new(),
            priority: 0,
            sha256: None,
            integrity_verified: false,
            created_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_or(0, |value| value.as_secs()),
            completed_at: None,
        }
    }
}
