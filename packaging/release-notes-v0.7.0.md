# Apocalipse Download Manager 0.7.0

- Native Nibiru Xtreme 0.2.0 included in the portable packages; its exact release and SHA-256 are checked during packaging.
- ED2K video preview: peers help complete the first and tail parts before middle parts. The button becomes available after the first authenticated part (up to 9,728,000 bytes), without requiring a 30–40 MB total or waiting for a source stall.
- Automatic retries preserve an already authenticated preview map during rechecking; the fresh result can revoke invalid parts. Manual resume and app restart require fresh verification.
- Missing ED2K parts remain pending with an explicit explanation, saved partial data and automatic retry. A preview cannot supply data absent from all available sources.
- HTTP destination locks are removed when transfers end. Removing an unfinished normal download also clears its resume identity file.
- Redundant portable-candidate builds removed; distro compatibility checks retained separately. Release builds no longer update the lockfile in CI.
- Browser extension remains 0.3.206; existing signed Firefox package is retained.
