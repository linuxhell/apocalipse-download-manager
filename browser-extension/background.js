importScripts("service-worker.js");

const RAPIDGATOR_BRIDGE = "http://127.0.0.1:17654";
const rapidgatorArmedTabs = new Map();
const chatgptLibraryTransfers = new Set();

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

async function rapidgatorBridgeHealth() {
  const pairingToken = await rapidgatorPairingToken();
  const response = await fetch(`${RAPIDGATOR_BRIDGE}/v1/health`, {
    method: "GET",
    headers: { "Authorization": `Bearer ${pairingToken}` },
  });
  if (!response.ok) throw new Error(`bridge_http_${response.status}`);
}

const finalRapidgatorUrl = (value) => {
  try {
    const url = new URL(value);
    if (!/^s\d+\.rapidgator\.net$/i.test(url.hostname)) return null;
    if (!/^\/download\/[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}\/?$/i.test(url.pathname)) return null;
    return url.href;
  } catch {
    return null;
  }
};

const chatgptLibraryUrl = (value) => {
  try {
    const url = new URL(value);
    if (url.hostname.toLowerCase() !== "chatgpt.com") return null;
    if (url.pathname !== "/backend-api/estuary/content") return null;
    return url.href;
  } catch {
    return null;
  }
};

function responseHeader(headers, name) {
  return (headers || []).find((header) => String(header.name || "").toLowerCase() === name.toLowerCase())?.value || "";
}

function fileNameFromDisposition(disposition, fallbackUrl) {
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

function chatgptFileName(item, disposition, fallbackUrl) {
  const itemName = String(item?.filename || "").split(/[\\/]/).pop();
  if (itemName) return itemName;
  const extended = disposition.match(/filename\*\s*=\s*UTF-8''([^;]+)/i)?.[1];
  if (extended) {
    try { return decodeURIComponent(extended.replace(/^"|"$/g, "")); } catch {}
  }
  const quoted = disposition.match(/filename\s*=\s*"([^"]+)"/i)?.[1];
  if (quoted) return quoted;
  const plain = disposition.match(/filename\s*=\s*([^;]+)/i)?.[1]?.trim();
  if (plain) return plain.replace(/^"|"$/g, "");
  try {
    const url = new URL(fallbackUrl);
    return url.searchParams.get("filename") || url.searchParams.get("name") || "chatgpt-library-download";
  } catch {
    return "chatgpt-library-download";
  }
}

function bytesFromIo(data, base64Encoded) {
  if (base64Encoded) {
    const binary = atob(data);
    const bytes = new Uint8Array(binary.length);
    for (let index = 0; index < binary.length; index += 1) bytes[index] = binary.charCodeAt(index);
    return bytes;
  }
  return new TextEncoder().encode(data);
}

function hex(bytes) {
  let value = "";
  for (const byte of bytes) value += byte.toString(16).padStart(2, "0");
  return value;
}

async function diagnostic(event, state = {}, extra = {}) {
  const startedAt = Number(state.startedAt || 0);
  const detail = [
    extra.detail || "",
    `extension_version=${chrome.runtime.getManifest().version}`,
    extra.contentType ? `content_type=${String(extra.contentType).slice(0, 160)}` : "",
    extra.disposition ? `content_disposition=${String(extra.disposition).slice(0, 240)}` : "",
    extra.error ? `error=${String(extra.error).slice(0, 1200).replace(/[\r\n]+/g, " ")}` : "",
    Number.isFinite(extra.chunkCount) ? `chunks=${extra.chunkCount}` : "",
  ].filter(Boolean).join(" ");
  await rapidgatorBridgePost("/v1/diagnostic", {
    event,
    level: extra.level || (extra.error ? "ERROR" : "INFO"),
    traceId: state.traceId || null,
    source: "chrome-extension",
    url: extra.url || state.url || state.pageUrl || null,
    status: Number.isFinite(extra.status) ? extra.status : null,
    bytes: Number.isFinite(extra.bytes) ? extra.bytes : (Number.isFinite(state.bytes) ? state.bytes : null),
    durationMs: startedAt ? Math.max(0, Date.now() - startedAt) : null,
    detail: detail || null,
  }).catch(() => {});
}

const cancelChromeDownload = (id) => new Promise((resolve) => {
  chrome.downloads.cancel(id, () => {
    void chrome.runtime.lastError;
    resolve();
  });
});

const eraseChromeDownload = (id) => new Promise((resolve) => {
  chrome.downloads.erase({ id }, () => {
    void chrome.runtime.lastError;
    resolve();
  });
});

async function streamChatGPTLibraryDownload(item) {
  const url = chatgptLibraryUrl(item?.finalUrl || item?.url || "");
  if (!url || !item?.id || chatgptLibraryTransfers.has(item.id)) return;
  chatgptLibraryTransfers.add(item.id);
  const state = {
    traceId: crypto.randomUUID(),
    url,
    pageUrl: item.referrer || "https://chatgpt.com/",
    startedAt: Date.now(),
    bytes: 0,
  };
  let uploadId = null;
  try {
    try {
      bypassUntil = Date.now() + 15000;
      bypassNextUntil = 0;
    } catch {}
    await diagnostic("chatgpt.library.intercepted", state, { detail: `download_id=${item.id}` });
    await cancelChromeDownload(item.id);
    await eraseChromeDownload(item.id);

    const response = await fetch(url, {
      method: "GET",
      credentials: "include",
      redirect: "follow",
      cache: "no-store",
    });
    const disposition = response.headers.get("content-disposition") || "";
    const contentType = response.headers.get("content-type") || "";
    const total = Number.parseInt(response.headers.get("content-length") || "0", 10) || 0;
    if (!response.ok) {
      throw new Error(`chatgpt_http_${response.status}`);
    }
    const fileName = chatgptFileName(item, disposition, response.url || url);
    await diagnostic("chatgpt.library.response", state, {
      status: response.status,
      contentType,
      disposition,
      detail: `expected_bytes=${total} file=${fileName}`,
    });
    const begin = await rapidgatorBridgePost("/v1/blob/begin", {
      fileName,
      total,
      source: state.pageUrl || url,
      streaming: total === 0,
      promptForDestination: true,
    });
    uploadId = begin?.uploadId || null;
    if (!uploadId) throw new Error("chatgpt_blob_begin_failed");
    if (!response.body) throw new Error("chatgpt_response_stream_unavailable");

    const reader = response.body.getReader();
    let chunks = 0;
    while (true) {
      const { value, done } = await reader.read();
      if (value?.length) {
        for (let offset = 0; offset < value.length; offset += 64 * 1024) {
          const slice = value.subarray(offset, Math.min(value.length, offset + 64 * 1024));
          await rapidgatorBridgePost("/v1/blob/chunk", { uploadId, data: hex(slice) });
          state.bytes += slice.length;
          chunks += 1;
        }
      }
      if (done) break;
    }
    await rapidgatorBridgePost("/v1/blob/end", { uploadId });
    await diagnostic("chatgpt.library.completed", state, {
      status: response.status,
      bytes: state.bytes,
      chunkCount: chunks,
      detail: `expected_bytes=${total}`,
    });
  } catch (error) {
    await diagnostic("chatgpt.library.failed", state, {
      level: "ERROR",
      bytes: state.bytes,
      error: String(error),
      detail: uploadId ? `upload_id=${uploadId}` : "upload_not_started",
    });
  } finally {
    chatgptLibraryTransfers.delete(item.id);
  }
}

chrome.downloads.onCreated.addListener((item) => {
  if (!chatgptLibraryUrl(item?.finalUrl || item?.url || "")) return;
  void streamChatGPTLibraryDownload(item);
});

async function disarmRapidgator(tabId, reason = "done") {
  const state = rapidgatorArmedTabs.get(tabId);
  if (!state) return;
  rapidgatorArmedTabs.delete(tabId);
  await diagnostic("rapidgator.cdp.disarmed", state, { detail: `reason=${reason}` });
  await debuggerDetach({ tabId });
}

async function armRapidgator(tabId, pageUrl) {
  if (!tabId) throw new Error("rapidgator_tab_missing");
  const existing = rapidgatorArmedTabs.get(tabId);
  if (existing) {
    existing.pageUrl = pageUrl || existing.pageUrl;
    return { armed: true, reused: true, traceId: existing.traceId };
  }

  await rapidgatorBridgeHealth();
  const state = {
    traceId: crypto.randomUUID(),
    pageUrl: pageUrl || null,
    url: null,
    startedAt: Date.now(),
    bytes: 0,
    busy: false,
  };
  await diagnostic("rapidgator.cdp.arm_requested", state, { detail: `tab=${tabId}` });

  const debuggee = { tabId };
  try {
    await debuggerAttach(debuggee);
    await diagnostic("rapidgator.cdp.debugger_attached", state);
    await debuggerCommand(debuggee, "Fetch.enable", {
      patterns: [{ urlPattern: "https://s*.rapidgator.net/download/*", requestStage: "Response" }],
    });
    rapidgatorArmedTabs.set(tabId, state);
    await diagnostic("rapidgator.cdp.fetch_armed", state, { detail: "pattern=s*.rapidgator.net/download/* stage=response" });
    return { armed: true, reused: false, traceId: state.traceId };
  } catch (error) {
    await diagnostic("rapidgator.cdp.arm_failed", state, { level: "ERROR", error: String(error) });
    await debuggerDetach(debuggee);
    throw error;
  }
}

async function streamRapidgatorResponse(tabId, params) {
  const state = rapidgatorArmedTabs.get(tabId);
  if (!state || state.busy) return;
  const url = finalRapidgatorUrl(params.request?.url || "");
  if (!url) return;
  state.busy = true;
  state.url = url;
  state.startedAt = Date.now();
  state.bytes = 0;

  const debuggee = { tabId };
  const status = Number(params.responseStatusCode || 0);
  const headers = params.responseHeaders || [];
  const contentType = responseHeader(headers, "content-type").toLowerCase();
  const disposition = responseHeader(headers, "content-disposition");
  const total = Number.parseInt(responseHeader(headers, "content-length"), 10) || 0;

  await diagnostic("rapidgator.cdp.response_paused", state, {
    status,
    contentType,
    disposition,
    detail: `expected_bytes=${total}`,
  });

  if (status < 200 || status >= 300) {
    state.busy = false;
    await diagnostic("rapidgator.cdp.non_success_response", state, { level: "WARN", status, contentType });
    await debuggerCommand(debuggee, "Fetch.continueRequest", { requestId: params.requestId }).catch(() => {});
    return;
  }

  const isFile = contentType.includes("application/octet-stream")
    || /attachment/i.test(disposition)
    || /application\/(?:x-rar|zip|x-7z-compressed)/i.test(contentType);
  if (!isFile) {
    state.busy = false;
    await diagnostic("rapidgator.cdp.response_not_file", state, { level: "WARN", status, contentType, disposition });
    await debuggerCommand(debuggee, "Fetch.continueRequest", { requestId: params.requestId }).catch(() => {});
    return;
  }

  let uploadId = null;
  try {
    const fileName = fileNameFromDisposition(disposition, url);
    await diagnostic("rapidgator.cdp.blob_begin", state, {
      status,
      contentType,
      disposition,
      detail: `expected_bytes=${total} file=${fileName}`,
    });
    const begin = await rapidgatorBridgePost("/v1/blob/begin", {
      fileName,
      total,
      source: state.pageUrl || url,
      streaming: total === 0,
      promptForDestination: true,
    });
    uploadId = begin?.uploadId || null;
    if (!uploadId) throw new Error("blob_begin_failed");

    const body = await debuggerCommand(debuggee, "Fetch.takeResponseBodyAsStream", { requestId: params.requestId });
    if (!body?.stream) throw new Error("rapidgator_stream_unavailable");

    let chunks = 0;
    let lastProgressAt = Date.now();
    let lastProgressBytes = 0;
    while (true) {
      const chunk = await debuggerCommand(debuggee, "IO.read", { handle: body.stream, size: 64 * 1024 });
      const bytes = bytesFromIo(chunk.data || "", Boolean(chunk.base64Encoded));
      if (bytes.length) {
        await rapidgatorBridgePost("/v1/blob/chunk", { uploadId, data: hex(bytes) });
        state.bytes += bytes.length;
        chunks += 1;
        const now = Date.now();
        if (state.bytes - lastProgressBytes >= 1024 * 1024 || now - lastProgressAt >= 1500) {
          lastProgressBytes = state.bytes;
          lastProgressAt = now;
          await diagnostic("rapidgator.cdp.progress", state, { bytes: state.bytes, chunkCount: chunks });
        }
      }
      if (chunk.eof) break;
    }

    await debuggerCommand(debuggee, "IO.close", { handle: body.stream }).catch(() => {});
    await rapidgatorBridgePost("/v1/blob/end", { uploadId });
    await diagnostic("rapidgator.cdp.completed", state, {
      status,
      bytes: state.bytes,
      chunkCount: chunks,
      detail: `expected_bytes=${total}`,
    });

    // The bytes are already inside Apocalipse. Abort the browser-side response
    // so Chrome never opens Save As and never creates the bogus JSON fallback.
    await debuggerCommand(debuggee, "Fetch.failRequest", {
      requestId: params.requestId,
      errorReason: "Aborted",
    }).catch(() => {});
    await disarmRapidgator(tabId, "completed");
  } catch (error) {
    await diagnostic("rapidgator.cdp.failed", state, {
      level: "ERROR",
      status,
      bytes: state.bytes,
      error: String(error),
      detail: uploadId ? `upload_id=${uploadId}` : "upload_not_started",
    });
    await debuggerCommand(debuggee, "Fetch.failRequest", {
      requestId: params.requestId,
      errorReason: "Aborted",
    }).catch(() => {});
    await disarmRapidgator(tabId, "failed");
  }
}

chrome.debugger.onEvent.addListener((source, method, params) => {
  if (!source.tabId || method !== "Fetch.requestPaused") return;
  if (!rapidgatorArmedTabs.has(source.tabId)) return;
  void streamRapidgatorResponse(source.tabId, params);
});

chrome.debugger.onDetach.addListener((source, reason) => {
  if (!source.tabId) return;
  const state = rapidgatorArmedTabs.get(source.tabId);
  if (!state) return;
  rapidgatorArmedTabs.delete(source.tabId);
  void diagnostic("rapidgator.cdp.debugger_detached", state, {
    level: "ERROR",
    error: reason || "unknown",
  });
});

chrome.tabs.onRemoved.addListener((tabId) => {
  if (rapidgatorArmedTabs.has(tabId)) void disarmRapidgator(tabId, "tab_closed");
});

chrome.runtime.onMessage.addListener((message, sender, reply) => {
  if (message?.type !== "APOCALIPSE_RAPIDGATOR_ARM") return;
  const tabId = sender.tab?.id;
  armRapidgator(tabId, message.pageUrl || sender.tab?.url || null)
    .then(reply)
    .catch((error) => reply({ armed: false, error: String(error) }));
  return true;
});
