# Apocalipse Download Manager 0.8.6

- Fix duplicate torrent root directories and single-file torrents being placed inside an extra folder.
- Send the reserved destination as parent `dir` plus exact `out`, honoring conflict suffixes such as `(1)`.
- Require and bundle Xtreme 0.2.3 with the `BitTorrent-Out` capability. Update separately configured engines under Settings → Tools; older backends are rejected before payload writes with guidance in all three languages.
- Existing downloaded folders are preserved. Mozilla-signed Firefox 0.3.207 is unchanged.
