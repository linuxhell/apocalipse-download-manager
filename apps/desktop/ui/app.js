const catalogs = {
  en: {
    downloads: "Downloads",
    media: "Media",
    recordings: "Recordings",
    torrents: "Torrents",
    ed2k: "ED2K",
    tools: "Tools",
    settings: "Settings",
    downloadsDescription: "Manage direct downloads, progress, speed and completed files.",
    mediaDescription: "Videos, audio, recordings and exports detected by Apocalipse.",
    recordingsDescription: "Follow active recordings, stop and save, export or open completed captures.",
    torrentsDescription: "Manage torrents, file selection, peers and previews.",
    ed2kDescription: "Search ED2K/Kad and manage decentralized transfers through the aMule 3 engine.",
    ed2kEngine: "aMule 3.0.1 engine", ed2kChecking: "Checking components…", ed2kSync: "Synchronize aMule", ed2kStart: "Start engine", ed2kConnect: "Connect networks", openAmule: "Open aMule",
    ed2kSources: "SOURCES", connected: "Connected", disconnected: "Disconnected", ed2kHighId: "High ID", ed2kLowId: "Low ID / firewalled",
    ed2kConnection: "Engine connection", ed2kHost: "Host", ed2kPort: "EC port", ed2kPasswordHint: "Enter the External Connections password",
    ed2kSearchTitle: "Search ED2K/Kad", ed2kSearchHint: "Search the decentralized networks and choose a result.", ed2kSearchPlaceholder: "File name…", ed2kGlobal: "Global", search: "Search",
    ed2kVideo: "Video", ed2kAudio: "Audio", ed2kImage: "Image", ed2kDocument: "Document", ed2kProgram: "Program", ed2kArchive: "Archive",
    ed2kSearching: "Searching…", ed2kWaitingResults: "Waiting for network results…", ed2kNoResults: "No results yet.", ed2kDownload: "Download", ed2kTransfers: "Transfers", ed2kTransfersHint: "Real progress reported by the aMule engine.", ed2kNoTransfers: "No ED2K transfers.", ed2kCancel: "Cancel", ed2kPriority: "Priority",
    ed2kReady: "Engine ready", ed2kIncomplete: "Select the official aMule 3.0.1 package in Tools.", ed2kPasswordRequired: "Configure the External Connections password.", ed2kConnectionSaved: "Connection saved.", ed2kSyncing: "Synchronizing with aMule…", ed2kSynced: "aMule synchronized. Transfers will now appear here.", ed2kRestartConfirm: "aMule must restart once to enable secure local synchronization. Active transfers will resume automatically. Restart now?", ed2kSyncStarting: "Synchronization configured. aMule is starting; wait a few seconds.",
    ed2kLinks: "ED2K file, server and server-list links",
    linkDescription: "Transfer files securely between this computer and a remote Apocalipse.",
    matrixPageDescription: "Continuous diagnostics and isolated site corrections with individual rollback.",
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
    saveTo: "Save to",
    fileName: "File name",
    queued: "Queued",
    inspecting: "Inspecting",
    downloading: "Downloading",
    paused: "Paused",
    failed: "Failed",
    pause: "Pause",
    resume: "Resume",
    retry: "Retry",
    openFolder: "Open folder",
    preview: "Preview",
    stopRecording: "Stop and save",
    recordingActive: "Recording",
    matrixPowered: "Powered by Matrix Ultimate v2 AI",
    matrixRollback: "Rollback",
    matrixAnalyze: "Analyze failures",
    matrixDescription: "Matrix continuously monitors local failures and proposes safe rules. No rule executes website code.",
    matrixSummary: "{active} active rules · {proposals} proposals",
    matrixNoProposal: "No new rule is required.",
    matrixReason: "{count} failed download(s); retry with one conservative connection",
    matrixApply: "Apply rule",
    matrixConfirm: "Apply the isolated correction for {host}? You can roll it back afterward.",
    matrixAvailable: "{count} correction(s) available",
    matrixApplied: "Applied corrections",
    matrixRollbackConfirm: "Roll back the correction for {host}? Other corrections will remain active.",
    matrixAnalyzing: "Analyzing…",
    matrixChecking: "Checking download failures…",
    matrixDone: "analysis completed now",
    matrixAnalysisFailed: "Analysis failed",
    matrixRollbackDone: "rollback completed",
    matrixRollbackUnavailable: "Rollback unavailable",
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
    siteRules: "Site rules",
    siteRulesHint: "Versioned fixes that can change site behavior without rebuilding the application",
    manageRules: "Manage rules",
    saveRules: "Validate and save",
    resetRules: "Restore defaults",
    exportRecording: "Export completed recording", outputFormat: "Output format", videoCodec: "Video codec", audioCodec: "Audio codec", export: "Export",
    searchHistory: "Search downloads…", importList: "Import list", advancedOptions: "Advanced options", mirrorUrls: "Mirror URLs (one per line)", priority: "Priority", priorityHigh: "High", priorityNormal: "Normal", priorityLow: "Low", verifyIntegrity: "Verify SHA-256", integrityPrompt: "Optional expected SHA-256 (leave blank to calculate only):", integrityOk: "SHA-256 verified",
    smartAutomation: "Smart automation", adaptiveEfficiency: "Adaptive efficiency", adaptiveEfficiencyHint: "Optimizes queue order and connection use for the current workload.", scheduler: "Download schedule", schedulerHint: "Automatically pauses outside the permitted local time window.", scheduleStart: "Start", scheduleEnd: "End",
  },
  "pt-BR": {
    downloads: "Downloads",
    media: "Mídia",
    recordings: "Gravações",
    torrents: "Torrents",
    ed2k: "ED2K",
    tools: "Ferramentas",
    settings: "Configurações",
    downloadsDescription: "Gerencie downloads diretos, progresso, velocidade e arquivos concluídos.",
    mediaDescription: "Vídeos, áudios, gravações e exportações detectados pelo Apocalipse.",
    recordingsDescription: "Acompanhe gravações ativas, pare e salve, exporte ou abra capturas concluídas.",
    torrentsDescription: "Gerencie torrents, escolha de arquivos, pares e pré-visualizações.",
    ed2kDescription: "Pesquise nas redes ED2K/Kad e gerencie transferências descentralizadas pelo motor aMule 3.",
    ed2kEngine: "Motor aMule 3.0.1", ed2kChecking: "Verificando componentes…", ed2kSync: "Sincronizar aMule", ed2kStart: "Iniciar motor", ed2kConnect: "Conectar redes", openAmule: "Abrir aMule",
    ed2kSources: "FONTES", connected: "Conectado", disconnected: "Desconectado", ed2kHighId: "ID alto", ed2kLowId: "ID baixo / com firewall",
    ed2kConnection: "Conexão com o motor", ed2kHost: "Servidor", ed2kPort: "Porta EC", ed2kPasswordHint: "Digite a senha de Conexões Externas",
    ed2kSearchTitle: "Pesquisar em ED2K/Kad", ed2kSearchHint: "Pesquise nas redes descentralizadas e escolha um resultado.", ed2kSearchPlaceholder: "Nome do arquivo…", ed2kGlobal: "Global", search: "Pesquisar",
    ed2kVideo: "Vídeo", ed2kAudio: "Áudio", ed2kImage: "Imagem", ed2kDocument: "Documento", ed2kProgram: "Programa", ed2kArchive: "Arquivo compactado",
    ed2kSearching: "Pesquisando…", ed2kWaitingResults: "Aguardando resultados da rede…", ed2kNoResults: "Ainda não há resultados.", ed2kDownload: "Baixar", ed2kTransfers: "Transferências", ed2kTransfersHint: "Progresso real informado pelo motor aMule.", ed2kNoTransfers: "Nenhuma transferência ED2K.", ed2kCancel: "Cancelar", ed2kPriority: "Prioridade",
    ed2kReady: "Motor pronto", ed2kIncomplete: "Selecione o pacote oficial aMule 3.0.1 em Ferramentas.", ed2kPasswordRequired: "Configure a senha de Conexões Externas.", ed2kConnectionSaved: "Conexão salva.", ed2kSyncing: "Sincronizando com o aMule…", ed2kSynced: "aMule sincronizado. As transferências agora aparecerão aqui.", ed2kRestartConfirm: "O aMule precisa reiniciar uma vez para ativar a sincronização local segura. As transferências ativas continuarão automaticamente. Reiniciar agora?", ed2kSyncStarting: "Sincronização configurada. O aMule está iniciando; aguarde alguns segundos.",
    ed2kLinks: "Links de arquivo, servidor e lista de servidores ED2K",
    linkDescription: "Transfira arquivos com segurança entre este computador e um Apocalipse remoto.",
    matrixPageDescription: "Diagnóstico contínuo e correções isoladas por site, com reversão individual.",
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
    saveTo: "Salvar em",
    fileName: "Nome do arquivo",
    queued: "Na fila",
    inspecting: "Analisando",
    downloading: "Baixando",
    paused: "Pausado",
    failed: "Falhou",
    pause: "Pausar",
    resume: "Continuar",
    retry: "Tentar novamente",
    openFolder: "Abrir pasta",
    preview: "Pré-visualizar",
    stopRecording: "Parar e salvar",
    recordingActive: "Gravando",
    matrixPowered: "Alimentada por Matrix Ultimate v2 AI",
    matrixRollback: "Reverter",
    matrixAnalyze: "Analisar falhas",
    matrixDescription: "A Matrix monitora continuamente as falhas locais e propõe regras seguras. Nenhuma regra executa código de sites.",
    matrixSummary: "{active} regras ativas · {proposals} propostas",
    matrixNoProposal: "Nenhuma nova regra necessária.",
    matrixReason: "{count} download(s) com falha; tentar novamente com uma conexão conservadora",
    matrixApply: "Aplicar regra",
    matrixConfirm: "Aplicar a correção isolada para {host}? Depois você poderá revertê-la.",
    matrixAvailable: "{count} correção(ões) disponível(is)",
    matrixApplied: "Correções aplicadas",
    matrixRollbackConfirm: "Reverter a correção de {host}? As outras correções continuarão ativas.",
    matrixAnalyzing: "Analisando…",
    matrixChecking: "Verificando falhas de download…",
    matrixDone: "análise concluída agora",
    matrixAnalysisFailed: "Falha na análise",
    matrixRollbackDone: "reversão concluída",
    matrixRollbackUnavailable: "Reversão indisponível",
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
    siteRules: "Regras por site",
    siteRulesHint: "Correções versionadas que alteram o comportamento dos sites sem recompilar o aplicativo",
    manageRules: "Gerenciar regras",
    saveRules: "Validar e salvar",
    resetRules: "Restaurar padrões",
    exportRecording: "Exportar gravação concluída", outputFormat: "Formato de saída", videoCodec: "Codec de vídeo", audioCodec: "Codec de áudio", export: "Exportar",
    searchHistory: "Pesquisar downloads…", importList: "Importar lista", advancedOptions: "Opções avançadas", mirrorUrls: "URLs espelho (uma por linha)", priority: "Prioridade", priorityHigh: "Alta", priorityNormal: "Normal", priorityLow: "Baixa", verifyIntegrity: "Verificar SHA-256", integrityPrompt: "SHA-256 esperado opcional (deixe vazio apenas para calcular):", integrityOk: "SHA-256 verificado",
    smartAutomation: "Automação inteligente", adaptiveEfficiency: "Eficiência adaptativa", adaptiveEfficiencyHint: "Otimiza a ordem da fila e o uso de conexões para a carga atual.", scheduler: "Agendamento de downloads", schedulerHint: "Pausa automaticamente fora do horário local permitido.", scheduleStart: "Início", scheduleEnd: "Fim",
  },
  "zh-CN": {
    downloads: "下载",
    media: "媒体",
    recordings: "录制",
    torrents: "种子",
    ed2k: "ED2K",
    tools: "工具",
    settings: "设置",
    downloadsDescription: "管理直接下载、进度、速度和已完成文件。",
    mediaDescription: "管理 Apocalipse 检测到的视频、音频、录制和导出。",
    recordingsDescription: "查看正在录制的内容、停止并保存、导出或打开已完成的录制。",
    torrentsDescription: "管理种子、文件选择、节点和预览。",
    ed2kDescription: "通过 aMule 3 引擎搜索 ED2K/Kad 网络并管理去中心化传输。",
    ed2kEngine: "aMule 3.0.1 引擎", ed2kChecking: "正在检查组件…", ed2kSync: "同步 aMule", ed2kStart: "启动引擎", ed2kConnect: "连接网络", openAmule: "打开 aMule",
    ed2kSources: "来源", connected: "已连接", disconnected: "未连接", ed2kHighId: "高 ID", ed2kLowId: "低 ID / 防火墙限制",
    ed2kConnection: "引擎连接", ed2kHost: "主机", ed2kPort: "EC 端口", ed2kPasswordHint: "输入外部连接密码",
    ed2kSearchTitle: "搜索 ED2K/Kad", ed2kSearchHint: "搜索去中心化网络并选择结果。", ed2kSearchPlaceholder: "文件名…", ed2kGlobal: "全局", search: "搜索",
    ed2kVideo: "视频", ed2kAudio: "音频", ed2kImage: "图像", ed2kDocument: "文档", ed2kProgram: "程序", ed2kArchive: "压缩包",
    ed2kSearching: "正在搜索…", ed2kWaitingResults: "正在等待网络结果…", ed2kNoResults: "暂无结果。", ed2kDownload: "下载", ed2kTransfers: "传输", ed2kTransfersHint: "由 aMule 引擎报告的真实进度。", ed2kNoTransfers: "没有 ED2K 传输。", ed2kCancel: "取消", ed2kPriority: "优先级",
    ed2kReady: "引擎就绪", ed2kIncomplete: "请在工具中选择官方 aMule 3.0.1 软件包。", ed2kPasswordRequired: "请配置外部连接密码。", ed2kConnectionSaved: "连接已保存。", ed2kSyncing: "正在与 aMule 同步…", ed2kSynced: "aMule 已同步，传输任务现在会显示在这里。", ed2kRestartConfirm: "aMule 需要重启一次以启用安全的本地同步。活动传输将自动继续。现在重启吗？", ed2kSyncStarting: "同步已配置。aMule 正在启动，请稍候。",
    ed2kLinks: "ED2K 文件、服务器和服务器列表链接",
    linkDescription: "在本机与远程 Apocalipse 之间安全传输文件。",
    matrixPageDescription: "持续诊断及可单独回滚的网站修正规则。",
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
    retry: "重试",
    openFolder: "打开文件夹",
    preview: "预览",
    stopRecording: "停止并保存",
    recordingActive: "正在录制",
    matrixPowered: "由 Matrix Ultimate v2 AI 驱动",
    matrixRollback: "回滚",
    matrixAnalyze: "分析故障",
    matrixDescription: "Matrix 会持续监控本地故障并建议安全规则。任何规则都不会执行网站代码。",
    matrixSummary: "{active} 条启用规则 · {proposals} 条建议",
    matrixNoProposal: "无需添加新规则。",
    matrixReason: "{count} 个下载失败；使用一个保守连接重试",
    matrixApply: "应用规则",
    matrixConfirm: "是否为 {host} 应用隔离修复？之后可以回滚。",
    matrixAvailable: "有 {count} 个可用修复",
    matrixApplied: "已应用的修复",
    matrixRollbackConfirm: "是否回滚 {host} 的修复？其他修复将保持启用。",
    matrixAnalyzing: "正在分析…",
    matrixChecking: "正在检查下载故障…",
    matrixDone: "分析刚刚完成",
    matrixAnalysisFailed: "分析失败",
    matrixRollbackDone: "回滚完成",
    matrixRollbackUnavailable: "回滚不可用",
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
    siteRules: "站点规则",
    siteRulesHint: "无需重新编译应用程序即可更改站点行为的版本化修复",
    manageRules: "管理规则",
    saveRules: "验证并保存",
    resetRules: "恢复默认值",
    exportRecording: "导出已完成的录制", outputFormat: "输出格式", videoCodec: "视频编码", audioCodec: "音频编码", export: "导出",
    searchHistory: "搜索下载…", importList: "导入列表", advancedOptions: "高级选项", mirrorUrls: "镜像网址（每行一个）", priority: "优先级", priorityHigh: "高", priorityNormal: "普通", priorityLow: "低", verifyIntegrity: "验证 SHA-256", integrityPrompt: "可选的预期 SHA-256（留空则仅计算）：", integrityOk: "SHA-256 已验证",
    smartAutomation: "智能自动化", adaptiveEfficiency: "自适应效率", adaptiveEfficiencyHint: "根据当前负载优化队列顺序和连接使用。", scheduler: "下载计划", schedulerHint: "在允许的本地时间之外自动暂停。", scheduleStart: "开始", scheduleEnd: "结束",
  },
};

let locale = localStorage.getItem("apocalipse.language") || "en";
const applyTheme = (theme) => {
  const valid = ["void", "inferno", "toxic", "synthwave", "royal", "crimson", "arctic", "obsidian", "monochrome", "midnight", "forest", "graphite", "deepsea", "eclipse", "hazard", "cyberstorm", "ultraviolet", "emeraldgold", "scarletice", "coppernavy", "matrixcode", "solarizednight"];
  document.documentElement.dataset.theme = valid.includes(theme) ? theme : "void";
};
applyTheme(localStorage.getItem("apocalipse.theme") || "void");
let pendingReferer = null;
let pendingDuration = null;
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
const invoke = (command, args = {}) => {
  const bridge = window.__TAURI__?.core?.invoke;
  if (!bridge) throw new Error("Desktop bridge unavailable in preview");
  return bridge(command, args);
};

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
  if (activePage === "ed2k") visible = visible.filter((task) => /^ed2k:/i.test(task.source));
  if (activePage === "media") visible = visible.filter((task) => !/\.recording\.webm$/i.test(`${task.source} ${task.destination}`) && /(?:\.m3u8(?:$|[?#])|youtube\.com|youtu\.be|facebook\.com|fb\.watch|tiktok\.com|instagram\.com)/i.test(`${task.source} ${task.destination}`));
  if (activePage === "recordings") visible = visible.filter((task) => /\.recording\.webm$/i.test(`${task.source} ${task.destination}`));
  if (activePage === "link") visible = visible.filter((task) => /^(?:ftp|sftp):/i.test(task.source));
  if (historyQuery) visible = visible.filter((task) => `${task.source} ${task.destination} ${task.sha256 || ""}`.toLocaleLowerCase().includes(historyQuery));
  if (activeFilter === "completed") return visible.filter((task) => task.state === "completed");
  if (activeFilter === "active") return visible.filter((task) => task.state !== "completed");
  return visible;
}

function renderDownloads() {
  const list = document.querySelector("#download-list");
  const empty = document.querySelector("#empty");
  list.replaceChildren();
  const visible = visibleDownloads();
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
    const info = Object.assign(document.createElement("div"), {
      className: "download-info",
    });
    const name = document.createElement("strong");
    name.textContent = task.destination.split(/[\\/]/).pop();
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
  document.querySelector("#language").value = locale;
  const descriptions = { downloads: "downloadsDescription", media: "mediaDescription", recordings: "recordingsDescription", torrents: "torrentsDescription", ed2k: "ed2kDescription", link: "linkDescription", matrix: "matrixPageDescription" };
  document.querySelector("#page-description").textContent = t(descriptions[activePage] || "downloadsDescription");
  renderDownloads();
  if (activePage === "matrix") refreshMatrix().catch(console.error);
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
const siteRulesDialog = document.querySelector("#site-rules-dialog");
const exportDialog = document.querySelector("#export-dialog");
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
        downloads.push(await invoke("enqueue_download", { url, destinationDirectory, fileName, formatSelection: null, torrentSelection: null, mirrors: null, priority: 0, context: {} }));
      } catch (error) { console.warn("import", url, error); }
    }
    renderDownloads();
  } catch (error) { console.error(error); }
  finally { button.disabled = false; }
};
document.querySelectorAll('nav [data-page]:not([data-page="settings"]):not([data-page="tools"])').forEach((button) => {
  button.onclick = () => {
    activePage = button.dataset.page;
    document.querySelectorAll("nav [data-page]").forEach((item) => item.classList.toggle("active", item === button));
    const heading = button.querySelector("b")?.textContent || t("downloads");
    document.querySelector("header h1").textContent = heading;
    const descriptions = { downloads: "downloadsDescription", media: "mediaDescription", recordings: "recordingsDescription", torrents: "torrentsDescription", ed2k: "ed2kDescription", link: "linkDescription", matrix: "matrixPageDescription" };
    document.querySelector("#page-description").textContent = t(descriptions[activePage] || "downloadsDescription");
    document.querySelector("#apocalipse-link-panel").hidden = activePage !== "link";
    document.querySelector("#matrix-panel").hidden = activePage !== "matrix";
    document.querySelector("#ed2k-panel").hidden = activePage !== "ed2k";
    document.querySelector(".metrics").hidden = ["link", "matrix", "ed2k"].includes(activePage);
    document.querySelector(".panel").hidden = ["link", "matrix", "ed2k"].includes(activePage);
    renderDownloads();
  };
});

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
async function refreshMatrix() {
  const status = await invoke("matrix_analyze");
  document.querySelector("#matrix-summary").textContent = tf("matrixSummary", { active: status.activeRules, proposals: status.proposals.length });
  const alert = document.querySelector("#matrix-alert");
  alert.hidden = !status.proposals.length;
  alert.textContent = status.proposals.length;
  alert.title = tf("matrixAvailable", { count: status.proposals.length });
  const root = document.querySelector("#matrix-proposals");
  root.replaceChildren();
  if (!status.proposals.length) {
    root.append(Object.assign(document.createElement("p"), { textContent: t("matrixNoProposal") }));
  } else {
    for (const proposal of status.proposals) {
      const row = document.createElement("div");
      row.className = "matrix-proposal";
      const info = document.createElement("span");
      info.append(Object.assign(document.createElement("b"), { textContent: proposal.host }), Object.assign(document.createElement("small"), { textContent: tf("matrixReason", { count: proposal.failures }) }));
      const confidence = Object.assign(document.createElement("b"), { textContent: `${proposal.confidence}%` });
      const apply = Object.assign(document.createElement("button"), { type: "button", textContent: t("matrixApply") });
      apply.onclick = async () => {
        if (!window.confirm(tf("matrixConfirm", { host: proposal.host }))) return;
        apply.disabled = true;
        try { await invoke("matrix_apply_rule", { host: proposal.host }); await refreshMatrix(); }
        catch (error) { console.error(error); }
        finally { apply.disabled = false; }
      };
      row.append(info, confidence, apply);
      root.append(row);
    }
  }
  if (status.appliedRules.length) {
    const title = Object.assign(document.createElement("b"), { className: "matrix-section-title", textContent: t("matrixApplied") });
    root.append(title);
    for (const rule of status.appliedRules) {
      const row = document.createElement("div");
      row.className = "matrix-proposal matrix-applied";
      const info = document.createElement("span");
      info.append(
        Object.assign(document.createElement("b"), { textContent: rule.name }),
        Object.assign(document.createElement("small"), { textContent: rule.host }),
      );
      const rollback = Object.assign(document.createElement("button"), { type: "button", textContent: t("matrixRollback") });
      rollback.onclick = async () => {
        if (!window.confirm(tf("matrixRollbackConfirm", { host: rule.host }))) return;
        rollback.disabled = true;
        try { await invoke("matrix_rollback_rule", { id: rule.id }); await refreshMatrix(); }
        catch (error) { console.error(error); }
        finally { rollback.disabled = false; }
      };
      row.append(info, rollback);
      root.append(row);
    }
  }
}
document.querySelector('[data-page="matrix"]').addEventListener("click", () => refreshMatrix().catch(console.error));

let ed2kSearchId = null;
let ed2kRefreshTimer = null;
function ed2kError(error) {
  const value = String(error);
  if (value.includes("ed2k_password_required")) return t("ed2kPasswordRequired");
  if (value.includes("ed2k_sync_starting")) return t("ed2kSyncStarting");
  return value;
}
function renderEd2kResults(results) {
  const root = document.querySelector("#ed2k-search-results");
  root.replaceChildren();
  if (!results.length) {
    root.append(Object.assign(document.createElement("small"), { textContent: t("ed2kNoResults") }));
    return;
  }
  for (const result of results) {
    const row = document.createElement("article");
    row.className = "ed2k-result";
    const info = document.createElement("div");
    info.append(Object.assign(document.createElement("strong"), { textContent: result.name }), Object.assign(document.createElement("small"), { textContent: `${result.sizeMib.toFixed(2)} MiB · ${t("ed2kSources")}: ${result.sources}` }));
    const button = Object.assign(document.createElement("button"), { className: "primary", textContent: t("ed2kDownload") });
    button.onclick = async () => {
      button.disabled = true;
      try { await invoke("ed2k_download_result", { number: result.number }); await refreshEd2kTransfers(); }
      catch (error) { document.querySelector("#ed2k-search-status").textContent = ed2kError(error); }
      finally { button.disabled = false; }
    };
    row.append(info, button); root.append(row);
  }
}
async function refreshEd2kResults() {
  const response = await invoke("ed2k_search_results", { searchId: ed2kSearchId });
  renderEd2kResults(response.results || []);
  document.querySelector("#ed2k-search-status").textContent = response.results?.length ? `${response.results.length} ${t("ed2kSources").toLocaleLowerCase()}` : t("ed2kWaitingResults");
}
function ed2kActionButton(label, action, hash) {
  const button = Object.assign(document.createElement("button"), { textContent: label });
  button.onclick = async () => { button.disabled = true; try { await invoke("control_ed2k_transfer", { action, hash }); await refreshEd2kTransfers(); } catch (error) { console.error(error); } finally { button.disabled = false; } };
  return button;
}
async function refreshEd2kTransfers() {
  const root = document.querySelector("#ed2k-transfer-list");
  const transfers = await invoke("list_ed2k_transfers");
  root.replaceChildren();
  if (!transfers.length) { root.append(Object.assign(document.createElement("small"), { textContent: t("ed2kNoTransfers") })); return; }
  for (const transfer of transfers) {
    const row = document.createElement("article"); row.className = "ed2k-transfer";
    const info = document.createElement("div");
    const progress = document.createElement("span"); progress.className = "ed2k-transfer-progress";
    progress.append(Object.assign(document.createElement("i"), { style: `width:${Math.max(0, Math.min(100, transfer.percent))}%` }));
    info.append(Object.assign(document.createElement("strong"), { textContent: transfer.name }), progress, Object.assign(document.createElement("small"), { textContent: `${transfer.percent.toFixed(1)}% · ${transfer.activeSources}/${transfer.totalSources} ${t("ed2kSources").toLocaleLowerCase()} · ${transfer.speed || transfer.status} · ${t("ed2kPriority")}: ${transfer.priority}` }));
    const actions = document.createElement("span"); actions.className = "ed2k-transfer-actions";
    const paused = /paused|stopped/i.test(transfer.status);
    actions.append(ed2kActionButton(paused ? t("resume") : t("pause"), paused ? "resume" : "pause", transfer.hash), ed2kActionButton(t("ed2kCancel"), "cancel", transfer.hash));
    row.append(info, actions); root.append(row);
  }
}
async function refreshEd2kStatus() {
  const engine = await invoke("get_ed2k_engine_status");
  const complete = engine.helperFound && engine.controllerFound && engine.daemonFound;
  document.querySelector("#ed2k-engine-status").textContent = complete ? `${t("ed2kReady")} · ${engine.version || "aMule 3"}` : t("ed2kIncomplete");
  if (!engine.connected) return;
  const status = await invoke("ed2k_network_status");
  document.querySelector("#ed2k-server-state").textContent = status.ed2kConnected ? `${t("connected")} · ${status.highId ? t("ed2kHighId") : t("ed2kLowId")}` : t("disconnected");
  document.querySelector("#ed2k-kad-state").textContent = status.kadConnected ? `${t("connected")}${status.firewalled ? ` · ${t("ed2kLowId")}` : ""}` : t("disconnected");
  document.querySelector("#ed2k-download-speed").textContent = status.downloadSpeed || "0 B/s";
  document.querySelector("#ed2k-upload-speed").textContent = status.uploadSpeed || "0 B/s";
  document.querySelector("#ed2k-source-count").textContent = status.sources;
}
async function loadEd2kPage() {
  const connection = await invoke("get_ed2k_connection");
  document.querySelector("#ed2k-host").value = connection.host;
  document.querySelector("#ed2k-port").value = connection.port;
  document.querySelector("#ed2k-password").placeholder = connection.passwordConfigured ? "••••••••" : t("ed2kPasswordHint");
  await refreshEd2kStatus().catch((error) => document.querySelector("#ed2k-engine-status").textContent = ed2kError(error));
  await refreshEd2kTransfers().catch(() => {});
  clearInterval(ed2kRefreshTimer);
  ed2kRefreshTimer = setInterval(() => { if (activePage === "ed2k") { refreshEd2kStatus().catch(() => {}); refreshEd2kTransfers().catch(() => {}); } }, 2500);
}
document.querySelector('[data-page="ed2k"]').addEventListener("click", () => loadEd2kPage().catch(console.error));
document.querySelector("#ed2k-sync").onclick = async () => {
  const button = document.querySelector("#ed2k-sync");
  const status = document.querySelector("#ed2k-engine-status");
  button.disabled = true; status.textContent = t("ed2kSyncing");
  try {
    let result;
    try { result = await invoke("synchronize_ed2k_engine", { restartRunning: false }); }
    catch (error) {
      if (!String(error).includes("ed2k_restart_confirmation_required") || !confirm(t("ed2kRestartConfirm"))) throw error;
      result = await invoke("synchronize_ed2k_engine", { restartRunning: true });
    }
    status.textContent = result.connected ? t("ed2kSynced") : t("ed2kSyncStarting");
    await loadEd2kPage();
  } catch (error) { status.textContent = ed2kError(error); }
  finally { button.disabled = false; }
};
document.querySelector("#ed2k-save-connection").onclick = async () => {
  try { await invoke("set_ed2k_connection", { host: document.querySelector("#ed2k-host").value, port: Number(document.querySelector("#ed2k-port").value), password: document.querySelector("#ed2k-password").value }); document.querySelector("#ed2k-password").value = ""; document.querySelector("#ed2k-search-status").textContent = t("ed2kConnectionSaved"); }
  catch (error) { document.querySelector("#ed2k-search-status").textContent = ed2kError(error); }
};
document.querySelector("#ed2k-start").onclick = async () => { try { await invoke("start_ed2k_engine"); setTimeout(() => refreshEd2kStatus().catch(() => {}), 1800); } catch (error) { document.querySelector("#ed2k-engine-status").textContent = ed2kError(error); } };
document.querySelector("#ed2k-connect").onclick = async () => { try { await invoke("connect_ed2k_networks"); await refreshEd2kStatus(); } catch (error) { document.querySelector("#ed2k-engine-status").textContent = ed2kError(error); } };
document.querySelector("#ed2k-refresh").onclick = () => Promise.all([refreshEd2kStatus(), refreshEd2kTransfers()]).catch(console.error);
document.querySelector("#open-amule").onclick = () => invoke("open_amule").catch((error) => document.querySelector("#ed2k-engine-status").textContent = ed2kError(error));
document.querySelector("#ed2k-search-form").onsubmit = async (event) => {
  event.preventDefault(); const button = event.submitter; button.disabled = true;
  const status = document.querySelector("#ed2k-search-status"); status.textContent = t("ed2kSearching");
  try {
    const response = await invoke("ed2k_search", { query: document.querySelector("#ed2k-query").value, searchType: document.querySelector("#ed2k-search-network").value, fileType: document.querySelector("#ed2k-file-type").value });
    ed2kSearchId = response.searchId; status.textContent = t("ed2kWaitingResults");
    setTimeout(() => refreshEd2kResults().catch((error) => status.textContent = ed2kError(error)), 1800);
  } catch (error) { status.textContent = ed2kError(error); } finally { button.disabled = false; }
};
refreshMatrix().catch(console.error);
setInterval(() => refreshMatrix().catch(console.error), 5000);
document.querySelector("#matrix-scan").onclick = async () => {
  const button = document.querySelector("#matrix-scan");
  const summary = document.querySelector("#matrix-summary");
  button.disabled = true;
  button.textContent = t("matrixAnalyzing");
  summary.textContent = t("matrixChecking");
  try {
    await refreshMatrix();
    summary.textContent += ` · ${t("matrixDone")}`;
  } catch (error) {
    summary.textContent = `${t("matrixAnalysisFailed")}: ${error}`;
  } finally {
    button.disabled = false;
    button.textContent = t("matrixAnalyze");
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
  document.querySelector("#torrent-inspection").hidden = true;
  document.querySelector("#torrent-files").replaceChildren();
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
document.querySelector("#language").onchange = (event) => {
  locale = event.target.value;
  localStorage.setItem("apocalipse.language", locale);
  translate();
};
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
document.querySelector('[data-page="settings"]').onclick = async () => {
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
  } catch (error) {
    console.error(error);
  }
};
document
  .querySelectorAll("[data-settings-close]")
  .forEach((button) => (button.onclick = () => {
    applyTheme(localStorage.getItem("apocalipse.theme") || "void");
    settingsDialog.close();
  }));
document.querySelector("#theme").onchange = (event) => applyTheme(event.target.value);
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
      nM3u8dlRe: document.querySelector("#tool-n-m3u8dl-re").value,
      aria2: document.querySelector("#tool-aria2").value,
      ed2k: document.querySelector("#tool-ed2k").value,
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
  event.currentTarget.disabled = true;
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
  finally { event.currentTarget.disabled = false; }
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
document.querySelector("#manage-site-rules").onclick = async () => {
  try {
    document.querySelector("#site-rules-json").value = await invoke("get_site_rules");
    document.querySelector("#site-rules-error").hidden = true;
    siteRulesDialog.showModal();
  } catch (error) { console.error(error); }
};
document.querySelector("#save-site-rules").onclick = async () => {
  const errorBox = document.querySelector("#site-rules-error");
  try {
    document.querySelector("#site-rules-json").value = await invoke("set_site_rules", {
      json: document.querySelector("#site-rules-json").value,
    });
    errorBox.hidden = true;
  } catch (error) {
    errorBox.textContent = String(error);
    errorBox.hidden = false;
  }
};
document.querySelector("#reset-site-rules").onclick = async () => {
  try {
    document.querySelector("#site-rules-json").value = await invoke("reset_site_rules");
    document.querySelector("#site-rules-error").hidden = true;
  } catch (error) { console.error(error); }
};
document.querySelectorAll("[data-site-rules-close]").forEach((button) => {
  button.onclick = () => siteRulesDialog.close();
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
  event.currentTarget.disabled = true;
  try {
    const input = document.querySelector("#media-player");
    const selected = await invoke("pick_executable", { initialPath: input.value });
    if (selected) input.value = selected;
  } catch (error) { console.error(error); }
  finally { event.currentTarget.disabled = false; }
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
  resetMediaInspection();
  const panel = document.querySelector("#media-inspection");
  const select = document.querySelector("#media-format");
  select.replaceChildren();
  option(select, "bestvideo+bestaudio/best", t("bestQuality"));
  for (const format of ["mp3", "m4a", "opus", "flac", "wav"])
    option(select, `audio:${format}`, `${t("audioOnly")} · ${format.toUpperCase()}`);
  try {
    const media = await invoke("inspect_media_formats", { url });
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
    document.querySelector("#media-title").textContent = t("mediaUnavailable");
    document.querySelector("#media-duration").textContent = "";
    const thumbnail = document.querySelector("#media-thumbnail");
    thumbnail.hidden = true;
    thumbnail.removeAttribute("src");
    panel.hidden = false;
  }
}
document.querySelector("#media-format").onchange = (event) => {
  const audio = event.target.value.match(/^audio:(.+)$/);
  if (!audio) return;
  const input = document.querySelector("#file-name");
  const base = input.value.replace(/\.[^.]+$/, "");
  input.value = `${base}.${audio[1]}`;
};
document.querySelector("#analyze").onclick = async () => {
  const url = document.querySelector("#url");
  if (!url.reportValidity()) return;
  const box = document.querySelector("#analysis");
  box.hidden = false;
  box.textContent = "…";
  try {
    const plan = await invoke("inspect_url", { url: url.value });
    const fileName = document.querySelector("#file-name");
    const suggestedFileName = await invoke(
      "suggest_download_name",
      { url: url.value },
    );
    if (plan.primary !== "NativeHttp" || !fileName.value.trim()) {
      fileName.value = suggestedFileName;
    }
    box.textContent = `${plan.primary} · ${plan.reason}`;
    if (plan.primary === "YtDlp") await showMediaInspection(url.value);
    else if (plan.primary === "Aria2Rpc" && (/^magnet:/i.test(url.value) || /\.torrent$/i.test(url.value.split(/[?#]/)[0]))) await showTorrentInspection(url.value);
    else if (plan.primary === "NM3u8DlRe") {
      const panel = document.querySelector("#media-inspection");
      const select = document.querySelector("#media-format");
      select.replaceChildren();
      option(select, "", t("bestQuality"));
      for (const format of ["mp3", "m4a", "opus", "flac", "wav"])
        option(select, `audio:${format}`, `${t("audioOnly")} · ${format.toUpperCase()}`);
      document.querySelector("#media-title").textContent = "HLS";
      document.querySelector("#media-duration").textContent = "";
      document.querySelector("#media-thumbnail").hidden = true;
      panel.hidden = false;
    }
    document.querySelector("#analyze").hidden = true;
    document.querySelector("#enqueue").hidden = false;
  } catch (error) {
    box.textContent = String(error);
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
        formatSelection: document.querySelector("#media-inspection").hidden ? null : document.querySelector("#media-format").value,
        torrentSelection,
        mirrors: document.querySelector("#mirrors").value.split(/\r?\n/).map((value) => value.trim()).filter(Boolean),
        priority: Number(document.querySelector("#priority").value),
        context: {
          referer: pendingReferer,
          knownDuration: pendingDuration,
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
    pendingCookieHeader = request.cookieHeader || null;
    pendingUserAgent = request.userAgent || null;
    pendingRequestMethod = request.requestMethod || null;
    pendingRequestBody = request.requestBody || null;
    pendingRequestContentType = request.requestContentType || null;
    document.querySelector("#url").value = request.url;
    document.querySelector("#file-name").value = request.fileName || "";
    document.querySelector("#analysis").hidden = true;
    document.querySelector("#enqueue").hidden = true;
    document.querySelector("#analyze").hidden = false;
    resetMediaInspection();
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
