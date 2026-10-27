//! Portable names: keep legitimate `$`/apostrophes, avoid Win32 aliases and endings.
const FORBIDDEN: &str = "<>:\"/\\|?*";
pub const MAX_FILE_NAME_UTF16: usize = 120;
fn bad_character(c: char) -> bool {
    c.is_control() || FORBIDDEN.contains(c)
}
fn trim_end(name: &str) -> &str {
    name.trim_end_matches(|c: char| c == '.' || c.is_whitespace())
}
pub fn windows_device_name(name: &str) -> bool {
    let base = name
        .split('.')
        .next()
        .unwrap_or(name)
        .trim_end()
        .to_uppercase();
    matches!(
        base.as_str(),
        "CON" | "PRN" | "AUX" | "NUL" | "CONIN$" | "CONOUT$"
    ) || ["COM", "LPT"].iter().any(|prefix| {
        base.strip_prefix(prefix).is_some_and(|suffix| {
            matches!(
                suffix,
                "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" | "¹" | "²" | "³"
            )
        })
    })
}
fn prefix_utf16(text: &str, limit: usize, byte_limit: usize) -> String {
    let mut used = 0;
    let mut bytes = 0;
    text.chars()
        .take_while(|c| {
            used += c.len_utf16();
            bytes += c.len_utf8();
            used <= limit && bytes <= byte_limit
        })
        .collect()
}
pub fn portable_file_name(name: &str) -> Result<String, &'static str> {
    let name = trim_end(name.trim());
    if name.is_empty() || name.chars().any(bad_character) {
        return Err("invalid_file_name");
    }
    let (stem, extension) = match name.rsplit_once('.') {
        Some((stem, extension)) if !stem.is_empty() && extension.encode_utf16().count() < 32 => {
            (stem, Some(extension))
        }
        _ => (name, None),
    };
    let mut stem = trim_end(stem).to_owned();
    if stem.is_empty() {
        stem = "download".into();
    }
    if windows_device_name(&stem) {
        stem.insert(0, '_');
    }
    let extension_width = extension.map_or(0, |ext| ext.encode_utf16().count() + 1);
    stem = trim_end(&prefix_utf16(
        &stem,
        MAX_FILE_NAME_UTF16 - extension_width,
        200 - extension.map_or(0, |ext| ext.len() + 1),
    ))
    .to_owned();
    if stem.is_empty() {
        stem = "download".into();
    }
    // Truncation must not turn a formerly ordinary name into a device name.
    if windows_device_name(&stem) {
        stem.insert(0, '_');
    }
    Ok(match extension {
        Some(ext) => format!("{stem}.{ext}"),
        None => stem,
    })
}
pub fn portable_component(name: &str) -> bool {
    !name.is_empty()
        && name != "."
        && name != ".."
        && !name.chars().any(bad_character)
        && trim_end(name) == name
        && !windows_device_name(name)
        && name.encode_utf16().count() <= 255
}
pub fn portable_relative_path(name: &str) -> bool {
    let normalized = name.replace('\\', "/");
    !normalized.starts_with('/')
        && normalized
            .split('/')
            .filter(|part| !part.is_empty() && *part != ".")
            .all(portable_component)
        && normalized
            .split('/')
            .any(|part| !part.is_empty() && part != ".")
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn movie_stems_do_not_create_trailing_space_workspaces() {
        for name in [
            "Movie .mp4",
            "Movie. .mp4",
            "Movie\u{a0}.mp4",
            "Movie.mp4. ",
        ] {
            assert_eq!(portable_file_name(name).unwrap(), "Movie.mp4");
        }
        assert_eq!(
            portable_file_name("$ O'Brien's Movie .mp4").unwrap(),
            "$ O'Brien's Movie.mp4"
        );
        assert_eq!(portable_file_name("$'Movie' .mkv").unwrap(), "$'Movie'.mkv");
    }
    #[test]
    fn device_aliases_and_control_characters_are_not_created() {
        for name in [
            "CON.mp4",
            "NUL",
            "LPT¹.mp4",
            "COM².txt",
            "CON .mp4",
            "conout$.log",
        ] {
            assert!(portable_file_name(name).unwrap().starts_with('_'), "{name}");
        }
        for name in [
            "",
            " . ",
            "..",
            "foo\0.mp4",
            "bad\nname.mkv",
            "C:movie.mkv",
            "../film.mp4",
        ] {
            assert!(portable_file_name(name).is_err(), "{name:?}");
        }
    }
    #[test]
    fn long_extensions_and_unicode_stay_within_the_bound() {
        for name in [
            format!("{}.mkv", "🍿".repeat(100)),
            format!("film.{}", "x".repeat(500)),
            "x".repeat(500),
        ] {
            let safe = portable_file_name(&name).unwrap();
            assert!(safe.encode_utf16().count() <= MAX_FILE_NAME_UTF16);
            assert!(safe.len() <= 200);
            let root = tempfile::tempdir().unwrap();
            std::fs::write(root.path().join(&safe), b"payload").unwrap();
            assert!(!safe.ends_with([' ', '.']));
        }
        assert!(portable_file_name(&format!("{}.mp4", "a".repeat(300)))
            .unwrap()
            .ends_with(".mp4"));
    }
    #[test]
    fn archive_components_are_checked_without_shell_quoting() {
        for name in [
            "Film /scene.mkv",
            "Film./a",
            "C:/out",
            "../out",
            "safe/../../out",
            "AUX/movie.mkv",
            "bad\n/file",
            "...",
        ] {
            assert!(!portable_relative_path(name), "{name}");
        }
        for name in [
            "./Film/scene.mkv",
            "Film/scene .mkv",
            "$ O'Brien/scene's.mkv",
            "Film/",
            "Film/sub/scene.mkv",
        ] {
            assert!(portable_relative_path(name), "{name}");
        }
    }
}

/// Reject extractor-created symlinks and inaccessible names before publishing any file.
/// The caller owns the staging directory; this does not repair or delete user folders.
pub fn validate_extracted_tree(root: &std::path::Path) -> std::io::Result<()> {
    let mut pending = vec![root.to_path_buf()];
    while let Some(directory) = pending.pop() {
        for entry in std::fs::read_dir(directory)? {
            let entry = entry?;
            let metadata = std::fs::symlink_metadata(entry.path())?;
            if metadata.file_type().is_symlink()
                || !portable_component(&entry.file_name().to_string_lossy())
            {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "archive_contains_unsafe_path",
                ));
            }
            if metadata.is_dir() {
                pending.push(entry.path());
            } else if !metadata.is_file() {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "archive_contains_special_file",
                ));
            }
        }
    }
    Ok(())
}
#[cfg(all(test, unix))]
mod extraction_tests {
    use super::*;
    #[test]
    fn extracted_symlinks_and_trailing_spaces_are_not_published() {
        let root = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        std::fs::write(outside.path().join("private"), b"keep").unwrap();
        std::os::unix::fs::symlink(outside.path(), root.path().join("redirect")).unwrap();
        assert!(validate_extracted_tree(root.path()).is_err());
        std::fs::remove_file(root.path().join("redirect")).unwrap();
        std::fs::create_dir(root.path().join("Movie ")).unwrap();
        assert!(validate_extracted_tree(root.path()).is_err());
        assert_eq!(
            std::fs::read(outside.path().join("private")).unwrap(),
            b"keep"
        );
    }
}
