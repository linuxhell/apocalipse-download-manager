const BRIDGE = "http://127.0.0.1:17654";
const HEARTBEAT_ALARM = "apocalipse-bridge-heartbeat";

async function bridgeRequest(path, options = {}, suppliedToken = null) {
  const { pairingToken = "" } = suppliedToken === null ? await chrome.storage.local.get({ pairingToken: "" }) : { pairingToken: suppliedToken };
  if (!pairingToken) throw new Error("not_paired");
  const response = await fetch(`${BRIDGE}${path}`, {
    ...options,
    headers: {
      "Authorization": `Bearer ${pairingToken}`,
      "Content-Type": "application/json",
      ...(options.headers || {}),
    },
  });
  if (!response.ok) throw new Error(`bridge_http_${response.status}`);
  return response.json();
}

function ensureHeartbeat() {
  chrome.alarms.create(HEARTBEAT_ALARM, { delayInMinutes: 0.1, periodInMinutes: 0.5 });
}

ensureHeartbeat();
chrome.runtime.onInstalled.addListener(ensureHeartbeat);
chrome.runtime.onStartup.addListener(ensureHeartbeat);
chrome.alarms.onAlarm.addListener((alarm) => {
  if (alarm.name === HEARTBEAT_ALARM) bridgeRequest("/v1/health").catch(() => {});
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

chrome.downloads.onCreated.addListener(async (item) => {
  const url = item.finalUrl || item.url;
  if (!item.id || !/^https?:/i.test(url)) return;
  try {
    await chrome.downloads.pause(item.id);
    const pageUrl = item.referrer || null;
    await bridgeRequest("/v1/download", {
      method: "POST",
      body: JSON.stringify({
        url,
        fileName: fileNameFromPath(item.filename),
        pageUrl,
        duration: null,
        cookieHeader: await cookieHeaderFor([url, item.url, pageUrl]),
        userAgent: navigator.userAgent,
      }),
    });
    await chrome.downloads.cancel(item.id);
    await chrome.downloads.erase({ id: item.id });
  } catch {
    chrome.downloads.resume(item.id).catch(() => {});
  }
});

chrome.runtime.onMessage.addListener((message, sender, reply) => {
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
