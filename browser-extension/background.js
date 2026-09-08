const BRIDGE = "http://127.0.0.1:17654";
const HEARTBEAT_ALARM = "apocalipse-bridge-heartbeat";
let bridgeConnected = false;
let bypassHeld = false;
let bypassUntil = 0;
let bypassNextUntil = 0;
let forceHeld = false;
let lastFormSubmission = null;
let siteRules = [{ id: "uupdump", hosts: ["uupdump.net", "*.uupdump.net"], action: "uupdump_post", enabled: true }];
const recentFileResponses = [];
const ASSISTED_PREFIX = "assisted-download:";
const DIRECT_PREFIX = "direct-download:";

function responseHeader(headers, name) {
  return (headers || []).find((header) => header.name?.toLowerCase() === name)?.value || "";
}

if (chrome.webRequest?.onResponseStarted) {
  chrome.webRequest.onResponseStarted.addListener((details) => {
    if (details.tabId < 0 || !/^https?:/i.test(details.url)) return;
    const contentType = responseHeader(details.responseHeaders, "content-type").toLowerCase();
    const disposition = responseHeader(details.responseHeaders, "content-disposition").toLowerCase();
    const looksLikeFile = disposition.includes("attachment")
      || (!contentType.includes("text/html") && /(?:application\/(?:octet-stream|x-rar|zip)|binary)/i.test(contentType));
    if (!looksLikeFile) return;
    recentFileResponses.push({ url: details.url, disposition, capturedAt: Date.now() });
    recentFileResponses.splice(0, Math.max(0, recentFileResponses.length - 50));
  }, { urls: ["http://*/*", "https://*/*"] }, ["responseHeaders"]);
}

async function bridgeRequest(path, options = {}, suppliedToken = null) {
  const { pairingToken = "" } = suppliedToken === null ? await chrome.storage.local.get({ pairingToken: "" }) : { pairingToken: suppliedToken };
  if (!pairingToken) {
    bridgeConnected = false;
    throw new Error("not_paired");
  }
  try {
    const response = await fetch(`${BRIDGE}${path}`, {
      ...options,
      headers: {
        "Authorization": `Bearer ${pairingToken}`,
        "Content-Type": "application/json",
        ...(options.headers || {}),
      },
    });
    if (!response.ok) throw new Error(`bridge_http_${response.status}`);
    bridgeConnected = true;
    return response.json();
  } catch (error) {
    bridgeConnected = false;
    throw error;
  }
}

function ensureHeartbeat() {
  chrome.alarms.create(HEARTBEAT_ALARM, { delayInMinutes: 0.1, periodInMinutes: 0.5 });
}

ensureHeartbeat();
chrome.runtime.onInstalled.addListener(ensureHeartbeat);
chrome.runtime.onStartup.addListener(ensureHeartbeat);
chrome.alarms.onAlarm.addListener((alarm) => {
  if (alarm.name === HEARTBEAT_ALARM) {
    bridgeRequest("/v1/health")
      .then(() => flushAssistedDownloads())
      .then(() => flushDirectDownloads())
      .catch(() => {});
  }
});

async function hlsDuration(url, depth = 0) {
  if (depth > 1) return { duration: null, requestUrls: [url] };
  const response = await fetch(url, { credentials: "include", redirect: "follow" });
  if (!response.ok) return { duration: null, requestUrls: [url] };
  const text = await response.text();
  const lines = text.split(/\r?\n/).map((line) => line.trim()).filter(Boolean);
  const variant = lines.findIndex((line) => line.startsWith("#EXT-X-STREAM-INF"));
  if (variant >= 0) {
    const child = lines.slice(variant + 1).find((line) => !line.startsWith("#"));
    if (!child) return { duration: null, requestUrls: [url] };
    const result = await hlsDuration(new URL(child, url).href, depth + 1);
    return { duration: result.duration, requestUrls: [url, ...result.requestUrls] };
  }
  const durations = lines.filter((line) => line.startsWith("#EXTINF:"))
    .map((line) => Number.parseFloat(line.slice(8))).filter(Number.isFinite);
  return { duration: durations.length ? durations.reduce((total, value) => total + value, 0) : null, requestUrls: [url] };
}

async function analyzeHls(urls, expectedDuration) {
  const items = await Promise.all((urls || []).slice(-20).map(async (url) => ({ url, ...(await hlsDuration(url).catch(() => ({ duration: null, requestUrls: [url] }))) })));
  const expected = Number(expectedDuration);
  const valid = items.filter((item) => Number.isFinite(item.duration));
  if (Number.isFinite(expected) && expected > 0) valid.sort((a, b) => Math.abs(a.duration - expected) - Math.abs(b.duration - expected));
  else valid.sort((a, b) => b.duration - a.duration);
  const recommendedUrl = valid[0]?.url || items.at(-1)?.url || null;
  return items.map((item) => ({ ...item, recommended: item.url === recommendedUrl }));
}

async function cookieHeaderFor(urls) {
  const cookies = await Promise.all([...new Set((urls || []).filter((url) => /^https?:/i.test(url)))]
    .map((url) => chrome.cookies.getAll({ url }).catch(() => [])));
  const values = new Map();
  for (const cookie of cookies.flat()) values.set(cookie.name, cookie.value);
  return [...values].map(([name, value]) => `${name}=${value}`).join("; ");
}

async function sourcePageUrl(sender) {
  if (sender.tab?.url) return sender.tab.url;
  const [tab] = await chrome.tabs.query({ active: true, currentWindow: true });
  return tab?.url || null;
}

const fileNameFromPath = (path) => String(path || "").split(/[\\/]/).pop() || null;

async function handOffTelegramBlob(item) {
  const tabs = await chrome.tabs.query({ url: ["https://web.telegram.org/*"] }).catch(() => []);
  for (const tab of tabs) {
    if (!tab.id) continue;
    try {
      const result = await chrome.tabs.sendMessage(tab.id, {
        type: "APOCALIPSE_UPLOAD_BLOB",
        url: item.finalUrl || item.url,
        fileName: fileNameFromPath(item.filename) || "telegram-download",
      });
      if (result?.started) return true;
    } catch {}
  }
  return false;
}

function uupDumpPost(url) {
  try {
    const parsed = new URL(url);
    const host = parsed.hostname.toLowerCase();
    const rule = siteRules.find((item) => item.enabled && item.action === "uupdump_post"
      && item.hosts?.some((pattern) => pattern.startsWith("*.")
        ? host === pattern.slice(2) || host.endsWith(`.${pattern.slice(2)}`)
        : host === pattern));
    if (!rule) return null;
    if (!["/download.php", "/get.php"].includes(parsed.pathname.toLowerCase())) return null;
    return {
      url: `https://uupdump.net/get.php${parsed.search}`,
      pageUrl: `https://uupdump.net/download.php${parsed.search}`,
      method: "POST",
      body: "autodl=2&updates=1",
      contentType: "application/x-www-form-urlencoded",
    };
  } catch {
    return null;
  }
}

function matchingRule(url, action) {
  try {
    const host = new URL(url).hostname.toLowerCase();
    return siteRules.find((rule) => rule.enabled && rule.action === action
      && rule.hosts?.some((pattern) => pattern.startsWith("*.")
        ? host === pattern.slice(2) || host.endsWith(`.${pattern.slice(2)}`)
        : host === pattern));
  } catch {
    return null;
  }
}

const assistedKey = (id) => `${ASSISTED_PREFIX}${id}`;

async function markAssistedDownload(item, url) {
  await chrome.storage.session.set({
    [assistedKey(item.id)]: { url, fileName: item.filename || null },
  });
}

async function completeAssistedDownload(id) {
  const key = assistedKey(id);
  const stored = await chrome.storage.session.get(key);
  const pending = stored[key];
  if (!pending) return;
  const [item] = await chrome.downloads.search({ id });
  if (!item || item.state !== "complete" || !item.filename) return;
  await bridgeRequest("/v1/browser-download-complete", {
    method: "POST",
    body: JSON.stringify({
      url: item.finalUrl || pending.url || item.url,
      fileName: item.filename,
      total: Number(item.fileSize || item.totalBytes) || null,
    }),
  });
  await chrome.storage.session.remove(key);
}

async function flushAssistedDownloads() {
  const values = await chrome.storage.session.get(null);
  await Promise.all(Object.keys(values)
    .filter((key) => key.startsWith(ASSISTED_PREFIX))
    .map((key) => completeAssistedDownload(Number(key.slice(ASSISTED_PREFIX.length))).catch(() => {})));
}

async function checkDirectDownload(taskId) {
  const key = `${DIRECT_PREFIX}${taskId}`;
  const stored = await chrome.storage.session.get(key);
  const pending = stored[key];
  if (!pending) return;
  const result = await bridgeRequest("/v1/download-status", {
    method: "POST",
    body: JSON.stringify({ taskId }),
  });
  if (result.status === "active") return;
  await chrome.storage.session.remove(key);
  if (result.status !== "failed") return;
  bypassNextUntil = Date.now() + 30000;
  chrome.downloads.download({ url: pending.url, saveAs: false }, () => void chrome.runtime.lastError);
}

async function flushDirectDownloads() {
  const values = await chrome.storage.session.get(null);
  await Promise.all(Object.keys(values)
    .filter((key) => key.startsWith(DIRECT_PREFIX))
    .map((key) => checkDirectDownload(key.slice(DIRECT_PREFIX.length)).catch(() => {})));
}

const cancelBrowserDownload = (id) => new Promise((resolve, reject) => {
  chrome.downloads.cancel(id, () => {
    const error = chrome.runtime.lastError;
    if (error) reject(new Error(error.message));
    else resolve();
  });
});

const eraseBrowserDownload = (id) => new Promise((resolve) => {
  chrome.downloads.erase({ id }, () => {
    void chrome.runtime.lastError;
    resolve();
  });
});

function isChatGPTLibraryDownload(value) {
  try {
    const url = new URL(value);
    return url.hostname.toLowerCase() === "chatgpt.com" && url.pathname === "/backend-api/estuary/content";
  } catch {
    return false;
  }
}

function isRapidgatorFinalDownload(value) {
  try {
    const url = new URL(value);
    return /^s\d+\.rapidgator\.net$/i.test(url.hostname)
      && /^\/download\/[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}\/?$/i.test(url.pathname);
  } catch {
    return false;
  }
}

async function takeBrowserDownload(item, eraseFromHistory = false) {
  let url = item.finalUrl || item.url;
  if (!item.id) return false;
  const modifierTabId = Number.isInteger(item.tabId) ? item.tabId : null;
  if (bypassIsActive(modifierTabId)) return false;
  if (/^blob:https:\/\/web\.telegram\.org\//i.test(url)) {
    if (bypassIsActive(modifierTabId)) return false;
    if (!bridgeConnected) return false;
    if (await handOffTelegramBlob(item)) {
      await cancelBrowserDownload(item.id).catch(() => {});
      if (eraseFromHistory) await eraseBrowserDownload(item.id);
      return true;
    }
    return false;
  }
  if (!/^https?:/i.test(url)) return false;
  const pageUrl = item.referrer || null;
  const now = Date.now();
  const expectedName = fileNameFromPath(item.filename)?.toLowerCase() || "";
  const candidates = recentFileResponses.filter((response) => now - response.capturedAt < 10000);
  const recentResponse = candidates.findLast((response) => {
    let decodedUrl = response.url.toLowerCase();
    try { decodedUrl = decodeURIComponent(decodedUrl); } catch {}
    return expectedName && (response.disposition.includes(expectedName) || decodedUrl.includes(expectedName));
  }) || (candidates.length === 1 ? candidates[0] : null);
  if (recentResponse && Date.now() - recentResponse.capturedAt < 10000) {
    let landingOrigin = "";
    let resolvedOrigin = "";
    try {
      landingOrigin = new URL(url).origin;
      resolvedOrigin = new URL(recentResponse.url).origin;
    } catch {}
    if (recentResponse.url !== url && resolvedOrigin && resolvedOrigin !== landingOrigin) {
      url = recentResponse.url;
    }
    const index = recentFileResponses.indexOf(recentResponse);
    if (index >= 0) recentFileResponses.splice(index, 1);
  }
  const pendingPost = lastFormSubmission && Date.now() - lastFormSubmission.capturedAt < 30000
    ? lastFormSubmission
    : null;
  let formRequest = null;
  if (pendingPost) {
    let sameRequest = pageUrl === pendingPost.pageUrl;
    try {
      sameRequest ||= new URL(url).origin === new URL(pendingPost.url).origin;
    } catch {}
    if (sameRequest) {
      lastFormSubmission = null;
      const uupRequest = uupDumpPost(url);
      if (!uupRequest) return false;
      url = uupRequest.url;
      formRequest = uupRequest;
    }
  }
  if (bypassIsActive(modifierTabId)) return false;
  if (!bridgeConnected) return false;
  const immediateTakeover = Boolean(matchingRule(url, "browser_assisted"));
  let cancelled = false;
  try {
    await cancelBrowserDownload(item.id);
    cancelled = true;
    if (eraseFromHistory || immediateTakeover) await eraseBrowserDownload(item.id);
    formRequest ||= lastFormSubmission
      && Date.now() - lastFormSubmission.capturedAt < 30000
      && pageUrl === lastFormSubmission.pageUrl
      ? lastFormSubmission
      : null;
    if (formRequest) lastFormSubmission = null;
    const handoff = await bridgeRequest("/v1/download", {
      method: "POST",
      body: JSON.stringify({
        url,
        fileName: fileNameFromPath(item.filename),
        pageUrl,
        duration: null,
        cookieHeader: await cookieHeaderFor([url, item.url, pageUrl]),
        userAgent: navigator.userAgent,
        requestMethod: formRequest?.method || "GET",
        requestBody: formRequest?.body || null,
        requestContentType: formRequest?.contentType || null,
        startImmediately: immediateTakeover,
      }),
    }).catch(async (error) => {
      if (error.message.includes("bridge_http_404") || error.message.includes("not found")) {
        bypassNextUntil = Date.now() + 30000;
        chrome.downloads.download({ url, saveAs: false }, () => void chrome.runtime.lastError);
        return true;
      }
      throw error;
    });
    if (immediateTakeover && handoff.taskId) {
      await chrome.storage.session.set({
        [`${DIRECT_PREFIX}${handoff.taskId}`]: { url },
      });
      setTimeout(() => void checkDirectDownload(handoff.taskId).catch(() => {}), 1500);
    }
    return true;
  } catch {
    if (cancelled) {
      bypassUntil = Date.now() + 2000;
      chrome.downloads.download({ url, saveAs: false }, () => void chrome.runtime.lastError);
      return true;
    }
    return false;
  }
}

if (chrome.downloads.onDeterminingFilename?.addListener) {
  chrome.downloads.onDeterminingFilename.addListener((item, suggest) => {
    void takeBrowserDownload(item).then((intercepted) => {
      if (!intercepted) suggest();
    }).catch(() => suggest());
    return true;
  });
} else {
  chrome.downloads.onCreated.addListener((item) => {
    void takeBrowserDownload(item, true);
  });
}

chrome.downloads.onChanged.addListener((delta) => {
  if (delta.state?.current === "complete") {
    void completeAssistedDownload(delta.id).catch(() => {});
  }
});

chrome.runtime.onMessage.addListener((message, sender, reply) => {
  if (message?.type === "APOCALIPSE_WORKER_PING") {
    reply({ ok: true, version: chrome.runtime.getManifest().version });
    return;
  }
  if (message?.type === "APOCALIPSE_PAIR") {
    const token = String(message.token || "").trim();
    if (!token) {
      bridgeConnected = false;
      reply({ connected: false, error: "not_paired" });
      return;
    }
    bridgeRequest("/v1/health", {}, token)
      .then(async () => {
        await chrome.storage.local.set({ pairingToken: token });
        bridgeConnected = true;
        ensureHeartbeat();
        reply({ connected: true });
      })
      .catch((error) => {
        bridgeConnected = false;
        reply({ connected: false, error: String(error) });
      });
    return true;
  }
  if (message?.type === "APOCALIPSE_BRIDGE_STATUS") {
    bridgeRequest("/v1/health")
      .then(() => reply({ connected: true }))
      .catch((error) => reply({ connected: false, error: String(error) }));
    return true;
  }
  if (message?.type === "APOCALIPSE_BLOB_BEGIN") {
    bridgeRequest("/v1/blob/begin", { method: "POST", body: JSON.stringify(message.request) }).then(reply)
      .catch((error) => reply({ error: String(error) }));
    return true;
  }
  if (message?.type === "APOCALIPSE_BLOB_CHUNK") {
    bridgeRequest("/v1/blob/chunk", { method: "POST", body: JSON.stringify(message.request) }).then(reply)
      .catch((error) => reply({ error: String(error) }));
    return true;
  }
  if (message?.type === "APOCALIPSE_BLOB_STATUS") {
    bridgeRequest("/v1/blob/status", { method: "POST", body: JSON.stringify(message.request) }).then(reply)
      .catch((error) => reply({ error: String(error) }));
    return true;
  }
  if (message?.type === "APOCALIPSE_BLOB_END") {
    bridgeRequest("/v1/blob/end", { method: "POST", body: JSON.stringify(message.request) }).then(reply)
      .catch((error) => reply({ error: String(error) }));
    return true;
  }
  if (message?.type === "APOCALIPSE_GET_PAGE") {
    sourcePageUrl(sender).then((pageUrl) => reply({ pageUrl })).catch(() => reply({ pageUrl: null }));
    return true;
  }
  if (message?.type === "APOCALIPSE_HLS_ANALYZE") {
    analyzeHls(message.urls, message.duration).then(reply).catch((error) => reply({ error: String(error) }));
    return true;
  }
});

const APOCALIPSE_WORKER_BUILD = "0.3.63-self-contained";
chrome.runtime.onMessage.addListener((message, sender, reply) => {
  if (message?.type !== "APOCALIPSE_WORKER_DIAGNOSTICS") return;
  reply({
    ok: true,
    version: chrome.runtime.getManifest().version,
    build: APOCALIPSE_WORKER_BUILD,
    trace: [{ phase: "background.self_contained.ready", at: new Date().toISOString() }],
    error: null,
  });
});


const APOCALIPSE_BRIDGE = "http://127.0.0.1:17654";
const activeCapturedUrls = new Map();
const CAPTURE_TTL_MS = 20000;

// Central modifier transaction state. Bypass always wins. Force survives the
// initiating click long enough for async pages/CDNs to create the real download.
let forceUntil = 0;
let forceTabId = null;
let bypassTabId = null;

function armBypass(tabId, ttlMs = 4000) {
  bypassUntil = Math.max(bypassUntil || 0, Date.now() + Math.max(500, Math.min(Number(ttlMs) || 4000, 30000)));
  bypassTabId = Number.isInteger(tabId) ? tabId : null;
  forceUntil = 0;
  forceTabId = null;
}
function armForce(tabId, ttlMs = 20000) {
  if (bypassHeld || Date.now() < (bypassUntil || 0)) return;
  forceUntil = Math.max(forceUntil, Date.now() + Math.max(1000, Math.min(Number(ttlMs) || 20000, 30000)));
  forceTabId = Number.isInteger(tabId) ? tabId : null;
}
function sameLeaseTab(leaseTabId, tabId) {
  return leaseTabId == null || tabId == null || leaseTabId === tabId;
}
function bypassIsActive(tabId = null) {
  try {
    const now = Date.now();
    return Boolean(bypassHeld)
      || (now < Number(bypassUntil || 0) && sameLeaseTab(bypassTabId, tabId))
      || now < Number(bypassNextUntil || 0);
  } catch { return false; }
}
function forceIsActive(tabId = null) {
  try {
    if (bypassIsActive(tabId)) return false;
    return Boolean(forceHeld) || (Date.now() < forceUntil && sameLeaseTab(forceTabId, tabId));
  } catch { return false; }
}

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

const genericHttpDownload = (value) => {
  try {
    const url = new URL(value);
    return /^(https?):$/i.test(url.protocol) ? { url: url.href, kind: "forced" } : null;
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
  const recognized = recognizedDownload(request?.url || "") || (request?.force ? genericHttpDownload(request?.url || "") : null);
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


chrome.runtime.onMessage.addListener((message, sender, reply) => {
  const shortcutTabId = Number.isInteger(sender.tab?.id) ? sender.tab.id : null;
  if (message?.type === "APOCALIPSE_SHORTCUT_STATE") {
    bypassHeld = Boolean(message.bypassPressed);
    forceHeld = Boolean(message.forcePressed) && !bypassHeld;
    if (bypassHeld) armBypass(shortcutTabId, 4000);
    else if (forceHeld) armForce(shortcutTabId, 20000);
    reply({ ok: true, mode: bypassHeld ? "bypass" : (forceHeld ? "force" : "normal") });
    return;
  }
  if (message?.type === "APOCALIPSE_BYPASS_NEXT") {
    armBypass(shortcutTabId, message.ttlMs);
    reply({ ok: true, mode: "bypass" });
    return;
  }
  if (message?.type === "APOCALIPSE_FORCE_NEXT") {
    armForce(shortcutTabId, message.ttlMs);
    reply({ ok: true, mode: "force" });
    return;
  }
  if (message?.type === "APOCALIPSE_CAPTURE_TRACE") {
    const state = { traceId: message.traceId || crypto.randomUUID(), pageUrl: message.pageUrl || sender.tab?.url || null, startedAt: Number(message.at || Date.now()), bytes: 0 };
    const detail = Object.entries(message.detail || {}).map(([k,v]) => `${k}=${String(v ?? "").slice(0,180)}`).join(" ");
    void diagnostic(`capture.${String(message.eventName || "event")}`, state, { detail: `mode=${message.mode || "normal"} ${detail}`.trim() });
    reply({ ok: true });
    return;
  }
  if (message?.type !== "APOCALIPSE_PRE_DOWNLOAD_URL") return;
  if (bypassIsActive(shortcutTabId)) {
    reply({ ok: false, bypass: true });
    return;
  }
  const request = {
    url: message.url,
    pageUrl: message.pageUrl || sender.tab?.url || null,
    fileName: message.fileName || "",
    source: message.source || "main-world",
    force: Boolean(message.force) || forceIsActive(shortcutTabId),
  };
  streamCapturedUrl(request)
    .then(reply)
    .catch((error) => {
      // Only on a real takeover failure, fall back to Chrome so the user does not
      // lose a one-use link. Normal successful flow never reaches chrome.downloads.
      const recognized = recognizedDownload(request.url) || (request.force ? genericHttpDownload(request.url) : null);
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
