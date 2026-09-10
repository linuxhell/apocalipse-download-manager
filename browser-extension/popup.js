globalThis.ADM_DIAG?.register("popup.js");
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
  en: { mediaIntelligence: "Media intelligence", video: "Video", audio: "Audio", images: "Images", download: "Download", externalPreview: "Open in player", incompleteTrack: "Unverified media", empty: "No media detected in this tab.", unknownSize: "Size unavailable", connected: "Connected to Apocalipse", disconnected: "Disconnected", pairingToken: "Pairing token", connect: "Connect", recommended: "Recommended", capturedResource: "Captured media resource", requestedMedia: "You tried to download", selectAll: "Select all", downloadSelected: "Download selected", forceShortcut: "Force Apocalipse", bypassShortcut: "Bypass Apocalipse" },
  pt_BR: { mediaIntelligence: "Inteligência de mídia", video: "Vídeo", audio: "Áudio", images: "Imagens", download: "Download", externalPreview: "Abrir no player", incompleteTrack: "Mídia não verificada", empty: "Nenhuma mídia detectada nesta aba.", unknownSize: "Tamanho indisponível", connected: "Conectada ao Apocalipse", disconnected: "Desconectada", pairingToken: "Token de pareamento", connect: "Conectar", recommended: "Recomendada", capturedResource: "Recurso de mídia capturado", requestedMedia: "Você tentou baixar", selectAll: "Selecionar todos", downloadSelected: "Baixar selecionados", forceShortcut: "Forçar Apocalipse", bypassShortcut: "Ignorar Apocalipse" },
  zh_CN: { mediaIntelligence: "媒体智能", video: "视频", audio: "音频", images: "图片", download: "下载", externalPreview: "在播放器中打开", incompleteTrack: "不完整音视频轨道", empty: "此标签页未检测到媒体。", unknownSize: "大小未知", connected: "已连接到 Apocalipse", disconnected: "未连接", pairingToken: "配对令牌", connect: "连接", recommended: "推荐", capturedResource: "已捕获的媒体资源", requestedMedia: "您尝试下载", selectAll: "全选", downloadSelected: "下载所选项目", forceShortcut: "强制使用 Apocalipse", bypassShortcut: "绕过 Apocalipse" }
};
const t = (key) => messages[locale]?.[key] || messages.en[key] || key;
const formatBytes = (bytes) => {
  if (!Number.isFinite(bytes) || bytes <= 0) return t("unknownSize");
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
// Preview and Download start from the SAME row identity. previewUrl is only a
// thumbnail/player hint and can be a partial track, blob, or a generic feed URL.
function previewRequestFor(item, pageUrl) {
  const url = item.extractorUrl || item.url;
  let parsed;
  try { parsed = new URL(url); } catch { return null; }
  if (!/^https?:$/.test(parsed.protocol) || parsed.username || parsed.password) return null;
  const pageExtractor = Boolean(item.extractorUrl || item.pageExtractor);
  if (item.ambiguousSocialTrack && item.kind !== "audio" && !pageExtractor && !item.audioUrl) return null;
  return { type: "APOCALIPSE_PREVIEW_MEDIA", url, pageUrl,
    audioUrl: pageExtractor ? null : item.audioUrl || null,
    mediaKind: item.kind, pageExtractor,
    contentType: pageExtractor ? null : item.contentType || null,
    userAgent: item.userAgent || null };
}

const extraLabels = {
  en: { analyze: 'Analyze media', unresolved: 'This resource has not been linked to a complete video. No download was sent. Refresh the page with diagnostics active.', unavailable: 'No response from the extension. Use Copy extension diagnostics above; verify pairing in ADM.', refresh: 'Refresh media', scanFailed: 'Part of the media scan failed. Available results are shown.', skipped: 'Unverified items were not sent.', sent: 'Request received by ADM.', preparing: 'ADM is preparing the complete media before opening the player.' },
  pt_BR: { analyze: 'Analisar m\u00eddia', unresolved: 'Este recurso ainda n\u00e3o foi associado a um v\u00eddeo completo. Nenhum download foi enviado. Recarregue a p\u00e1gina com o diagn\u00f3stico ativo.', unavailable: 'A extens\u00e3o n\u00e3o respondeu. Use Copiar diagn\u00f3stico da extens\u00e3o acima e confira o pareamento no ADM.', refresh: 'Atualizar m\u00eddias', scanFailed: 'Uma etapa da busca falhou. Os resultados dispon\u00edveis est\u00e3o sendo exibidos.', skipped: 'Itens n\u00e3o verificados n\u00e3o foram enviados.', sent: 'Solicita\u00e7\u00e3o recebida pelo ADM.', preparing: 'O ADM est\u00e1 preparando a m\u00eddia completa antes de abrir o player.' },
  zh_CN: { analyze: '\u5206\u6790\u5a92\u4f53', unresolved: '\u6b64\u8d44\u6e90\u5c1a\u672a\u5173\u8054\u5230\u5b8c\u6574\u89c6\u9891\uff0c\u672a\u53d1\u9001\u4e0b\u8f7d\u3002', unavailable: '\u6269\u5c55\u672a\u54cd\u5e94\uff0c\u8bf7\u590d\u5236\u6269\u5c55\u8bca\u65ad\u5e76\u68c0\u67e5\u914d\u5bf9\u3002', refresh: '\u5237\u65b0\u5a92\u4f53', scanFailed: '\u90e8\u5206\u626b\u63cf\u5931\u8d25\uff0c\u663e\u793a\u53ef\u7528\u7ed3\u679c\u3002', skipped: '\u672a\u9a8c\u8bc1\u7684\u9879\u76ee\u672a\u53d1\u9001\u3002', sent: 'ADM \u5df2\u6536\u5230\u8bf7\u6c42\u3002', preparing: 'ADM \u6b63\u5728\u51c6\u5907\u5b8c\u6574\u5a92\u4f53\u3002' },
};
const label = key => (extraLabels[locale] || extraLabels.en)[key];
const notice = document.createElement('p'); notice.id = 'popup-notice'; notice.setAttribute('role', 'status'); notice.hidden = true;
document.querySelector('#bulk-actions').after(notice);
function showNotice(text) { notice.textContent = text; notice.hidden = false; }
const refreshButton = document.createElement('button'); refreshButton.id = 'refresh-media'; refreshButton.type = 'button'; refreshButton.textContent = label('refresh');
document.querySelector('nav').after(refreshButton);
let tabId = null, revision = 0, scanning = false, uiBusy = 0, lastInventory = '';
const usable = item => Boolean(previewRequestFor(item, activePageUrl)) && !item.ambiguousSocialTrack;
const record = (event, detail = {}, trace = null, level = 'INFO') => {
  try { void globalThis.ADM_DIAG?.emit(event, detail, trace, level); } catch {}
};
function updateBulk() {
  const matches = media.filter(item => item.kind === selected);
  const count = matches.filter(item => selectedUrls.has(item.url)).length;
  document.querySelector('#download-selected').disabled = count === 0 || uiBusy > 0;
  const all = document.querySelector('#select-all');
  all.disabled = matches.length === 0;
  all.checked = matches.length > 0 && count === matches.length;
  all.indeterminate = count > 0 && count < matches.length;
}
async function transfer(button, item, preview) {
  if (uiBusy) return;
  uiBusy++; button.disabled = true; updateBulk();
  try {
    let current = item;
    if (!usable(current)) {
      await refreshMedia();
      // A refresh may only upgrade THIS identity. Never substitute the newly
      // visible reel for a stale/ambiguous row from a different resource.
      current = media.find(next => usable(next) && (next.url === item.url || next.previewUrl === item.url));
      if (!current) { showNotice(label('unresolved')); record('popup.unresolved_action', { reason: 'no_bound_complete_media' }, null, 'WARN'); return; }
    }
    const traceId = globalThis.ADM_DIAG?.begin(preview ? 'popup.preview_clicked' : 'popup.download_clicked', { url: current.url, kind: current.kind }) || crypto.randomUUID();
    const message = preview ? { ...previewRequestFor(current, activePageUrl), traceId }
      : { type: 'APOCALIPSE_DOWNLOAD', item: { ...current, traceId } };
    const result = await ADM_POPUP.runtime(message);
    record('popup.action_reply', { ok: result?.ok === true }, traceId, result?.ok ? 'INFO' : 'ERROR');
    if (!result?.ok) {
      if (/not_paired|401|bridge_http|Failed to fetch/i.test(result?.error || '')) showBridgeError(result?.error);
      showNotice(result?.error === 'incomplete_social_media_track' ? label('unresolved') : label('unavailable'));
    } else showNotice(result.preparing ? label('preparing') : label('sent'));
  } catch { showNotice(label('unavailable')); }
  finally { button.disabled = false; uiBusy--; updateBulk(); }
}
const render = () => {
  const root = document.querySelector('#items'); root.textContent = '';
  const matches = media.filter(item => item.kind === selected);
  record('popup.final_inventory', { count: media.length, displayed: matches.length,
    unresolved: matches.filter(item => !usable(item)).length, kind: selected });
  if (!matches.length) {
    const empty = document.createElement('div'); empty.id = 'empty'; empty.textContent = t('empty'); root.append(empty);
  }
  for (const item of matches) {
    try {
      const row = document.querySelector('#row').content.cloneNode(true);
      const image = row.querySelector('img'), preview = row.querySelector('.preview'), audio = row.querySelector('.audio-icon');
      const checkbox = row.querySelector('.media-select');
      checkbox.checked = selectedUrls.has(item.url);
      checkbox.onchange = () => { checkbox.checked ? selectedUrls.add(item.url) : selectedUrls.delete(item.url); updateBulk(); };
      if (item.kind === 'audio') { preview.hidden = true; audio.hidden = false; } else loadThumbnail(image, item);
      let parsed; try { parsed = new URL(item.url); } catch {}
      const pathName = parsed?.pathname?.split('/').filter(Boolean).pop() || '';
      const extension = String(item.ext || pathName.match(/\.([a-z0-9]{2,8})$/i)?.[1] || item.kind).toUpperCase();
      row.querySelector('b').textContent = item.title || ADM_POPUP.safeDecode(pathName) || item.url;
      row.querySelector('small').textContent = [extension, formatBytes(item.size), formatDuration(item.duration), !usable(item) ? t('incompleteTrack') : item.recommended ? t('recommended') : '', parsed?.hostname].filter(Boolean).join(' \u00b7 ');
      const previewButton = row.querySelector('.external-preview');
      previewButton.textContent = t('externalPreview'); previewButton.hidden = item.kind === 'image';
      previewButton.title = usable(item) ? t('externalPreview') : label('unresolved');
      previewButton.onclick = () => transfer(previewButton, item, true);
      const download = row.querySelector('.download-item');
      download.textContent = usable(item) ? t('download') : label('analyze');
      download.title = usable(item) ? t('download') : label('unresolved');
      download.onclick = () => transfer(download, item, false);
      record('popup.row_state', { url: item.url, kind: item.kind, previewEnabled: !previewButton.hidden,
        downloadEnabled: true, reason: usable(item) ? 'valid_selection' : 'analysis_required' });
      root.append(row);
    } catch (error) { ADM_POPUP.record('row_render', error); record('popup.row_error', { errorName: error?.name || 'Error' }, null, 'ERROR'); }
  }
  updateBulk();
};
function networkItems(items) {
  return items.map(item => ({ url: item.url, contentType: item.contentType, capturedAt: item.capturedAt,
    frameId: item.frameId, kind: networkMediaKind(item), size: item.contentLength || null,
    title: t('capturedResource'), networkCaptured: true }));
}
async function refreshMedia() {
  if (!Number.isInteger(tabId)) return;
  const mine = ++revision; scanning = true;
  let dom = [], network = [], anySuccess = false;
  function apply(final = false) {
    if (mine !== revision || (!final && lastInventory)) return;
    const merged = mergeDetectedMedia(dom, network, activePageUrl);
    const signature = JSON.stringify(merged);
    if (signature === lastInventory) return;
    lastInventory = signature; media = merged;
    if (final) for (const value of selectedUrls) if (!media.some(item => item.url === value)) selectedUrls.delete(value);
    render(); record('popup.merged_inventory', { domCount: dom.length, networkCount: network.length, mergedCount: media.length });
  }
  const scan = /(^|\.)tiktok\.com$/i.test(new URL(activePageUrl).hostname) && globalThis.ADM_TIKTOK_SCAN
    ? globalThis.ADM_TIKTOK_SCAN(tabId)
    : ADM_POPUP.tab(tabId, { type: 'APOCALIPSE_SCAN_FAST' }, { frameId: 0 }, 4000).then(reply => {
        if (reply.error) throw new Error(reply.error); return reply.media || [];
      });
  await Promise.allSettled([
    scan.then(items => { dom = items; anySuccess = true; apply(); }).catch(error => { ADM_POPUP.record('dom_scan', error); if (mine === revision) showNotice(label('scanFailed')); }),
    ADM_POPUP.runtime({ type: 'APOCALIPSE_RECENT_TAB_MEDIA', tabId }, 4000)
      .then(reply => { network = networkItems(reply.media || []); anySuccess = true; apply(); })
      .catch(error => { ADM_POPUP.record('network_scan', error); if (mine === revision) showNotice(label('scanFailed')); }),
  ]);
  if (mine === revision) { apply(true); scanning = false; if (!anySuccess) showNotice(label('unavailable')); }
}
// Install UI handlers before any optional data retrieval.
document.querySelectorAll('nav button').forEach(button => {
  button.onclick = () => {
    selected = button.dataset.kind;
    document.querySelectorAll('nav button').forEach(item => item.classList.toggle('active', item === button));
    render();
  };
});
document.querySelector('#select-all').onchange = event => {
  for (const item of media.filter(value => value.kind === selected)) event.target.checked ? selectedUrls.add(item.url) : selectedUrls.delete(item.url);
  render();
};
document.querySelector('#download-selected').onclick = async () => {
  if (uiBusy) return;
  uiBusy++; updateBulk();
  try {
    const chosen = media.filter(value => selectedUrls.has(value.url));
    const items = chosen.filter(usable);
    if (!items.length) { showNotice(label('unresolved')); return; }
    const result = await ADM_POPUP.runtime({ type: 'APOCALIPSE_DOWNLOAD_BATCH', items }, 12000);
    if (!result?.ok) { showNotice(label('unavailable')); return; }
    for (const item of items) selectedUrls.delete(item.url);
    showNotice(chosen.length > items.length ? label('skipped') : label('sent'));
  } catch { showNotice(label('unavailable')); }
  finally { uiBusy--; render(); }
};
refreshButton.onclick = async () => {
  refreshButton.disabled = true;
  try { await refreshMedia(); } finally { refreshButton.disabled = false; }
};
const saveShortcuts = changed => {
  const force = document.querySelector('#force-shortcut'), bypass = document.querySelector('#bypass-shortcut');
  if (force.value === bypass.value) {
    if (changed === 'force') bypass.value = force.value === 'Alt' ? 'Shift' : 'Alt';
    else force.value = bypass.value === 'Shift' ? 'Alt' : 'Shift';
  }
  void chrome.storage.local.set({ forceShortcut: force.value, bypassShortcut: bypass.value }).catch(() => showNotice(label('unavailable')));
};
document.querySelector('#force-shortcut').onchange = () => saveShortcuts('force');
document.querySelector('#bypass-shortcut').onchange = () => saveShortcuts('bypass');
document.querySelector('#language').onchange = event => {
  locale = event.target.value;
  void chrome.storage.local.set({ language: locale });
  translate(); refreshButton.textContent = label('refresh'); render();
};
document.querySelector('#connect').onclick = async () => {
  const button = document.querySelector('#connect'), token = document.querySelector('#pairing-token').value.trim();
  button.disabled = true;
  try {
    await directBridgeHealth(token); await chrome.storage.local.set({ pairingToken: token }); setBridgeStatus(true);
    const worker = await workerSelfTest();
    if (!worker?.ok) { await showWorkerWarning(worker); showNotice(label('unavailable')); }
  } catch (error) { setBridgeStatus(false); showBridgeError(String(error)); }
  finally { button.disabled = false; }
};
void (async () => {
  try {
    const values = await chrome.storage.local.get({ language: 'en', pairingToken: '', forceShortcut: 'Shift', bypassShortcut: 'Alt' });
    locale = values.language; document.querySelector('#language').value = locale;
    document.querySelector('#pairing-token').value = values.pairingToken;
    document.querySelector('#force-shortcut').value = values.forceShortcut;
    document.querySelector('#bypass-shortcut').value = values.bypassShortcut;
    translate(); refreshButton.textContent = label('refresh'); render();
    const [tab] = await chrome.tabs.query({ active: true, currentWindow: true });
    if (Number.isInteger(tab?.id) && /^https?:/i.test(tab.url || '')) { tabId = tab.id; activePageUrl = tab.url; void refreshMedia(); }
    if (values.pairingToken) {
      try { await directBridgeHealth(values.pairingToken); setBridgeStatus(true); }
      catch (error) { setBridgeStatus(false); showBridgeError(String(error)); }
    }
  } catch (error) { ADM_POPUP.record('initialization', error); showNotice(label('unavailable')); }
})();
setInterval(() => { if (!document.hidden && !scanning && !uiBusy) void refreshMedia(); }, 2000);
ADM_POPUP.record('popup_handlers_ready');
