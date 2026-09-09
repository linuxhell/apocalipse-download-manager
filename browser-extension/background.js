// background.js diagnostics receiver and storage for TikTok scans
// Minimal and non-intrusive: stores diagnostic blobs in chrome.storage.local (rotated)

const TIKTOK_DIAG_KEY = 'apocalipse_tiktok_diag';
const TIKTOK_DIAG_MAX = 200;

function rotateAndStoreDiag(entry) {
  try {
    chrome.storage.local.get({ [TIKTOK_DIAG_KEY]: [] }, (result) => {
      const list = Array.isArray(result[TIKTOK_DIAG_KEY]) ? result[TIKTOK_DIAG_KEY] : [];
      list.push(entry);
      while (list.length > TIKTOK_DIAG_MAX) list.shift();
      chrome.storage.local.set({ [TIKTOK_DIAG_KEY]: list });
    });
  } catch (e) { /* ignore */ }
}

chrome.runtime.onMessage.addListener((message, sender, reply) => {
  if (message?.type === 'APOCALIPSE_TIKTOK_SCAN_DIAG' && message.diagnostics) {
    try {
      const entry = { tabId: message.tabId || sender.tab?.id || null, receivedAt: Date.now(), diagnostics: message.diagnostics };
      rotateAndStoreDiag(entry);
      // also forward a lightweight diagnostic to bridge for remote collection if paired
      try {
        diagnostic('tiktok.scan.received', { traceId: crypto.randomUUID(), pageUrl: entry.diagnostics?.pageUrl || null, startedAt: Date.now() }, { detail: `videos=${(entry.diagnostics?.videos||[]).length}` });
      } catch (e) {}
    } catch (e) {}
    try { reply({ ok: true }); } catch (e) {}
    return true;
  }
  if (message?.type === 'APOCALIPSE_TIKTOK_IDENTITY_DIAG' && message.payload) {
    try {
      const payload = { receivedAt: Date.now(), payload: message.payload };
      rotateAndStoreDiag(payload);
      try {
        diagnostic('tiktok.identity.attempt', { traceId: crypto.randomUUID(), pageUrl: payload.payload?.page || null, startedAt: Date.now() }, { detail: `resolved=${String(payload.payload?.resolved).slice(0,200)}` });
      } catch (e) {}
    } catch (e) {}
    try { reply({ ok: true }); } catch (e) {}
    return true;
  }
});
