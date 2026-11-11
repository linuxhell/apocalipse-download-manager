# Apocalipse Download Manager 0.8.17

- Bundle Nibiru Xtreme 0.2.8 from the same exact revision as the in-process library.
- Reuse the first HTTP GET, honor bounded 429/503 Retry-After, learn temporary origin caps and recover interrupted ranges without losing verified blocks.
- Honor the selected thread maximum over size defaults; reuse buffers and prepare sparse partial files on Windows through a small safe platform interface.
- Preserve validators, proxy/headers, bandwidth limits, atomic publication, finalization status and torrent/ED2K behavior.
- Keep Chrome/Edge 0.3.210 and Mozilla-signed Firefox 0.3.210 unchanged. Firefox SHA256: 5b730b9ecdd6ba2c0f80447624ef8942672ee2e436f594856a99cb352a51c202.
- Document source audit and controlled comparisons; no claim of higher Internet capacity or universal superiority over IDM/MDM.
