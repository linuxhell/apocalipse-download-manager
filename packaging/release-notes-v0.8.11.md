# Apocalipse Download Manager 0.8.11

- Keep the clicked player source ahead of unrelated HLS adverts/previews. Empty placeholders do not inherit other playlists, and direct HLS players analyze only their own manifest.

- Chrome/Edge 0.3.208 refreshes late player sources so direct MP4 downloads reveal Download instead of leaving only Record.
- Give recording startup visible feedback, prevent repeated starts and recover from playback timeouts. Firefox retains signed 0.3.207; the new extension fix is in Chrome/Edge 0.3.208.

- Bundle Xtreme 0.2.5 with automatic peer-port selection that reserves UDP/uTP before opening TCP on the same port. Addresses the uTP startup failure reported as Windows error 10013.
- Bound automatic retries and cancel abandoned socket dispatchers. Preserve explicitly configured ports, uTP, torrent and magnet behavior.
- Retain HTTP startup improvements, Vimeo video/audio handling, the compact dialog, nine recent paths and Mozilla-signed Firefox extension 0.3.207.
- Update separately configured Xtreme executables under Settings → Tools.
