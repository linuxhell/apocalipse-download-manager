(() => {
  const words = {
    en: { start:'Start desktop diagnostics (30 min)', mark:'Mark problem now', stop:'Stop diagnostics', copy:'Copy report for AI', clear:'Clear detailed session',
      active:'Detailed diagnostics active', off:'Detailed diagnostics off', copied:'Report copied.', hint:'For a browser problem, start "Diagnose this tab" in the extension. Reproduce, mark each problem when it happens (you can mark multiple incidents), then stop diagnostics and export the ZIP. Only the chosen tab is observed; no cookies or page text are collected.' },
    'pt-BR': { start:'Iniciar diagn\u00f3stico do ADM (30 min)', mark:'Marcar problema agora', stop:'Encerrar diagn\u00f3stico', copy:'Copiar relat\u00f3rio para IA', clear:'Limpar sess\u00e3o detalhada',
      active:'Diagn\u00f3stico detalhado ativo', off:'Diagn\u00f3stico detalhado desligado', copied:'Relat\u00f3rio copiado.', hint:'Para problema no navegador, use "Diagnosticar esta aba" na extens\u00e3o. Reproduza, marque cada problema quando acontecer (pode marcar v\u00e1rios incidentes), depois encerre o diagn\u00f3stico e exporte o ZIP. Somente a aba escolhida \u00e9 observada; sem cookies ou texto da p\u00e1gina.' },
    'zh-CN': { start:'\u5f00\u59cb ADM \u8bca\u65ad (30 \u5206\u949f)', mark:'\u6807\u8bb0\u95ee\u9898', stop:'\u505c\u6b62\u8bca\u65ad', copy:'\u590d\u5236 AI \u62a5\u544a', clear:'\u6e05\u9664\u8bca\u65ad\u4f1a\u8bdd', active:'\u8be6\u7ec6\u8bca\u65ad\u5df2\u5f00\u542f', off:'\u8be6\u7ec6\u8bca\u65ad\u5df2\u5173\u95ed', copied:'\u62a5\u544a\u5df2\u590d\u5236', hint:'\u6d4f\u89c8\u5668\u95ee\u9898\u8bf7\u5728\u6269\u5c55\u4e2d\u542f\u52a8\u6807\u7b7e\u9875\u8bca\u65ad\u3002\u91cd\u73b0\u95ee\u9898\u65f6\u53ef\u591a\u6b21\u6807\u8bb0\u4e0d\u540c\u4e8b\u4ef6\uff0c\u7136\u540e\u505c\u6b62\u8bca\u65ad\u5e76\u5bfc\u51fa ZIP\u3002\u4e0d\u6536\u96c6 Cookie \u6216\u9875\u9762\u6587\u672c\u3002' },
  };
  const healthWords = {
    en: { healthy:'healthy', degraded:'degraded', unavailable:'unavailable', unknown:'not yet checked', log:'General log', persistence:'Saved state', write:'Write errors', rotation:'Rotation errors', errors:'Errors', browser:'Extension', queued:'Queued', accepted:'Accepted', dropped:'Dropped', transport:'Transport errors', storage:'Storage errors', last:'Last successful write', upload:'Last upload', sync:'Last config sync', failed:'Last transport error', collector:'Collector write errors' },
    'pt-BR': { healthy:'saudável', degraded:'degradado', unavailable:'indisponível', unknown:'ainda não verificado', log:'Log geral', persistence:'Estado salvo', write:'Erros de escrita', rotation:'Erros de rotação', errors:'Erros', browser:'Extensão', queued:'Na fila', accepted:'Aceitos', dropped:'Descartados', transport:'Erros de transporte', storage:'Erros de armazenamento', last:'Última escrita bem-sucedida', upload:'Último envio', sync:'Última sincronização', failed:'Último erro de transporte', collector:'Erros de escrita do coletor' },
    'zh-CN': { healthy:'正常', degraded:'降级', unavailable:'不可用', unknown:'尚未检查', log:'常规日志', persistence:'已保存状态', write:'写入错误', rotation:'轮转错误', errors:'错误', browser:'扩展', queued:'排队', accepted:'已接收', dropped:'已丢弃', transport:'传输错误', storage:'存储错误', last:'上次成功写入', upload:'上次上传', sync:'上次配置同步', failed:'上次传输错误', collector:'收集器写入错误' },
  };
  const invokeNative = (command, args = {}) => window.__TAURI__?.core?.invoke(command,args) || Promise.reject(new Error('desktop_unavailable'));
  const panel = document.querySelector('#logs-panel');
  if (!panel) return;
  const controls = document.createElement('section'); controls.className='diagnostics-panel';
  controls.innerHTML='<p class="diag-hint"></p><div class="diag-buttons"><button data-diag="start"></button><button data-diag="mark"></button><button data-diag="stop"></button><button data-diag="copy"></button><button data-diag="clear"></button></div><p class="diag-status" role="status"></p><p class="diag-health" role="status" style="white-space:pre-line;overflow-wrap:anywhere"></p>';
  panel.querySelector('header').after(controls);
  let busy = false, language = '', status = {};
  const labels = () => words[typeof locale === 'string' ? locale : 'en'] || words.en;
  function paint() {
    const text=labels();
    controls.querySelectorAll('[data-diag]').forEach(b=>{ b.textContent=text[b.dataset.diag]; b.disabled=busy || (['mark','stop'].includes(b.dataset.diag) && !status.active); });
    controls.querySelector('[data-diag="start"]').disabled=busy || Boolean(status.active);
    controls.querySelector('.diag-hint').textContent=text.hint;
    controls.querySelector('.diag-status').textContent=`${status.active ? text.active : text.off}${status.sessionId ? ` | ${status.sessionId}` : ''}`;
    const h = healthWords[typeof locale === 'string' ? locale : 'en'] || healthWords.en;
    const time = value => value ? new Date(value).toLocaleString() : '—';
    const log = status.runtimeHealth?.generalLog, persistence = status.runtimeHealth?.persistence, browser = status.clientHealth;
    const lines = [];
    if (log) lines.push(`${h.log}: ${h[log.status] || h.unknown} | ${h.write}: ${log.logWriteErrors} | ${h.rotation}: ${log.logRotationErrors} | ${h.last}: ${time(log.lastSuccessfulWrite)}`);
    if (persistence) lines.push(`${h.persistence}: ${h[persistence.status] || h.unknown} | ${h.errors}: ${persistence.persistenceErrors}`);
    if (status.collectorWriteErrors !== undefined) lines.push(`${h.collector}: ${status.collectorWriteErrors}`);
    if (browser && typeof browser.queued === 'number') {
      lines.push(`${h.browser}: ${h[browser.status] || h.unknown} | ${h.queued}: ${browser.queued} | ${h.accepted}: ${browser.accepted || 0} | ${h.dropped}: ${browser.dropped} | ${h.transport}: ${browser.transportErrors} | ${h.storage}: ${browser.storageErrors}`);
      lines.push(`${h.upload}: ${time(browser.lastSuccessfulUploadAt)} | ${h.sync}: ${time(browser.lastSuccessfulConfigSyncAt)} | ${h.failed}: ${time(browser.lastTransportErrorAt)}`);
    }
    controls.querySelector('.diag-health').textContent = lines.join('\n');
  }
  async function refresh() {
    if (busy || panel.hidden) return;
    try { status=await invokeNative('diagnostics_status'); paint(); } catch { controls.querySelector('.diag-health').textContent = (healthWords[typeof locale === 'string' ? locale : 'en'] || healthWords.en).unavailable; }
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
