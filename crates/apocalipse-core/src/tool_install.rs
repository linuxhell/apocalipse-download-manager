//! Publish a validated tool set together, retaining originals on failure.
use std::{fs, io, path::Path, sync::Mutex};

// Shared by callers in this process; staging/validation happens before this lock.
static PUBLICATION: Mutex<()> = Mutex::new(());

fn locked(error: &io::Error) -> bool {
    cfg!(windows) && matches!(error.raw_os_error(), Some(5 | 32 | 33))
}

fn rename(from: &Path, to: &Path) -> io::Result<()> {
    // A just-exited version probe or a scanner can briefly retain a Windows handle.
    for attempt in 0..6 {
        match fs::rename(from, to) {
            Err(error) if locked(&error) && attempt < 5 => {
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
            result => return result,
        }
    }
    unreachable!()
}

/// Sources must already be validated and on the destination's filesystem.
/// `validate` checks the installed set. It must not recursively call `publish`.
/// A locked old backup is retained for later manual cleanup, never reported as
/// an installation failure after the new executable set has passed validation.
pub fn publish(files: &[(&Path, &Path)], validate: impl FnOnce() -> bool) -> Result<(), String> {
    let _publication = PUBLICATION.lock().map_err(|e| e.to_string())?;
    let parent = files
        .first()
        .and_then(|(_, target)| target.parent())
        .ok_or("tool_target_has_no_directory")?;
    let backups = tempfile::Builder::new()
        .prefix(".adm-tool-backup-")
        .tempdir_in(parent)
        .map_err(|e| e.to_string())?;
    let paths: Vec<_> = (0..files.len())
        .map(|index| backups.path().join(index.to_string()))
        .collect();
    let mut saved = vec![false; files.len()];
    let mut installed = vec![false; files.len()];
    let outcome: Result<(), String> = (|| {
        for (index, (_, target)) in files.iter().enumerate() {
            match fs::symlink_metadata(target) {
                Ok(metadata) => {
                    if !metadata.is_file() || metadata.file_type().is_symlink() {
                        return Err("tool_target_must_be_a_file".into());
                    }
                    rename(target, &paths[index]).map_err(operation_error)?;
                    saved[index] = true;
                }
                Err(error) if error.kind() == io::ErrorKind::NotFound => {}
                Err(error) => return Err(operation_error(error)),
            }
        }
        for (index, (source, target)) in files.iter().enumerate() {
            rename(source, target).map_err(operation_error)?;
            installed[index] = true;
        }
        if !validate() {
            return Err("updated_tool_validation_failed_original_restored".into());
        }
        Ok(())
    })();
    if let Err(error) = outcome {
        let mut failures = Vec::new();
        for (index, (_, target)) in files.iter().enumerate().rev() {
            if installed[index] {
                if let Err(failure) = fs::remove_file(target) {
                    failures.push(format!("{}: {failure}", target.display()));
                    continue;
                }
            }
            if saved[index] {
                if let Err(failure) = rename(&paths[index], target) {
                    failures.push(format!("{}: {failure}", target.display()));
                }
            }
        }
        if !failures.is_empty() {
            // Never let TempDir delete an original that could not be restored.
            let recovery = backups.keep();
            return Err(format!(
                "tool_update_restore_failed: {error}; backups={}; {}",
                recovery.display(),
                failures.join("; ")
            ));
        }
        return Err(error);
    }
    // Deleting a renamed, still-running Windows executable can fail. The new
    // binaries have already been checked; leave the unique backup directory.
    let _ = backups.close();
    Ok(())
}

fn operation_error(error: io::Error) -> String {
    if locked(&error) {
        "tool_update_in_use".into()
    } else {
        error.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn a_failed_second_publication_restores_the_entire_original_pair() {
        let dir = tempfile::tempdir().unwrap();
        let ffmpeg = dir.path().join("ffmpeg");
        let ffprobe = dir.path().join("ffprobe");
        let new = dir.path().join("new-ffmpeg");
        fs::write(&ffmpeg, b"old ffmpeg").unwrap();
        fs::write(&ffprobe, b"old ffprobe").unwrap();
        fs::write(&new, b"new ffmpeg").unwrap();
        let missing = dir.path().join("missing-ffprobe");
        assert!(publish(&[(&new, &ffmpeg), (&missing, &ffprobe)], || true).is_err());
        assert_eq!(fs::read(ffmpeg).unwrap(), b"old ffmpeg");
        assert_eq!(fs::read(ffprobe).unwrap(), b"old ffprobe");
    }
    #[test]
    fn validation_failure_restores_both_tools_and_success_replaces_both() {
        let dir = tempfile::tempdir().unwrap();
        let targets = [dir.path().join("ffmpeg"), dir.path().join("ffprobe")];
        let sources = [
            dir.path().join("new-ffmpeg"),
            dir.path().join("new-ffprobe"),
        ];
        for target in &targets {
            fs::write(target, b"old").unwrap();
        }
        let files = [(&*sources[0], &*targets[0]), (&*sources[1], &*targets[1])];
        for source in &sources {
            fs::write(source, b"new").unwrap();
        }
        assert!(publish(&files, || false).is_err());
        for target in &targets {
            assert_eq!(fs::read(target).unwrap(), b"old");
        }
        for source in &sources {
            fs::write(source, b"new").unwrap();
        }
        publish(&files, || {
            targets.iter().all(|p| fs::read(p).unwrap() == b"new")
        })
        .unwrap();
        for target in &targets {
            assert_eq!(fs::read(target).unwrap(), b"new");
        }
        assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 2);
    }
    #[test]
    fn a_blocked_second_target_restores_the_first_before_publication() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("ffmpeg");
        let invalid_target = dir.path().join("ffprobe");
        let source = dir.path().join("new");
        fs::write(&target, b"old").unwrap();
        fs::write(&source, b"new").unwrap();
        fs::create_dir(&invalid_target).unwrap();
        assert!(publish(&[(&source, &target), (&source, &invalid_target)], || true).is_err());
        assert_eq!(fs::read(target).unwrap(), b"old");
        assert!(invalid_target.is_dir());
    }

    #[test]
    fn failed_restore_keeps_the_original_in_an_identified_backup() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("ffmpeg");
        let source = dir.path().join("new");
        fs::write(&target, b"original").unwrap();
        fs::write(&source, b"replacement").unwrap();
        let error = publish(&[(&source, &target)], || {
            fs::remove_file(&target).unwrap();
            fs::create_dir(&target).unwrap();
            false
        })
        .unwrap_err();
        assert!(error.starts_with("tool_update_restore_failed:"));
        let recovery = fs::read_dir(dir.path())
            .unwrap()
            .filter_map(Result::ok)
            .find(|entry| {
                entry
                    .file_name()
                    .to_string_lossy()
                    .starts_with(".adm-tool-backup-")
            })
            .unwrap()
            .path();
        assert!(error.contains(&recovery.display().to_string()));
        assert_eq!(fs::read(recovery.join("0")).unwrap(), b"original");
    }

    #[cfg(windows)]
    #[test]
    fn a_locked_ffprobe_restores_ffmpeg_and_allows_retry_after_unlock() {
        use std::os::windows::fs::OpenOptionsExt;
        let dir = tempfile::tempdir().unwrap();
        let ffmpeg = dir.path().join("ffmpeg.exe");
        let ffprobe = dir.path().join("ffprobe.exe");
        let new_ffmpeg = dir.path().join("new-ffmpeg.exe");
        let new_ffprobe = dir.path().join("new-ffprobe.exe");
        for path in [&ffmpeg, &ffprobe] {
            fs::write(path, b"old").unwrap();
        }
        for path in [&new_ffmpeg, &new_ffprobe] {
            fs::write(path, b"new").unwrap();
        }
        let locked = fs::OpenOptions::new()
            .read(true)
            .share_mode(1)
            .open(&ffprobe)
            .unwrap();
        let files = [(&*new_ffmpeg, &*ffmpeg), (&*new_ffprobe, &*ffprobe)];
        assert_eq!(publish(&files, || true).unwrap_err(), "tool_update_in_use");
        assert_eq!(fs::read(&ffmpeg).unwrap(), b"old");
        assert_eq!(fs::read(&ffprobe).unwrap(), b"old");
        drop(locked);
        publish(&files, || true).unwrap();
        assert_eq!(fs::read(ffmpeg).unwrap(), b"new");
        assert_eq!(fs::read(ffprobe).unwrap(), b"new");
    }
}
