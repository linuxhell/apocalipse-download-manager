# Apocalipse Download Manager 0.4.90 — Extensão 0.3.203

## Português do Brasil

Inclui o XPI do Firefox 0.3.203 assinado pela Mozilla, com as mesmas correções de conexão e de overlay do Chrome e do Edge. O app é idêntico ao 0.4.89.

## English

Ships the Mozilla-signed Firefox XPI 0.3.203 with the same connection and overlay fixes as Chrome and Edge. The app is identical to 0.4.89.

## 简体中文

包含 Mozilla 已签名的 Firefox XPI 0.3.203，与 Chrome 和 Edge 拥有相同的连接与按钮修复。应用本身与 0.4.89 相同。

# Apocalipse Download Manager 0.4.89 — Extensão 0.3.203

## Português do Brasil

A ponte local atende cada conexão em uma thread própria, evitando que requisições lentas ou rajadas de diagnósticos atrasem o heartbeat da extensão. O idioma salvo no `settings.json` é restaurado na abertura, como o tema. A extensão deixa de registrar cada vídeo de prévia ignorado a cada varredura e, no Chrome e no Edge, passa a ter ID fixo (`lfgkfogkggkgacahaidkbhggdolpojjf`). Firefox continua com o XPI assinado 0.3.202.

## English

The local bridge serves each connection on its own thread so slow requests or diagnostic bursts cannot delay the extension heartbeat. The language saved in `settings.json` is restored on startup, like the theme. The extension no longer reports every skipped preview video on every scan and now has a fixed ID on Chrome and Edge (`lfgkfogkggkgacahaidkbhggdolpojjf`). Firefox keeps the signed 0.3.202 XPI.

## 简体中文

本地桥为每个连接使用独立线程，避免缓慢请求或诊断请求激增延迟扩展心跳。启动时恢复 `settings.json` 中保存的语言（与主题一致）。扩展不再在每次扫描时上报每个被跳过的预览视频，并在 Chrome 和 Edge 上使用固定 ID（`lfgkfogkggkgacahaidkbhggdolpojjf`）。Firefox 继续使用已签名的 0.3.202 XPI。

# Apocalipse Download Manager 0.4.88 — Extensão 0.3.202

## Português do Brasil

Corrige playlists HLS sem `.m3u8` na URL (detectadas por Content-Type), o download de vídeo errado em páginas com vários vídeos de mesma duração, o botão Baixar nos YouTube Shorts e a exportação de gravações repetidas (`título.recording (1).webm`), agora com mensagem de erro visível. Após um 403 por impressão digital TLS, o yt-dlp é repetido uma vez imitando um navegador. A verificação de manifesto assinado das ferramentas foi ligada ao instalador e só é aplicada depois que uma chave pública for cadastrada (`docs/tool-update-signing.md`). Extensão 0.3.202 com XPI assinado.

## English

Fixes HLS playlists without `.m3u8` in the URL (detected by Content-Type), wrong-video downloads on pages with several equal-length videos, the Download button on YouTube Shorts and export of repeated recordings (`title.recording (1).webm`), now with a visible error message. After a 403 caused by TLS fingerprinting, yt-dlp is retried once impersonating a browser. Signed-manifest verification for tools is wired into the installer and enforced only once a public key is configured (`docs/tool-update-signing.md`). Extension 0.3.202 with the signed XPI.

## 简体中文

修复 URL 中不含 `.m3u8` 的 HLS 播放列表（按 Content-Type 检测）、多个等长视频页面下载错误视频、YouTube Shorts 下载按钮缺失，以及重复录制（`标题.recording (1).webm`）无法导出的问题，并显示错误提示。遇到因 TLS 指纹导致的 403 时，yt-dlp 会模拟浏览器重试一次。工具的已签名清单校验已接入安装器，添加受信任公钥后才强制执行（`docs/tool-update-signing.md`）。扩展 0.3.202，含已签名 XPI。

# Apocalipse Download Manager 0.4.85 — Extensão 0.3.191

## Português do Brasil

A extensão 0.3.191 aplica à reconexão automática a recuperação do processo de captura já usada pelo botão Conectar, mantendo o token salvo. Também recupera quando o ADM fica disponível depois de abrir o popup. Chrome, Edge e Firefox usam o A azul moderno do tray. Firefox inclui o XPI assinado fornecido pelo mantenedor. A correção do tema da versão 0.4.84 permanece incluída.

## English

Extension 0.3.191 applies the Connect button's worker recovery to automatic reconnection, preserving the stored token, including when ADM becomes available after opening the popup. Chrome, Edge and Firefox use the modern blue A tray icon. Firefox includes the maintainer-supplied signed XPI. The 0.4.84 theme persistence fix remains included.

## 简体中文

扩展 0.3.191 在自动重新连接时使用连接按钮已有的捕获进程恢复逻辑，保留已保存的令牌，并在打开弹窗后 ADM 恢复可用时尝试恢复。Chrome、Edge 和 Firefox 使用托盘的现代蓝色 A 图标。Firefox 包含维护者提供的已签名 XPI。继续包含 0.4.84 的主题保存修复。


# Apocalipse Download Manager 0.4.84 — extensão / extension / 扩展 0.3.190

## 0.4.84 — 2026-10-03

- PT-BR: Corrige a restauração do tema ao iniciar: o tema salvo em settings.json prevalece sobre a memória da janela. Validado pelo usuário após reiniciar o Windows com a extensão existente. A extensão 0.3.190 permanece inalterada.
- EN: Restore the persisted desktop theme at startup instead of overwriting it with a missing or stale webview cache. User-validated after a Windows restart with the existing extension. Browser extension 0.3.190 is unchanged.
- 简体中文：启动时恢复已保存的桌面主题，避免窗口缓存缺失或过期时覆盖配置。用户已在 Windows 重启后使用现有扩展验证。浏览器扩展 0.3.190 保持不变。

# Apocalipse Download Manager 0.4.79 — extensão / extension / 扩展 0.3.189

## 0.4.79 — 2026-10-01

- Direct audio analysis offers post-download FFmpeg conversion; MP4 retains its existing options without the extra control.
- Optional authenticated QUIC receives in Link, extra-path attempts, transfer progress/cancellation and HTTPS fallback.
- Explicit RFC 9842 dictionary downloads in Tools and bounded MoQ object capture in Recordings, with three-language controls.
- Existing files are preserved by the new transports and direct-audio conversion. MARS remains research.
- General-log health, queue/settings persistence failures and extension transport/storage health are now visible in diagnostics and exports. Caught UI errors retain trace correlation; JSON and quoted secrets are sanitized.
- Browser extension updated to 0.3.189. The Firefox package in this release is the Mozilla-signed XPI supplied by the maintainer (SHA-256 `06c39c075b04a81fb2db53338a5a074d424c60e74beff92497b6d6d69e58c78e`). See [release notes](release-notes-v0.4.79.md) for scope and compatibility requirements.


## Português do Brasil

> [!IMPORTANT]
> **Baixe o novo Apocalipse Download Manager 0.4.78, instale a extensão 0.3.188 e atualize o aria2 em Ferramentas para usufruir dos benefícios.**

Esta versão sai de uma auditoria completa de código, revisada linha a linha e com toda a suíte de testes (Rust e navegador) rodada e confirmada antes da publicação. Corrige vários bugs reais de perda de dados e de comportamento silenciosamente incorreto:

- `queue.json` e `settings.json` agora são gravados de forma atômica (arquivo temporário + `rename`); um travamento no meio da escrita já apagou a fila de downloads e as regras de host no passado — isso não acontece mais, e um arquivo corrompido é posto de lado (`.corrupt-<hora>`) em vez de silenciosamente virar um estado vazio.
- Upload pelo Apocalipse Link (painel móvel, porta 17655): uma conexão interrompida no meio não destrói mais o arquivo original; o corpo da requisição agora tem limite de 1 MiB antes de ser lido na memória, e conexões ociosas não autenticadas têm tempo limite.
- Um mirror fora do ar não descarta mais os outros mirrors saudáveis já confirmados na mesma busca.
- Downloads HTTP com mirrors ou SHA-256 esperado agora usam o motor nativo em vez do aria2, que ignorava os dois silenciosamente e mesmo assim marcava a tarefa como concluída.
- A verificação de SHA-256 não sobrescreve mais o hash esperado com o hash de um arquivo corrompido (o que fazia a próxima checagem "passar" contra o mesmo arquivo ruim).
- "Remover do disco" não apaga mais arquivos de mídia de mesmo nome que não têm relação com a tarefa (ex.: remover um download HTTP comum não varre mais `.mp3/.flac/...` do usuário).
- Extração automática de arquivos não sobrescreve mais uma pasta existente com o mesmo nome.
- Correção de um bug introduzido numa versão anterior: a detecção de erro 429 (limite de taxa) usava busca de texto e podia disparar em falso por um nome de arquivo ou contagem de bytes contendo "429"; agora verifica o status HTTP real.
- Pacotes Windows, Linux tar.gz/AppImage e macOS são x64; Firefox inclui o XPI assinado pela Mozilla fornecido pelo mantenedor.

## English

> [!IMPORTANT]
> **Download the new Apocalipse Download Manager 0.4.78, install browser extension 0.3.188, and update aria2 in Tools to benefit from these improvements.**

This release comes from a full code audit, reviewed line by line, with the entire test suite (Rust and browser) re-run and confirmed before publishing. It fixes several real data-loss bugs and silently-incorrect behavior:

- `queue.json` and `settings.json` are now written atomically (temp file + rename); a crash mid-write used to permanently erase the download queue and host rules — that can no longer happen, and a corrupted file is set aside (`.corrupt-<timestamp>`) instead of silently becoming empty default state.
- Apocalipse Link uploads (mobile panel, port 17655): a connection interrupted mid-upload no longer destroys the original file; the request body is now capped at 1 MiB before being buffered, and unauthenticated idle connections now time out.
- A single unreachable mirror no longer discards the other healthy mirrors already confirmed in the same probe.
- HTTP downloads with mirrors or an expected SHA-256 now use the native engine instead of aria2, which silently ignored both and still marked the task Completed.
- SHA-256 verification no longer overwrites the expected hash with a corrupted file's own digest (which used to make the next check "pass" against the same bad file).
- "Remove from disk" no longer deletes unrelated same-named media files (e.g. removing a plain HTTP download no longer sweeps the user's own `.mp3`/`.flac`/etc.).
- Automatic archive extraction no longer overwrites an existing folder with the same name.
- Fixed a bug introduced in an earlier release: 429 (rate limit) detection used plain text matching and could false-trigger on a filename or byte count containing "429"; it now checks the real HTTP status.
- Windows, Linux tar.gz/AppImage and macOS packages are x64; Firefox includes the supplied Mozilla-signed XPI.

## 简体中文

> [!IMPORTANT]
> **请下载新版 Apocalipse Download Manager 0.4.78，安装浏览器扩展 0.3.188，并在"工具"中更新 aria2，以享受这些改进。**

此版本基于一次完整的代码审计：逐行审查，并在发布前重新运行并确认了全部测试套件（Rust 与浏览器端）。修复了多个真实存在的数据丢失问题和静默的错误行为：

- `queue.json` 和 `settings.json` 现在以原子方式写入（临时文件 + 重命名）；以前在写入过程中崩溃会永久清空下载队列和主机规则——现在不会再发生，损坏的文件会被重命名保留（`.corrupt-<时间戳>`），而不是被静默替换为空的默认状态。
- Apocalipse Link 上传（移动端面板，17655 端口）：上传中途连接中断不再破坏原始文件；请求体在缓冲前现有 1 MiB 的大小限制，未认证的空闲连接现在会超时断开。
- 单个失效的镜像不再导致同一次探测中已确认健康的其他镜像被一并丢弃。
- 带有镜像或预期 SHA-256 的 HTTP 下载现在改用原生引擎，而不是会静默忽略这两项、却仍将任务标记为已完成的 aria2。
- SHA-256 校验不再用损坏文件自身的摘要覆盖预期哈希（这曾导致下一次校验对同一个坏文件"通过"）。
- "从磁盘删除"不再删除与任务无关的同名媒体文件（例如删除普通 HTTP 下载不会再清除用户自己的 `.mp3`/`.flac` 等文件）。
- 自动解压不再覆盖已存在的同名文件夹。
- 修复了早期版本引入的一个缺陷：429（限速）检测此前使用纯文本匹配，可能因文件名或字节计数中含有 "429" 而误触发；现在改为检查真实的 HTTP 状态码。
- Windows、Linux tar.gz/AppImage 和 macOS 软件包均为 x64；Firefox 包含维护者提供的 Mozilla 签名 XPI。

[Histórico anterior / Previous releases / 历史版本](https://github.com/linuxhell/apocalipse-download-manager/releases)
