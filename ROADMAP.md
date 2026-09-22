# Roadmap

## Current usable baseline — v0.4.74

- Rust workspace and stable task domain model
- HTTP/HTTPS downloads with queue, pause, resume and retry, including
  segmented transfer, a resume journal with ETag/digest identity checks,
  and SHA-256 verification before promoting a finished file
- Media inspection through yt-dlp with format and audio selection
- Chrome, Edge and signed Firefox browser extensions
- Facebook, Instagram and TikTok media workflows
- Torrent/magnet downloads with file selection, peer data and player preview
- Native tray, themes, clipboard detection and protocol/file associations
- Per-tool updates for yt-dlp, FFmpeg, ffprobe, aria2, N_m3u8DL-RE and QuickJS
- Apocalipse Link authenticated local/remote file browsing and transfers,
  TLS-encrypted with trust-on-first-use certificate pinning
- Every persisted secret (bridge token, Link password, proxy password,
  per-host website credentials) stored in the OS credential vault via
  `keyring`, with automatic migration from legacy plaintext storage
- End-to-end structured diagnostics with privacy-safe ZIP export
- Linux AppImage bundle alongside the portable builds
- CI on Windows, Linux and macOS

## Next — reliability and security

- Add an explicit remote-connection consent prompt for Apocalipse Link.
  Transport is already TLS-encrypted with certificate pinning, but a
  first-time remote connection is accepted on password match alone —
  there is no user-facing approval step before it's granted.
- Wire `signed_update.rs`'s manifest/signature/checksum verification into
  the actual tool-update path. The verifier (ed25519 signatures, anti-rollback
  sequencing, expiry, SHA-256 artifact checks) is implemented and unit-tested,
  but `verify_update_manifest` is never called from the real download flow —
  `update_tool`/`download_tool` fetch straight from the GitHub "latest
  release" API and only sanity-check the binary by running `--version`.
- Make Apocalipse Link transfers resumable with integrity checks. Today
  pause/resume only holds an in-memory flag on the live transfer task;
  `download_link_file_to` truncates the destination and re-requests the
  whole file with no `Range`/`If-Range` support and no checksum, unlike the
  HTTP download engine in `apocalipse-core`, which already has both. A
  dropped connection or app restart mid-transfer means starting over.
- Crash recovery beyond the existing HTTP segment/resume journal: extend
  equivalent recovery to Apocalipse Link transfers and torrent sessions.
- Fuzzing and large-file soak tests. Neither exists yet — no `fuzz/`
  target, no fuzzing dependency, and CI runs only the regular test suite,
  no long-running/large-file soak job.
- Close the accessibility gap on recently added screens. Localization is
  in good shape (`link.html` already uses `data-i18n` throughout), but
  ARIA coverage is thin: `link.html` and `mobile.html` have close to no
  `aria-*` attributes, so transfer status/progress and controls like
  pause/cancel/delete aren't announced to assistive tech.

## Later — expanded distribution

- ARM64 builds for Windows and Linux and native Apple Silicon validation.
  The release matrix currently only builds Windows x64, Linux x64 and
  macOS x64 (Intel runner) portable builds.
- Native installers (MSI, DMG, deb) beyond the AppImage already shipped
  for Linux — Windows and macOS still ship as portable/zipped builds only.
- Direct internet connectivity through an encrypted resumable relay (no
  STUN/TURN/NAT traversal exists yet; Apocalipse Link is local/LAN-only).
- Signed, reviewable site-rule updates. Site classification rules
  (`classifier.rs`, `strategy.rs`) are static domain lists compiled into
  the binary today, with no remote update mechanism of any kind yet —
  signed or not.
