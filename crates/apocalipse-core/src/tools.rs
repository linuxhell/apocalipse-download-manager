use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ExternalTool {
    YtDlp,
    Ffmpeg,
    NM3u8dlRe,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInstallation {
    pub tool: ExternalTool,
    pub executable: PathBuf,
    pub detected_version: Option<String>,
    pub latest_version: Option<String>,
}

impl ToolInstallation {
    pub fn update_available(&self) -> bool {
        matches!((&self.detected_version, &self.latest_version), (Some(a), Some(b)) if a != b)
    }
}
