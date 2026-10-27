//! Commit only the engine's fixed output name, without replacing user files.
use anyhow::{bail, Context, Result};
use std::{
    fs, io,
    path::{Path, PathBuf},
};
pub const OUTPUT_STEM: &str = "apocalipse-media";
const MEDIA_EXTENSIONS: &[&str] = &[
    "mp4", "mkv", "ts", "webm", "mov", "m4a", "mp3", "wav", "flac", "opus", "aac", "ogg",
];
/// Synchronous disk work; desktop callers run this on a blocking worker.
pub fn commit(workspace: &Path, requested: &Path) -> Result<(PathBuf, u64)> {
    let mut candidates = Vec::new();
    for entry in fs::read_dir(workspace)? {
        let entry = entry?;
        let path = entry.path();
        if entry.file_type()?.is_symlink() {
            continue;
        }
        let Some(extension) = path.extension().and_then(|e| e.to_str()) else {
            continue;
        };
        if path.file_stem().and_then(|s| s.to_str()) != Some(OUTPUT_STEM)
            || !MEDIA_EXTENSIONS.contains(&extension.to_ascii_lowercase().as_str())
        {
            continue;
        }
        let metadata = entry.metadata()?;
        if metadata.is_file() && metadata.len() > 0 {
            candidates.push((path, metadata.len()));
        }
    }
    candidates.sort_by_key(|(_, size)| std::cmp::Reverse(*size));
    let (source, size) = candidates
        .into_iter()
        .next()
        .context("external_final_file_missing")?;
    let destination = if requested.extension().is_none() {
        requested.with_extension(source.extension().unwrap())
    } else {
        requested.to_path_buf()
    };
    if destination.exists() {
        bail!("external_destination_already_exists");
    }
    let parent = destination.parent().context("invalid_destination")?;
    fs::create_dir_all(parent)?;
    // Hard-link publication is atomic, does not overwrite, and avoids copying a
    // whole film on the same filesystem. Cross-volume/FAT destinations use a
    // staged copy; never expose or truncate the user's existing file.
    match fs::hard_link(&source, &destination) {
        Ok(()) => {}
        Err(e) if e.kind() == io::ErrorKind::AlreadyExists => return Err(e.into()),
        Err(_) => {
            let mut temporary = tempfile::Builder::new()
                .prefix(".adm-output-")
                .tempfile_in(parent)?;
            let copied = io::copy(&mut fs::File::open(&source)?, temporary.as_file_mut())?;
            if copied != size {
                bail!("external_output_changed");
            }
            temporary.as_file().sync_all()?;
            temporary
                .persist_noclobber(&destination)
                .map_err(|e| e.error)?;
        }
    }
    // The published output is independent of workspace cleanup. A locked log
    // must not turn a successful download into a failure.
    let _ = fs::remove_dir_all(workspace);
    Ok((destination, size))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn only_a_completed_fixed_media_file_can_be_published() {
        let root = tempfile::tempdir().unwrap();
        let work = root.path().join("work");
        fs::create_dir(&work).unwrap();
        for name in [
            "browser-cookies.txt",
            "yt-dlp-credentials.txt",
            "apocalipse-media.mp4.part",
            "apocalipse-media.f137.mp4",
            "other.mp4",
        ] {
            fs::write(work.join(name), b"private or partial content").unwrap();
        }
        let target = root.path().join("$ O'Brien's film.mp4");
        assert!(commit(&work, &target).is_err());
        assert!(!target.exists());
        fs::write(work.join("apocalipse-media.mp4"), b"media").unwrap();
        assert_eq!(commit(&work, &target).unwrap(), (target.clone(), 5));
        assert_eq!(fs::read(target).unwrap(), b"media");
        assert!(!work.exists());
    }
    #[test]
    fn existing_destinations_and_unrelated_folders_survive() {
        let root = tempfile::tempdir().unwrap();
        let work = root.path().join("work");
        fs::create_dir(&work).unwrap();
        fs::write(work.join("apocalipse-media.mp4"), b"new").unwrap();
        let target = root.path().join("Film.mp4");
        fs::write(&target, b"original").unwrap();
        let unrelated = root.path().join("Film");
        fs::create_dir(&unrelated).unwrap();
        fs::write(unrelated.join("mine"), b"keep").unwrap();
        assert!(commit(&work, &target).is_err());
        assert_eq!(fs::read(target).unwrap(), b"original");
        assert_eq!(fs::read(unrelated.join("mine")).unwrap(), b"keep");
        assert!(work.exists());
    }
    #[cfg(unix)]
    #[test]
    fn a_media_named_symlink_cannot_publish_a_private_file() {
        let root = tempfile::tempdir().unwrap();
        let work = root.path().join("work");
        fs::create_dir(&work).unwrap();
        let secret = root.path().join("secret");
        fs::write(&secret, b"secret").unwrap();
        std::os::unix::fs::symlink(secret, work.join("apocalipse-media.mp4")).unwrap();
        assert!(commit(&work, &root.path().join("out.mp4")).is_err());
    }
}
