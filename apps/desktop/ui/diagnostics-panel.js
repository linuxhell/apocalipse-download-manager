(() => {
  const words = {
    en: { start:'Start desktop diagnostics (10 min)', mark:'Mark problem now', stop:'Stop diagnostics', copy:'Copy report for AI', clear:'Clear detailed session',
      active:'Detailed diagnostics active', off:'Detailed diagnostics off', copied:'Report copied.', hint:'For a browser problem, start "Diagnose this tab" in the extension. Reproduce, mark the problem, then export the existing ZIP. Only the chosen tab is observed; no cookies or page text are collected.' },
    'pt-BR': { start:'Iniciar diagn\u00f3stico do ADM (10 min)', mark:'Marcar problema agora', stop:'Encerrar diagn\u00f3stico', copy:'Copiar relat\u00f3rio para IA', clear:'Limpar sess\u00e3o detalhada',
      active:'Diagn\u00f3stico detalhado ativo', off:'Diagn\u00f3stico detalhado desligado', copied:'Relat\u00f3rio copiado.', hint:'Para problema no navegador, use "Diagnosticar esta aba" na extens\u00e3o. Reproduza, marque o problema e exporte o ZIP pelo bot\u00e3o existente. Somente a aba escolhida \u00e9 observada; sem cookies ou texto da p\u00e1gina.' },
    'zh-CN': { start:'\u5f00\u59cb ADM \u8bca\u65ad (10 \u5206\u949f)', mark:'\u6807\u8bb0\u95ee\u9898', stop:'\u505c\u6b62\u8bca\u65ad', copy:'\u590d\u5236 AI \u62a5\u544a', clear:'\u6e05\u9664\u8bca\u65ad\u4f1a\u8bdd', active:'\u8be6\u7ec6\u8bca\u65ad\u5df2\u5f00\u542f', off:'\u8be6\u7ec6\u8bca\u65ad\u5df2\u5173\u95ed', copied:'\u62a5\u544a\u5df2\u590d\u5236', hint:'\u6d4f\u89c8\u5668\u95ee\u9898\u8bf7\u5728\u6269\u5c55\u4e2d\u542f\u52a8\u6807\u7b7e\u9875\u8bca\u65ad\u3002\u91cd\u73b0\u5e76\u6807\u8bb0\u95ee\u9898\u540e\u5bfc\u51fa ZIP\u3002\u4e0d\u6536\u96c6 Cookie \u6216\u9875\u9762\u6587\u672c\u3002' },
  };
  const invokeNative = (command, args = {}) => window.__TAURI__?.core?.invoke(command,args) || Promise.reject(new Error('desktop_unavailable'));
  const panel = document.querySelector('#logs-panel');
  if (!panel) return;
  const controls = document.createElement('section'); controls.className='diagnostics-panel';
  controls.innerHTML='<p class="diag-hint"></p><div class="diag-buttons"><button data-diag="start"></button><button data-diag="mark"></button><button data-diag="stop"></button><button data-diag="copy"></button><button data-diag="clear"></button></div><p class="diag-status" role="status"></p>';
  panel.querySelector('header').after(controls);
  let busy = false, language = '', status = {};
  const labels = () => words[typeof locale === 'string' ? locale : 'en'] || words.en;
  function paint() {
    const text=labels();
    controls.querySelectorAll('[data-diag]').forEach(b=>{ b.textContent=text[b.dataset.diag]; b.disabled=busy || (['mark','stop'].includes(b.dataset.diag) && !status.active); });
    controls.querySelector('[data-diag="start"]').disabled=busy || Boolean(status.active);
    controls.querySelector('.diag-hint').textContent=text.hint;
    controls.querySelector('.diag-status').textContent=`${status.active ? text.active : text.off}${status.sessionId ? ` | ${status.sessionId}` : ''}`;
  }
  async function refresh() {
    if (busy || panel.hidden) return;
    try { status=await invokeNative('diagnostics_status'); paint(); } catch { /* The normal Logs view remains usable. */ }
  }
  controls.addEventListener('click',async event=>{
    const b=event.target.closest('[data-diag]'); if (!b || busy) return;
    busy=true;paint();
    try {
      if (b.dataset.diag==='copy') { await invokeNative('copy_diagnostics_report'); controls.querySelector('.diag-status').textContent=labels().copied; }
      else { status=await invokeNative('diagnostics_control',{action:b.dataset.diag}); }
    } catch(error) { controls.querySelector('.diag-status').textContent=String(error); }
    finally { busy=false; if(b.dataset.diag!=='copy') paint(); else { const message=controls.querySelector('.diag-status').textContent; paint(); controls.querySelector('.diag-status').textContent=message; } }
  });
  window.ADM_TASK_DIAGNOSTICS=(event,detail)=>invokeNative('record_diagnostics_ui',{event,detail}).catch(()=>{});
  paint();setInterval(refresh,2000);
})();
