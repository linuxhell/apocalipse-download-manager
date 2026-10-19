(() => {
  "use strict";
  const catalogs = {
    en: {
      preview: "Experimental preview", connect: "Connect ED2K", disconnect: "Disconnect", bootstrap: "Bootstrap Kad", networks: "Networks", search: "Search", preferences: "Preferences", log: "Log", servers: "ED2K servers", serversHint: "Select a server or connect using the initial list.", serverName: "Name", address: "Address", port: "Port", description: "Description", scopeTitle: "Current capabilities", scopeHint: "Native Nibiru server login and search. Peer transfers, sharing and persistent Kad routing are still in development; this preview is not a complete aMule replacement.", kadHint: "Bootstrap checks a node and retrieves contacts. It does not yet keep a Kad session running.", searchHint: "Search the connected ED2K server. Results contain ED2K links; downloading from peers is not available in this preview.", query: "File name", file: "File", size: "Size", sources: "Sources", preferencesHint: "Language and colour mode follow the main ADM window.", listUrl: "HTTPS list URL", updateServers: "Update server.met", importServers: "Import server.met", updateNodes: "Update nodes.dat", importNodes: "Import nodes.dat", listSafety: "Lists are size-limited and validated before replacing the current list. Existing data is preserved if validation fails.", clear: "Clear", engine: "Native Rust · Nibiru Xtreme", ready: "Ready", offline: "Disconnected", online: "Connected", lowId: "Low ID · incoming connections unavailable", kadInactive: "Kad: inactive", kadReady: "Kad: bootstrap responded", nodeCount: "{count} initial Kad nodes", kadContacts: "Bootstrap: {count} contacts received from {host}", serverCount: "{count} servers", working: "Working…", failed: "Operation failed", updated: "List updated: {count} entries", imported: "List imported: {count} entries", resultCount: "{count} results", noResults: "No results", copy: "Copy link", copied: "ED2K link copied", desktopRequired: "Open this window from the desktop application.", connectedTo: "Connected to {name}", connectionClosed: "Connection closed", invalidUrl: "Enter an HTTPS URL without credentials.",
    },
    "pt-BR": {
      preview: "Prévia experimental", connect: "Conectar ED2K", disconnect: "Desconectar", bootstrap: "Iniciar bootstrap Kad", networks: "Redes", search: "Pesquisar", preferences: "Preferências", log: "Log", servers: "Servidores ED2K", serversHint: "Escolha um servidor ou conecte usando a lista inicial.", serverName: "Nome", address: "Endereço", port: "Porta", description: "Descrição", scopeTitle: "Recursos atuais", scopeHint: "Login e pesquisa de servidores pelo Nibiru nativo. Transferências entre peers, compartilhamento e roteamento Kad contínuo ainda estão em desenvolvimento; esta prévia não substitui o aMule completo.", kadHint: "O bootstrap verifica um nó e recebe contatos. Ele ainda não mantém uma sessão Kad ativa.", searchHint: "Pesquise no servidor ED2K conectado. Os resultados contêm links ED2K; downloads de peers ainda não estão disponíveis nesta prévia.", query: "Nome do arquivo", file: "Arquivo", size: "Tamanho", sources: "Fontes", preferencesHint: "O idioma e as cores acompanham a janela principal do ADM.", listUrl: "URL HTTPS da lista", updateServers: "Atualizar server.met", importServers: "Importar server.met", updateNodes: "Atualizar nodes.dat", importNodes: "Importar nodes.dat", listSafety: "As listas têm limite de tamanho e são validadas antes da substituição. Os dados existentes são preservados se a validação falhar.", clear: "Limpar", engine: "Rust nativo · Nibiru Xtreme", ready: "Pronto", offline: "Desconectado", online: "Conectado", lowId: "ID baixo · conexões de entrada indisponíveis", kadInactive: "Kad: inativo", kadReady: "Kad: bootstrap respondeu", nodeCount: "{count} nós Kad iniciais", kadContacts: "Bootstrap: {count} contatos recebidos de {host}", serverCount: "{count} servidores", working: "Processando…", failed: "Falha na operação", updated: "Lista atualizada: {count} entradas", imported: "Lista importada: {count} entradas", resultCount: "{count} resultados", noResults: "Nenhum resultado", copy: "Copiar link", copied: "Link ED2K copiado", desktopRequired: "Abra esta janela pelo aplicativo desktop.", connectedTo: "Conectado a {name}", connectionClosed: "Conexão encerrada", invalidUrl: "Informe uma URL HTTPS sem credenciais.",
    },
    "zh-CN": {
      preview: "实验预览版", connect: "连接 ED2K", disconnect: "断开连接", bootstrap: "引导 Kad", networks: "网络", search: "搜索", preferences: "首选项", log: "日志", servers: "ED2K 服务器", serversHint: "选择服务器，或使用初始列表连接。", serverName: "名称", address: "地址", port: "端口", description: "说明", scopeTitle: "当前功能", scopeHint: "通过原生 Nibiru 登录并搜索服务器。节点传输、文件共享和持续 Kad 路由仍在开发中；此预览版尚不能完整替代 aMule。", kadHint: "引导会检测节点并获取联系人，目前不会保持持续的 Kad 会话。", searchHint: "搜索已连接的 ED2K 服务器。结果包含 ED2K 链接；此预览版尚不支持从节点下载。", query: "文件名", file: "文件", size: "大小", sources: "来源", preferencesHint: "语言和颜色模式跟随 ADM 主窗口。", listUrl: "列表的 HTTPS 地址", updateServers: "更新 server.met", importServers: "导入 server.met", updateNodes: "更新 nodes.dat", importNodes: "导入 nodes.dat", listSafety: "列表有大小限制，验证通过后才替换当前列表。验证失败时保留原有数据。", clear: "清除", engine: "原生 Rust · Nibiru Xtreme", ready: "就绪", offline: "未连接", online: "已连接", lowId: "低 ID · 入站连接不可用", kadInactive: "Kad：未启用", kadReady: "Kad：引导节点已响应", nodeCount: "{count} 个初始 Kad 节点", kadContacts: "引导：从 {host} 收到 {count} 个联系人", serverCount: "{count} 个服务器", working: "正在处理…", failed: "操作失败", updated: "列表已更新：{count} 条", imported: "列表已导入：{count} 条", resultCount: "{count} 个结果", noResults: "没有结果", copy: "复制链接", copied: "已复制 ED2K 链接", desktopRequired: "请从桌面应用程序打开此窗口。", connectedTo: "已连接至 {name}", connectionClosed: "连接已关闭", invalidUrl: "请输入不含凭据的 HTTPS 地址。",
    },
  };
  let locale = "en";
  try { const cached = localStorage.getItem("apocalipse.language"); if (catalogs[cached]) locale = cached; } catch {}
  const invoke = window.__TAURI__?.core?.invoke;
  const $ = selector => document.querySelector(selector);
  const t = (key, values = {}) => Object.entries(values).reduce((s, [k,v]) => s.replaceAll(`{${k}}`, String(v)), catalogs[locale][key] || catalogs.en[key] || key);
  let overview = null, results = [], busy = false;
  let feedback = { key: "ready", values: {}, error: false };
  const logLines = [];
  function message(key, values = {}, error = false) { feedback = { key, values, error }; $("#ed2k-feedback").textContent = t(key, values); $("#ed2k-feedback").dataset.error = String(error); }
  function log(text) { logLines.push(`${new Date().toLocaleTimeString(locale)}  ${text}`); if (logLines.length > 300) logLines.shift(); $("#ed2k-log").textContent = logLines.join("\n"); }
  function controls() {
    const connected = Boolean(overview?.server);
    $("#ed2k-connect").disabled = busy || !overview?.servers?.length;
    $("#ed2k-disconnect").disabled = !connected && !busy;
    $("#ed2k-bootstrap").disabled = busy || !overview?.node_count;
    $("#ed2k-search-submit").disabled = busy || !connected;
    for (const button of document.querySelectorAll(".ed2k-setting button, #ed2k-servers button")) button.disabled = busy;
  }
  function size(bytes) { return `${new Intl.NumberFormat(locale, {maximumFractionDigits:2}).format(bytes / (1024*1024))} MiB`; }
  function renderResults() {
    const body = $("#ed2k-results"); body.replaceChildren();
    for (const result of results) {
      const row = document.createElement("tr");
      for (const value of [result.name, size(result.size), result.sources, result.hash]) { const td = document.createElement("td"); td.textContent = String(value); td.title = String(value); row.append(td); }
      row.children[3].className = "hash";
      const td = document.createElement("td"), copy = document.createElement("button"); copy.type = "button"; copy.textContent = t("copy");
      copy.onclick = () => run("ed2k_copy_link", {link:result.link}, () => message("copied")); td.append(copy); row.append(td); body.append(row);
    }
    $("#search-summary").textContent = results.length ? t("resultCount", {count:results.length}) : t("noResults");
  }
  function render() {
    document.documentElement.lang = locale;
    for (const node of document.querySelectorAll("[data-i18n]")) node.textContent = t(node.dataset.i18n);
    message(feedback.key, feedback.values, feedback.error);
    $("#ed2k-status").textContent = `ED2K: ${overview?.server ? t("online") : t("offline")}${overview?.low_id ? ` · ${t("lowId")}` : ""}`;
    $("#ed2k-status").dataset.online = String(Boolean(overview?.server));
    $("#kad-status").textContent = t(overview?.kad_bootstrap ? "kadReady" : "kadInactive");
    $("#kad-summary").textContent = overview?.kad_bootstrap ? t("kadContacts", {count:overview.kad_bootstrap.contacts.length, host:overview.kad_bootstrap.responder.host}) : t("nodeCount", {count:overview?.node_count || 0});
    $("#server-count").textContent = t("serverCount", {count:overview?.servers?.length || 0});
    const body = $("#ed2k-servers"); body.replaceChildren();
    for (const server of overview?.servers || []) {
      const row = document.createElement("tr");
      for (const value of [server.name, server.host, server.port, server.description]) { const td = document.createElement("td"); td.textContent = String(value); td.title = String(value); row.append(td); }
      const td = document.createElement("td"), connect = document.createElement("button"); connect.type = "button"; connect.textContent = t("connect"); connect.onclick = () => run("ed2k_connect", {server}, () => message("connectedTo", {name:server.name})); td.append(connect); row.append(td); body.append(row);
    }
    renderResults(); controls();
  }
  async function refresh() { if (!invoke) return; overview = await invoke("ed2k_overview"); render(); }
  async function run(command, args = {}, success = () => message("ready")) {
    if (!invoke) { message("desktopRequired", {}, true); return; }
    if (busy) return;
    busy = true; message("working"); controls();
    try { const result = await invoke(command, args); await refresh(); success(result); log(`${t("ready")}: ${command}`); }
    catch (error) { message("failed", {}, true); log(`${t("failed")}: ${String(error)}`); }
    finally { busy = false; controls(); }
  }
  $("#ed2k-connect").onclick = () => run("ed2k_connect", {server:null});
  $("#ed2k-disconnect").onclick = async () => { try { await invoke?.("ed2k_disconnect"); await refresh(); message("connectionClosed"); } catch(error) { message("failed", {}, true); log(String(error)); } };
  $("#ed2k-bootstrap").onclick = () => run("ed2k_bootstrap");
  for (const kind of ["servers", "nodes"]) {
    $("#ed2k-update-"+kind).onclick = () => {
      const url = $(kind === "servers" ? "#ed2k-server-url" : "#ed2k-node-url").value.trim();
      try { const parsed = new URL(url); if (parsed.protocol !== "https:" || parsed.username || parsed.password) throw new Error(); }
      catch { message("invalidUrl", {}, true); return; }
      run("ed2k_update_list", {kind,url}, count => message("updated", {count}));
    };
    $("#ed2k-import-"+kind).onclick = () => run("ed2k_import_list", {kind}, count => message(count === null ? "ready" : "imported", {count}));
  }
  $("#ed2k-search-form").onsubmit = event => { event.preventDefault(); const query = $("#ed2k-query").value.trim(); if (query) run("ed2k_search", {query}, value => { results=value; renderResults(); message("resultCount", {count:results.length}); }); };
  $("#ed2k-clear-log").onclick = () => { logLines.length=0; $("#ed2k-log").textContent=""; };
  for (const tab of document.querySelectorAll("[data-tab]")) tab.onclick = () => {
    for (const button of document.querySelectorAll("[data-tab]")) button.setAttribute("aria-selected",String(button === tab));
    for (const panel of document.querySelectorAll("[role=tabpanel]")) panel.hidden = panel.id !== "panel-"+tab.dataset.tab;
  };
  function language(value) { if (!catalogs[value]) return; locale=value; try { localStorage.setItem("apocalipse.language",locale); } catch {} render(); }
  window.addEventListener("storage", event => { if (event.key === "apocalipse.language") language(event.newValue); if (event.key === "apocalipse.mode") document.documentElement.dataset.mode = event.newValue === "dark" ? "dark" : "light"; });
  let unlisten;
  window.__TAURI__?.event?.listen("apocalipse-language-changed", event => language(event.payload)).then(stop => { unlisten=stop; }).catch(error => log(String(error)));
  window.addEventListener("pagehide", () => unlisten?.(), {once:true});
  render();
  if (invoke) {
    Promise.all([invoke("get_application_language"),invoke("get_dark_mode")]).then(([saved,dark]) => { language(saved); document.documentElement.dataset.mode = dark ? "dark" : "light"; }).catch(error => log(String(error)));
    refresh().catch(error => { message("failed", {}, true); log(String(error)); });
    const timer = setInterval(() => { if (!busy && !document.hidden) refresh().catch(error => log(String(error))); },2000);
    window.addEventListener("pagehide", () => clearInterval(timer), {once:true});
  } else { message("desktopRequired", {}, true); }
})();
