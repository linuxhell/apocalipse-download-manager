# Apocalipse Download Manager 0.8.21

- Adds Settings > Themes with eight new coordinated palettes alongside White and Black. Text, controls, status colors and progress bars use matching color pairs; folder icons remain yellow.
- Saves the selected theme in settings.json, restores it before paint, and synchronizes auxiliary windows and the paired Chrome, Edge and Firefox extensions. Download and Record buttons over videos follow the selected theme automatically.
- Includes Portuguese, English and Simplified Chinese labels for themes, feedback and file-part legends.
- Keeps long filenames within their own area in Downloads, Torrents and Recordings, with separate progress bars and action controls. CJK titles have a stable line height.
- Adds a collapsible file-part panel inside the existing list without increasing the window size. It reads confirmed native/Xtreme engine checkpoints and groups blocks into at most 128 cells, preserving gaps and partial groups. Checkpoint updates may lag the transfer; unavailable positions are never inferred from total percentage. Torrent engines without piece-position data and sequential recordings show explicit fallback states.
- Includes ADM portable packages for Windows x64, Linux x64 and macOS Intel x64, plus Linux AppImage. Preserves Nibiru Xtreme 0.2.8 and the existing Reddit and MSU download fixes.
- Includes Chrome/Edge 0.3.217 and the user-supplied Mozilla-signed Firefox 0.3.217 XPI. The Firefox payload matches the tested package, with manifest formatting only; CMS signature and JAR digests were verified against the existing AMO signing CA. XPI: 219606 bytes; SHA256 `69b275514cef74bbefee3f581b941e259e529571baf3490c2a9b0a1e21372825`.
- Validation: 441 Node tests, 180 browser layout combinations using the production renderer/CSS with I/O stubs, and successful Windows/Linux/macOS CI builds with native executable and packaging checks.
