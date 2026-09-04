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
      .then(() => bridgeRequest("/v1/site-rules"))
      .then((rules) => { if (Array.isArray(rules)) siteRules = rules; })
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

async function takeBrowserDownload(item, eraseFromHistory = false) {
  let url = item.finalUrl || item.url;
  if (!item.id) return false;
  if (/^blob:https:\/\/web\.telegram\.org\//i.test(url)) {
    if (Date.now() < bypassNextUntil) {
      bypassNextUntil = 0;
      return false;
    }
    if (!bridgeConnected || bypassHeld || Date.now() < bypassUntil) return false;
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
  if (Date.now() < bypassNextUntil) {
    bypassNextUntil = 0;
    return false;
  }
  if (!bridgeConnected || bypassHeld || Date.now() < bypassUntil) return false;
  let cancelled = false;
  try {
    await cancelBrowserDownload(item.id);
    cancelled = true;
    if (eraseFromHistory) await eraseBrowserDownload(item.id);
    formRequest ||= lastFormSubmission
      && Date.now() - lastFormSubmission.capturedAt < 30000
      && pageUrl === lastFormSubmission.pageUrl
      ? lastFormSubmission
      : null;
    if (formRequest) lastFormSubmission = null;
    await bridgeRequest("/v1/download", {
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
      }),
    });
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

chrome.runtime.onMessage.addListener((message, sender, reply) => {
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
  if (message?.type === "APOCALIPSE_BLOB_END") {
    bridgeRequest("/v1/blob/end", { method: "POST", body: JSON.stringify(message.request) }).then(reply)
      .catch((error) => reply({ error: String(error) }));
    return true;
  }
  if (message?.type === "APOCALIPSE_FORM_SUBMIT" && message.request?.method === "POST") {
    lastFormSubmission = message.request;
    reply({ ok: true });
    return;
  }
  if (message?.type === "APOCALIPSE_BYPASS_NEXT") {
    bypassNextUntil = Date.now() + Math.min(Math.max(Number(message.ttlMs) || 15000, 2000), 30000);
    reply({ ok: true });
    return;
  }
  if (message?.type === "APOCALIPSE_SHORTCUT_STATE") {
    const wasBypassHeld = bypassHeld;
    bypassHeld = Boolean(message.bypassPressed);
    forceHeld = Boolean(message.forcePressed);
    if (wasBypassHeld && !bypassHeld) bypassUntil = Date.now() + 2000;
    reply({ ok: true, forceHeld });
    return;
  }
  if (message?.type === "APOCALIPSE_MEDIA" && sender.tab?.id) {
    chrome.storage.session.set({ [`media:${sender.tab.id}`]: message.media });
  }
  if (message?.type === "APOCALIPSE_PROBE") {
    fetch(message.url, { method: "HEAD", credentials: "include", redirect: "follow" })
      .then((response) => reply({
        size: Number(response.headers.get("content-length")) || null,
        contentType: response.headers.get("content-type") || "",
      }))
      .catch(() => reply({ size: null }));
    return true;
  }
  if (message?.type === "APOCALIPSE_SELECT_HLS") {
    analyzeHls(message.urls, message.expectedDuration).then((items) => reply(items.find((item) => item.recommended) || null));
    return true;
  }
  if (message?.type === "APOCALIPSE_ANALYZE_HLS") {
    analyzeHls(message.urls, message.expectedDuration).then(reply);
    return true;
  }
  if (message?.type === "APOCALIPSE_PAIR") {
    const token = message.token.trim();
    bridgeRequest("/v1/health", {}, token)
      .then(() => chrome.storage.local.set({ pairingToken: token }))
      .then(() => ensureHeartbeat())
      .then(() => reply({ connected: true }))
      .catch((error) => reply({ connected: false, error: String(error) }));
    return true;
  }
  if (message?.type === "APOCALIPSE_BRIDGE_STATUS") {
    bridgeRequest("/v1/health")
      .then(() => reply({ connected: true }))
      .catch(() => reply({ connected: false }));
    return true;
  }
  if (message?.type === "APOCALIPSE_DOWNLOAD" && message.item?.url) {
    sourcePageUrl(sender).then(async (pageUrl) => bridgeRequest("/v1/download", {
        method: "POST",
        body: JSON.stringify({
          url: message.item.url,
          fileName: message.item.title || null,
          pageUrl,
          duration: Number.isFinite(message.item.duration) ? message.item.duration : null,
          cookieHeader: await cookieHeaderFor([message.item.url, ...(message.item.requestUrls || []), pageUrl]),
          userAgent: message.item.userAgent || null,
        }),
      }))
      .then(() => reply({ target: "apocalipse" }))
      .catch((error) => reply({ target: "error", error: String(error) }));
    return true;
  }
});
