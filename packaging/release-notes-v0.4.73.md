# Apocalipse Download Manager v0.4.73

## Português (Brasil)

### Apocalipse AI
- O ícone do **Apocalipse AI** agora pisca quando há atividade nova (mensagem da assistente) e o painel não está aberto, deixando de piscar assim que o painel é aberto.
- Legenda da página **Sobre** simplificada, mantendo o texto igual nos três idiomas suportados.

### Diagnóstico
- O arquivo `logs/by-component/thumbnails.jsonl` agora é sempre incluído no pacote de diagnóstico exportado, mesmo quando nenhum evento de miniatura foi registrado, mantendo o pacote alinhado com `debugger-index.json`.

### Extensão 0.3.173
- Corrigida a detecção de vídeo patrocinado do Facebook: quando um post é identificado como patrocinado, os botões **Baixar** e **Gravar** ficam ocultos juntos, em vez de apenas um deles.
- Corrigido o travamento de teclas modificadoras (Alt/Shift): o estado de tecla pressionada agora é limpo ao perder o foco, recuperar o foco e ao esconder a aba, evitando falha silenciosa de captura em `claude.ai` e no Rapidgator quando um `keyup` era perdido (por exemplo após Alt+Tab).
- Links de download descartáveis (de uso único) nunca mais são reenviados ao Apocalipse, mesmo com **forçar captura** ativo — eles sempre seguem o caminho assistido pelo navegador, evitando erro 404 por link já consumido.
- Corrigido o filtro de captura de rede do SoundCloud para reconhecer mídia servida também em `sndcdn.com`, além de `soundcloud.com`.

### Qualidade
- Novos testes de regressão cobrindo o piscar do ícone do Apocalipse AI, a legenda do Sobre, a inclusão garantida de `thumbnails.jsonl`, o ocultamento conjunto dos botões do Facebook patrocinado, a limpeza do estado de teclas modificadoras e a captura de mídia do SoundCloud.

---

## English

### Apocalipse AI
- The **Apocalipse AI** icon now blinks when there is new activity (an assistant message) and the panel is not open, and stops blinking as soon as the panel is opened.
- Simplified the **About** page caption, keeping it consistent across all three supported languages.

### Diagnostics
- `logs/by-component/thumbnails.jsonl` is now always included in the exported diagnostic bundle, even when no thumbnail event was recorded, keeping the bundle consistent with `debugger-index.json`.

### Extension 0.3.173
- Fixed Facebook sponsored-video detection: when a post is classified as sponsored, both the **Download** and **Record** buttons are now hidden together instead of only one of them.
- Fixed a "stuck" modifier key (Alt/Shift) bug: the held-key state is now reset on blur, on focus and when the tab is hidden, preventing capture from silently failing to trigger on `claude.ai` and Rapidgator when a `keyup` event was missed (for example after Alt+Tab).
- Disposable (single-use) download links are never resent to Apocalipse again, even with **force capture** enabled — they always take the browser-assisted path, avoiding a 404 from an already-consumed link.
- Fixed the SoundCloud network capture filter to also recognize media served from `sndcdn.com`, in addition to `soundcloud.com`.

### Quality
- Added regression coverage for the Apocalipse AI activity blink, the About caption, guaranteed `thumbnails.jsonl` inclusion, hiding both Facebook sponsored buttons together, held-modifier-key resets and SoundCloud media capture.

---

## 简体中文

### Apocalipse AI
- 当有新的助手活动消息且面板未打开时，**Apocalipse AI** 图标现在会闪烁；打开面板后闪烁立即停止。
- 简化了“关于”页面的说明文字，三种语言保持一致。

### 诊断
- 导出的诊断包现在始终包含 `logs/by-component/thumbnails.jsonl`，即使没有记录任何缩略图事件，使诊断包与 `debugger-index.json` 保持一致。

### 扩展 0.3.173
- 修复了 Facebook 赞助视频检测问题：当帖子被识别为赞助内容时，**下载** 和 **录制** 按钮现在会一起隐藏，而不是只隐藏其中一个。
- 修复了修饰键（Alt/Shift）“卡住”的问题：失去焦点、重新获得焦点以及标签页被隐藏时都会重置按键状态，避免在错过 `keyup` 事件时（例如 Alt+Tab 之后）导致 `claude.ai` 和 Rapidgator 上的捕获静默失败。
- 一次性（不可重复使用）的下载链接不会再次发送给 Apocalipse，即使启用了“强制捕获”，也始终采用浏览器辅助路径，避免因链接已被消耗而出现 404 错误。
- 修复了 SoundCloud 网络捕获过滤器，使其除了 `soundcloud.com` 外，也能识别来自 `sndcdn.com` 的媒体。

### 质量
- 新增回归测试，覆盖 Apocalipse AI 活动闪烁、关于页面说明、`thumbnails.jsonl` 的保证导出、Facebook 赞助内容按钮的联合隐藏、修饰键状态重置以及 SoundCloud 媒体捕获。
