<p align="center">
  <img src="assets/branding/apocalipse-alien.png" width="280" alt="Apocalipse Download Manager alien logo">
</p>

<h1 align="center">Apocalipse Download Manager</h1>

<p align="center">
  <strong>Open-source download manager and browser media detector for Windows, Linux and macOS.</strong><br>
  <strong>Gerenciador de downloads livre com detecção de mídia para Windows, Linux e macOS.</strong><br>
  <strong>适用于 Windows、Linux 和 macOS 的开源下载管理器及浏览器媒体检测工具。</strong>
</p>

<p align="center">
  <a href="https://github.com/linuxhell/apocalipse-download-manager/releases/latest"><img alt="Latest release" src="https://img.shields.io/github/v/release/linuxhell/apocalipse-download-manager?style=flat-square"></a>
  <a href="LICENSE"><img alt="License: GPL-3.0-or-later" src="https://img.shields.io/badge/license-GPL--3.0--or--later-0aa8c2?style=flat-square"></a>
  <img alt="Platforms: Windows, Linux and macOS" src="https://img.shields.io/badge/platforms-Windows%20%7C%20Linux%20%7C%20macOS-182533?style=flat-square">
</p>

<p align="center">
  <a href="https://github.com/linuxhell/apocalipse-download-manager/releases/latest"><strong>Download the latest release</strong></a>
  · <a href="#portable-builds">Choose your platform</a>
  · <a href="https://github.com/linuxhell/apocalipse-download-manager/issues">Report a bug</a>
</p>

<p align="center"><a href="#english">English</a> · <a href="#português-do-brasil">Português do Brasil</a> · <a href="#简体中文">简体中文</a></p>

> [!TIP]
> **Does Apocalipse help you? [Donate via PayPal](https://www.paypal.com/cgi-bin/webscr?cmd=_donations&business=jv12802%40gmail.com&currency_code=BRL) to keep the project alive.**<br>
> **O Apocalipse ajuda você? [Faça uma doação pelo PayPal](https://www.paypal.com/cgi-bin/webscr?cmd=_donations&business=jv12802%40gmail.com&currency_code=BRL) para manter o projeto vivo.**<br>
> **Apocalipse 对您有帮助吗？[通过 PayPal 捐赠](https://www.paypal.com/cgi-bin/webscr?cmd=_donations&business=jv12802%40gmail.com&currency_code=BRL)，帮助这个项目持续发展。**

<a id="english"></a>

> [!IMPORTANT]
> FROM A SEED, SOMETHING MAGNIFICENT IS BORN! · DE UMA SEMENTE NASCE ALGO GRANDIOSO! · 一颗种子，孕育出非凡之物！

## Vision

Apocalipse combines fast resumable downloads, media discovery, streaming capture and torrent workflows in one lightweight application. Its engine is written in Rust, while browser integrations use the cross-browser WebExtension standard.

### Why Apocalipse?

- One portable download manager for Windows, Linux (`.tar.gz` and AppImage) and macOS — no ARM/Apple Silicon build yet
- Media discovery for video, audio and images through signed Chrome, Edge and Firefox extensions (the Firefox extension is reviewed and signed by Mozilla/AMO)
- Resumable HTTP/HTTPS downloads, HLS capture, yt-dlp, FFmpeg and torrent/magnet downloads (aria2, with DHT, Peer Exchange, Local Peer Discovery, encrypted peer connections, multi-tracker and WebSeeding support, file selection, peer info and player preview)
- Apocalipse Link for authenticated, TLS-encrypted file transfers between two computers
- Progressive browser recording with later MP4/AAC export
- 21 visual themes (light and dark), with configurable corner rounding and window transparency
- Native interface and browser extension available in English, Brazilian Portuguese and Simplified Chinese
- Open source, privacy-conscious and built in Rust

## Version 0.4 highlights

- Smart priority queue, duplicate protection, searchable history and URL-list import
- Optional mirrors with automatic failover and SHA-256 verification
- Local-time scheduler and adaptive connection allocation
- Authenticated mobile dashboard at `http://YOUR-PC-IP:17655/mobile`
- Progressive browser recording with later format/codec export
- Versioned, reviewable per-site compatibility rules
- Optional prompt to also delete the original `.torrent` file when removing a torrent from disk
- Per-site credentials and the Apocalipse Link password stored in the operating system's secure credential vault

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
- Explainable strategy selection with aria2 RPC acceleration, native HTTP fallback and automatic content validation
- Removable per-site credentials backed by the operating system secure vault
- Windows 10+, modern Linux distributions and macOS 13+

Sites protected by DRM or access controls are intentionally not bypassed. Users are responsible for downloading only content they are authorized to save.

> [!IMPORTANT]
> Apocalipse Link uses an encrypted TLS transport, operating-system account authentication and trust on first use (TOFU) certificate pinning. Only explicitly shared files, folders and drives are exposed with their configured read-only or read/write permission. For an Internet connection, the listening port must still be reachable through the firewall/router or a trusted VPN.

## Competitive engineering priorities


- strengthen the native HTTP engine with adaptive range scheduling, slow-connection recovery and live mirror rebalancing;
- publish reproducible benchmarks for aria2 RPC and the native fallback, including throughput, CPU, memory, retry behavior and integrity under latency and packet loss;
- improve packaging and distribution while preserving the portable builds;
- keep refining interface consistency, accessibility and first-run behavior.

Benchmark claims will only be published with repeatable scripts, identical connection counts, verified output hashes and raw results. Vendor-authored benchmark numbers are treated as hypotheses until independently reproduced.

## Architecture

| Component | Responsibility |
| --- | --- |
| `apocalipse-core` | Task model, URL classification, resumable HTTP engine and tool abstractions |
| `apocalipse-cli` | Headless development client and core integration testing |
| `apps/desktop` | Multilingual Tauri desktop interface and native tray foundation |
| `browser-extension` | Chromium/Firefox media detector and native-app bridge |

## Portable builds

Apocalipse is distributed primarily as a portable application, with no mandatory installer:

- Windows (x64): a complete folder inside a `.zip`, launched directly from the executable
- Linux (x64): a portable `.tar.gz` and a self-contained AppImage
- macOS (x64): an application bundle (`.app`) inside a `.zip`
- Browser extensions: signed packages for Chrome, Edge and Firefox (Firefox is reviewed and signed by Mozilla/AMO)

The **Portable builds** workflow can be run manually for test artifacts. Tags beginning with `v` attach the same validated packages to GitHub Releases. There are currently no ARM64/Apple Silicon builds or native installers (MSI/DMG/deb) — only the portable x64 packages above. Android is not supported or built.

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

- Downloads portáteis para Windows, Linux (`.tar.gz` e AppImage) e macOS — sem versão ARM/Apple Silicon por enquanto
- Extensões assinadas para Chrome, Edge e Firefox (a extensão do Firefox é revisada e assinada pela Mozilla/AMO)
- Torrents e magnet via aria2, com DHT, Peer Exchange, Local Peer Discovery, conexões criptografadas, multi-tracker e WebSeeding, seleção de arquivos, dados de peers e prévia em player
- Ao remover um torrent, opção de apagar também o arquivo `.torrent` original salvo pelo aplicativo
- Apocalipse Link para transferência de arquivos autenticada e criptografada (TLS) entre dois computadores
- Gravação progressiva de mídia do navegador com exportação posterior em MP4/AAC
- 21 temas visuais (claros e escuros), cantos arredondados e transparência de janela configuráveis
- Melhor vídeo e melhor áudio selecionados por padrão
- Pausa, retomada, filas, temas, proxy e DNS personalizado
- Fila inteligente, pesquisa no histórico, importação de listas, espelhos e verificação SHA-256
- Agendamento local, conexões adaptativas e painel móvel autenticado em `http://IP-DO-PC:17655/mobile`
- Credenciais por site e a senha do Apocalipse Link guardadas no cofre seguro do sistema operacional

Se o Apocalipse for útil para você, [faça uma doação pelo PayPal](https://www.paypal.com/cgi-bin/webscr?cmd=_donations&business=jv12802%40gmail.com&currency_code=BRL) e ajude a manter o desenvolvimento.

## 简体中文

Apocalipse 是一款适用于 Windows、Linux 和 macOS 的自由开源下载管理器。它集成了可恢复 HTTP 下载、媒体检测、yt-dlp、FFmpeg、HLS、种子下载、链接关联和浏览器扩展。程序不会绕过 DRM 或访问控制；请只下载您有权保存的内容。

- Windows、Linux（`.tar.gz` 和 AppImage）和 macOS 便携版本——暂不提供 ARM/Apple Silicon 版本
- 经过签名的 Chrome、Edge 和 Firefox 扩展（Firefox 扩展经 Mozilla/AMO 审核并签名）
- 通过 aria2 支持种子和磁力链接下载，具备 DHT、PEX、本地节点发现、加密连接、多 Tracker 和 WebSeeding 支持，可选择文件、查看节点信息并在播放器中预览
- 删除种子任务时，可选择同时删除应用保存的原始 `.torrent` 文件
- Apocalipse Link：两台电脑之间经过身份验证、TLS 加密的文件传输
- 渐进式浏览器录制，支持后续导出为 MP4/AAC
- 21 种视觉主题（深色和浅色），可自定义圆角和窗口透明度
- 默认选择最佳视频和最佳音频
- 支持暂停、继续、队列、主题、代理和自定义 DNS
- 智能优先级队列、历史搜索、网址列表导入、镜像故障转移和 SHA-256 验证
- 本地时间计划、自适应连接和经过身份验证的移动面板 `http://电脑IP:17655/mobile`
- 按站点保存的凭据和 Apocalipse Link 密码均存储在操作系统的安全凭据库中

如果 Apocalipse 对您有帮助，请[通过 PayPal 捐赠](https://www.paypal.com/cgi-bin/webscr?cmd=_donations&business=jv12802%40gmail.com&currency_code=BRL)，支持项目继续开发。

## License

GPL-3.0-or-later.
