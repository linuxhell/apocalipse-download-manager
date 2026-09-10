(() => {
  const D = globalThis.ApocalipseDiagnostics;
  const labels = {
    en: ['Diagnostic v3', 'Capture this tab (5 min)', 'Mark problem now', 'Stop capture', 'Idle', 'Capturing', 'Pending', 'Dropped', 'Delivery errors', 'Wait for Pending = 0 before exporting from ADM.'],
    pt_BR: ['Diagnostico v3', 'Capturar esta aba (5 min)', 'Marcar problema agora', 'Encerrar captura', 'Inativo', 'Capturando', 'Pendentes', 'Descartados', 'Falhas de envio', 'Antes de exportar no ADM, confira Pendentes = 0.'],
    zh_CN: ['Diagnostic v3', 'Capture tab (5 min)', 'Mark problem', 'Stop', 'Idle', 'Capturing', 'Pending', 'Dropped', 'Delivery errors', 'Wait for Pending = 0 before exporting from ADM.'],
  };
  const root = document.createElement('section'); root.id = 'diagnostic-v3';
  const heading = document.createElement('b'), controls = document.createElement('div'), status = document.createElement('p');
  const buttons = ['start', 'mark', 'stop'].map(command => {
    const b = document.createElement('button'); b.type = 'button'; b.dataset.command = command;
    b.onclick = async () => {
      b.disabled = true;
      try {
        const result = await chrome.runtime.sendMessage({ type: 'APOCALIPSE_DIAGNOSTIC_CONTROL', command });
        if (!result?.ok) throw new Error(result?.error || 'diagnostic_no_response');
        await refresh();
      } catch (error) { status.textContent = String(error); }
      finally { b.disabled = false; }
    };
    controls.append(b); return b;
  });
  root.append(heading, controls, status); document.querySelector('nav').before(root);
  async function refresh() {
    const stored = await chrome.storage.local.get({ language: 'en' });
    const text = labels[stored.language] || labels.en;
    heading.textContent = `${text[0]} - ${chrome.runtime.getManifest().version}`;
    buttons.forEach((button, i) => { button.textContent = text[i + 1]; });
    const result = await chrome.runtime.sendMessage({ type: 'APOCALIPSE_DIAGNOSTIC_STATUS' });
    if (!result) { status.textContent = 'Diagnostic collector unavailable'; return; }
    const seconds = result.active ? Math.max(0, Math.ceil((result.session.expiresAt - Date.now()) / 1000)) : 0;
    status.textContent = `${result.active ? `${text[5]} (${seconds}s)` : text[4]} | ${text[6]}: ${result.pending} | ${text[7]}: ${result.dropped} | ${text[8]}: ${result.deliveryErrors}${result.lastDeliveryError ? ` (${result.lastDeliveryError})` : ""}. ${text[9]}`;
    root.dataset.active = String(result.active);
  }
  void refresh().catch(() => {});
  setInterval(() => void refresh().catch(() => {}), 1500);
  void D?.emit('collector.popup_ready', { script: 'diagnostics-popup.js' }, null, 'INFO', 'extension.popup');
})();
