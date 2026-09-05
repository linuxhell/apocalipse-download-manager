<p align="center">
  <img src="assets/branding/apocalipse-alien.png" width="280" alt="Apocalipse Download Manager alien logo">
</p>

<h1 align="center">Apocalipse Download Manager</h1>

<p align="center"><a href="#english">English</a> · <a href="#português-do-brasil">Português do Brasil</a> · <a href="#简体中文">简体中文</a></p>

<a id="english"></a>

<p align="center"><strong>A powerful, intelligent and open-source download manager for Windows, Linux and macOS.</strong></p>

> [!IMPORTANT]
> DE UMA SEMENTE NASCE ALGO GRANDIOSO ! 一颗种子，孕育出非凡之物！FROM A SEED, SOMETHING MAGNIFICENT IS BORN!

## Vision

Apocalipse combines fast resumable downloads, media discovery, streaming capture and torrent workflows in one lightweight application. Its engine is written in Rust, while browser integrations use the cross-browser WebExtension standard.

## Planned capabilities

- Accelerated HTTP/HTTPS downloads with pause, resume, retry and integrity checks
- `.torrent`, magnet, `.m3u8` and URL protocol/file associations
- yt-dlp format discovery with best video + audio selected by default
- FFmpeg and N_m3u8DL-RE integration, health checks and safe updates
- TS to MP4 conversion with lossless fast remux and an H.264/AAC compatibility mode
- Selective torrent file window and peer/session information
- Progressive torrent video preview in VLC, mpv or a user-configured player
- Apocalipse Link for authenticated direct file transfers between two computers, including a same-PC test mode
- Browser media discovery grouped into Video, Audio and Images
- Correct thumbnails, estimated sizes and format/quality selection
- In-page download button for supported media, with an explicit user action
- HLS recording to MP4/AAC where the stream and applicable law permit it
- Native tray integration and a low-memory background mode
- Complete UI localization: English by default, Brazilian Portuguese and Simplified Chinese, including the extension
- Explainable strategy selection with optional aria2 RPC and automatic content validation
- Removable per-site credentials backed by the operating system secure vault
- Windows 10+, modern Linux distributions and macOS 13+

Sites protected by DRM or access controls are intentionally not bypassed. Users are responsible for downloading only content they are authorized to save.

> [!WARNING]
> The current Apocalipse Link transport is not end-to-end encrypted. Use it only on a trusted local network or through a trusted VPN. Internet relay, transport encryption and resumable transfers remain planned hardening work.

## Architecture

| Component | Responsibility |
| --- | --- |
| `apocalipse-core` | Task model, URL classification, resumable HTTP engine and tool abstractions |
| `apocalipse-cli` | Headless development client and core integration testing |
| `apps/desktop` | Multilingual Tauri desktop interface and native tray foundation |
| `browser-extension` | Chromium/Firefox media detector and native-app bridge |

## Portable builds

Apocalipse is distributed primarily as a portable application, with no mandatory installer:

- Windows: a complete folder inside a `.zip`, launched directly from the executable
- Linux: a portable `.tar.gz`; AppImage will be added after compatibility validation
- macOS: an application bundle (`.app`) inside a `.zip`
- Browser-extension files are kept inside the `browser extensions` directory in every package
- Windows ARM64, Linux ARM64 and Apple Silicon-native packages will be added as each target is validated

The **Portable builds** workflow can be run manually for test artifacts. Tags beginning with `v` attach the same validated packages to GitHub Releases. Android is not supported or built.

## Try the current core

```bash
cargo run -p apocalipse-cli -- https://example.com/file.zip ./file.zip
cargo test --workspace
```

See [ROADMAP.md](ROADMAP.md) for delivery milestones and [SECURITY.md](SECURITY.md) for the security model.

## Support the project

If Apocalipse helps you, [donate via PayPal](https://www.paypal.com/cgi-bin/webscr?cmd=_donations&business=jv12802%40gmail.com&currency_code=BRL) to support continued development.

## Português do Brasil

O Apocalipse é um gerenciador de downloads livre para Windows, Linux e macOS. Ele reúne downloads HTTP retomáveis, detecção de mídia, yt-dlp, FFmpeg, HLS, torrents, associações de links e integração com extensões do navegador. Sites protegidos por DRM ou controles de acesso não são contornados. Baixe somente conteúdos que você tenha autorização para salvar.

- Downloads portáteis para Windows, Linux e macOS
- Extensões para Chrome, Edge e Firefox
- Melhor vídeo e melhor áudio selecionados por padrão
- Pausa, retomada, filas, temas, proxy e DNS personalizado

Se o Apocalipse for útil para você, [faça uma doação pelo PayPal](https://www.paypal.com/cgi-bin/webscr?cmd=_donations&business=jv12802%40gmail.com&currency_code=BRL) e ajude a manter o desenvolvimento.

## 简体中文

Apocalipse 是一款适用于 Windows、Linux 和 macOS 的自由开源下载管理器。它集成了可恢复 HTTP 下载、媒体检测、yt-dlp、FFmpeg、HLS、种子下载、链接关联和浏览器扩展。程序不会绕过 DRM 或访问控制；请只下载您有权保存的内容。

- Windows、Linux 和 macOS 便携版本
- Chrome、Edge 和 Firefox 扩展
- 默认选择最佳视频和最佳音频
- 支持暂停、继续、队列、主题、代理和自定义 DNS

如果 Apocalipse 对您有帮助，请[通过 PayPal 捐赠](https://www.paypal.com/cgi-bin/webscr?cmd=_donations&business=jv12802%40gmail.com&currency_code=BRL)，支持项目继续开发。

## License

GPL-3.0-or-later.
