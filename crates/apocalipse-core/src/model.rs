use serde::{Deserialize, Serialize};
use std::path::PathBuf;
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
        }
    }
}

