// MAIN-world probe exposes only bounded capability counts. Page-controlled observations are untrusted.
(() => {
  document.addEventListener('adm-diagnostic-main-probe', event => {
    const video = event.target;
    if (video?.tagName !== 'VIDEO') return;
    const scopes = globalThis.ApocalipseTikTokIdentity?.scopesFor(video) || [video];
    const frameworkNodes = scopes.filter(node => Object.getOwnPropertyNames(node).some(key => /^__(?:reactProps|reactFiber|vue)/i.test(key))).length;
    video.setAttribute('data-adm-diagnostic-main', JSON.stringify({ collectorVersion: 3,
      identityInstalled: Boolean(globalThis.ApocalipseTikTokIdentity), scopeCount: scopes.length,
      frameworkNodes, trust: 'page_observation' }));
  }, true);
})();
