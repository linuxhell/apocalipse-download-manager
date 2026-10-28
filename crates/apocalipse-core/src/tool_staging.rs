//! Owned updater staging and conservative cleanup of abandoned candidates.
use crate::tool_install::retry_io;
use std::{
    fs, io,
    path::Path,
    time::{Duration, SystemTime},
};

const OWNER: &str = ".adm-owner.lock";
const PREFIX: &str = ".adm-tool-update-";
const STALE_AFTER: Duration = Duration::from_secs(3600);

pub struct Staging {
    directory: Option<tempfile::TempDir>,
    owner: Option<fs::File>,
}
impl Staging {
    pub fn new(parent: &Path) -> io::Result<Self> {
        let directory = tempfile::Builder::new().prefix(PREFIX).tempdir_in(parent)?;
        let owner = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .open(directory.path().join(OWNER))?;
        owner.lock()?;
        Ok(Self {
            directory: Some(directory),
            owner: Some(owner),
        })
    }
    pub fn path(&self) -> &Path {
        self.directory.as_ref().unwrap().path()
    }
}
impl Drop for Staging {
    fn drop(&mut self) {
        // Close the Windows handle before removing the directory. If a scanner
        // keeps a staged binary mapped, later status checks can retry safely.
        drop(self.owner.take());
        if let Some(directory) = self.directory.take() {
            let path = directory.keep();
            let _ = retry_io(|| fs::remove_dir_all(&path));
        }
    }
}

fn managed(name: &str, validated_names: &[String]) -> bool {
    let stem = name.strip_suffix(".exe").unwrap_or(name);
    [
        "ffmpeg",
        "ffprobe",
        "yt-dlp",
        "qjs",
        "N_m3u8DL-RE",
        "nibiru-xtreme",
    ]
    .iter()
    .any(|known| stem.eq_ignore_ascii_case(known))
        && validated_names
            .iter()
            .any(|valid| valid.eq_ignore_ascii_case(name))
}
fn stale(metadata: &fs::Metadata, now: SystemTime) -> bool {
    metadata
        .modified()
        .ok()
        .and_then(|at| now.duration_since(at).ok())
        .is_some_and(|age| age >= STALE_AFTER)
}

/// Call only with names whose installed executables have passed a version probe.
/// Never scans recursively, follows symlinks, removes recovery backups, or sweeps
/// unknown files. The age gate also protects pre-lock updater versions in use.
pub fn cleanup(parent: &Path, validated_names: &[String], now: SystemTime) -> usize {
    let Ok(entries) = fs::read_dir(parent) else {
        return 0;
    };
    let mut removed = 0;
    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();
        let Ok(metadata) = fs::symlink_metadata(&path) else {
            continue;
        };
        if metadata.file_type().is_symlink() || !stale(&metadata, now) {
            continue;
        }
        let Some(name) = entry.file_name().to_str().map(str::to_owned) else {
            continue;
        };
        if metadata.is_file() {
            let candidate = name
                .strip_prefix(".apocalipse-new-")
                .or_else(|| name.strip_prefix(".apocalipse-download-"));
            if candidate.is_some_and(|name| managed(name, validated_names))
                && retry_io(|| fs::remove_file(&path)).is_ok()
            {
                removed += 1;
            }
            continue;
        }
        let Some(suffix) = name.strip_prefix(PREFIX) else {
            continue;
        };
        if !metadata.is_dir()
            || suffix.len() != 6
            || !suffix.bytes().all(|c| c.is_ascii_alphanumeric())
        {
            continue;
        }
        let Ok(children) = fs::read_dir(&path) else {
            continue;
        };
        let children: Result<Vec<_>, _> = children
            .map(|entry| {
                let entry = entry?;
                let kind = entry.file_type()?;
                let name = entry.file_name();
                if !kind.is_file()
                    || kind.is_symlink()
                    || !(name == OWNER
                        || name.to_str().is_some_and(|s| managed(s, validated_names)))
                {
                    return Err(io::Error::other("unknown staging content"));
                }
                Ok(entry.path())
            })
            .collect();
        let Ok(children) = children else {
            continue;
        };
        let marker = path.join(OWNER);
        let owner = if children.iter().any(|p| p == &marker) {
            let Ok(file) = fs::OpenOptions::new().read(true).write(true).open(&marker) else {
                continue;
            };
            if file.try_lock().is_err() {
                continue;
            }
            Some(file)
        } else {
            None
        };
        // Remove only the inspected files. A newly added/unknown child prevents
        // remove_dir from succeeding instead of being recursively deleted.
        for child in children.iter().filter(|p| *p != &marker) {
            let _ = retry_io(|| fs::remove_file(child));
        }
        drop(owner);
        if marker.is_file() {
            let _ = retry_io(|| fs::remove_file(&marker));
        }
        if fs::remove_dir(&path).is_ok() {
            removed += 1;
        }
    }
    removed
}

#[cfg(test)]
mod tests {
    use super::*;
    fn names() -> Vec<String> {
        vec!["ffmpeg.exe".into(), "ffprobe.exe".into()]
    }
    fn later() -> SystemTime {
        SystemTime::now() + STALE_AFTER * 2
    }
    #[test]
    fn legacy_candidates_are_removed_only_for_a_valid_installed_tool() {
        let root = tempfile::tempdir().unwrap();
        let old = root.path().join(".apocalipse-new-ffmpeg.exe");
        let missing = root.path().join(".apocalipse-new-qjs.exe");
        let backup = root.path().join(".ffmpeg.exe.apocalipse-backup");
        let real = root.path().join("ffmpeg.exe");
        for path in [&old, &missing, &backup, &real] {
            fs::write(path, b"keep or candidate").unwrap();
        }
        assert_eq!(cleanup(root.path(), &names(), SystemTime::now()), 0);
        assert_eq!(cleanup(root.path(), &names(), later()), 1);
        assert!(!old.exists());
        for path in [&missing, &backup, &real] {
            assert!(path.is_file());
        }
    }
    #[test]
    fn legacy_workspaces_require_all_contents_to_be_known_and_validated() {
        let root = tempfile::tempdir().unwrap();
        for (name, child) in [
            (".adm-tool-update-Ab1234", "ffmpeg.exe"),
            (".adm-tool-update-Ab1235", "ffprobe.exe"),
            (".adm-tool-update-Ab1236", "user-document.txt"),
            (".adm-tool-backup-Ab1234", "ffmpeg.exe"),
        ] {
            let dir = root.path().join(name);
            fs::create_dir(&dir).unwrap();
            fs::write(dir.join(child), b"x").unwrap();
        }
        assert_eq!(cleanup(root.path(), &["ffmpeg.exe".into()], later()), 1);
        assert!(root
            .path()
            .join(".adm-tool-update-Ab1235/ffprobe.exe")
            .exists());
        assert!(root
            .path()
            .join(".adm-tool-update-Ab1236/user-document.txt")
            .exists());
        assert!(root
            .path()
            .join(".adm-tool-backup-Ab1234/ffmpeg.exe")
            .exists());
    }
    #[test]
    fn active_owned_workspace_survives_cleanup_and_is_removed_on_drop() {
        let root = tempfile::tempdir().unwrap();
        let active = Staging::new(root.path()).unwrap();
        let path = active.path().to_path_buf();
        fs::write(path.join("ffmpeg.exe"), b"candidate").unwrap();
        assert_eq!(cleanup(root.path(), &names(), later()), 0);
        assert!(path.join("ffmpeg.exe").exists());
        drop(active);
        assert!(!path.exists());
    }
    #[test]
    fn abandoned_owned_workspace_is_cleaned_after_its_lock_is_released() {
        let root = tempfile::tempdir().unwrap();
        let mut abandoned = Staging::new(root.path()).unwrap();
        let path = abandoned.path().to_path_buf();
        fs::write(path.join("ffmpeg.exe"), b"candidate").unwrap();
        let directory = abandoned.directory.take().unwrap().keep();
        drop(abandoned);
        assert_eq!(cleanup(root.path(), &names(), later()), 1);
        assert!(!directory.exists());
    }
    #[cfg(unix)]
    #[test]
    fn symlinks_and_directories_with_unknown_children_are_preserved() {
        let root = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        fs::write(outside.path().join("ffmpeg.exe"), b"user").unwrap();
        std::os::unix::fs::symlink(outside.path(), root.path().join(".adm-tool-update-Ab1234"))
            .unwrap();
        std::os::unix::fs::symlink(
            outside.path().join("ffmpeg.exe"),
            root.path().join(".apocalipse-new-ffmpeg.exe"),
        )
        .unwrap();
        assert_eq!(cleanup(root.path(), &names(), later()), 0);
        assert_eq!(
            fs::read(outside.path().join("ffmpeg.exe")).unwrap(),
            b"user"
        );
    }
}
