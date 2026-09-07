importScripts("service-worker.js");

const RAPIDGATOR_BRIDGE = "http://127.0.0.1:17654";
const rapidgatorTransports = new Map();

const debuggerAttach = (debuggee) => new Promise((resolve, reject) => {
  chrome.debugger.attach(debuggee, "1.3", () => {
    const error = chrome.runtime.lastError;
    if (error) reject(new Error(error.message));
    else resolve();
  });
});

const debuggerDetach = (debuggee) => new Promise((resolve) => {
  chrome.debugger.detach(debuggee, () => {
    void chrome.runtime.lastError;
    resolve();
  });
});

const debuggerCommand = (debuggee, method, params = {}) => new Promise((resolve, reject) => {
  chrome.debugger.sendCommand(debuggee, method, params, (result) => {
    const error = chrome.runtime.lastError;
    if (error) reject(new Error(error.message));
    else resolve(result || {});
  });
});

async function rapidgatorPairingToken() {
  const { pairingToken = "" } = await chrome.storage.local.get({ pairingToken: "" });
  if (!pairingToken) throw new Error("not_paired");
  return pairingToken;
}

async function rapidgatorBridgeHealth() {
  const pairingToken = await rapidgatorPairingToken();
  const response = await fetch(`${RAPIDGATOR_BRIDGE}/v1/health`, {
    method: "GET",
    headers: { "Authorization": `Bearer ${pairingToken}` },
  });
  if (!response.ok) throw new Error(`bridge_http_${response.status}`);
}

async function rapidgatorBridgePost(path, body) {
  const pairingToken = await rapidgatorPairingToken();
  const response = await fetch(`${RAPIDGATOR_BRIDGE}${path}`, {
    method: "POST",
    headers: {
      "Authorization": `Bearer ${pairingToken}`,
      "Content-Type": "application/json",
    },
    body: JSON.stringify(body),
  });
  if (!response.ok) throw new Error(`bridge_http_${response.status}`);
  return response.json();
}

function rapidgatorResponseHeader(headers, name) {
  return (headers || []).find((header) => String(header.name || "").toLowerCase() === name.toLowerCase())?.value || "";
}

function rapidgatorFileName(disposition, fallbackUrl) {
  const extended = disposition.match(/filename\*\s*=\s*UTF-8''([^;]+)/i)?.[1];
  if (extended) {
    try { return decodeURIComponent(extended.replace(/^"|"$/g, "")); } catch {}
  }
  const quoted = disposition.match(/filename\s*=\s*"([^"]+)"/i)?.[1];
  if (quoted) return quoted;
  const plain = disposition.match(/filename\s*=\s*([^;]+)/i)?.[1]?.trim();
  if (plain) return plain.replace(/^"|"$/g, "");
  try {
    const token = new URL(fallbackUrl).pathname.split("/").filter(Boolean).pop();
    return token ? `rapidgator-${token}.bin` : "rapidgator-download.bin";
  } catch {
    return "rapidgator-download.bin";
  }
}

function rapidgatorBytesFromIo(data, base64Encoded) {
  if (base64Encoded) {
    const binary = atob(data);
    const bytes = new Uint8Array(binary.length);
    for (let index = 0; index < binary.length; index += 1) bytes[index] = binary.charCodeAt(index);
    return bytes;
  }
  return new TextEncoder().encode(data);
}

function rapidgatorHex(bytes) {
  let value = "";
  for (const byte of bytes) value += byte.toString(16).padStart(2, "0");
  return value;
}

async function rapidgatorFinishTransport(tabId, ok, error = null) {
  const transport = rapidgatorTransports.get(tabId);
  if (!transport) return;
  rapidgatorTransports.delete(tabId);
  await chrome.tabs.sendMessage(tabId, {
    type: "APOCALIPSE_RAPIDGATOR_BROWSER_TRANSPORT_DONE",
    transportId: transport.transportId,
    ok,
    error,
  }, { frameId: transport.frameId }).catch(() => {});
  await debuggerDetach({ tabId });
}

async function rapidgatorStreamPausedResponse(tabId, params) {
  const transport = rapidgatorTransports.get(tabId);
  if (!transport || params.request?.url !== transport.url) return;
  const debuggee = { tabId };
  transport.requestId = params.requestId;

  try {
    const status = Number(params.responseStatusCode || 0);
    const headers = params.responseHeaders || [];
    const contentType = rapidgatorResponseHeader(headers, "content-type").toLowerCase();
    if (status < 200 || status >= 300) throw new Error(`rapidgator_http_${status || "unknown"}`);
    if (contentType.includes("application/json") || contentType.includes("text/html")) {
      throw new Error(`rapidgator_unexpected_${contentType || "response"}`);
    }

    const disposition = rapidgatorResponseHeader(headers, "content-disposition");
    const total = Number.parseInt(rapidgatorResponseHeader(headers, "content-length"), 10) || 0;
    const fileName = rapidgatorFileName(disposition, transport.url);
    const begin = await rapidgatorBridgePost("/v1/blob/begin", {
      fileName,
      total,
      source: transport.pageUrl || transport.url,
      streaming: total === 0,
    });
    if (!begin?.uploadId) throw new Error("blob_begin_failed");

    const body = await debuggerCommand(debuggee, "Fetch.takeResponseBodyAsStream", { requestId: params.requestId });
    if (!body?.stream) throw new Error("rapidgator_stream_unavailable");

    while (true) {
      const chunk = await debuggerCommand(debuggee, "IO.read", { handle: body.stream, size: 64 * 1024 });
      const bytes = rapidgatorBytesFromIo(chunk.data || "", Boolean(chunk.base64Encoded));
      if (bytes.length) {
        await rapidgatorBridgePost("/v1/blob/chunk", {
          uploadId: begin.uploadId,
          data: rapidgatorHex(bytes),
        });
      }
      if (chunk.eof) break;
    }

    await debuggerCommand(debuggee, "IO.close", { handle: body.stream }).catch(() => {});
    await rapidgatorBridgePost("/v1/blob/end", { uploadId: begin.uploadId });
    await debuggerCommand(debuggee, "Fetch.failRequest", {
      requestId: params.requestId,
      errorReason: "Aborted",
    }).catch(() => {});
    await rapidgatorFinishTransport(tabId, true);
  } catch (error) {
    await debuggerCommand(debuggee, "Fetch.failRequest", {
      requestId: params.requestId,
      errorReason: "Aborted",
    }).catch(() => {});
    await rapidgatorFinishTransport(tabId, false, String(error));
  }
}

chrome.debugger.onEvent.addListener((source, method, params) => {
  if (!source.tabId || method !== "Fetch.requestPaused") return;
  void rapidgatorStreamPausedResponse(source.tabId, params);
});

chrome.debugger.onDetach.addListener((source) => {
  if (source.tabId) rapidgatorTransports.delete(source.tabId);
});

async function startRapidgatorBrowserTransport(item, sender) {
  const tabId = sender.tab?.id;
  if (!tabId) throw new Error("rapidgator_tab_missing");
  if (rapidgatorTransports.has(tabId)) throw new Error("rapidgator_transfer_already_active");

  const debuggee = { tabId };
  const transportId = crypto.randomUUID();
  await rapidgatorBridgeHealth();
  await debuggerAttach(debuggee);
  try {
    await debuggerCommand(debuggee, "Fetch.enable", {
      patterns: [{ urlPattern: item.url, requestStage: "Response" }],
    });
    rapidgatorTransports.set(tabId, {
      transportId,
      url: item.url,
      pageUrl: sender.tab?.url || null,
      frameId: Number.isInteger(sender.frameId) ? sender.frameId : 0,
    });
    const result = await chrome.tabs.sendMessage(tabId, {
      type: "APOCALIPSE_RAPIDGATOR_START_BROWSER_TRANSPORT",
      transportId,
      url: item.url,
    }, { frameId: Number.isInteger(sender.frameId) ? sender.frameId : 0 });
    if (!result?.started) throw new Error(result?.error || "rapidgator_browser_request_not_started");
    return { target: "apocalipse", transport: "browser_stream", transportId };
  } catch (error) {
    rapidgatorTransports.delete(tabId);
    await debuggerDetach(debuggee);
    throw error;
  }
}

chrome.runtime.onMessage.addListener((message, sender, reply) => {
  if (message?.type !== "APOCALIPSE_RAPIDGATOR_DOWNLOAD" || !message.item?.url) return;
  startRapidgatorBrowserTransport(message.item, sender)
    .then(reply)
    .catch((error) => reply({ target: "error", error: String(error) }));
  return true;
});
