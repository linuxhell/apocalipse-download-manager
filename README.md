<p align="center">
  <img src="assets/branding/apocalipse-alien.png" width="280" alt="Apocalipse Download Manager alien logo">
</p>

<h1 align="center">Apocalipse Download Manager</h1>

<p align="center"><strong>A powerful, intelligent and open-source download manager for Windows, Linux and macOS.</strong></p>

> [!IMPORTANT]
> DE UMA SEMENTE NASCE ALGO GRANDIOSO ! 一颗种子，孕育出非凡之物！FROM A SEED, SOMETHING MAGNIFICENT IS BORN!

## Vision

Apocalipse combines fast resumable downloads, media discovery, streaming capture and torrent workflows in one lightweight application. Its engine is written in Rust, while browser integrations use the cross-browser WebExtension standard.

## Planned capabilities

- Accelerated HTTP/HTTPS downloads with pause, resume, retry and integrity checks
- `.torrent`, magnet, `.m3u8` and URL protocol/file associations
- yt-dlp format discovery with best video + audio selected by default
- FFmpeg and N_m3u8DL-RE integration, health checks and safe updates
- TS to MP4 conversion with lossless fast remux and an H.264/AAC compatibility mode
- Selective torrent file window and peer/session information
- Progressive torrent video preview in VLC, mpv or a user-configured player
- Apocalipse Link for end-to-end encrypted file transfers between two computers, with direct connections and resumable relay fallback
- Browser media discovery grouped into Video, Audio and Images
- Correct thumbnails, estimated sizes and format/quality selection
- In-page download button for supported media, with an explicit user action
- HLS recording to MP4/AAC where the stream and applicable law permit it
- Native tray integration and a low-memory background mode
- Complete UI localization: English by default, Brazilian Portuguese and Simplified Chinese, including the extension
- Explainable strategy selection with optional aria2 RPC and automatic content validation
- Removable per-site credentials backed by the operating system secure vault
- Windows 10+, modern Linux distributions and macOS 13+

Sites protected by DRM or access controls are intentionally not bypassed. Users are responsible for downloading only content they are authorized to save.

## Architecture

| Component | Responsibility |
| --- | --- |
| `apocalipse-core` | Task model, URL classification, resumable HTTP engine and tool abstractions |
| `apocalipse-cli` | Headless development client and core integration testing |
| `apps/desktop` | Multilingual Tauri desktop interface and native tray foundation |
| `browser-extension` | Chromium/Firefox media detector and native-app bridge |

## Portable builds

Apocalipse is distributed primarily as a portable application, with no mandatory installer:

- Windows: a complete folder inside a `.zip`, launched directly from the executable
- Linux: a portable `.tar.gz`; AppImage will be added after compatibility validation
- macOS: an application bundle (`.app`) inside a `.zip`
- Browser-extension files are kept inside the `browser extensions` directory in every package
- Windows ARM64, Linux ARM64 and Apple Silicon-native packages will be added as each target is validated

The **Portable builds** workflow can be run manually for test artifacts. Tags beginning with `v` attach the same validated packages to GitHub Releases. Android is not supported or built.

## Try the current core

```bash
cargo run -p apocalipse-cli -- https://example.com/file.zip ./file.zip
cargo test --workspace
```

See [ROADMAP.md](ROADMAP.md) for delivery milestones and [SECURITY.md](SECURITY.md) for the security model.

## License

GPL-3.0-or-later.
