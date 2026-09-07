importScripts("service-worker.js");

const RAPIDGATOR_BRIDGE = "http://127.0.0.1:17654";

async function rapidgatorCookieHeader(url) {
  const cookies = await chrome.cookies.getAll({ url }).catch(() => []);
  // Match the browser's Cookie header as closely as possible. Do not merge
  // cookies from the landing page or deduplicate names: Rapidgator can keep
  // host/path-specific values with the same name and the final CDN request
  // must receive only cookies that are actually valid for that URL.
  cookies.sort((left, right) => {
    const pathLength = String(right.path || "/").length - String(left.path || "/").length;
    if (pathLength !== 0) return pathLength;
    return Number(left.hostOnly) - Number(right.hostOnly);
  });
  return cookies.map((cookie) => `${cookie.name}=${cookie.value}`).join("; ");
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
      cookieHeader: await rapidgatorCookieHeader(url),
      userAgent: item.userAgent || null,
      requestMethod: "GET",
      // Empty body is an internal marker that disables HEAD/Range preflights.
      // The core intentionally does NOT put this empty body on the wire, so
      // Rapidgator receives a normal browser-like GET without Content-Length: 0.
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
