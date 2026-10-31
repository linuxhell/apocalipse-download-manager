# Apocalipse Download Manager

[Português](README.md) · [English](README.en.md) · [简体中文](README.zh-CN.md)

Free, portable download manager for Windows, Linux and macOS x64.

[**Download**](https://github.com/linuxhell/apocalipse-download-manager/releases/latest)

| Platform | Package |
| --- | --- |
| Windows x64 | [Portable ZIP](https://github.com/linuxhell/apocalipse-download-manager/releases/latest/download/apocalipse-download-manager-windows-x64-portable.zip) |
| Linux x64 | [Portable tar.gz](https://github.com/linuxhell/apocalipse-download-manager/releases/latest/download/apocalipse-download-manager-linux-x64-portable.tar.gz) · [AppImage](https://github.com/linuxhell/apocalipse-download-manager/releases/latest/download/apocalipse-download-manager-linux-x64.AppImage) |
| macOS x64 | [Portable ZIP](https://github.com/linuxhell/apocalipse-download-manager/releases/latest/download/apocalipse-download-manager-macos-x64-portable.zip) |
| Chrome | [0.3.207 ZIP](https://github.com/linuxhell/apocalipse-download-manager/releases/latest/download/apocalipse-chrome-0.3.207.zip) |
| Edge | [0.3.207 ZIP](https://github.com/linuxhell/apocalipse-download-manager/releases/latest/download/apocalipse-edge-0.3.207.zip) |
| Firefox | [0.3.207 XPI](https://github.com/linuxhell/apocalipse-download-manager/releases/latest/download/apocalipse-firefox-0.3.207.xpi) |

![Downloads](assets/screenshots/downloads.png)

The thread slider is visible in the new-download dialog and takes priority over engine defaults. The checkbox below saves the site preference immediately; changing the slider updates it and unchecking removes that preference while preserving credentials and other site settings.

## Get started

1. Download and extract the portable package, then launch the application. It includes Nibiru Xtreme built from the same pinned revision used by ADM.
2. Install the matching browser extension. For Chrome/Edge, extract the ZIP and use **Load unpacked** in developer mode. For Firefox, open the Mozilla-signed XPI.
3. Open **Tools** to install or update engines such as yt-dlp and FFmpeg. The AppImage does not bundle the Xtreme executable; install it there. On Linux, mark the AppImage executable.

## Features

- Resumable HTTP/HTTPS downloads, multiple connections, mirrors and SHA-256 verification.
- Native torrents, magnets and ED2K/Kad, file selection and video preview.
- Browser media capture, yt-dlp, HLS, FFmpeg and progressive recordings.
- Apocalipse Link for authenticated TLS transfers between computers.
- Per-site credentials in the operating system vault; Portuguese, English and Simplified Chinese interfaces.

## Version 0.8.6

Fixes duplicated torrent folders: ADM sends the reserved destination as a separate parent directory and final basename. Single-file torrents also use the correct final path. Requires Xtreme 0.2.3 or newer; update it under Settings → Tools if using a separately configured executable. Existing downloaded folders are not moved automatically.

Tool updates retain the executable names and paths. Completion requires validating the new set and removing the old files; failure during old-file cleanup also restores the previous version. Abandoned tool staging files are cleaned after validating the installed executables, with protection for active updates and recovery backups.

Fixes FFmpeg/FFprobe updates on Windows: the pair is restored on failure, staging paths are unique, and backup cleanup cannot turn a successful update into an error. If another process blocks replacement, ADM preserves the previous files and asks you to stop the operation using the tool.

Fixes movie names with trailing spaces or dots in their base name and keeps media temporary files in a dedicated per-task directory. Finalization selects only generated media and preserves existing files. The selected connection count also applies to YouTube fragments. Includes thumbnail cache concurrency fixes and torrent validation through Xtreme 0.2.2. See the [audit and its limits](docs/AUDIT-2026-10-27.md).

Fixes the title overlapping torrent controls in the compact window. Header actions wrap onto another line when needed in all three languages.

The download dialog no longer numbers its sections and shows up to **12 recent folders**, with the default location first. The release includes the **Mozilla-signed Firefox 0.3.207 extension**, alongside Chrome/Edge 0.3.207.

**Convert file** is available from the download toolbar, the Add download window and completed download actions. Choose a local file or use a completed download. FFprobe detects the actual content and streams, even with a misleading extension. Embedded covers do not count as video. Audio-only MP4 offers audio formats; silent video offers video formats only. When audio and video are present, every frame is checked before classifying all-black video as audio; analysis can take time for long files.

Outputs: PNG, lossless WebP, JPEG, TIFF and BMP; FLAC, WAV, MP3, M4A, Opus and Ogg; MP4, MKV, lossless MKV and WebM. Available formats depend on the installed FFmpeg encoders. Audio-track selection, progress and cancellation are supported. Conversion preserves the source and writes a separate file without overwriting existing files. Lossless encoding cannot recover detail already lost; JPEG and lossy formats use high quality. Recognized animated images are handled as video to preserve their frames.

Install **FFmpeg with FFprobe** under **Settings → Tools**. The download dialog fills the available main-window area, with advanced options always visible and two columns where space allows. Extension thumbnails are retained, and invalid images try the YouTube fallback. Chrome/Edge: **0.3.207**; Firefox retains the signed **0.3.207** XPI.

The window expands when **Add download** opens, giving the form, thread controls and advanced options more room. Closing the dialog restores the previous window size and position; scrolling remains available on smaller screens.

ED2K **Preview** becomes available after the first MD4-verified part: up to **9,728,000 bytes**, about 9.28 MiB. There is no fixed 30–40 MB total-download threshold. Peers prioritize the beginning and end before middle parts; MP4/AVI may also require the index at the end. Missing ranges wait for data.

Automatic retries preserve the current process's verified preview map while rechecking. The fresh result can invalidate parts; restarting the app or resuming manually requires verification again. If no peer has the remaining parts, the download stays incomplete, retains partial data and retries.

This release also fixes HTTP lock cleanup, resume identity removal when deleting normal downloads, Xtreme listener shutdown and concurrent encrypted-cache writes. ADM **0.8.6**, Xtreme **0.2.3**, extension **0.3.207**.

## Build and contribute

Use the Rust version pinned in `rust-toolchain.toml` and the system dependencies listed in [CI](.github/workflows/ci.yml). Run `cargo test --workspace --locked` and `node --test tests/*.test.cjs`. CI tests all three systems and the transport lab; extra Debian, Fedora and Arch checks are available through manual dispatch. Release reuses validated CI artifacts.

See the [changelog](CHANGELOG.md), [technical review](docs/AUDIT-2026-10-22.md), [performance notes](docs/PERFORMANCE.md) and [security policy](SECURITY.md). After this release, maintenance focuses on bugs found during everyday use.

[PayPal](https://www.paypal.com/cgi-bin/webscr?cmd=_donations&business=jv12802%40gmail.com&currency_code=BRL) · GPL-3.0-or-later.
