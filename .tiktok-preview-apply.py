from pathlib import Path
import hashlib
import sys

def read(path, expected=None):
    data = Path(path).read_bytes()
    if expected:
        actual = hashlib.sha1(f'blob {len(data)}\0'.encode() + data).hexdigest()
        assert actual == expected, f'Unexpected base for {path}: {actual}'
    return data.decode('utf-8')
def replace_one(text, old, new):
    assert text.count(old) == 1, f'Expected exactly one occurrence: {old[:100]}'
    return text.replace(old, new, 1)
def write(path, text):
    Path(path).write_text(text, encoding='utf-8')

p = 'apps/desktop/src-tauri/src/main.rs'
s = read(p, '86d397ece10de1a887bda22fe14ce7e7373c2699')
s = replace_one(s, 'use apocalipse_core::{', 'mod tiktok_preview;\n\nuse apocalipse_core::{')
s = replace_one(s, 'struct MediaPreviewRequest {\n    url: String,\n    user_agent: Option<String>,\n    referer: Option<String>,\n}', 'struct MediaPreviewRequest {\n    url: String,\n    user_agent: Option<String>,\n    referer: Option<String>,\n    cookie_header: Option<String>,\n    content_type: Option<String>,\n}')
s = replace_one(s, 'fn open_media_preview(state: &AppState, request: MediaPreviewRequest) -> Result<(), String> {', 'fn open_media_preview(app: &tauri::AppHandle, state: &AppState, request: MediaPreviewRequest) -> Result<(), String> {\n    if tiktok_preview::is_candidate(&request) {\n        return tiktok_preview::open(app, request);\n    }')
s = replace_one(s, '.and_then(|request| open_media_preview(&state, request))', '.and_then(|request| open_media_preview(app, &state, request))')
a = s.index('    } else if first.starts_with("POST /v1/preview-media ") {')
b = s.index('    } else if first.starts_with("POST /v1/download-status ") {', a)
section = s[a:b]
section = replace_one(section, '            Err(_) => bridge_response(&mut stream, "400 Bad Request", origin, "{\\"ok\\":false}"),', '''            Err(error) => {
                diagnostic_log(&state, "ERROR", "media.preview_rejected", &error);
                bridge_response(&mut stream, "400 Bad Request", origin,
                    &serde_json::json!({"ok": false, "error": error}).to_string());
            }''')
write(p, s[:a] + section + s[b:])

p = 'crates/apocalipse-core/src/download.rs'
s = read(p, '5b2069a660bc87f3b47900fc9de6e54590705652')
a = s.index('        let mut builder = Client::builder()', s.index('    pub fn with_network('))
b = s.index('        let client = builder.build()?;', a)
body = s[a:b]
s = s[:a] + '''        let client = Self::network_client_builder(proxy_url, username, password, dns_servers)?.build()?;
        Ok(Self { client })
    }

    /// Shared proxy/DNS settings with a caller-controlled redirect policy for preview.
    pub fn network_client_builder(
        proxy_url: Option<&str>,
        username: Option<&str>,
        password: Option<&str>,
        dns_servers: &[IpAddr],
    ) -> Result<reqwest::ClientBuilder> {
''' + body + '''        Ok(builder)
    }
''' + s[s.index('\n    pub async fn download(', b):]
write(p, s)

p = 'browser-extension/background.js'
s = read(p, '5d6bf3ce8f8a4b375873a04b6b1ecae60c825b90')
s = replace_one(s, '    if (!response.ok) throw new Error(`bridge_http_${response.status}`);', '''    if (!response.ok) {
      const body = path === "/v1/preview-media" ? await response.json().catch(() => ({})) : {};
      throw new Error(typeof body.error === "string" ? body.error : `bridge_http_${response.status}`);
    }''')
a = s.index('  if (message?.type === "APOCALIPSE_PREVIEW_MEDIA") {')
b = s.index('  if (message?.type === "APOCALIPSE_MEDIA_PICKER_CONTEXT")', a)
s = s[:a] + r'''  if (message?.type === "APOCALIPSE_PREVIEW_MEDIA") {
    // Snapshot the clicked resource before awaiting the exact-URL cookie lookup.
    const url = message.url;
    const referer = message.pageUrl || sender.tab?.url || null;
    const contentType = message.contentType || null;
    const userAgent = message.userAgent || navigator.userAgent;
    const traceId = crypto.randomUUID();
    (async () => {
      let parsed;
      try { parsed = new URL(url); } catch { throw new Error("invalid_preview_url"); }
      if (!/^https?:$/.test(parsed.protocol) || !parsed.hostname || parsed.username || parsed.password
        || typeof url !== "string" || /[\u0000-\u001f\u007f]/.test(url)) throw new Error("invalid_preview_url");
      const host = parsed.hostname.toLowerCase();
      const tiktok = ["tiktok.com", "tiktokcdn.com", "tiktokcdn-us.com", "tiktokcdn-eu.com", "tiktokv.com", "tiktokv.us", "byteoversea.com", "ibytedtos.com", "muscdn.com"]
        .some(domain => host === domain || host.endsWith(`.${domain}`));
      // Never merge page cookies into a different CDN's request.
      const cookies = tiktok ? await chrome.cookies.getAll({ url }).catch(() => []) : [];
      const cookieHeader = cookies.map(cookie => `${cookie.name}=${cookie.value}`).join("; ") || null;
      return bridgeRequest("/v1/preview-media", {
        method: "POST",
        body: JSON.stringify({ url, userAgent, referer, cookieHeader, contentType }),
      });
    })().then(result => {
      void diagnostic("popup.preview_handed_off", { traceId, url, pageUrl: referer, startedAt: Date.now() }, {});
      reply(result);
    }).catch(error => {
      void diagnostic("popup.preview_failed", { traceId, url, pageUrl: referer, startedAt: Date.now() }, { level: "ERROR", error: String(error) });
      reply({ ok: false, error: String(error) });
    });
    return true;
  }
''' + s[b:]
write(p, s)

p = 'browser-extension/popup.js'
s = read(p, 'ca080ac23b368b2546f276c162abc17e449078c2')
s = replace_one(s, '''    previewButton.onclick = () => chrome.runtime.sendMessage({ type: "APOCALIPSE_PREVIEW_MEDIA", url: item.url, pageUrl: activePageUrl }, (result) => {
      if (!result?.ok || chrome.runtime.lastError) showBridgeError(result?.error || chrome.runtime.lastError?.message || "unavailable");
    });''', r'''    previewButton.onclick = () => {
      previewButton.disabled = true;
      chrome.runtime.sendMessage({ type: "APOCALIPSE_PREVIEW_MEDIA", url: item.url, pageUrl: activePageUrl, contentType: item.contentType || null, userAgent: item.userAgent || null }, (result) => {
        previewButton.disabled = false;
        const error = chrome.runtime.lastError?.message || result?.error;
        if (!result?.ok || error) {
          const label = document.querySelector("#bridge-label");
          label.removeAttribute("data-i18n");
          const prefix = locale === "pt_BR" ? "Falha ao abrir a pr\u00e9via" : locale === "zh_CN" ? "\u65e0\u6cd5\u6253\u5f00\u9884\u89c8" : "Could not open preview";
          label.textContent = `${prefix}: ${error || "unavailable"}`;
        }
      });
    };''')
write(p, s)

p = 'apps/desktop/ui/app.js'
s = read(p, '864bf16ca9396bb4e66322b079d1d7b2e64e550e')
needle = 'window.__TAURI__?.event?.listen?.("recording-completed", async (event) => {'
s = replace_one(s, needle, r'''window.__TAURI__?.event?.listen?.("media-preview-error", (event) => {
  const prefix = locale === "pt-BR" ? "Falha na pr\u00e9-visualiza\u00e7\u00e3o. Consulte Logs para os detalhes."
    : locale === "zh-CN" ? "\u9884\u89c8\u5931\u8d25\u3002\u8bf7\u67e5\u770b\u65e5\u5fd7\u3002" : "Preview failed. See Logs for details.";
  window.alert(`${prefix}\n${String(event.payload || "preview_failed")}`);
}).catch(console.error);
''' + needle)
write(p, s)
for p in ['Cargo.toml', 'Cargo.lock', 'apps/desktop/src-tauri/tauri.conf.json']:
    s = read(p); assert '"0.4.24"' in s; s = s.replace('"0.4.24"', '"0.4.25"')
    if p == 'Cargo.toml': s = replace_one(s, '"macros", "process"', '"macros", "net", "process"')
    write(p, s)
p = 'browser-extension/manifest.json'; write(p, replace_one(read(p), '"0.3.95"', '"0.3.96"'))
p = 'packaging/macos/Info.plist'; write(p, replace_one(read(p), '<string>0.4.0</string>', '<string>0.4.25</string>'))
if '--local' not in sys.argv:
    for p in ['.github/workflows/release.yml', '.github/workflows/validate-portable.yml']:
        write(p, replace_one(read(p), 'node --test tests/extension-handoff.test.cjs', 'node --test tests/*.test.cjs'))
