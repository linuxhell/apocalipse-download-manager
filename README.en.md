# Apocalipse Download Manager

[Português](README.md) · [English](README.en.md) · [简体中文](README.zh-CN.md)

Free, portable download manager for Windows, Linux and macOS x64.

[**Download**](https://github.com/linuxhell/apocalipse-download-manager/releases/latest)

| Platform | Package |
| --- | --- |
| Windows x64 | [Portable ZIP](https://github.com/linuxhell/apocalipse-download-manager/releases/latest/download/apocalipse-download-manager-windows-x64-portable.zip) |
| Linux x64 | [Portable tar.gz](https://github.com/linuxhell/apocalipse-download-manager/releases/latest/download/apocalipse-download-manager-linux-x64-portable.tar.gz) · [AppImage](https://github.com/linuxhell/apocalipse-download-manager/releases/latest/download/apocalipse-download-manager-linux-x64.AppImage) |
| macOS x64 | [Portable ZIP](https://github.com/linuxhell/apocalipse-download-manager/releases/latest/download/apocalipse-download-manager-macos-x64-portable.zip) |
| Chrome | [0.3.206 ZIP](https://github.com/linuxhell/apocalipse-download-manager/releases/latest/download/apocalipse-chrome-0.3.206.zip) |
| Edge | [0.3.206 ZIP](https://github.com/linuxhell/apocalipse-download-manager/releases/latest/download/apocalipse-edge-0.3.206.zip) |
| Firefox | [0.3.206 XPI](https://github.com/linuxhell/apocalipse-download-manager/releases/latest/download/apocalipse-firefox-0.3.206.xpi) |

![Downloads](assets/screenshots/downloads.png)

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

## Version 0.7.0

ED2K **Preview** becomes available after the first MD4-verified part: up to **9,728,000 bytes**, about 9.28 MiB. There is no fixed 30–40 MB total-download threshold. Peers prioritize the beginning and end before middle parts; MP4/AVI may also require the index at the end. Missing ranges wait for data.

Automatic retries preserve the current process's verified preview map while rechecking. The fresh result can invalidate parts; restarting the app or resuming manually requires verification again. If no peer has the remaining parts, the download stays incomplete, retains partial data and retries.

This release also fixes HTTP lock cleanup, resume identity removal when deleting normal downloads, Xtreme listener shutdown and concurrent encrypted-cache writes. ADM **0.7.0**, Xtreme **0.2.0**, extension **0.3.206**.

## Build and contribute

Use the Rust version pinned in `rust-toolchain.toml` and the system dependencies listed in [CI](.github/workflows/ci.yml). Run `cargo test --workspace --locked` and `node --test tests/*.test.cjs`. CI tests all three systems and the transport lab; extra Debian, Fedora and Arch checks are available through manual dispatch. Release reuses validated CI artifacts.

See the [changelog](CHANGELOG.md), [technical review](docs/AUDIT-2026-10-22.md), [performance notes](docs/PERFORMANCE.md) and [security policy](SECURITY.md). After this release, maintenance focuses on bugs found during everyday use.

[PayPal](https://www.paypal.com/cgi-bin/webscr?cmd=_donations&business=jv12802%40gmail.com&currency_code=BRL) · GPL-3.0-or-later.
