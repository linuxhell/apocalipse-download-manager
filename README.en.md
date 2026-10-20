<p align="center"><img src="assets/branding/apocalipse-a-blue.svg" width="180" alt="Apocalipse Download Manager"></p>

# Apocalipse Download Manager

[Português do Brasil](README.md) · [English](README.en.md) · [简体中文](README.zh-CN.md)

A portable download manager with a Rust/Tauri interface, browser extension, media capture, torrents, computer-to-computer transfers and native Nibiru engine integration.

**State of this tree: ADM `0.6.1`, extension `0.3.206`, Nibiru/ED2K integration under development on 2026-10-20.** The application version does not mean every branch change has shipped. This README describes the current source; published versions have their own notes in the [changelog](CHANGELOG.md) and [Releases](https://github.com/linuxhell/apocalipse-download-manager/releases).

## Current features

| Area | Implemented in this tree |
| --- | --- |
| HTTP/HTTPS | File-identity-aware resume, segmentation, verified mirrors, SHA-256, queue and adaptive concurrency |
| Torrents/magnets | Nibiru Xtreme, file selection, peer/session information and media preview |
| Browser | Video, audio and image capture; ADM handoff; `pt-BR`, `en` and `zh-CN` |
| Page extraction | Nibiru Power for supported sources and yt-dlp routes for other existing workflows |
| HLS/DASH | Capability-based Nibiru M3U8 integration; existing routes where applicable |
| Local media | Typed Colapse operations: inspection, WAV, FLAC, QOA, experimental QOIR with PPM/PAM, Y4M and MP4 faststart |
| Recordings | Progressive recording and export through configured media engines |
| Apocalipse Link | Authenticated TLS transfers; QUIC is a separate option, disabled by default |
| ED2K/Kad | Independent window, server search, sources, verified parts, explicit sharing and bounded Kad session |
| Interface | 980×620 main window, light/dark mode, system tray and runtime language changes |

<p align="center"><img src="assets/screenshots/downloads.png" width="800" alt="Download interface showing sample data"></p>

Screenshots illustrate the interface; they do not establish public connectivity or execution of the latest checkpoint on every system.

## Getting started

1. Obtain a package from [Releases](https://github.com/linuxhell/apocalipse-download-manager/releases) or build this branch. Extract the entire package to a writable directory.
2. Run ADM. On Windows, keep the supplied DLLs beside the executable; the interface requires WebView2. The Linux desktop uses GTK/WebKit.
3. In **Settings → Tools**, check paths and capabilities. Nibiru binaries use subdirectories under `data/tools/`; custom paths are also supported.
4. Install the corresponding Chrome, Edge or Firefox extension. This tree's manifest is `0.3.206`; building an XPI locally does not automatically give it a Mozilla signature.
5. Select `Português do Brasil`, `English` or `简体中文`. The extension and ED2K window follow the configured language.

The `data/` directory beside the executable stores settings, the queue, logs and tools. Passwords and tokens use the operating system credential vault; copying the portable folder does not guarantee that these secrets migrate. Download only content you are authorized to save.

## Engines and integration

| Engine | Role in ADM | Current contract |
| --- | --- | --- |
| [Nibiru Xtreme](https://github.com/linuxhell/nibiru-xtreme) | Transfers/torrents and embedded ED2K library | Native `nibiru.*` RPC with identity/capability checks |
| [Nibiru Power](https://github.com/linuxhell/nibiru-power) | Media discovery on supported pages | `extract URL --json --timeout 30`; ADM supervisor deadline 40 s |
| [Nibiru M3U8](https://github.com/linuxhell/nibiru-m3u8) | Supported playlists and segments | Native CLI, capabilities and JSONL events |
| [Nibiru Colapse](https://github.com/linuxhell/nibiru-colapse) | Local media files | Typed operations; JSON with `schema_version=1`, `engine`, `operation`, `result` |

All four executables implement `capabilities --json`. ADM rejects incompatible identities/schemas. Power is not passed yt-dlp commands, and Colapse is not passed FFmpeg commands. FFmpeg, FFprobe, yt-dlp and QuickJS still serve routes the native engines do not cover; check which binaries your package contains.

Old `aria2_rpc_*` settings keys are accepted for migration; the current integration launches Xtreme and uses `nibiru.*` RPC. Executable revisions are in [scripts/nibiru-engines.json](scripts/nibiru-engines.json). The ED2K library has its own pin in the [desktop Cargo.toml](apps/desktop/src-tauri/Cargo.toml) and `Cargo.lock`; later documentation-only commits do not change the pinned code.

## Current parameters

### Application and CLI

| Command | Effect |
| --- | --- |
| `apocalipse-desktop` | Open the application |
| `apocalipse-desktop --hidden` | Start with the main window hidden |
| `apocalipse-desktop --open URL` | Open a recognized source; also used by OS associations |
| `apocalipse-desktop --verify-media-routing REPORT.json` | CI diagnostic; write a report and exit before opening the interface |
| `apocalipse-cli URL DESTINATION` | Download through the Rust core, request 8 connections and preserve existing destinations |
| `apocalipse-cli benchmark URL DESTINATION [CONNECTIONS] [MIRROR ...]` | Default 8 connections, range 1–32, adaptive scheduling and JSON summary |

These are Cargo target names; packages may rename the desktop executable. Add `.exe` on Windows. The CLI is a separate core utility and does not control every interface feature. It exposes no header, checksum or overwrite flags.

### Interface settings

| Field in `settings.json` | Default and interpretation |
| --- | --- |
| `max_active_downloads` | `3`; configurable from 1 to 20 tasks |
| `connections_per_download` | `16`; requested limit 1–32; multiplexed streams do not necessarily correspond to TCP connections |
| `adaptive_efficiency` | `true`; adapt concurrency to the engine/transfer |
| `global_bandwidth_limit` | `0` means unlimited; bytes/s, maximum 10 GiB/s |
| `download_directory` | Initially `null`; chosen in the interface |
| `xtreme_path`, `power_path`, `m3u8_path`, `colapse_path` | Optional; default `data/tools/NAME/NAME[.exe]` |
| `xtreme_rpc_enabled`, `xtreme_rpc_auto_start` | `true`; automatic port when `xtreme_rpc_port` is `null` |
| `ffmpeg_path`, `yt_dlp_path`, `qjs_path`, `n_m3u8dl_re_path` | Optional paths for corresponding routes |
| `proxy_enabled`, `proxy_url`, `dns_enabled`, `dns_servers` | Configurable proxy/DNS; enabled flags initially `false` |
| `language`, `dark_mode` | Languages `en`, `pt-BR`, `zh-CN`; defaults `en` and `false` |
| `link_quic_enabled` | `false`; experimental option separate from Link TLS |

Prefer the interface when changing settings. The bridge listens on `127.0.0.1:17654`; Link's base port is `17655`. These numbers are not CLI flags. Do not put secrets in configuration examples.

### Local media

The Nibiru adapter accepts 13 typed operations, 12 of which appear in the panel. Concatenation is exposed through the contract without a multiple-input picker in the current interface. Paths must be absolute; inputs must exist and outputs must be new. Unknown fields and fields belonging to a different operation are rejected.

| Operation | Request fields |
| --- | --- |
| `inspect` | `input` |
| `wav-cut`, `y4m-cut` | `input`, `output`, `start`, `frames` |
| `wav-gain` | `input`, `output`, `db` |
| `wav-normalize` | `input`, `output`, `peak_db` |
| `wav-concat` | `inputs`, `output` |
| `flac-encode`, `flac-decode`, `qoa-encode`, `qoa-decode` | `input`, `output` |
| `qoir-encode` | `input`, `output`; optional `input_color`: `bt709` or `srgb` |
| `qoir-decode` | `input`, `output` |
| `mp4-faststart` | `input`, `output` |

`start`/`frames` count sample frames in WAV and video frames in Y4M. Gain uses dB; normalization targets peak dBFS without LUFS. FLAC is lossless for supported integer profiles. QOA uses lossy PCM16 and has no checksum; it does not preserve the original audio exactly. MP4 inspection does not decode identified codecs. See Colapse's capabilities and README limits.

QOIR is an experimental image format, distinct from QOI. Encoding accepts one P6 PPM or P7 PAM image, with 8-bit RGB/RGBA, and produces lossless QOIR; decoding exports PAM and also accepts the format's lossy levels. The limit is 67,108,864 pixels. Input color defaults to BT.709 unless a recognized color comment or explicit `input_color` applies; selecting sRGB does not transform the pixels. PAM export rejects premultiplied alpha and metadata it cannot represent, including ICC profiles. QOIR has no mandatory checksum. The panel labels the format experimental and shows the color choice only for encoding.

The ADM contract requires `frames >= 1` and a nonoverflowing `start + frames` for cuts; gain ranges from −120 to +120 dB, and normalization through ADM accepts −120 to 0 dBFS, combining the adapter's bound with Colapse's ceiling. Concatenation accepts 1–256 inputs. The media supervisor has a six-hour deadline. These adapter bounds are additional to Colapse core limits.

## ED2K/Kad: actual state

The implementation includes server login/search, sources, direct downloads with MD4/part verification, resume, user-selected sharing, upload queue and server publication of the shared list. Kad uses HELLO, endpoint challenges, iterative queries and direct sources. The [ED2K/Kad document](docs/ed2k-kad-native.md) details packet, contact, queue and cancellation bounds.

Two ED2K downloads can run concurrently; each transfer uses one peer at a time and tries up to 16 direct sources per attempt. Uploads allow 1–64 slots and a total KiB/s limit. The TCP listener prefers 4662, using another port if occupied. The server-assigned ID is authoritative: a local open port does not establish NAT/firewall reachability.

Kad supports up to 512 contacts, 32 pending queries, three seconds per query and endpoint proofs valid for 20 minutes. Direct sources can feed the queue without an ED2K server. Kad keyword search/publication, credits, AICH, obfuscation and full aMule parity remain pending. Local tests between our own components do not establish public interoperability with eMule/aMule.

## Building, testing and packaging

Use stable Rust and run from the root to load `.cargo/config.toml`, which enables reqwest's HTTP/3 API. JavaScript CI uses Node 22; build scripts use Python 3.11 or newer.

```sh
cargo test --locked -p apocalipse-core
cargo build --locked --release -p apocalipse-cli
cargo build --locked --release -p apocalipse-desktop
node --test tests/*.test.cjs
python3 scripts/build_nibiru_engines.py --output dist/nibiru-tools --require-all
```

On Debian/Ubuntu, the desktop uses `build-essential`, `pkg-config`, `libwebkit2gtk-4.1-dev`, `libappindicator3-dev`, `librsvg2-dev`, `libpam0g-dev`, `libssl-dev`. The [CI matrix](.github/workflows/validate-portable.yml) covers Fedora/Arch commands and Windows/macOS jobs. Building only core/CLI does not need GTK/WebKit.

| `build_nibiru_engines.py` option | Behavior |
| --- | --- |
| `--output DIRECTORY` | Required; output engines and `build.json` |
| `--require-all` | Fail if any required revision is missing |
| `--source-root DIRECTORY` | Use local Git clones at `DIRECTORY/nibiru-*`, always at the pinned revision; supports unpublished commits |

The script builds for its execution system, checks versions/capabilities, copies license notices and records SHA-256. It is not a cross-compiler. Unpublished pins require local clones or prior publication of the commits.

### Validation and limits

The current checkpoint passed 341 JavaScript tests across 44 files, executed directly with Node and TAP counts, and all 33 normal native-harness tests. A separate test executed all 13 typed operations with the real Colapse binary. Within the JavaScript suite, four Node/VM panel tests check live language changes and 36 mocked requests across all three languages, including color choices. All three workflows passed YAML validation, Bash-block syntax checks and verification of the integration filter. The earlier audit checkpoint also ran 52 core tests and all four real engines, and cross-compiled the desktop for Windows GNU x64. These are distinct scopes, with repeated tests not added to a total; syntax validation does not execute remote jobs.

The Colapse round 4 checkpoint, with QOIR and parallel FLAC decoding, passed 172 Rust tests (four external/corpus-dependent tests ignored in that invocation), 53 Python tests, formatting, Clippy and smoke checks for 13 operations/21 codec directions. The same frozen Linux executable was used in the real ADM adapter test. [Later fixes guided by FFmpeg source review](https://github.com/linuxhell/nibiru-colapse/blob/main/docs/SOURCE_GUIDED_DEVELOPMENT_2026-10-20.md), independent audio/pixel comparisons and the performance matrix retain their own records in the engine repository. Logs, commands, hashes and limits bind each execution to the tested executable; counts do not add repeated runs or establish complete codec coverage. VM interface tests do not establish Windows WebView runtime behavior.

Compiling Windows does not mean its interface ran. Native Windows execution, public ED2K/Kad connectivity, AppImage packaging and a complete semantic audit still require specific evidence. The [scoped audit](docs/AUDITORIA-NIBIRU-ED2K-2026-10-20.md) records actual coverage. The installer supports signed manifests, but the trusted-key list is empty, so that signature is not yet enforced. See [tool updates](docs/tool-update-signing.md).

## Development and contributions

New technology is evaluated for measurable speed, integrity, quality, compatibility or maintenance benefits. HTTP/3, native codecs and parallelism must retain reproducible results, applicable fallbacks and synchronized documentation. Experiments are not presented as proven improvements. See [performance](docs/PERFORMANCE.md), the [roadmap](ROADMAP.md), [security](SECURITY.md) and [issues](https://github.com/linuxhell/apocalipse-download-manager/issues); historical documents identify their snapshot.

You can [support development through PayPal](https://www.paypal.com/cgi-bin/webscr?cmd=_donations&business=jv12802%40gmail.com&currency_code=BRL).

## License

ADM uses [GPL-3.0-or-later](LICENSE). Engines and tools retain their own licenses and notices; ADM's license does not replace those terms.
