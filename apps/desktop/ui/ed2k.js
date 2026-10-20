(() => {
  "use strict";
  const catalogs = {
    en: {
      incomingPort: "TCP: {port} · {count} handshakes", incomingInactive: "TCP: inactive",
      transfers: "Transfers", transfersHint: "Downloads use the ADM download folder. Direct peers, verified ED2K parts and resume are supported; Low-ID sources need a reachable incoming port. Uploads are still in development.", fileLink: "ED2K file link", addDownload: "Add download", download: "Download", addedDownload: "Download added", verifiedProgress: "Verified", status: "Status", pause: "Pause", resume: "Resume", removeTask: "Remove from list", retainedFilesHint: "Removing a record preserves downloaded files and verified partial data. Unfinished downloads reopen paused and are checked again when resumed.", stage_queued: "Queued", stage_connecting: "Finding a peer", stage_waiting_peer: "Waiting for a peer slot", stage_checking: "Checking saved parts", stage_downloading: "Downloading", stage_pausing: "Pausing…", stage_paused: "Paused", stage_error: "Failed — resume to retry", stage_completed: "Completed", queueRank: "Queue position: {rank}",
      findSources: "Find sources", sourceTitle: "Sources: {name}", sourceHint: "Addresses reported by the connected server. Low-ID peers require server callbacks and a reachable incoming port.", connectionType: "Connection", directPeer: "Direct peer", callbackPeer: "Server callback required", sourceCount: "{count} sources returned by the server", filterResults: "Filter results", sortResults: "Sort by", sortSources: "Most sources", sortSize: "Largest first", sortName: "File name", filteredCount: "{count} of {total} results",
      preview: "Experimental preview", connect: "Connect ED2K", disconnect: "Disconnect", stopKad:"Stop Kad",kadRouting:"{count}/{total} contacts verified · UDP {port}",bootstrap: "Start Kad", networks: "Networks", search: "Search", preferences: "Preferences", log: "Log", servers: "ED2K servers", serversHint: "Select a server or connect using the initial list.", serverName: "Name", address: "Address", port: "Port", description: "Description", scopeTitle: "Current capabilities", scopeHint: "Native server search, source discovery and verified downloads from direct peers. Sharing and persistent Kad routing are still in development.", kadHint: "Kad maintains contacts and looks up direct sources. Keyword search, publishing, obfuscation and Kad callbacks are not available yet.", searchHint: "Search the connected ED2K server, inspect sources or add files to the transfer queue.", query: "File name", file: "File", size: "Size", sources: "Sources", preferencesHint: "Language and colour mode follow the main ADM window.", listUrl: "HTTPS list URL", updateServers: "Update server.met", importServers: "Import server.met", updateNodes: "Update nodes.dat", importNodes: "Import nodes.dat", listSafety: "Lists are size-limited and validated before replacing the current list. Existing data is preserved if validation fails.", clear: "Clear", engine: "Native Rust · Nibiru Xtreme", ready: "Ready", offline: "Disconnected", online: "Connected", lowId: "Low ID · incoming connections unavailable", kadInactive: "Kad: inactive", kadReady: "Kad: bootstrap responded", nodeCount: "{count} initial Kad nodes", kadContacts: "Bootstrap: {count} contacts received from {host}", serverCount: "{count} servers", working: "Working…", failed: "Operation failed", updated: "List updated: {count} entries", imported: "List imported: {count} entries", resultCount: "{count} results", noResults: "No results", copy: "Copy link", copied: "ED2K link copied", desktopRequired: "Open this window from the desktop application.", connectedTo: "Connected to {name}", connectionClosed: "Connection closed", invalidUrl: "Enter an HTTPS URL without credentials.",

      shared: "Shared", uploads: "Uploads", sharedHint: "Only the files and folders you choose here are shared with other ED2K users. Nothing is shared automatically.", shareFile: "Share a file…", shareFolder: "Share a folder…", includeSubfolders: "Include subfolders", rescan: "Rescan", sharedItems: "Shared items", sharedFilesTitle: "Shared files", sharedPath: "Path", kind: "Type", kindFile: "File", kindDirectory: "Folder", sharedFiles: "Files", sharingOn: "Sharing", sharingOff: "Paused", pauseSharing: "Pause sharing", resumeSharing: "Resume sharing", stopSharing: "Stop sharing", priority: "Priority", priority0: "Very low", priority1: "Low", priority2: "Normal", priority3: "High", priority4: "Release", requests: "Requests", uploaded: "Uploaded", openFolder: "Open folder", hashing: "Preparing {done} of {total} files…", hashingNow: "Now: {name}", scanProblems: "{count} problem(s): {first}", sharedCount: "{count} shared files", publishedCount: "Offered {count} files to the server", notPublished: "Not offered yet — connect to an ED2K server.", removeHint: "Files you stop sharing stay in the server's index until you reconnect.", uploadsHint: "Other users download from you through slots. When every slot is busy they wait in a queue ordered by file priority and then by waiting time.", uploadSlots: "Simultaneous uploads", uploadLimit: "Upload limit (KiB/s, 0 = unlimited)", saveSettings: "Save", settingsSaved: "Upload settings saved", totalUploaded: "Uploaded in total: {size}", activeUploads: "Active uploads", waitingQueue: "Waiting queue", client: "Client", rank: "Position", waited: "Waiting", elapsed: "Time", noneActive: "No active uploads", noneWaiting: "Nobody is waiting", activeCount: "{count} of {max} slots in use", waitingCount: "{count} waiting", invalidUploadSettings: "Slots must be 1–64 and the limit 0–1000000.", shareAdded: "Added to shared items",
      transfersHint: "Downloads use the ADM download folder. Direct peers, verified ED2K parts and resume are supported; Low-ID sources need a reachable incoming port.", scopeHint: "Native server search, source discovery, verified downloads, file sharing, an upload queue and publishing to the server. Kad maintains routing and direct-source lookup; keyword search and Kad publishing remain pending."
    },
    "pt-BR": {
      incomingPort: "TCP: {port} · {count} conexões", incomingInactive: "TCP: inativo",
      transfers: "Transferências", transfersHint: "Os arquivos usam a pasta de downloads do ADM. Há suporte a peers diretos, partes ED2K verificadas e retomada; fontes com ID baixo precisam de porta de entrada acessível. Uploads ainda estão em desenvolvimento.", fileLink: "Link ED2K do arquivo", addDownload: "Adicionar download", download: "Baixar", addedDownload: "Download adicionado", verifiedProgress: "Verificado", status: "Estado", pause: "Pausar", resume: "Retomar", removeTask: "Remover da lista", retainedFilesHint: "Remover um registro preserva os arquivos baixados e as partes verificadas. Downloads inacabados reabrem pausados e são conferidos novamente ao retomar.", stage_queued: "Na fila", stage_connecting: "Buscando peer", stage_waiting_peer: "Aguardando vaga no peer", stage_checking: "Conferindo partes salvas", stage_downloading: "Baixando", stage_pausing: "Pausando…", stage_paused: "Pausado", stage_error: "Falhou — retome para tentar novamente", stage_completed: "Concluído", queueRank: "Posição na fila: {rank}",
      findSources: "Consultar fontes", sourceTitle: "Fontes: {name}", sourceHint: "Endereços informados pelo servidor conectado. Peers com ID baixo precisam de callback do servidor e porta de entrada acessível.", connectionType: "Conexão", directPeer: "Peer direto", callbackPeer: "Exige callback do servidor", sourceCount: "{count} fontes retornadas pelo servidor", filterResults: "Filtrar resultados", sortResults: "Ordenar por", sortSources: "Mais fontes", sortSize: "Maiores primeiro", sortName: "Nome do arquivo", filteredCount: "{count} de {total} resultados",
      preview: "Prévia experimental", connect: "Conectar ED2K", disconnect: "Desconectar", stopKad:"Parar Kad",kadRouting:"{count}/{total} contatos verificados · UDP {port}",bootstrap: "Iniciar Kad", networks: "Redes", search: "Pesquisar", preferences: "Preferências", log: "Log", servers: "Servidores ED2K", serversHint: "Escolha um servidor ou conecte usando a lista inicial.", serverName: "Nome", address: "Endereço", port: "Porta", description: "Descrição", scopeTitle: "Recursos atuais", scopeHint: "Pesquisa de servidores, consulta de fontes e downloads verificados de peers diretos pelo Nibiru nativo. Compartilhamento e roteamento Kad contínuo ainda estão em desenvolvimento.", kadHint: "O Kad mantém contatos e consulta fontes diretas. Busca por palavras, publicação, ofuscação e callbacks Kad ainda não estão disponíveis.", searchHint: "Pesquise no servidor ED2K conectado, consulte as fontes ou adicione arquivos à fila de transferências.", query: "Nome do arquivo", file: "Arquivo", size: "Tamanho", sources: "Fontes", preferencesHint: "O idioma e as cores acompanham a janela principal do ADM.", listUrl: "URL HTTPS da lista", updateServers: "Atualizar server.met", importServers: "Importar server.met", updateNodes: "Atualizar nodes.dat", importNodes: "Importar nodes.dat", listSafety: "As listas têm limite de tamanho e são validadas antes da substituição. Os dados existentes são preservados se a validação falhar.", clear: "Limpar", engine: "Rust nativo · Nibiru Xtreme", ready: "Pronto", offline: "Desconectado", online: "Conectado", lowId: "ID baixo · conexões de entrada indisponíveis", kadInactive: "Kad: inativo", kadReady: "Kad: contatos verificados", nodeCount: "{count} nós Kad iniciais", kadContacts: "Bootstrap: {count} contatos recebidos de {host}", serverCount: "{count} servidores", working: "Processando…", failed: "Falha na operação", updated: "Lista atualizada: {count} entradas", imported: "Lista importada: {count} entradas", resultCount: "{count} resultados", noResults: "Nenhum resultado", copy: "Copiar link", copied: "Link ED2K copiado", desktopRequired: "Abra esta janela pelo aplicativo desktop.", connectedTo: "Conectado a {name}", connectionClosed: "Conexão encerrada", invalidUrl: "Informe uma URL HTTPS sem credenciais.",

      shared: "Compartilhados", uploads: "Uploads", sharedHint: "Somente os arquivos e pastas escolhidos aqui são compartilhados com outros usuários ED2K. Nada é compartilhado automaticamente.", shareFile: "Compartilhar um arquivo…", shareFolder: "Compartilhar uma pasta…", includeSubfolders: "Incluir subpastas", rescan: "Atualizar", sharedItems: "Itens compartilhados", sharedFilesTitle: "Arquivos compartilhados", sharedPath: "Caminho", kind: "Tipo", kindFile: "Arquivo", kindDirectory: "Pasta", sharedFiles: "Arquivos", sharingOn: "Compartilhando", sharingOff: "Pausado", pauseSharing: "Pausar compartilhamento", resumeSharing: "Retomar compartilhamento", stopSharing: "Parar de compartilhar", priority: "Prioridade", priority0: "Muito baixa", priority1: "Baixa", priority2: "Normal", priority3: "Alta", priority4: "Lançamento", requests: "Pedidos", uploaded: "Enviado", openFolder: "Abrir pasta", hashing: "Preparando {done} de {total} arquivos…", hashingNow: "Agora: {name}", scanProblems: "{count} problema(s): {first}", sharedCount: "{count} arquivos compartilhados", publishedCount: "{count} arquivos oferecidos ao servidor", notPublished: "Ainda não oferecido — conecte-se a um servidor ED2K.", removeHint: "Arquivos que você deixa de compartilhar continuam no índice do servidor até você reconectar.", uploadsHint: "Outros usuários baixam de você por vagas. Quando todas estão ocupadas, esperam numa fila ordenada pela prioridade do arquivo e depois pelo tempo de espera.", uploadSlots: "Uploads simultâneos", uploadLimit: "Limite de upload (KiB/s, 0 = ilimitado)", saveSettings: "Salvar", settingsSaved: "Configurações de upload salvas", totalUploaded: "Total enviado: {size}", activeUploads: "Uploads ativos", waitingQueue: "Fila de espera", client: "Cliente", rank: "Posição", waited: "Espera", elapsed: "Tempo", noneActive: "Nenhum upload ativo", noneWaiting: "Ninguém está esperando", activeCount: "{count} de {max} vagas em uso", waitingCount: "{count} esperando", invalidUploadSettings: "As vagas devem ser de 1 a 64 e o limite de 0 a 1000000.", shareAdded: "Adicionado aos itens compartilhados",
      transfersHint: "Os arquivos usam a pasta de downloads do ADM. Há suporte a peers diretos, partes ED2K verificadas e retomada; fontes com ID baixo precisam de porta de entrada acessível.", scopeHint: "Pesquisa em servidores, consulta de fontes, downloads verificados, compartilhamento de arquivos, fila de upload e publicação no servidor, tudo nativo. Kad mantém roteamento e consulta fontes diretas; busca por palavras e publicação Kad ainda estão pendentes."
    },
    "zh-CN": {
      incomingPort: "TCP：{port} · {count} 次握手", incomingInactive: "TCP：未启用",
      transfers: "传输", transfersHint: "文件保存到 ADM 下载目录。支持直接节点、ED2K 分块校验和续传；低 ID 来源需要可达的入站端口。上传仍在开发中。", fileLink: "ED2K 文件链接", addDownload: "添加下载", download: "下载", addedDownload: "已添加下载", verifiedProgress: "已验证", status: "状态", pause: "暂停", resume: "继续", removeTask: "从列表移除", retainedFilesHint: "移除记录会保留已下载文件和验证过的分块。未完成任务重新打开后暂停，继续时会再次校验。", stage_queued: "排队中", stage_connecting: "查找节点", stage_waiting_peer: "等待节点传输位", stage_checking: "验证已保存分块", stage_downloading: "下载中", stage_pausing: "正在暂停…", stage_paused: "已暂停", stage_error: "失败，可继续重试", stage_completed: "已完成", queueRank: "排队位置：{rank}",
      findSources: "查找来源", sourceTitle: "来源：{name}", sourceHint: "已连接服务器返回的地址。低 ID 节点需要服务器回调和可访问的入站端口。", connectionType: "连接", directPeer: "直接连接", callbackPeer: "需要服务器回调", sourceCount: "服务器返回 {count} 个来源", filterResults: "筛选结果", sortResults: "排序", sortSources: "来源最多", sortSize: "文件最大", sortName: "文件名", filteredCount: "{total} 个结果中的 {count} 个",
      preview: "实验预览版", connect: "连接 ED2K", disconnect: "断开连接", stopKad:"停止 Kad",kadRouting:"已验证 {count}/{total} 个联系人 · UDP {port}",bootstrap: "启动 Kad", networks: "网络", search: "搜索", preferences: "首选项", log: "日志", servers: "ED2K 服务器", serversHint: "选择服务器，或使用初始列表连接。", serverName: "名称", address: "地址", port: "端口", description: "说明", scopeTitle: "当前功能", scopeHint: "支持原生服务器搜索、来源查询和直接节点的验证下载。文件共享和持续 Kad 路由仍在开发中。", kadHint: "Kad 持续维护路由并查询直连来源；关键词搜索、发布、混淆和 Kad 回调尚未实现。", searchHint: "搜索已连接的 ED2K 服务器，查询来源或将文件加入传输队列。", query: "文件名", file: "文件", size: "大小", sources: "来源", preferencesHint: "语言和颜色模式跟随 ADM 主窗口。", listUrl: "列表的 HTTPS 地址", updateServers: "更新 server.met", importServers: "导入 server.met", updateNodes: "更新 nodes.dat", importNodes: "导入 nodes.dat", listSafety: "列表有大小限制，验证通过后才替换当前列表。验证失败时保留原有数据。", clear: "清除", engine: "原生 Rust · Nibiru Xtreme", ready: "就绪", offline: "未连接", online: "已连接", lowId: "低 ID · 入站连接不可用", kadInactive: "Kad：未启用", kadReady: "Kad：已验证联系人", nodeCount: "{count} 个初始 Kad 节点", kadContacts: "引导：从 {host} 收到 {count} 个联系人", serverCount: "{count} 个服务器", working: "正在处理…", failed: "操作失败", updated: "列表已更新：{count} 条", imported: "列表已导入：{count} 条", resultCount: "{count} 个结果", noResults: "没有结果", copy: "复制链接", copied: "已复制 ED2K 链接", desktopRequired: "请从桌面应用程序打开此窗口。", connectedTo: "已连接至 {name}", connectionClosed: "连接已关闭", invalidUrl: "请输入不含凭据的 HTTPS 地址。",

      shared: "共享", uploads: "上传", sharedHint: "只有你在这里选择的文件和文件夹才会与其他 ED2K 用户共享，不会自动共享任何内容。", shareFile: "共享文件…", shareFolder: "共享文件夹…", includeSubfolders: "包含子文件夹", rescan: "重新扫描", sharedItems: "共享项目", sharedFilesTitle: "共享的文件", sharedPath: "路径", kind: "类型", kindFile: "文件", kindDirectory: "文件夹", sharedFiles: "文件数", sharingOn: "共享中", sharingOff: "已暂停", pauseSharing: "暂停共享", resumeSharing: "恢复共享", stopSharing: "停止共享", priority: "优先级", priority0: "极低", priority1: "低", priority2: "普通", priority3: "高", priority4: "发布", requests: "请求", uploaded: "已上传", openFolder: "打开文件夹", hashing: "正在准备 {done}/{total} 个文件…", hashingNow: "当前：{name}", scanProblems: "{count} 个问题：{first}", sharedCount: "{count} 个共享文件", publishedCount: "已向服务器提供 {count} 个文件", notPublished: "尚未提供——请先连接 ED2K 服务器。", removeHint: "停止共享的文件在你重新连接之前仍会保留在服务器索引中。", uploadsHint: "其他用户通过名额从你这里下载。名额占满时，他们会按文件优先级、再按等待时间排队。", uploadSlots: "同时上传数", uploadLimit: "上传限速（KiB/s，0 = 不限）", saveSettings: "保存", settingsSaved: "上传设置已保存", totalUploaded: "累计上传：{size}", activeUploads: "正在上传", waitingQueue: "等待队列", client: "客户端", rank: "位置", waited: "等待", elapsed: "时间", noneActive: "没有正在进行的上传", noneWaiting: "没有人在等待", activeCount: "已用 {count}/{max} 个名额", waitingCount: "{count} 个在等待", invalidUploadSettings: "名额须为 1–64，限速须为 0–1000000。", shareAdded: "已添加到共享项目",
      transfersHint: "下载使用 ADM 的下载文件夹。支持直连节点、已验证的 ED2K 分块和续传；低 ID 来源需要可访问的入站端口。", scopeHint: "原生服务器搜索、来源发现、已验证下载、文件共享、上传队列以及向服务器发布。Kad 持续维护路由并查询直连来源；关键词搜索和 Kad 发布尚未实现。"
    },
  };
  let locale = "en";
  try { const cached = localStorage.getItem("apocalipse.language"); if (catalogs[cached]) locale = cached; } catch {}
  const invoke = window.__TAURI__?.core?.invoke;
  const $ = selector => document.querySelector(selector);
  const t = (key, values = {}) => Object.entries(values).reduce((s, [k,v]) => s.replaceAll(`{${k}}`, String(v)), catalogs[locale][key] || catalogs.en[key] || key);
  let overview = null, results = [], sourceView = null, busy = false, shared = null, currentTab = "networks", settingsDirty = false;
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
    $("#ed2k-add-submit").disabled = busy;
    for (const id of ["#ed2k-share-file", "#ed2k-share-folder", "#ed2k-share-rescan", "#ed2k-upload-save"]) $(id).disabled = busy;
    for (const button of document.querySelectorAll("#ed2k-downloads button")) button.disabled = busy || button.dataset.pausing === "true" || (button.dataset.connection === "true" && !connected && !overview?.kad_session?.running);
    for (const button of document.querySelectorAll(".ed2k-setting button, #ed2k-servers button")) button.disabled = busy;
    for (const button of document.querySelectorAll("#ed2k-results button")) button.disabled = busy || (button.dataset.sources === "true" && !connected);
  }
  function size(bytes) { const units=["B","KiB","MiB","GiB","TiB"]; const index=bytes>0 ? Math.min(4,Math.floor(Math.log(bytes)/Math.log(1024))) : 0; return `${new Intl.NumberFormat(locale,{maximumFractionDigits:2}).format(bytes/1024**index)} ${units[index]}`; }
  function renderResults() {
    const body = $("#ed2k-results"); body.replaceChildren();
    const filter = $("#ed2k-result-filter").value.trim().toLocaleLowerCase(locale);
    const order = $("#ed2k-result-sort").value;
    const visible = results.filter(result => result.name.toLocaleLowerCase(locale).includes(filter)).sort((a,b) => (order === "name" ? a.name.localeCompare(b.name,locale) : order === "size" ? b.size-a.size : b.sources-a.sources) || a.hash.localeCompare(b.hash));
    for (const result of visible) {
      const row = document.createElement("tr");
      for (const value of [result.name, size(result.size), result.sources, result.hash]) { const td = document.createElement("td"); td.textContent = String(value); td.title = String(value); row.append(td); }
      row.children[3].className = "hash";
      const td = document.createElement("td"), copy = document.createElement("button"); copy.type = "button"; copy.textContent = t("copy");
      copy.onclick = () => run("ed2k_copy_link", {link:result.link}, () => message("copied")); td.append(copy);
      const find = document.createElement("button"); find.type="button"; find.dataset.sources="true"; find.textContent=t("findSources");
      find.onclick = () => run("ed2k_find_sources", {hash:result.hash}, sources => { sourceView={name:result.name,sources}; renderSources(); message("sourceCount", {count:sources.length}); });
      td.append(find);
      const download = document.createElement("button"); download.type="button"; download.textContent=t("download");
      download.onclick = () => addDownload(result.link); td.append(download);
      row.append(td); body.append(row);
    }
    $("#search-summary").textContent = results.length ? t("filteredCount", {count:visible.length,total:results.length}) : t("noResults");
    controls();
  }
  function renderDownloads() {
    const body = $("#ed2k-downloads"); body.replaceChildren();
    for (const task of overview?.downloads || []) {
      const row = document.createElement("tr");
      const percent = Math.min(100,Math.max(0,task.verified/task.file.size*100));
      const state = t("stage_"+task.status)+(task.queue_rank ? ` · ${t("queueRank", {rank:task.queue_rank})}` : "");
      for (const value of [task.file.name,size(task.file.size),`${new Intl.NumberFormat(locale,{maximumFractionDigits:1}).format(percent)}%`,state,task.peer || "—"]) {
        const td = document.createElement("td"); td.textContent=String(value); td.title=task.error || `${task.directory} · ${value}`; row.append(td);
      }
      const actions = document.createElement("td");
      const inactive = ["paused","error","completed"].includes(task.status);
      const addAction = (action,key) => {
        const button = document.createElement("button"); button.type="button"; button.textContent=t(key);
        button.dataset.connection=String(action==="resume"); button.dataset.pausing=String(task.status==="pausing");
        button.onclick=()=>run("ed2k_download_action", {id:task.id,action}); actions.append(button);
      };
      if (task.status!=="completed") addAction(inactive ? "resume" : "pause",inactive ? "resume" : "pause");
      if (inactive) addAction("remove","removeTask");
      row.append(actions); body.append(row);
    }
  }
  function addDownload(link) { return run("ed2k_add_download", {link}, () => { $("#ed2k-file-link").value=""; selectTab("transfers"); message("addedDownload"); }); }
  function renderSources() {
    $("#ed2k-source-panel").hidden = !sourceView;
    const body = $("#ed2k-source-results"); body.replaceChildren();
    if (!sourceView) return;
    $("#ed2k-source-title").textContent = t("sourceTitle", {name:sourceView.name});
    for (const source of sourceView.sources) {
      const row = document.createElement("tr");
      for (const value of [source.host || `ID ${source.client_id}`,source.port,t(source.requires_callback ? "callbackPeer" : "directPeer")]) {
        const td = document.createElement("td"); td.textContent=String(value); row.append(td);
      }
      body.append(row);
    }
    $("#ed2k-source-summary").textContent=t("sourceCount", {count:sourceView.sources.length});
  }
  function duration(seconds) { const m=Math.floor(seconds/60), h=Math.floor(m/60); return h ? `${h}h ${m%60}m` : m ? `${m}m ${seconds%60}s` : `${seconds}s`; }
  function cell(row, value, title) { const td = document.createElement("td"); td.textContent = String(value); td.title = String(title ?? value); row.append(td); return td; }
  function button(label, onclick, extra = {}) { const b = document.createElement("button"); b.type = "button"; b.textContent = label; b.onclick = onclick; Object.assign(b.dataset, extra); return b; }
  function renderShared() {
    const data = shared, connected = Boolean(overview?.server);
    const entries = data?.entries || [], files = data?.files || [], scan = data?.scan;
    $("#shared-count").textContent = t("sharedCount", {count:files.length});
    const scanBox = $("#ed2k-scan");
    scanBox.hidden = !scan?.running;
    if (scan?.running) {
      $("#ed2k-scan-bar").max = Math.max(1, scan.total); $("#ed2k-scan-bar").value = scan.done;
      const name = (scan.current || "").split(/[\\/]/).pop();
      $("#ed2k-scan-text").textContent = `${t("hashing", {done:scan.done,total:scan.total})} ${name ? t("hashingNow", {name}) : ""}`.trim();
    }
    const errors = scan?.errors || [];
    $("#ed2k-scan-errors").hidden = errors.length === 0;
    $("#ed2k-scan-errors").textContent = errors.length ? t("scanProblems", {count:errors.length, first:errors[0]}) : "";
    const entryBody = $("#ed2k-share-entries"); entryBody.replaceChildren();
    for (const entry of entries) {
      const row = document.createElement("tr");
      cell(row, entry.path); cell(row, t(entry.kind === "file" ? "kindFile" : "kindDirectory") + (entry.kind === "directory" && entry.recursive ? ` · ${t("includeSubfolders")}` : ""));
      cell(row, entry.files); cell(row, t(entry.enabled ? "sharingOn" : "sharingOff"));
      const actions = document.createElement("td");
      actions.append(button(t(entry.enabled ? "pauseSharing" : "resumeSharing"), () => run("ed2k_shared_set_enabled", {id:entry.id, enabled:!entry.enabled})));
      actions.append(button(t("stopSharing"), () => run("ed2k_shared_remove", {id:entry.id})));
      row.append(actions); entryBody.append(row);
    }
    const fileBody = $("#ed2k-shared-files"); fileBody.replaceChildren();
    for (const file of files) {
      const row = document.createElement("tr");
      cell(row, file.name, file.path); cell(row, size(file.size)); cell(row, file.hash).className = "hash";
      const priority = document.createElement("td"), select = document.createElement("select");
      for (let level = 0; level <= 4; level++) { const option = document.createElement("option"); option.value = String(level); option.textContent = t("priority" + level); select.append(option); }
      select.value = String(file.priority); select.onchange = () => run("ed2k_shared_set_priority", {hash:file.hash, priority:Number(select.value)});
      priority.append(select); row.append(priority);
      cell(row, `${file.requests} / ${file.accepted}`); cell(row, size(file.uploaded));
      const actions = document.createElement("td");
      actions.append(button(t("copy"), () => run("ed2k_copy_link", {link:file.link}, () => message("copied"))));
      actions.append(button(t("openFolder"), () => run("ed2k_shared_open_folder", {hash:file.hash})));
      row.append(actions); fileBody.append(row);
    }
    const published = data?.published;
    $("#ed2k-publish-status").textContent = connected && published ? t("publishedCount", {count:published.count}) : t("notPublished");
  }
  function renderUploads() {
    const uploads = shared?.uploads, names = new Map((shared?.files || []).map(f => [f.hash, f.name]));
    $("#upload-total").textContent = t("totalUploaded", {size:size(uploads?.totalUploaded || 0)});
    if (!settingsDirty && shared?.settings) { $("#ed2k-upload-slots").value = String(shared.settings.upload_slots); $("#ed2k-upload-kib").value = String(shared.settings.upload_kib); }
    const fill = (id, list, columns, empty, summary) => {
      const body = $(id); body.replaceChildren();
      for (const client of list) { const row = document.createElement("tr"); for (const value of columns(client)) cell(row, value); body.append(row); }
      $(summary).textContent = list.length ? "" : t(empty);
    };
    const active = uploads?.active || [], waiting = uploads?.waiting || [];
    fill("#ed2k-upload-active", active, c => [c.name || "—", c.address, names.get(c.file) || c.file, size(c.uploaded), duration(c.seconds)], "noneActive", "#ed2k-upload-active-summary");
    fill("#ed2k-upload-waiting", waiting, c => [c.rank, c.name || "—", c.address, names.get(c.file) || c.file, duration(c.seconds)], "noneWaiting", "#ed2k-upload-waiting-summary");
    if (active.length) $("#ed2k-upload-active-summary").textContent = t("activeCount", {count:active.length, max:uploads.maxActive});
    if (waiting.length) $("#ed2k-upload-waiting-summary").textContent = t("waitingCount", {count:waiting.length});
  }
  function render() {
    document.documentElement.lang = locale;
    for (const node of document.querySelectorAll("[data-i18n]")) node.textContent = t(node.dataset.i18n);
    message(feedback.key, feedback.values, feedback.error);
    $("#ed2k-status").textContent = `ED2K: ${overview?.server ? t("online") : t("offline")}${overview?.low_id ? ` · ${t("lowId")}` : ""}`;
    $("#ed2k-status").dataset.online = String(Boolean(overview?.server));
    $("#incoming-status").textContent = overview?.incoming?.running ? t("incomingPort", {port:overview.incoming.tcp_port,count:overview.incoming.completed_handshakes}) : t("incomingInactive");
    $("#kad-status").textContent = t(overview?.kad_session?.verified_contacts > 0 ? "kadReady" : "kadInactive");
    $("#kad-summary").textContent = overview?.kad_session?.running ? t("kadRouting", {count:overview.kad_session.verified_contacts,total:overview.kad_session.contacts,port:overview.kad_session.udp_port}) : t("nodeCount", {count:overview?.node_count || 0});
    $("#ed2k-bootstrap").textContent=t(overview?.kad_session?.running ? "stopKad" : "bootstrap");
    $("#server-count").textContent = t("serverCount", {count:overview?.servers?.length || 0});
    const body = $("#ed2k-servers"); body.replaceChildren();
    for (const server of overview?.servers || []) {
      const row = document.createElement("tr");
      for (const value of [server.name, server.host, server.port, server.description]) { const td = document.createElement("td"); td.textContent = String(value); td.title = String(value); row.append(td); }
      const td = document.createElement("td"), connect = document.createElement("button"); connect.type = "button"; connect.textContent = t("connect"); connect.onclick = () => run("ed2k_connect", {server}, () => message("connectedTo", {name:server.name})); td.append(connect); row.append(td); body.append(row);
    }
    renderResults(); renderSources(); renderDownloads(); renderShared(); renderUploads(); controls();
  }
  async function refresh() {
    if (!invoke) return;
    overview = await invoke("ed2k_overview");
    if (currentTab === "shared" || currentTab === "uploads" || shared === null) shared = await invoke("ed2k_shared_list");
    render();
  }
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
  $("#ed2k-bootstrap").onclick = () => run(overview?.kad_session?.running ? "ed2k_stop_kad" : "ed2k_start_kad");
  for (const kind of ["servers", "nodes"]) {
    $("#ed2k-update-"+kind).onclick = () => {
      const url = $(kind === "servers" ? "#ed2k-server-url" : "#ed2k-node-url").value.trim();
      try { const parsed = new URL(url); if (parsed.protocol !== "https:" || parsed.username || parsed.password) throw new Error(); }
      catch { message("invalidUrl", {}, true); return; }
      run("ed2k_update_list", {kind,url}, count => message("updated", {count}));
    };
    $("#ed2k-import-"+kind).onclick = () => run("ed2k_import_list", {kind}, count => message(count === null ? "ready" : "imported", {count}));
  }
  $("#ed2k-search-form").onsubmit = event => { event.preventDefault(); const query = $("#ed2k-query").value.trim(); if (query) run("ed2k_search", {query}, value => { results=value; sourceView=null; renderResults(); renderSources(); message("resultCount", {count:results.length}); }); };
  $("#ed2k-add-form").onsubmit = event => { event.preventDefault(); const link=$("#ed2k-file-link").value.trim(); if (link) addDownload(link); };
  for (const [id, kind] of [["#ed2k-share-file", "file"], ["#ed2k-share-folder", "directory"]]) {
    $(id).onclick = () => run("ed2k_shared_add", {kind, recursive:Boolean($("#ed2k-share-recursive").checked)}, entry => { if (entry) message("shareAdded"); });
  }
  $("#ed2k-share-rescan").onclick = () => run("ed2k_shared_rescan");
  for (const id of ["#ed2k-upload-slots", "#ed2k-upload-kib"]) $(id).oninput = () => { settingsDirty = true; };
  $("#ed2k-upload-form").onsubmit = event => {
    event.preventDefault();
    const slots = Number($("#ed2k-upload-slots").value), kib = Number($("#ed2k-upload-kib").value);
    if (!Number.isInteger(slots) || slots < 1 || slots > 64 || !Number.isInteger(kib) || kib < 0 || kib > 1000000) { message("invalidUploadSettings", {}, true); return; }
    run("ed2k_upload_settings", {slots, kib}, () => { settingsDirty = false; message("settingsSaved"); });
  };
  $("#ed2k-result-filter").oninput = renderResults;
  $("#ed2k-result-sort").onchange = renderResults;
  $("#ed2k-clear-log").onclick = () => { logLines.length=0; $("#ed2k-log").textContent=""; };
  function selectTab(name) {
    currentTab = name;
    if (invoke && (name === "shared" || name === "uploads")) refresh().catch(error => log(String(error)));
    for (const button of document.querySelectorAll("[data-tab]")) button.setAttribute("aria-selected",String(button.dataset.tab === name));
    for (const panel of document.querySelectorAll("[role=tabpanel]")) panel.hidden = panel.id !== "panel-"+name;
  }
  for (const tab of document.querySelectorAll("[data-tab]")) tab.onclick = () => selectTab(tab.dataset.tab);
  function language(value) { if (!catalogs[value]) return; locale=value; try { localStorage.setItem("apocalipse.language",locale); } catch {} render(); }
  window.addEventListener("storage", event => { if (event.key === "apocalipse.language") language(event.newValue); if (event.key === "apocalipse.mode") document.documentElement.dataset.mode = event.newValue === "dark" ? "dark" : "light"; });
  let unlisten, unlistenMode;
  window.__TAURI__?.event?.listen("apocalipse-language-changed", event => language(event.payload)).then(stop => { unlisten=stop; }).catch(error => log(String(error)));
  window.__TAURI__?.event?.listen("apocalipse-mode-changed", event => { document.documentElement.dataset.mode = event.payload ? "dark" : "light"; }).then(stop => { unlistenMode=stop; }).catch(error => log(String(error)));
  window.addEventListener("pagehide", () => { unlisten?.(); unlistenMode?.(); }, {once:true});
  render();
  if (invoke) {
    Promise.all([invoke("get_application_language"),invoke("get_dark_mode")]).then(([saved,dark]) => { language(saved); document.documentElement.dataset.mode = dark ? "dark" : "light"; }).catch(error => log(String(error)));
    refresh().catch(error => { message("failed", {}, true); log(String(error)); });
    const timer = setInterval(() => { if (!busy && !document.hidden) refresh().catch(error => log(String(error))); },2000);
    window.addEventListener("pagehide", () => clearInterval(timer), {once:true});
  } else { message("desktopRequired", {}, true); }
})();
