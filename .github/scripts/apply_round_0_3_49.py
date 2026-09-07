from pathlib import Path
import json

# 0.3.49
# Remove every executable CDP/debugger path from the shipped extension.
# Replace it with a MAIN-world pre-download hook that captures the final URL before
# Chrome owns it, then streams that URL through the existing Apocalipse blob bridge.
# Alt/bypass is respected by the page hook; normal and Shift use Apocalipse.

background = r'''importScripts("service-worker.js");

const APOCALIPSE_BRIDGE = "http://127.0.0.1:17654";
const activeCapturedUrls = new Map();
const CAPTURE_TTL_MS = 20000;

async function pairingToken() {
  const { pairingToken = "" } = await chrome.storage.local.get({ pairingToken: "" });
  if (!pairingToken) throw new Error("not_paired");
  return pairingToken;
}

async function bridgePost(path, body) {
  const token = await pairingToken();
  const response = await fetch(`${APOCALIPSE_BRIDGE}${path}`, {
    method: "POST",
    headers: {
      "Authorization": `Bearer ${token}`,
      "Content-Type": "application/json",
    },
    body: JSON.stringify(body),
  });
  if (!response.ok) throw new Error(`bridge_http_${response.status}`);
  return response.json();
}

const chatgptLibraryUrl = (value) => {
  try {
    const url = new URL(value);
    return url.hostname.toLowerCase() === "chatgpt.com" && url.pathname === "/backend-api/estuary/content" ? url.href : null;
  } catch { return null; }
};

const rapidgatorFinalUrl = (value) => {
  try {
    const url = new URL(value);
    if (!/^s\d+\.rapidgator\.net$/i.test(url.hostname)) return null;
    if (!/^\/download\/[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}\/?$/i.test(url.pathname)) return null;
    return url.href;
  } catch { return null; }
};

const recognizedDownload = (value) => {
  const library = chatgptLibraryUrl(value);
  if (library) return { url: library, kind: "chatgpt-library" };
  const rapidgator = rapidgatorFinalUrl(value);
  if (rapidgator) return { url: rapidgator, kind: "rapidgator" };
  return null;
};

function dispositionFileName(disposition, fallback, hint = "") {
  const safeHint = String(hint || "").split(/[\\/]/).pop();
  if (safeHint) return safeHint;
  const extended = String(disposition || "").match(/filename\*\s*=\s*UTF-8''([^;]+)/i)?.[1];
  if (extended) {
    try { return decodeURIComponent(extended.replace(/^"|"$/g, "")); } catch {}
  }
  const quoted = String(disposition || "").match(/filename\s*=\s*"([^"]+)"/i)?.[1];
  if (quoted) return quoted;
  const plain = String(disposition || "").match(/filename\s*=\s*([^;]+)/i)?.[1]?.trim();
  if (plain) return plain.replace(/^"|"$/g, "");
  try {
    const url = new URL(fallback);
    return url.searchParams.get("filename") || url.searchParams.get("name") || url.pathname.split("/").filter(Boolean).pop() || "download.bin";
  } catch { return "download.bin"; }
}

function hex(bytes) {
  let out = "";
  for (const byte of bytes) out += byte.toString(16).padStart(2, "0");
  return out;
}

async function diagnostic(event, state = {}, extra = {}) {
  const detail = [
    extra.detail || "",
    `extension_version=${chrome.runtime.getManifest().version}`,
    extra.error ? `error=${String(extra.error).slice(0, 1200).replace(/[\r\n]+/g, " ")}` : "",
  ].filter(Boolean).join(" ");
  await bridgePost("/v1/diagnostic", {
    event,
    level: extra.level || (extra.error ? "ERROR" : "INFO"),
    traceId: state.traceId || null,
    source: "chrome-extension",
    url: state.url || state.pageUrl || null,
    status: Number.isFinite(extra.status) ? extra.status : null,
    bytes: Number.isFinite(extra.bytes) ? extra.bytes : (Number.isFinite(state.bytes) ? state.bytes : null),
    durationMs: state.startedAt ? Math.max(0, Date.now() - state.startedAt) : null,
    detail: detail || null,
  }).catch(() => {});
}

function claim(url, source) {
  const now = Date.now();
  const current = activeCapturedUrls.get(url);
  if (current && now - current.at < CAPTURE_TTL_MS) return false;
  activeCapturedUrls.set(url, { at: now, source });
  return true;
}

function release(url) {
  activeCapturedUrls.delete(url);
}

async function waitWhilePaused(uploadId) {
  while (true) {
    const status = await bridgePost("/v1/blob/status", { uploadId });
    if (!status?.paused) return;
    await new Promise((resolve) => setTimeout(resolve, 250));
  }
}

async function streamCapturedUrl(request) {
  const recognized = recognizedDownload(request?.url || "");
  if (!recognized) throw new Error("unsupported_pre_download_url");
  const { url, kind } = recognized;
  if (!claim(url, request.source || "prehook")) return { ok: true, duplicate: true };

  const state = {
    traceId: crypto.randomUUID(),
    url,
    pageUrl: request.pageUrl || null,
    startedAt: Date.now(),
    bytes: 0,
  };
  let uploadId = null;
  try {
    await diagnostic(`${kind}.prehook.accepted`, state, { detail: `source=${request.source || "main-world"}` });
    const response = await fetch(url, {
      method: "GET",
      credentials: "include",
      redirect: "follow",
      cache: "no-store",
      referrer: request.pageUrl || undefined,
    });
    const disposition = response.headers.get("content-disposition") || "";
    const contentType = response.headers.get("content-type") || "";
    const total = Number.parseInt(response.headers.get("content-length") || "0", 10) || 0;
    if (!response.ok) throw new Error(`${kind}_http_${response.status}`);
    const fileName = dispositionFileName(disposition, response.url || url, request.fileName || "");
    await diagnostic(`${kind}.prehook.response`, state, {
      status: response.status,
      detail: `expected_bytes=${total} content_type=${contentType.slice(0, 120)} file=${fileName}`,
    });

    const begin = await bridgePost("/v1/blob/begin", {
      fileName,
      total,
      source: request.pageUrl || url,
      streaming: total === 0,
      promptForDestination: true,
    });
    uploadId = begin?.uploadId || null;
    if (!uploadId) throw new Error("blob_begin_failed");
    if (!response.body) throw new Error("response_stream_unavailable");

    const reader = response.body.getReader();
    let chunks = 0;
    while (true) {
      const { value, done } = await reader.read();
      if (value?.length) {
        for (let offset = 0; offset < value.length; offset += 16 * 1024) {
          const slice = value.subarray(offset, Math.min(value.length, offset + 16 * 1024));
          await waitWhilePaused(uploadId);
          await bridgePost("/v1/blob/chunk", { uploadId, data: hex(slice) });
          state.bytes += slice.length;
          chunks += 1;
        }
      }
      if (done) break;
    }
    await bridgePost("/v1/blob/end", { uploadId });
    await diagnostic(`${kind}.prehook.completed`, state, {
      status: response.status,
      bytes: state.bytes,
      detail: `chunks=${chunks} expected_bytes=${total}`,
    });
    return { ok: true, kind, bytes: state.bytes };
  } catch (error) {
    await diagnostic(`${kind}.prehook.failed`, state, {
      level: "ERROR",
      bytes: state.bytes,
      error: String(error),
      detail: uploadId ? `upload_id=${uploadId}` : "upload_not_started",
    });
    throw error;
  } finally {
    release(url);
  }
}

function bypassIsActive() {
  try {
    if (bypassHeld) return true;
    if (Date.now() < bypassUntil) return true;
    if (Date.now() < bypassNextUntil) {
      bypassNextUntil = 0;
      return true;
    }
  } catch {}
  return false;
}

const cancelBrowserDownload = (id) => new Promise((resolve) => {
  chrome.downloads.cancel(id, () => { void chrome.runtime.lastError; resolve(); });
});
const eraseBrowserDownload = (id) => new Promise((resolve) => {
  chrome.downloads.erase({ id }, () => { void chrome.runtime.lastError; resolve(); });
});

// Fallback only. The preferred path is the MAIN-world prehook, which prevents
// Chrome from creating a download at all. This listener exists only if a site
// changes its JavaScript and bypasses one of the trapped primitives.
chrome.downloads.onCreated.addListener((item) => {
  const recognized = recognizedDownload(item?.finalUrl || item?.url || "");
  if (!recognized || bypassIsActive()) return;
  try { chrome.downloads.cancel(item.id); } catch {}
  try { chrome.downloads.erase({ id: item.id }); } catch {}
  void diagnostic(`${recognized.kind}.prehook.missed`, {
    traceId: crypto.randomUUID(), url: recognized.url, pageUrl: item.referrer || null, startedAt: Date.now(), bytes: 0,
  }, { level: "WARN", detail: `download_id=${item.id}` });
  void cancelBrowserDownload(item.id)
    .then(() => eraseBrowserDownload(item.id))
    .then(() => streamCapturedUrl({
      url: recognized.url,
      pageUrl: item.referrer || null,
      fileName: String(item.filename || "").split(/[\\/]/).pop(),
      source: "chrome.downloads.fallback",
    }))
    .catch(() => {});
});

chrome.runtime.onMessage.addListener((message, sender, reply) => {
  if (message?.type !== "APOCALIPSE_PRE_DOWNLOAD_URL") return;
  if (bypassIsActive()) {
    reply({ ok: false, bypass: true });
    return;
  }
  const request = {
    url: message.url,
    pageUrl: message.pageUrl || sender.tab?.url || null,
    fileName: message.fileName || "",
    source: message.source || "main-world",
  };
  streamCapturedUrl(request)
    .then(reply)
    .catch((error) => {
      // Only on a real takeover failure, fall back to Chrome so the user does not
      // lose a one-use link. Normal successful flow never reaches chrome.downloads.
      const recognized = recognizedDownload(request.url);
      if (recognized) {
        try {
          bypassNextUntil = Date.now() + 15000;
          chrome.downloads.download({ url: recognized.url, saveAs: true }, () => void chrome.runtime.lastError);
        } catch {}
      }
      reply({ ok: false, error: String(error) });
    });
  return true;
});
'''

Path('browser-extension/background.js').write_text(background, encoding='utf-8')

# No CDP arming remains in the Rapidgator-specific isolated script.
Path('browser-extension/rapidgator.js').write_text(r'''(() => {
  if (!/(^|\.)rapidgator\.net$/i.test(location.hostname)) return;
  // Rapidgator takeover is handled before the native download by page-hook.js.
  // This file intentionally contains no chrome.debugger/CDP code.
})();
''', encoding='utf-8')

# Relay MAIN-world captures to the extension and mirror configured shortcut names.
p = Path('browser-extension/content.js')
s = p.read_text(encoding='utf-8')
relay = r'''
  // MAIN-world pre-download relay (no CDP). The page hook traps the final URL
  // before Chrome creates a native download, while this isolated-world script
  // retains access to chrome.runtime and the existing Alt/Shift configuration.
  const postApocalipseShortcutConfig = () => {
    window.postMessage({
      source: "apocalipse-extension",
      type: "shortcut-config",
      bypass: shortcutKeys.bypass,
      force: shortcutKeys.force,
    }, "*");
  };
  postApocalipseShortcutConfig();
  chrome.storage.onChanged.addListener((changes, area) => {
    if (area === "local" && (changes.forceShortcut || changes.bypassShortcut)) {
      setTimeout(postApocalipseShortcutConfig, 0);
    }
  });
  window.addEventListener("message", (event) => {
    if (event.source !== window) return;
    const data = event.data;
    if (!data || data.source !== "apocalipse-page-hook" || data.type !== "pre-download-url") return;
    chrome.runtime.sendMessage({
      type: "APOCALIPSE_PRE_DOWNLOAD_URL",
      url: data.url,
      pageUrl: location.href,
      fileName: data.fileName || "",
      source: data.kind || "main-world",
    }).then((result) => {
      window.postMessage({
        source: "apocalipse-extension",
        type: "pre-download-result",
        requestId: data.requestId,
        result,
      }, "*");
    }).catch((error) => {
      window.postMessage({
        source: "apocalipse-extension",
        type: "pre-download-result",
        requestId: data.requestId,
        result: { ok: false, error: String(error) },
      }, "*");
    });
  });
'''
if 'MAIN-world pre-download relay (no CDP)' not in s:
    pos = s.rfind('})();')
    if pos < 0:
        raise SystemExit('content.js closure not found')
    s = s[:pos] + relay + '\n' + s[pos:]
p.write_text(s, encoding='utf-8')

# MAIN-world hook: captures only recognized one-use/download URLs. Normal click and
# Shift are Apocalipse-owned; configured bypass (Alt by default) leaves the browser untouched.
page_hook = r'''(() => {
  if (window.__apocalipsePreDownloadHook) return;
  window.__apocalipsePreDownloadHook = true;

  let shortcuts = { bypass: "Alt", force: "Shift" };
  const held = new Set();
  let activeLibraryFileName = "";

  const canonicalKey = (event) => {
    if (event.altKey) held.add("Alt"); else held.delete("Alt");
    if (event.shiftKey) held.add("Shift"); else held.delete("Shift");
    if (event.ctrlKey) held.add("Control"); else held.delete("Control");
  };
  addEventListener("keydown", canonicalKey, true);
  addEventListener("keyup", canonicalKey, true);
  addEventListener("blur", () => held.clear(), true);

  addEventListener("message", (event) => {
    if (event.source !== window || event.data?.source !== "apocalipse-extension") return;
    if (event.data.type === "shortcut-config") {
      shortcuts = {
        bypass: event.data.bypass || "Alt",
        force: event.data.force || "Shift",
      };
    }
  });

  const bypassPressed = () => held.has(shortcuts.bypass || "Alt");
  const classify = (value) => {
    try {
      const url = new URL(String(value || ""), location.href);
      if (url.hostname.toLowerCase() === "chatgpt.com" && url.pathname === "/backend-api/estuary/content") {
        return { url: url.href, kind: "chatgpt-library" };
      }
      if (/^s\d+\.rapidgator\.net$/i.test(url.hostname)
          && /^\/download\/[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}\/?$/i.test(url.pathname)) {
        return { url: url.href, kind: "rapidgator" };
      }
    } catch {}
    return null;
  };

  const emit = (candidate, primitive) => {
    if (!candidate || bypassPressed()) return false;
    const requestId = `${Date.now()}-${Math.random().toString(36).slice(2)}`;
    window.postMessage({
      source: "apocalipse-page-hook",
      type: "pre-download-url",
      requestId,
      url: candidate.url,
      kind: candidate.kind,
      primitive,
      fileName: candidate.kind === "chatgpt-library" ? activeLibraryFileName : "",
    }, "*");
    return true;
  };

  // Remember which Library row opened the Radix menu. The menu itself is portaled
  // under <body>, so this association must be captured before the menu item is clicked.
  document.addEventListener("pointerdown", (event) => {
    const button = event.target?.closest?.('button[data-testid^="file-row-actions-"]');
    if (!button || location.hostname !== "chatgpt.com") return;
    const label = button.getAttribute("aria-label") || "";
    activeLibraryFileName = label
      .replace(/^.*?(?:ações de|acoes de|actions for|actions of)\s+/i, "")
      .trim();
  }, true);

  const originalAnchorClick = HTMLAnchorElement.prototype.click;
  HTMLAnchorElement.prototype.click = function(...args) {
    const candidate = classify(this.href);
    if (emit(candidate, "anchor.click")) return;
    return originalAnchorClick.apply(this, args);
  };

  const originalOpen = window.open;
  window.open = function(url, ...args) {
    const candidate = classify(url);
    if (emit(candidate, "window.open")) return null;
    return originalOpen.call(this, url, ...args);
  };

  const originalFetch = window.fetch;
  window.fetch = function(input, init) {
    const value = typeof input === "string" || input instanceof URL ? String(input) : input?.url;
    const candidate = classify(value);
    if (emit(candidate, "window.fetch")) {
      return Promise.resolve(new Response("", { status: 204, statusText: "Handled by Apocalipse" }));
    }
    return originalFetch.call(this, input, init);
  };

  // Catch ordinary anchors after page/React handlers had a chance to update href,
  // but before the browser performs the default navigation/download action.
  document.addEventListener("click", (event) => {
    const anchor = event.target?.closest?.("a[href]");
    const candidate = classify(anchor?.href);
    if (!candidate || bypassPressed()) return;
    event.preventDefault();
    event.stopImmediatePropagation();
    emit(candidate, "document.click");
  }, false);

  // Some pages call Location.assign/replace instead of clicking an anchor.
  for (const method of ["assign", "replace"]) {
    try {
      const original = Location.prototype[method];
      Location.prototype[method] = function(url) {
        const candidate = classify(url);
        if (emit(candidate, `location.${method}`)) return;
        return original.call(this, url);
      };
    } catch {}
  }
})();
'''
Path('browser-extension/page-hook.js').write_text(page_hook, encoding='utf-8')

# Manifest: no debugger permission, no Rapidgator CDP content script; add MAIN-world hook.
p = Path('browser-extension/manifest.json')
manifest = json.loads(p.read_text(encoding='utf-8'))
manifest['version'] = '0.3.49'
manifest['permissions'] = [x for x in manifest.get('permissions', []) if x != 'debugger']
content_scripts = []
for entry in manifest.get('content_scripts', []):
    js = entry.get('js', [])
    if 'rapidgator.js' in js:
        continue
    content_scripts.append(entry)
content_scripts.insert(0, {
    'matches': [
        'https://chatgpt.com/*',
        'https://rapidgator.net/*',
        'https://*.rapidgator.net/*',
    ],
    'js': ['page-hook.js'],
    'run_at': 'document_start',
    'all_frames': True,
    'world': 'MAIN',
})
manifest['content_scripts'] = content_scripts
p.write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')

# Hard validation: shipped executable extension must contain no CDP/debugger API use.
for path in [Path('browser-extension/background.js'), Path('browser-extension/rapidgator.js'), Path('browser-extension/content.js'), Path('browser-extension/page-hook.js')]:
    text = path.read_text(encoding='utf-8')
    forbidden = ['chrome.debugger', 'Fetch.enable', 'Fetch.requestPaused', 'Fetch.takeResponseBodyAsStream', 'IO.read', 'Page.setDownloadBehavior', 'Browser.setDownloadBehavior']
    hit = [token for token in forbidden if token in text]
    if hit:
        raise SystemExit(f'CDP token still present in {path}: {hit}')
if 'debugger' in manifest.get('permissions', []):
    raise SystemExit('debugger permission still present')
