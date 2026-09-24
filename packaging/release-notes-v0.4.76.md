# Apocalipse Download Manager v0.4.76

## Português (Brasil)

### Correções
- Downloads sem nome de arquivo real (ex.: `gopeed.com/api/download?tpl=...`) agora consultam corretamente o cabeçalho `Content-Disposition` do servidor antes de recair para o nome genérico "download". A verificação anterior deixava de consultar o servidor sempre que a extensão enviava o título da aba junto com o nome genérico — o que acontecia em praticamente todo caso real, já que um título de página nunca foi um nome de arquivo utilizável.
- Corrigido o erro 403 ao baixar do SoundCloud. A causa raiz: o aplicativo reutilizava a URL assinada e de curta duração do manifesto HLS capturada pelo navegador, que já estava expirada no momento da tentativa de download. Agora:
  - O SoundCloud é reconhecido como página de mídia, e uma captura de faixa é redirecionada para a página `soundcloud.com/artista/faixa`, permitindo que o mecanismo de extração resolva uma nova URL assinada em vez de reutilizar a antiga.
  - Se apenas a URL assinada estiver disponível e ela já tiver expirado, o download falha imediatamente com uma mensagem clara, em vez de gastar 10 tentativas inúteis contra um manifesto morto.
  - Transmissões HLS somente de áudio (como o `aac_96k` do SoundCloud) não são mais nomeadas com a extensão `.mp4` incorretamente.

### Observação
- Essas correções são gerais, não específicas de um site: qualquer serviço com URLs de download genéricas (`/download`, `/api/download`) ou com links de mídia assinados e de curta duração se beneficia das mesmas mudanças.

---

## English

### Fixes
- Downloads with no real file name (e.g. `gopeed.com/api/download?tpl=...`) now correctly consult the server's `Content-Disposition` header before falling back to the generic name "download". The previous check skipped the server lookup whenever the extension sent the tab's title alongside the generic name — which happened in nearly every real case, since a page title was never a usable file name to begin with.
- Fixed the 403 error when downloading from SoundCloud. Root cause: the app was reusing the browser-captured, short-lived signed HLS manifest URL, which had already expired by the time the download was attempted. Now:
  - SoundCloud is recognized as a media page, and a track capture is redirected to the `soundcloud.com/artist/track` page, letting the extraction engine resolve a fresh signed URL instead of reusing the stale one.
  - If only the signed URL is available and it has already expired, the download fails immediately with a clear message instead of wasting 10 retries against a dead manifest.
  - Audio-only HLS streams (like SoundCloud's `aac_96k`) are no longer mislabeled with a `.mp4` extension.

### Note
- These fixes are general, not site-specific: any service using generic download URLs (`/download`, `/api/download`) or short-lived signed media links benefits from the same changes.

---

## 简体中文

### 修复
- 没有真实文件名的下载（例如 `gopeed.com/api/download?tpl=...`）现在会正确地在回退到通用名称 "download" 之前查询服务器的 `Content-Disposition` 响应头。此前的检查逻辑在扩展同时发送标签页标题和通用文件名时会跳过服务器查询——而这几乎是所有真实场景，因为页面标题本来就不是可用的文件名。
- 修复了从 SoundCloud 下载时出现的 403 错误。根本原因：应用重用了浏览器捕获的、有效期很短的已签名 HLS 清单 URL，而该 URL 在尝试下载时已经过期。现在：
  - SoundCloud 被识别为媒体页面，单曲捕获会被重定向到 `soundcloud.com/艺术家/曲目` 页面，让提取引擎重新解析一个新的签名 URL，而不是重用过期的旧 URL。
  - 如果只有已签名的 URL 可用且已过期，下载会立即失败并给出明确的错误信息，而不是对一个失效的清单浪费 10 次无意义的重试。
  - 纯音频的 HLS 流（例如 SoundCloud 的 `aac_96k`）不再被错误地标记为 `.mp4` 扩展名。

### 说明
- 这些修复是通用的，并非针对特定网站：任何使用通用下载 URL（`/download`、`/api/download`）或短期签名媒体链接的服务都会受益于同样的改动。
