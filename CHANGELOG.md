# Apocalipse Download Manager 0.4.78 — extensão / extension / 扩展 0.3.188

## 0.4.79 — 2026-10-01

- Direct audio analysis offers post-download FFmpeg conversion; MP4 retains its existing options without the extra control.
- Optional authenticated QUIC receives in Link, extra-path attempts, transfer progress/cancellation and HTTPS fallback.
- Explicit RFC 9842 dictionary downloads in Tools and bounded MoQ object capture in Recordings, with three-language controls.
- Existing files are preserved by the new transports and direct-audio conversion. MARS remains research.
- Browser extension remains 0.3.188. See [release notes](release-notes-v0.4.79.md) for scope and compatibility requirements.


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
