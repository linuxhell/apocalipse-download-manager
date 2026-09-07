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

async function rapidgatorDiagnostic(event, transport = {}, extra = {}) {
  const traceId = extra.traceId || transport.transportId || null;
  const startedAt = Number(transport.startedAt || extra.startedAt || 0);
  const durationMs = startedAt ? Math.max(0, Date.now() - startedAt) : null;
  const detail = [
    extra.detail || "",
    extra.contentType ? `content_type=${String(extra.contentType).slice(0, 160)}` : "",
    extra.disposition ? `content_disposition=${String(extra.disposition).slice(0, 200)}` : "",
    extra.error ? `error=${String(extra.error).slice(0, 1000).replace(/[\r\n]+/g, " ")}` : "",
    Number.isFinite(extra.chunkCount) ? `chunks=${extra.chunkCount}` : "",
  ].filter(Boolean).join(" ");
  await rapidgatorBridgePost("/v1/diagnostic", {
    event,
    level: extra.level || (extra.error ? "ERROR" : "INFO"),
    traceId,
    source: "chrome-extension",
    url: extra.url || transport.url || null,
    status: Number.isFinite(extra.status) ? extra.status : null,
    bytes: Number.isFinite(extra.bytes) ? extra.bytes : (Number.isFinite(transport.bytes) ? transport.bytes : null),
    durationMs,
    detail: detail || null,
  }).catch(() => {});
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

async function rapidgatorNotifyFrame(tabId, transport, ok, error = null) {
  await chrome.tabs.sendMessage(tabId, {
    type: "APOCALIPSE_RAPIDGATOR_BROWSER_TRANSPORT_DONE",
    transportId: transport.transportId,
    ok,
    error,
  }, { frameId: transport.frameId }).catch(() => {});
}

async function rapidgatorFinishTransport(tabId, ok, error = null) {
  const transport = rapidgatorTransports.get(tabId);
  if (!transport) return;
  rapidgatorTransports.delete(tabId);
  if (transport.timeoutId) clearTimeout(transport.timeoutId);
  await rapidgatorNotifyFrame(tabId, transport, ok, error);
  await debuggerDetach({ tabId });
}

async function rapidgatorStreamPausedResponse(tabId, params) {
  const transport = rapidgatorTransports.get(tabId);
  if (!transport || params.request?.url !== transport.url) return;
  const debuggee = { tabId };
  transport.requestId = params.requestId;
  transport.responseSeen = true;
  if (transport.timeoutId) {
    clearTimeout(transport.timeoutId);
    transport.timeoutId = null;
  }

  try {
    const status = Number(params.responseStatusCode || 0);
    const headers = params.responseHeaders || [];
    const contentType = rapidgatorResponseHeader(headers, "content-type").toLowerCase();
    const disposition = rapidgatorResponseHeader(headers, "content-disposition");
    await rapidgatorDiagnostic("rapidgator.browser_transport.response_paused", transport, {
      status,
      contentType,
      disposition,
    });
    if (status < 200 || status >= 300) throw new Error(`rapidgator_http_${status || "unknown"}`);
    if (contentType.includes("application/json")) {
      await rapidgatorDiagnostic("rapidgator.browser_transport.unexpected_json", transport, {
        level: "ERROR",
        status,
        contentType,
      });
      throw new Error(`rapidgator_unexpected_${contentType || "response"}`);
    }
    if (contentType.includes("text/html")) {
      await rapidgatorDiagnostic("rapidgator.browser_transport.unexpected_html", transport, {
        level: "ERROR",
        status,
        contentType,
      });
      throw new Error(`rapidgator_unexpected_${contentType || "response"}`);
    }

    const total = Number.parseInt(rapidgatorResponseHeader(headers, "content-length"), 10) || 0;
    const fileName = rapidgatorFileName(disposition, transport.url);
    await rapidgatorDiagnostic("rapidgator.browser_transport.blob_begin", transport, {
      status,
      bytes: 0,
      detail: `expected_bytes=${total} file=${fileName}`,
    });
    const begin = await rapidgatorBridgePost("/v1/blob/begin", {
      fileName,
      total,
      source: transport.pageUrl || transport.url,
      streaming: total === 0,
    });
    if (!begin?.uploadId) throw new Error("blob_begin_failed");

    const body = await debuggerCommand(debuggee, "Fetch.takeResponseBodyAsStream", { requestId: params.requestId });
    if (!body?.stream) throw new Error("rapidgator_stream_unavailable");

    let chunks = 0;
    let lastProgressBytes = 0;
    let lastProgressAt = Date.now();
    while (true) {
      const chunk = await debuggerCommand(debuggee, "IO.read", { handle: body.stream, size: 64 * 1024 });
      const bytes = rapidgatorBytesFromIo(chunk.data || "", Boolean(chunk.base64Encoded));
      if (bytes.length) {
        await rapidgatorBridgePost("/v1/blob/chunk", {
          uploadId: begin.uploadId,
          data: rapidgatorHex(bytes),
        });
        transport.bytes += bytes.length;
        chunks += 1;
        const now = Date.now();
        if (transport.bytes - lastProgressBytes >= 1024 * 1024 || now - lastProgressAt >= 2000) {
          lastProgressBytes = transport.bytes;
          lastProgressAt = now;
          await rapidgatorDiagnostic("rapidgator.browser_transport.progress", transport, {
            bytes: transport.bytes,
            chunkCount: chunks,
          });
        }
      }
      if (chunk.eof) break;
    }

    await debuggerCommand(debuggee, "IO.close", { handle: body.stream }).catch(() => {});
    await rapidgatorBridgePost("/v1/blob/end", { uploadId: begin.uploadId });
    await rapidgatorDiagnostic("rapidgator.browser_transport.completed", transport, {
      status,
      bytes: transport.bytes,
      chunkCount: chunks,
      detail: `expected_bytes=${total}`,
    });
    await debuggerCommand(debuggee, "Fetch.failRequest", {
      requestId: params.requestId,
      errorReason: "Aborted",
    }).catch(() => {});
    await rapidgatorFinishTransport(tabId, true);
  } catch (error) {
    await rapidgatorDiagnostic("rapidgator.browser_transport.failed", transport, {
      level: "ERROR",
      error: String(error),
      bytes: transport.bytes,
    });
    await debuggerCommand(debuggee, "Fetch.failRequest", {
      requestId: params.requestId,
      errorReason: "Aborted",
    }).catch(() => {});
    await rapidgatorFinishTransport(tabId, false, String(error));
  }
}

chrome.debugger.onEvent.addListener((source, method, params) => {
  if (!source.tabId || method !== "Fetch.requestPaused") return;
  const transport = rapidgatorTransports.get(source.tabId);
  if (transport) {
    void rapidgatorDiagnostic("rapidgator.browser_transport.debugger_event", transport, {
      detail: `method=${method} request_stage=${params.responseStatusCode ? "response" : "request"}`,
    });
  }
  void rapidgatorStreamPausedResponse(source.tabId, params);
});

chrome.debugger.onDetach.addListener((source, reason) => {
  if (!source.tabId) return;
  const transport = rapidgatorTransports.get(source.tabId);
  if (!transport) return;
  rapidgatorTransports.delete(source.tabId);
  if (transport.timeoutId) clearTimeout(transport.timeoutId);
  void rapidgatorNotifyFrame(source.tabId, transport, false, `debugger_detached:${reason || "unknown"}`);
  void rapidgatorDiagnostic("rapidgator.browser_transport.debugger_detached", transport, {
    level: "ERROR",
    error: reason || "unknown",
  });
});

chrome.downloads.onCreated.addListener((item) => {
  const itemUrl = item.finalUrl || item.url || "";
  const match = [...rapidgatorTransports.entries()].find(([, transport]) => transport.url === itemUrl);
  if (!match) return;
  const [tabId, transport] = match;
  void rapidgatorDiagnostic("rapidgator.browser_transport.download_escape", transport, {
    level: "ERROR",
    error: "chrome_download_created_before_response_interception",
  });
  chrome.downloads.cancel(item.id, () => {
    void chrome.runtime.lastError;
    chrome.downloads.erase({ id: item.id }, () => void chrome.runtime.lastError);
  });
  void rapidgatorFinishTransport(tabId, false, "chrome_download_escape");
});

async function startRapidgatorBrowserTransport(item, sender) {
  const tabId = sender.tab?.id;
  if (!tabId) throw new Error("rapidgator_tab_missing");
  if (rapidgatorTransports.has(tabId)) throw new Error("rapidgator_transfer_already_active");

  const debuggee = { tabId };
  const transportId = crypto.randomUUID();
  const transport = {
    transportId,
    url: item.url,
    pageUrl: sender.tab?.url || null,
    frameId: Number.isInteger(sender.frameId) ? sender.frameId : 0,
    startedAt: Date.now(),
    bytes: 0,
    responseSeen: false,
    requestId: null,
    timeoutId: null,
  };

  await rapidgatorBridgeHealth();
  await rapidgatorDiagnostic("rapidgator.browser_transport.requested", transport, {
    detail: `tab=${tabId} frame=${transport.frameId}`,
  });
  try {
    await debuggerAttach(debuggee);
    await rapidgatorDiagnostic("rapidgator.browser_transport.debugger_attached", transport);
    await debuggerCommand(debuggee, "Fetch.enable", {
      patterns: [{ urlPattern: item.url, requestStage: "Response" }],
    });
    await rapidgatorDiagnostic("rapidgator.browser_transport.fetch_enabled", transport);
    rapidgatorTransports.set(tabId, transport);

    // If the CDP response interception unexpectedly misses the download, keep
    // the generic interceptor from re-sending this one-shot URL to the native
    // HTTP engine. The escape listener above will cancel that browser download.
    await chrome.runtime.sendMessage({ type: "APOCALIPSE_BYPASS_NEXT", ttlMs: 30000 }).catch(() => {});

    const result = await chrome.tabs.sendMessage(tabId, {
      type: "APOCALIPSE_RAPIDGATOR_START_BROWSER_TRANSPORT",
      transportId,
      url: item.url,
    }, { frameId: transport.frameId });
    if (!result?.started) throw new Error(result?.error || "rapidgator_browser_request_not_started");
    await rapidgatorDiagnostic("rapidgator.browser_transport.browser_request_started", transport);

    transport.timeoutId = setTimeout(() => {
      const current = rapidgatorTransports.get(tabId);
      if (!current || current.responseSeen) return;
      void rapidgatorDiagnostic("rapidgator.browser_transport.response_timeout", current, {
        level: "ERROR",
        error: "no_Fetch.requestPaused_response_within_20s",
      });
      void rapidgatorFinishTransport(tabId, false, "rapidgator_response_interception_timeout");
    }, 20000);

    return { target: "apocalipse", transport: "browser_stream", transportId };
  } catch (error) {
    rapidgatorTransports.delete(tabId);
    if (transport.timeoutId) clearTimeout(transport.timeoutId);
    await rapidgatorDiagnostic("rapidgator.browser_transport.setup_failed", transport, {
      level: "ERROR",
      error: String(error),
    });
    await debuggerDetach(debuggee);
    throw error;
  }
}

chrome.runtime.onMessage.addListener((message, sender, reply) => {
  if (message?.type !== "APOCALIPSE_RAPIDGATOR_DOWNLOAD" || !message.item?.url) return;
  startRapidgatorBrowserTransport(message.item, sender)
    .then(reply)
    .catch((error) => reply({ target: "error", error: String(error), noReplay: true }));
  return true;
});
