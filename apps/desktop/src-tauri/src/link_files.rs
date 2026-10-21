//! A Link transfer replaces its destination only after a complete, synced copy.
use std::{fs::File, path::Path};

pub(crate) fn staged_replace(
    destination: &Path,
    copy: impl FnOnce(&mut File) -> Result<(), String>,
) -> Result<(), String> {
    let parent = destination
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    let mut temporary = tempfile::NamedTempFile::new_in(parent).map_err(|e| e.to_string())?;
    copy(temporary.as_file_mut())?;
    temporary.as_file().sync_all().map_err(|e| e.to_string())?;
    // Link's save/upload flow permits replacement. Interrupted copies must not do it.
    temporary
        .persist(destination)
        .map_err(|e| e.error.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn cancelled_or_failed_copy_keeps_existing_file_and_cleans_its_temporary() {
        let dir = tempfile::tempdir().unwrap();
        let destination = dir.path().join("file");
        std::fs::write(&destination, b"previous file").unwrap();
        let result = staged_replace(&destination, |file| {
            file.write_all(b"partial").unwrap();
            Err("cancelled".into())
        });
        assert!(result.is_err());
        assert_eq!(std::fs::read(&destination).unwrap(), b"previous file");
        assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 1);
    }

    #[test]
    fn copying_from_the_destination_itself_does_not_truncate_the_source() {
        let dir = tempfile::tempdir().unwrap();
        let destination = dir.path().join("file");
        std::fs::write(&destination, b"complete source").unwrap();
        staged_replace(&destination, |output| {
            let mut input = File::open(&destination).unwrap();
            std::io::copy(&mut input, output).unwrap();
            Ok(())
        })
        .unwrap();
        assert_eq!(std::fs::read(&destination).unwrap(), b"complete source");
        assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 1);
    }
}
