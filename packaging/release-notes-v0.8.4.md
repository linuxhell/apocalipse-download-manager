# Apocalipse Download Manager 0.8.4

- Fix FFmpeg/FFprobe update failures on Windows, including sharing violations (os error 32).
- Restore the previous pair if either replacement or post-installation validation fails.
- Use unique staging and backup directories; retain locked backups without reporting a successfully validated update as failed.
- Briefly retry Windows replacement locks and show a Portuguese, English or Chinese message if another process continues to block replacement. Running downloads, recordings and conversions are not terminated by the updater.
- Includes the unchanged Mozilla-signed Firefox 0.3.207 extension and Xtreme 0.2.2.
