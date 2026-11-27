# Apocalipse Download Manager 0.9.0

- Integrate Nibiru Xtreme 0.3.0: automatic HTTP/3 competition with HTTP/2/1.1 fallback, per-origin useful concurrency learning, native IOCP/io_uring positional disk writes where supported, and BLAKE3 verification of completed HTTP blocks before resume. Preserve connection limits, TLS, proxy routing, publisher SHA-256 and selective torrent storage.
- Store the download queue and history in SQLite transactions. Use WAL on positively identified local filesystems and a rollback journal elsewhere. Migrate the previous JSON once, preserve its original and a backup, reject stale concurrent writers, and preserve existing files on corruption or unsupported schemas.
- Add JSON queue export and URL import from an exported JSON list. Imports create new transfers rather than replaying another installation's states or destinations.
- Retry failed additional QUIC Link paths without blocking the working payload stream. Report per-path UDP counters, RTT, recovery attempts and transfer time. Include an alternating independent-interface benchmark harness; physical multi-WAN aggregation remains experimental and unverified here.
- Keep Chrome/Edge and Mozilla-signed Firefox extensions at 0.3.217; their capture protocol remains compatible.

Performance depends on the server, connection, OS and storage. The maintainer's earlier Windows ISO result (ADM 84 seconds, IDM 93 seconds with eight connections) remains a reported result, not an independent universal comparison. See docs/UPGRADE-2026-11-27.md for validation and limits.
