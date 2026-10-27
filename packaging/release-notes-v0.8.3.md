# Apocalipse Download Manager 0.8.3

- Fix portable movie filenames and isolate media temporary directories by task; retain legitimate dollar signs and apostrophes.
- Finalize only the generated media without replacing existing files; remove unsafe cleanup based on filename similarity.
- Respect the chosen YouTube fragment connection count and fix concurrent thumbnail-cache writes.
- Validate extraction trees before publishing and use Xtreme 0.2.2 torrent parsing, including v2/hybrid torrents.
- Includes the Mozilla-signed Firefox 0.3.207 extension and all existing converter and download-dialog features.
- Audit scope, regression checks and limitations: docs/AUDIT-2026-10-27.md.
