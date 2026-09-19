# Competitive audit: VidBee vs Apocalipse Download Manager

Date: 2026-09-19

## Competitive target

Apocalipse should combine:
- the transfer speed and breadth expected from IDM/Ghost-class managers;
- VidBee-class social/media workflows;
- Apocalipse's existing browser capture, HLS, torrent, portable architecture, Matrix diagnostics and Apocalipse Link.

The goal is not to clone VidBee. The goal is to preserve Apocalipse's stronger acquisition paths and add the missing media-workflow layer.

## Social-network audit

### Facebook
Apocalipse is already deeper in browser capture. It binds actions to the visible player, resolves Reels/feed/watch/permalink identities, canonicalizes composite Facebook video URLs, carries browser cookies/user-agent/referer to the desktop downloader, and uses an aria2 acceleration path for Facebook media.

VidBee is stronger in reusable authentication setup and cookie-health guidance.

Use in Apocalipse:
- keep player-bound capture and canonicalization;
- add generic cookie-source health/fallback;
- use a stronger generic yt-dlp retry policy.

### Instagram
Apocalipse resolves the clicked player and can use an already-playable media URL before falling back to a permalink. This is a strong feed/Reels path.

VidBee is stronger in reusable browser/cookies.txt authentication and generic post-processing.

Use in Apocalipse:
- keep direct-player capture;
- add configurable browser/cookies.txt fallback;
- add subtitles/metadata/thumbnail controls where yt-dlp exposes them.

### TikTok
Apocalipse is substantially more specialized:
- player identity resolver;
- per-video permalink resolution;
- blob/MediaSource fallback;
- captured network-media inspection;
- video/audio pairing by timing and metadata;
- ambiguity detection;
- exact-stream recording fallback;
- authenticated local preview;
- trace diagnostics.

VidBee mainly uses generic yt-dlp plus authentication/retry/subtitle handling.

Use in Apocalipse:
- preserve the current specialized TikTok path;
- borrow only VidBee's generic retry/auth/error-classification strengths.

### Twitch
VidBee is ahead in deliberate Twitch handling. It knows the auth-token cookie, recognizes Twitch as authenticated media, and avoids subtitle attempts when auth is missing.

Implemented in the current Apocalipse branch:
- Twitch domains now participate in portable Netscape-cookie handling;
- Twitch inherits the stronger generic yt-dlp retry/extractor/socket policy.

Remaining:
- Twitch VOD/live UX;
- chat/subtitle policy;
- cookie-health diagnostics;
- live reconnect tests.

### Bilibili
VidBee is ahead in:
- SESSDATA/DedeUserID cookie health;
- Bilibili AI subtitle aliases;
- danmaku-aware behavior;
- auth-gated subtitle logic;
- metadata truncation recovery;
- format fallback detection;
- Bilibili-specific media/transcription workarounds.

Implemented in the current Apocalipse branch:
- bilibili.com, b23.tv and bili.tv participate in authenticated cookie handling;
- cookie scope is normalized to bilibili.com;
- generic yt-dlp resilience applies to Bilibili inspection and download.

Remaining:
- subtitle/danmaku rules;
- premium-format fallback detection;
- richer playlist/channel support;
- explicit Bilibili cookie-health UI.

## The 1000+ sites layer

Both products rely on yt-dlp's extractor catalog for the long tail. The difference is architectural.

VidBee treats yt-dlp as the main acquisition abstraction and builds a strong media-library workflow around it.

Apocalipse combines yt-dlp with:
- its own segmented Rust HTTP engine;
- N_m3u8DL-RE HLS;
- aria2/torrent;
- browser request interception;
- direct CDN capture;
- player identity for difficult social feeds;
- recording fallback;
- Matrix/diagnostics.

Best competitive strategy:
1. keep specialized capture where it is stronger than generic yt-dlp;
2. make generic yt-dlp behavior uniformly resilient across the extractor catalog;
3. share authentication, retry, error classification and post-processing across both paths;
4. fall back automatically between page extraction, direct captured media, HLS/DASH and recording.

## VidBee capabilities currently stronger than Apocalipse

- browser/cookies.txt authentication setup and cookie health;
- generic bounded yt-dlp retries, extractor retries and socket timeout;
- subtitles and automatic captions;
- embedded thumbnail/metadata/chapters;
- MP4/MKV/WebM/original output controls;
- custom filename templates;
- time-range downloads;
- playlist/channel inspection;
- retry-scheduled task state and stable error taxonomy;
- RSS subscriptions;
- local media import;
- searchable local transcription;
- GPU-aware ASR selection;
- speaker diarization;
- AI over transcripts with multiple providers;
- richer metadata history and completion notifications.

## Areas where Apocalipse is stronger

- own segmented Rust HTTP engine;
- browser-bound social capture for difficult feeds;
- direct CDN request identity;
- HLS/N_m3u8DL-RE path;
- torrents/magnets;
- generic file downloads beyond media;
- Matrix diagnostics and site-rule repair;
- portable lightweight architecture;
- Apocalipse Link remote transfer.

## First changes implemented from this audit

- metadata inspection now ignores external yt-dlp config and uses bounded retries, extractor retries, retry sleeps and a socket timeout;
- media downloads now use the stronger retry policy across the full yt-dlp extractor catalog, not only selected social sites;
- authenticated cookie handling now covers Twitch and Bilibili aliases in addition to Facebook, Instagram and TikTok;
- lookalike/spoofed domains remain rejected by exact/subdomain matching tests.

## Next implementation priorities

1. cookie source settings + health checks;
2. stable media error categories + retry scheduling;
3. subtitle/metadata/thumbnail/chapter/container controls;
4. playlist/channel inspection and selectable child tasks;
5. RSS subscriptions;
6. local media library + transcription;
7. optional AI over transcripts;
8. complete the previously approved next-generation transfer-engine priorities as a separate milestone.
