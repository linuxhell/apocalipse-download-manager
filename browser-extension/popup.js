let media = [], selected = "video", locale = "en", activePageUrl = "";
const selectedUrls = new Set();
const SOCIAL_TRACK_PAIR_WINDOW_MS = 8_000;
const isSocialPage = (url) => {
  try { return /(^|\.)(?:facebook|tiktok)\.com$/i.test(new URL(url).hostname); } catch { return false; }
};
const pairSocialTracks = (items, pageUrl) => {
  if (!isSocialPage(pageUrl)) return items;
  const audio = items.filter((item) => item.kind === "audio" && item.networkCaptured);
  const pairedAudio = new Set();
  const result = items.map((item) => {
    if (item.kind !== "video" || item.pageExtractor || item.audioUrl || /\.(?:m3u8|mpd)(?:$|[?#])/i.test(item.url)) return item;
    const source = item.networkCaptured
      ? item
      : items.find((candidate) => candidate.networkCaptured && candidate.kind === "video" && candidate.url === item.url);
    if (!Number.isFinite(source?.capturedAt) || source.capturedAt <= 0) return { ...item, ambiguousSocialTrack: true };
    const candidates = audio
      .filter((candidate) => Number.isFinite(candidate.capturedAt) && candidate.capturedAt > 0
        && (candidate.frameId === source.frameId || (candidate.frameId == null && source.frameId == null)))
      .filter((candidate) => !items.some((other) => other.kind === "video" && other.networkCaptured
        && other.url !== source.url && Number.isFinite(other.capturedAt) && other.capturedAt > 0
        && (other.frameId === candidate.frameId || (other.frameId == null && candidate.frameId == null))
        && Math.abs(other.capturedAt - candidate.capturedAt) <= Math.abs(source.capturedAt - candidate.capturedAt)))
      .map((candidate) => ({ candidate, delta: Math.abs(candidate.capturedAt - source.capturedAt) }))
      .filter(({ delta }) => delta <= SOCIAL_TRACK_PAIR_WINDOW_MS)
      .sort((left, right) => left.delta - right.delta);
    const companion = candidates.length > 1 && candidates[0].delta === candidates[1].delta ? null : candidates[0]?.candidate;
    if (!companion) return { ...item, ambiguousSocialTrack: true };
    pairedAudio.add(companion.url);
    return { ...item, audioUrl: companion.url, recommended: true, ambiguousSocialTrack: false };
  });
  return result.filter((item) => !(item.kind === "audio" && item.networkCaptured && pairedAudio.has(item.url)));
};
const networkMediaKind = (item) => {
  if (/^audio\//i.test(item.contentType || "")) return "audio";
  return /^video\//i.test(item.contentType || "") || /(?:\/video\/tos\/|mime_type=video|\.mp4(?:$|[?#]))/i.test(item.url || "") ? "video" : "audio";
};
const mergeDetectedMedia = (scanned, network, pageUrl) => {
  const unique = new Map();
  const hasSocialPageItems = isSocialPage(pageUrl) && scanned.some((item) => item?.kind === "video" && item.pageExtractor);
  for (const item of [...scanned, ...network]) {
    if (!item?.url) continue;
    if (hasSocialPageItems && item.networkCaptured && (item.kind === "video" || item.kind === "audio")) continue;
    const previous = unique.get(item.url);
    if (!previous) unique.set(item.url, item);
    else if (item.networkCaptured) unique.set(item.url, {
      ...item, ...previous,
      kind: /^audio\//i.test(item.contentType || "") ? "audio" : previous.kind,
      contentType: item.contentType || previous.contentType,
      capturedAt: item.capturedAt, frameId: item.frameId, networkCaptured: true,
    });
  }
  return pairSocialTracks([...unique.values()], pageUrl);
};
const messages = {
  en: { mediaIntelligence: "Media intelligence", video: "Video", audio: "Audio", images: "Images", download: "Download", externalPreview: "Open in player", incompleteTrack: "Incomplete track", empty: "No media detected in this tab.", unknownSize: "Size unavailable", connected: "Connected to Apocalipse", disconnected: "Disconnected", pairingToken: "Pairing token", connect: "Connect", recommended: "Recommended", capturedResource: "Captured media resource", requestedMedia: "You tried to download", selectAll: "Select all", downloadSelected: "Download selected", forceShortcut: "Force Apocalipse", bypassShortcut: "Bypass Apocalipse" },
  pt_BR: { mediaIntelligence: "Inteligência de mídia", video: "Vídeo", audio: "Áudio", images: "Imagens", download: "Download", externalPreview: "Abrir no player", incompleteTrack: "Faixa incompleta", empty: "Nenhuma mídia detectada nesta aba.", unknownSize: "Tamanho indisponível", connected: "Conectada ao Apocalipse", disconnected: "Desconectada", pairingToken: "Token de pareamento", connect: "Conectar", recommended: "Recomendada", capturedResource: "Recurso de mídia capturado", requestedMedia: "Você tentou baixar", selectAll: "Selecionar todos", downloadSelected: "Baixar selecionados", forceShortcut: "Forçar Apocalipse", bypassShortcut: "Ignorar Apocalipse" },
  zh_CN: { mediaIntelligence: "媒体智能", video: "视频", audio: "音频", images: "图片", download: "下载", externalPreview: "在播放器中打开", incompleteTrack: "不完整音视频轨道", empty: "此标签页未检测到媒体。", unknownSize: "大小未知", connected: "已连接到 Apocalipse", disconnected: "未连接", pairingToken: "配对令牌", connect: "连接", recommended: "推荐", capturedResource: "已捕获的媒体资源", requestedMedia: "您尝试下载", selectAll: "全选", downloadSelected: "下载所选项目", forceShortcut: "强制使用 Apocalipse", bypassShortcut: "绕过 Apocalipse" }
};
const t = (key) => messages[locale]?.[key] || messages.en[key] || key;
const formatBytes = (bytes) => {
  if (!bytes) return t("unknownSize");
  const units = ["B", "KB", "MB", "GB"];
  const index = Math.min(Math.floor(Math.log(bytes) / Math.log(1024)), units.length - 1);
  return `${(bytes / 1024 ** index).toFixed(index ? 1 : 0)} ${units[index]}`;
};
const formatDuration = (seconds) => {
  if (!Number.isFinite(seconds)) return "";
  const hours = Math.floor(seconds / 3600), minutes = Math.floor((seconds % 3600) / 60), rest = Math.floor(seconds % 60);
  return [hours, minutes, rest].filter((_, index) => index || hours).map((value) => String(value).padStart(2, "0")).join(":");
};
const translate = () => document.querySelectorAll("[data-i18n]").forEach((element) => {
  element.textContent = t(element.dataset.i18n);
});
const setBridgeStatus = (connected) => {
  document.querySelector("#bridge-dot").classList.toggle("connected", connected);
  const label = document.querySelector("#bridge-label");
  label.dataset.i18n = connected ? "connected" : "disconnected";
  label.textContent = t(label.dataset.i18n);
};
const POPUP_BRIDGE = "http://127.0.0.1:17654";
const directBridgeHealth = async (token) => {
  const value = String(token || "").trim();
  if (!value) throw new Error("not_paired");
  const controller = new AbortController();
  const timeout = setTimeout(() => controller.abort(), 4000);
  try {
    const response = await fetch(`${POPUP_BRIDGE}/v1/health`, {
      method: "GET",
      headers: { "Authorization": `Bearer ${value}` },
      cache: "no-store",
      signal: controller.signal,
    });
    if (!response.ok) throw new Error(`bridge_http_${response.status}`);
    const payload = await response.json().catch(() => ({}));
    if (payload?.ok === false) throw new Error("bridge_health_failed");
    return true;
  } finally { clearTimeout(timeout); }
};
const workerBridgeStatus = (timeoutMs = 1800) => new Promise((resolve) => {
  let settled = false;
  const finish = (value) => { if (settled) return; settled = true; resolve(value); };
  const timer = setTimeout(() => finish({ connected: false, error: "service_worker_timeout" }), timeoutMs);
  try {
    chrome.runtime.sendMessage({ type: "APOCALIPSE_BRIDGE_STATUS" }, (status) => {
      clearTimeout(timer);
      const error = chrome.runtime.lastError;
      if (error) finish({ connected: false, error: error.message || "service_worker_unavailable" });
      else finish(status || { connected: false, error: "service_worker_no_response" });
    });
  } catch (error) {
    clearTimeout(timer);
    finish({ connected: false, error: String(error) });
  }
});
const workerSelfTest = (timeoutMs = 1800) => new Promise((resolve) => {
  let settled = false;
  const finish = (value) => { if (settled) return; settled = true; resolve(value); };
  const timer = setTimeout(() => finish({ ok: false, error: "service_worker_timeout" }), timeoutMs);
  try {
    chrome.runtime.sendMessage({ type: "APOCALIPSE_WORKER_DIAGNOSTICS" }, (status) => {
      clearTimeout(timer);
      const error = chrome.runtime.lastError;
      if (error) finish({ ok: false, error: error.message || "service_worker_unavailable" });
      else finish(status || { ok: false, error: "service_worker_no_response" });
    });
  } catch (error) {
    clearTimeout(timer);
    finish({ ok: false, error: String(error) });
  }
});

const lastWorkerBootstrap = async () => {
  const stored = await chrome.storage.local.get({ workerBootstrapTrace: [] });
  const trace = Array.isArray(stored.workerBootstrapTrace) ? stored.workerBootstrapTrace : [];
  return trace.at(-1) || null;
};

const showWorkerWarning = async (worker = null) => {
  const label = document.querySelector("#bridge-label");
  const last = await lastWorkerBootstrap();
  const phase = last?.phase || "sem_rastro_de_bootstrap";
  const reason = worker?.error || last?.error || "worker_sem_resposta";
  label.removeAttribute("data-i18n");
  if (locale === "pt_BR") label.textContent = `Desktop conectado; motor de captura indisponível (${phase}: ${reason}).`;
  else if (locale === "zh_CN") label.textContent = `桌面端已连接；捕获引擎不可用 (${phase}: ${reason}).`;
  else label.textContent = `Desktop connected; capture engine unavailable (${phase}: ${reason}).`;
};

const showBridgeError = (error) => {
  const label = document.querySelector("#bridge-label");
  label.removeAttribute("data-i18n");
  const invalid = String(error).includes("401");
  if (locale === "pt_BR") label.textContent = invalid ? "Token de pareamento inválido." : "Não foi possível encontrar o Apocalipse. Mantenha o programa aberto.";
  else if (locale === "zh_CN") label.textContent = invalid ? "配对令牌无效。" : "无法连接 Apocalipse。请保持桌面程序运行。";
  else label.textContent = invalid ? "Invalid pairing token." : "Apocalipse is not reachable. Keep the desktop app open.";
};
const loadThumbnail = (image, item) => {
  const fallback = chrome.runtime.getURL("icons/alien-48.png");
  const source = item.thumbnail || (item.kind === "image" ? item.url : "");
  if (!source) {
    image.src = fallback;
    return;
  }
  image.src = source;
  image.onerror = () => {
    image.onerror = null;
    chrome.runtime.sendMessage({ type: "APOCALIPSE_FETCH_THUMBNAIL", url: source }, (result) => {
      image.src = !chrome.runtime.lastError && result?.dataUrl ? result.dataUrl : fallback;
    });
  };
};
const render = () => {
  const root = document.querySelector("#items");
  root.textContent = "";
  const matches = media.filter((item) => item.kind === selected);
  const selectable = matches.filter((item) => !item.ambiguousSocialTrack);
  const updateBulk = () => {
    const chosen = selectable.filter((item) => selectedUrls.has(item.url)).length;
    document.querySelector("#download-selected").disabled = chosen === 0;
    document.querySelector("#select-all").checked = selectable.length > 0 && chosen === selectable.length;
    document.querySelector("#select-all").indeterminate = chosen > 0 && chosen < selectable.length;
  };
  if (!matches.length) {
    const empty = document.createElement("div");
    empty.id = "empty";
    empty.textContent = t("empty");
    root.append(empty);
    return;
  }
  for (const item of matches) {
    const row = document.querySelector("#row").content.cloneNode(true);
    const image = row.querySelector("img");
    const preview = row.querySelector(".preview");
    const metadata = row.querySelector("small");
    const checkbox = row.querySelector(".media-select");
    checkbox.disabled = Boolean(item.ambiguousSocialTrack);
    checkbox.checked = selectedUrls.has(item.url);
    checkbox.onchange = () => { checkbox.checked ? selectedUrls.add(item.url) : selectedUrls.delete(item.url); updateBulk(); };
    const audio = row.querySelector(".audio-icon");
    if (selected === "audio") {
      preview.hidden = true;
      audio.hidden = false;
    } else {
      loadThumbnail(image, item);
    }
    let parsed;
    try { parsed = new URL(item.url); } catch { parsed = null; }
    const pathName = parsed?.pathname?.split("/").filter(Boolean).pop() || "";
    const extension = (pathName.match(/\.([a-z0-9]{2,8})$/i)?.[1] || item.ext || (/\.m3u8(?:$|[?#])/i.test(item.url) ? "m3u8" : item.kind)).toUpperCase();
    row.querySelector("b").textContent = item.title || decodeURIComponent(pathName) || item.url;
    metadata.textContent = [extension, formatBytes(item.size), formatDuration(item.duration), item.ambiguousSocialTrack ? t("incompleteTrack") : (item.recommended ? t("recommended") : ""), parsed?.hostname].filter(Boolean).join(" · ");
    const previewButton = row.querySelector(".external-preview");
    previewButton.textContent = t("externalPreview");
    previewButton.hidden = item.kind === "image" || item.ambiguousSocialTrack;
    previewButton.onclick = () => {
      previewButton.disabled = true;
      chrome.runtime.sendMessage({ type: "APOCALIPSE_PREVIEW_MEDIA", url: item.previewUrl || item.url, pageUrl: activePageUrl, contentType: item.contentType || null, userAgent: item.userAgent || null }, (result) => {
        previewButton.disabled = false;
        const error = chrome.runtime.lastError?.message || result?.error;
        if (!result?.ok || error) {
          const label = document.querySelector("#bridge-label");
          label.removeAttribute("data-i18n");
          const prefix = locale === "pt_BR" ? "Falha ao abrir a pr\u00e9via" : locale === "zh_CN" ? "\u65e0\u6cd5\u6253\u5f00\u9884\u89c8" : "Could not open preview";
          label.textContent = `${prefix}: ${error || "unavailable"}`;
        }
      });
    };
    const button = row.querySelector(".download-item");
    button.textContent = t("download");
    button.disabled = Boolean(item.ambiguousSocialTrack);
    button.onclick = () => chrome.runtime.sendMessage({ type: "APOCALIPSE_DOWNLOAD", item }, (result) => {
      if (result?.target === "error" || chrome.runtime.lastError) {
        showBridgeError(result?.error || chrome.runtime.lastError?.message || "unavailable");
      }
    });
    root.append(row);
  }
  updateBulk();
};
document.querySelectorAll("nav button").forEach((button) => {
  button.onclick = () => {
    selected = button.dataset.kind;
    document.querySelectorAll("nav button").forEach((item) => item.classList.toggle("active", item === button));
    render();
  };
});
document.querySelector("#select-all").onchange = (event) => {
  for (const item of media.filter((value) => value.kind === selected)) {
    if (item.ambiguousSocialTrack) continue;
    if (event.target.checked) selectedUrls.add(item.url); else selectedUrls.delete(item.url);
  }
  render();
};
document.querySelector("#download-selected").onclick = async () => {
  const button = document.querySelector("#download-selected");
  button.disabled = true;
  const items = media.filter((value) => selectedUrls.has(value.url));
  const result = await chrome.runtime.sendMessage({ type: "APOCALIPSE_DOWNLOAD_BATCH", items }).catch((error) => ({ ok: false, error: String(error) }));
  if (result?.ok) selectedUrls.clear();
  else showBridgeError(result?.error || "unavailable");
  render();
};
chrome.storage.local.get({ language: "en" }, ({ language }) => {
  locale = language;
  document.querySelector("#language").value = locale;
  translate();
  chrome.tabs.query({ active: true, currentWindow: true }, (tabs) => {
    const tab = tabs[0];
    activePageUrl = tab?.url || "";
    if (!tab?.id || !/^https?:/i.test(tab.url || "")) {
      media = [];
      render();
      return;
    }
    // Social sites contain many cross-origin iframes. Without an explicit
    // frame, Chrome may return the empty scan from an advertisement/player
    // iframe instead of the visible page.
    chrome.tabs.sendMessage(tab.id, { type: "APOCALIPSE_SCAN" }, { frameId: 0 }, async (response) => {
      const error = chrome.runtime.lastError;
      const scanned = error ? [] : (response?.media || []);
      const captured = await chrome.runtime.sendMessage({ type: "APOCALIPSE_RECENT_TAB_MEDIA", tabId: tab.id }).catch(() => null);
      const network = (captured?.media || []).map((item) => {
        const video = networkMediaKind(item) === "video";
        return {
          url: item.url,
          contentType: item.contentType || null,
          kind: video ? "video" : "audio",
          size: item.contentLength || null,
          ext: video ? "mp4" : "audio",
          title: (() => { try { return new URL(item.url).hostname.includes("tiktok") ? `TikTok — ${t("capturedResource")}` : t("capturedResource"); } catch { return t("capturedResource"); } })(),
          capturedAt: item.capturedAt,
          frameId: item.frameId,
          networkCaptured: true,
        };
      });
      media = mergeDetectedMedia(scanned, network, tab.url);
      const picker = await chrome.runtime.sendMessage({ type: "APOCALIPSE_MEDIA_PICKER_CONTEXT", tabId: tab.id }).catch(() => null);
      const requested = document.querySelector("#requested-media");
      if (picker?.context) {
        requested.hidden = false;
        requested.querySelector("b").textContent = picker.context.title || t("capturedResource");
        loadThumbnail(requested.querySelector("img"), picker.context);
      } else requested.hidden = true;
      render();
    });
  });
});
chrome.storage.local.get({ pairingToken: "" }, async ({ pairingToken }) => {
  document.querySelector("#pairing-token").value = pairingToken;
  if (!pairingToken) { setBridgeStatus(false); return; }
  try {
    await directBridgeHealth(pairingToken);
    setBridgeStatus(true);
    const worker = await workerSelfTest();
    if (!worker?.ok) await showWorkerWarning(worker);
  } catch (error) {
    setBridgeStatus(false);
    showBridgeError(String(error));
  }
});
chrome.storage.local.get({ forceShortcut: "Shift", bypassShortcut: "Alt" }, (value) => {
  document.querySelector("#force-shortcut").value = value.forceShortcut;
  document.querySelector("#bypass-shortcut").value = value.bypassShortcut;
});
const saveShortcuts = (changed) => {
  const force = document.querySelector("#force-shortcut");
  const bypass = document.querySelector("#bypass-shortcut");
  if (force.value === bypass.value) {
    if (changed === "force") bypass.value = force.value === "Alt" ? "Shift" : "Alt";
    else force.value = bypass.value === "Shift" ? "Alt" : "Shift";
  }
  chrome.storage.local.set({ forceShortcut: force.value, bypassShortcut: bypass.value });
};
document.querySelector("#force-shortcut").onchange = () => saveShortcuts("force");
document.querySelector("#bypass-shortcut").onchange = () => saveShortcuts("bypass");
setInterval(async () => {
  const { pairingToken = "" } = await chrome.storage.local.get({ pairingToken: "" });
  if (!pairingToken) { setBridgeStatus(false); return; }
  try {
    await directBridgeHealth(pairingToken);
    setBridgeStatus(true);
  } catch (error) {
    setBridgeStatus(false);
    showBridgeError(String(error));
  }
}, 5000);
document.querySelector("#connect").onclick = async () => {
  const token = document.querySelector("#pairing-token").value.trim();
  const button = document.querySelector("#connect");
  button.disabled = true;
  try {
    await directBridgeHealth(token);
    await chrome.storage.local.set({ pairingToken: token });
    setBridgeStatus(true);
    // Wake/synchronize the worker, but never make the button depend on it.
    const worker = await workerSelfTest();
    if (!worker?.ok) {
      try { chrome.runtime.sendMessage({ type: "APOCALIPSE_PAIR", token }, () => void chrome.runtime.lastError); } catch {}
      await showWorkerWarning(worker);
    }
  } catch (error) {
    setBridgeStatus(false);
    showBridgeError(String(error));
  } finally {
    button.disabled = false;
  }
};
document.querySelector("#language").onchange = (event) => {
  locale = event.target.value;
  chrome.storage.local.set({ language: locale });
  translate();
  render();
};
