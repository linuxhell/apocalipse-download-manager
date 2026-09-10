(() => {
  const D = globalThis.ApocalipseDiagnostics;
  if (!D) return;
  let lastSnapshot = '', lastAt = 0;
  const frame = () => window === window.top ? 'top' : 'child';
  const inScope = async () => {
    if (!D.active()) return false;
    const result = await chrome.runtime.sendMessage({ type: 'APOCALIPSE_DIAGNOSTIC_SCOPE' }).catch(() => null);
    return Boolean(result?.active);
  };
  async function snapshot(force = false) {
    if (!(await inScope())) return;
    const videos = [...document.querySelectorAll('video')].slice(0, 12);
    const states = videos.map(video => {
      const rect = video.getBoundingClientRect();
      video.removeAttribute('data-adm-diagnostic-main');
      video.dispatchEvent(new Event('adm-diagnostic-main-probe', { bubbles: true }));
      const main = video.getAttribute('data-adm-diagnostic-main');
      video.removeAttribute('data-adm-diagnostic-main');
      let mainProbe = null;
      try { mainProbe = main ? JSON.parse(main) : null; } catch {}
      return { ...D.player(video), mainWorldResponded: Boolean(main), mainProbe,
        visible: rect.width > 40 && rect.height > 30 && rect.bottom > 0 && rect.top < innerHeight,
        isolatedIdentityInstalled: Boolean(globalThis.ApocalipseTikTokIdentity),
        tikTokHandlerInstalled: Boolean(globalThis.ApocalipseTikTokClickDiagnostics),
        scopedPermalinkCount: globalThis.ApocalipseTikTokIdentity?.scopesFor(video).reduce((n, scope) =>
          n + (scope.querySelectorAll?.('a[href*="/video/"]').length || 0), 0) || 0 };
    });
    const serialized = JSON.stringify(states);
    if (!force && serialized === lastSnapshot && Date.now() - lastAt < 10000) return;
    lastSnapshot = serialized; lastAt = Date.now();
    await D.emit('player.snapshot', { frame: frame(), videoCount: videos.length, players: states,
      iframeCount: document.querySelectorAll('iframe').length, resourceCount: performance.getEntriesByType('resource').length }, null, 'DEBUG');
    const resources = performance.getEntriesByType('resource').filter(entry =>
      /\.(?:mp4|webm|mpd|m3u8)(?:$|[?#])|\/video\/tos\/|\/aweme\/v1\/(?:play|download)\//i.test(entry.name) || entry.initiatorType === 'video').slice(-24);
    if (force) await D.emit('network.performance_snapshot', { entries: resources.map(entry => ({ url: entry.name,
      initiatorType: entry.initiatorType, transferSize: entry.transferSize, encodedBodySize: entry.encodedBodySize,
      startTime: entry.startTime, duration: entry.duration })), note: 'resource_timing_is_not_a_downloadable_source_proof' }, null, 'DEBUG');
  }
  window.addEventListener('click', event => {
    const button = event.target?.closest?.('.apocalipse-media-download');
    if (!button) return;
    const id = D.beginAction(button);
    const video = globalThis.ApocalipseTikTokIdentity?.videoFor(button);
    void D.emit('overlay.click_observed', { frame: frame(), hasBoundPlayer: Boolean(video),
      player: D.player(video), isRecordingButton: button.classList.contains('apocalipse-media-record'),
      disabled: button.disabled, tikTokHandlerInstalled: Boolean(globalThis.ApocalipseTikTokClickDiagnostics) }, id);
  }, true);
  for (const event of ['loadedmetadata', 'emptied', 'error']) {
    document.addEventListener(event, e => {
      if (e.target?.tagName === 'VIDEO' && D.active()) void snapshot(true);
    }, true);
  }
  chrome.runtime.onMessage.addListener((message, _sender, reply) => {
    if (message?.type !== 'APOCALIPSE_DIAGNOSTIC_SNAPSHOT') return;
    snapshot(true).then(() => reply({ ok: true, collectorVersion: 3 }), () => reply({ ok: false })); return true;
  });
  window.addEventListener('error', event => {
    if (String(event.filename || '').startsWith(chrome.runtime.getURL(''))) {
      void D.emit('collector.javascript_error', { message: event.message, line: event.lineno, column: event.colno,
        script: String(event.filename).split('/').pop() }, null, 'ERROR');
    }
  });
  setInterval(() => { if (D.active()) void snapshot(); }, 2000);
  void D.emit('collector.content_ready', { frame: frame(), script: 'diagnostics-content.js',
    isolatedWorld: true });
})();
