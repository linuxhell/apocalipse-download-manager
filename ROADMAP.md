# Roadmap

## M0 — foundation (current)

- Rust workspace and stable task domain model
- URL/media classifier
- Streaming HTTP download with safe partial-file resume
- Cross-browser media detector prototype
- CI on Windows, Linux and macOS

## M1 — desktop alpha

- Tauri 2 desktop shell and native tray
- SQLite queue, scheduler, retry policy and bandwidth controls
- Segmented HTTP engine with server capability detection
- Native Messaging bridge authenticated per browser profile
- Windows/macOS/Linux packages in GitHub Releases

## M2 — media intelligence

- yt-dlp inspection and format picker
- FFmpeg and N_m3u8DL-RE adapters
- TS to MP4 conversion (fast remux and compatibility transcode)
- Video/Audio/Image extension tabs with thumbnails and reliable size labels
- HLS capture/recording and merge progress

## M3 — torrents

- libtorrent-backed magnet and torrent sessions
- File selection, priorities, peers, trackers and sequential mode
- Magnet and `.torrent` associations

## M4 — hardening

- Signed update manifests, tool checksum verification and rollback
- Crash recovery, fuzzing and large-file soak tests
- Accessibility, translations and stable releases

## M5 — optional power modules

- aria2 JSON-RPC fallback engine secured to loopback with a per-install secret
- ED2K/Kademlia integration through an optional aMule adapter
- Apocalipse Link: consent-based, end-to-end encrypted computer-to-computer transfers
- Optional local/cloud AI assistant for diagnostics and site-rule suggestions, disabled by default
