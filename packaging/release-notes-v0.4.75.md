# Apocalipse Download Manager v0.4.75

## Português (Brasil)

### Correções
- Ao remover um torrent pausado ou com falha e escolher apagar do disco, o aplicativo agora também pergunta se deseja apagar o arquivo `.torrent` original salvo em `data/torrents`. Antes, essa pergunta só aparecia para torrents já concluídos, então pausar um torrent antes de removê-lo deixava o arquivo `.torrent` para trás indefinidamente.
- A extensão do navegador (popup) agora acompanha corretamente o tema visual escolhido no aplicativo. A lista de temas reconhecidos pelo popup estava desatualizada (uma paleta antiga de 26 temas), então qualquer tema atual (como "Céu") era descartado e o popup sempre aparecia com as cores padrão, mesmo com o app e a extensão conectados e sincronizados corretamente.

### Extensão do navegador
- Versão da extensão atualizada para 0.3.176, com paletas de cor completas para os 21 temas atuais do aplicativo em Chrome, Edge e Firefox.
- Pacote do Firefox assinado pela Mozilla/AMO.

### Documentação
- README e roteiro do projeto atualizados: removida menção a compilações ARM64 (não planejadas), e confirmado que o Linux já recebe tanto `.tar.gz` portátil quanto AppImage.

---

## English

### Fixes
- Removing a paused or failed torrent and choosing to delete it from disk now also asks whether to delete the original `.torrent` file saved in `data/torrents`. Previously this prompt only fired for already-completed torrents, so pausing a torrent before removing it left the `.torrent` file behind indefinitely.
- The browser extension popup now correctly follows the app's selected theme. The popup's recognized theme list was stale (an old 26-theme palette), so any current theme (like "Sky") was silently discarded and the popup always rendered with default colors, even while properly connected and synced with the app.

### Browser extension
- Extension version bumped to 0.3.176, with full color palettes for the app's current 21 themes across Chrome, Edge and Firefox.
- Firefox package signed by Mozilla/AMO.

### Documentation
- Updated README and roadmap: removed the ARM64 build mention (not planned), and confirmed Linux already ships both a portable `.tar.gz` and an AppImage.

---

## 简体中文

### 修复
- 移除一个已暂停或失败的种子任务并选择从磁盘删除时，现在也会询问是否同时删除保存在 `data/torrents` 中的原始 `.torrent` 文件。此前该提示只在种子任务已完成时才会出现，因此在删除前暂停种子会导致 `.torrent` 文件被无限期遗留。
- 浏览器扩展弹出窗口现在能正确跟随应用中选择的主题。此前弹出窗口识别的主题列表已过时（旧的 26 种主题调色板），因此任何当前主题（如"天空"）都会被静默忽略，弹出窗口始终显示默认配色，即使应用与扩展已正确连接并同步。

### 浏览器扩展
- 扩展版本升级至 0.3.176，为 Chrome、Edge 和 Firefox 提供了应用当前 21 种主题的完整配色方案。
- Firefox 安装包已通过 Mozilla/AMO 审核并签名。

### 文档
- 更新了 README 和路线图：移除了 ARM64 构建的相关内容（暂无计划），并确认 Linux 已同时提供便携式 `.tar.gz` 和 AppImage 两种格式。
