// Turn the one remaining visible Facebook/TikTok Blob/MediaStream placeholder
// into a canonical page-extractor item on demand, then hand the click back to
// the existing popup Preview/Download code. Resolved items stay in the popup
// catalog while the user continues scrolling the feed.
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

  const resolveItem = async (item, traceId = null) => {
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
      type: 'APOCALIPSE_RESOLVE_VISIBLE_SOCIAL_MEDIA_V2',
      traceId,
      request: {
        pageUrl: current.playerPageUrl || activePageUrl,
        playerId: current.playerId || item.playerId || null,
        playerBindingValidated: bindingSource === 'player_id_scan',
        rect: current.rect || item.rect,
        viewport: current.viewport || item.viewport || null,
        duration: current.duration || item.duration || null,
        thumbnail: current.thumbnail || item.thumbnail || '',
      },
    }, { frameId: 0 }).catch(() => null);
    if (!result?.ok || !result.item?.url) return {
      item: null, failureStage: result?.failureStage || 'content_message',
      reason: result?.reason || (result ? 'media_identity_unresolved' : 'content_message_failed'),
      platform: result?.platform || 'unknown', bindingSource,
    };
    return { item: rememberResolved(item, result.item), failureStage: 'none', reason: 'resolved',
      platform: result.platform || 'unknown', bindingSource };
  };
  const forwardClick = (item, className) => {
    const items = visibleItems();
    const index = items.findIndex(value => value.url === item.url);
    const row = visibleRows()[index];
    const button = row?.querySelector(className);
    if (button && !button.disabled) button.click();
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
    const className = isPreview ? '.external-preview' : '.download-item';
    const action = isPreview ? 'preview' : 'download';
    const traceId = globalThis.ADM_DIAG?.begin?.('popup.player_identity_click', {
      url: item.url, action, stage: 'popup_click', playerBound: Boolean(item.playerBound),
      retained: Boolean(item.retained), recommended: Boolean(item.recommended),
    }) || null;
    const outcome = await resolveItem(item, traceId);
    const resolved = outcome.item;
    void globalThis.ADM_DIAG?.emit?.('popup.player_identity_result', { resolved: Boolean(resolved),
      action, failureStage: outcome.failureStage || 'none', reason: outcome.reason || 'unknown',
      platform: outcome.platform || 'unknown', url: resolved?.url || '' }, traceId, resolved ? 'INFO' : 'WARN');
    if (!resolved) { showFailure(); button.disabled = false; syncRows(); return; }
    forwardClick(resolved, className);
  }, true);

  const root = document.querySelector('#items');
  if (root) new MutationObserver(() => queueMicrotask(syncRows)).observe(root, { childList: true, subtree: true });
  setInterval(syncRows, 700);
  queueMicrotask(syncRows);

  globalThis.ADM_POPUP_SOCIAL_PLAYER_RESOLUTION = { resolvedCatalog, aliases, resolvable, restoreResolved };
})();
