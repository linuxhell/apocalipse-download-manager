<p align="center"><img src="assets/branding/apocalipse-a-blue.svg" width="160" alt="Apocalipse"></p>

[Português](README.md) · [English](README.en.md) · [简体中文](README.zh-CN.md)

# Apocalipse Download Manager

Portable Rust/Tauri download manager with a browser extension.

## Project status

This source base is **0.6.1**, with extension **0.3.206**. Code auditing is in progress; the source version does not mean this branch's changes have been released.

ADM continues to use external **aria2**, **yt-dlp nightly**, **N_m3u8DL-RE**, **FFmpeg**, **QuickJS**, extractor and player tools. Nibiru Xtreme is developed separately: replacing aria2 requires completing the engine and validating its integration. This branch does not integrate Nibiru M3U8, Power or Colapse.

## Installation

Use the [releases page](https://github.com/linuxhell/apocalipse-download-manager/releases/latest). Check the actual published version before choosing an extension.

| Platform | Package |
| --- | --- |
| Windows x64 | Portable ZIP |
| Linux x64 | AppImage or portable tar.gz |
| macOS x64 | Portable ZIP |
| Chrome / Edge | Extension ZIP |
| Firefox | Mozilla-signed XPI, when included in the release |

Extract the package, launch the app and configure tools under **Settings → Tools**. Install the matching extension to send downloads from your browser. An unsigned test XPI is not equivalent to a signed package for stable Firefox.

## Features and limitations

- Resumable HTTP/HTTPS, queues, bandwidth limits, mirrors and SHA-256 verification.
- Torrents and magnets through aria2, file selection and preview through the configured player.
- Extension media detection and yt-dlp extraction; N_m3u8DL-RE for HLS and FFmpeg for audio/conversion flows.
- QuickJS as yt-dlp's JavaScript runtime; keep its path configured for sites requiring JavaScript challenges.
- Progressive recordings and media export.
- Apocalipse Link: selected shares, TLS and system-account authentication for remote access. SMB share discovery depends on the operating system.
- Brazilian Portuguese, English and Simplified Chinese in the app and extension.

Compatibility depends on the site, installed tools and operating system. Transport prototypes do not establish universal support or public-network validation. Signed tool manifests require configured trusted keys; see [update signing](docs/tool-update-signing.md).

## Structure and development

| Path | Responsibility |
| --- | --- |
| `crates/apocalipse-core` | Models, classification, transfers and policies |
| `crates/apocalipse-cli` | Command-line interface |
| `apps/desktop/src-tauri` | Desktop, tools, persistence and bridge |
| `apps/desktop/ui` | App and Link interfaces |
| `browser-extension` | Browser capture and handoff |
| `experiments/transport-lab` | Experimental transports used by the desktop |
| `tests` | JavaScript regressions and contracts |

Requires stable Rust and Node.js. Linux desktop builds also require WebKitGTK 4.1, GTK/AppIndicator, librsvg, PAM and build tools; workflows list distribution-specific packages.

```sh
cargo fmt --all -- --check
cargo test --locked -p apocalipse-core -p apocalipse-cli
cargo test --workspace --locked
cargo build --release --locked -p apocalipse-desktop
node --test tests/*.test.cjs
```

Full workspace tests require the desktop's native dependencies. Compilation and passing tests do not establish real-world validation on Windows, Linux and macOS.

## CI and documentation

- `validate-portable.yml`: tests, Linux compatibility and candidate packages on three platforms; no release publication.
- `transport-lab.yml`: regressions for transports used by the desktop.
- `release.yml`: builds and publishes releases; an audit must not be presented as a final release.

See the [changelog](CHANGELOG.md), [roadmap](ROADMAP.md), [security policy](SECURITY.md) and [performance notes](docs/PERFORMANCE.md). Downloaded files, partials and checkpoints are not repository garbage.

## Support and license

[Donate via PayPal](https://www.paypal.com/cgi-bin/webscr?cmd=_donations&business=jv12802%40gmail.com&currency_code=BRL).

GPL-3.0-or-later; see [LICENSE](LICENSE).
