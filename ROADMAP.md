# Roadmap

## Current usable baseline — v0.4.5

- Rust workspace and stable task domain model
- HTTP/HTTPS downloads with queue, pause, resume and retry
- Media inspection through yt-dlp with format and audio selection
- Chrome, Edge and signed Firefox browser extensions
- Facebook, Instagram and TikTok media workflows
- Torrent/magnet downloads with file selection, peer data and player preview
- Native tray, themes, clipboard detection and protocol/file associations
- Per-tool updates for yt-dlp, FFmpeg, ffprobe, aria2, N_m3u8DL-RE and QuickJS
- Apocalipse Link authenticated local/remote file browsing and transfers
- Matrix Ultimate v2 AI continuous diagnostics, correction proposals and per-site rollback
- CI on Windows, Linux and macOS

## Next — reliability and security

- Encrypt Apocalipse Link transport end-to-end and add explicit remote consent
- Store every persisted secret in the operating-system credential vault
- Signed engine-update manifests with mandatory checksum verification
- Crash recovery, fuzzing and large-file soak tests
- Resumable Apocalipse Link transfers with progress and integrity checks
- Accessibility and complete localization of every recently added screen

## Later — expanded distribution

- ARM64 builds for Windows and Linux and native Apple Silicon validation
- AppImage and optional native installers
- Direct internet connectivity through an encrypted resumable relay
- Further Matrix versions with signed, reviewable rule updates
