# Apocalipse Download Manager 0.4.91

## Português (Brasil)
- Download pelo botão Baixar da extensão: quando um CDN responde 403 apenas porque o ADM enviou o Referer da página que incorporou o vídeo (por exemplo `video.twimg.com` em sites que tocam o vídeo sem Referer), o ADM faz uma sondagem de 1 byte e, se o servidor aceita sem o Referer, inicia o download sem ele. Sites que exigem o Referer continuam recebendo.
- Ícone: o app usa o "A" do Apocalipse no lugar do alien, e o ícone da janela/barra de tarefas é pintado com a cor do tema atual (atualiza ao trocar de tema). O ícone da bandeja não mudou.
- Extensões Chrome, Edge e Firefox continuam na 0.3.203 (XPI assinado).

## English
- Downloads from the extension's Download button: when a CDN answers 403 only because ADM sent the Referer of the embedding page (for example `video.twimg.com` on sites that play the video without a Referer), ADM probes once with a 1-byte request and, if the server accepts it without the Referer, starts the download without it. Sites that need the Referer still receive it.
- Icon: the app uses the Apocalipse "A" instead of the alien, and the window/taskbar icon is tinted with the current theme colour (updated when the theme changes). The tray icon is unchanged.
- Chrome, Edge and Firefox extensions stay at 0.3.203 (signed XPI).

## 简体中文
- 通过扩展“下载”按钮下载：当 CDN 仅因 ADM 发送了嵌入页面的 Referer 而返回 403（例如在不带 Referer 播放视频的网站上的 `video.twimg.com`）时，ADM 会先发送 1 字节探测请求，如果服务器在不带 Referer 时接受，则不带 Referer 开始下载。需要 Referer 的网站仍会收到它。
- 图标：应用使用 Apocalipse 的“A”取代外星人图标，窗口/任务栏图标会按当前主题颜色着色（切换主题时更新）。托盘图标未变。
- Chrome、Edge 和 Firefox 扩展保持 0.3.203（已签名 XPI）。
