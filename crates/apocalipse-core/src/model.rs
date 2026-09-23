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
    pub resume_supported: Option<bool>,
    #[serde(default)]
    pub download_speed: Option<u64>,
    #[serde(default)]
    pub upload_speed: Option<u64>,
    #[serde(default)]
    pub torrent_selection: Vec<usize>,
    /// aria2-next GID persisted across ADM restarts. The engine's session
    /// file restores unfinished transfers with the same GID, allowing the UI
    /// task to reconnect to the already-restored transfer instead of creating
    /// a duplicate.
    #[serde(default)]
    pub aria2_gid: Option<String>,
    #[serde(default)]
    pub torrent_seeders: Option<u64>,
    #[serde(default)]
    pub torrent_leechers: Option<u64>,
    #[serde(default)]
    pub torrent_eta: Option<String>,
    /// Number of HTTP/HTTPS webseed sources (BEP 19) currently in use
    /// alongside the BT swarm for this task, when the torrent declares any.
    #[serde(default)]
    pub torrent_web_seeds: Option<u64>,
    #[serde(default)]
    pub format_selection: Option<String>,
    #[serde(default)]
    pub referer: Option<String>,
    #[serde(default)]
    pub known_duration: Option<f64>,
    #[serde(default)]
    pub is_live: bool,
    /// Human-readable media title supplied by inspection or the browser bridge.
    #[serde(default)]
    pub display_title: Option<String>,
    /// Preview image for this task. Older queue files simply deserialize it as absent.
    #[serde(default)]
    pub thumbnail: Option<String>,
    /// Optional separate audio stream for browser-captured adaptive media.
    #[serde(default)]
    pub companion_audio_url: Option<String>,
    #[serde(default)]
    pub mirrors: Vec<String>,
    #[serde(default)]
    pub priority: i8,
    #[serde(default)]
    pub bandwidth_limit: Option<u64>,
    /// User-selected connection count for this task only. `None` keeps the
    /// global/site behavior and preserves compatibility with older queues.
    #[serde(default)]
    pub connections_override: Option<usize>,
    #[serde(default)]
    pub expected_size: Option<u64>,
    #[serde(default)]
    pub sha256: Option<String>,
    #[serde(default)]
    pub integrity_verified: bool,
    /// Extract a completed archive with the user-configured external extractor.
    #[serde(default)]
    pub auto_extract: bool,
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
            resume_supported: None,
            download_speed: None,
            upload_speed: None,
            torrent_selection: Vec::new(),
            aria2_gid: None,
            torrent_seeders: None,
            torrent_leechers: None,
            torrent_eta: None,
            torrent_web_seeds: None,
            format_selection: None,
            referer: None,
            known_duration: None,
            is_live: false,
            display_title: None,
            thumbnail: None,
            companion_audio_url: None,
            mirrors: Vec::new(),
            priority: 0,
            bandwidth_limit: None,
            connections_override: None,
            expected_size: None,
            sha256: None,
            integrity_verified: false,
            auto_extract: false,
            created_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_or(0, |value| value.as_secs()),
            completed_at: None,
        }
    }
}
