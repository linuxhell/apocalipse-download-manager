



<p align="center">
  <img src="assets/branding/apocalipse-a-blue.svg" width="220" alt="Apocalipse Download Manager modern blue A logo">
</p>

<h1 align="center">Apocalipse Download Manager</h1>

<p align="center">
  <strong>Fast, portable, open-source downloads, media capture and torrents.</strong><br>
  <strong>Downloads rápidos, portáteis e livres, captura de mídia e torrents.</strong><br>
  <strong>快速、便携、开源的下载、媒体捕获与种子管理工具。</strong>
</p>

<p align="center">
  <a href="https://github.com/linuxhell/apocalipse-download-manager/releases/latest"><img alt="Latest release" src="https://img.shields.io/github/v/release/linuxhell/apocalipse-download-manager?style=flat-square"></a>
  <a href="LICENSE"><img alt="GPL-3.0-or-later" src="https://img.shields.io/badge/license-GPL--3.0--or--later-0aa8c2?style=flat-square"></a>
  <img alt="Windows Linux macOS" src="https://img.shields.io/badge/platforms-Windows%20%7C%20Linux%20%7C%20macOS-182533?style=flat-square">
  <img alt="Extension 0.3.203" src="https://img.shields.io/badge/browser%20extension-0.3.203-2363ef?style=flat-square">
</p>

<p align="center">
  <a href="https://github.com/linuxhell/apocalipse-download-manager/releases/latest"><strong>Download / Baixar / 下载</strong></a>
  · <a href="#quick-start--início-rápido--快速开始">Quick start</a>
  · <a href="CHANGELOG.md">Changelog</a>
  · <a href="https://github.com/linuxhell/apocalipse-download-manager/issues">Issues</a>
</p>

## Demo / Demonstração / 演示


<p align="center">
  <a href="https://github.com/linuxhell/apocalipse-download-manager/blob/main/assets/demo/adm-demo.mp4">
    <img alt="Apocalipse Download Manager demo (click for the full-quality video)" src="assets/demo/adm-demo-preview.gif" width="800">
  </a>
</p>

<p align="center"><sub>Click the preview to watch the full-quality video.</sub></p>

> [!IMPORTANT]
> **Current release: ADM 0.4.90 + browser extension 0.3.203.** Steadier extension connection (bridge no longer serialized), consistent language between app and extension, fixed extension ID on Chrome/Edge. Earlier fixes: HLS playlists without `.m3u8`, wrong-video downloads on multi-video pages, the Download button on YouTube Shorts and recording export, plus a TLS-impersonation retry for 403 blocks and signed tool-update verification (off until a key is configured). Cinema Premium keeps its **28 HD themes** and compact queues. The Firefox XPI in the release is Mozilla-signed.

## Download the current release

| Platform | Package |
| --- | --- |
| Windows x64 | [Portable ZIP](https://github.com/linuxhell/apocalipse-download-manager/releases/latest/download/apocalipse-download-manager-windows-x64-portable.zip) |
| Linux x64 | [AppImage](https://github.com/linuxhell/apocalipse-download-manager/releases/latest/download/apocalipse-download-manager-linux-x64.AppImage) · [Portable tar.gz](https://github.com/linuxhell/apocalipse-download-manager/releases/latest/download/apocalipse-download-manager-linux-x64-portable.tar.gz) |
| macOS x64 | [Portable ZIP](https://github.com/linuxhell/apocalipse-download-manager/releases/latest/download/apocalipse-download-manager-macos-x64-portable.zip) |
| Chrome | [Extension 0.3.203](https://github.com/linuxhell/apocalipse-download-manager/releases/latest/download/apocalipse-chrome-0.3.203.zip) |
| Edge | [Extension 0.3.203](https://github.com/linuxhell/apocalipse-download-manager/releases/latest/download/apocalipse-edge-0.3.203.zip) |
| Firefox | [Mozilla-signed XPI 0.3.203](https://github.com/linuxhell/apocalipse-download-manager/releases/latest/download/apocalipse-firefox-0.3.203.xpi) |

## Quick start / Início rápido / 快速开始

### English

1. Download the portable package for your platform and extract it. On Linux AppImage, mark the file executable before launching.
2. Install the matching browser extension if you want automatic media detection and browser-to-ADM handoff.
3. Open **Tools** in ADM and update the engines you use, especially aria2, yt-dlp and FFmpeg.

### Português do Brasil

1. Baixe o pacote portátil da sua plataforma e extraia. No Linux AppImage, marque o arquivo como executável antes de abrir.
2. Instale a extensão do seu navegador se quiser detecção automática de mídia e envio direto para o ADM.
3. Abra **Ferramentas** no ADM e atualize os motores que você usa, principalmente aria2, yt-dlp e FFmpeg.

### 简体中文

1. 下载适合您平台的便携包并解压。Linux AppImage 首次运行前请先赋予可执行权限。
2. 如需自动检测网页媒体并发送到 ADM，请安装对应的浏览器扩展。
3. 在 ADM 中打开**工具**，更新您使用的引擎，尤其是 aria2、yt-dlp 和 FFmpeg。

## What Apocalipse does

- Resumable HTTP/HTTPS downloads, mirrors, SHA-256 verification and an adaptive queue
- Torrent and magnet workflows with file selection, peer/session information and player preview
- Browser media discovery for video, audio and images; yt-dlp, HLS and FFmpeg workflows
- Progressive recordings with later MP4/AAC export
- Apocalipse Link for authenticated, TLS-encrypted file transfers between computers
- Per-site credentials stored in the operating system secure credential vault
- 28 visual themes with Cinema Premium scenic backgrounds, adjustable transparency and readable contrast
- English, Brazilian Portuguese and Simplified Chinese in the desktop app and browser extension
- Portable Windows, Linux and macOS x64 builds, with no mandatory installer

Sites protected by DRM or access controls are intentionally not bypassed. Download only content you are authorized to save.

## Release 0.4.90 highlights

This release ships the Mozilla-signed Firefox XPI 0.3.203, so Firefox gets the same connection and overlay fixes as Chrome and Edge.

## Release 0.4.89 highlights

- **Extension connection:** the local bridge serves each connection on its own thread, the extension stops flooding it with per-scan diagnostics, and the UI restores its language from `settings.json`.
- **Fixed extension ID** on Chrome/Edge (`lfgkfogkggkgacahaidkbhggdolpojjf`), so reinstalling from another folder keeps the same ID. The pairing token is still per install.

### Earlier in 0.4.88

- **Capture:** HLS playlists served without `.m3u8` in the URL are detected by Content-Type; on pages with several equal-length videos the download uses the playlist of the video on screen; the Download button is back on YouTube Shorts.
- **Recordings:** repeated recordings (`title.recording (1).webm`) now export to the chosen format, and export errors are shown.
- **Media downloads:** after a 403 caused by TLS fingerprinting, yt-dlp is retried once impersonating a browser (`--impersonate chrome`) when the installed build supports it.
- **Tool updates:** signed-manifest verification (signature, anti-rollback, expiry, SHA-256) is wired into the tool installer and stays dormant until a trusted key is added. See [docs/tool-update-signing.md](docs/tool-update-signing.md).

The interface (introduced in 0.4.83) keeps Downloads, Torrents and Recordings in the same narrow, scrollable task column. Pause/resume, the red remove control and the yellow folder control stay visible on each task. On Windows and macOS, the folder action reveals the task file directly when it exists instead of opening an unrelated folder view.

Cinema Premium adds 28 HD themes with palette-aware contrast and configurable transparency. The About page keeps the selected palette without the scenic wallpaper, while Logs use balanced diagnostic/event panes.

Browser extension 0.3.202 follows the selected palette. The current release includes Chrome and Edge ZIP packages plus the Mozilla-signed Firefox XPI.

## Performance and engineering

Benchmark details and limitations live in [docs/PERFORMANCE.md](docs/PERFORMANCE.md), keeping the homepage focused on downloading and using the application.

For architecture, development status and security details, see [ROADMAP.md](ROADMAP.md), [SECURITY.md](SECURITY.md) and the [changelog](CHANGELOG.md).

## Português do Brasil

O Apocalipse é um gerenciador de downloads livre e portátil para Windows, Linux e macOS. Ele reúne downloads HTTP retomáveis, torrents/magnet, detecção de mídia no navegador, yt-dlp, HLS, FFmpeg, gravações progressivas e Apocalipse Link em uma única interface.

A versão **0.4.88** corrige a captura de playlists HLS sem `.m3u8` na URL, o download de vídeo errado em páginas com vários vídeos, o botão Baixar nos YouTube Shorts e a exportação de gravações repetidas. Também tenta de novo o yt-dlp imitando um navegador quando o site bloqueia com 403 e liga a verificação de atualizações de ferramentas assinadas (ativa quando uma chave for cadastrada). O Cinema Premium mantém os **28 temas em HD** e as filas compactas. A extensão atual é a **0.3.202**, com XPI do Firefox assinado pela Mozilla.

Os detalhes de desempenho foram movidos para [docs/PERFORMANCE.md](docs/PERFORMANCE.md) para a página inicial ficar mais clara.

## 简体中文

Apocalipse 是适用于 Windows、Linux 和 macOS 的自由开源便携下载管理器。它把可恢复 HTTP 下载、种子/磁力链接、浏览器媒体检测、yt-dlp、HLS、FFmpeg、渐进式录制和 Apocalipse Link 集成在同一界面中。

**0.4.88** 版本修复了 URL 中不含 `.m3u8` 的 HLS 播放列表捕获、多视频页面下载错误视频、YouTube Shorts 下载按钮缺失以及重复录制无法导出的问题。遇到 403（TLS 指纹拦截）时，yt-dlp 会模拟浏览器重试一次，并接入了已签名工具更新的校验（添加受信任密钥后启用）。Cinema Premium 保留 **28 个高清主题**与紧凑队列。当前浏览器扩展版本为 **0.3.202**，Firefox XPI 已由 Mozilla 签名。

性能测试的详细数据与限制已移至 [docs/PERFORMANCE.md](docs/PERFORMANCE.md)，使项目首页更简洁。

## Support the project

If Apocalipse helps you, [donate via PayPal](https://www.paypal.com/cgi-bin/webscr?cmd=_donations&business=jv12802%40gmail.com&currency_code=BRL) to support continued development.

Se o Apocalipse for útil para você, [faça uma doação pelo PayPal](https://www.paypal.com/cgi-bin/webscr?cmd=_donations&business=jv12802%40gmail.com&currency_code=BRL) e ajude a manter o desenvolvimento.

如果 Apocalipse 对您有帮助，请[通过 PayPal 捐赠](https://www.paypal.com/cgi-bin/webscr?cmd=_donations&business=jv12802%40gmail.com&currency_code=BRL)，支持项目继续开发。

## License

GPL-3.0-or-later.
