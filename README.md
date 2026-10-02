<p align="center">
  <img src="assets/branding/apocalipse-a-blue.svg" width="240" alt="Apocalipse Download Manager modern blue A logo">
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

> [!IMPORTANT]
> **Download Apocalipse Download Manager 0.4.83, install browser extension 0.3.190, and update aria2 in Tools to use the current release.**

### What's new in 0.4.83 / 0.3.190

ADM 0.4.83 introduces Cinema Premium: 28 HD visual themes with adjustable transparency and palette-aware contrast, compact Downloads/Torrents/Recordings queues, persistent pause/remove/reveal controls, balanced Logs panes and the modern blue A branding. Browser extension 0.3.190 follows the selected palette, and the Firefox XPI included with the release is Mozilla-signed. Full list in [CHANGELOG.md](CHANGELOG.md).

### Performance

Reported real measurement, using the same 8.17 GB ISO in every test:

| Engine | Configuration | Total time | Average speed |
| --- | --- | --- | --- |
| Classic aria2 | Old default (`-x 1`, one connection) | 103 s | ~79.3 MB/s |
| aria2-next | `--stream-max-connections=16 --file-allocation=trunc` | 88.98 s | ~91.8 MB/s |
| **aria2-ultra** | `-s16 -x16 --file-allocation=trunc` (**current default; no configuration needed**) | **71 s** | **~115.1 MB/s** |

**Compared with classic aria2:** a single connection is exposed to throughput fluctuations. In this test it dropped from ~100 MB/s to ~62 MB/s at 80–88% completion, while aria2-ultra reportedly stayed near ~117.7 MB/s from start to finish. The total times correspond to **~31% less time and ~45% higher average throughput**. The fork changes its defaults so users start with the configuration that performed better in this measurement, instead of discovering and supplying the flags manually.

**Compared with aria2-next:** with 16 connections and `trunc` allocation on both sides, aria2-ultra took **~20% less time** (~25% higher average throughput). aria2-next uses libcurl for HTTP and libtorrent-rasterbar for BitTorrent; aria2-ultra retains classic aria2's engine and adds native BitTorrent v2/hybrid support. Avoiding the replacement of both stacks is the fork's architectural rationale, but this timing test alone does not establish library overhead as the cause or prove that v2/hybrid is aria2-next's only advantage.

These figures describe this particular test, not a guaranteed speed on every server or network. The original fork report states 47%; the rounded times and average speeds above yield approximately 45%. Sources: [aria2-ultra benchmark](https://github.com/linuxhell/aria2-ultra#desempenho) and [aria2-next architecture](https://github.com/AnInsomniacy/aria2-next).

## Vision

Apocalipse combines fast resumable downloads, media discovery, streaming capture and torrent workflows in one lightweight application. Its engine is written in Rust, while browser integrations use the cross-browser WebExtension standard.

### Why Apocalipse?

- One portable download manager for Windows, Linux (`.tar.gz` and AppImage) and macOS — no ARM/Apple Silicon build yet
- Media discovery for video, audio and images through signed Chrome, Edge and Firefox extensions (the Firefox extension is reviewed and signed by Mozilla/AMO)
- Resumable HTTP/HTTPS downloads, HLS capture, yt-dlp, FFmpeg and torrent/magnet downloads (aria2, with DHT, Peer Exchange, Local Peer Discovery, encrypted peer connections, multi-tracker and WebSeeding support, file selection, peer info and player preview)
- Apocalipse Link for authenticated, TLS-encrypted file transfers between two computers
- Progressive browser recording with later MP4/AAC export
- 28 visual themes (light and dark), with configurable corner rounding and window transparency
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

> [!IMPORTANT]
> **Baixe o Apocalipse Download Manager 0.4.83, instale a extensão 0.3.190 e atualize o aria2 em Ferramentas para usar a versão atual.**

### Novidades da 0.4.83 / 0.3.190

O ADM 0.4.83 traz o Cinema Premium: 28 temas visuais em HD, transparência ajustável e contraste adaptado à paleta, filas compactas em Downloads/Torrents/Gravações, controles persistentes de pausar/remover/revelar, painéis de Logs equilibrados e a identidade com o A azul moderno. A extensão 0.3.190 acompanha a paleta selecionada, e o XPI do Firefox incluído na release é assinado pela Mozilla. Lista completa em [CHANGELOG.md](CHANGELOG.md).

### Desempenho

Medição real relatada, usando o mesmo arquivo ISO de 8,17 GB em todos os testes:

| Engine | Configuração | Tempo total | Velocidade média |
| --- | --- | --- | --- |
| aria2 clássico | Padrão antigo (`-x 1`, uma conexão) | 103 s | ~79,3 MB/s |
| aria2-next | `--stream-max-connections=16 --file-allocation=trunc` | 88,98 s | ~91,8 MB/s |
| **aria2-ultra** | `-s16 -x16 --file-allocation=trunc` (**padrão atual, nada a configurar**) | **71 s** | **~115,1 MB/s** |

**Comparado ao aria2 clássico:** uma única conexão fica à mercê da variação de throughput. Neste teste, houve uma queda de ~100 MB/s para ~62 MB/s entre 80–88% do download, enquanto o aria2-ultra manteve, segundo o relato, uma taxa próxima de ~117,7 MB/s do início ao fim. Os tempos totais correspondem a **~31% menos tempo e ~45% mais velocidade média**. Os defaults do fork mudaram para que ele já comece com a configuração que teve melhor desempenho nessa medição, sem exigir que cada usuário descubra e passe as flags manualmente.

**Comparado ao aria2-next:** com 16 conexões e alocação `trunc` nos dois lados, o aria2-ultra levou **~20% menos tempo** (~25% mais velocidade média). O aria2-next usa libcurl para HTTP e libtorrent-rasterbar para BitTorrent; o aria2-ultra mantém o motor do aria2 clássico e adiciona suporte nativo a BitTorrent v2/híbrido. Evitar a substituição das duas stacks é a justificativa arquitetural do fork, mas este teste de tempo, sozinho, não comprova que a sobrecarga das bibliotecas causou a diferença nem que v2/híbrido seja a única vantagem do aria2-next.

Os números descrevem este teste, sem garantir a mesma velocidade em qualquer servidor ou rede. O relato original do fork informa 47%; os tempos e velocidades arredondados da tabela resultam em aproximadamente 45%. Fontes: [medição do aria2-ultra](https://github.com/linuxhell/aria2-ultra#desempenho) e [arquitetura do aria2-next](https://github.com/AnInsomniacy/aria2-next).


O Apocalipse é um gerenciador de downloads livre para Windows, Linux e macOS. Ele reúne downloads HTTP retomáveis, detecção de mídia, yt-dlp, FFmpeg, HLS, torrents, associações de links e integração com extensões do navegador. Sites protegidos por DRM ou controles de acesso não são contornados. Baixe somente conteúdos que você tenha autorização para salvar.

- Downloads portáteis para Windows, Linux (`.tar.gz` e AppImage) e macOS — sem versão ARM/Apple Silicon por enquanto
- Extensões assinadas para Chrome, Edge e Firefox (a extensão do Firefox é revisada e assinada pela Mozilla/AMO)
- Torrents e magnet via aria2, com DHT, Peer Exchange, Local Peer Discovery, conexões criptografadas, multi-tracker e WebSeeding, seleção de arquivos, dados de peers e prévia em player
- Ao remover um torrent, opção de apagar também o arquivo `.torrent` original salvo pelo aplicativo
- Apocalipse Link para transferência de arquivos autenticada e criptografada (TLS) entre dois computadores
- Gravação progressiva de mídia do navegador com exportação posterior em MP4/AAC
- 28 temas visuais (claros e escuros), cantos arredondados e transparência de janela configuráveis
- Melhor vídeo e melhor áudio selecionados por padrão
- Pausa, retomada, filas, temas, proxy e DNS personalizado
- Fila inteligente, pesquisa no histórico, importação de listas, espelhos e verificação SHA-256
- Agendamento local, conexões adaptativas e painel móvel autenticado em `http://IP-DO-PC:17655/mobile`
- Credenciais por site e a senha do Apocalipse Link guardadas no cofre seguro do sistema operacional

Se o Apocalipse for útil para você, [faça uma doação pelo PayPal](https://www.paypal.com/cgi-bin/webscr?cmd=_donations&business=jv12802%40gmail.com&currency_code=BRL) e ajude a manter o desenvolvimento.

## 简体中文

> [!IMPORTANT]
> **请下载 Apocalipse Download Manager 0.4.83，安装浏览器扩展 0.3.190，并在“工具”中更新 aria2，以使用当前版本。**

### 0.4.83 / 0.3.190 更新内容

ADM 0.4.83 带来 Cinema Premium：28 个高清视觉主题、可调透明度与随配色自动适配的对比度；下载、种子和录制队列更加紧凑；暂停、移除和定位文件控件保持清晰可见；日志面板布局更加均衡，并采用现代蓝色 A 品牌标识。浏览器扩展 0.3.190 会跟随所选配色，发布包中的 Firefox XPI 已由 Mozilla 签名。完整列表见 [CHANGELOG.md](CHANGELOG.md)。

### 性能

以下为项目报告的实际测量；所有测试均使用同一个 8.17 GB ISO 文件：

| 引擎 | 配置 | 总耗时 | 平均速度 |
| --- | --- | --- | --- |
| 经典 aria2 | 旧默认值（`-x 1`，单连接） | 103 秒 | ~79.3 MB/s |
| aria2-next | `--stream-max-connections=16 --file-allocation=trunc` | 88.98 秒 | ~91.8 MB/s |
| **aria2-ultra** | `-s16 -x16 --file-allocation=trunc`（**当前默认值，无需额外配置**） | **71 秒** | **~115.1 MB/s** |

**与经典 aria2 相比：**单连接更容易受到吞吐量波动影响。本次测试在下载进度 80–88% 时，从约 100 MB/s 降至约 62 MB/s；据报告，aria2-ultra 从开始到结束保持在约 117.7 MB/s。总耗时对应**约 31% 的时间缩短和约 45% 的平均速度提升**。此分支调整了默认值，让用户直接使用本次测量中更快的配置，无需自行寻找并手动输入参数。

**与 aria2-next 相比：**双方均使用 16 个连接及 `trunc` 文件分配时，aria2-ultra **耗时减少约 20%**（平均速度提高约 25%）。aria2-next 的 HTTP 使用 libcurl，BitTorrent 使用 libtorrent-rasterbar；aria2-ultra 保留经典 aria2 引擎，并加入原生 BitTorrent v2/混合种子支持。避免替换两套协议栈是此分支的架构思路，但单次耗时测试不能证明差异由库的开销造成，也不能证明 v2/混合种子是 aria2-next 的唯一优势。

这些数字仅描述本次测试，不保证所有服务器或网络都有相同速度。分支原报告写为 47%；上述四舍五入的耗时和平均速度计算得出约 45%。来源：[aria2-ultra 测量报告](https://github.com/linuxhell/aria2-ultra#desempenho)及 [aria2-next 架构](https://github.com/AnInsomniacy/aria2-next)。


Apocalipse 是一款适用于 Windows、Linux 和 macOS 的自由开源下载管理器。它集成了可恢复 HTTP 下载、媒体检测、yt-dlp、FFmpeg、HLS、种子下载、链接关联和浏览器扩展。程序不会绕过 DRM 或访问控制；请只下载您有权保存的内容。

- Windows、Linux（`.tar.gz` 和 AppImage）和 macOS 便携版本——暂不提供 ARM/Apple Silicon 版本
- 经过签名的 Chrome、Edge 和 Firefox 扩展（Firefox 扩展经 Mozilla/AMO 审核并签名）
- 通过 aria2 支持种子和磁力链接下载，具备 DHT、PEX、本地节点发现、加密连接、多 Tracker 和 WebSeeding 支持，可选择文件、查看节点信息并在播放器中预览
- 删除种子任务时，可选择同时删除应用保存的原始 `.torrent` 文件
- Apocalipse Link：两台电脑之间经过身份验证、TLS 加密的文件传输
- 渐进式浏览器录制，支持后续导出为 MP4/AAC
- 28 种视觉主题（深色和浅色），可自定义圆角和窗口透明度
- 默认选择最佳视频和最佳音频
- 支持暂停、继续、队列、主题、代理和自定义 DNS
- 智能优先级队列、历史搜索、网址列表导入、镜像故障转移和 SHA-256 验证
- 本地时间计划、自适应连接和经过身份验证的移动面板 `http://电脑IP:17655/mobile`
- 按站点保存的凭据和 Apocalipse Link 密码均存储在操作系统的安全凭据库中

如果 Apocalipse 对您有帮助，请[通过 PayPal 捐赠](https://www.paypal.com/cgi-bin/webscr?cmd=_donations&business=jv12802%40gmail.com&currency_code=BRL)，支持项目继续开发。

## License

GPL-3.0-or-later.


### ADM 0.4.79: optional transports and direct audio

**Português:** Na análise de áudio direto, escolha conversão por FFmpeg para MP3, M4A, OPUS, FLAC ou WAV. MP4 mantém as opções existentes, sem o novo controle. Para testar QUIC no Link, ative a opção nas duas máquinas; IPs adicionais devem pertencer a interfaces locais utilizáveis. Envios usam HTTPS e recebimentos voltam para HTTPS se QUIC falhar. Em **Ferramentas**, o download com dicionário exige servidor RFC 9842 compatível e SHA-256 confiável. Em **Gravações**, a captura MoQ exige relay/transmissão/faixa compatíveis e gera `.admmoq`, sem vídeo reproduzível. MARS permanece pesquisa. Extensão 0.3.189 inclui indicadores de saúde dos diagnósticos.

**English:** Direct-audio analysis supports post-download FFmpeg conversion. MP4 retains its existing options. Enable optional QUIC on both Link peers; extra source IPs must belong to usable local interfaces. Uploads use HTTPS; failed QUIC receives fall back to HTTPS. Dictionary downloads in Tools require compatible RFC 9842 servers and a trusted SHA-256. MoQ capture in Recordings produces `.admmoq` objects, not a playable video. MARS remains research. Extension 0.3.189 adds diagnostics health indicators.

**简体中文：** 直接音频分析支持下载完成后通过 FFmpeg 转换。MP4 保留原有选项。可在两个 Link 客户端上启用 QUIC，额外源 IP 必须属于可用的本地接口。上传使用 HTTPS，QUIC 接收失败时回退至 HTTPS。工具中的字典下载需要兼容 RFC 9842 的服务器和可信 SHA-256。录制中的 MoQ 捕获生成 `.admmoq` 对象，不是可播放视频。MARS 仍为研究。扩展 0.3.189 新增诊断健康状态。

[Scope and testing instructions / Escopo e instruções / 范围与说明](release-notes-v0.4.79.md)
