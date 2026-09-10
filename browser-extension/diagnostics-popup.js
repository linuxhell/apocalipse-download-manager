(() => {
  const labels = {
    en: { start: 'Diagnose this tab (10 min)', mark: 'Mark problem now', stop: 'Stop', active: 'Detailed capture active for this tab', inactive: 'Detailed capture off', hint: 'Only the selected tab. No cookies or page text. Export the ZIP in ADM after reproducing.', failed: 'Diagnostics unavailable. Update and open ADM.' },
    pt_BR: { start: 'Diagnosticar esta aba (10 min)', mark: 'Marcar problema agora', stop: 'Encerrar', active: 'Captura detalhada ativa nesta aba', inactive: 'Captura detalhada desligada', hint: 'Somente a aba escolhida. Sem cookies ou texto da p\u00e1gina. Depois de reproduzir, exporte o ZIP no ADM.', failed: 'Diagn\u00f3stico indispon\u00edvel. Atualize e abra o ADM.' },
    zh_CN: { start: '\u8bca\u65ad\u6b64\u6807\u7b7e\u9875 (10 \u5206\u949f)', mark: '\u6807\u8bb0\u95ee\u9898', stop: '\u505c\u6b62', active: '\u8be6\u7ec6\u8bb0\u5f55\u5df2\u5f00\u542f', inactive: '\u8be6\u7ec6\u8bb0\u5f55\u5df2\u5173\u95ed', hint: '\u4ec5\u8bb0\u5f55\u9009\u5b9a\u6807\u7b7e\u9875\uff0c\u4e0d\u542b Cookie \u6216\u7f51\u9875\u6587\u672c\u3002\u91cd\u73b0\u540e\u5728 ADM \u5bfc\u51fa ZIP\u3002', failed: '\u8bca\u65ad\u4e0d\u53ef\u7528\uff0c\u8bf7\u66f4\u65b0\u5e76\u6253\u5f00 ADM\u3002' },
  };
  let language = 'en', busy = false;
  const root = document.querySelector('#logs-panel .diag-controls');
  const text = () => labels[language] || labels.en;
  function translate() {
    root.querySelectorAll('button').forEach(button => { button.textContent = text()[button.dataset.action]; });
    root.querySelector('p').textContent = text().hint;
  }
  async function status() {
    if (busy) return;
    try {
      const result = await chrome.runtime.sendMessage({ type: 'ADM_DIAG_STATUS' });
      const [tab] = await chrome.tabs.query({ active: true, currentWindow: true });
      const active = Boolean(result?.active && result.tabId === tab?.id && result.expiresAt > Date.now());
      root.querySelector('small').textContent = result?.error ? text().failed : `${active ? text().active : text().inactive} | v${chrome.runtime.getManifest().version}`;
      root.querySelector('[data-action="mark"]').disabled = !active;
      root.querySelector('[data-action="stop"]').disabled = !active;
      root.querySelector('[data-action="start"]').disabled = active;
    } catch { root.querySelector('small').textContent = text().failed; }
  }
  root.addEventListener('click', async event => {
    const button = event.target.closest('button[data-action]'); if (!button || busy) return;
    busy = true; root.querySelectorAll('button').forEach(b => { b.disabled = true; });
    try {
      const result = await chrome.runtime.sendMessage({ type: 'ADM_DIAG_CONTROL', action: button.dataset.action });
      if (result?.error) throw new Error(result.error);
      await globalThis.ADM_DIAG?.refresh();
      if (button.dataset.action === 'start' && typeof render === 'function') render();
      await globalThis.ADM_DIAG?.flush();
    } catch { root.querySelector('small').textContent = text().failed; }
    finally { busy = false; await status(); }
  });
  chrome.storage.local.get({ language: 'en' }).then(value => { language = value.language; translate(); void status(); });
  chrome.storage.onChanged.addListener((changes, area) => {
    if (area === 'local' && changes.language) { language = changes.language.newValue; translate(); }
  });
  setInterval(status, 2000);
})();
