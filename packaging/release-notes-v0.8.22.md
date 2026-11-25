# Apocalipse Download Manager 0.8.22

- Bundle Nibiru Xtreme 0.2.9 and link the same pinned engine revision into ADM. Excluded torrent files are no longer created or overwritten when they share pieces with selected files; required boundary bytes stay in internal piece storage for hash verification, resume and upload.
- Require the corrected selection capability before submitting a partial torrent to an externally configured backend. Show update guidance in Portuguese, English and Simplified Chinese.
- Reject explicit empty or invalid file selections, exclude padding from inspection totals, preserve zero expected size and clear stale torrent inspection metadata.
- Reduce direct HTTP request overhead with ranges up to 128 MiB while retaining 4 MiB write buffers, connection caps, validated resume, Retry-After and atomic publication. Documented measurements and a repeatable Windows benchmark are included in the repositories; speed depends on the source, network and storage.
- Preserve Chrome/Edge 0.3.217 and the existing Mozilla-signed Firefox 0.3.217 XPI. No extension code changes or new Firefox signature are required for this release.
- Publish Windows x64, Linux x64 and macOS Intel x64 portable packages and the Linux AppImage, with SHA-256 checksums.

Validation includes the workspace and browser regression suites, backend selection tests, cross-platform CI and independent BitTorrent/ED2K interoperability tests in the engine. See [the audit and measurement limits](https://github.com/linuxhell/apocalipse-download-manager/blob/v0.8.22/docs/AUDIT-2026-11-25.md).
