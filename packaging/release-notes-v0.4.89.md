# Apocalipse Download Manager 0.4.89

## Português (Brasil)
- Conexão com a extensão: a ponte local agora atende cada conexão em uma thread própria (limite de 32). Antes, uma requisição lenta ou uma rajada de diagnósticos atrasava o heartbeat até o navegador mostrar "desconectado".
- A extensão deixou de registrar a cada varredura cada vídeo de prévia ignorado (centenas de requisições por minuto em páginas com vários vídeos).
- Idioma: a interface restaura o idioma salvo no `settings.json` (como já era feito com o tema), evitando que o app, o popup e os botões da extensão mostrem idiomas diferentes depois de um reinício.
- ID da extensão fixo no Chrome e no Edge (`lfgkfogkggkgacahaidkbhggdolpojjf`): reinstalar a extensão a partir de outra pasta não troca mais o ID. O token de pareamento ainda é guardado por instalação; ao remover a extensão é preciso colar o token de novo.
- Extensões Chrome e Edge 0.3.203. O Firefox continua com o XPI assinado 0.3.202 até haver um XPI assinado da 0.3.203.

## English
- Extension connection: the local bridge now serves each connection on its own thread (capped at 32). Previously a slow request or a burst of diagnostics delayed the heartbeat until the browser showed "disconnected".
- The extension no longer reports every skipped preview video on every scan (hundreds of requests per minute on multi-video pages).
- Language: the UI restores the language saved in `settings.json` (as already done for the theme), so the app, popup and extension buttons cannot show different languages after a restart.
- Fixed extension ID on Chrome and Edge (`lfgkfogkggkgacahaidkbhggdolpojjf`): reinstalling the extension from another folder no longer changes the ID. The pairing token is still stored per install; removing the extension requires pasting the token again.
- Chrome and Edge extensions 0.3.203. Firefox keeps the signed 0.3.202 XPI until a signed 0.3.203 XPI is available.

## 简体中文
- 扩展连接：本地桥现在为每个连接使用独立线程（上限 32）。此前一个缓慢请求或一批诊断请求会延迟心跳，导致浏览器显示“未连接”。
- 扩展不再在每次扫描时为每个被跳过的预览视频上报（多视频页面每分钟数百次请求）。
- 语言：界面从 `settings.json` 恢复已保存的语言（与主题一致），避免重启后应用、弹窗和扩展按钮显示不同语言。
- Chrome 和 Edge 的扩展 ID 固定为 `lfgkfogkggkgacahaidkbhggdolpojjf`：从其他文件夹重新安装不再改变 ID。配对令牌仍按安装保存；移除扩展后需重新粘贴令牌。
- Chrome 和 Edge 扩展 0.3.203。Firefox 继续使用已签名的 0.3.202 XPI，直到有 0.3.203 的已签名 XPI。
