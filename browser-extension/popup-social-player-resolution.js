// Resolve the one remaining visible Facebook/TikTok player on demand.
// 0.3.133 preserves the original Preview/Download intent explicitly: after
// identity resolution we dispatch that action directly instead of re-rendering
// and synthetically clicking a new row button.
(() => {
  if (globalThis.ADM_POPUP_SOCIAL_PLAYER_RESOLUTION) return;
  const resolvedCatalog = new Map();
  const aliases = new Map();
  let syncing = false;

  const resolvable = item => Boolean(item?.kind === 'video' && item?.visualOnly && item?.playerBound
    && item?.recommended && !item?.retained && item?.rect && activeMediaTab?.id);

  const visibleRows = () => [...document.querySelectorAll('#items > article')];
  const visibleItems = () => Array.isArray(media) ? media.filter(item => item.kind === selected) : [];

  const itemForRow = row => {
    const rows = visibleRows();
    const index = rows.indexOf(row);
    return index >= 0 ? visibleItems()[index] || null : null;
  };

  const showFailure = () => {
    const label = document.querySelector('#bridge-label');
    if (!label) return;
    label.removeAttribute('data-i18n');
    label.textContent = locale === 'pt_BR'
      ? 'Não foi possível identificar com segurança este vídeo. Role um pouco e tente novamente.'
      : locale === 'zh_CN'
        ? '无法安全识别此视频。请稍微滚动后重试。'
        : 'Could not safely identify this video. Scroll a little and try again.';
  };


  const showPreviewState = result => {
    const label = document.querySelector('#bridge-label');
    if (!label) return;
    label.removeAttribute('data-i18n');
    if (result?.ok && result.preparing) {
      label.textContent = locale === 'pt_BR' ? 'Preparando a mídia completa no ADM antes de abrir o player...'
        : locale === 'zh_CN' ? 'ADM 正在准备完整媒体...' : 'ADM is preparing complete media before opening the player...';
      return;
    }
    if (!result?.ok) {
      const prefix = locale === 'pt_BR' ? 'Falha ao abrir a prévia' : locale === 'zh_CN' ? '无法打开预览' : 'Could not open preview';
      label.textContent = `${prefix}: ${result?.error || 'unavailable'}`;
    }
  };
  const directFacebookUrl = value => {
    const text = String(value || '').trim();
    const raw = text.match(/https?:\/\/[^\s<>"']+/i)?.[0] || text;
    try {
      const url = new URL(raw);
      const host = url.hostname.toLowerCase();
      if (host === 'fb.watch') return null;
      if (!(host === 'facebook.com' || host.endsWith('.facebook.com'))) return null;
      const v = url.searchParams.get('v');
      if (v && /^\d{5,}$/.test(v)) return `https://www.facebook.com/watch/?v=${v}`;
      const path = url.pathname.replace(/\/{2,}/g, '/');
      const pair = path.match(/^\/watch\/[^/?#]+\/(\d{5,})(?:\/|$)/i);
      if (pair) return `https://www.facebook.com/watch/?v=${pair[1]}`;
      const single = path.match(/^\/watch\/(\d{5,})(?:\/|$)/i);
      if (single) return `https://www.facebook.com/watch/?v=${single[1]}`;
      if (/^\/watch(?:\/|$)/i.test(path)) return null;
      if (/\/(?:reel|reels|videos|posts)\/[^/?#]+/i.test(path)) return url.href;
    } catch {}
    return null;
  };
  const facebookClipboardUrl = async value => {
    const text = String(value || '').trim();
    const raw = text.match(/https?:\/\/[^\s<>"']+/i)?.[0] || text;
    const direct = directFacebookUrl(raw);
    if (direct) return direct;
    try {
      const url = new URL(raw);
      const host = url.hostname.toLowerCase();
      const redirectable = host === 'fb.watch' || ((host === 'facebook.com' || host.endsWith('.facebook.com')) && /^\/share\/[rv]\//i.test(url.pathname));
      if (!redirectable) return null;
      const response = await fetch(url.href, { method: 'GET', redirect: 'follow', credentials: 'include', cache: 'no-store' });
      return directFacebookUrl(response.url);
    } catch { return null; }
  };
  const canonicalTikTok = value => {
    try {
      const url = new URL(String(value || '').trim());
      const match = url.pathname.match(/^\/@([A-Za-z0-9._-]+)\/video\/(\d+)\/?$/);
      return /(^|\.)tiktok\.com$/i.test(url.hostname) && match ? `https://www.tiktok.com/@${match[1]}/video/${match[2]}` : null;
    } catch { return null; }
  };
  const tiktokClipboardUrl = async value => {
    const text = String(value || '').trim();
    const raw = text.match(/https?:\/\/[^\s<>"']+/i)?.[0] || text;
    const direct = canonicalTikTok(raw);
    if (direct) return direct;
    try {
      const url = new URL(raw);
      const host = url.hostname.toLowerCase();
      const short = host === 'vm.tiktok.com' || host === 'vt.tiktok.com' || ((host === 'tiktok.com' || host.endsWith('.tiktok.com')) && /^\/t\//i.test(url.pathname));
      if (!short) return null;
      const response = await fetch(url.href, { method: 'GET', redirect: 'follow', credentials: 'omit', cache: 'no-store' });
      return canonicalTikTok(response.url);
    } catch { return null; }
  };
  const readClipboardCanonical = async (platform, before, traceId, candidate = '') => {
    let text = '';
    for (let attempt = 0; attempt < 12; attempt += 1) {
      if (attempt) await new Promise(resolve => setTimeout(resolve, 80));
      try { text = await navigator.clipboard.readText(); } catch {
        void globalThis.ADM_DIAG?.emit?.('popup.clipboard_identity', { platform, result: 'rejected', reason: 'extension_clipboard_read_failed' }, traceId, 'WARN');
        break;
      }
      if (text && (!before || text !== before)) break;
    }
    let url = platform === 'facebook' ? await facebookClipboardUrl(text) : platform === 'tiktok' ? await tiktokClipboardUrl(text) : null;
    if (!url && candidate) url = platform === 'facebook' ? await facebookClipboardUrl(candidate) : platform === 'tiktok' ? await tiktokClipboardUrl(candidate) : null;
    void globalThis.ADM_DIAG?.emit?.('popup.clipboard_identity', { platform, result: url ? 'resolved' : 'rejected',
      reason: url ? 'extension_clipboard_canonical' : text === before ? 'extension_clipboard_unchanged' : 'extension_clipboard_not_canonical', url: url || '' }, traceId, url ? 'INFO' : 'WARN');
    return url;
  };
  const dispatchResolvedAction = async (item, action, traceId) => {
    if (action === 'preview') {
      const request = previewRequestFor(item, activePageUrl);
      if (!request) return { ok: false, error: 'invalid_preview_source' };
      void globalThis.ADM_DIAG?.emit?.('popup.resolved_action_dispatch', { actionIntent: 'preview', workerRoute: 'preview_media', url: request.url }, traceId);
      const result = await chrome.runtime.sendMessage({ ...request, traceId, actionIntent: 'preview' }).catch(error => ({ ok: false, error: String(error) }));
      void globalThis.ADM_DIAG?.emit?.('popup.preview_reply', { ok: Boolean(result?.ok), preparing: Boolean(result?.preparing), actionIntent: 'preview' }, traceId, result?.ok ? 'INFO' : 'ERROR');
      showPreviewState(result);
      return result;
    }
    void globalThis.ADM_DIAG?.emit?.('popup.resolved_action_dispatch', { actionIntent: 'download', workerRoute: 'download', url: item.url }, traceId);
    const result = await chrome.runtime.sendMessage({ type: 'APOCALIPSE_DOWNLOAD', actionIntent: 'download', item: { ...manualMediaSelection(item), traceId, actionIntent: 'download' } }).catch(error => ({ ok: false, target: 'error', error: String(error) }));
    void globalThis.ADM_DIAG?.emit?.('popup.download_reply', { ok: Boolean(result?.ok), errorRef: result?.error || '', actionIntent: 'download' }, traceId, result?.ok ? 'INFO' : 'ERROR');
    if (result?.target === 'error' || !result?.ok) showBridgeError(result?.error || 'unavailable');
    return result;
  };

  const restoreResolved = () => {
    if (syncing || !Array.isArray(media)) return false;
    let changed = false;
    const aliasUrls = new Set(aliases.keys());
    const canonicalUrls = new Set(resolvedCatalog.keys());
    const filtered = media.filter(item => {
      if (aliasUrls.has(item.url)) { changed = true; return false; }
      if (canonicalUrls.has(item.url)) return false;
      return true;
    });
    for (const item of resolvedCatalog.values()) filtered.push(item);
    if (filtered.length !== media.length || changed || canonicalUrls.size) {
      const before = media.map(item => item.url).join('\n');
      const after = filtered.map(item => item.url).join('\n');
      if (before !== after) { media = filtered; changed = true; }
    }
    return changed;
  };

  const syncRows = () => {
    if (syncing) return;
    syncing = true;
    try {
      if (restoreResolved()) render();
      const items = visibleItems();
      for (const [index, row] of visibleRows().entries()) {
        const item = items[index];
        if (!resolvable(item)) continue;
        for (const button of row.querySelectorAll('.external-preview,.download-item')) {
          button.disabled = false;
          button.dataset.apocalipseResolvePlayer = '1';
          button.title = button.classList.contains('external-preview') ? t('externalPreview') : t('download');
        }
      }
    } finally { syncing = false; }
  };

  const rememberResolved = (original, resolved) => {
    const item = { ...original, ...resolved, visualOnly: false, retained: false,
      thumbnail: resolved.thumbnail || original.thumbnail || '' };
    aliases.set(original.url, item.url);
    resolvedCatalog.set(item.url, item);
    while (resolvedCatalog.size > 100) resolvedCatalog.delete(resolvedCatalog.keys().next().value);
    media = media.filter(current => current.url !== original.url && current.url !== item.url);
    media.push(item);
    render();
    return item;
  };

  const resolveItem = async (item, traceId = null, clipboardBefore = '') => {
    if (!resolvable(item)) return { item: null, failureStage: 'popup_validation', reason: 'item_not_resolvable' };
    let current = item;
    let bindingSource = 'none';
    if (item.playerId) {
      const scan = await chrome.tabs.sendMessage(activeMediaTab.id, { type: 'APOCALIPSE_SCAN' }, { frameId: 0 }).catch(() => null);
      const fresh = Array.isArray(scan?.media)
        ? scan.media.find(candidate => candidate?.kind === 'video' && candidate.playerId === item.playerId && !candidate.retained)
        : null;
      if (!fresh) {
        void globalThis.ADM_DIAG?.emit?.('popup.player_binding_validation', {
          result: 'rejected', reason: 'player_id_stale', bindingSource: 'player_id_scan', playerId: item.playerId,
        }, traceId, 'WARN');
        return { item: null, failureStage: 'player_binding', reason: 'player_id_stale', platform: 'unknown' };
      }
      current = { ...item, ...fresh, thumbnail: fresh.thumbnail || item.thumbnail || '' };
      bindingSource = 'player_id_scan';
      void globalThis.ADM_DIAG?.emit?.('popup.player_binding_validation', {
        result: 'matched', reason: fresh.visualOnly ? 'player_id_live_visual' : 'player_id_live_canonical',
        bindingSource, playerId: item.playerId, visualOnly: Boolean(fresh.visualOnly), pageExtractor: Boolean(fresh.pageExtractor),
        url: fresh.url || '',
      }, traceId);
      if (!fresh.visualOnly && fresh.pageExtractor && /^https?:/i.test(fresh.url || '')) {
        return { item: rememberResolved(item, current), failureStage: 'none', reason: 'player_id_scan_canonical',
          platform: 'unknown', bindingSource };
      }
    }
    const result = await chrome.tabs.sendMessage(activeMediaTab.id, {
      type: 'APOCALIPSE_RESOLVE_VISIBLE_SOCIAL_MEDIA_V3',
      traceId,
      request: {
        pageUrl: current.playerPageUrl || activePageUrl,
        playerId: current.playerId || item.playerId || null,
        playerBindingValidated: bindingSource === 'player_id_scan',
        rect: current.rect || item.rect,
        viewport: current.viewport || item.viewport || null,
        duration: current.duration || item.duration || null,
        previewUrl: current.previewUrl || item.previewUrl || null,
        thumbnail: current.thumbnail || item.thumbnail || '',
      },
    }, { frameId: 0 }).catch(() => null);
    if (result?.needsClipboard) {
      const url = await readClipboardCanonical(result.platform, clipboardBefore, traceId, result.clipboardCandidate || '');
      if (url) {
        const resolved = { ...current, url, extractorUrl: url, kind: 'video', pageExtractor: true, visualOnly: false,
          recommended: true, retained: false, ambiguousSocialTrack: false, thumbnail: current.thumbnail || item.thumbnail || '' };
        return { item: rememberResolved(item, resolved), failureStage: 'none', reason: 'extension_clipboard_resolved',
          platform: result.platform || 'unknown', bindingSource };
      }
      return { item: null, failureStage: 'clipboard_read', reason: 'extension_clipboard_unresolved',
        platform: result.platform || 'unknown', bindingSource };
    }
    if (!result?.ok || !result.item?.url) return {
      item: null, failureStage: result?.failureStage || 'content_message',
      reason: result?.reason || (result ? 'media_identity_unresolved' : 'content_message_failed'),
      platform: result?.platform || 'unknown', bindingSource,
    };
    return { item: rememberResolved(item, result.item), failureStage: 'none', reason: 'resolved',
      platform: result.platform || 'unknown', bindingSource };
  };

  document.addEventListener('click', async event => {
    const button = event.target?.closest?.('#items .external-preview,#items .download-item');
    if (!button || button.dataset.apocalipseResolvePlayer !== '1') return;
    const row = button.closest('article');
    const item = itemForRow(row);
    if (!resolvable(item)) return;
    event.preventDefault();
    event.stopImmediatePropagation();
    button.disabled = true;
    const isPreview = button.classList.contains('external-preview');
    const action = isPreview ? 'preview' : 'download';
    const traceId = globalThis.ADM_DIAG?.begin?.('popup.player_identity_click', {
      url: item.url, action, actionIntent: action, stage: 'popup_click', playerBound: Boolean(item.playerBound),
      retained: Boolean(item.retained), recommended: Boolean(item.recommended),
    }) || null;
    // Copy Link is an internal identity probe. Tell ADM to ignore the temporary
    // clipboard change so a Preview can never open the save-location dialog.
    if (action === 'preview') {
      await chrome.runtime.sendMessage({ type: 'APOCALIPSE_PREVIEW_IDENTITY_BEGIN', traceId, actionIntent: 'preview' }).catch(() => null);
    }
    let clipboardBefore = '';
    try { clipboardBefore = await navigator.clipboard.readText(); } catch {}
    const outcome = await resolveItem(item, traceId, clipboardBefore);
    const resolved = outcome.item;
    void globalThis.ADM_DIAG?.emit?.('popup.player_identity_result', { resolved: Boolean(resolved),
      action, actionIntent: action, failureStage: outcome.failureStage || 'none', reason: outcome.reason || 'unknown',
      platform: outcome.platform || 'unknown', url: resolved?.url || '' }, traceId, resolved ? 'INFO' : 'WARN');
    if (!resolved) { showFailure(); button.disabled = false; syncRows(); return; }
    const result = await dispatchResolvedAction(resolved, action, traceId);
    if (!result?.ok) { button.disabled = false; syncRows(); }
  }, true);

  const root = document.querySelector('#items');
  if (root) new MutationObserver(() => queueMicrotask(syncRows)).observe(root, { childList: true, subtree: true });
  setInterval(syncRows, 700);
  queueMicrotask(syncRows);

  globalThis.ADM_POPUP_SOCIAL_PLAYER_RESOLUTION = { resolvedCatalog, aliases, resolvable, restoreResolved };
})();
