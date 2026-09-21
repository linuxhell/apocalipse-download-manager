# Apocalipse Download Manager v0.4.72

## Português (Brasil)

### Ferramentas e portabilidade
- Adicionado botão **Baixar** ao lado de **Procurar** para FFmpeg, yt-dlp, QuickJS, N_m3u8DL-RE, aria2, Extrator e Player.
- Os downloads usam a versão compatível mais recente e são instalados dentro da pasta portátil `tools` do Apocalipse Download Manager.
- O caminho baixado é preenchido automaticamente, mas só é gravado definitivamente quando o usuário clica em **Salvar**.
- **Player** e **Extrator** continuam sem botão **Atualizar**, conforme o fluxo definido.
- Download multiplataforma de ferramentas para Windows, Linux e macOS.
- Player portátil baseado em **mpv**.
- Extrator portátil baseado em **7-Zip/7zz**.
- No Linux, o mpv AppImage pode ser executado sem depender do FUSE do sistema.
- Corrigido o status do Player para que abrir **Ferramentas** nunca execute VLC ou outro player configurado só para detectar versão.
- Janela de **Ferramentas** ampliada para evitar campos e botões espremidos ou cortados.
- Interface e novos botões respeitam Português do Brasil, Inglês e Chinês Simplificado.

### Extensão 0.3.169 e interceptação
- Corrigida a interceptação de downloads em abas já abertas após instalar/recarregar a extensão.
- A extensão agora detecta quando o content script/page-hook não está ativo e faz reinjeção automática.
- Verificações automáticas ocorrem ao instalar, iniciar, ativar aba, concluir navegação e durante heartbeat.
- **Insert** passa a funcionar como atalho fixo de força para captura, além do atalho configurável.
- Melhorada a captura de links de arquivos do ChatGPT: clique normal pode ser entregue diretamente ao Apocalipse sem depender de Alt.
- Mantido o fluxo de bypass configurável.
- Adicionados eventos forenses para diagnosticar camada ausente, reinjeção, prontidão do content script, prontidão do MAIN hook e falhas de reparo.
- O heartbeat não registra mais `layer_main_hook_repaired` repetidamente quando nada foi realmente reparado.
- Novo evento `capture.layer_healthy` para indicar camada saudável, com limitação de frequência.
- `layer_main_hook_repaired` agora só é emitido depois de confirmar que o hook respondeu.
- Novo `layer_main_hook_repair_unverified` quando a tentativa de reparo não pode ser confirmada.

### Nomes de mídia
- Corrigida a falha intermitente que podia salvar mídia como `Vídeo.mp4` ou outro nome genérico.
- Nomes genéricos como Vídeo, Video, Audio, Media, Download e File deixam de ter prioridade sobre o título real.
- O popup passa a aproveitar título real do elemento, metadados da página, título da aba e contexto da página.
- No SoundCloud, o nome também pode ser reconstruído a partir do slug da faixa quando necessário.
- O fluxo nativo/assistido do navegador corrige o nome antes do handoff para o desktop.
- Novos eventos `browser_download.filename_resolved` e `browser_download.filename_fallback` informam a origem da decisão sem gravar o título completo no log.

### Qualidade
- Novos testes de regressão para download automático de ferramentas, player configurado, autorrecuperação da extensão, Insert, heartbeat verificado e recuperação de nome de mídia.
- Validação multiplataforma em Windows x64, Linux x64, macOS x64, Debian, Fedora e Arch Linux.

---

## English

### Tools and portability
- Added a **Download** button next to **Browse** for FFmpeg, yt-dlp, QuickJS, N_m3u8DL-RE, aria2, Extractor and Player.
- Downloads use the latest compatible version and are installed inside the portable Apocalipse Download Manager `tools` directory.
- Downloaded paths are filled automatically but are only persisted after the user clicks **Save**.
- **Player** and **Extractor** intentionally remain without an **Update** button.
- Cross-platform tool downloads for Windows, Linux and macOS.
- Portable **mpv** player support.
- Portable **7-Zip/7zz** extractor support.
- On Linux, the mpv AppImage can run without relying on system FUSE.
- Fixed Player status detection so opening **Tools** never launches VLC or another configured player just to detect a version.
- Enlarged the **Tools** window to prevent clipped or compressed controls.
- New controls follow Brazilian Portuguese, English and Simplified Chinese localization.

### Extension 0.3.169 and interception
- Fixed download interception in tabs that were already open when the extension was installed/reloaded.
- The extension now detects a missing content script/page-hook and automatically reinjects the capture layer.
- Automatic checks run on install, startup, tab activation, completed navigation and heartbeat.
- **Insert** is now always available as a force-capture shortcut in addition to the configurable shortcut.
- Improved ChatGPT generated-file interception so a normal click can be handed to Apocalipse without relying on Alt.
- Configurable bypass behavior is preserved.
- Added forensic events for missing layers, reinjection, content readiness, MAIN hook readiness and repair failures.
- Heartbeat no longer repeatedly reports `layer_main_hook_repaired` when no real repair occurred.
- Added throttled `capture.layer_healthy` events for healthy capture layers.
- `layer_main_hook_repaired` is emitted only after the hook responds successfully.
- Added `layer_main_hook_repair_unverified` when a repair attempt cannot be verified.

### Media filenames
- Fixed the intermittent issue that could save media as `Video.mp4` or another generic filename.
- Generic labels such as Video, Audio, Media, Download and File no longer override real titles.
- The popup can use the real element title, page metadata, browser tab title and page context.
- On SoundCloud, the track slug can also be used as a fallback name.
- Native/browser-assisted downloads repair generic filenames before desktop handoff.
- Added `browser_download.filename_resolved` and `browser_download.filename_fallback` diagnostics that record the decision source without logging the full page title.

### Quality
- Added regression coverage for automatic tool downloads, configured-player safety, extension self-healing, Insert, verified heartbeat behavior and media filename recovery.
- Cross-platform validation covers Windows x64, Linux x64, macOS x64, Debian, Fedora and Arch Linux.

---

## 简体中文

### 工具与便携性
- 在 FFmpeg、yt-dlp、QuickJS、N_m3u8DL-RE、aria2、解压工具和播放器的 **浏览** 旁新增 **下载** 按钮。
- 自动下载最新兼容版本，并安装到 Apocalipse Download Manager 便携目录中的 `tools` 文件夹。
- 下载后会自动填写路径，但只有点击 **保存** 后才会永久写入配置。
- **播放器** 和 **解压工具** 按设计继续不显示 **更新** 按钮。
- 支持 Windows、Linux 和 macOS 的跨平台工具下载。
- 支持便携式 **mpv** 播放器。
- 支持便携式 **7-Zip/7zz** 解压工具。
- Linux 下的 mpv AppImage 可在不依赖系统 FUSE 的情况下运行。
- 修复播放器状态检测：打开 **工具** 时不会为了读取版本而启动 VLC 或其他已配置播放器。
- 扩大 **工具** 窗口，避免按钮和路径字段被挤压或截断。
- 新控件支持巴西葡萄牙语、英语和简体中文。

### 扩展 0.3.169 与下载拦截
- 修复扩展安装或重新加载时已经打开的标签页无法稳定拦截下载的问题。
- 扩展会检测 content script/page-hook 是否缺失，并自动重新注入捕获层。
- 安装、启动、激活标签页、导航完成以及 heartbeat 时都会自动检查。
- **Insert** 现在始终可作为强制捕获快捷键，同时保留可配置快捷键。
- 改进 ChatGPT 生成文件的拦截，普通点击即可交给 Apocalipse，不再依赖 Alt。
- 保留可配置的绕过行为。
- 新增捕获层缺失、重新注入、content script 就绪、MAIN hook 就绪和修复失败等诊断事件。
- heartbeat 不再在没有真正修复时重复记录 `layer_main_hook_repaired`。
- 新增限频的 `capture.layer_healthy` 事件表示捕获层健康。
- 只有确认 hook 已成功响应后才记录 `layer_main_hook_repaired`。
- 无法验证修复时记录 `layer_main_hook_repair_unverified`。

### 媒体文件名
- 修复媒体偶尔被保存为 `Video.mp4` / `Vídeo.mp4` 或其他通用文件名的问题。
- Video、Audio、Media、Download、File 等通用标签不再覆盖真实标题。
- 弹出窗口可使用元素真实标题、页面元数据、浏览器标签标题和页面上下文。
- 在 SoundCloud 中，必要时还可从曲目 URL slug 恢复名称。
- 浏览器原生/辅助下载会在交给桌面程序之前修正通用文件名。
- 新增 `browser_download.filename_resolved` 和 `browser_download.filename_fallback` 诊断事件，记录名称来源但不记录完整页面标题。

### 质量
- 新增自动工具下载、已配置播放器安全性、扩展自恢复、Insert、已验证 heartbeat 和媒体文件名恢复的回归测试。
- 跨平台验证覆盖 Windows x64、Linux x64、macOS x64、Debian、Fedora 和 Arch Linux。
