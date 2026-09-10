// A small independent bootstrap: a broken collector/worker must not lock the UI.
(() => {
  const failures = [], phases = new Set(), callbacks = new Set();
  const version = chrome.runtime.getManifest().version;
  function record(phase, error = null) {
    phases.add(phase);
    if (error) {
      failures.push({ phase, name: String(error.name || 'Error').slice(0, 60),
        code: String(error.code || 'operation_failed').replace(/[^a-z0-9_.-]/gi, '_').slice(0, 100) });
      if (failures.length > 30) failures.shift();
    }
  }
  function bounded(invoke, ms = 6000) {
    return new Promise((resolve, reject) => {
      let done = false;
      const finish = (value, error) => {
        if (done) return;
        done = true; clearTimeout(timer);
        if (error) { record('ipc', error); reject(error); } else resolve(value);
      };
      const problem = code => Object.assign(new Error(code), { code });
      const timer = setTimeout(() => finish(null, problem('response_timeout')), ms);
      try {
        invoke(value => {
          const error = chrome.runtime.lastError;
          finish(value, error ? problem('message_channel_unavailable') : value == null ? problem('empty_response') : null);
        });
      } catch { finish(null, problem('extension_context_unavailable')); }
    });
  }
  const runtime = (message, ms) => bounded(reply => chrome.runtime.sendMessage(message, reply), ms);
  const tab = (tabId, message, options = { frameId: 0 }, ms) => bounded(reply => chrome.tabs.sendMessage(tabId, message, options, reply), ms);
  async function deadline(promise, ms = 4000) {
    let timer;
    try {
      return await Promise.race([promise, new Promise((_, reject) => {
        timer = setTimeout(() => reject(Object.assign(new Error('response_timeout'), { code: 'response_timeout' })), ms);
      })]);
    } finally { clearTimeout(timer); }
  }
  globalThis.ADM_POPUP = { runtime, tab, deadline, record, version,
    safeDecode: value => { try { return decodeURIComponent(value); } catch { return value; } },
    onChange: fn => callbacks.add(fn), changed: () => callbacks.forEach(fn => fn()),
  };
  if (typeof document === 'undefined') return;
  addEventListener('error', event => record('script_error', { name: event.error?.name || 'Error', code: `line_${event.lineno || 0}` }));
  addEventListener('unhandledrejection', event => record('unhandled_rejection', { name: event.reason?.name || 'Error' }));
  const status = document.createElement('section'); status.className = 'popup-recovery';
  const stamp = document.createElement('span'); stamp.textContent = `ADM ${version}`;
  const button = document.createElement('button'); button.type = 'button'; button.id = 'copy-extension-report';
  const feedback = document.createElement('span'); feedback.setAttribute('role', 'status');
  status.append(stamp, button, feedback); document.querySelector('header').after(status);
  let language = 'en';
  const label = () => { button.textContent = language === 'pt_BR' ? 'Copiar diagn\u00f3stico da extens\u00e3o' : language === 'zh_CN' ? '\u590d\u5236\u6269\u5c55\u8bca\u65ad' : 'Copy extension diagnostics'; };
  label();
  chrome.storage.local.get({ language: 'en' }).then(v => { language = v.language; label(); }).catch(() => {});
  chrome.storage.onChanged.addListener((changes, area) => {
    if (area === 'local' && changes.language) { language = changes.language.newValue; label(); }
  });
  button.onclick = async () => {
    button.disabled = true;
    try {
      const worker = await runtime({ type: 'APOCALIPSE_WORKER_PING' }, 1800).catch(e => ({ ok: false, code: e.code }));
      const saved = await deadline(chrome.storage.local.get({ pairingToken: '' }), 1800).catch(() => ({}));
      const controls = [...document.querySelectorAll('button,input[type="checkbox"],select')].map(el => ({
        id: el.id || el.className || el.tagName, disabled: el.disabled, hidden: el.hidden,
      })).slice(0, 60);
      const report = JSON.stringify({ format: 'adm-extension-recovery-v1', extensionVersion: version,
        pairedTokenPresent: Boolean(saved.pairingToken), worker: { ok: worker?.ok === true, version: worker?.version || null, code: worker?.code || null },
        phases: [...phases], failures, controls }, null, 2);
      try {
        await navigator.clipboard.writeText(report);
        feedback.textContent = language === 'pt_BR' ? 'Copiado.' : 'Copied.';
      } catch {
        let area = document.querySelector('#extension-report-text');
        if (!area) { area = document.createElement('textarea'); area.id = 'extension-report-text'; area.readOnly = true; status.after(area); }
        area.value = report; area.focus(); area.select();
        feedback.textContent = language === 'pt_BR' ? 'Copie o texto selecionado.' : 'Copy the selected text.';
      }
    } finally { button.disabled = false; }
  };
  record('bootstrap_ready');
})();
