const catalogs = {
  en: {
    downloads: "Downloads",
    media: "Media",
    recordings: "Recordings",
    torrents: "Torrents",
    tools: "Tools",
    settings: "Settings",
    themes: "Themes", language: "Language",
    themesDescription: "Personalize colors, transparency, corners and interface size.",
    languageDescription: "Choose the language used throughout Apocalipse and in the tray menu.",
    chooseTheme: "Choose theme", themeOptions: "Theme options", windowTransparency: "Window transparency", windowTransparencyHint: "Make the application window transparent", transparencyLevel: "Transparency level", roundedCorners: "Rounded corners", roundedCornersHint: "Use rounded corners on windows, panels and controls", cornerRadius: "Corner radius", interfaceSize: "Interface size", interfaceSizeHint: "Adjust text and element sizes", compact: "Compact", normal: "Normal", large: "Large", chooseLanguage: "Choose language", languageHint: "The entire application and tray menu use the selected language.",
    logs: "Logs", logsDescription: "End-to-end diagnostics for extension, shortcuts, interface, bridge and downloads.", exportLog: "Export log", searchLogs: "Search events…", allLevels: "All levels",
    downloadsDescription: "Manage direct downloads, progress, speed and completed files.",
    mediaDescription: "Videos, audio, recordings and exports detected by Apocalipse.",
    recordingsDescription: "Follow active recordings, stop and save, export or open completed captures.",
    torrentsDescription: "Manage torrents, file selection, peers and previews.",
    linkDescription: "Transfer files securely between this computer and a remote Apocalipse.",
    toolsPageDescription: "Manage the engines used for media, transfers, conversion and preview.",
    settingsDescription: "Configure appearance, integrations, network and application behavior.",
    toolbox: "TOOLBOX", update: "Update", mediaPlayer: "VLC / mpv / media player",
    donatePaypal: "Donate via PayPal",
    overview: "OVERVIEW",
    engineReady: "Engine ready",
    addDownload: "Add download",
    downloadSpeed: "DOWNLOAD SPEED",
    uploadSpeed: "UPLOAD SPEED",
    completed: "COMPLETED",
    queue: "IN QUEUE",
    all: "All",
    active: "Active",
    clearFinished: "Clear finished",
    selectAll: "Select all",
    removeSelected: "Remove selected",
    redownloadSelected: "Download again",
    manageList: "MANAGE LIST",
    removeChoice: "What do you want to remove?",
    listOnly: "Clear from list",
    keepFiles: "Keep downloaded and partial files on disk",
    listAndFiles: "Clear list and files",
    deleteFiles: "Permanently delete downloaded and partial files",
    emptyTitle: "Ready for your next download",
    emptyText: "Add a URL, drop a torrent, or use the browser extension.",
    addFirst: "Add your first download",
    bridgeStatus: "Extension bridge not configured",
    newTask: "NEW TASK",
    sourceUrl: "Source URL",
    cancel: "Cancel",
    analyze: "Analyze",
    torrentMetadataSeeking: "Finding peers and receiving torrent metadata",
    saveTo: "Save to",
    fileName: "File name",
    queued: "Queued",
    inspecting: "Inspecting",
    downloading: "Downloading",
    paused: "Paused",
    failed: "Failed",
    pause: "Pause",
    resume: "Resume",
    resumeCapability: "Resume capability:",
    resumeYes: "Yes",
    resumeNo: "No",
    resumeChecking: "Checking…",
    retry: "Retry",
    openFolder: "Open folder",
    preview: "Preview",
    stopRecording: "Stop and save",
    recordingActive: "Recording",
    linkThisComputer: "This computer",
    linkRemoteControl: "Control remote computer",
    linkRemoteId: "Remote ID",
    linkNewPassword: "New password",
    linkAccessNotice: "Authorized access shows all drives and folders on this computer.",
    linkConnect: "Connect",
    linkSelfTest: "Test on this PC",
    linkSend: "Send →",
    linkRemoteComputer: "Remote computer",
    linkDownload: "← Download",
    linkDrives: "Drives",
    linkConnected: "Connected",
    linkConnectionFailed: "Connection failed",
    linkTransferring: "Transferring…",
    linkSending: "Sending…",
    linkCompleted: "Completed",
    linkTransferFailed: "Transfer failed",
    linkUploadFailed: "Upload failed",
    linkSendTitle: "Send a file directly",
    linkSendHint: "Create a private, one-use link valid for 10 minutes on your local network.",
    linkChooseFile: "Choose file and create link",
    linkCopy: "Copy link",
    linkReceiveTitle: "Receive a file",
    linkReceiveHint: "Paste an Apocalipse Link received from another computer.",
    linkReceive: "Receive",
    preferences: "PREFERENCES",
    appearanceTheme: "Interface theme",
    themeHint: "Colors and text contrast are adjusted together for readability.",
    associations: "File and link associations",
    associationsHint: "Choose individually what the system should open with Apocalipse.",
    startWithSystem: "Start with the system",
    startHidden: "Open hidden in the system tray",
    defaultDirectory: "Default download directory",
    save: "Save",
    browse: "Browse…",
    captureClipboard: "Capture clipboard links",
    captureClipboardHint: "Open recognized HTTP, HLS, magnet and media links automatically",
    userAgent: "Custom User-Agent",
    userAgentHint: "Automatic — use the browser identity",
    proxy: "Proxy",
    proxyHint: "Route downloads through an HTTP, HTTPS or SOCKS proxy",
    proxyAddress: "Proxy address",
    proxyUsername: "Username",
    proxyPassword: "Password",
    proxyPasswordHint: "Leave blank to keep the saved password",
    proxyClearPassword: "Remove the saved proxy password",
    proxyPortableWarning: "The proxy configuration is saved in the portable data/settings.json file.",
    websiteCredentials: "Site credentials",
    websiteCredentialsHint: "Automatically authenticate compatible HTTP, FTP and media downloads.",
    websiteHost: "Site domain",
    websiteHostHint: "example.com",
    websiteCredentialAdd: "Add or update",
    websiteCredentialRemove: "Remove",
    websiteCredentialSaved: "Credential saved",
    websiteCredentialsEmpty: "No site credentials saved.",
    websiteCredentialsLocalWarning: "Passwords are stored locally in the portable data/settings.json file. Protect access to this folder.",
    customDns: "Custom DNS",
    customDnsHint: "Resolve native downloads without changing the operating system DNS",
    dnsProvider: "Provider",
    dnsCustom: "Custom",
    dnsServers: "DNS servers",
    dnsScopeHint: "Applied to the native HTTP engine and aria2. SOCKS5H continues resolving through the proxy.",
    maxTasks: "Maximum simultaneous tasks",
    connections: "Connections per download",
    taskConnections: "Threads for this download",
    taskConnectionsHint: "Only changes this task. Use 1 on sites that reject segmented downloads.",
    downloadBandwidthLimit: "This download limit", megabytesPerSecond: "MB/s", unlimited: "Unlimited", smartAutomation: "Smart automation", bandwidthPanel: "Bandwidth", adaptiveEfficiency: "Adaptive efficiency", adaptiveEfficiencyHint: "Optimizes queue order and connection use for the current workload.", scheduler: "Download schedule", schedulerHint: "Automatically pauses outside the permitted local time window.", scheduleStart: "Start", scheduleEnd: "End", bandwidthPanelHint: "Set limits without changing the window size.", currentBandwidth: "Current usage", globalBandwidthLimit: "Global download limit",
    defaults: "Default",
    extensionPairing: "Browser extension pairing",
    pairingToken: "Pairing token",
    copy: "Copy",
    regenerate: "Regenerate",
    bridgeConnected: "Extension connected",
    bridgeWaiting: "Waiting for extension",
    recentLocations: "Download locations",
    clearLocations: "Clear download locations",
    defaultLocation: "Default",
    unavailableLocation: "Unavailable",
    qualityFormat: "Quality and format",
    bestQuality: "Best video + best audio (recommended)",
    audioOnly: "Audio only",
    duration: "Duration",
    mediaUnavailable: "Media details are unavailable; the default format can still be used.",
    externalTools: "Required media and transfer tools",
    toolsHint: "Configure each executable. Apocalipse uses these exact paths for downloads.",
    installed: "Detected",
    missing: "Not found",
    checkTools: "Check versions",
    removeFailed: "Could not remove the selected files",
    diagnostics: "Diagnostics",
    diagnosticsHint: "Safe activity log with credentials and URL parameters hidden",
    openLog: "Open diagnostic log",
    clearLog: "Clear log",
    refreshLog: "Refresh",
    closeLog: "Close",
    emptyLog: "No diagnostic events recorded yet.",
    logEditor: "Log editor",
    logEditorHint: "Choose an editor executable, including a portable application",
    chooseEditor: "Choose editor…",
    removeEditor: "Remove editor",
    openExternal: "Open in editor",
    exportRecording: "Export completed recording", outputFormat: "Output format", videoCodec: "Video codec", audioCodec: "Audio codec", export: "Export",
    searchHistory: "Search downloads…", importList: "Import list", advancedOptions: "Advanced options", mirrorUrls: "Mirror URLs (one per line)", priority: "Priority", priorityHigh: "High", priorityNormal: "Normal", priorityLow: "Low", verifyIntegrity: "Verify SHA-256", integrityPrompt: "Optional expected SHA-256 (leave blank to calculate only):", integrityOk: "SHA-256 verified",
  },
  "pt-BR": {
    downloads: "Downloads",
    media: "Mídia",
    recordings: "Gravações",
    torrents: "Torrents",
    tools: "Ferramentas",
    settings: "Configurações",
    themes: "Temas", language: "Idioma",
    themesDescription: "Personalize cores, transparência, cantos e tamanho da interface.",
    languageDescription: "Escolha o idioma usado em todo o Apocalipse e no menu da bandeja.",
    chooseTheme: "Escolher tema", themeOptions: "Opções do tema", windowTransparency: "Transparência da janela", windowTransparencyHint: "Deixa a janela do aplicativo transparente", transparencyLevel: "Nível de transparência", roundedCorners: "Cantos arredondados", roundedCornersHint: "Usar cantos arredondados nas janelas, painéis e controles", cornerRadius: "Raio dos cantos", interfaceSize: "Tamanho da interface", interfaceSizeHint: "Ajusta o tamanho dos textos e elementos", compact: "Compacto", normal: "Normal", large: "Grande", chooseLanguage: "Escolher idioma", languageHint: "Todo o aplicativo e o menu da bandeja usam o idioma selecionado.",
    logs: "Logs", logsDescription: "Diagnóstico de ponta a ponta da extensão, atalhos, interface, ponte e downloads.", exportLog: "Exportar log", searchLogs: "Pesquisar eventos…", allLevels: "Todos os níveis",
    downloadsDescription: "Gerencie downloads diretos, progresso, velocidade e arquivos concluídos.",
    mediaDescription: "Vídeos, áudios, gravações e exportações detectados pelo Apocalipse.",
    recordingsDescription: "Acompanhe gravações ativas, pare e salve, exporte ou abra capturas concluídas.",
    torrentsDescription: "Gerencie torrents, escolha de arquivos, pares e pré-visualizações.",
    linkDescription: "Transfira arquivos com segurança entre este computador e um Apocalipse remoto.",
    toolsPageDescription: "Gerencie os motores usados para mídia, transferências, conversão e pré-visualização.",
    settingsDescription: "Configure aparência, integrações, rede e comportamento do aplicativo.",
    toolbox: "CAIXA DE FERRAMENTAS", update: "Atualizar", mediaPlayer: "VLC / mpv / reprodutor de mídia",
    donatePaypal: "Faça uma doação pelo PayPal",
    overview: "VISÃO GERAL",
    engineReady: "Motor pronto",
    addDownload: "Adicionar download",
    downloadSpeed: "VELOCIDADE DE DOWNLOAD",
    uploadSpeed: "VELOCIDADE DE ENVIO",
    completed: "CONCLUÍDOS",
    queue: "NA FILA",
    all: "Todos",
    active: "Ativos",
    clearFinished: "Limpar concluídos",
    selectAll: "Selecionar todos",
    removeSelected: "Remover selecionados",
    redownloadSelected: "Baixar novamente",
    manageList: "GERENCIAR LISTA",
    removeChoice: "O que você deseja remover?",
    listOnly: "Limpar somente da lista",
    keepFiles: "Manter no disco os arquivos baixados e parciais",
    listAndFiles: "Limpar lista e arquivos",
    deleteFiles: "Excluir permanentemente os arquivos baixados e parciais",
    emptyTitle: "Pronto para o próximo download",
    emptyText:
      "Adicione uma URL, arraste um torrent ou use a extensão do navegador.",
    addFirst: "Adicionar primeiro download",
    bridgeStatus: "Ponte da extensão não configurada",
    newTask: "NOVA TAREFA",
    sourceUrl: "URL de origem",
    cancel: "Cancelar",
    analyze: "Analisar",
    torrentMetadataSeeking: "Procurando pares e recebendo metadados do torrent",
    saveTo: "Salvar em",
    fileName: "Nome do arquivo",
    queued: "Na fila",
    inspecting: "Analisando",
    downloading: "Baixando",
    paused: "Pausado",
    failed: "Falhou",
    pause: "Pausar",
    resume: "Continuar",
    resumeCapability: "Capacidade de continuar:",
    resumeYes: "Sim",
    resumeNo: "Não",
    resumeChecking: "Verificando…",
    retry: "Tentar novamente",
    openFolder: "Abrir pasta",
    preview: "Pré-visualizar",
    stopRecording: "Parar e salvar",
    recordingActive: "Gravando",
    linkThisComputer: "Este computador",
    linkRemoteControl: "Controlar computador remoto",
    linkRemoteId: "ID remoto",
    linkNewPassword: "Nova senha",
    linkAccessNotice: "O acesso autorizado mostra todas as unidades e pastas deste computador.",
    linkConnect: "Conectar",
    linkSelfTest: "Testar neste PC",
    linkSend: "Enviar →",
    linkRemoteComputer: "Computador remoto",
    linkDownload: "← Baixar",
    linkDrives: "Unidades",
    linkConnected: "Conectado",
    linkConnectionFailed: "Falha na conexão",
    linkTransferring: "Transferindo…",
    linkSending: "Enviando…",
    linkCompleted: "Concluído",
    linkTransferFailed: "Falha na transferência",
    linkUploadFailed: "Falha no envio",
    linkSendTitle: "Enviar um arquivo diretamente",
    linkSendHint: "Crie um link privado de uso único, válido por 10 minutos na sua rede local.",
    linkChooseFile: "Escolher arquivo e criar link",
    linkCopy: "Copiar link",
    linkReceiveTitle: "Receber um arquivo",
    linkReceiveHint: "Cole um Apocalipse Link recebido de outro computador.",
    linkReceive: "Receber",
    preferences: "PREFERÊNCIAS",
    appearanceTheme: "Tema da interface",
    themeHint: "As cores e o contraste do texto são ajustados juntos para manter a leitura.",
    associations: "Associações de arquivos e links",
    associationsHint: "Escolha individualmente o que o sistema deve abrir com o Apocalipse.",
    startWithSystem: "Iniciar com o sistema",
    startHidden: "Abrir oculto na bandeja do sistema",
    defaultDirectory: "Diretório padrão de downloads",
    save: "Salvar",
    browse: "Procurar…",
    captureClipboard: "Capturar links da área de transferência",
    captureClipboardHint: "Abrir automaticamente links HTTP, HLS, magnet e de mídia reconhecidos",
    userAgent: "User-Agent personalizado",
    userAgentHint: "Automático — usar a identidade do navegador",
    proxy: "Proxy",
    proxyHint: "Encaminhar os downloads por um proxy HTTP, HTTPS ou SOCKS",
    proxyAddress: "Endereço do proxy",
    proxyUsername: "Nome de usuário",
    proxyPassword: "Senha",
    proxyPasswordHint: "Deixe vazio para manter a senha salva",
    proxyClearPassword: "Remover a senha de proxy salva",
    proxyPortableWarning: "A configuração do proxy é salva no arquivo portátil data/settings.json.",
    websiteCredentials: "Credenciais de sites",
    websiteCredentialsHint: "Autenticar automaticamente downloads HTTP, FTP e de mídia compatíveis.",
    websiteHost: "Domínio do site",
    websiteHostHint: "exemplo.com.br",
    websiteCredentialAdd: "Adicionar ou atualizar",
    websiteCredentialRemove: "Remover",
    websiteCredentialSaved: "Credencial salva",
    websiteCredentialsEmpty: "Nenhuma credencial de site salva.",
    websiteCredentialsLocalWarning: "As senhas ficam armazenadas localmente no arquivo portátil data/settings.json. Proteja o acesso a essa pasta.",
    customDns: "DNS personalizado",
    customDnsHint: "Resolver downloads nativos sem alterar o DNS do sistema operacional",
    dnsProvider: "Provedor",
    dnsCustom: "Personalizado",
    dnsServers: "Servidores DNS",
    dnsScopeHint: "Aplicado ao motor HTTP nativo e ao aria2. O SOCKS5H continua resolvendo pelo proxy.",
    maxTasks: "Máximo de tarefas simultâneas",
    connections: "Conexões por download",
    taskConnections: "Threads para este download",
    taskConnectionsHint: "Altera somente esta tarefa. Use 1 em sites que não aceitam downloads segmentados.",
    downloadBandwidthLimit: "Limite deste download", megabytesPerSecond: "MB/s", unlimited: "Ilimitado", smartAutomation: "Automação inteligente", bandwidthPanel: "Banda", adaptiveEfficiency: "Eficiência adaptativa", adaptiveEfficiencyHint: "Otimiza a ordem da fila e o uso de conexões para a carga atual.", scheduler: "Agendamento de downloads", schedulerHint: "Pausa automaticamente fora do horário local permitido.", scheduleStart: "Início", scheduleEnd: "Fim", bandwidthPanelHint: "Defina limites sem alterar o tamanho da janela.", currentBandwidth: "Uso atual", globalBandwidthLimit: "Limite global de download",
    defaults: "Padrão",
    extensionPairing: "Conexão com a extensão",
    pairingToken: "Token de pareamento",
    copy: "Copiar",
    regenerate: "Gerar outro",
    bridgeConnected: "Extensão conectada",
    bridgeWaiting: "Aguardando extensão",
    recentLocations: "Locais de download",
    clearLocations: "Limpar caminhos de download",
    defaultLocation: "Padrão",
    unavailableLocation: "Indisponível",
    qualityFormat: "Qualidade e formato",
    bestQuality: "Melhor vídeo + melhor áudio (recomendado)",
    audioOnly: "Somente áudio",
    duration: "Duração",
    mediaUnavailable: "Os detalhes da mídia não estão disponíveis; ainda é possível usar o formato padrão.",
    externalTools: "Ferramentas obrigatórias de mídia e transferência",
    toolsHint: "Configure cada executável. O Apocalipse usa exatamente estes caminhos nos downloads.",
    installed: "Detectado",
    missing: "Não encontrado",
    checkTools: "Verificar versões",
    removeFailed: "Não foi possível apagar os arquivos selecionados",
    diagnostics: "Diagnóstico",
    diagnosticsHint: "Log seguro de atividades com credenciais e parâmetros das URLs ocultados",
    openLog: "Abrir log de diagnóstico",
    clearLog: "Limpar log",
    refreshLog: "Atualizar",
    closeLog: "Fechar",
    emptyLog: "Ainda não há eventos de diagnóstico registrados.",
    logEditor: "Editor de log",
    logEditorHint: "Escolha o executável de um editor, inclusive um aplicativo portátil",
    chooseEditor: "Escolher editor…",
    removeEditor: "Remover editor",
    openExternal: "Abrir no editor",
    exportRecording: "Exportar gravação concluída", outputFormat: "Formato de saída", videoCodec: "Codec de vídeo", audioCodec: "Codec de áudio", export: "Exportar",
    searchHistory: "Pesquisar downloads…", importList: "Importar lista", advancedOptions: "Opções avançadas", mirrorUrls: "URLs espelho (uma por linha)", priority: "Prioridade", priorityHigh: "Alta", priorityNormal: "Normal", priorityLow: "Baixa", verifyIntegrity: "Verificar SHA-256", integrityPrompt: "SHA-256 esperado opcional (deixe vazio apenas para calcular):", integrityOk: "SHA-256 verificado",
  },
  "zh-CN": {
    downloads: "下载",
    media: "媒体",
    recordings: "录制",
    torrents: "种子",
    tools: "工具",
    settings: "设置",
    themes: "主题", language: "语言",
    themesDescription: "自定义颜色、透明度、圆角和界面大小。",
    languageDescription: "选择整个 Apocalipse 和托盘菜单使用的语言。",
    chooseTheme: "选择主题", themeOptions: "主题选项", windowTransparency: "窗口透明度", windowTransparencyHint: "使应用程序窗口透明", transparencyLevel: "透明度级别", roundedCorners: "圆角", roundedCornersHint: "为窗口、面板和控件使用圆角", cornerRadius: "圆角半径", interfaceSize: "界面大小", interfaceSizeHint: "调整文本和元素大小", compact: "紧凑", normal: "正常", large: "大", chooseLanguage: "选择语言", languageHint: "整个应用程序和托盘菜单都使用所选语言。",
    logs: "日志", logsDescription: "扩展、快捷键、界面、桥接和下载的端到端诊断。", exportLog: "导出日志", searchLogs: "搜索事件…", allLevels: "所有级别",
    downloadsDescription: "管理直接下载、进度、速度和已完成文件。",
    mediaDescription: "管理 Apocalipse 检测到的视频、音频、录制和导出。",
    recordingsDescription: "查看正在录制的内容、停止并保存、导出或打开已完成的录制。",
    torrentsDescription: "管理种子、文件选择、节点和预览。",
    linkDescription: "在本机与远程 Apocalipse 之间安全传输文件。",
    toolsPageDescription: "管理媒体、传输、转换和预览所使用的引擎。",
    settingsDescription: "配置外观、集成、网络和应用行为。",
    toolbox: "工具箱", update: "更新", mediaPlayer: "VLC / mpv / 媒体播放器",
    donatePaypal: "通过 PayPal 捐赠",
    overview: "概览",
    engineReady: "引擎已就绪",
    addDownload: "添加下载",
    downloadSpeed: "下载速度",
    uploadSpeed: "上传速度",
    completed: "已完成",
    queue: "队列中",
    all: "全部",
    active: "进行中",
    clearFinished: "清除已完成",
    selectAll: "全选",
    removeSelected: "移除所选项目",
    redownloadSelected: "重新下载",
    manageList: "管理列表",
    removeChoice: "您想移除哪些内容？",
    listOnly: "仅从列表中清除",
    keepFiles: "保留磁盘上的已下载文件和部分文件",
    listAndFiles: "清除列表和文件",
    deleteFiles: "永久删除已下载文件和部分文件",
    emptyTitle: "准备开始新的下载",
    emptyText: "添加网址、拖入种子或使用浏览器扩展。",
    addFirst: "添加第一个下载",
    bridgeStatus: "扩展桥接尚未配置",
    newTask: "新任务",
    sourceUrl: "来源网址",
    cancel: "取消",
    analyze: "分析",
    saveTo: "保存到",
    fileName: "文件名",
    queued: "已排队",
    inspecting: "正在检查",
    downloading: "正在下载",
    paused: "已暂停",
    failed: "失败",
    pause: "暂停",
    resume: "继续",
    resumeCapability: "续传能力：",
    resumeYes: "是",
    resumeNo: "否",
    resumeChecking: "检查中…",
    retry: "重试",
    openFolder: "打开文件夹",
    preview: "预览",
    stopRecording: "停止并保存",
    recordingActive: "正在录制",
    linkThisComputer: "此电脑",
    linkRemoteControl: "控制远程电脑",
    linkRemoteId: "远程 ID",
    linkNewPassword: "新密码",
    linkAccessNotice: "授权访问会显示此电脑上的所有驱动器和文件夹。",
    linkConnect: "连接",
    linkSelfTest: "在此电脑上测试",
    linkSend: "发送 →",
    linkRemoteComputer: "远程电脑",
    linkDownload: "← 下载",
    linkDrives: "驱动器",
    linkConnected: "已连接",
    linkConnectionFailed: "连接失败",
    linkTransferring: "正在传输…",
    linkSending: "正在发送…",
    linkCompleted: "已完成",
    linkTransferFailed: "传输失败",
    linkUploadFailed: "发送失败",
    linkSendTitle: "直接发送文件",
    linkSendHint: "创建一个在本地网络中有效十分钟的私密一次性链接。",
    linkChooseFile: "选择文件并创建链接",
    linkCopy: "复制链接",
    linkReceiveTitle: "接收文件",
    linkReceiveHint: "粘贴从另一台计算机收到的 Apocalipse Link。",
    linkReceive: "接收",
    preferences: "偏好设置",
    appearanceTheme: "界面主题",
    themeHint: "颜色与文字对比度会同步调整，以保持清晰易读。",
    associations: "文件和链接关联",
    associationsHint: "单独选择由系统使用 Apocalipse 打开的类型。",
    startWithSystem: "随系统启动",
    startHidden: "启动后隐藏到系统托盘",
    defaultDirectory: "默认下载目录",
    save: "保存",
    browse: "浏览…",
    captureClipboard: "捕获剪贴板链接",
    captureClipboardHint: "自动打开识别出的 HTTP、HLS、磁力和媒体链接",
    userAgent: "自定义 User-Agent",
    userAgentHint: "自动 — 使用浏览器身份",
    proxy: "代理服务器",
    proxyHint: "通过 HTTP、HTTPS 或 SOCKS 代理下载",
    proxyAddress: "代理地址",
    proxyUsername: "用户名",
    proxyPassword: "密码",
    proxyPasswordHint: "留空以保留已保存的密码",
    proxyClearPassword: "删除已保存的代理密码",
    proxyPortableWarning: "代理配置保存在便携式 data/settings.json 文件中。",
    websiteCredentials: "网站凭据",
    websiteCredentialsHint: "自动验证兼容的 HTTP、FTP 和媒体下载。",
    websiteHost: "网站域名",
    websiteHostHint: "example.com",
    websiteCredentialAdd: "添加或更新",
    websiteCredentialRemove: "删除",
    websiteCredentialSaved: "凭据已保存",
    websiteCredentialsEmpty: "尚未保存网站凭据。",
    websiteCredentialsLocalWarning: "密码保存在便携式 data/settings.json 文件中。请保护此文件夹的访问权限。",
    customDns: "自定义 DNS",
    customDnsHint: "解析原生下载而不更改操作系统 DNS",
    dnsProvider: "提供商",
    dnsCustom: "自定义",
    dnsServers: "DNS 服务器",
    dnsScopeHint: "应用于原生 HTTP 引擎和 aria2。SOCKS5H 仍通过代理解析。",
    maxTasks: "最大同时任务数",
    connections: "每个下载的连接数",
    taskConnections: "此下载的线程数",
    taskConnectionsHint: "仅更改此任务。对于不允许分段下载的网站，请使用 1。",
    downloadBandwidthLimit: "此下载的限制", megabytesPerSecond: "MB/秒", unlimited: "不限速", smartAutomation: "智能自动化", bandwidthPanel: "带宽", adaptiveEfficiency: "自适应效率", adaptiveEfficiencyHint: "根据当前负载优化队列顺序和连接使用。", scheduler: "下载计划", schedulerHint: "在允许的本地时间之外自动暂停。", scheduleStart: "开始", scheduleEnd: "结束", bandwidthPanelHint: "无需改变窗口大小即可设置限制。", currentBandwidth: "当前使用量", globalBandwidthLimit: "全局下载限制",
    defaults: "默认",
    extensionPairing: "浏览器扩展配对",
    pairingToken: "配对令牌",
    copy: "复制",
    regenerate: "重新生成",
    bridgeConnected: "扩展已连接",
    bridgeWaiting: "正在等待扩展",
    recentLocations: "下载位置",
    clearLocations: "清除下载路径",
    defaultLocation: "默认",
    unavailableLocation: "不可用",
    qualityFormat: "质量和格式",
    bestQuality: "最佳视频 + 最佳音频（推荐）",
    audioOnly: "仅音频",
    duration: "时长",
    mediaUnavailable: "媒体详情不可用；仍可使用默认格式。",
    externalTools: "必需的媒体和传输工具",
    toolsHint: "配置每个可执行文件。Apocalipse 将在下载时使用这些确切路径。",
    installed: "已检测",
    missing: "未找到",
    checkTools: "检查版本",
    removeFailed: "无法删除所选文件",
    diagnostics: "诊断",
    diagnosticsHint: "隐藏凭据和网址参数的安全活动日志",
    openLog: "打开诊断日志",
    clearLog: "清除日志",
    refreshLog: "刷新",
    closeLog: "关闭",
    emptyLog: "尚未记录诊断事件。",
    logEditor: "日志编辑器",
    logEditorHint: "选择编辑器可执行文件，包括便携式应用程序",
    chooseEditor: "选择编辑器…",
    removeEditor: "移除编辑器",
    openExternal: "在编辑器中打开",
    exportRecording: "导出已完成的录制", outputFormat: "输出格式", videoCodec: "视频编码", audioCodec: "音频编码", export: "导出",
    searchHistory: "搜索下载…", importList: "导入列表", advancedOptions: "高级选项", mirrorUrls: "镜像网址（每行一个）", priority: "优先级", priorityHigh: "高", priorityNormal: "普通", priorityLow: "低", verifyIntegrity: "验证 SHA-256", integrityPrompt: "可选的预期 SHA-256（留空则仅计算）：", integrityOk: "SHA-256 已验证",
  },
};

let locale = localStorage.getItem("apocalipse.language") || "en";
const valid = ["void", "inferno", "toxic", "synthwave", "royal", "crimson", "arctic", "obsidian", "monochrome", "midnight", "forest", "graphite", "deepsea", "eclipse", "hazard", "cyberstorm", "ultraviolet", "emeraldgold", "scarletice", "coppernavy", "solarizednight"];
const applyTheme = (theme) => {
  document.documentElement.dataset.theme = valid.includes(theme) ? theme : "void";
};
applyTheme(localStorage.getItem("apocalipse.theme") || "void");
const appearanceDefaults = { transparencyEnabled: false, transparencyLevel: 30, roundedEnabled: true, cornerRadius: 10, interfaceSize: "normal" };
function readAppearance() {
  try { return { ...appearanceDefaults, ...JSON.parse(localStorage.getItem("apocalipse.appearance") || "{}") }; }
  catch { return { ...appearanceDefaults }; }
}
function applyAppearance(settings = readAppearance()) {
  const transparency = Math.max(0, Math.min(70, Number(settings.transparencyLevel) || 0));
  const radius = Math.max(0, Math.min(28, Number(settings.cornerRadius) || 0));
  document.documentElement.dataset.transparency = settings.transparencyEnabled ? "on" : "off";
  document.documentElement.dataset.rounded = settings.roundedEnabled ? "on" : "off";
  document.documentElement.dataset.uiSize = ["compact", "normal", "large"].includes(settings.interfaceSize) ? settings.interfaceSize : "normal";
  document.documentElement.style.setProperty("--window-opacity-percent", settings.transparencyEnabled ? `${100 - transparency}%` : "100%");
  document.documentElement.style.setProperty("--corner-radius", settings.roundedEnabled ? `${radius}px` : "0px");
}
applyAppearance();
let pendingReferer = null;
let pendingDuration = null;
let pendingTitle = null;
let pendingThumbnail = null;
let pendingMediaKind = null;
let pendingExpectedSize = null;
let pendingCookieHeader = null;
let pendingUserAgent = null;
let pendingRequestMethod = null;
let pendingRequestBody = null;
let pendingRequestContentType = null;
let downloads = [];
let activeFilter = "all";
let activePage = "downloads";
let overallSpeed = 0;
let overallUploadSpeed = 0;
let lastClipboardLink = "";
let clipboardMonitorPrimed = false;
const busyIds = new Set();
const selectedIds = new Set();
const speedSamples = new Map();
const schedulerPaused = new Set();
let selectionPointerActive = false;
let historyQuery = "";
const t = (key) => catalogs[locale]?.[key] || catalogs.en[key] || key;
const tf = (key, values) => Object.entries(values).reduce((text, [name, value]) => text.replaceAll(`{${name}}`, value), t(key));
const descriptions = { downloads: "downloadsDescription", media: "mediaDescription", recordings: "recordingsDescription", torrents: "torrentsDescription", link: "linkDescription", logs: "logsDescription", themes: "themesDescription", language: "languageDescription", settings: "settingsDescription", tools: "toolsPageDescription" };
const invoke = (command, args = {}) => {
  const bridge = window.__TAURI__?.core?.invoke;
  if (!bridge) throw new Error("Desktop bridge unavailable in preview");
  const started = performance.now();
  const quiet = new Set(["list_downloads", "read_general_log", "get_bridge_pairing"]);
  if (!quiet.has(command) && command !== "record_ui_diagnostic") bridge("record_ui_diagnostic", { level: "DEBUG", event: "command_started", detail: `command=${command}` }).catch(() => {});
  return bridge(command, args).then((result) => {
    if (!quiet.has(command) && command !== "record_ui_diagnostic") bridge("record_ui_diagnostic", { level: "DEBUG", event: "command_completed", detail: `command=${command} duration_ms=${Math.round(performance.now() - started)}` }).catch(() => {});
    return result;
  }).catch((error) => {
    if (command !== "record_ui_diagnostic") bridge("record_ui_diagnostic", { level: "ERROR", event: "command_failed", detail: `command=${command} duration_ms=${Math.round(performance.now() - started)} error=${String(error)}` }).catch(() => {});
    throw error;
  });
};
window.addEventListener("error", (event) => invoke("record_ui_diagnostic", { level: "ERROR", event: "javascript_error", detail: `message=${event.message} file=${event.filename || "inline"} line=${event.lineno || 0} column=${event.colno || 0}` }).catch(() => {}));
window.addEventListener("unhandledrejection", (event) => invoke("record_ui_diagnostic", { level: "ERROR", event: "unhandled_rejection", detail: `reason=${String(event.reason)}` }).catch(() => {}));

function stateName(state) {
  return t(stateKey(state));
}

function stateKey(state) {
  return typeof state === "string" ? state : Object.keys(state)[0];
}

function formatBytes(bytes) {
  if (!bytes) return "0 B";
  const units = ["B", "KB", "MB", "GB", "TB"];
  const index = Math.min(
    Math.floor(Math.log(bytes) / Math.log(1024)),
    units.length - 1,
  );
  return `${(bytes / 1024 ** index).toFixed(index ? 1 : 0)} ${units[index]}`;
}

function updateSpeeds(tasks) {
  const now = performance.now();
  const currentIds = new Set(tasks.map((task) => task.id));
  overallSpeed = 0;
  overallUploadSpeed = 0;
  for (const [id] of speedSamples)
    if (!currentIds.has(id)) speedSamples.delete(id);
  for (const task of tasks) {
    const previous = speedSamples.get(task.id);
    const active = stateKey(task.state) === "downloading";
    const changed = !previous || task.received !== previous.bytes;
    const changedAt = changed ? now : previous.changedAt;
    const externalSpeed = now - changedAt < 2000 ? Number(task.download_speed) || 0 : 0;
    let speed = active ? previous?.speed || 0 : 0;
    if (previous && active) {
      const elapsed = Math.max(0.001, (now - previous.at) / 1000);
      const delta = Math.max(0, task.received - previous.bytes);
      if (delta > 0) {
        const instantaneous = delta / elapsed;
        speed = previous.speed ? instantaneous * 0.65 + previous.speed * 0.35 : instantaneous;
      } else if (now - changedAt >= 1500) {
        speed = 0;
      }
    }
    speedSamples.set(task.id, {
      at: now,
      bytes: task.received,
      speed: externalSpeed || speed,
      changedAt,
    });
    if (active) {
      overallSpeed += externalSpeed || speed;
      overallUploadSpeed += now - changedAt < 2000 ? Number(task.upload_speed) || 0 : 0;
    }
  }
}

function visibleDownloads() {
  let visible = downloads;
  if (activePage === "torrents") visible = visible.filter((task) => /^(?:magnet:)|\.torrent(?:$|[?#])/i.test(task.source));
  if (activePage === "media") visible = visible.filter((task) => !/\.recording\.webm$/i.test(`${task.source} ${task.destination}`) && /(?:\.m3u8(?:$|[?#])|youtube\.com|youtu\.be|facebook\.com|fb\.watch|tiktok\.com|instagram\.com)/i.test(`${task.source} ${task.destination}`));
  if (activePage === "recordings") visible = visible.filter((task) => /\.recording\.webm$/i.test(`${task.source} ${task.destination}`));
  if (activePage === "link") visible = visible.filter((task) => /^(?:ftp|sftp):/i.test(task.source));
  if (historyQuery) visible = visible.filter((task) => `${task.source} ${task.destination} ${task.sha256 || ""}`.toLocaleLowerCase().includes(historyQuery));
  if (activeFilter === "completed") return visible.filter((task) => task.state === "completed");
  if (activeFilter === "active") return visible.filter((task) => task.state !== "completed");
  return visible;
}

const failedThumbnailUrls = new Set();
let lastDownloadRenderSignature = "";

function renderDownloads(force = false) {
  const list = document.querySelector("#download-list");
  const empty = document.querySelector("#empty");
  const visible = visibleDownloads();
  const signature = JSON.stringify({
    locale,
    page: activePage,
    filter: activeFilter,
    query: historyQuery,
    selected: [...selectedIds].sort(),
    tasks: visible,
  });
  if (!force && signature === lastDownloadRenderSignature) return;
  lastDownloadRenderSignature = signature;
  list.replaceChildren();
  list.hidden = visible.length === 0;
  empty.hidden = visible.length !== 0;
  for (const task of visible) {
    const row = document.createElement("article");
    row.className = "download-row";
    const select = document.createElement("input");
    select.type = "checkbox";
    select.className = "task-select";
    select.checked = selectedIds.has(task.id);
    select.setAttribute(
      "aria-label",
      `${t("removeSelected")}: ${task.destination}`,
    );
    select.onchange = () => {
      if (select.checked) selectedIds.add(task.id);
      else selectedIds.delete(task.id);
      updateSelectionControls();
    };
    const icon = Object.assign(document.createElement("span"), {
      className: "download-icon",
      textContent: "⇩",
    });
    if (task.thumbnail && !failedThumbnailUrls.has(task.thumbnail)) {
      const thumbnail = document.createElement("img");
      thumbnail.className = "download-thumbnail";
      thumbnail.alt = "";
      thumbnail.referrerPolicy = "no-referrer";
      thumbnail.src = task.thumbnail;
      thumbnail.onerror = () => {
        failedThumbnailUrls.add(task.thumbnail);
        icon.replaceChildren(document.createTextNode("⇩"));
      };
      icon.replaceChildren(thumbnail);
      icon.classList.add("has-thumbnail");
    } else if (task.thumbnail) {
      icon.classList.add("has-thumbnail");
    }
    const info = Object.assign(document.createElement("div"), {
      className: "download-info",
    });
    const name = document.createElement("strong");
    name.textContent = task.display_title || task.destination.split(/[\\/]/).pop();
    name.title = task.display_title || "";
    const source = document.createElement("small");
    source.textContent = task.source;
    source.title = task.source;
    info.append(name, source);
    const progress = document.createElement("div");
    progress.className = "task-progress";
    const bar = document.createElement("i");
    const hasReportedPercent = task.progress_percent !== null
      && task.progress_percent !== undefined
      && Number.isFinite(Number(task.progress_percent));
    const reportedPercent = hasReportedPercent ? Number(task.progress_percent) : 0;
    const percent = hasReportedPercent
      ? Math.min(100, Math.max(0, reportedPercent))
      : task.total
        ? Math.min(100, (task.received / task.total) * 100)
        : 0;
    bar.style.width = `${percent}%`;
    const details = document.createElement("small");
    const speed = speedSamples.get(task.id)?.speed || 0;
    const uploadSpeed = performance.now() - (speedSamples.get(task.id)?.changedAt || 0) < 2000
      ? Number(task.upload_speed) || 0 : 0;
    const progressText = hasReportedPercent && !task.total
      ? `${percent.toFixed(1)}%`
      : task.total
      ? `${formatBytes(task.received)} / ${formatBytes(task.total)} · ${percent.toFixed(1)}%`
      : formatBytes(task.received);
    const torrentStats = task.torrent_seeders !== null && task.torrent_seeders !== undefined
      ? ` · S:${task.torrent_seeders} L:${task.torrent_leechers || 0}${task.torrent_eta ? ` · ETA ${task.torrent_eta}` : ""}` : "";
    details.textContent =
      speed && task.state === "downloading"
        ? `${progressText} · ↓ ${formatBytes(speed)}/s · ↑ ${formatBytes(uploadSpeed)}/s${torrentStats}`
        : `${progressText}${torrentStats}`;
    progress.append(bar);
    info.append(progress, details);
    const resumeCapability = document.createElement("strong");
    resumeCapability.className = "resume-capability";
    const resumeValue = task.resume_supported === true
      ? t("resumeYes")
      : task.resume_supported === false
        ? t("resumeNo")
        : t("resumeChecking");
    resumeCapability.textContent = `${t("resumeCapability")} ${resumeValue}`;
    resumeCapability.dataset.supported = task.resume_supported === true ? "true" : task.resume_supported === false ? "false" : "unknown";
    info.append(resumeCapability);
    const state = Object.assign(document.createElement("span"), {
      className: "download-state",
      textContent: /\.recording\.webm$/i.test(task.destination) && stateKey(task.state) === "downloading" ? t("recordingActive") : stateName(task.state),
    });
    if (typeof task.state === "object")
      state.title = task.state.failed?.message || "";
    const actions = document.createElement("div");
    actions.className = "task-actions";
    const addAction = (label, command) => {
      const button = document.createElement("button");
      button.className = "task-action";
      button.textContent = label;
      const execute = async () => {
        if (busyIds.has(task.id)) return;
        busyIds.add(task.id);
        button.disabled = true;
        try {
          await invoke(command, { id: task.id });
          await refreshDownloads();
        } catch (error) {
          console.error(error);
        } finally {
          busyIds.delete(task.id);
          button.disabled = false;
        }
      };
      button.onpointerdown = (event) => {
        if (event.button !== 0) return;
        event.preventDefault();
        execute();
      };
      button.onclick = (event) => {
        if (event.detail === 0) execute();
      };
      actions.append(button);
    };
    const key = stateKey(task.state);
    const recording = /\.recording\.webm$/i.test(task.destination);
    if (recording && key === "downloading") addAction(t("stopRecording"), "stop_recording");
    else if (key === "downloading" || key === "inspecting")
      addAction(t("pause"), "pause_download");
    if (key === "paused") addAction(t("resume"), "resume_download");
    if (key === "failed") addAction(t("retry"), "resume_download");
    if (key === "completed" && /\.recording\.webm$/i.test(task.destination)) {
      const exportButton = document.createElement("button");
      exportButton.className = "task-action";
      exportButton.textContent = t("export");
      exportButton.onclick = () => {
        exportTaskId = task.id;
        document.querySelector("#export-source").textContent = task.destination;
        document.querySelector("#export-destination").value = task.destination.replace(/[\\/][^\\/]+$/, "");
        document.querySelector("#export-format").value = "mkv";
        document.querySelector("#export-video-codec").value = "copy";
        document.querySelector("#export-audio-codec").value = "copy";
        exportDialog.showModal();
      };
      actions.append(exportButton);
    }
    if (key === "completed") {
      const verify = document.createElement("button");
      verify.className = "task-action";
      verify.textContent = task.integrity_verified ? "SHA-256 ✓" : t("verifyIntegrity");
      verify.title = task.sha256 || "";
      verify.onclick = async () => {
        const expectedSha256 = window.prompt(t("integrityPrompt"), task.sha256 || "");
        if (expectedSha256 === null) return;
        verify.disabled = true;
        try {
          const digest = await invoke("verify_download_integrity", { id: task.id, expectedSha256: expectedSha256 || null });
          window.alert(`${t("integrityOk")}: ${digest}`);
          await refreshDownloads();
        } catch (error) { window.alert(String(error)); }
        finally { verify.disabled = false; }
      };
      actions.append(verify);
    }
    if (/^(?:magnet:)|\.torrent(?:$|[?#])/i.test(task.source) && ["downloading", "paused", "completed"].includes(key))
      addAction(t("preview"), "preview_torrent");
    if (["queued", "inspecting", "downloading", "paused"].includes(key)) {
      const bandwidth = document.createElement("button");
      bandwidth.className = "task-action";
      bandwidth.textContent = t("bandwidthAction");
      bandwidth.title = task.bandwidth_limit ? `${(task.bandwidth_limit / 1024 / 1024).toFixed(1)} ${t("megabytesPerSecond")}` : t("unlimited");
      bandwidth.onclick = async () => {
        const current = task.bandwidth_limit ? task.bandwidth_limit / 1024 / 1024 : 0;
        const value = window.prompt(t("bandwidthPrompt"), String(current));
        if (value === null) return;
        const megabytes = Number(value.replace(",", "."));
        if (!Number.isFinite(megabytes) || megabytes < 0 || megabytes > 10240) return;
        await invoke("set_download_bandwidth_limit", {
          id: task.id,
          bandwidthLimit: Math.round(megabytes * 1024 * 1024),
        });
        await refreshDownloads();
      };
      actions.append(bandwidth);
    }
    addAction(t("openFolder"), "reveal_download");
    const status = document.createElement("div");
    status.className = "task-status";
    status.append(state, actions);
    row.append(select, icon, info, status);
    list.append(row);
  }
  document.querySelector(".metrics article:nth-child(4) strong").textContent =
    downloads.filter((task) => task.state === "queued").length;
  document.querySelector(".metrics article:nth-child(3) strong").textContent =
    downloads.filter((task) => task.state === "completed").length;
  document.querySelector(".metrics article:first-child strong").textContent =
    `${formatBytes(overallSpeed)}/s`;
  const bandwidthCurrent = document.querySelector("#bandwidth-current");
  if (bandwidthCurrent) bandwidthCurrent.textContent = `${formatBytes(overallSpeed)}/s`;
  document.querySelector(".metrics article:nth-child(2) strong").textContent =
    `${formatBytes(overallUploadSpeed)}/s`;
  updateSelectionControls();
}

function updateSelectionControls() {
  const visible = visibleDownloads();
  const selectAll = document.querySelector("#select-all");
  selectAll.disabled = visible.length === 0;
  selectAll.checked =
    visible.length > 0 && visible.every((task) => selectedIds.has(task.id));
  selectAll.indeterminate =
    visible.some((task) => selectedIds.has(task.id)) && !selectAll.checked;
  document.querySelector("#manage-list").disabled = selectedIds.size === 0;
  document.querySelector("#redownload-selected").disabled = selectedIds.size === 0;
}

function translate() {
  document.documentElement.lang = locale;
  document
    .querySelectorAll("[data-i18n]")
    .forEach((element) => (element.textContent = t(element.dataset.i18n)));
  document
    .querySelectorAll("[data-i18n-placeholder]")
    .forEach((element) => (element.placeholder = t(element.dataset.i18nPlaceholder)));
  document.querySelectorAll("[data-language-choice]").forEach((button) =>
    button.classList.toggle("active", button.dataset.languageChoice === locale));
  const activeNavigation = document.querySelector(`nav [data-page="${activePage}"]`);
  if (activeNavigation) document.querySelector("main > header h1").textContent = activeNavigation.querySelector("b")?.textContent || t("downloads");
  document.querySelector("#page-description").textContent = t(descriptions[activePage] || "downloadsDescription");
  renderDownloads(true);
  if (activePage === "link") {
    document.querySelector("#link-local-path").textContent = linkLocalPath || t("linkDrives");
    document.querySelector("#link-remote-path").textContent = linkRemotePath || t("linkDrives");
  }
}

async function refreshDownloads() {
  try {
    const refreshed = await invoke("list_downloads");
    if (selectionPointerActive) return;
    downloads = refreshed;
    const ids = new Set(downloads.map((task) => task.id));
    for (const id of selectedIds) if (!ids.has(id)) selectedIds.delete(id);
    updateSpeeds(downloads);
    renderDownloads();
  } catch (error) {
    console.error(error);
  }
}

const dialog = document.querySelector("#add-dialog");
const clearDialog = document.querySelector("#clear-dialog");
const settingsDialog = document.querySelector("#settings-dialog");
const toolsDialog = document.querySelector("#tools-dialog");
const logDialog = document.querySelector("#log-dialog");
const exportDialog = document.querySelector("#export-dialog");
const bandwidthDialog = document.querySelector("#bandwidth-dialog");
let exportTaskId = null;
document.querySelector("#history-search").oninput = (event) => {
  historyQuery = event.target.value.trim().toLocaleLowerCase();
  renderDownloads();
};
document.querySelector("#import-list").onclick = async (event) => {
  const button = event.currentTarget;
  button.disabled = true;
  try {
    const [urls, destinationDirectory] = await Promise.all([invoke("pick_url_list"), invoke("default_download_directory")]);
    for (const url of urls) {
      try {
        const fileName = await invoke("suggest_download_name", { url });
        downloads.push(await invoke("enqueue_download", { url, destinationDirectory, fileName, formatSelection: null, torrentSelection: null, mirrors: null, priority: 0, bandwidthLimit: null, connectionsOverride: 8, context: {} }));
      } catch (error) { console.warn("import", url, error); }
    }
    renderDownloads();
  } catch (error) { console.error(error); }
  finally { button.disabled = false; }
};
document.querySelectorAll('nav [data-page]:not([data-page="settings"]):not([data-page="tools"])').forEach((button) => {
  button.onclick = () => {
    const openedAt = performance.now();
    activePage = button.dataset.page;
    document.querySelectorAll("nav [data-page]").forEach((item) => item.classList.toggle("active", item === button));
    const heading = button.querySelector("b")?.textContent || t("downloads");
    document.querySelector("header h1").textContent = heading;
    document.querySelector("#page-description").textContent = t(descriptions[activePage] || "downloadsDescription");
    document.querySelector("#apocalipse-link-panel").hidden = activePage !== "link";
    document.querySelector("#logs-panel").hidden = activePage !== "logs";
    document.querySelector("#themes-panel").hidden = activePage !== "themes";
    document.querySelector("#language-panel").hidden = activePage !== "language";
    document.querySelector(".metrics").hidden = ["link", "logs", "themes", "language"].includes(activePage);
    document.querySelector(".panel").hidden = ["link", "logs", "themes", "language"].includes(activePage);
    renderDownloads();
    invoke("record_ui_diagnostic", { level: "INFO", event: "page_opened", detail: `page=${activePage} panel_present=${activePage === "link" ? Boolean(document.querySelector("#apocalipse-link-panel")) : activePage === "logs" ? Boolean(document.querySelector("#logs-panel")) : true} duration_ms=${Math.round(performance.now() - openedAt)}` }).catch(() => {});
    if (activePage === "logs") refreshLogEvents().catch(console.error);
  };
});

let logEvents = [];
function renderLogEvents() {
  const root = document.querySelector("#log-event-list");
  const empty = document.querySelector("#log-empty");
  const query = document.querySelector("#log-search").value.trim().toLowerCase();
  const level = document.querySelector("#log-level").value;
  const visible = logEvents.filter((item) => (!level || item.level === level) && (!query || JSON.stringify(item).toLowerCase().includes(query))).slice(-1000).reverse();
  root.replaceChildren();
  empty.hidden = visible.length > 0;
  for (const item of visible) {
    const row = document.createElement("article");
    row.className = `log-event level-${String(item.level || "INFO").toLowerCase()}`;
    const header = document.createElement("header");
    header.append(
      Object.assign(document.createElement("time"), { textContent: item.timestamp || item.time || "" }),
      Object.assign(document.createElement("b"), { textContent: item.level || "INFO" }),
      Object.assign(document.createElement("strong"), { textContent: item.event || "legacy" }),
      Object.assign(document.createElement("small"), { textContent: item.source || "desktop" }),
    );
    row.append(header, Object.assign(document.createElement("p"), { textContent: item.detail || item.raw || "" }));
    root.append(row);
  }
}
async function refreshLogEvents() {
  const contents = await invoke("read_general_log");
  logEvents = String(contents || "").split(/\r?\n/).filter(Boolean).map((line) => {
    try { return JSON.parse(line); } catch { return { level: / ERROR /.test(line) ? "ERROR" : / WARN /.test(line) ? "WARN" : "INFO", event: "legacy", raw: line }; }
  });
  renderLogEvents();
}
document.querySelector("#log-search").oninput = renderLogEvents;
document.querySelector("#log-level").onchange = renderLogEvents;
document.querySelector("#export-logs").onclick = async (event) => {
  // Event.currentTarget is cleared after the handler yields. Keep the element
  // itself so every completion path (saved, cancelled, or failed) re-enables it.
  const button = event.currentTarget;
  button.disabled = true;
  try { await invoke("export_diagnostic_bundle"); } catch (error) { window.alert(String(error)); }
  finally { button.disabled = false; }
};
document.querySelector("#clear-logs").onclick = async () => { await invoke("clear_general_log"); await refreshLogEvents(); };

let linkLocalPath = "";
let linkRemotePath = "";
let linkRemoteId = "";
let linkRemotePassword = "";
let linkSelectedLocal = "";
let linkSelectedRemote = "";
const linkParent = (path) => /^[A-Za-z]:[\\/]?$/.test(path) ? "" : path.replace(/[\\/]+$/, "").replace(/[\\/][^\\/]*$/, "");
function updateLinkTransferButtons() {
  document.querySelector("#link-upload-local").disabled = !linkSelectedLocal || !linkRemoteId || !linkRemotePath;
  document.querySelector("#link-download-remote").disabled = !linkSelectedRemote;
}
function renderLinkFiles(target, entries, open, select) {
  const root = document.querySelector(target);
  root.replaceChildren();
  for (const entry of entries) {
    const row = document.createElement("button");
    row.type = "button";
    row.className = "link-file";
    row.append(
      Object.assign(document.createElement("span"), { textContent: entry.directory ? "📁" : "📄" }),
      Object.assign(document.createElement("span"), { textContent: entry.name }),
      Object.assign(document.createElement("small"), { textContent: entry.directory ? "" : formatBytes(entry.size) }),
    );
    row.ondblclick = () => entry.directory && open(entry.path);
    row.onclick = () => {
      root.querySelectorAll(".selected").forEach((item) => item.classList.remove("selected"));
      row.classList.add("selected");
      select?.(entry);
    };
    root.append(row);
  }
}
async function openLocalLink(path = "") {
  linkLocalPath = path;
  linkSelectedLocal = "";
  updateLinkTransferButtons();
  document.querySelector("#link-local-path").textContent = path || t("linkDrives");
  renderLinkFiles("#link-local-files", await invoke("list_local_link_files", { path }), openLocalLink, (entry) => {
    linkSelectedLocal = entry.directory ? "" : entry.path;
    updateLinkTransferButtons();
  });
}
async function openRemoteLink(path = "") {
  linkRemotePath = path;
  linkSelectedRemote = "";
  updateLinkTransferButtons();
  document.querySelector("#link-remote-path").textContent = path || t("linkDrives");
  const entries = await invoke("list_remote_link_files", { id: linkRemoteId, password: linkRemotePassword, path });
  renderLinkFiles("#link-remote-files", entries, openRemoteLink, (entry) => {
    linkSelectedRemote = entry.directory ? "" : entry.path;
    updateLinkTransferButtons();
  });
}
async function loadLinkIdentity() {
  const identity = await invoke("get_link_identity");
  document.querySelector("#link-own-id").value = identity.id;
  document.querySelector("#link-own-password").value = identity.password;
  await openLocalLink();
  return identity;
}
document.querySelector('[data-page="link"]').addEventListener("click", () => loadLinkIdentity().catch(console.error));
document.querySelector("#link-new-password").onclick = async () => {
  document.querySelector("#link-own-password").value = await invoke("regenerate_link_password");
};
document.querySelector("#link-connect").onclick = async () => {
  linkRemoteId = document.querySelector("#link-remote-id").value.trim();
  linkRemotePassword = document.querySelector("#link-remote-password").value.trim();
  try { await openRemoteLink(); document.querySelector("#link-status").textContent = t("linkConnected"); }
  catch (error) { document.querySelector("#link-status").textContent = `${t("linkConnectionFailed")}: ${error}`; }
};
document.querySelector("#link-self-test").onclick = async () => {
  const identity = await loadLinkIdentity();
  document.querySelector("#link-remote-id").value = `127.0.0.1:${identity.port}`;
  document.querySelector("#link-remote-password").value = identity.password;
  document.querySelector("#link-connect").click();
};
document.querySelector("#link-local-up").onclick = () => openLocalLink(linkParent(linkLocalPath)).catch(console.error);
document.querySelector("#link-remote-up").onclick = () => openRemoteLink(linkParent(linkRemotePath)).catch(console.error);
document.querySelector("#link-download-remote").onclick = async () => {
  if (!linkSelectedRemote) return;
  const status = document.querySelector("#link-status");
  status.textContent = t("linkTransferring");
  try {
    const destination = await invoke("download_remote_link_file", { id: linkRemoteId, password: linkRemotePassword, path: linkSelectedRemote });
    status.textContent = `${t("linkCompleted")}: ${destination}`;
  } catch (error) { if (`${error}` !== "cancelled") status.textContent = `${t("linkTransferFailed")}: ${error}`; }
};
document.querySelector("#link-upload-local").onclick = async () => {
  if (!linkSelectedLocal || !linkRemoteId || !linkRemotePath) return;
  const status = document.querySelector("#link-status");
  const button = document.querySelector("#link-upload-local");
  status.textContent = t("linkSending");
  button.disabled = true;
  try {
    const remotePath = await invoke("upload_remote_link_file", { id: linkRemoteId, password: linkRemotePassword, remoteDirectory: linkRemotePath, localPath: linkSelectedLocal });
    status.textContent = `${t("linkCompleted")}: ${remotePath}`;
    await openRemoteLink(linkRemotePath);
  } catch (error) {
    status.textContent = `${t("linkUploadFailed")}: ${error}`;
  } finally {
    updateLinkTransferButtons();
  }
};
function updateLogEditorControls() {
  const configured = Boolean(document.querySelector("#log-editor").value.trim());
  document.querySelector("#remove-log-editor").disabled = !configured;
  document.querySelector("#open-log-external").disabled = !configured;
}
document.querySelector("#download-list").addEventListener("pointerdown", (event) => {
  if (event.target.closest?.(".task-select")) selectionPointerActive = true;
});
const finishSelectionPointer = () => setTimeout(() => {
  if (!selectionPointerActive) return;
  selectionPointerActive = false;
  refreshDownloads();
}, 0);
window.addEventListener("pointerup", finishSelectionPointer);
window.addEventListener("pointercancel", finishSelectionPointer);
async function refreshToolStatuses() {
  const button = document.querySelector("#check-tools");
  if (button) button.disabled = true;
  try {
    const tools = await invoke("get_tool_statuses");
    for (const tool of tools) {
      document.querySelector(`#tool-${tool.id}`).value = tool.path;
      const status = document.querySelector(`[data-tool="${tool.id}"] > span small`);
      status.textContent = tool.found ? `${t("installed")} · ${tool.version}` : t("missing");
      status.classList.toggle("tool-found", tool.found);
    }
  } catch (error) { console.error(error); }
  finally { if (button) button.disabled = false; }
}
async function refreshDestinationHistory() {
  try {
    const destinations = await invoke("list_download_directories");
    const root = document.querySelector("#destination-list");
    root.replaceChildren();
    for (const item of destinations) {
      const row = document.createElement("div");
      row.className = "destination-row";
      row.classList.toggle("unavailable", !item.available);
      const select = document.createElement("button");
      select.type = "button";
      select.className = "destination-select";
      const path = document.createElement("span");
      path.textContent = item.path;
      const badge = document.createElement("small");
      badge.textContent = item.isDefault ? t("defaultLocation") : (!item.available ? t("unavailableLocation") : "");
      select.append(path, badge);
      select.onclick = () => {
        document.querySelector("#destination").value = item.path;
      };
      row.append(select);
      if (!item.isDefault) {
        const remove = document.createElement("button");
        remove.type = "button";
        remove.className = "destination-remove";
        remove.textContent = "×";
        remove.onclick = async () => {
          await invoke("remove_download_directory", { path: item.path });
          await refreshDestinationHistory();
        };
        row.append(remove);
      }
      root.append(row);
    }
  } catch (error) {
    console.error(error);
  }
}
document.querySelectorAll("[data-pick-for]").forEach((button) => {
  button.onclick = async () => {
    const input = document.querySelector(`#${button.dataset.pickFor}`);
    button.disabled = true;
    try {
      const selected = await invoke("pick_directory", {
        initialDirectory: input.value,
      });
      if (selected) input.value = selected;
    } catch (error) {
      console.error(error);
    } finally {
      button.disabled = false;
    }
  };
});
document
  .querySelectorAll("[data-dialog-close]")
  .forEach((button) => (button.onclick = () => dialog.close()));
function resetMediaInspection() {
  const panel = document.querySelector("#media-inspection");
  const thumbnail = document.querySelector("#media-thumbnail");
  panel.hidden = true;
  thumbnail.hidden = true;
  thumbnail.removeAttribute("src");
  document.querySelector("#media-title").textContent = "";
  document.querySelector("#media-duration").textContent = "";
  document.querySelector("#media-format").replaceChildren();
  document.querySelector("#media-format-control").hidden = false;
  document.querySelector("#torrent-inspection").hidden = true;
  document.querySelector("#torrent-files").replaceChildren();
}

function showCapturedPreview({ title, thumbnail, kind, duration, size, showFormats = false }) {
  const panel = document.querySelector("#media-inspection");
  const image = document.querySelector("#media-thumbnail");
  document.querySelector("#media-title").textContent = title || document.querySelector("#file-name").value || t("newTask");
  document.querySelector("#media-duration").textContent = [
    kind ? String(kind).toUpperCase() : "",
    Number.isFinite(size) && size > 0 ? formatBytes(size) : "",
    Number.isFinite(duration) && duration > 0 ? `${t("duration")}: ${secondsLabel(duration)}` : "",
  ].filter(Boolean).join(" · ");
  document.querySelector("#media-format-control").hidden = !showFormats;
  image.hidden = !thumbnail;
  if (thumbnail) {
    image.src = thumbnail;
    image.onerror = () => { image.hidden = true; image.removeAttribute("src"); };
  } else image.removeAttribute("src");
  panel.hidden = false;
}

async function showTorrentInspection(source) {
  const torrent = await invoke("inspect_torrent_metadata", { source });
  document.querySelector("#torrent-title").textContent = torrent.name;
  document.querySelector("#torrent-total").textContent = formatBytes(torrent.totalSize);
  const root = document.querySelector("#torrent-files");
  root.replaceChildren();
  for (const file of torrent.files) {
    const row = document.createElement("label");
    const input = Object.assign(document.createElement("input"), { type: "checkbox", checked: true });
    input.dataset.torrentIndex = file.index;
    row.append(input, Object.assign(document.createElement("span"), { textContent: file.path }), Object.assign(document.createElement("small"), { textContent: formatBytes(file.size) }));
    root.append(row);
  }
  document.querySelector("#torrent-inspection").hidden = false;
}
document.querySelectorAll("#add,#empty-add").forEach(
  (button) =>
    (button.onclick = () => {
      document.querySelector("#analysis").hidden = true;
      document.querySelector("#enqueue").hidden = true;
      document.querySelector("#analyze").hidden = false;
      pendingReferer = null;
      pendingDuration = null;
      pendingTitle = null;
      pendingThumbnail = null;
      pendingMediaKind = null;
      pendingExpectedSize = null;
      pendingCookieHeader = null;
      pendingUserAgent = null;
      pendingRequestMethod = null;
      pendingRequestBody = null;
      pendingRequestContentType = null;
      resetMediaInspection();
      invoke("default_download_directory")
        .then((path) => {
          document.querySelector("#destination").value = path;
        })
        .catch(console.error);
      refreshDestinationHistory();
      dialog.showModal();
    }),
);
const selectLanguage = (language) => {
  locale = ["en", "pt-BR", "zh-CN"].includes(language) ? language : "en";
  localStorage.setItem("apocalipse.language", locale);
  translate();
  invoke("set_application_language", { language: locale }).catch(console.error);
};
document.querySelectorAll("[data-language-choice]").forEach((button) => {
  button.onclick = () => selectLanguage(button.dataset.languageChoice);
});
document.querySelectorAll(".tabs [data-filter]").forEach((button) => {
  button.onclick = () => {
    activeFilter = button.dataset.filter;
    document
      .querySelectorAll(".tabs [data-filter]")
      .forEach((tab) => tab.classList.toggle("active", tab === button));
    renderDownloads();
  };
});
document.querySelector("#select-all").onchange = (event) => {
  for (const task of visibleDownloads()) {
    if (event.target.checked) selectedIds.add(task.id);
    else selectedIds.delete(task.id);
  }
  renderDownloads();
};
document.querySelector("#manage-list").onclick = () => clearDialog.showModal();
document.querySelector("#redownload-selected").onclick = async (event) => {
  const button = event.currentTarget;
  button.disabled = true;
  try {
    await invoke("redownload_downloads", { ids: [...selectedIds] });
    selectedIds.clear();
    await refreshDownloads();
  } catch (error) {
    console.error(error);
  } finally {
    updateSelectionControls();
  }
};
document.querySelector("#clear-destinations").onclick = async () => {
  try {
    await invoke("clear_download_directories");
    await refreshDestinationHistory();
  } catch (error) {
    console.error(error);
  }
};
document
  .querySelectorAll("[data-clear-cancel]")
  .forEach((button) => (button.onclick = () => clearDialog.close()));
document.querySelectorAll("[data-clear-mode]").forEach((button) => {
  button.onclick = async () => {
    button.disabled = true;
    try {
      await invoke("remove_downloads", {
        ids: [...selectedIds],
        deleteFiles: button.dataset.clearMode === "files",
      });
      selectedIds.clear();
      clearDialog.close();
      await refreshDownloads();
    } catch (error) {
      console.error(error);
      window.alert(`${t("removeFailed")}: ${error}`);
    } finally {
      button.disabled = false;
    }
  };
});
function updateProxyControls() {
  const enabled = document.querySelector("#proxy-enabled").checked;
  for (const id of ["#proxy-url", "#proxy-username", "#proxy-password", "#proxy-clear-password"]) {
    document.querySelector(id).disabled = !enabled;
  }
}
document.querySelector("#proxy-enabled").onchange = updateProxyControls;
function updateDnsControls() {
  const enabled = document.querySelector("#dns-enabled").checked;
  document.querySelector("#dns-preset").disabled = !enabled;
  document.querySelector("#dns-servers").disabled = !enabled;
}
document.querySelector("#dns-enabled").onchange = updateDnsControls;
document.querySelector("#dns-preset").onchange = (event) => {
  if (event.target.value !== "custom") {
    document.querySelector("#dns-servers").value = event.target.value;
  }
};
function renderWebsiteCredentials(credentials) {
  const list = document.querySelector("#website-credential-list");
  list.replaceChildren();
  if (!credentials.length) {
    const empty = document.createElement("small");
    empty.textContent = t("websiteCredentialsEmpty");
    list.append(empty);
    return;
  }
  for (const credential of credentials) {
    const row = document.createElement("div");
    const identity = document.createElement("span");
    const host = document.createElement("b");
    const username = document.createElement("small");
    const remove = document.createElement("button");
    host.textContent = credential.host;
    username.textContent = credential.username;
    identity.append(host, username);
    remove.type = "button";
    remove.textContent = t("websiteCredentialRemove");
    remove.onclick = async () => {
      remove.disabled = true;
      try {
        renderWebsiteCredentials(await invoke("remove_website_credential", { host: credential.host }));
      } catch (error) {
        console.error(error);
        remove.disabled = false;
      }
    };
    row.append(identity, remove);
    list.append(row);
  }
}
document.querySelector("#save-website-credential").onclick = async (event) => {
  const button = event.currentTarget;
  const host = document.querySelector("#website-credential-host");
  const username = document.querySelector("#website-credential-username");
  const password = document.querySelector("#website-credential-password");
  if (![host, username, password].every((input) => input.reportValidity()) || !host.value.trim() || !username.value.trim() || !password.value) return;
  button.disabled = true;
  try {
    renderWebsiteCredentials(await invoke("save_website_credential", {
      host: host.value,
      username: username.value,
      password: password.value,
    }));
    host.value = "";
    username.value = "";
    password.value = "";
  } catch (error) {
    console.error(error);
    window.alert(String(error));
  } finally {
    button.disabled = false;
  }
};
const openSettings = async (target = "general") => {
  try {
    const [autostart, directory, clipboard, limits, pairing, userAgent, logEditor, proxy, dns, associations, websiteCredentials] = await Promise.all([
      invoke("get_autostart"),
      invoke("default_download_directory"),
      invoke("get_clipboard_monitor"),
      invoke("get_transfer_limits"),
      invoke("get_bridge_pairing"),
      invoke("get_user_agent"),
      invoke("get_log_editor"),
      invoke("get_proxy_setting"),
      invoke("get_dns_setting"),
      invoke("get_associations"),
      invoke("list_website_credentials"),
    ]);
    document.querySelector("#autostart").checked = autostart.enabled;
    document.querySelector("#theme").value = document.documentElement.dataset.theme;
    document.querySelector("#adaptive-efficiency").checked = limits.adaptiveEfficiency;
    document.querySelector("#schedule-enabled").checked = localStorage.getItem("apocalipse.schedule.enabled") === "true";
    document.querySelector("#schedule-start").value = localStorage.getItem("apocalipse.schedule.start") || "00:00";
    document.querySelector("#schedule-end").value = localStorage.getItem("apocalipse.schedule.end") || "23:59";
    for (const association of associations) {
      const input = document.querySelector(`[data-association="${association.id}"]`);
      input.checked = association.enabled;
      input.dataset.initial = String(association.enabled);
      input.disabled = !association.supported;
    }
    document.querySelector("#default-directory").value = directory;
    document.querySelector("#capture-clipboard").checked = clipboard.enabled;
    document.querySelector("#max-tasks").value = limits.maxActiveDownloads;
    document.querySelector("#connections").value = limits.connectionsPerDownload;
    document.querySelector("#global-bandwidth-limit").value = limits.globalBandwidthLimit
      ? (limits.globalBandwidthLimit / 1024 / 1024).toFixed(1) : 0;
    updateLimitLabels();
    document.querySelector("#pairing-token").value = pairing.token;
    document.querySelector("#user-agent").value = userAgent.userAgent;
    document.querySelector("#log-editor").value = logEditor;
    document.querySelector("#proxy-enabled").checked = proxy.enabled;
    document.querySelector("#proxy-url").value = proxy.url;
    document.querySelector("#proxy-username").value = proxy.username;
    document.querySelector("#proxy-password").value = "";
    document.querySelector("#proxy-password").placeholder = proxy.hasPassword
      ? "••••••••"
      : t("proxyPasswordHint");
    document.querySelector("#proxy-clear-password").checked = false;
    updateProxyControls();
    const dnsValue = dns.servers.join(",");
    document.querySelector("#dns-enabled").checked = dns.enabled;
    document.querySelector("#dns-servers").value = dns.servers.join(", ");
    document.querySelector("#dns-preset").value = ["1.1.1.1,1.0.0.1", "8.8.8.8,8.8.4.4", "9.9.9.9,149.112.112.112"].includes(dnsValue)
      ? dnsValue
      : "custom";
    updateDnsControls();
    renderWebsiteCredentials(websiteCredentials);
    updateLogEditorControls();
    settingsDialog.showModal();
    const targetElement = {
      general: document.querySelector("#autostart"),
    }[target];
    targetElement?.scrollIntoView?.({ block: "center" });
    targetElement?.focus?.();
    invoke("record_ui_diagnostic", { level: "INFO", event: "settings_section_opened", detail: `section=${target} found=${Boolean(targetElement)}` }).catch(() => {});
  } catch (error) {
    console.error(error);
  }
};
document.querySelectorAll("nav [data-settings-target]").forEach((button) => {
  button.onclick = () => {
    document.querySelectorAll("nav button").forEach((item) => item.classList.toggle("active", item === button));
    document.querySelector("main > header h1").textContent = button.querySelector("b")?.textContent || t("settings");
    document.querySelector("#page-description").textContent = t("settingsDescription");
    openSettings(button.dataset.settingsTarget).catch(console.error);
  };
});
document
  .querySelectorAll("[data-settings-close]")
  .forEach((button) => (button.onclick = () => {
    applyTheme(localStorage.getItem("apocalipse.theme") || "void");
    settingsDialog.close();
  }));
document.querySelector("#theme").onchange = (event) => {
  localStorage.setItem("apocalipse.theme", event.target.value);
  applyTheme(event.target.value);
};
function syncAppearanceControls() {
  const settings = readAppearance();
  document.querySelector("#transparency-enabled").checked = settings.transparencyEnabled;
  document.querySelector("#transparency-level").value = settings.transparencyLevel;
  document.querySelector("#transparency-level").disabled = !settings.transparencyEnabled;
  document.querySelector("#transparency-value").textContent = `${settings.transparencyLevel}%`;
  document.querySelector("#rounded-enabled").checked = settings.roundedEnabled;
  document.querySelector("#corner-radius").value = settings.cornerRadius;
  document.querySelector("#corner-radius").disabled = !settings.roundedEnabled;
  document.querySelector("#corner-radius-value").textContent = `${settings.cornerRadius} px`;
  document.querySelector("#interface-size").value = settings.interfaceSize;
}
function saveAppearanceFromControls() {
  const settings = {
    transparencyEnabled: document.querySelector("#transparency-enabled").checked,
    transparencyLevel: Number(document.querySelector("#transparency-level").value),
    roundedEnabled: document.querySelector("#rounded-enabled").checked,
    cornerRadius: Number(document.querySelector("#corner-radius").value),
    interfaceSize: document.querySelector("#interface-size").value,
  };
  localStorage.setItem("apocalipse.appearance", JSON.stringify(settings));
  applyAppearance(settings);
  syncAppearanceControls();
  invoke("record_ui_diagnostic", { level: "INFO", event: "appearance_changed", detail: `transparency=${settings.transparencyEnabled} level=${settings.transparencyLevel} rounded=${settings.roundedEnabled} radius=${settings.cornerRadius} size=${settings.interfaceSize}` }).catch(() => {});
}
["transparency-enabled", "transparency-level", "rounded-enabled", "corner-radius", "interface-size"].forEach((id) => {
  document.querySelector(`#${id}`).oninput = saveAppearanceFromControls;
  document.querySelector(`#${id}`).onchange = saveAppearanceFromControls;
});
syncAppearanceControls();
document.querySelector("#save-settings").onclick = async () => {
  const button = document.querySelector("#save-settings");
  const directory = document.querySelector("#default-directory");
  if (!directory.reportValidity()) return;
  button.disabled = true;
  try {
    const theme = document.querySelector("#theme").value;
    localStorage.setItem("apocalipse.theme", theme);
    localStorage.setItem("apocalipse.schedule.enabled", String(document.querySelector("#schedule-enabled").checked));
    localStorage.setItem("apocalipse.schedule.start", document.querySelector("#schedule-start").value);
    localStorage.setItem("apocalipse.schedule.end", document.querySelector("#schedule-end").value);
    applyTheme(theme);
    await invoke("set_default_download_directory", { path: directory.value });
    await invoke("set_autostart", {
      enabled: document.querySelector("#autostart").checked,
    });
    await invoke("set_clipboard_monitor", {
      enabled: document.querySelector("#capture-clipboard").checked,
    });
    await invoke("set_transfer_limits", {
      maxActiveDownloads: Number(document.querySelector("#max-tasks").value),
      connectionsPerDownload: Number(document.querySelector("#connections").value),
      adaptiveEfficiency: document.querySelector("#adaptive-efficiency").checked,
      globalBandwidthLimit: Math.round((Number(document.querySelector("#global-bandwidth-limit").value) || 0) * 1024 * 1024),
    });
    await invoke("set_user_agent", {
      userAgent: document.querySelector("#user-agent").value,
    });
    await invoke("set_proxy_setting", {
      enabled: document.querySelector("#proxy-enabled").checked,
      url: document.querySelector("#proxy-url").value,
      username: document.querySelector("#proxy-username").value,
      password: document.querySelector("#proxy-password").value,
      clearPassword: document.querySelector("#proxy-clear-password").checked,
    });
    await invoke("set_dns_setting", {
      enabled: document.querySelector("#dns-enabled").checked,
      servers: document.querySelector("#dns-servers").value
        .split(/[;,\s]+/)
        .map((server) => server.trim())
        .filter(Boolean),
    });
    for (const input of document.querySelectorAll("[data-association]")) {
      if (!input.disabled && input.dataset.initial !== String(input.checked)) await invoke("set_association", {
        id: input.dataset.association,
        enabled: input.checked,
      });
    }
    settingsDialog.close();
  } catch (error) {
    console.error(error);
  } finally {
    button.disabled = false;
  }
};
document.querySelector("#open-bandwidth-panel").onclick = () => bandwidthDialog.showModal();
document.querySelectorAll("[data-bandwidth-close]").forEach((button) => button.onclick = () => bandwidthDialog.close());
document.querySelector("#save-bandwidth").onclick = async (event) => {
  const button = event.currentTarget;
  const value = document.querySelector("#global-bandwidth-limit");
  if (!value.reportValidity()) return;
  button.disabled = true;
  try {
    const limits = await invoke("get_transfer_limits");
    await invoke("set_transfer_limits", {
      maxActiveDownloads: limits.maxActiveDownloads,
      connectionsPerDownload: limits.connectionsPerDownload,
      adaptiveEfficiency: limits.adaptiveEfficiency,
      globalBandwidthLimit: Math.round((Number(value.value) || 0) * 1024 * 1024),
    });
    bandwidthDialog.close();
  } catch (error) { window.alert(String(error)); }
  finally { button.disabled = false; }
};
document.querySelector('[data-page="tools"]').onclick = async () => {
  try {
    document.querySelector("#media-player").value = await invoke("get_media_player");
    await refreshToolStatuses();
    toolsDialog.showModal();
  } catch (error) { console.error(error); }
};
document.querySelectorAll("[data-tools-close]").forEach((button) => button.onclick = () => toolsDialog.close());
document.querySelector("#save-tools").onclick = async (event) => {
  const button = event.currentTarget;
  button.disabled = true;
  try {
    await invoke("set_tool_paths", {
      ffmpeg: document.querySelector("#tool-ffmpeg").value,
      ytDlp: document.querySelector("#tool-yt-dlp").value,
      qjs: document.querySelector("#tool-qjs").value,
      nM3u8dlRe: document.querySelector("#tool-n-m3u8dl-re").value,
      aria2: document.querySelector("#tool-aria2").value,
    });
    await invoke("set_media_player", { path: document.querySelector("#media-player").value });
    toolsDialog.close();
  } catch (error) { console.error(error); }
  finally { button.disabled = false; }
};
document.querySelectorAll("[data-tool-update]").forEach((button) => {
  button.onclick = async () => {
    button.disabled = true;
    try {
      const message = await invoke("update_tool", { id: button.dataset.toolUpdate });
      await refreshToolStatuses();
      alert(message);
    } catch (error) {
      alert(String(error).replace("manual_update_required:", "Atualização manual necessária:"));
    } finally {
      button.disabled = false;
    }
  };
});
document.querySelectorAll("[data-export-close]").forEach((button) => button.onclick = () => exportDialog.close());
document.querySelector("#donate-paypal").onclick = () => invoke("open_paypal_donation").catch(console.error);
document.querySelector("#export-format").onchange = (event) => {
  document.querySelector("#export-video-codec").disabled = ["mp3", "m4a", "opus", "flac", "wav"].includes(event.target.value);
};
document.querySelector("#export-recording").onclick = async (event) => {
  const button = event.currentTarget;
  button.disabled = true;
  try {
    await invoke("export_recording", {
      id: exportTaskId,
      format: document.querySelector("#export-format").value,
      videoCodec: document.querySelector("#export-video-codec").value,
      audioCodec: document.querySelector("#export-audio-codec").value,
      outputDirectory: document.querySelector("#export-destination").value,
    });
    exportDialog.close();
    await refreshDownloads();
  } catch (error) { console.error(error); }
  finally { button.disabled = false; }
};
async function refreshDiagnosticLog() {
  const output = document.querySelector("#diagnostic-log");
  const contents = await invoke("read_general_log");
  output.textContent = contents || t("emptyLog");
  output.scrollTop = output.scrollHeight;
}
document.querySelector("#open-log").onclick = async () => {
  try {
    await refreshDiagnosticLog();
    logDialog.showModal();
  } catch (error) { console.error(error); }
};
document.querySelector("#clear-log").onclick = async (event) => {
  const button = event.currentTarget;
  button.disabled = true;
  try {
    await invoke("clear_general_log");
    if (logDialog.open) await refreshDiagnosticLog();
  }
  catch (error) { console.error(error); }
  finally { button.disabled = false; }
};
document.querySelector("#refresh-log").onclick = () => refreshDiagnosticLog().catch(console.error);
document.querySelector("#open-log-external").onclick = () => invoke("open_log_external").catch(console.error);
document.querySelector("#pick-log-editor").onclick = async () => {
  const input = document.querySelector("#log-editor");
  try {
    const selected = await invoke("pick_executable", { initialPath: input.value });
    if (selected) {
      input.value = await invoke("set_log_editor", { path: selected });
      updateLogEditorControls();
    }
  } catch (error) { console.error(error); }
};
document.querySelector("#remove-log-editor").onclick = async () => {
  try {
    document.querySelector("#log-editor").value = await invoke("set_log_editor", { path: "" });
    updateLogEditorControls();
  } catch (error) { console.error(error); }
};
document.querySelectorAll("[data-log-close]").forEach((button) => {
  button.onclick = () => logDialog.close();
});
document.querySelectorAll("[data-tool-pick]").forEach((button) => {
  button.onclick = async () => {
    const input = document.querySelector(`#tool-${button.dataset.toolPick}`);
    button.disabled = true;
    try {
      const selected = await invoke("pick_executable", { initialPath: input.value });
      if (selected) input.value = selected;
    } catch (error) { console.error(error); }
    finally { button.disabled = false; }
  };
});
document.querySelector("#pick-media-player").onclick = async (event) => {
  const button = event.currentTarget;
  button.disabled = true;
  try {
    const input = document.querySelector("#media-player");
    const selected = await invoke("pick_executable", { initialPath: input.value });
    if (selected) input.value = selected;
  } catch (error) { console.error(error); }
  finally { button.disabled = false; }
};
function updateLimitLabels() {
  document.querySelector("#max-tasks-value").value = document.querySelector("#max-tasks").value;
  document.querySelector("#connections-value").value = document.querySelector("#connections").value;
}
document.querySelector("#max-tasks").oninput = updateLimitLabels;
document.querySelector("#connections").oninput = updateLimitLabels;
document.querySelector("#default-limits").onclick = () => {
  document.querySelector("#max-tasks").value = 3;
  document.querySelector("#connections").value = 8;
  updateLimitLabels();
};
document.querySelector("#copy-pairing").onclick = () => invoke("copy_bridge_token").catch(console.error);
document.querySelector("#regenerate-pairing").onclick = async () => {
  try {
    const pairing = await invoke("regenerate_bridge_token");
    document.querySelector("#pairing-token").value = pairing.token;
  } catch (error) {
    console.error(error);
  }
};
document.querySelector("#url").oninput = () => {
  document.querySelector("#analysis").hidden = true;
  document.querySelector("#enqueue").hidden = true;
  document.querySelector("#analyze").hidden = false;
  resetMediaInspection();
};

function secondsLabel(value) {
  if (!Number.isFinite(value)) return "";
  const hours = Math.floor(value / 3600);
  const minutes = Math.floor((value % 3600) / 60);
  const seconds = Math.floor(value % 60);
  return [hours, minutes, seconds].filter((_, index) => index || hours).map((item) => String(item).padStart(2, "0")).join(":");
}

function option(select, value, label) {
  select.append(Object.assign(document.createElement("option"), { value, textContent: label }));
}

async function showMediaInspection(url) {
  const panel = document.querySelector("#media-inspection");
  const select = document.querySelector("#media-format");
  select.replaceChildren();
  option(select, "bestvideo+bestaudio/best", t("bestQuality"));
  document.querySelector("#media-format-control").hidden = false;
  for (const format of ["mp3", "m4a", "opus", "flac", "wav"])
    option(select, `audio:${format}`, `${t("audioOnly")} · ${format.toUpperCase()}`);
  try {
    const media = await invoke("inspect_media_formats", {
      url,
      cookieHeader: pendingCookieHeader,
      userAgent: pendingUserAgent,
      referer: pendingReferer,
    });
    pendingTitle = media.title || pendingTitle;
    pendingThumbnail = media.thumbnail || pendingThumbnail;
    pendingDuration = Number.isFinite(media.duration) ? media.duration : pendingDuration;
    document.querySelector("#media-title").textContent = media.title;
    document.querySelector("#media-duration").textContent = media.duration ? `${t("duration")}: ${secondsLabel(media.duration)}` : "";
    const thumbnail = document.querySelector("#media-thumbnail");
    thumbnail.hidden = !media.thumbnail;
    if (media.thumbnail) thumbnail.src = media.thumbnail;
    else thumbnail.removeAttribute("src");
    for (const format of media.formats) option(select, format.selection, format.label);
    document.querySelector("#file-name").value = media.suggestedFileName;
    panel.hidden = false;
  } catch (error) {
    console.warn(error);
    // The extension may already have supplied trustworthy title/thumbnail
    // metadata. Preserve it when yt-dlp inspection is blocked by a VPN,
    // CAPTCHA or transient anti-bot response.
    showCapturedPreview({
      title: pendingTitle || t("mediaUnavailable"),
      thumbnail: pendingThumbnail,
      kind: pendingMediaKind || "video",
      duration: pendingDuration,
      size: pendingExpectedSize,
      showFormats: true,
    });
  }
}
document.querySelector("#media-format").onchange = (event) => {
  const audio = event.target.value.match(/^audio:(.+)$/);
  if (!audio) return;
  const input = document.querySelector("#file-name");
  const base = input.value.replace(/\.[^.]+$/, "");
  input.value = `${base}.${audio[1]}`;
};
document.querySelector("#task-connections").oninput = (event) => {
  document.querySelector("#task-connections-value").value = event.target.value;
};
document.querySelector("#analyze").onclick = async () => {
  const url = document.querySelector("#url");
  if (!url.reportValidity()) return;
  const analyzeButton = document.querySelector("#analyze");
  if (analyzeButton.disabled) return;
  analyzeButton.disabled = true;
  const box = document.querySelector("#analysis");
  box.hidden = false;
  box.textContent = "…";
  let metadataTimer = null;
  try {
    const plan = await invoke("inspect_url", { url: url.value });
    const fileName = document.querySelector("#file-name");
    const suggestedFileName = await invoke(
      "suggest_download_name",
      { url: url.value },
    );
    const currentName = fileName.value.trim();
    const genericName = /^(?:watch|reel|video|download)(?:\.[a-z0-9]{1,10})?$/i.test(currentName);
    if (!currentName || (genericName && !pendingTitle)) {
      fileName.value = suggestedFileName;
    }
    box.textContent = `${plan.primary} · ${plan.reason}`;
    if (plan.primary === "YtDlp") await showMediaInspection(url.value);
    else if (plan.primary === "Aria2Rpc" && (/^magnet:/i.test(url.value) || /\.torrent$/i.test(url.value.split(/[?#]/)[0]))) {
      const startedAt = Date.now();
      const updateMetadataStatus = () => {
        const seconds = Math.floor((Date.now() - startedAt) / 1000);
        box.textContent = `${t("torrentMetadataSeeking")} · ${seconds}s`;
      };
      updateMetadataStatus();
      metadataTimer = window.setInterval(updateMetadataStatus, 1000);
      await showTorrentInspection(url.value);
      window.clearInterval(metadataTimer);
      metadataTimer = null;
      box.textContent = `${plan.primary} · ${plan.reason}`;
    }
    else if (plan.primary === "NM3u8DlRe") {
      const select = document.querySelector("#media-format");
      select.replaceChildren();
      option(select, "", t("bestQuality"));
      for (const format of ["mp3", "m4a", "opus", "flac", "wav"])
        option(select, `audio:${format}`, `${t("audioOnly")} · ${format.toUpperCase()}`);
      showCapturedPreview({ title: pendingTitle || "HLS", thumbnail: pendingThumbnail, kind: "M3U8 / HLS", duration: pendingDuration, size: pendingExpectedSize, showFormats: true });
    } else if (pendingMediaKind === "image" || /\.(?:avif|bmp|gif|jpe?g|png|svg|webp)(?:$|[?#])/i.test(url.value)) {
      showCapturedPreview({ title: pendingTitle || fileName.value, thumbnail: pendingThumbnail || url.value, kind: pendingMediaKind || "image", duration: null, size: pendingExpectedSize });
    }
    document.querySelector("#analyze").hidden = true;
    document.querySelector("#enqueue").hidden = false;
  } catch (error) {
    box.textContent = String(error);
  } finally {
    if (metadataTimer) window.clearInterval(metadataTimer);
    analyzeButton.disabled = false;
  }
};
document.querySelector("#enqueue").onclick = async () => {
  const url = document.querySelector("#url");
  const button = document.querySelector("#enqueue");
  button.disabled = true;
  try {
    const torrentSelection = document.querySelector("#torrent-inspection").hidden
      ? null : [...document.querySelectorAll("[data-torrent-index]:checked")].map((input) => Number(input.dataset.torrentIndex));
    if (torrentSelection && !torrentSelection.length) throw new Error("Selecione pelo menos um arquivo do torrent.");
    downloads.push(
      await invoke("enqueue_download", {
        url: url.value,
        destinationDirectory: document.querySelector("#destination").value,
        fileName: document.querySelector("#file-name").value,
        formatSelection: document.querySelector("#media-inspection").hidden || document.querySelector("#media-format-control").hidden ? null : document.querySelector("#media-format").value,
        torrentSelection,
        mirrors: document.querySelector("#mirrors").value.split(/\r?\n/).map((value) => value.trim()).filter(Boolean),
        priority: Number(document.querySelector("#priority").value),
        bandwidthLimit: Math.round((Number(document.querySelector("#download-bandwidth-limit").value) || 0) * 1024 * 1024) || null,
        connectionsOverride: Number(document.querySelector("#task-connections").value) || 8,
        context: {
          referer: pendingReferer,
          knownDuration: pendingDuration,
          title: pendingTitle,
          thumbnail: pendingThumbnail,
          cookieHeader: pendingCookieHeader,
          userAgent: pendingUserAgent,
          requestMethod: pendingRequestMethod,
          requestBody: pendingRequestBody,
          requestContentType: pendingRequestContentType,
        },
      }),
    );
    renderDownloads();
    dialog.close();
    url.value = "";
    document.querySelector("#mirrors").value = "";
    document.querySelector("#priority").value = "0";
    document.querySelector("#download-bandwidth-limit").value = "0";
    document.querySelector("#task-connections").value = "8";
    document.querySelector("#task-connections-value").value = "8";
    pendingTitle = null;
    pendingThumbnail = null;
    pendingMediaKind = null;
    pendingExpectedSize = null;
    resetMediaInspection();
  } catch (error) {
    const box = document.querySelector("#analysis");
    box.hidden = false;
    box.textContent = String(error);
  } finally {
    button.disabled = false;
  }
};

translate();
refreshDownloads();
setInterval(refreshDownloads, 250);
setInterval(async () => {
  if (localStorage.getItem("apocalipse.schedule.enabled") !== "true") return;
  const now = new Date();
  const current = now.getHours() * 60 + now.getMinutes();
  const minutes = (value) => { const [h, m] = value.split(":").map(Number); return h * 60 + m; };
  const start = minutes(localStorage.getItem("apocalipse.schedule.start") || "00:00");
  const end = minutes(localStorage.getItem("apocalipse.schedule.end") || "23:59");
  const allowed = start <= end ? current >= start && current <= end : current >= start || current <= end;
  for (const task of downloads) {
    const key = stateKey(task.state);
    if (!allowed && key === "downloading" && !schedulerPaused.has(task.id)) {
      schedulerPaused.add(task.id);
      invoke("pause_download", { id: task.id }).catch(console.error);
    } else if (allowed && key === "paused" && schedulerPaused.delete(task.id)) {
      invoke("resume_download", { id: task.id }).catch(console.error);
    }
  }
}, 5000);
setInterval(async () => {
  try {
    const link = await invoke("read_clipboard_link");
    if (!clipboardMonitorPrimed) {
      lastClipboardLink = link || "";
      clipboardMonitorPrimed = true;
      return;
    }
    if (!link || link === lastClipboardLink) return;
    lastClipboardLink = link;
    // Never replace a link already being reviewed in the save dialog. This is
    // especially important on SPA feeds such as TikTok, where the clipboard may
    // still contain a previously copied reel while the extension sends a new one.
    if (dialog.open) return;
    pendingReferer = null;
    pendingDuration = null;
    pendingTitle = null;
    pendingThumbnail = null;
    pendingMediaKind = null;
    pendingExpectedSize = null;
    pendingCookieHeader = null;
    pendingUserAgent = null;
    pendingRequestMethod = null;
    pendingRequestBody = null;
    pendingRequestContentType = null;
    const url = document.querySelector("#url");
    url.value = link;
    document.querySelector("#analysis").hidden = true;
    document.querySelector("#enqueue").hidden = true;
    document.querySelector("#analyze").hidden = false;
    resetMediaInspection();
    await invoke("activate_main_window");
    if (!dialog.open) dialog.showModal();
    url.focus();
  } catch (error) {
    console.error(error);
  }
}, 750);
let consumingBridgeDownload = false;
async function consumeBridgeDownload() {
  if (consumingBridgeDownload) return;
  consumingBridgeDownload = true;
  try {
    const currentUrl = dialog.open ? document.querySelector("#url").value : null;
    const request = await invoke("take_bridge_download", { currentUrl });
    if (!request) return;
    lastClipboardLink = request.url;
    pendingReferer = request.pageUrl || null;
    pendingDuration = Number.isFinite(request.duration) ? request.duration : null;
    pendingTitle = request.title || null;
    pendingThumbnail = request.thumbnail || null;
    pendingMediaKind = request.mediaKind || null;
    pendingExpectedSize = Number.isFinite(request.expectedSize) ? request.expectedSize : null;
    pendingCookieHeader = request.cookieHeader || null;
    pendingUserAgent = request.userAgent || null;
    pendingRequestMethod = request.requestMethod || null;
    pendingRequestBody = request.requestBody || null;
    pendingRequestContentType = request.requestContentType || null;
    document.querySelector("#url").value = request.url;
    const requestedName = request.fileName || "";
    const genericMediaName = /^(?:watch|reel|video|download)(?:\.[a-z0-9]{1,10})?$/i.test(requestedName.trim());
    const titleName = String(pendingTitle || "")
      .replace(/[<>:\"/\\|?*\u0000-\u001f]/g, "_")
      .replace(/[. ]+$/g, "")
      .trim();
    document.querySelector("#file-name").value = pendingMediaKind === "video" && titleName && (!requestedName || genericMediaName)
      ? `${[...titleName].slice(0, 110).join("")}.mp4`
      : requestedName;
    document.querySelector("#analysis").hidden = true;
    document.querySelector("#enqueue").hidden = true;
    document.querySelector("#analyze").hidden = false;
    resetMediaInspection();
    if (pendingThumbnail || pendingTitle || pendingMediaKind === "image") {
      showCapturedPreview({ title: pendingTitle, thumbnail: pendingThumbnail || (pendingMediaKind === "image" ? request.url : null), kind: pendingMediaKind, duration: pendingDuration, size: pendingExpectedSize });
    }
    document.querySelector("#destination").value = await invoke("default_download_directory");
    await refreshDestinationHistory();
    await invoke("activate_main_window");
    if (!dialog.open) dialog.showModal();
    document.querySelector("#url").focus();
  } catch (error) { console.error(error); }
  finally { consumingBridgeDownload = false; }
}
setInterval(consumeBridgeDownload, 400);
window.__TAURI__?.event?.listen?.("bridge-download-ready", consumeBridgeDownload).catch(console.error);
window.__TAURI__?.event?.listen?.("recording-completed", async (event) => {
  try {
    await invoke("activate_main_window");
    await refreshDownloads();
    exportTaskId = event.payload;
    const task = downloads.find((item) => item.id === exportTaskId);
    document.querySelector("#export-source").textContent = task?.destination || "";
    document.querySelector("#export-destination").value = task?.destination?.replace(/[\\/][^\\/]+$/, "") || await invoke("default_download_directory");
    document.querySelector("#export-format").value = "mkv";
    document.querySelector("#export-video-codec").value = "copy";
    document.querySelector("#export-audio-codec").value = "copy";
    if (!exportDialog.open) exportDialog.showModal();
  } catch (error) { console.error(error); }
}).catch(console.error);
async function refreshBridgeStatus() {
  try {
    const status = await invoke("get_bridge_pairing");
    const root = document.querySelector("footer > span:first-child");
    root.classList.toggle("bridge-waiting", !status.connected);
    root.classList.toggle("bridge-connected", status.connected);
    root.querySelector("b").textContent = status.connected ? t("bridgeConnected") : t("bridgeWaiting");
  } catch (error) {
    console.error(error);
  }
}
refreshBridgeStatus();
setInterval(refreshBridgeStatus, 3000);
