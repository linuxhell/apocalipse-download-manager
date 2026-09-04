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
  if (depth > 1) return null;
  const response = await fetch(url, { credentials: "include", redirect: "follow" });
  if (!response.ok) return null;
  const text = await response.text();
  const lines = text.split(/\r?\n/).map((line) => line.trim()).filter(Boolean);
  const variant = lines.findIndex((line) => line.startsWith("#EXT-X-STREAM-INF"));
  if (variant >= 0) {
    const child = lines.slice(variant + 1).find((line) => !line.startsWith("#"));
    return child ? hlsDuration(new URL(child, url).href, depth + 1) : null;
  }
  const durations = lines.filter((line) => line.startsWith("#EXTINF:"))
    .map((line) => Number.parseFloat(line.slice(8))).filter(Number.isFinite);
  return durations.length ? durations.reduce((total, value) => total + value, 0) : null;
}

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
    Promise.all((message.urls || []).slice(-20).map(async (url) => ({ url, duration: await hlsDuration(url).catch(() => null) })))
      .then((items) => {
        const valid = items.filter((item) => Number.isFinite(item.duration));
        const expected = Number(message.expectedDuration);
        if (Number.isFinite(expected) && expected > 0) valid.sort((a, b) => Math.abs(a.duration - expected) - Math.abs(b.duration - expected));
        else valid.sort((a, b) => b.duration - a.duration);
        reply(valid[0] || { url: message.urls?.at(-1) || null, duration: null });
      });
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
    bridgeRequest("/v1/download", {
      method: "POST",
      body: JSON.stringify({
        url: message.item.url,
        fileName: message.item.title || null,
        pageUrl: sender.tab?.url || null,
      }),
    }).then(() => reply({ target: "apocalipse" })).catch(() => {
      chrome.downloads.download({ url: message.item.url, saveAs: true }, (downloadId) => {
        reply({ target: "browser", downloadId, error: chrome.runtime.lastError?.message });
      });
    });
    return true;
  }
});
