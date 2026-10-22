# Apocalipse Download Manager 0.7.0

- Native Nibiru Xtreme 0.2.0 included in the portable packages; built from the same pinned source revision linked by the desktop app. Release assets include SHA-256 checksums.
- ED2K video preview: peers help complete the first and tail parts before middle parts. The button becomes available after the first authenticated part (up to 9,728,000 bytes), without requiring a 30–40 MB total or waiting for a source stall.
- Automatic retries preserve an already authenticated preview map during rechecking; the fresh result can revoke invalid parts. Manual resume and app restart require fresh verification.
- Missing ED2K parts remain pending with an explicit explanation, saved partial data and automatic retry. A preview cannot supply data absent from all available sources.
- HTTP destination locks are removed when transfers end. Removing an unfinished normal download also clears its resume identity file.
- CI and Release consolidated: publication reuses the three tested platform builds. Transport tests run in CI; extra distro checks remain available manually. Lockfiles remain fixed.
- Browser extension remains 0.3.206; existing signed Firefox package is retained.

- Idle engine listeners now stop with their owner. Encrypted chunk cache writes use independent temporary files, normalize digest casing and allow repair after authentication failures.
