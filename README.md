<p align="center">
  <img src="assets/branding/apocalipse-alien.png" width="280" alt="Apocalipse Download Manager alien logo">
</p>

<h1 align="center">Apocalipse Download Manager</h1>

<p align="center"><strong>A powerful, intelligent and open-source download manager for Windows, Linux and macOS.</strong></p>

> [!IMPORTANT]
> Apocalipse is being built from scratch. The repository currently contains the tested core foundation and browser-extension prototype; it is not yet a production release.

## Vision

Apocalipse combines fast resumable downloads, media discovery, streaming capture and torrent workflows in one lightweight application. Its engine is written in Rust, while browser integrations use the cross-browser WebExtension standard.

## Planned capabilities

- Accelerated HTTP/HTTPS downloads with pause, resume, retry and integrity checks
- `.torrent`, magnet, `.m3u8` and URL protocol/file associations
- yt-dlp format discovery with best video + audio selected by default
- FFmpeg and N_m3u8DL-RE integration, health checks and safe updates
- Selective torrent file window and peer/session information
- Browser media discovery grouped into Video, Audio and Images
- Correct thumbnails, estimated sizes and format/quality selection
- In-page download button for supported media, with an explicit user action
- HLS recording to MP4/AAC where the stream and applicable law permit it
- Native tray integration and a low-memory background mode
- Complete UI localization: English by default, Brazilian Portuguese and Simplified Chinese, including the extension
- Windows 10+, modern Linux distributions and macOS 13+

Sites protected by DRM or access controls are intentionally not bypassed. Users are responsible for downloading only content they are authorized to save.

## Architecture

| Component | Responsibility |
| --- | --- |
| `apocalipse-core` | Task model, URL classification, resumable HTTP engine and tool abstractions |
| `apocalipse-cli` | Headless development client and core integration testing |
| `apps/desktop` | Tauri desktop interface (next milestone) |
| `browser-extension` | Chromium/Firefox media detector and native-app bridge |

## Try the current core

```bash
cargo run -p apocalipse-cli -- https://example.com/file.zip ./file.zip
cargo test --workspace
```

See [ROADMAP.md](ROADMAP.md) for delivery milestones and [SECURITY.md](SECURITY.md) for the security model.

## License

GPL-3.0-or-later.
