# Apocalipse Download Manager — Technical Direction 2026

Date: 2026-09-22
Baseline: desktop 0.4.72, browser extension 0.3.169

## Recently fixed and intentionally preserved

The browser capture layer now self-heals when a browser tab was already open before an extension install/reload. The extension probes the content script and MAIN-world hook, reinjects missing layers, keeps Insert as a force-capture path, and records explicit health/recovery events.

Media filename recovery was also hardened. Generic names such as Video.mp4 / Vídeo.mp4 no longer override real page/media titles. SoundCloud can recover a track name from page metadata, browser-tab context or the track URL slug. Browser-assisted downloads can repair a generic filename before desktop handoff.

Heartbeat diagnostics distinguish a healthy capture layer from a real repair:
- capture.layer_healthy
- capture.layer_main_hook_repaired only after verified hook response
- capture.layer_main_hook_repair_unverified when repair cannot be verified

These behaviors are regression-tested and should remain part of the extension reliability contract.

## Current transfer-engine strengths

The native Rust engine already includes:
- segmented HTTP downloads;
- HTTP/3 probing with HTTP fallback;
- safe resume using ETag/Last-Modified identity;
- dual-slot resume journals;
- adaptive worker admission;
- live range stealing from slower workers;
- per-host/network capacity hints;
- multi-source/mirror striping only after content identity is established;
- remote SHA-256 discovery and final integrity verification;
- global and per-download bandwidth limiting;
- custom DNS and proxy support.

aria2 remains useful as a specialist for BitTorrent, Metalink, SFTP and mature multi-protocol workflows. It should not become the universal HTTP engine because its HTTP implementation remains HTTP/1.1 while the native engine already has a path to HTTP/3.

## Recommended engineering priorities

### P1 — Transport capability cache and HTTP Happy Eyeballs

Replace the current session-long “HTTP/3 failed for this host” behavior with a versioned per-origin capability cache:
- HTTP/3 / HTTP/2 / HTTP/1.1 result and TTL;
- range support;
- stable resume validator availability;
- observed RTT, first-byte latency and sustained goodput;
- useful concurrency range;
- transient error rate.

For HTTPS origins, race HTTP/3 against the normal HTTP path after a small delay instead of blocking on a long HTTP/3-only attempt. Cache the successful path for a bounded period and periodically reprobe.

Goal: faster starts, fewer unnecessary retries and better behavior on networks where QUIC is intermittently blocked.

### P2 — Adaptive scheduler v2: streams, not “connections”

The current adaptive worker controller is already strong. The next version should distinguish:
- HTTP/1.1 parallel TCP connections;
- HTTP/2 multiplexed range requests;
- HTTP/3 multiplexed QUIC streams.

Use marginal goodput, RTT, error rate and disk-write pressure to decide concurrency. Do not assume that “16 workers” means 16 network connections on HTTP/2/3.

Add per-origin fairness so one host cannot consume all global worker slots.

### P3 — Durable task database

The desktop currently persists the queue to queue.json. Move operational state to SQLite in WAL mode while keeping JSON import/export for portability.

Suggested tables:
- tasks;
- attempts;
- sources/mirrors;
- transfer checkpoints;
- media metadata;
- engine decisions;
- history;
- diagnostic references.

Benefits:
- atomic task transitions;
- crash-safe history;
- cheaper frequent updates;
- better search/filtering;
- easier future RSS/playlist/subscription support;
- less risk of a single malformed JSON file affecting the whole queue.

### P4 — Signed tool supply chain

The core already contains Ed25519 signed-update manifest verification, but one-click tool downloads currently rely mainly on HTTPS/GitHub release selection plus executable validation.

Connect the Tools downloader to signed manifests containing:
- tool id;
- version;
- platform/architecture;
- URL;
- exact length;
- SHA-256;
- expiry;
- monotonic sequence.

Keep the current version probe as a secondary validation, not the trust mechanism.

Add one-click rollback to the previous verified tool binary.

### P5 — Unified media strategy layer

Do not duplicate what specialist media engines already do well.

Recommended automatic strategy:
1. browser-captured direct file -> native Rust engine;
2. page/extractor workflow -> yt-dlp for metadata/extraction;
3. HLS/DASH/MSS manifest -> N_m3u8DL-RE when its stream model is a better fit;
4. separated audio/video -> parallel acquisition + FFmpeg remux;
5. live/ephemeral media -> recording path;
6. fallback to alternate engine only after classifying the failure.

yt-dlp already supports concurrent HLS/DASH fragment downloads, fragment retries and throttling detection. N_m3u8DL-RE is a dedicated cross-platform DASH/HLS/MSS downloader and continues to add manifest/live features. Treat FFmpeg primarily as verifier/remuxer/transcoder, not the first-choice network downloader.

Expose the selected engine and the reason in the UI: “Native HTTP — direct file”, “N_m3u8DL-RE — DASH manifest”, etc.

### P6 — Download Context Envelope between extension and desktop

Create one versioned structure carried from the browser to the desktop:
- resource URL;
- page URL;
- HTTP method;
- sanitized request headers;
- content type/length;
- suggested filename;
- filename source;
- media title;
- referer/origin scope;
- capture timestamp;
- expiring-URL hint;
- authentication scope identifier;
- companion audio/video relationship;
- trace id.

Never place raw passwords or long-lived secrets in the envelope.

This should become the single handoff contract for normal clicks, popup downloads, captured requests and assisted browser downloads. It will reduce site-specific filename/auth/referrer bugs and make diagnostics much more conclusive.

### P7 — Mirror/source scoring

The native engine correctly refuses to stripe unproven mirrors. Preserve that safety.

After mirrors are verified as the same content, continuously score them by:
- first-byte latency;
- sustained goodput;
- failures/timeouts;
- remaining range performance.

Reassign ranges live when one mirror degrades. Persist short-lived per-origin scores so the next download starts with better ordering.

### P8 — Stronger partial-file integrity

Keep final SHA-256 as the interoperability check.

Additionally maintain a fast internal chunk hash tree for downloaded parts (BLAKE3 is a good candidate) so crash recovery can verify already-written chunks without hashing the entire partial file on every restart.

The internal hash is for local resume integrity; it must not replace a publisher-provided SHA-256/signature.

### P9 — Torrent engine evaluation, not immediate replacement

Keep aria2 as the production torrent engine for now.

Run a side-by-side prototype with a modern libtorrent integration before deciding whether to replace it. Evaluate:
- magnet metadata time;
- fast resume;
- piece/file priority;
- streaming preview behavior;
- uTP;
- DHT/PEX;
- disk I/O;
- memory use;
- packaging size and portability.

Only migrate if the measured benefit is large enough to justify the heavier dependency.

### P10 — Split the desktop backend into modules

apps/desktop/src-tauri/src/main.rs has accumulated many responsibilities.

Move toward modules/services such as:
- task_manager;
- transfer_router;
- browser_bridge;
- media_pipeline;
- tools_manager;
- torrent_manager;
- archive_manager;
- apocalipse_link;
- credentials;
- diagnostics;
- persistence.

Keep Tauri commands thin. This is the highest-leverage maintainability change because it reduces regressions as new protocols and sites are added.

## Features worth adding after reliability work

- first-class WebDAV downloads;
- playlist/channel child-task selection;
- RSS/subscription scheduler;
- automatic subtitle/thumbnail/metadata embedding;
- file-name templates;
- time-range media downloads;
- Apple Silicon and Windows/Linux ARM64 packages;
- reproducible network benchmark suite with latency/packet-loss shaping.

## Things not recommended now

- Replacing the native Rust HTTP engine with aria2.
- Implementing a second custom HLS/DASH parser when yt-dlp and N_m3u8DL-RE already cover the specialist layer.
- Adding many new site-specific rules before the Download Context Envelope and generic failure taxonomy are complete.
- Publishing “faster than X” claims without reproducible benchmark scripts and verified output hashes.

## Suggested delivery sequence

Milestone A:
- transport capability cache + HTTP Happy Eyeballs;
- signed tool manifests + rollback;
- Download Context Envelope.

Milestone B:
- SQLite/WAL task persistence;
- source scoring/rebalancing;
- adaptive scheduler v2.

Milestone C:
- unified media strategy + richer media controls;
- torrent-engine comparison prototype;
- ARM64 distribution.

Milestone D:
- subscriptions/library workflows;
- advanced metadata/subtitle/transcription features.

The objective is to make Apocalipse more reliable, observable and measurably fast before increasing the feature count again.
