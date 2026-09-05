use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tokio::process::{Child, Command};

const MINIMUM_PREFIX: u64 = 8 * 1024 * 1024;
const MAXIMUM_PREFIX: u64 = 64 * 1024 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerConfig {
    pub executable: PathBuf,
    /// Optional player arguments. `{file}` is replaced by the media path.
    pub arguments: Vec<String>,
}

impl PlayerConfig {
    pub fn validate(&self) -> Result<()> {
        if !self.executable.is_file() {
            bail!(
                "media player executable does not exist: {}",
                self.executable.display()
            );
        }
        if self
            .arguments
            .iter()
            .filter(|arg| arg.contains("{file}"))
            .count()
            > 1
        {
            bail!("the player argument template may contain only one {{file}} placeholder");
        }
        Ok(())
    }

    fn resolved_arguments(&self, media: &Path) -> Vec<String> {
        let file = media.to_string_lossy();
        if self.arguments.iter().any(|arg| arg.contains("{file}")) {
            self.arguments
                .iter()
                .map(|arg| arg.replace("{file}", &file))
                .collect()
        } else {
            self.arguments
                .iter()
                .cloned()
                .chain([file.into_owned()])
                .collect()
        }
    }
}

pub async fn launch_player(config: &PlayerConfig, media: &Path) -> Result<Child> {
    config.validate()?;
    if !media.is_file() {
        bail!("preview media file does not exist: {}", media.display());
    }
    Command::new(&config.executable)
        .args(config.resolved_arguments(media))
        .kill_on_drop(false)
        .spawn()
        .with_context(|| {
            format!(
                "could not start media player at {}",
                config.executable.display()
            )
        })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TorrentPreviewPolicy {
    pub sequential_download: bool,
    pub prioritize_first_pieces: bool,
    pub prioritize_last_piece: bool,
}

impl TorrentPreviewPolicy {
    pub const fn for_video(container_index_may_be_at_end: bool) -> Self {
        Self {
            sequential_download: true,
            prioritize_first_pieces: true,
            prioritize_last_piece: container_index_may_be_at_end,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PreviewReadiness {
    pub ready: bool,
    pub required_contiguous_bytes: u64,
    pub available_contiguous_bytes: u64,
}

impl PreviewReadiness {
    pub fn calculate(
        file_size: u64,
        available_contiguous_bytes: u64,
        metadata_ready: bool,
    ) -> Self {
        let required =
            (file_size / 100).clamp(MINIMUM_PREFIX.min(file_size), MAXIMUM_PREFIX.min(file_size));
        Self {
            ready: metadata_ready && available_contiguous_bytes >= required,
            required_contiguous_bytes: required,
            available_contiguous_bytes,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preview_requires_metadata_and_a_contiguous_prefix() {
        let size = 1_000_000_000;
        assert!(!PreviewReadiness::calculate(size, 20_000_000, false).ready);
        assert!(PreviewReadiness::calculate(size, 20_000_000, true).ready);
    }

    #[test]
    fn video_policy_prioritizes_tail_only_when_needed() {
        assert!(!TorrentPreviewPolicy::for_video(false).prioritize_last_piece);
        assert!(TorrentPreviewPolicy::for_video(true).prioritize_last_piece);
    }
}
