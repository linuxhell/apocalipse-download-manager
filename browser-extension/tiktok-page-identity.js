// Runs in MAIN, with no extension APIs. The shared resolver is card/source-bound.
(() => {
  const identity = globalThis.ApocalipseTikTokIdentity;
  document.addEventListener('apocalipse-tiktok-identity-request', event => {
    const video = event.target;
    if (video?.tagName !== 'VIDEO' || !video.isConnected) return;
    const source = String(video.currentSrc || video.src || '');
    const url = identity.resolveLocal(video, false);
    if (url && source === String(video.currentSrc || video.src || '')) {
      video.setAttribute('data-apocalipse-current-permalink', url);
    }
  }, true);
})();
