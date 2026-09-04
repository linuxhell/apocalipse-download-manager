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
