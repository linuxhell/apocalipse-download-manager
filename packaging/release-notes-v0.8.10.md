# Apocalipse Download Manager 0.8.10

- Bundle Xtreme 0.2.4 to reduce HTTP/HTTPS startup overhead. Single-connection downloads skip HEAD and the one-byte probe; files that cannot be split skip the unused range test.
- Preserve the slider connection cap, validated resume, destination protection, headers, proxies and rate limits. Local measurements show faster startup, with no demonstrated sustained throughput gain for multi-connection transfers.
- Align Tauri package metadata with the desktop version. Retain the Vimeo video/audio fix, compact download dialog, nine recent paths and Mozilla-signed Firefox extension 0.3.207.
- Separately configured Xtreme executables can be updated under Settings → Tools.
