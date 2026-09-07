importScripts("service-worker.js");

const RAPIDGATOR_BRIDGE = "http://127.0.0.1:17654";

async function rapidgatorCookieHeader(urls) {
  const groups = await Promise.all([...new Set((urls || []).filter((url) => /^https?:/i.test(url)))]
    .map((url) => chrome.cookies.getAll({ url }).catch(() => [])));
  const values = new Map();
  for (const cookie of groups.flat()) values.set(cookie.name, cookie.value);
  return [...values].map(([name, value]) => `${name}=${value}`).join("; ");
}

async function rapidgatorBridgeRequest(item, sender) {
  const { pairingToken = "" } = await chrome.storage.local.get({ pairingToken: "" });
  if (!pairingToken) throw new Error("not_paired");

  const pageUrl = sender.tab?.url || null;
  const url = item.url;
  const response = await fetch(`${RAPIDGATOR_BRIDGE}/v1/download`, {
    method: "POST",
    headers: {
      "Authorization": `Bearer ${pairingToken}`,
      "Content-Type": "application/json",
    },
    body: JSON.stringify({
      url,
      fileName: item.title || null,
      pageUrl,
      duration: null,
      cookieHeader: await rapidgatorCookieHeader([url, pageUrl]),
      userAgent: item.userAgent || null,
      requestMethod: "GET",
      // An explicit empty body keeps the core on the single-request path:
      // no HEAD, no Range 0-0 probe and no segmented preflight before the real GET.
      requestBody: "",
      requestContentType: null,
      startImmediately: true,
    }),
  });
  if (!response.ok) throw new Error(`bridge_http_${response.status}`);
  return response.json();
}

chrome.runtime.onMessage.addListener((message, sender, reply) => {
  if (message?.type !== "APOCALIPSE_RAPIDGATOR_DOWNLOAD" || !message.item?.url) return;
  rapidgatorBridgeRequest(message.item, sender)
    .then((result) => reply({ target: "apocalipse", taskId: result?.taskId || null }))
    .catch((error) => reply({ target: "error", error: String(error) }));
  return true;
});
