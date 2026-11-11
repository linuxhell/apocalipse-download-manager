//! AppImage mounts are read-only; mutable data belongs beside the image or in app data.
#[cfg(any(target_os = "linux", test))]
use std::ffi::OsStr;
use std::{
    fs, io,
    path::{Path, PathBuf},
};

pub fn running_image_data(executable: &Path) -> Option<PathBuf> {
    #[cfg(target_os = "linux")]
    {
        image_data(
            executable,
            std::env::var_os("APPIMAGE").as_deref(),
            std::env::var_os("APPDIR").as_deref(),
        )
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = executable;
        None
    }
}
#[cfg(any(target_os = "linux", test))]
fn image_data(executable: &Path, image: Option<&OsStr>, mount: Option<&OsStr>) -> Option<PathBuf> {
    let (image, mount) = (Path::new(image?), Path::new(mount?));
    if !image.is_absolute() || !mount.is_absolute() || !executable.starts_with(mount) {
        return None;
    }
    Some(image.parent()?.join("data"))
}

pub fn install_backend(executable: &Path, directory: &Path) -> io::Result<()> {
    let destination = directory.join("nibiru-xtreme");
    if destination.try_exists()? {
        return Ok(());
    }
    let source = executable
        .parent()
        .ok_or_else(|| io::Error::other("executable has no parent"))?
        .join("nibiru-xtreme");
    let temporary = tempfile::NamedTempFile::new_in(directory)?;
    fs::copy(source, temporary.path())?;
    temporary.as_file().sync_all()?;
    match temporary.persist_noclobber(destination) {
        Ok(_) => Ok(()),
        Err(e) if e.error.kind() == io::ErrorKind::AlreadyExists => Ok(()),
        Err(e) => Err(e.error),
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn mounted_image_uses_external_data_only_for_its_own_executable() {
        let root = tempfile::tempdir().unwrap();
        let mount = root.path().join("mount");
        let image = root.path().join("apps/ADM.AppImage");
        let exe = mount.join("usr/bin/apocalipse-desktop");
        assert_eq!(
            image_data(&exe, Some(image.as_os_str()), Some(mount.as_os_str())),
            Some(root.path().join("apps/data"))
        );
        assert!(image_data(
            &root.path().join("another/app"),
            Some(image.as_os_str()),
            Some(mount.as_os_str())
        )
        .is_none());
        assert!(image_data(
            &exe,
            Some(OsStr::new("relative.AppImage")),
            Some(mount.as_os_str())
        )
        .is_none());
        assert!(image_data(&exe, None, Some(mount.as_os_str())).is_none());
    }
    #[test]
    fn backend_install_is_complete_and_preserves_existing_engine() {
        let root = tempfile::tempdir().unwrap();
        let binaries = root.path().join("bin");
        let target = root.path().join("tools");
        fs::create_dir_all(&binaries).unwrap();
        fs::create_dir_all(&target).unwrap();
        let source = binaries.join("nibiru-xtreme");
        fs::write(&source, b"pinned engine payload").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&source, fs::Permissions::from_mode(0o755)).unwrap();
        }
        install_backend(&binaries.join("apocalipse-desktop"), &target).unwrap();
        assert_eq!(
            fs::read(target.join("nibiru-xtreme")).unwrap(),
            b"pinned engine payload"
        );
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_ne!(
                fs::metadata(target.join("nibiru-xtreme"))
                    .unwrap()
                    .permissions()
                    .mode()
                    & 0o111,
                0
            );
        }
        fs::write(&source, b"different payload").unwrap();
        install_backend(&binaries.join("apocalipse-desktop"), &target).unwrap();
        assert_eq!(
            fs::read(target.join("nibiru-xtreme")).unwrap(),
            b"pinned engine payload"
        );
    }
    #[test]
    fn missing_bundled_engine_leaves_no_partial_destination() {
        let root = tempfile::tempdir().unwrap();
        assert!(install_backend(&root.path().join("app"), root.path()).is_err());
        assert!(!root.path().join("nibiru-xtreme").exists());
    }
}
