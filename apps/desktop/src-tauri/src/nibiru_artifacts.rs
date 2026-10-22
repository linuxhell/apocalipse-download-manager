//! Exact auxiliary paths for a single Xtreme destination; no directory sweeps.
use std::{
    fs, io,
    path::{Path, PathBuf},
    time::Duration,
};

pub fn paths(destination: &Path) -> Vec<PathBuf> {
    [
        ".nibiru-http.part.json.tmp",
        ".nibiru-http.part.json",
        ".nibiru-http.part",
        ".nibiru-http.lock",
    ]
    .into_iter()
    .map(|suffix| {
        let mut name = destination.as_os_str().to_owned();
        name.push(suffix);
        PathBuf::from(name)
    })
    .collect()
}

fn remove_file(path: &Path) -> io::Result<()> {
    match fs::symlink_metadata(path) {
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error),
        Ok(metadata) if metadata.is_file() => fs::remove_file(path),
        Ok(_) => Err(io::Error::other(
            "refusing to remove a directory or symlink as an Xtreme sidecar",
        )),
    }
}

/// Wait for the engine to close its destination lock before removing resumable data.
/// Called on a blocking worker only after the task has been cancelled/removed.
pub fn cleanup(destination: &Path) -> io::Result<()> {
    let paths = paths(destination);
    let lock = paths.last().unwrap();
    let mut options = fs::OpenOptions::new();
    options.read(true).write(true);
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        // Exclusive open prevents deleting data while an older engine handle exists.
        // Delete-on-close removes the lock atomically when this exclusive handle closes.
        options
            .share_mode(0)
            .access_mode(0xC0010000)
            .custom_flags(0x04000000);
    }
    let mut lease = None;
    for attempt in 0..40 {
        match fs::symlink_metadata(lock) {
            Ok(metadata) if !metadata.is_file() || metadata.file_type().is_symlink() => {
                return Err(io::Error::other("invalid Xtreme lock path"));
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => break,
            Err(error) => return Err(error),
            _ => {}
        }
        if let Ok(file) = options.open(lock) {
            if file.try_lock().is_ok() {
                lease = Some(file);
                break;
            }
        }
        if attempt == 39 {
            return Err(io::Error::other(
                "Xtreme destination is still in use; partial files preserved",
            ));
        }
        std::thread::sleep(Duration::from_millis(250));
    }
    for path in paths.iter().take(paths.len() - 1) {
        remove_file(path)?;
    }
    #[cfg(not(windows))]
    remove_file(lock)?;
    drop(lease);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn root() -> PathBuf {
        let path = std::env::temp_dir().join(format!(
            "adm-sidecars-{}-{:?}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&path).unwrap();
        path
    }

    #[test]
    fn cleanup_removes_only_exact_sidecars_and_preserves_iso_and_neighbor() {
        let root = root();
        let dest = root.join("Windows11.iso");
        fs::write(&dest, b"completed iso").unwrap();
        let neighbor = root.join("Windows11.iso.other.nibiru-http.part");
        fs::write(&neighbor, b"unrelated").unwrap();
        for path in paths(&dest) {
            fs::write(path, b"").unwrap();
        }
        cleanup(&dest).unwrap();
        assert!(paths(&dest).iter().all(|path| !path.exists()));
        assert_eq!(fs::read(&dest).unwrap(), b"completed iso");
        assert!(neighbor.exists());
        cleanup(&dest).unwrap();
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn cleanup_refuses_a_directory_at_a_sidecar_name() {
        let root = root();
        let dest = root.join("file.iso");
        let part = paths(&dest)[2].clone();
        fs::create_dir(&part).unwrap();
        fs::write(part.join("keep"), b"user file").unwrap();
        assert!(cleanup(&dest).is_err());
        assert!(part.join("keep").exists());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn cleanup_waits_for_the_engine_to_finish_writing_resume_state() {
        let root = root();
        let dest = root.join("active.iso");
        let files = paths(&dest);
        let lock = fs::OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(files.last().unwrap())
            .unwrap();
        lock.try_lock().unwrap();
        fs::write(&files[2], b"partial data").unwrap();
        let target = dest.clone();
        let (started, ready) = std::sync::mpsc::channel();
        let worker = std::thread::spawn(move || {
            started.send(()).unwrap();
            cleanup(&target)
        });
        ready.recv().unwrap();
        std::thread::sleep(Duration::from_millis(100));
        assert!(
            files[2].exists(),
            "partial removed while the engine still owns the lock"
        );
        fs::write(&files[1], b"last resume state from cancelled worker").unwrap();
        drop(lock);
        worker.join().unwrap().unwrap();
        assert!(paths(&dest).iter().all(|path| !path.exists()));
        fs::remove_dir_all(root).unwrap();
    }
}
