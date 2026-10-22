(() => {
  "use strict";
  const catalogs = {
    en: {
      saveIn: "Save in", chooseDirectory: "Choose folder…", watch: "Preview", previewWaiting: "Waiting for verified video parts", incomingPort: "TCP: {port} · {count} handshakes", incomingInactive: "TCP: inactive",
      transfers: "Transfers", transfersHint: "Choose the destination below; the ADM download folder is suggested. Direct peers, verified ED2K parts and resume are supported; Low-ID sources need a reachable incoming port.", fileLink: "ED2K file link", addDownload: "Add download", download: "Download", addedDownload: "Download added", verifiedProgress: "Verified", status: "Status", pause: "Pause", resume: "Resume", removeTask: "Remove from list", retainedFilesHint: "Removing a finished download only removes it from the list and keeps the file. Removing an unfinished one also deletes its partial file and saved hashes. Unfinished downloads reopen paused and are checked again when resumed.", stage_queued: "Queued", stage_connecting: "Finding a peer", stage_waiting_peer: "Waiting for a peer slot", stage_checking: "Checking saved parts", stage_downloading: "Downloading", stage_pausing: "Pausing…", stage_paused: "Paused", stage_error: "Failed — resume to retry", stage_completed: "Completed", queueRank: "Queue position: {rank}",
      findSources: "Find sources", sourceTitle: "Sources: {name}", sourceHint: "Addresses reported by the connected server. Low-ID peers require callbacks; peer transfers are still in development.", connectionType: "Connection", directPeer: "Direct peer", callbackPeer: "Server callback required", sourceCount: "{count} sources returned by the server", filterResults: "Filter results", sortResults: "Sort by", sortSources: "Most sources", sortSize: "Largest first", sortName: "File name", filteredCount: "{count} of {total} results",
      connect: "Connect ED2K", disconnect: "Disconnect", bootstrap: "Bootstrap Kad", networks: "Networks", search: "Search", preferences: "Preferences", log: "Log", servers: "ED2K servers", serversHint: "Select a server or connect using the initial list.", serverName: "Name", address: "Address", port: "Port", description: "Description", scopeTitle: "Current capabilities", scopeHint: "Native server search, source discovery and verified downloads from direct peers. You choose what to share in the Shared files tab; persistent Kad routing is still in development.", kadHint: "Bootstrap checks a node and retrieves contacts. It does not yet keep a Kad session running.", searchHint: "Search the connected ED2K server, inspect sources or add files to the transfer queue.", query: "File name", file: "File", size: "Size", sources: "Sources", preferencesHint: "Language and colour mode follow the main ADM window.", listUrl: "HTTPS list URL", updateServers: "Update server.met", importServers: "Import server.met", updateNodes: "Update nodes.dat", importNodes: "Import nodes.dat", listSafety: "Lists are size-limited and validated before replacing the current list. Existing data is preserved if validation fails.", clear: "Clear", engine: "Native Rust · Nibiru Xtreme", ready: "Ready", offline: "Disconnected", online: "Connected", lowId: "Low ID · incoming connections unavailable", kadInactive: "Kad: inactive", kadReady: "Kad: bootstrap responded", nodeCount: "{count} initial Kad nodes", kadContacts: "Bootstrap: {count} contacts received from {host}", serverCount: "{count} servers", working: "Working…", failed: "Operation failed", updated: "List updated: {count} entries", imported: "List imported: {count} entries", resultCount: "{count} results", noResults: "No results", copy: "Copy link", copied: "ED2K link copied", desktopRequired: "Open this window from the desktop application.", connectedTo: "Connected to {name}", connectionClosed: "Connection closed", invalidUrl: "Enter an HTTPS URL without credentials.",
    },
    "pt-BR": {
      saveIn: "Salvar em", chooseDirectory: "Escolher pasta…", watch: "Visualizar", previewWaiting: "Aguardando partes verificadas do vídeo", incomingPort: "TCP: {port} · {count} conexões", incomingInactive: "TCP: inativo",
      transfers: "Transferências", transfersHint: "Escolha a pasta abaixo; a pasta de downloads do ADM é a sugestão inicial. Há suporte a peers diretos, partes ED2K verificadas e retomada; fontes com ID baixo precisam de porta de entrada acessível.", fileLink: "Link ED2K do arquivo", addDownload: "Adicionar download", download: "Baixar", addedDownload: "Download adicionado", verifiedProgress: "Verificado", status: "Estado", pause: "Pausar", resume: "Retomar", removeTask: "Remover da lista", retainedFilesHint: "Remover um download concluído só tira da lista e mantém o arquivo. Remover um inacabado também apaga o arquivo parcial e os hashes salvos. Downloads inacabados reabrem pausados e são conferidos novamente ao retomar.", stage_queued: "Na fila", stage_connecting: "Buscando peer", stage_waiting_peer: "Aguardando vaga no peer", stage_checking: "Conferindo partes salvas", stage_downloading: "Baixando", stage_pausing: "Pausando…", stage_paused: "Pausado", stage_error: "Falhou — retome para tentar novamente", stage_completed: "Concluído", queueRank: "Posição na fila: {rank}",
      findSources: "Consultar fontes", sourceTitle: "Fontes: {name}", sourceHint: "Endereços informados pelo servidor conectado. Peers com ID baixo precisam de callback; as transferências entre peers ainda estão em desenvolvimento.", connectionType: "Conexão", directPeer: "Peer direto", callbackPeer: "Exige callback do servidor", sourceCount: "{count} fontes retornadas pelo servidor", filterResults: "Filtrar resultados", sortResults: "Ordenar por", sortSources: "Mais fontes", sortSize: "Maiores primeiro", sortName: "Nome do arquivo", filteredCount: "{count} de {total} resultados",
      connect: "Conectar ED2K", disconnect: "Desconectar", bootstrap: "Iniciar bootstrap Kad", networks: "Redes", search: "Pesquisar", preferences: "Preferências", log: "Log", servers: "Servidores ED2K", serversHint: "Escolha um servidor ou conecte usando a lista inicial.", serverName: "Nome", address: "Endereço", port: "Porta", description: "Descrição", scopeTitle: "Recursos atuais", scopeHint: "Pesquisa de servidores, consulta de fontes e downloads verificados de peers diretos pelo Nibiru nativo. Você escolhe o que compartilhar na aba Compartilhados; o roteamento Kad contínuo ainda está em desenvolvimento.", kadHint: "O bootstrap verifica um nó e recebe contatos. Ele ainda não mantém uma sessão Kad ativa.", searchHint: "Pesquise no servidor ED2K conectado, consulte as fontes ou adicione arquivos à fila de transferências.", query: "Nome do arquivo", file: "Arquivo", size: "Tamanho", sources: "Fontes", preferencesHint: "O idioma e as cores acompanham a janela principal do ADM.", listUrl: "URL HTTPS da lista", updateServers: "Atualizar server.met", importServers: "Importar server.met", updateNodes: "Atualizar nodes.dat", importNodes: "Importar nodes.dat", listSafety: "As listas têm limite de tamanho e são validadas antes da substituição. Os dados existentes são preservados se a validação falhar.", clear: "Limpar", engine: "Rust nativo · Nibiru Xtreme", ready: "Pronto", offline: "Desconectado", online: "Conectado", lowId: "ID baixo · conexões de entrada indisponíveis", kadInactive: "Kad: inativo", kadReady: "Kad: bootstrap respondeu", nodeCount: "{count} nós Kad iniciais", kadContacts: "Bootstrap: {count} contatos recebidos de {host}", serverCount: "{count} servidores", working: "Processando…", failed: "Falha na operação", updated: "Lista atualizada: {count} entradas", imported: "Lista importada: {count} entradas", resultCount: "{count} resultados", noResults: "Nenhum resultado", copy: "Copiar link", copied: "Link ED2K copiado", desktopRequired: "Abra esta janela pelo aplicativo desktop.", connectedTo: "Conectado a {name}", connectionClosed: "Conexão encerrada", invalidUrl: "Informe uma URL HTTPS sem credenciais.",
    },
    "zh-CN": {
      saveIn: "保存到", chooseDirectory: "选择文件夹…", watch: "预览", previewWaiting: "等待已验证的视频分块",
      incomingPort: "TCP：{port} · {count} 次握手", incomingInactive: "TCP：未启用",
      transfers: "传输", transfersHint: "文件保存到 ADM 下载目录。支持直接节点、ED2K 分块校验和续传；低 ID 来源需要可达的入站端口。", fileLink: "ED2K 文件链接", addDownload: "添加下载", download: "下载", addedDownload: "已添加下载", verifiedProgress: "已验证", status: "状态", pause: "暂停", resume: "继续", removeTask: "从列表移除", retainedFilesHint: "移除已完成的下载只会从列表中移除并保留文件；移除未完成的下载还会删除其临时文件和已保存的哈希。未完成任务重新打开后暂停，继续时会再次校验。", stage_queued: "排队中", stage_connecting: "查找节点", stage_waiting_peer: "等待节点传输位", stage_checking: "验证已保存分块", stage_downloading: "下载中", stage_pausing: "正在暂停…", stage_paused: "已暂停", stage_error: "失败，可继续重试", stage_completed: "已完成", queueRank: "排队位置：{rank}",
      findSources: "查找来源", sourceTitle: "来源：{name}", sourceHint: "已连接服务器返回的地址。低 ID 节点需要回调；节点传输仍在开发中。", connectionType: "连接", directPeer: "直接连接", callbackPeer: "需要服务器回调", sourceCount: "服务器返回 {count} 个来源", filterResults: "筛选结果", sortResults: "排序", sortSources: "来源最多", sortSize: "文件最大", sortName: "文件名", filteredCount: "{total} 个结果中的 {count} 个",
      connect: "连接 ED2K", disconnect: "断开连接", bootstrap: "引导 Kad", networks: "网络", search: "搜索", preferences: "首选项", log: "日志", servers: "ED2K 服务器", serversHint: "选择服务器，或使用初始列表连接。", serverName: "名称", address: "地址", port: "端口", description: "说明", scopeTitle: "当前功能", scopeHint: "支持原生服务器搜索、来源查询和直接节点的验证下载。您可在“共享文件”页选择要共享的内容；持续 Kad 路由仍在开发中。", kadHint: "引导会检测节点并获取联系人，目前不会保持持续的 Kad 会话。", searchHint: "搜索已连接的 ED2K 服务器，查询来源或将文件加入传输队列。", query: "文件名", file: "文件", size: "大小", sources: "来源", preferencesHint: "语言和颜色模式跟随 ADM 主窗口。", listUrl: "列表的 HTTPS 地址", updateServers: "更新 server.met", importServers: "导入 server.met", updateNodes: "更新 nodes.dat", importNodes: "导入 nodes.dat", listSafety: "列表有大小限制，验证通过后才替换当前列表。验证失败时保留原有数据。", clear: "清除", engine: "原生 Rust · Nibiru Xtreme", ready: "就绪", offline: "未连接", online: "已连接", lowId: "低 ID · 入站连接不可用", kadInactive: "Kad：未启用", kadReady: "Kad：引导节点已响应", nodeCount: "{count} 个初始 Kad 节点", kadContacts: "引导：从 {host} 收到 {count} 个联系人", serverCount: "{count} 个服务器", working: "正在处理…", failed: "操作失败", updated: "列表已更新：{count} 条", imported: "列表已导入：{count} 条", resultCount: "{count} 个结果", noResults: "没有结果", copy: "复制链接", copied: "已复制 ED2K 链接", desktopRequired: "请从桌面应用程序打开此窗口。", connectedTo: "已连接至 {name}", connectionClosed: "连接已关闭", invalidUrl: "请输入不含凭据的 HTTPS 地址。",
    },
  };
  Object.assign(catalogs.en, {
      shares: "Shared files", sharesHint: "Nothing is shared until you add it here. Files you add are offered to other ED2K users while you are connected to a server. Hidden files and ADM's own data are never shared. As in aMule, choose a folder (with or without its subfolders) or single files.", shareAddFolder: "Add folder…", shareAddFiles: "Add files…", shareRescan: "Rescan", shareSubfolders: "Include subfolders", shareFolders: "Shared folders", shareFolder: "Folder", shareScope: "Scope", shareWithSub: "With subfolders", shareOnlyThis: "This folder only", shareFiles: "Shared files", sharePriority: "Priority", shareRequests: "Requests", shareUploaded: "Uploaded", shareStop: "Stop sharing", shareNoFolders: "No shared folders.", shareNoFiles: "Nothing is being shared.", prioVeryLow: "Very low", prioLow: "Low", prioNormal: "Normal", prioHigh: "High", prioRelease: "Release", shareHashing: "Preparing {done}/{total}: {name}", sharePublished: "{count} files offered to the server", shareNotConnected: "Connect to an ED2K server to offer these files. They are only served while this window is connected.", sharePublishFailed: "Could not offer the files to the server", shareLimits: "Upload limits", shareLimitKib: "Upload limit (KiB/s, 0 = unlimited)", shareSlots: "Upload slots", shareApply: "Apply", shareUploads: "Uploads: {active} active, {waiting} waiting · {total} sent", shareClient: "Client", shareActive: "Uploading", shareWaiting: "Queue position {rank}", shareFolderAdded: "Folder added: {path}", shareFilesAdded: "{count} files added", shareLimitsApplied: "Limits applied", shareTooBroad: "Choose a more specific folder: drive roots and folders that contain ADM's data are not allowed.", shareFailedFiles: "Some files could not be read: {error}",
      kadConnect: "Connect Kad", kadDisconnect: "Disconnect Kad", kadStatusConnecting: "Kad: connecting…", kadStatusConnected: "Kad: connected · {verified}/{contacts} contacts", kadSummaryRunning: "Kad session on UDP {udp}: {contacts} contacts, {verified} verified. Used to find sources for downloads.", kadHint: "Kad runs as a real session: it keeps a routing table and looks up sources for downloads. Keyword search and publishing are not available yet.", scopeHint: "Native server search, source discovery through the server and Kad, verified downloads, and sharing of the files you choose. Credits, AICH, messaging and Kad keyword search or publishing are not available yet.",
      upnpOff: "UPnP: off", upnpIdle: "UPnP: idle", upnpSearching: "UPnP: looking for a router…", upnpMapped: "UPnP: ports forwarded", upnpMappedIp: "UPnP: ports forwarded · external IP {ip}", upnpPartial: "UPnP: some ports could not be forwarded", upnpFailed: "UPnP: no router found", networkPrefs: "Network", obfuscationLabel: "Protocol obfuscation", obfOff: "Off", obfPrefer: "Preferred", obfRequire: "Required (refuses plain peers)", upnpLabel: "Ask the router (UPnP) to forward the incoming ports", upnpHint: "Only while connected; the forwarding is removed when you disconnect or close ADM. It does not work behind CGNAT or when the router has UPnP turned off.", kadStarted: "Kad connected", kadStopped: "Kad disconnected", prefsSaved: "Preferences saved",
      stage_waiting_sources: "Waiting for sources — retrying automatically", retryNow: "Retry now", nextAttempt: "next attempt in {seconds} s", waitingHint: "Verified parts are kept. A download that cannot finish keeps asking for sources and tries again until you pause it; connecting to a server or Kad tries again at once.",
      stall_not_connected: "Not connected to an ED2K server or Kad", stall_no_sources: "No sources found right now", stall_unreachable: "Sources found, but none could be reached", stall_metadata_timeout: "Peers did not send the file information in time", stall_missing_parts: "The connected peers do not have the missing parts yet", stall_queue_timeout: "Peers have the parts but kept us waiting in their queue", stall_peer_refused: "Peers refused or do not share the file", stall_part_failed: "A peer failed while sending a part", stall_sources_failed: "Could not ask for sources",
  });
  Object.assign(catalogs["pt-BR"], {
      shares: "Compartilhados", sharesHint: "Nada é compartilhado até você adicionar aqui. Os arquivos adicionados são oferecidos a outros usuários ED2K enquanto você estiver conectado a um servidor. Arquivos ocultos e os dados do próprio ADM nunca são compartilhados. Como no aMule, escolha uma pasta (com ou sem subpastas) ou arquivos avulsos.", shareAddFolder: "Adicionar pasta…", shareAddFiles: "Adicionar arquivos…", shareRescan: "Reanalisar", shareSubfolders: "Incluir subpastas", shareFolders: "Pastas compartilhadas", shareFolder: "Pasta", shareScope: "Alcance", shareWithSub: "Com subpastas", shareOnlyThis: "Somente esta pasta", shareFiles: "Arquivos compartilhados", sharePriority: "Prioridade", shareRequests: "Pedidos", shareUploaded: "Enviado", shareStop: "Parar de compartilhar", shareNoFolders: "Nenhuma pasta compartilhada.", shareNoFiles: "Nada está sendo compartilhado.", prioVeryLow: "Muito baixa", prioLow: "Baixa", prioNormal: "Normal", prioHigh: "Alta", prioRelease: "Lançamento", shareHashing: "Preparando {done}/{total}: {name}", sharePublished: "{count} arquivos oferecidos ao servidor", shareNotConnected: "Conecte-se a um servidor ED2K para oferecer estes arquivos. Eles só são servidos enquanto esta janela estiver conectada.", sharePublishFailed: "Não foi possível oferecer os arquivos ao servidor", shareLimits: "Limites de upload", shareLimitKib: "Limite de upload (KiB/s, 0 = sem limite)", shareSlots: "Vagas de upload", shareApply: "Aplicar", shareUploads: "Uploads: {active} ativos, {waiting} na fila · {total} enviados", shareClient: "Cliente", shareActive: "Enviando", shareWaiting: "Posição na fila {rank}", shareFolderAdded: "Pasta adicionada: {path}", shareFilesAdded: "{count} arquivos adicionados", shareLimitsApplied: "Limites aplicados", shareTooBroad: "Escolha uma pasta mais específica: raízes de disco e pastas que contêm os dados do ADM não são permitidas.", shareFailedFiles: "Alguns arquivos não puderam ser lidos: {error}",
      kadConnect: "Conectar Kad", kadDisconnect: "Desconectar Kad", kadStatusConnecting: "Kad: conectando…", kadStatusConnected: "Kad: conectado · {verified}/{contacts} contatos", kadSummaryRunning: "Sessão Kad no UDP {udp}: {contacts} contatos, {verified} verificados. Usada para achar fontes dos downloads.", kadHint: "O Kad roda como sessão de verdade: mantém uma tabela de roteamento e procura fontes para os downloads. Busca por palavra-chave e publicação ainda não estão disponíveis.", scopeHint: "Pesquisa no servidor nativo, busca de fontes pelo servidor e pelo Kad, downloads verificados e compartilhamento dos arquivos que você escolher. Créditos, AICH, mensagens e busca ou publicação por palavra-chave no Kad ainda não estão disponíveis.",
      upnpOff: "UPnP: desligado", upnpIdle: "UPnP: em espera", upnpSearching: "UPnP: procurando o roteador…", upnpMapped: "UPnP: portas encaminhadas", upnpMappedIp: "UPnP: portas encaminhadas · IP externo {ip}", upnpPartial: "UPnP: algumas portas não puderam ser encaminhadas", upnpFailed: "UPnP: roteador não encontrado", networkPrefs: "Rede", obfuscationLabel: "Ofuscação do protocolo", obfOff: "Desligada", obfPrefer: "Preferida", obfRequire: "Obrigatória (recusa peers sem ofuscação)", upnpLabel: "Pedir ao roteador (UPnP) que encaminhe as portas de entrada", upnpHint: "Só enquanto estiver conectado; o encaminhamento é removido ao desconectar ou fechar o ADM. Não funciona atrás de CGNAT nem com o UPnP desligado no roteador.", kadStarted: "Kad conectado", kadStopped: "Kad desconectado", prefsSaved: "Preferências salvas",
      stage_waiting_sources: "Aguardando fontes — tentando de novo automaticamente", retryNow: "Tentar agora", nextAttempt: "próxima tentativa em {seconds} s", waitingHint: "As partes verificadas são mantidas. Um download que não consegue terminar continua pedindo fontes e tentando de novo até você pausar; conectar a um servidor ou ao Kad tenta de novo na hora.",
      stall_not_connected: "Sem conexão com um servidor ED2K nem com o Kad", stall_no_sources: "Nenhuma fonte encontrada agora", stall_unreachable: "Fontes encontradas, mas nenhuma pôde ser alcançada", stall_metadata_timeout: "Os peers não enviaram as informações do arquivo a tempo", stall_missing_parts: "Os peers conectados ainda não têm as partes que faltam", stall_queue_timeout: "Os peers têm as partes, mas nos mantiveram na fila de espera", stall_peer_refused: "Os peers recusaram ou não compartilham o arquivo", stall_part_failed: "Um peer falhou ao enviar uma parte", stall_sources_failed: "Não foi possível pedir fontes",
  });
  Object.assign(catalogs["zh-CN"], {
      shares: "共享文件", sharesHint: "只有在此处添加后才会共享。连接到服务器期间，您添加的文件会提供给其他 ED2K 用户。隐藏文件和 ADM 自身的数据永远不会被共享。与 aMule 一样，可选择文件夹（含或不含子文件夹）或单个文件。", shareAddFolder: "添加文件夹…", shareAddFiles: "添加文件…", shareRescan: "重新扫描", shareSubfolders: "包含子文件夹", shareFolders: "共享文件夹", shareFolder: "文件夹", shareScope: "范围", shareWithSub: "包含子文件夹", shareOnlyThis: "仅此文件夹", shareFiles: "共享的文件", sharePriority: "优先级", shareRequests: "请求", shareUploaded: "已上传", shareStop: "停止共享", shareNoFolders: "没有共享文件夹。", shareNoFiles: "当前没有共享任何内容。", prioVeryLow: "很低", prioLow: "低", prioNormal: "普通", prioHigh: "高", prioRelease: "发布", shareHashing: "正在准备 {done}/{total}：{name}", sharePublished: "已向服务器提供 {count} 个文件", shareNotConnected: "请连接 ED2K 服务器以提供这些文件。仅在此窗口保持连接时才会提供。", sharePublishFailed: "无法向服务器提供文件", shareLimits: "上传限制", shareLimitKib: "上传限制（KiB/秒，0 = 不限）", shareSlots: "上传位", shareApply: "应用", shareUploads: "上传：{active} 个进行中，{waiting} 个排队 · 已发送 {total}", shareClient: "客户端", shareActive: "上传中", shareWaiting: "排队位置 {rank}", shareFolderAdded: "已添加文件夹：{path}", shareFilesAdded: "已添加 {count} 个文件", shareLimitsApplied: "限制已应用", shareTooBroad: "请选择更具体的文件夹：不允许磁盘根目录和包含 ADM 数据的文件夹。", shareFailedFiles: "部分文件无法读取：{error}",
      kadConnect: "连接 Kad", kadDisconnect: "断开 Kad", kadStatusConnecting: "Kad：正在连接…", kadStatusConnected: "Kad：已连接 · {verified}/{contacts} 个联系人", kadSummaryRunning: "Kad 会话运行于 UDP {udp}：{contacts} 个联系人，{verified} 个已验证。用于查找下载来源。", kadHint: "Kad 作为真实会话运行：维护路由表并为下载查找来源。关键词搜索和发布暂不可用。", scopeHint: "支持原生服务器搜索、通过服务器和 Kad 查找来源、验证下载，以及共享您选择的文件。积分、AICH、消息，以及 Kad 关键词搜索或发布暂不可用。",
      upnpOff: "UPnP：已关闭", upnpIdle: "UPnP：空闲", upnpSearching: "UPnP：正在查找路由器…", upnpMapped: "UPnP：端口已转发", upnpMappedIp: "UPnP：端口已转发 · 外部 IP {ip}", upnpPartial: "UPnP：部分端口无法转发", upnpFailed: "UPnP：未找到路由器", networkPrefs: "网络", obfuscationLabel: "协议混淆", obfOff: "关闭", obfPrefer: "优先", obfRequire: "必须（拒绝未混淆的节点）", upnpLabel: "请求路由器（UPnP）转发入站端口", upnpHint: "仅在连接期间有效；断开连接或关闭 ADM 时会移除转发。在 CGNAT 后或路由器关闭 UPnP 时不起作用。", kadStarted: "Kad 已连接", kadStopped: "Kad 已断开", prefsSaved: "首选项已保存",
      stage_waiting_sources: "等待来源，自动重试", retryNow: "立即重试", nextAttempt: "{seconds} 秒后重试", waitingHint: "已验证的分块会保留。无法完成的下载会持续查询来源并重试，直到您暂停；连接服务器或 Kad 时会立即重试。",
      stall_not_connected: "未连接到 ED2K 服务器或 Kad", stall_no_sources: "目前没有找到来源", stall_unreachable: "找到来源，但都无法连接", stall_metadata_timeout: "节点未及时发送文件信息", stall_missing_parts: "已连接的节点还没有缺少的分块", stall_queue_timeout: "节点有所需分块，但让我们在队列中等待", stall_peer_refused: "节点拒绝或未共享该文件", stall_part_failed: "节点在发送分块时失败", stall_sources_failed: "无法查询来源",
  });
  Object.assign(catalogs.en, {"logAll": "All levels", "logWarnings": "Warnings and errors", "logErrors": "Errors only", "logAreaAll": "All areas", "area_server": "ED2K server", "area_kad": "Kad", "area_upnp": "UPnP", "area_download": "Download", "area_peer": "Peers", "area_share": "Sharing", "area_ui": "Window", "logSearch": "Search the log", "logFollow": "Follow", "logCopy": "Copy", "logSave": "Save to file", "logAsk": "Ask where to save", "logSaveCancelled": "Save cancelled", "logCopied": "Log copied", "logSavedTo": "Log saved to {path}", "logSaveFailed": "Could not save the log: {error}", "logSummary": "{shown} of {total} entries · {warnings} warnings · {errors} errors", "logEmpty": "Nothing logged yet. Connect to ED2K or start a download.", "kind_unreachable": "could not be reached", "kind_handshake_timeout": "did not answer the handshake", "kind_metadata_timeout": "did not send file info in time", "kind_refused": "refused", "kind_queue_timeout": "upload queue too long", "ev_connecting": "Connecting to an ED2K server — candidates: {candidates}", "ev_connected_high_id": "Connected to {name} ({address}) — High ID {client_id}. UPnP: {upnp}; TCP port {tcp}", "ev_connected_low_id": "Connected to {name} ({address}) — LOW ID {client_id}: the server could not reach the incoming TCP port {tcp} (UPnP: {upnp} {upnp_error}). Sources that need a callback will not work through this server.", "ev_server_failed": "Server {address} failed: {error}", "ev_all_servers_failed": "No ED2K server accepted the connection: {error}", "ev_disconnected": "Disconnected from {name}", "ev_upnp_before_login": "UPnP: forwarding TCP port {tcp} before the login", "ev_upnp_skipped_login": "UPnP: a recent search found no router, so the login does not wait for it ({error})", "ev_upnp_login_wait_expired": "UPnP: the router did not answer within {seconds} s; logging in anyway (probably Low ID)", "ev_upnp_searching": "UPnP: looking for a router…", "ev_upnp_mapped": "UPnP: ports forwarded via {gateway} — external IP {external_ip} — {ports}", "ev_upnp_partial": "UPnP: some ports could not be forwarded ({error}) — {ports}", "ev_upnp_failed": "UPnP: no router found — {error}", "ev_kad_started": "Kad started on UDP {udp} with {seed_nodes} seed nodes", "ev_kad_stopped": "Kad stopped", "ev_attempt": "{name}: new pass — {sources} sources, {verified} of {size} verified (server: {server}, Kad: {kad})", "ev_server_sources": "{name}: the server returned {count} sources", "ev_server_sources_failed": "{name}: the server returned no sources (try {try}): {error}", "ev_kad_sources": "{name}: Kad returned {count} sources", "ev_kad_sources_failed": "{name}: the Kad search failed: {error}", "ev_waiting": "{name}: pass {attempt} ended — {reason}. Next try in {next_in_s}. {detail}", "ev_completed": "{name}: download completed and verified", "ev_paused": "{name}: paused", "ev_failed": "{name}: failed — {error}", "ev_peer_connecting": "{name}: connecting to {peer}", "ev_peer_failed": "{name}: {peer} unusable — {kind}: {error}", "ev_peer_ready": "{name}: {peer} is ready — it has {offers} parts, {wanted} of them still missing here", "ev_peer_nothing_new": "{name}: {peer} has none of the missing parts", "ev_peer_queued": "{name}: {peer} queued us — position {rank}", "ev_peer_queue_failed": "{name}: {peer} gave no upload slot — {kind}: {error}", "ev_peer_slot": "{name}: {peer} granted a slot after {waited_s}", "ev_part_started": "{name}: part {part} requested from {peer}", "ev_part_helping": "{name}: {peer} takes blocks of part {part} too (helping)", "ev_part_slow": "{name}: part {part} from {peer} is slow — {received} in {elapsed_s}", "ev_part_done": "{name}: part {part} verified ({bytes}, blocks from {peers} peers, last one from {peer}, {elapsed_s})", "ev_part_failed": "{name}: part {part} from {peer} failed after {elapsed_s} — {error}", "ev_published": "Offered {count} shared files to the server", "ev_publish_failed": "Could not offer the shared files: {error}"});
  Object.assign(catalogs["pt-BR"], {"logAll": "Todos os níveis", "logWarnings": "Avisos e erros", "logErrors": "Só erros", "logAreaAll": "Todas as áreas", "area_server": "Servidor ED2K", "area_kad": "Kad", "area_upnp": "UPnP", "area_download": "Download", "area_peer": "Peers", "area_share": "Compartilhamento", "area_ui": "Janela", "logSearch": "Buscar no log", "logFollow": "Acompanhar", "logCopy": "Copiar", "logSave": "Salvar em arquivo", "logAsk": "Perguntar onde salvar", "logSaveCancelled": "Salvamento cancelado", "logCopied": "Log copiado", "logSavedTo": "Log salvo em {path}", "logSaveFailed": "Não foi possível salvar o log: {error}", "logSummary": "{shown} de {total} registros · {warnings} avisos · {errors} erros", "logEmpty": "Nada registrado ainda. Conecte ao ED2K ou inicie um download.", "kind_unreachable": "não foi alcançado", "kind_handshake_timeout": "não respondeu ao handshake", "kind_metadata_timeout": "não enviou as informações do arquivo a tempo", "kind_refused": "recusou", "kind_queue_timeout": "fila de envio longa demais", "ev_connecting": "Conectando a um servidor ED2K — candidatos: {candidates}", "ev_connected_high_id": "Conectado a {name} ({address}) — ID alto {client_id}. UPnP: {upnp}; porta TCP {tcp}", "ev_connected_low_id": "Conectado a {name} ({address}) — ID BAIXO {client_id}: o servidor não alcançou a porta TCP de entrada {tcp} (UPnP: {upnp} {upnp_error}). Fontes que exigem retorno de chamada não funcionam por este servidor.", "ev_server_failed": "Servidor {address} falhou: {error}", "ev_all_servers_failed": "Nenhum servidor ED2K aceitou a conexão: {error}", "ev_disconnected": "Desconectado de {name}", "ev_upnp_before_login": "UPnP: encaminhando a porta TCP {tcp} antes do login", "ev_upnp_skipped_login": "UPnP: uma busca recente não achou roteador, então o login não espera por ele ({error})", "ev_upnp_login_wait_expired": "UPnP: o roteador não respondeu em {seconds} s; entrando mesmo assim (provavelmente ID baixo)", "ev_upnp_searching": "UPnP: procurando o roteador…", "ev_upnp_mapped": "UPnP: portas encaminhadas via {gateway} — IP externo {external_ip} — {ports}", "ev_upnp_partial": "UPnP: algumas portas não puderam ser encaminhadas ({error}) — {ports}", "ev_upnp_failed": "UPnP: roteador não encontrado — {error}", "ev_kad_started": "Kad iniciado no UDP {udp} com {seed_nodes} nós iniciais", "ev_kad_stopped": "Kad parado", "ev_attempt": "{name}: nova rodada — {sources} fontes, {verified} de {size} verificados (servidor: {server}, Kad: {kad})", "ev_server_sources": "{name}: o servidor devolveu {count} fontes", "ev_server_sources_failed": "{name}: o servidor não devolveu fontes (tentativa {try}): {error}", "ev_kad_sources": "{name}: o Kad devolveu {count} fontes", "ev_kad_sources_failed": "{name}: a busca no Kad falhou: {error}", "ev_waiting": "{name}: rodada {attempt} terminou — {reason}. Próxima tentativa em {next_in_s}. {detail}", "ev_completed": "{name}: download concluído e verificado", "ev_paused": "{name}: pausado", "ev_failed": "{name}: falhou — {error}", "ev_peer_connecting": "{name}: conectando a {peer}", "ev_peer_failed": "{name}: {peer} inutilizável — {kind}: {error}", "ev_peer_ready": "{name}: {peer} está pronto — tem {offers} partes, {wanted} delas ainda faltam aqui", "ev_peer_nothing_new": "{name}: {peer} não tem nenhuma das partes que faltam", "ev_peer_queued": "{name}: {peer} nos colocou na fila — posição {rank}", "ev_peer_queue_failed": "{name}: {peer} não liberou vaga de envio — {kind}: {error}", "ev_peer_slot": "{name}: {peer} liberou uma vaga após {waited_s}", "ev_part_started": "{name}: parte {part} pedida a {peer}", "ev_part_helping": "{name}: {peer} também pega blocos da parte {part} (ajudando)", "ev_part_slow": "{name}: parte {part} de {peer} está lenta — {received} em {elapsed_s}", "ev_part_done": "{name}: parte {part} verificada ({bytes}, blocos de {peers} peers, o último de {peer}, em {elapsed_s})", "ev_part_failed": "{name}: parte {part} de {peer} falhou após {elapsed_s} — {error}", "ev_published": "{count} arquivos compartilhados oferecidos ao servidor", "ev_publish_failed": "Não foi possível oferecer os arquivos compartilhados: {error}"});
  Object.assign(catalogs["zh-CN"], {"logAll": "所有级别", "logWarnings": "警告和错误", "logErrors": "仅错误", "logAreaAll": "所有类别", "area_server": "ED2K 服务器", "area_kad": "Kad", "area_upnp": "UPnP", "area_download": "下载", "area_peer": "节点", "area_share": "共享", "area_ui": "窗口", "logSearch": "搜索日志", "logFollow": "跟随", "logCopy": "复制", "logSave": "保存到文件", "logAsk": "保存时询问位置", "logSaveCancelled": "已取消保存", "logCopied": "日志已复制", "logSavedTo": "日志已保存到 {path}", "logSaveFailed": "无法保存日志：{error}", "logSummary": "{shown} / {total} 条 · {warnings} 条警告 · {errors} 条错误", "logEmpty": "暂无记录。请连接 ED2K 或开始下载。", "kind_unreachable": "无法连接", "kind_handshake_timeout": "握手无响应", "kind_metadata_timeout": "未及时发送文件信息", "kind_refused": "已拒绝", "kind_queue_timeout": "上传队列过长", "ev_connecting": "正在连接 ED2K 服务器 — 候选：{candidates}", "ev_connected_high_id": "已连接到 {name}（{address}）— 高 ID {client_id}。UPnP：{upnp}；TCP 端口 {tcp}", "ev_connected_low_id": "已连接到 {name}（{address}）— 低 ID {client_id}：服务器无法访问入站 TCP 端口 {tcp}（UPnP：{upnp} {upnp_error}）。需要回调的来源无法通过此服务器使用。", "ev_server_failed": "服务器 {address} 失败：{error}", "ev_all_servers_failed": "没有 ED2K 服务器接受连接：{error}", "ev_disconnected": "已断开 {name}", "ev_upnp_before_login": "UPnP：登录前转发 TCP 端口 {tcp}", "ev_upnp_skipped_login": "UPnP：最近一次搜索未找到路由器，登录不会等待它（{error}）", "ev_upnp_login_wait_expired": "UPnP：路由器在 {seconds} 秒内未响应；仍然登录（可能是低 ID）", "ev_upnp_searching": "UPnP：正在查找路由器…", "ev_upnp_mapped": "UPnP：端口已通过 {gateway} 转发 — 外部 IP {external_ip} — {ports}", "ev_upnp_partial": "UPnP：部分端口无法转发（{error}）— {ports}", "ev_upnp_failed": "UPnP：未找到路由器 — {error}", "ev_kad_started": "Kad 已在 UDP {udp} 启动，种子节点 {seed_nodes} 个", "ev_kad_stopped": "Kad 已停止", "ev_attempt": "{name}：新一轮 — {sources} 个来源，已验证 {verified} / {size}（服务器：{server}，Kad：{kad}）", "ev_server_sources": "{name}：服务器返回 {count} 个来源", "ev_server_sources_failed": "{name}：服务器未返回来源（第 {try} 次）：{error}", "ev_kad_sources": "{name}：Kad 返回 {count} 个来源", "ev_kad_sources_failed": "{name}：Kad 搜索失败：{error}", "ev_waiting": "{name}：第 {attempt} 轮结束 — {reason}。{next_in_s} 后重试。{detail}", "ev_completed": "{name}：下载完成并已验证", "ev_paused": "{name}：已暂停", "ev_failed": "{name}：失败 — {error}", "ev_peer_connecting": "{name}：正在连接 {peer}", "ev_peer_failed": "{name}：{peer} 不可用 — {kind}：{error}", "ev_peer_ready": "{name}：{peer} 就绪 — 拥有 {offers} 个分块，其中 {wanted} 个本地仍缺失", "ev_peer_nothing_new": "{name}：{peer} 没有任何缺失的分块", "ev_peer_queued": "{name}：{peer} 将我们排入队列 — 位置 {rank}", "ev_peer_queue_failed": "{name}：{peer} 未提供上传名额 — {kind}：{error}", "ev_peer_slot": "{name}：{peer} 在 {waited_s} 后提供了名额", "ev_part_started": "{name}：向 {peer} 请求分块 {part}", "ev_part_helping": "{name}：{peer} 也在获取分块 {part} 的数据块（协助）", "ev_part_slow": "{name}：来自 {peer} 的分块 {part} 很慢 — {elapsed_s} 内收到 {received}", "ev_part_done": "{name}：分块 {part} 已验证（{bytes}，来自 {peers} 个节点的数据块，最后一块来自 {peer}，用时 {elapsed_s}）", "ev_part_failed": "{name}：来自 {peer} 的分块 {part} 在 {elapsed_s} 后失败 — {error}", "ev_published": "已向服务器提供 {count} 个共享文件", "ev_publish_failed": "无法提供共享文件：{error}"});
  let locale = "en";
  try { const cached = localStorage.getItem("apocalipse.language"); if (catalogs[cached]) locale = cached; } catch {}
  const invoke = window.__TAURI__?.core?.invoke;
  const $ = selector => document.querySelector(selector);
  const t = (key, values = {}) => Object.entries(values).reduce((s, [k,v]) => s.replaceAll(`{${k}}`, String(v)), catalogs[locale][key] || catalogs.en[key] || key);
  let overview = null, results = [], sourceView = null, busy = false;
  let feedback = { key: "ready", values: {}, error: false };
  // The log shows entries from the backend (what the network layer and the downloads did and why)
  // plus what this window did. Nothing here is inserted as HTML.
  const entries = [];
  let lastEventId = 0, clearedAt = 0, localSeq = 0;
  const pad = (value, width = 2) => String(value).padStart(width, "0");
  const clock = at => { const d = new Date(at); return `${pad(d.getHours())}:${pad(d.getMinutes())}:${pad(d.getSeconds())}.${pad(d.getMilliseconds(), 3)}`; };
  function message(key, values = {}, error = false) { feedback = { key, values, error }; $("#ed2k-feedback").textContent = t(key, values); $("#ed2k-feedback").dataset.error = String(error); }
  function log(text, level = "info") { entries.push({ id: 0, seq: ++localSeq, at: Date.now(), level, area: "ui", code: "", fields: {}, text }); if (entries.length > 3000) entries.shift(); renderLog(); }
  function displayFields(fields) {
    const shown = {};
    for (const [key, value] of Object.entries(fields || {})) {
      let text;
      if (value === null || value === undefined || value === "") text = "—";
      else if (Array.isArray(value)) text = value.length ? value.join(", ") : "—";
      else if (typeof value === "boolean") text = value ? "✓" : "✗";
      else if (key.endsWith("_s") && typeof value === "number") text = `${new Intl.NumberFormat(locale, { maximumFractionDigits: 1 }).format(value)} s`;
      else if (["received", "bytes", "verified", "size"].includes(key) && typeof value === "number") text = size(value);
      else if (key === "kind" || key === "reason") text = t((key === "kind" ? "kind_" : "stall_") + value);
      else text = String(value);
      shown[key] = text;
    }
    return shown;
  }
  function entryText(entry) {
    if (entry.text) return entry.text;
    const key = "ev_" + entry.code;
    return catalogs[locale][key] || catalogs.en[key] ? t(key, displayFields(entry.fields)) : `${entry.code} ${JSON.stringify(entry.fields)}`;
  }
  function visibleEntries() {
    const level = $("#ed2k-log-level").value || "all", area = $("#ed2k-log-area").value || "all";
    const query = ($("#ed2k-log-search").value || "").trim().toLowerCase();
    return entries.filter(entry => (entry.id === 0 || entry.id > clearedAt)
      && (level === "all" || (level === "warn" ? entry.level !== "info" : entry.level === "error"))
      && (area === "all" || entry.area === area)
      && (!query || entryText(entry).toLowerCase().includes(query)));
  }
  function renderLog() {
    const box = $("#ed2k-log"), shown = visibleEntries().sort((a, b) => a.at - b.at || a.id - b.id || (a.seq || 0) - (b.seq || 0));
    const live = entries.filter(entry => entry.id === 0 || entry.id > clearedAt);
    const rows = shown.slice(-1500).map(entry => {
      const row = document.createElement("div");
      row.dataset.level = entry.level;
      row.textContent = `${clock(entry.at)}  ${t("area_" + entry.area)}  ${entryText(entry)}`;
      row.title = entry.code ? `${entry.code} ${JSON.stringify(entry.fields)}` : "";
      return row;
    });
    if (!rows.length) { const empty = document.createElement("div"); empty.dataset.level = "empty"; empty.textContent = t("logEmpty"); rows.push(empty); }
    box.replaceChildren(...rows);
    $("#ed2k-log-summary").textContent = t("logSummary", { shown: shown.length, total: live.length, warnings: live.filter(e => e.level === "warn").length, errors: live.filter(e => e.level === "error").length });
    if ($("#ed2k-log-follow").checked !== false) box.scrollTop = box.scrollHeight;
  }
  function logAsText() {
    return [`Apocalipse ED2K log — ${new Date().toISOString()} — ${visibleEntries().length} entries`, ...visibleEntries().sort((a, b) => a.at - b.at || a.id - b.id || (a.seq || 0) - (b.seq || 0)).map(entry => {
      const stamp = new Date(entry.at), date = `${stamp.getFullYear()}-${pad(stamp.getMonth() + 1)}-${pad(stamp.getDate())} ${clock(entry.at)}`;
      return `${date} ${entry.level.toUpperCase().padEnd(5)} ${entry.area.padEnd(8)} ${entryText(entry)}${entry.code ? `  | ${entry.code} ${JSON.stringify(entry.fields)}` : ""}`;
    })].join("\n");
  }
  async function fetchEvents() {
    const fresh = await invoke("ed2k_events", { after: lastEventId });
    if (!Array.isArray(fresh) || !fresh.length) return;
    for (const entry of fresh) { entries.push(entry); lastEventId = Math.max(lastEventId, entry.id); }
    if (entries.length > 3000) entries.splice(0, entries.length - 3000);
    renderLog();
  }
  function controls() {
    const connected = Boolean(overview?.server);
    $("#ed2k-connect").disabled = busy || !overview?.servers?.length;
    $("#ed2k-disconnect").disabled = !connected && !busy;
    const kadRunning = Boolean(overview?.network?.kad);
    $("#ed2k-kad-toggle").disabled = busy || (!kadRunning && !overview?.node_count);
    $("#ed2k-search-submit").disabled = busy || !connected;
    $("#ed2k-add-submit").disabled = busy;
    $("#ed2k-choose-directory").disabled = busy;
    for (const button of document.querySelectorAll("#ed2k-downloads button")) button.disabled = busy || button.dataset.pausing === "true" || button.dataset.unavailable === "true" || (button.dataset.connection === "true" && !connected && !kadRunning);
    for (const button of document.querySelectorAll(".ed2k-setting button, #ed2k-servers button")) button.disabled = busy;
    for (const button of document.querySelectorAll("#ed2k-results button")) button.disabled = busy || (button.dataset.sources === "true" && !connected);
    for (const id of ["#ed2k-share-folder", "#ed2k-share-files", "#ed2k-share-rescan", "#ed2k-share-limits"]) $(id).disabled = busy;
    for (const button of document.querySelectorAll("#ed2k-share-folders button, #ed2k-share-list button")) button.disabled = busy;
  }
  function upnpText(net) {
    if (!net) return t("upnpIdle");
    if (!net.upnp_enabled) return t("upnpOff");
    const upnp = net.upnp || {};
    if (upnp.state === "searching") return t("upnpSearching");
    if (upnp.state === "mapped") return upnp.external_ip ? t("upnpMappedIp", {ip: upnp.external_ip}) : t("upnpMapped");
    if (upnp.state === "partial") return t("upnpPartial");
    if (upnp.state === "failed") return t("upnpFailed");
    return t("upnpIdle");
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
      find.onclick = () => run("ed2k_find_sources", {hash:result.hash, size:result.size}, sources => { sourceView={name:result.name,sources}; renderSources(); message("sourceCount", {count:sources.length}); });
      td.append(find);
      const download = document.createElement("button"); download.type="button"; download.textContent=t("download");
      download.onclick = () => { $("#ed2k-file-link").value=result.link; selectTab("transfers"); }; td.append(download);
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
      let state = t("stage_"+task.status)+(task.queue_rank ? ` · ${t("queueRank", {rank:task.queue_rank})}` : "");
      if (task.status === "waiting_sources") {
        const seconds = Math.max(0, Math.round((task.retry_at || 0) - Date.now() / 1000));
        state += ` · ${t("stall_" + (task.stall || "no_sources"))}` + (task.retry_at ? ` · ${t("nextAttempt", {seconds})}` : "");
      }
      for (const value of [task.file.name,size(task.file.size),`${new Intl.NumberFormat(locale,{maximumFractionDigits:1}).format(percent)}%${task.in_flight > 0 ? ` · +${size(task.in_flight)}` : ""}`,state,task.peer || "—"]) {
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
      if (task.status==="waiting_sources") addAction("retry","retryNow");
      if (/\.(mp4|m4v|mkv|webm|mov|avi|ts)$/i.test(task.file.name)) {
        const button = document.createElement("button"); button.type="button"; button.textContent=t("watch");
        button.dataset.unavailable = String(!task.preview_ready && task.status !== "completed");
        if (button.dataset.unavailable === "true") button.title=t("previewWaiting");
        button.onclick=()=>run("ed2k_preview_download", {id:task.id}); actions.append(button);
      }
      if (inactive) addAction("remove","removeTask");
      row.append(actions); body.append(row);
    }
  }
  function addDownload(link) { return run("ed2k_add_download", {link, directory: $("#ed2k-save-directory").value || null}, () => { $("#ed2k-file-link").value=""; selectTab("transfers"); message("addedDownload"); }); }
  const PRIORITIES = [["prioVeryLow", 0], ["prioLow", 1], ["prioNormal", 2], ["prioHigh", 3], ["prioRelease", 4]];
  const KNOWN_ERRORS = { share_folder_too_broad: "shareTooBroad" };
  function cell(value, title) { const td = document.createElement("td"); td.textContent = String(value); td.title = String(title ?? value); return td; }
  function renderShares() {
    const shares = overview?.shares || { folders: [], files: [], uploads: { active: [], waiting: [], total_uploaded: 0, files: [] }, published: {}, upload_kib: 0, slots: 3 };
    const connected = Boolean(overview?.server);
    const folders = $("#ed2k-share-folders"); folders.replaceChildren();
    for (const folder of shares.folders) {
      const row = document.createElement("tr");
      row.append(cell(folder.path), cell(t(folder.recursive ? "shareWithSub" : "shareOnlyThis")));
      const actions = document.createElement("td"), remove = document.createElement("button");
      remove.type = "button"; remove.textContent = t("shareStop");
      remove.onclick = () => run("ed2k_share_remove_folder", { path: folder.path });
      actions.append(remove); row.append(actions); folders.append(row);
    }
    if (!shares.folders.length) { const row = document.createElement("tr"); row.append(cell(t("shareNoFolders"))); folders.append(row); }
    const list = $("#ed2k-share-list"); list.replaceChildren();
    for (const file of shares.files) {
      const row = document.createElement("tr");
      row.append(cell(file.name, file.path), cell(size(file.size)));
      const priority = document.createElement("td"), select = document.createElement("select");
      for (const [key, value] of PRIORITIES) { const option = document.createElement("option"); option.value = String(value); option.textContent = t(key); select.append(option); }
      select.value = String(file.priority);
      select.onchange = () => run("ed2k_share_set_priority", { hash: file.hash, priority: Number(select.value) });
      priority.append(select);
      row.append(priority, cell(file.requests), cell(size(file.uploaded)));
      const actions = document.createElement("td");
      const copy = document.createElement("button"); copy.type = "button"; copy.textContent = t("copy");
      copy.onclick = () => run("ed2k_copy_link", { link: file.link }, () => message("copied"));
      const stop = document.createElement("button"); stop.type = "button"; stop.textContent = t("shareStop");
      stop.onclick = () => run("ed2k_share_remove_file", { hash: file.hash });
      actions.append(copy, stop); row.append(actions); list.append(row);
    }
    if (!shares.files.length) { const row = document.createElement("tr"); row.append(cell(t("shareNoFiles"))); list.append(row); }
    let status = "";
    if (shares.work) status = t("shareHashing", { done: shares.work.done, total: shares.work.total, name: shares.work.current });
    else if (shares.error) status = t("shareFailedFiles", { error: shares.error });
    else if (shares.published?.error) status = t("sharePublishFailed");
    else if (shares.files.length && !connected) status = t("shareNotConnected");
    else if (shares.files.length) status = t("sharePublished", { count: shares.published?.files || 0 });
    $("#share-status").textContent = status;
    const uploads = shares.uploads;
    $("#share-uploads").textContent = t("shareUploads", { active: uploads.active.length, waiting: uploads.waiting.length, total: size(uploads.total_uploaded) });
    const clients = $("#ed2k-share-clients"); clients.replaceChildren();
    for (const client of [...uploads.active.map(c => ({ ...c, state: t("shareActive") })), ...uploads.waiting.map(c => ({ ...c, state: t("shareWaiting", { rank: c.rank ?? "—" }) }))]) {
      const name = shares.files.find(file => file.hash === client.file)?.name || client.file;
      const row = document.createElement("tr");
      row.append(cell(`${client.name || "—"} · ${client.address}`), cell(name), cell(size(client.uploaded)), cell(client.state));
      clients.append(row);
    }
    if (document.activeElement !== $("#ed2k-upload-limit")) $("#ed2k-upload-limit").value = String(shares.upload_kib);
    if (document.activeElement !== $("#ed2k-upload-slots")) $("#ed2k-upload-slots").value = String(shares.slots);
  }
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
  function render() {
    document.documentElement.lang = locale;
    for (const node of document.querySelectorAll("[data-i18n]")) node.textContent = t(node.dataset.i18n);
    $("#ed2k-log-search").setAttribute("placeholder", t("logSearch"));
    renderLog();
    message(feedback.key, feedback.values, feedback.error);
    $("#ed2k-status").textContent = `ED2K: ${overview?.server ? t("online") : t("offline")}${overview?.low_id ? ` · ${t("lowId")}` : ""}`;
    $("#ed2k-status").dataset.online = String(Boolean(overview?.server));
    $("#incoming-status").textContent = overview?.incoming?.running ? t("incomingPort", {port:overview.incoming.tcp_port,count:overview.incoming.completed_handshakes}) : t("incomingInactive");
    const net = overview?.network, kad = net?.kad;
    $("#kad-status").textContent = kad ? t(kad.verified_contacts > 0 ? "kadStatusConnected" : "kadStatusConnecting", {contacts:kad.contacts, verified:kad.verified_contacts}) : t("kadInactive");
    $("#kad-summary").textContent = kad ? t("kadSummaryRunning", {udp:kad.udp_port, contacts:kad.contacts, verified:kad.verified_contacts}) : t("nodeCount", {count:overview?.node_count || 0});
    $("#ed2k-kad-toggle").textContent = t(kad ? "kadDisconnect" : "kadConnect");
    $("#upnp-status").textContent = upnpText(net);
    if (document.activeElement !== $("#ed2k-obfuscation")) $("#ed2k-obfuscation").value = net?.obfuscation || "prefer";
    $("#ed2k-upnp").checked = net ? Boolean(net.upnp_enabled) : true;
    $("#server-count").textContent = t("serverCount", {count:overview?.servers?.length || 0});
    const body = $("#ed2k-servers"); body.replaceChildren();
    for (const server of overview?.servers || []) {
      const row = document.createElement("tr");
      for (const value of [server.name, server.host, server.port, server.description]) { const td = document.createElement("td"); td.textContent = String(value); td.title = String(value); row.append(td); }
      const td = document.createElement("td"), connect = document.createElement("button"); connect.type = "button"; connect.textContent = t("connect"); connect.onclick = () => run("ed2k_connect", {server}, () => message("connectedTo", {name:server.name})); td.append(connect); row.append(td); body.append(row);
    }
    renderResults(); renderSources(); renderDownloads(); renderShares(); controls();
  }
  async function refresh() { if (!invoke) return; overview = await invoke("ed2k_overview"); render(); await fetchEvents(); }
  async function run(command, args = {}, success = () => message("ready")) {
    if (!invoke) { message("desktopRequired", {}, true); return; }
    if (busy) return;
    busy = true; message("working"); controls();
    try { const result = await invoke(command, args); await refresh(); success(result); log(`${t("ready")}: ${command}`); }
    catch (error) {
      const known = Object.keys(KNOWN_ERRORS).find(code => String(error).includes(code));
      message(known ? KNOWN_ERRORS[known] : "failed", {}, true); log(`${t("failed")}: ${String(error)}`);
    }
    finally { busy = false; controls(); }
  }
  $("#ed2k-connect").onclick = () => run("ed2k_connect", {server:null});
  $("#ed2k-disconnect").onclick = async () => { try { await invoke?.("ed2k_disconnect"); await refresh(); message("connectionClosed"); } catch(error) { message("failed", {}, true); log(String(error)); } };
  $("#ed2k-kad-toggle").onclick = () => overview?.network?.kad
    ? run("ed2k_kad_disconnect", {}, () => message("kadStopped"))
    : run("ed2k_kad_connect", {}, () => message("kadStarted"));
  const savePrefs = () => run("ed2k_set_prefs", { obfuscation: $("#ed2k-obfuscation").value, upnp: Boolean($("#ed2k-upnp").checked) }, () => message("prefsSaved"));
  $("#ed2k-obfuscation").onchange = savePrefs;
  $("#ed2k-upnp").onchange = savePrefs;
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
  $("#ed2k-choose-directory").onclick = () => run("pick_directory", {initialDirectory: $("#ed2k-save-directory").value || null}, path => { if (path) $("#ed2k-save-directory").value = path; });
  $("#ed2k-add-form").onsubmit = event => { event.preventDefault(); const link=$("#ed2k-file-link").value.trim(); if (link) addDownload(link); };
  $("#ed2k-share-folder").onclick = () => run("ed2k_share_add_folder", { recursive: Boolean($("#ed2k-share-recursive").checked) }, path => path && message("shareFolderAdded", { path }));
  $("#ed2k-share-files").onclick = () => run("ed2k_share_add_files", {}, count => count && message("shareFilesAdded", { count }));
  $("#ed2k-share-rescan").onclick = () => run("ed2k_share_rescan");
  $("#ed2k-share-limits").onclick = () => {
    const kib = Number($("#ed2k-upload-limit").value), slots = Number($("#ed2k-upload-slots").value);
    if (!Number.isInteger(kib) || kib < 0 || !Number.isInteger(slots) || slots < 1 || slots > 20) { message("failed", {}, true); return; }
    run("ed2k_share_set_limits", { uploadKib: kib, slots }, () => message("shareLimitsApplied"));
  };
  $("#ed2k-result-filter").oninput = renderResults;
  $("#ed2k-result-sort").onchange = renderResults;
  $("#ed2k-clear-log").onclick = () => { clearedAt = lastEventId; for (let i = entries.length - 1; i >= 0; i--) if (entries[i].id === 0) entries.splice(i, 1); renderLog(); };
  for (const id of ["#ed2k-log-level", "#ed2k-log-area"]) $(id).onchange = renderLog;
  $("#ed2k-log-search").oninput = renderLog;
  $("#ed2k-log-follow").onchange = renderLog;
  $("#ed2k-log-copy").onclick = async () => { try { await navigator.clipboard.writeText(logAsText()); message("logCopied"); } catch (error) { log(String(error), "warn"); } };
  try { if (localStorage.getItem("apocalipse.ed2k.logAsk") === "0") $("#ed2k-log-ask").checked = false; } catch {}
  $("#ed2k-log-ask").onchange = () => { try { localStorage.setItem("apocalipse.ed2k.logAsk", $("#ed2k-log-ask").checked === false ? "0" : "1"); } catch {} };
  $("#ed2k-log-save").onclick = async () => {
    try {
      const path = await invoke("ed2k_save_log", { text: logAsText(), ask: $("#ed2k-log-ask").checked !== false });
      if (!path) { message("logSaveCancelled"); return; }
      message("logSavedTo", { path }); log(t("logSavedTo", { path }));
    } catch (error) { message("logSaveFailed", { error: String(error) }, true); }
  };
  function selectTab(name) {
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
    invoke("default_download_directory").then(path => { if (!$("#ed2k-save-directory").value) $("#ed2k-save-directory").value = path; }).catch(error => log(String(error)));
    refresh().catch(error => { message("failed", {}, true); log(String(error)); });
    const timer = setInterval(() => { if (!busy && !document.hidden) refresh().catch(error => log(String(error))); },2000);
    window.addEventListener("pagehide", () => clearInterval(timer), {once:true});
  } else { message("desktopRequired", {}, true); }
})();
