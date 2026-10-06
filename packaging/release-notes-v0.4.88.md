# Apocalipse Download Manager 0.4.88

## Português (Brasil)
- Mídia (yt-dlp): quando um site responde 403 por causa da impressão digital TLS do cliente, o ADM tenta de novo uma vez imitando um navegador (`--impersonate chrome`), se o yt-dlp instalado suportar (os binários oficiais incluem curl_cffi). Sem 403 nada muda.
- Atualização de ferramentas: a verificação de manifesto assinado (ed25519, anti-rollback, validade, SHA-256 e tamanho) agora está ligada a `download_tool`/`update_tool`. Ela só é aplicada depois que o mantenedor cadastrar uma chave pública; enquanto isso o comportamento anterior é mantido. Ver `docs/tool-update-signing.md`.
- Extensões Chrome, Edge e Firefox continuam na 0.3.202 (XPI assinado).

## English
- Media (yt-dlp): when a site answers 403 because of the client's TLS fingerprint, ADM retries once impersonating a browser (`--impersonate chrome`) if the installed yt-dlp supports it (the official binaries bundle curl_cffi). Nothing changes without a 403.
- Tool updates: signed-manifest verification (ed25519, anti-rollback, expiry, SHA-256 and size) is now wired into `download_tool`/`update_tool`. It is enforced only once the maintainer adds a trusted public key; until then the previous behavior is kept. See `docs/tool-update-signing.md`.
- Chrome, Edge and Firefox extensions remain at 0.3.202 (signed XPI).

## 简体中文
- 媒体（yt-dlp）：当网站因客户端 TLS 指纹返回 403 时，ADM 会在已安装的 yt-dlp 支持时（官方二进制包含 curl_cffi）模拟浏览器（`--impersonate chrome`）重试一次。未出现 403 时行为不变。
- 工具更新：已签名清单校验（ed25519、防回滚、有效期、SHA-256 与大小）现已接入 `download_tool`/`update_tool`。仅在维护者添加受信任公钥后才会强制执行；在此之前保持原有行为。详见 `docs/tool-update-signing.md`。
- Chrome、Edge 和 Firefox 扩展保持 0.3.202（已签名 XPI）。
