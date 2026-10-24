const catalogs = {
  en: {
    archiveExtractor: "Archive extractor (7-Zip / RAR / UnRAR / unar / bsdtar / tar)",
    autoExtract: "Extract automatically after download",
    autoExtractHint: "Shown only for archive files. Loose root files are kept inside a folder named after the archive.",
    browserAssistedArchiveReady: "Archive received from the browser. Choose where to save it and whether to extract it automatically.",
    saveTorrentMetadata: "Save .torrent in data/torrents",
    clearTorrentStore: "Clear .torrents in data/torrents",
    clearTorrentStoreConfirm: "Delete every saved .torrent file in data/torrents that isn't used by an active or queued torrent?",
    torrentStoreCleared: "Removed {count} .torrent file(s) from data/torrents.",
    networkWaiting: "Waiting for network",
    networkWaitingHint: "The connection changed or went offline. This task will resume automatically when a network interface is available.",
    downloads: "Downloads",
    media: "Media",
    recordings: "Recordings",
    torrents: "Torrents",
    tools: "Tools",
    settings: "Settings",
    language: "Language",
    languageDescription: "Choose the language used throughout Apocalipse and in the tray menu.",
    compact: "Compact", normal: "Normal", large: "Large", chooseLanguage: "Choose language", languageHint: "The entire application and tray menu use the selected language.",
    logs: "Logs", logsDescription: "End-to-end diagnostics for extension, shortcuts, interface, bridge and downloads.", exportLog: "Export log", searchLogs: "Search events…", allLevels: "All levels",
    downloadsDescription: "Manage direct downloads, progress, speed and completed files.",
    recordingsDescription: "Follow active recordings, stop and save, export or open completed captures.",
    torrentsDescription: "Manage torrents, file selection, peers and previews.",
    linkDescription: "Transfer files securely between this computer and a remote Apocalipse.",
    aiCorrectionHistory: "Correction history", aiCorrectionsFound: "Corrections found",
    aiDeleteAll: "Delete all", aiNoCorrections: "No corrections found.", aiInputHint: "Ask about Apocalipse…", aiSend: "Send",
    aiCopyName: "Copy name", aiDeleteCorrection: "Delete", aiApplyCorrection: "Apply for testing", aiUndoCorrection: "Undo",
    aiStatusProposed: "Awaiting approval", aiStatusTesting: "Testing", aiStatusSaved: "Saved", aiStatusConfirmed: "Confirmed", aiStatusRejected: "Did not work",
    toolsPageDescription: "Manage the engines used for media, transfers, conversion and preview.",
    settingsDescription: "Configure appearance, integrations, network and application behavior.",
    toolbox: "TOOLBOX", update: "Update", downloadTool: "Download", downloadingTool: "Downloading…", toolDownloaded: "downloaded", xtremeBackend: "Nibiru Xtreme (HTTP/HTTPS, FTP/SFTP, torrent and magnet)", toolUpdated: "updated", toolCurrent: "already current", manualUpdateRequired: "Manual update required", mediaPlayer: "mpv / media player",
    about: "About", aboutDescription: "About the creator.", aboutCreator: "Creator: Juliano - Brazil", aboutPause: "Pause", aboutPlay: "Play", aboutStop: "Stop", aboutVolume: "Volume", facebookRecordingFallback: "The Facebook extractor failed to read this Reel. Update yt-dlp or try capturing its link again. Recording is available as an alternative.",
    addDownload: "Add download",
    downloadSpeed: "DOWNLOAD SPEED",
    uploadSpeed: "UPLOAD SPEED",
    completed: "COMPLETED",
    queue: "IN QUEUE",
    all: "All",
    active: "Active",
    selectAll: "Select all",
    removeSelected: "Remove selected",
    redownloadSelected: "Download again",
    manageList: "MANAGE LIST",
    removeChoice: "What do you want to remove?",
    listOnly: "Clear from list",
    keepFiles: "Keep downloaded and partial files on disk",
    listAndFiles: "Clear list and files",
    deleteFiles: "Permanently delete downloaded and partial files",
    bridgeStatus: "Extension bridge not configured",
    newTask: "NEW TASK",
    sourceUrl: "Source URL",
    cancel: "Cancel",
    analyze: "Analyze",
    torrentMetadataSeeking: "Finding peers and receiving torrent metadata",
    torrentMetadataPeersStalled: "Peers were reached, but none delivered the torrent metadata in time. This may be specific to this swarm or its peers. Try again or use a torrent file if available.",
    torrentMetadataTimeout: "No peers responded with metadata for this torrent/magnet in time. Trackers or DHT may be slow or the swarm may have no active seeds. You can try again, or check the link.",
    torrentMetadataNoPeers: "The metadata attempt reached no peers. Check the BitTorrent listen port, firewall/VPN/network, DHT/tracker responses and the swarm itself. Open the diagnostic ZIP to identify which stage failed.",
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
    webMirror: "HTTP mirror",
    locateFile: "Locate file…",
    locateFileHint: "If you moved the partial file to another folder or drive, point the app at it to resume from there instead of starting over.",
    openFolder: "Open folder", taskDetails: "Details",
    preview: "Preview",
    stopRecording: "Stop and save",
    recordingActive: "Recording",
    linkThisComputer: "This computer",
    linkRemoteControl: "Remote connection",
    linkRemoteId: "Remote IP / host",
    linkRemoteAddressExamples: "Use the same connection flow for this PC (127.0.0.1), a local-network IP or a public Internet IP/host.",
    linkAccessNotice: "Only explicitly shared files, folders and drives are exposed. Each share keeps its read-only or read/write permission.",
    linkConnect: "Connect", linkDisconnect: "Disconnect", linkDisconnected: "Disconnected.",
    convertWithFfmpeg: "Convert using FFmpeg after download to:",
    linkSend: "Send →",
    linkRemoteComputer: "Remote computer",
    linkDownload: "← Download",
    linkDrives: "Drives",
    linkConnectionFailed: "Connection failed",
    linkTransferring: "Transferring…",
    linkSending: "Sending…",
    linkCompleted: "Completed",
    linkTransferFailed: "Transfer failed",
    linkUploadFailed: "Upload failed",
    linkShareNotice: "Share a file, folder or mapped drive here. Windows and Linux SMB shared folders are also discovered automatically.",
    linkRemoteShareNotice: "Only files, folders and drives shared by the other user appear below.",
    linkNoShares: "No shared files, folders or drives yet. Share an item above to make it appear here.",
    linkUseForDownload: "Use for download",
    linkUseForDownloadHint: "This matches your paused/failed download “{name}”. Fill it from here instead of downloading over the internet.",
    linkUseForDownloadCompleted: "“{name}” filled from Apocalipse Link.",
    linkRemoteUsername: "Operating-system username",
    linkRemoteSystemPassword: "System account password",
    linkCredentialsRequired: "Enter the remote IP/host, operating-system username and account password.",
    linkAuthenticating: "Authenticating system account…",
    linkAuthenticationFailed: "System account authentication failed",
    linkLocalSessionReady: "Connected to this Apocalipse. Shared items are available below.",
    linkRemoteSessionReady: "Encrypted TLS connection established. Remote shares are available below.", linkRemoteFirstTrust: "First connection: this Apocalipse TLS certificate was trusted for this address.",
    linkRemoteAuthPlan: "Remote access uses one login: IP/host + operating-system username + account password.",
    linkRemoteAccountFormats: "Windows: use the local/domain/Microsoft account name (for example juliano or MicrosoftAccount\\name@hotmail.com). Linux and macOS: use the local system username (for example juliano). Windows Hello PIN is not a remote password. The system password is never saved.",
    linkRemoteSecurityNotice: "The system password is never saved and is sent only inside the encrypted TLS channel. Only explicitly shared items remain visible, with their read-only or read/write permission.",
    linkShareFile: "Share file", linkShareFolder: "Share folder or drive", linkReadOnly: "Read only", linkReadWrite: "Read and write", linkStopSharing: "Stop sharing",
    linkDelete: "Delete",
    linkDeleteConfirm: "Permanently delete {name}?",
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
    hostRules: "Per-site transfer rules",
    hostRulesHint: "Apply exact hosts or wildcard subdomains and optionally save credentials, connections, speed and User-Agent in one rule. Existing site credentials remain compatible.",
    hostRulePattern: "Host pattern",
    hostRulePatternHint: "*.example.com",
    hostRulePasswordHint: "Leave blank to keep the saved password",
    hostRuleBandwidth: "Speed limit (MB/s)",
    hostRuleUnlimitedHint: "0 or empty = unlimited",
    hostRuleClearPassword: "Remove the saved password for this rule",
    hostRuleAdd: "Add or update rule",
    hostRuleRemove: "Remove",
    hostRuleRemoveConfirm: "Remove the rule for {pattern}?",
    hostRulesEmpty: "No per-site rules saved.",
    hostRulesVaultWarning: "Passwords are kept in the operating system credential vault, not in settings.json.",
    customDns: "Custom DNS",
    customDnsHint: "Resolve native downloads without changing the operating system DNS",
    dnsProvider: "Provider",
    dnsCustom: "Custom",
    dnsServers: "DNS servers",
    dnsScopeHint: "Applied to the native HTTP engine. xtreme uses the system resolver; SOCKS5H continues resolving through the proxy.",
    xtremeRpcTitle: "Nibiru Xtreme RPC",
    xtremeRpcEnabled: "Use Nibiru Xtreme RPC",
    xtremeRpcEnabledHint: "Control accelerated HTTP/HTTPS and FTP transfers through the local Nibiru Xtreme engine.",
    xtremeRpcAutoStart: "Start Nibiru Xtreme automatically when needed",
    xtremeRpcAutoStartHint: "Keeps one local Nibiru Xtreme backend for the current Apocalipse session.",
    xtremeRpcPort: "RPC port",
    xtremeRpcPortHint: "Automatic",
    xtremeRpcStatus: "RPC status",
    xtremeRpcConnected: "Connected",
    xtremeRpcDisconnected: "Disconnected",
    xtremeRpcTesting: "Testing…",
    xtremeRpcTest: "Test RPC connection",
    xtremeRpcRegenerateToken: "Regenerate RPC token",
    xtremeRpcTokenRegenerated: "RPC token regenerated",
    maxTasks: "Maximum simultaneous tasks",
    connections: "Connections per download",
    automatic: "Automatic",
    taskConnections: "Threads for this download",
    rememberSiteThreads: "Check this box to save how many threads this site will use.",
    rememberSiteThreadsHint: "Updates when you change the slider. Uncheck to remove the saved preference.",
    rememberSiteThreadsFor: "Saved for {host}. Slider changes update it; uncheck to remove it.",
    rememberSiteThreadsOffer: "Tick to use this thread count for {host} next time.",
    bandwidthPrompt: "Speed limit in MB/s (0 = unlimited)", verifying: "Verifying",
    taskConnectionsHint: "This choice takes priority over the engine defaults. Use 1 for a single stream.",
    downloadBandwidthLimit: "This download limit", megabytesPerSecond: "MB/s", unlimited: "Unlimited", smartAutomation: "Smart automation", bandwidthAction: "Speed limit", bandwidthPanel: "Bandwidth", adaptiveEfficiency: "Adaptive efficiency", adaptiveEfficiencyHint: "Optimizes queue order and connection use for the current workload.", scheduler: "Download schedule", schedulerHint: "Automatically pauses outside the permitted local time window.", scheduleStart: "Start", scheduleEnd: "End", bandwidthPanelHint: "Set limits without changing the window size.", currentBandwidth: "Current usage", globalBandwidthLimit: "Global download limit",
    defaults: "Default",
    extensionPairing: "Browser extension pairing",
    pairingToken: "Pairing token",
    copy: "Copy",
    regenerate: "Regenerate",
    bridgeConnected: "Extension connected",
    bridgeWaiting: "Waiting for extension",
    bridgeDisconnected: "Extension disconnected",
    recentLocations: "Recent folders",
    clearLocations: "Clear download locations",
    defaultLocation: "Default",
    unavailableLocation: "Unavailable",
    qualityFormat: "Quality and format",
    bestQuality: "Best video + best audio (recommended)",
    audioOnly: "Audio only",
    duration: "Duration",
    mediaUnavailable: "Media details are unavailable; the default format can still be used.",
    externalTools: "Required media and transfer tools",
    toolsHint: "Browse for an existing executable or download the latest compatible version automatically into the portable tools folder. Downloaded paths are applied after you click Save.",
    installed: "Detected",
    missing: "Not found",
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
    archiveExtractor: "Extrator de arquivos (7-Zip / RAR / UnRAR / unar / bsdtar / tar)",
    autoExtract: "Extrair automaticamente após o download",
    autoExtractHint: "Aparece somente para arquivos compactados. Arquivos soltos ficam dentro de uma pasta com o nome do arquivo compactado.",
    browserAssistedArchiveReady: "Arquivo compactado recebido do navegador. Escolha onde salvar e se deseja extrair automaticamente.",
    saveTorrentMetadata: "Salvar .torrent em data/torrents",
    clearTorrentStore: "Limpar .torrents em data/torrents",
    clearTorrentStoreConfirm: "Apagar todos os arquivos .torrent salvos em data/torrents que não estejam em uso por um torrent ativo ou na fila?",
    torrentStoreCleared: "{count} arquivo(s) .torrent removido(s) de data/torrents.",
    networkWaiting: "Aguardando rede",
    networkWaitingHint: "A conexão mudou ou ficou offline. Esta tarefa será retomada automaticamente quando uma interface de rede estiver disponível.",
    downloads: "Downloads",
    media: "Mídia",
    recordings: "Gravações",
    torrents: "Torrents",
    tools: "Ferramentas",
    settings: "Configurações",
    language: "Idioma",
    languageDescription: "Escolha o idioma usado em todo o Apocalipse e no menu da bandeja.",
    compact: "Compacto", normal: "Normal", large: "Grande", chooseLanguage: "Escolher idioma", languageHint: "Todo o aplicativo e o menu da bandeja usam o idioma selecionado.",
    logs: "Logs", logsDescription: "Diagnóstico de ponta a ponta da extensão, atalhos, interface, ponte e downloads.", exportLog: "Exportar log", searchLogs: "Pesquisar eventos…", allLevels: "Todos os níveis",
    downloadsDescription: "Gerencie downloads diretos, progresso, velocidade e arquivos concluídos.",
    recordingsDescription: "Acompanhe gravações ativas, pare e salve, exporte ou abra capturas concluídas.",
    torrentsDescription: "Gerencie torrents, escolha de arquivos, pares e pré-visualizações.",
    linkDescription: "Transfira arquivos com segurança entre este computador e um Apocalipse remoto.",
    aiCorrectionHistory: "Histórico de correções", aiCorrectionsFound: "Correções encontradas",
    aiDeleteAll: "Apagar todas", aiNoCorrections: "Nenhuma correção encontrada.", aiInputHint: "Pergunte sobre o Apocalipse…", aiSend: "Enviar",
    aiCopyName: "Copiar nome", aiDeleteCorrection: "Apagar", aiApplyCorrection: "Aplicar para teste", aiUndoCorrection: "Desfazer",
    aiStatusProposed: "Aguardando aprovação", aiStatusTesting: "Em teste", aiStatusSaved: "Guardada", aiStatusConfirmed: "Confirmada", aiStatusRejected: "Não funcionou",
    toolsPageDescription: "Gerencie os motores usados para mídia, transferências, conversão e pré-visualização.",
    settingsDescription: "Configure aparência, integrações, rede e comportamento do aplicativo.",
    toolbox: "CAIXA DE FERRAMENTAS", update: "Atualizar", downloadTool: "Baixar", downloadingTool: "Baixando…", toolDownloaded: "baixado", xtremeBackend: "Nibiru Xtreme (HTTP/HTTPS, FTP/SFTP, torrent e magnet)", toolUpdated: "atualizado", toolCurrent: "já está atualizado", manualUpdateRequired: "Atualização manual necessária", mediaPlayer: "mpv / reprodutor de mídia",
    about: "Sobre", aboutDescription: "Sobre o criador.", aboutCreator: "Criador: Juliano - Brasil", aboutPause: "Pausar", aboutPlay: "Tocar", aboutStop: "Parar", aboutVolume: "Volume", facebookRecordingFallback: "O extrator do Facebook falhou ao ler este Reel. Atualize o yt-dlp ou tente capturar o link novamente. Gravar está disponível como alternativa.",
    addDownload: "Adicionar download",
    downloadSpeed: "VELOCIDADE DE DOWNLOAD",
    uploadSpeed: "VELOCIDADE DE ENVIO",
    completed: "CONCLUÍDOS",
    queue: "NA FILA",
    all: "Todos",
    active: "Ativos",
    selectAll: "Selecionar todos",
    removeSelected: "Remover selecionados",
    redownloadSelected: "Baixar novamente",
    manageList: "GERENCIAR LISTA",
    removeChoice: "O que você deseja remover?",
    listOnly: "Limpar somente da lista",
    keepFiles: "Manter no disco os arquivos baixados e parciais",
    listAndFiles: "Limpar lista e arquivos",
    deleteFiles: "Excluir permanentemente os arquivos baixados e parciais",
    bridgeStatus: "Ponte da extensão não configurada",
    newTask: "NOVA TAREFA",
    sourceUrl: "URL de origem",
    cancel: "Cancelar",
    analyze: "Analisar",
    torrentMetadataSeeking: "Procurando pares e recebendo metadados do torrent",
    torrentMetadataPeersStalled: "Pares foram alcançados, mas nenhum entregou os metadados do torrent a tempo. Isso pode ser específico deste swarm ou de seus pares. Tente novamente ou use um arquivo .torrent, se disponível.",
    torrentMetadataTimeout: "Nenhum par respondeu com os metadados deste torrent/magnet a tempo. Os trackers ou o DHT podem estar lentos, ou a rede pode não ter seeds ativos no momento. Você pode tentar novamente ou verificar o link.",
    torrentMetadataNoPeers: "Não foi possível obter os metadados deste Magnet: nenhum peer foi alcançado durante a tentativa. Verifique porta BitTorrent ocupada, firewall/VPN/rede, DHT/tracker e a disponibilidade real do swarm. Consulte o diagnóstico para identificar a causa.",
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
    webMirror: "mirror HTTP",
    locateFile: "Localizar arquivo…",
    locateFileHint: "Se você moveu o arquivo parcial para outra pasta ou disco, indique o novo local para continuar de onde parou em vez de começar do zero.",
    openFolder: "Abrir pasta", taskDetails: "Detalhes",
    preview: "Visualizar",
    stopRecording: "Parar e salvar",
    recordingActive: "Gravando",
    linkThisComputer: "Este computador",
    linkRemoteControl: "Conexão remota",
    linkRemoteId: "IP / host remoto",
    linkRemoteAddressExamples: "Use o mesmo fluxo para este PC (127.0.0.1), um IP da rede local ou um IP/host público da Internet.",
    linkAccessNotice: "Somente arquivos, pastas e unidades compartilhados explicitamente ficam expostos. Cada compartilhamento mantém sua permissão de Somente leitura ou Leitura e gravação.",
    linkConnect: "Conectar", linkDisconnect: "Desconectar", linkDisconnected: "Desconectado.",
    convertWithFfmpeg: "Converter usando FFmpeg ao final do download para:",
    linkSend: "Enviar →",
    linkRemoteComputer: "Computador remoto",
    linkDownload: "← Baixar",
    linkDrives: "Unidades",
    linkConnectionFailed: "Falha na conexão",
    linkTransferring: "Transferindo…",
    linkSending: "Enviando…",
    linkCompleted: "Concluído",
    linkTransferFailed: "Falha na transferência",
    linkUploadFailed: "Falha no envio",
    linkShareNotice: "Compartilhe um arquivo, pasta ou unidade por aqui. Pastas compartilhadas pelo Windows ou Linux via SMB também aparecem automaticamente.",
    linkRemoteShareNotice: "Abaixo aparecem somente arquivos, pastas e unidades compartilhados pelo outro usuário.",
    linkNoShares: "Nenhum arquivo, pasta ou unidade foi compartilhado. Compartilhe um item acima para ele aparecer aqui.",
    linkUseForDownload: "Usar para download",
    linkUseForDownloadHint: "Este arquivo corresponde ao seu download pausado/com falha “{name}”. Preencha a partir daqui em vez de baixar pela internet.",
    linkUseForDownloadCompleted: "“{name}” preenchido pelo Apocalipse Link.",
    linkRemoteUsername: "Usuário do sistema operacional",
    linkRemoteSystemPassword: "Senha da conta do sistema",
    linkCredentialsRequired: "Informe o IP/host remoto, o usuário do sistema operacional e a senha da conta.",
    linkAuthenticating: "Autenticando conta do sistema…",
    linkAuthenticationFailed: "Falha na autenticação da conta do sistema",
    linkLocalSessionReady: "Conectado a este Apocalipse. Os compartilhamentos estão disponíveis abaixo.",
    linkRemoteSessionReady: "Conexão TLS criptografada estabelecida. Os compartilhamentos remotos estão disponíveis abaixo.", linkRemoteFirstTrust: "Primeira conexão: o certificado TLS deste Apocalipse foi confiado para este endereço.",
    linkRemoteAuthPlan: "O acesso remoto usa um único login: IP/host + usuário do sistema operacional + senha da conta.",
    linkRemoteAccountFormats: "Windows: use o usuário da conta local, domínio ou Microsoft (por exemplo juliano ou MicrosoftAccount\\nome@hotmail.com). Linux e macOS: use o usuário local do sistema (por exemplo juliano). O PIN do Windows Hello não é uma senha remota. A senha do sistema nunca é salva.",
    linkRemoteSecurityNotice: "A senha do sistema nunca é salva e só é enviada dentro do canal TLS criptografado. Continuam visíveis apenas os itens compartilhados explicitamente, respeitando Somente leitura ou Leitura e gravação.",
    linkShareFile: "Compartilhar arquivo", linkShareFolder: "Compartilhar pasta ou unidade", linkReadOnly: "Somente leitura", linkReadWrite: "Leitura e gravação", linkStopSharing: "Parar de compartilhar",
    linkDelete: "Apagar",
    linkDeleteConfirm: "Apagar permanentemente {name}?",
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
    hostRules: "Regras de transferência por site",
    hostRulesHint: "Aplique hosts exatos ou subdomínios com curinga e, se quiser, salve credenciais, conexões, velocidade e User-Agent na mesma regra. Credenciais antigas continuam compatíveis.",
    hostRulePattern: "Padrão de host",
    hostRulePatternHint: "*.exemplo.com",
    hostRulePasswordHint: "Deixe vazio para manter a senha salva",
    hostRuleBandwidth: "Limite de velocidade (MB/s)",
    hostRuleUnlimitedHint: "0 ou vazio = ilimitado",
    hostRuleClearPassword: "Remover a senha salva desta regra",
    hostRuleAdd: "Adicionar ou atualizar regra",
    hostRuleRemove: "Remover",
    hostRuleRemoveConfirm: "Remover a regra de {pattern}?",
    hostRulesEmpty: "Nenhuma regra por site salva.",
    hostRulesVaultWarning: "As senhas ficam no cofre de credenciais do sistema operacional, não no settings.json.",
    customDns: "DNS personalizado",
    customDnsHint: "Resolver downloads nativos sem alterar o DNS do sistema operacional",
    dnsProvider: "Provedor",
    dnsCustom: "Personalizado",
    dnsServers: "Servidores DNS",
    dnsScopeHint: "Aplicado ao motor HTTP nativo. O xtreme usa a resolução do sistema; o SOCKS5H continua resolvendo pelo proxy.",
    xtremeRpcTitle: "Nibiru Xtreme RPC",
    xtremeRpcEnabled: "Usar Nibiru Xtreme RPC",
    xtremeRpcEnabledHint: "Controla HTTP/HTTPS acelerado e FTP pelo motor Nibiru Xtreme local.",
    xtremeRpcAutoStart: "Iniciar o Nibiru Xtreme automaticamente quando necessário",
    xtremeRpcAutoStartHint: "Mantém um único backend Nibiru Xtreme local durante a sessão atual do Apocalipse.",
    xtremeRpcPort: "Porta RPC",
    xtremeRpcPortHint: "Automática",
    xtremeRpcStatus: "Status RPC",
    xtremeRpcConnected: "Conectado",
    xtremeRpcDisconnected: "Desconectado",
    xtremeRpcTesting: "Testando…",
    xtremeRpcTest: "Testar conexão RPC",
    xtremeRpcRegenerateToken: "Regenerar token RPC",
    xtremeRpcTokenRegenerated: "Token RPC regenerado",
    maxTasks: "Máximo de tarefas simultâneas",
    connections: "Conexões por download",
    automatic: "Automático",
    taskConnections: "Threads para este download",
    rememberSiteThreads: "Marque essa caixa para salvar quantas threads este site usará.",
    rememberSiteThreadsHint: "Atualiza ao mudar o slider. Desmarque para remover a preferência salva.",
    rememberSiteThreadsFor: "Salvo para {host}. Mudar o slider atualiza; desmarcar remove.",
    rememberSiteThreadsOffer: "Marque para usar esta quantidade de threads em {host} da próxima vez.",
    bandwidthPrompt: "Limite de velocidade em MB/s (0 = ilimitado)", verifying: "Verificando",
    taskConnectionsHint: "Este valor prevalece sobre o padrão do motor. Use 1 para uma única conexão.",
    downloadBandwidthLimit: "Limite deste download", megabytesPerSecond: "MB/s", unlimited: "Ilimitado", smartAutomation: "Automação inteligente", bandwidthAction: "Limitar velocidade", bandwidthPanel: "Banda", adaptiveEfficiency: "Eficiência adaptativa", adaptiveEfficiencyHint: "Otimiza a ordem da fila e o uso de conexões para a carga atual.", scheduler: "Agendamento de downloads", schedulerHint: "Pausa automaticamente fora do horário local permitido.", scheduleStart: "Início", scheduleEnd: "Fim", bandwidthPanelHint: "Defina limites sem alterar o tamanho da janela.", currentBandwidth: "Uso atual", globalBandwidthLimit: "Limite global de download",
    defaults: "Padrão",
    extensionPairing: "Conexão com a extensão",
    pairingToken: "Token de pareamento",
    copy: "Copiar",
    regenerate: "Gerar outro",
    bridgeConnected: "Extensão conectada",
    bridgeWaiting: "Aguardando extensão",
    bridgeDisconnected: "Extensão desconectada",
    recentLocations: "Pastas recentes",
    clearLocations: "Limpar caminhos de download",
    defaultLocation: "Padrão",
    unavailableLocation: "Indisponível",
    qualityFormat: "Qualidade e formato",
    bestQuality: "Melhor vídeo + melhor áudio (recomendado)",
    audioOnly: "Somente áudio",
    duration: "Duração",
    mediaUnavailable: "Os detalhes da mídia não estão disponíveis; ainda é possível usar o formato padrão.",
    externalTools: "Ferramentas obrigatórias de mídia e transferência",
    toolsHint: "Procure um executável existente ou baixe automaticamente a versão compatível mais recente para a pasta portátil tools. Os caminhos baixados passam a valer depois de clicar em Salvar.",
    installed: "Detectado",
    missing: "Não encontrado",
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
    archiveExtractor: "压缩文件解压工具（7-Zip / RAR / UnRAR / unar / bsdtar / tar）",
    autoExtract: "下载完成后自动解压",
    autoExtractHint: "仅在压缩文件时显示。根目录中的零散文件会解压到以压缩文件命名的文件夹中。",
    browserAssistedArchiveReady: "已从浏览器接收压缩文件。请选择保存位置以及是否自动解压。",
    saveTorrentMetadata: "将 .torrent 保存到 data/torrents",
    clearTorrentStore: "清空 data/torrents 中的 .torrent 文件",
    clearTorrentStoreConfirm: "是否删除 data/torrents 中未被任何活动或排队种子使用的所有 .torrent 文件？",
    torrentStoreCleared: "已从 data/torrents 删除 {count} 个 .torrent 文件。",
    networkWaiting: "等待网络",
    networkWaitingHint: "网络连接已更改或断开。可用网络接口恢复后，此任务会自动继续。",
    downloads: "下载",
    media: "媒体",
    recordings: "录制",
    torrents: "种子",
    tools: "工具",
    settings: "设置",
    language: "语言",
    languageDescription: "选择整个 Apocalipse 和托盘菜单使用的语言。",
    compact: "紧凑", normal: "正常", large: "大", chooseLanguage: "选择语言", languageHint: "整个应用程序和托盘菜单都使用所选语言。",
    logs: "日志", logsDescription: "扩展、快捷键、界面、桥接和下载的端到端诊断。", exportLog: "导出日志", searchLogs: "搜索事件…", allLevels: "所有级别",
    downloadsDescription: "管理直接下载、进度、速度和已完成文件。",
    recordingsDescription: "查看正在录制的内容、停止并保存、导出或打开已完成的录制。",
    torrentsDescription: "管理种子、文件选择、节点和预览。",
    linkDescription: "在本机与远程 Apocalipse 之间安全传输文件。",
    aiCorrectionHistory: "修正历史", aiCorrectionsFound: "发现的修正",
    aiDeleteAll: "全部删除", aiNoCorrections: "没有发现修正。", aiInputHint: "询问有关 Apocalipse 的问题…", aiSend: "发送",
    aiCopyName: "复制名称", aiDeleteCorrection: "删除", aiApplyCorrection: "应用测试", aiUndoCorrection: "撤销",
    aiStatusProposed: "等待批准", aiStatusTesting: "测试中", aiStatusSaved: "已保存", aiStatusConfirmed: "已确认", aiStatusRejected: "未解决",
    toolsPageDescription: "管理媒体、传输、转换和预览所使用的引擎。",
    settingsDescription: "配置外观、集成、网络和应用行为。",
    toolbox: "工具箱", update: "更新", downloadTool: "下载", downloadingTool: "正在下载…", toolDownloaded: "已下载", xtremeBackend: "Nibiru Xtreme（HTTP/HTTPS、FTP/SFTP、种子和磁力链接）", toolUpdated: "已更新", toolCurrent: "已是最新版本", manualUpdateRequired: "需要手动更新", mediaPlayer: "mpv / 媒体播放器",
    about: "关于", aboutDescription: "关于创作者。", aboutCreator: "创作者：Juliano - 巴西", aboutPause: "暂停", aboutPlay: "播放", aboutStop: "停止", aboutVolume: "音量", facebookRecordingFallback: "Facebook 提取器无法读取此 Reel。请更新 yt-dlp 或重新捕获链接。也可以选择录制。",
    addDownload: "添加下载",
    downloadSpeed: "下载速度",
    uploadSpeed: "上传速度",
    completed: "已完成",
    queue: "队列中",
    all: "全部",
    active: "进行中",
    selectAll: "全选",
    removeSelected: "移除所选项目",
    redownloadSelected: "重新下载",
    manageList: "管理列表",
    removeChoice: "您想移除哪些内容？",
    listOnly: "仅从列表中清除",
    keepFiles: "保留磁盘上的已下载文件和部分文件",
    listAndFiles: "清除列表和文件",
    deleteFiles: "永久删除已下载文件和部分文件",
    bridgeStatus: "扩展桥接尚未配置",
    newTask: "新任务",
    sourceUrl: "来源网址",
    cancel: "取消",
    analyze: "分析",
    torrentMetadataSeeking: "正在查找节点并接收种子元数据",
    torrentMetadataPeersStalled: "已连接到节点，但没有节点及时传送种子元数据。这可能与该种子群或其节点有关。请重试，或使用可用的 .torrent 文件。",
    torrentMetadataTimeout: "没有节点及时返回这个种子/磁力链接的元数据。Tracker 或 DHT 可能响应缓慢，或者该网络目前没有活跃的做种者。您可以重试，或检查链接是否正确。",
    torrentMetadataNoPeers: "本次元数据尝试未连接到任何节点。请检查 BitTorrent 监听端口、防火墙/VPN/网络、DHT/Tracker 响应以及该 swarm 的实际可用性，并查看诊断包确定失败阶段。",
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
    webMirror: "HTTP 镜像",
    locateFile: "定位文件…",
    locateFileHint: "如果您已将部分下载的文件移动到其他文件夹或磁盘，请指定新位置以从原进度继续，而不是重新开始。",
    openFolder: "打开文件夹", taskDetails: "详情",
    preview: "预览",
    stopRecording: "停止并保存",
    recordingActive: "正在录制",
    linkThisComputer: "此电脑",
    linkRemoteControl: "远程连接",
    linkRemoteId: "远程 IP / 主机",
    linkRemoteAddressExamples: "本机 (127.0.0.1)、局域网 IP 或公网 IP/主机都使用同一个连接流程。",
    linkAccessNotice: "只会公开明确共享的文件、文件夹和驱动器。每个共享项都保留只读或读写权限。",
    linkConnect: "连接", linkDisconnect: "断开连接", linkDisconnected: "已断开连接。",
    convertWithFfmpeg: "下载完成后使用 FFmpeg 转换为：",
    linkSend: "发送 →",
    linkRemoteComputer: "远程电脑",
    linkDownload: "← 下载",
    linkDrives: "驱动器",
    linkConnectionFailed: "连接失败",
    linkTransferring: "正在传输…",
    linkSending: "正在发送…",
    linkCompleted: "已完成",
    linkTransferFailed: "传输失败",
    linkUploadFailed: "发送失败",
    linkShareNotice: "可在此共享文件、文件夹或映射驱动器；Windows 和 Linux 的 SMB 共享文件夹也会自动显示。",
    linkRemoteShareNotice: "下方仅显示对方用户共享的文件、文件夹和驱动器。",
    linkNoShares: "尚未共享文件、文件夹或驱动器。请先在上方共享项目。",
    linkUseForDownload: "用于此下载",
    linkUseForDownloadHint: "此文件与您暂停/失败的下载“{name}”匹配。可从这里直接填充，而不必通过互联网下载。",
    linkUseForDownloadCompleted: "“{name}”已通过 Apocalipse Link 填充完成。",
    linkRemoteUsername: "操作系统用户名",
    linkRemoteSystemPassword: "系统账户密码",
    linkCredentialsRequired: "请输入远程 IP/主机、操作系统用户名和账户密码。",
    linkAuthenticating: "正在验证系统账户…",
    linkAuthenticationFailed: "系统账户身份验证失败",
    linkLocalSessionReady: "已连接到本机 Apocalipse。共享项目已显示在下方。",
    linkRemoteSessionReady: "TLS 加密连接已建立。远程共享项目已显示在下方。", linkRemoteFirstTrust: "首次连接：已信任此地址的 Apocalipse TLS 证书。",
    linkRemoteAuthPlan: "远程访问使用一次登录：IP/主机 + 操作系统用户名 + 账户密码。",
    linkRemoteAccountFormats: "Windows：使用本地、域或 Microsoft 账户用户名（例如 juliano 或 MicrosoftAccount\\name@hotmail.com）。Linux 和 macOS：使用本地系统用户名（例如 juliano）。Windows Hello PIN 不是远程密码。系统密码绝不会保存。",
    linkRemoteSecurityNotice: "系统密码永不保存，并且只会通过 TLS 加密通道发送。仍只显示明确共享的项目，并遵守只读或读写权限。",
    linkShareFile: "共享文件", linkShareFolder: "共享文件夹或驱动器", linkReadOnly: "只读", linkReadWrite: "读写", linkStopSharing: "停止共享",
    linkDelete: "删除",
    linkDeleteConfirm: "永久删除 {name}？",
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
    hostRules: "按网站传输规则",
    hostRulesHint: "为精确主机或通配子域创建规则，并可在同一规则中保存凭据、连接数、速度和 User-Agent。现有网站凭据仍然兼容。",
    hostRulePattern: "主机模式",
    hostRulePatternHint: "*.example.com",
    hostRulePasswordHint: "留空以保留已保存的密码",
    hostRuleBandwidth: "速度限制（MB/秒）",
    hostRuleUnlimitedHint: "0 或留空 = 不限速",
    hostRuleClearPassword: "删除此规则保存的密码",
    hostRuleAdd: "添加或更新规则",
    hostRuleRemove: "删除",
    hostRuleRemoveConfirm: "删除 {pattern} 的规则吗？",
    hostRulesEmpty: "尚未保存按网站规则。",
    hostRulesVaultWarning: "密码保存在操作系统凭据保险库中，而不是 settings.json。",
    customDns: "自定义 DNS",
    customDnsHint: "解析原生下载而不更改操作系统 DNS",
    dnsProvider: "提供商",
    dnsCustom: "自定义",
    dnsServers: "DNS 服务器",
    dnsScopeHint: "应用于原生 HTTP 引擎。xtreme 使用系统解析器；SOCKS5H 仍通过代理解析。",
    xtremeRpcTitle: "Nibiru Xtreme RPC",
    xtremeRpcEnabled: "使用 Nibiru Xtreme RPC",
    xtremeRpcEnabledHint: "通过本地 Nibiru Xtreme 引擎控制加速 HTTP/HTTPS 和 FTP 传输。",
    xtremeRpcAutoStart: "需要时自动启动 Nibiru Xtreme",
    xtremeRpcAutoStartHint: "当前 Apocalipse 会话只保留一个本地 Nibiru Xtreme 后端。",
    xtremeRpcPort: "RPC 端口",
    xtremeRpcPortHint: "自动",
    xtremeRpcStatus: "RPC 状态",
    xtremeRpcConnected: "已连接",
    xtremeRpcDisconnected: "未连接",
    xtremeRpcTesting: "测试中…",
    xtremeRpcTest: "测试 RPC 连接",
    xtremeRpcRegenerateToken: "重新生成 RPC 令牌",
    xtremeRpcTokenRegenerated: "RPC 令牌已重新生成",
    maxTasks: "最大同时任务数",
    connections: "每个下载的连接数",
    automatic: "自动",
    taskConnections: "此下载的线程数",
    rememberSiteThreads: "勾选此框，保存此网站将使用的线程数。",
    rememberSiteThreadsHint: "更改滑块即更新。取消勾选即可删除已保存的设置。",
    rememberSiteThreadsFor: "已为 {host} 保存。更改滑块即更新，取消勾选即可删除。",
    rememberSiteThreadsOffer: "勾选后，下次从 {host} 下载将使用此线程数。",
    bandwidthPrompt: "限速（MB/秒，0 = 不限速）", verifying: "校验中",
    taskConnectionsHint: "此设置优先于引擎默认值。选择 1 使用单连接。",
    downloadBandwidthLimit: "此下载的限制", megabytesPerSecond: "MB/秒", unlimited: "不限速", smartAutomation: "智能自动化", bandwidthAction: "限速", bandwidthPanel: "带宽", adaptiveEfficiency: "自适应效率", adaptiveEfficiencyHint: "根据当前负载优化队列顺序和连接使用。", scheduler: "下载计划", schedulerHint: "在允许的本地时间之外自动暂停。", scheduleStart: "开始", scheduleEnd: "结束", bandwidthPanelHint: "无需改变窗口大小即可设置限制。", currentBandwidth: "当前使用量", globalBandwidthLimit: "全局下载限制",
    defaults: "默认",
    extensionPairing: "浏览器扩展配对",
    pairingToken: "配对令牌",
    copy: "复制",
    regenerate: "重新生成",
    bridgeConnected: "扩展已连接",
    bridgeWaiting: "正在等待扩展",
    bridgeDisconnected: "扩展已断开连接",
    recentLocations: "最近的文件夹",
    clearLocations: "清除下载路径",
    defaultLocation: "默认",
    unavailableLocation: "不可用",
    qualityFormat: "质量和格式",
    bestQuality: "最佳视频 + 最佳音频（推荐）",
    audioOnly: "仅音频",
    duration: "时长",
    mediaUnavailable: "媒体详情不可用；仍可使用默认格式。",
    externalTools: "必需的媒体和传输工具",
    toolsHint: "可选择现有可执行文件，或自动将最新兼容版本下载到便携式 tools 文件夹。下载后的路径会在点击“保存”后生效。",
    installed: "已检测",
    missing: "未找到",
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
window.apocalipseCatalogs = catalogs;

Object.assign(catalogs["en"], {"linkQuic": "Use QUIC for receiving shared files (experimental)", "linkQuicHint": "Enable on both computers. Uploads use HTTPS. If UDP is unavailable, receiving falls back to HTTPS.", "linkQuicIps": "Additional local interface IPs (up to 3, separated by commas)", "linkQuicSave": "Save transport options", "linkQuicSaved": "Transport options saved.", "linkQuicUsed": "QUIC transfer: {paths} additional paths; {failed} unavailable.", "linkHttpUsed": "HTTPS transfer", "dictionaryTitle": "Download with a compression dictionary", "dictionaryHint": "For compatible HTTPS servers. Same origin, exact-path dictionary and trusted SHA-256 required. Maximum 256 MiB; existing files are preserved. This does not install an update.", "dictionaryUrl": "Dictionary HTTPS URL", "artifactUrl": "File HTTPS URL", "expectedHash": "Expected file SHA-256", "dictionaryDownload": "Download and verify", "transportWorking": "Working…", "transportSaved": "Saved: ", "moqTitle": "MoQ object capture (experimental)", "moqHint": "Requires a compatible MoQ relay, broadcast and track. Saves complete objects to .admmoq, not a playable video. Maximum 256 MiB / 1 hour.", "moqRelay": "HTTPS relay URL", "moqBroadcast": "Broadcast", "moqTrack": "Track", "moqSeconds": "Duration (seconds)", "moqStart": "Start object capture", "moqStop": "Stop and save complete objects", "moqActive": "Capturing objects…", "moqStopping": "Stopping…"});
Object.assign(catalogs["pt-BR"], {"linkQuic": "Usar QUIC para receber arquivos compartilhados (experimental)", "linkQuicHint": "Ative nos dois computadores. Envios usam HTTPS. Se UDP estiver indisponível, o recebimento volta para HTTPS.", "linkQuicIps": "IPs adicionais das interfaces locais (até 3, separados por vírgulas)", "linkQuicSave": "Salvar opções de transporte", "linkQuicSaved": "Opções de transporte salvas.", "linkQuicUsed": "Transferência QUIC: {paths} caminhos adicionais; {failed} indisponíveis.", "linkHttpUsed": "Transferência HTTPS", "dictionaryTitle": "Download com dicionário de compressão", "dictionaryHint": "Para servidores HTTPS compatíveis. Exige mesma origem, dicionário para o caminho exato e SHA-256 confiável. Máximo de 256 MiB; preserva arquivos existentes. Não instala atualização.", "dictionaryUrl": "URL HTTPS do dicionário", "artifactUrl": "URL HTTPS do arquivo", "expectedHash": "SHA-256 esperado do arquivo", "dictionaryDownload": "Baixar e verificar", "transportWorking": "Processando…", "transportSaved": "Salvo: ", "moqTitle": "Captura de objetos MoQ (experimental)", "moqHint": "Exige relay MoQ compatível, transmissão e faixa. Salva objetos completos em .admmoq; ainda não gera vídeo reproduzível. Máximo de 256 MiB / 1 hora.", "moqRelay": "URL HTTPS do relay", "moqBroadcast": "Transmissão", "moqTrack": "Faixa", "moqSeconds": "Duração (segundos)", "moqStart": "Iniciar captura de objetos", "moqStop": "Parar e salvar objetos completos", "moqActive": "Capturando objetos…", "moqStopping": "Parando…"});
Object.assign(catalogs["zh-CN"], {"linkQuic": "使用 QUIC 接收共享文件（实验性）", "linkQuicHint": "请在两台电脑上启用。上传使用 HTTPS。UDP 不可用时接收将回退到 HTTPS。", "linkQuicIps": "额外本地接口 IP（最多 3 个，以逗号分隔）", "linkQuicSave": "保存传输选项", "linkQuicSaved": "已保存传输选项。", "linkQuicUsed": "QUIC 传输：{paths} 个额外路径；{failed} 个不可用。", "linkHttpUsed": "HTTPS 传输", "dictionaryTitle": "使用压缩字典下载", "dictionaryHint": "需要兼容的 HTTPS 服务器、同源精确路径字典以及可信的 SHA-256。上限 256 MiB，保留已有文件。此操作不安装更新。", "dictionaryUrl": "字典 HTTPS 网址", "artifactUrl": "文件 HTTPS 网址", "expectedHash": "预期文件 SHA-256", "dictionaryDownload": "下载并验证", "transportWorking": "处理中…", "transportSaved": "已保存：", "moqTitle": "MoQ 对象捕获（实验性）", "moqHint": "需要兼容的 MoQ 中继、广播和轨道。完整对象保存为 .admmoq，尚不可作为视频播放。上限 256 MiB / 1 小时。", "moqRelay": "HTTPS 中继网址", "moqBroadcast": "广播", "moqTrack": "轨道", "moqSeconds": "时长（秒）", "moqStart": "开始捕获对象", "moqStop": "停止并保存完整对象", "moqActive": "正在捕获对象…", "moqStopping": "正在停止…"});

Object.assign(catalogs.en, {originalAudio: "Original audio"});
Object.assign(catalogs["pt-BR"], {originalAudio: "Áudio original"});
Object.assign(catalogs["zh-CN"], {originalAudio: "原始音频"});
Object.assign(catalogs["en"], {"windowMinimize": "Minimize", "windowMaximize": "Maximize / restore", "windowClose": "Close", "taskDestination": "Destination", "taskEngine": "Engine", "taskEta": "Remaining", "taskConnectionsLabel": "Connections"});
Object.assign(catalogs["pt-BR"], {"windowMinimize": "Minimizar", "windowMaximize": "Maximizar / restaurar", "windowClose": "Fechar", "taskDestination": "Destino", "taskEngine": "Motor", "taskEta": "Restante", "taskConnectionsLabel": "Conexões"});
Object.assign(catalogs["zh-CN"], {"windowMinimize": "最小化", "windowMaximize": "最大化 / 还原", "windowClose": "关闭", "taskDestination": "位置", "taskEngine": "引擎", "taskEta": "剩余时间", "taskConnectionsLabel": "连接"});
Object.assign(catalogs.en, {darkMode: "Dark mode", darkModeHint: "Near-black surfaces with light text. Progress colours stay the same.", donate: "Donate", settingsGeneral: "General", settingsNetwork: "Network and proxy", settingsExtension: "Browser extension", settingsLink: "Apocalipse Link"});
Object.assign(catalogs["pt-BR"], {darkMode: "Modo escuro", darkModeHint: "Superfícies quase pretas com texto claro. As cores do progresso continuam iguais.", donate: "Doação", settingsGeneral: "Geral", settingsNetwork: "Rede e proxy", settingsExtension: "Extensão do navegador", settingsLink: "Apocalipse Link"});
Object.assign(catalogs["zh-CN"], {darkMode: "深色模式", darkModeHint: "近黑色背景与浅色文字，进度条颜色不变。", donate: "捐赠", settingsGeneral: "常规", settingsNetwork: "网络与代理", settingsExtension: "浏览器扩展", settingsLink: "Apocalipse Link"});
for (const catalog of Object.values(catalogs)) catalog.ed2kKad = "Apocalipse ED2K/Kad";
let locale = localStorage.getItem("apocalipse.language") || "en";
let pendingDiagnosticTrace = null;
let pendingReferer = null;
let pendingDuration = null;
let pendingIsLive = false;
let pendingTitle = null;
let pendingThumbnail = null;
let pendingAudioUrl = null;
let pendingMediaKind = null;
let pendingExpectedSize = null;
let pendingCookieHeader = null;
let pendingUserAgent = null;
let pendingRequestMethod = null;
let pendingRequestBody = null;
let pendingRequestContentType = null;
let pendingBrowserAssistedPath = null;
let pendingTorrentMetadataPath = null;
let taskConnectionsManuallyChanged = false;
let analysisGeneration = 0;
let downloads = [];
const downloadListState = createTaskListState();
let activeFilter = "all";
let activePage = "downloads";
let overallSpeed = 0;
let overallUploadSpeed = 0;
let lastClipboardLink = "";
let clipboardMonitorPrimed = false;
const busyIds = new Set();
const selectedIds = new Set();
const speedSamples = new Map();
const SPEED_EWMA_SECONDS = 2.0;
const schedulerPaused = new Set();
let selectionPointerActive = false;
let historyQuery = "";
for (const [language, entries] of Object.entries(window.AdmConversionCatalogs || {})) Object.assign(catalogs[language], entries);
const t = (key) => catalogs[locale]?.[key] || catalogs.en[key] || key;
const tf = (key, values) => Object.entries(values).reduce((text, [name, value]) => text.replaceAll(`{${name}}`, value), t(key));
Object.assign(catalogs.en, {"stepLink": "Link", "stepFound": "Media found", "stepSave": "Save", "conflictInUse": "Some files are in use by another program: {files}. Close the program that is using them, or choose Rename.", "conflictProtected": "This location contains the running Apocalipse. Replacing or deleting it would break the app, so only Rename is offered.", "conflictFileEyebrow": "Existing file", "conflictFileTitle": "This file already exists", "conflictFileCopy": "A file with this name is already in {folder}. What do you want to do?", "conflictOnDisk": "In the folder", "conflictNew": "New download", "conflictExtractEyebrow": "Extraction", "conflictExtractTitle": "Files already exist at the destination", "conflictExtractCopy": "Extracting {archive} into {folder}: {count} files already exist.", "conflictMore": "+ {n} more files", "conflictRename": "Rename", "conflictReplace": "Replace", "conflictSkip": "Skip", "conflictRenameFileHint": "Keep both. The new file is saved as “{name}”", "conflictRenameExtractHint": "Extract into a new folder and keep the existing one", "conflictReplaceFileHint": "Delete the existing file and download the new one in its place", "conflictReplaceExtractHint": "Extract into the existing folder, overwriting only the files in conflict", "conflictSkipHint": "Do not download this file", "conflictDelete": "Delete the old folder and extract", "conflictDeleteHint": "Erase the current contents of the destination before extracting", "conflictRemember": "Use this choice for the next conflicts too", "conflictContinue": "Continue", "conflictConfirmDelete": "Confirm deletion", "conflictOriginalUntouched": "The original archive is not changed"});
Object.assign(catalogs["pt-BR"], {"stepLink": "Link", "stepFound": "Mídia encontrada", "stepSave": "Salvar", "conflictInUse": "Alguns arquivos estão em uso por outro programa: {files}. Feche o programa que os está usando ou escolha Renomear.", "conflictProtected": "Este local contém o Apocalipse em execução. Substituir ou excluir quebraria o aplicativo, então só Renomear está disponível.", "conflictFileEyebrow": "Arquivo existente", "conflictFileTitle": "Este arquivo já existe", "conflictFileCopy": "Já há um arquivo com este nome em {folder}. O que fazer?", "conflictOnDisk": "Na pasta", "conflictNew": "Novo download", "conflictExtractEyebrow": "Extração", "conflictExtractTitle": "Já existem arquivos no destino", "conflictExtractCopy": "Ao extrair {archive} em {folder}, {count} arquivos já existem.", "conflictMore": "+ {n} outros arquivos", "conflictRename": "Renomear", "conflictReplace": "Substituir", "conflictSkip": "Pular", "conflictRenameFileHint": "Manter os dois. O novo será salvo como “{name}”", "conflictRenameExtractHint": "Extrair para uma pasta nova e manter a existente", "conflictReplaceFileHint": "Excluir o arquivo existente e baixar o novo no lugar", "conflictReplaceExtractHint": "Extrair na pasta existente, sobrescrevendo só os arquivos em conflito", "conflictSkipHint": "Não baixar este arquivo", "conflictDelete": "Excluir a pasta antiga e extrair", "conflictDeleteHint": "Apagar o conteúdo atual do destino antes de extrair", "conflictRemember": "Usar esta escolha nos próximos conflitos", "conflictContinue": "Continuar", "conflictConfirmDelete": "Confirmar exclusão", "conflictOriginalUntouched": "O arquivo compactado original não é alterado"});
Object.assign(catalogs["zh-CN"], {"stepLink": "链接", "stepFound": "找到的媒体", "stepSave": "保存", "conflictInUse": "部分文件正被其他程序占用：{files}。请关闭占用它们的程序，或选择重命名。", "conflictProtected": "此位置包含正在运行的 Apocalipse。替换或删除会破坏应用，因此只能选择重命名。", "conflictFileEyebrow": "文件已存在", "conflictFileTitle": "此文件已存在", "conflictFileCopy": "{folder} 中已有同名文件。要怎么处理？", "conflictOnDisk": "文件夹中", "conflictNew": "新下载", "conflictExtractEyebrow": "解压", "conflictExtractTitle": "目标位置已有文件", "conflictExtractCopy": "将 {archive} 解压到 {folder} 时，有 {count} 个文件已存在。", "conflictMore": "另有 {n} 个文件", "conflictRename": "重命名", "conflictReplace": "替换", "conflictSkip": "跳过", "conflictRenameFileHint": "两个都保留。新文件将保存为“{name}”", "conflictRenameExtractHint": "解压到新文件夹并保留现有文件夹", "conflictReplaceFileHint": "删除现有文件，并用新下载的文件替代", "conflictReplaceExtractHint": "解压到现有文件夹，仅覆盖冲突的文件", "conflictSkipHint": "不下载此文件", "conflictDelete": "删除旧文件夹并解压", "conflictDeleteHint": "解压前清空目标位置的现有内容", "conflictRemember": "后续冲突也使用此选择", "conflictContinue": "继续", "conflictConfirmDelete": "确认删除", "conflictOriginalUntouched": "原压缩文件不会被修改"});
const descriptions = { downloads: "downloadsDescription", recordings: "recordingsDescription", torrents: "torrentsDescription", link: "linkDescription", logs: "logsDescription", language: "languageDescription", about: "aboutDescription", settings: "settingsDescription", tools: "toolsPageDescription" };
let lastUiInteractionTrace = null;
const freshUiTrace = () => {
  const now = performance.now();
  if (lastUiInteractionTrace && now - lastUiInteractionTrace.at < 2000) return lastUiInteractionTrace.id;
  return crypto.randomUUID();
};
// Use the raw bridge so a failed diagnostic write never recursively reports itself.
function reportUiError(component, operation, error, level = "ERROR") {
  const traceId = freshUiTrace();
  console[level === "WARN" ? "warn" : "error"](`[${component}:${operation}]`, error);
  const bridge = window.__TAURI__?.core?.invoke;
  if (!bridge) return Promise.resolve();
  const detail = { component, operation, traceId, level, errorName: String(error?.name || "Error").slice(0, 80) };
  const message = String(error?.message || error || "unknown_error").slice(0, 2000);
  return Promise.allSettled([
    Promise.resolve().then(() => bridge("record_diagnostics_ui", { event: "caught_error", detail })),
    Promise.resolve().then(() => bridge("record_ui_diagnostic", { level, event: "error_reported",
      detail: `trace=${traceId} component=${component} operation=${operation} error=${message}` })),
  ]).then(() => {});
}
const recordStructuredUi = (bridge, event, detail = {}) =>
  bridge("record_diagnostics_ui", { event, detail }).catch(() => {});
document.addEventListener("click", (event) => {
  const control = event.target?.closest?.("button,[role='button'],a,input[type='button'],input[type='submit']");
  if (!control) return;
  const bridge = window.__TAURI__?.core?.invoke;
  if (!bridge) return;
  const traceId = crypto.randomUUID();
  lastUiInteractionTrace = { id: traceId, at: performance.now() };
  const activePage = document.querySelector(".nav-item.active")?.dataset?.page || null;
  recordStructuredUi(bridge, "control_clicked", {
    traceId,
    level: "INFO",
    window: "main",
    page: activePage,
    controlTag: control.tagName?.toLowerCase() || "unknown",
    controlType: control.getAttribute?.("type") || control.getAttribute?.("role") || "default",
    controlId: control.id || null,
    action: control.dataset?.action || control.dataset?.toolUpdate || control.dataset?.page || null,
  });
}, true);
const invoke = (command, args = {}) => {
  const bridge = window.__TAURI__?.core?.invoke;
  if (!bridge) throw new Error("Desktop bridge unavailable in preview");
  const started = performance.now();
  const quiet = new Set(["list_downloads", "read_general_log", "get_bridge_pairing", "read_clipboard_link", "take_bridge_download", "diagnostics_status"]);
  const traceId = freshUiTrace();
  const taskId = typeof args?.id === "string" ? args.id : null;
  const detailBase = {
    traceId,
    taskId,
    window: "main",
    command,
    argKeys: Object.keys(args || {}).sort(),
  };
  if (!quiet.has(command) && command !== "record_ui_diagnostic" && command !== "record_diagnostics_ui") {
    recordStructuredUi(bridge, "command_started", { ...detailBase, level: "DEBUG" });
  }
  return bridge(command, args).then((result) => {
    if (!quiet.has(command) && command !== "record_ui_diagnostic" && command !== "record_diagnostics_ui") {
      recordStructuredUi(bridge, "command_completed", {
        ...detailBase,
        level: "DEBUG",
        durationMs: Math.round(performance.now() - started),
        resultType: result == null ? "null" : Array.isArray(result) ? "array" : typeof result,
      });
    }
    return result;
  }).catch((error) => {
    if (!quiet.has(command) && command !== "record_ui_diagnostic" && command !== "record_diagnostics_ui") {
      recordStructuredUi(bridge, "command_failed", {
        ...detailBase,
        level: "ERROR",
        durationMs: Math.round(performance.now() - started),
        errorName: String(error?.name || "command_error"),
      });
    }
    throw error;
  });
};
invoke("get_app_version").then((version) => {
  document.querySelector("#app-version").textContent = `v${version}`;
}).catch(() => {});
window.addEventListener("error", event => { void reportUiError("main", "javascript_error", event.error || event.message); });
window.addEventListener("unhandledrejection", event => { void reportUiError("main", "unhandled_rejection", event.reason); });

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
    // When the backend has an engine-reported byte/s value, display that
    // value directly. Do not apply a second UI EWMA: xtreme, yt-dlp and
    // N_m3u8DL-RE already report their own live throughput. Byte-delta EWMA
    // remains only as a fallback for engines that cannot provide byte/s.
    const hasEngineSpeed = active
      && task.download_speed !== null
      && task.download_speed !== undefined
      && Number.isFinite(Number(task.download_speed));
    const engineSpeed = hasEngineSpeed ? Math.max(0, Number(task.download_speed)) : 0;
    let speed = active ? previous?.speed || 0 : 0;
    if (!hasEngineSpeed && previous && active) {
      const elapsed = Math.max(0.001, (now - previous.at) / 1000);
      const delta = Math.max(0, task.received - previous.bytes);
      if (delta > 0) {
        const instantaneous = delta / elapsed;
        const alpha = 1 - Math.exp(-elapsed / SPEED_EWMA_SECONDS);
        speed = previous.speed
          ? instantaneous * alpha + previous.speed * (1 - alpha)
          : instantaneous;
      } else if (now - changedAt >= 1500) {
        speed = 0;
      }
    }
    const displaySpeed = hasEngineSpeed ? engineSpeed : speed;
    speedSamples.set(task.id, {
      at: now,
      bytes: task.received,
      speed: displaySpeed,
      changedAt,
    });
    if (active) {
      overallSpeed += displaySpeed;
      overallUploadSpeed += Number(task.upload_speed) || 0;
    }
  }
}

const isTorrent = (task) => /^(?:magnet:)|\.torrent(?:$|[?#])/i.test(task.source);

function visibleDownloads() {
  const isRecording = (task) => /\.recording(?: \(\d+\))?\.webm$/i.test(`${task.source} ${task.destination}`);
  let visible = activePage === "recordings"
    ? downloads.filter(isRecording)
    : activePage === "torrents"
      ? downloads.filter(isTorrent)
      : downloads.filter((task) => !isRecording(task) && !isTorrent(task));
  if (historyQuery) visible = visible.filter((task) => `${task.source} ${task.destination} ${task.sha256 || ""}`.toLocaleLowerCase().includes(historyQuery));
  if (activeFilter === "completed") return visible.filter((task) => task.state === "completed");
  if (activeFilter === "active") return visible.filter((task) => task.state !== "completed");
  return visible;
}

const thumbnailDataCache = new Map();
const thumbnailPending = new Map();
const thumbnailRetryAfter = new Map();

async function resolveCachedThumbnail(url) {
  if (!url) return null;
  if (/^data:image\//i.test(url)) return url;
  if (thumbnailDataCache.has(url)) return thumbnailDataCache.get(url);
  if ((thumbnailRetryAfter.get(url) || 0) > Date.now()) return null;
  if (thumbnailPending.has(url)) return thumbnailPending.get(url);

  const pending = invoke("resolve_thumbnail", { url })
    .then((resolved) => {
      if (resolved) {
        thumbnailDataCache.set(url, resolved);
        thumbnailRetryAfter.delete(url);
        return resolved;
      }
      thumbnailRetryAfter.set(url, Date.now() + 60_000);
      return null;
    })
    .catch((error) => {
      reportUiError("main", "thumbnail-cache", error, "WARN");
      thumbnailRetryAfter.set(url, Date.now() + 60_000);
      return null;
    })
    .finally(() => thumbnailPending.delete(url));
  thumbnailPending.set(url, pending);
  return pending;
}

// yt-dlp's "thumbnail" for YouTube is often the maxres image, which does not
// exist for every video (404 -> broken image). hqdefault always exists.
function youtubeThumbnailFallback(source) {
  let parsed;
  try { parsed = new URL(String(source || "")); } catch { return null; }
  const host = parsed.hostname.replace(/^www\.|^m\./, "");
  let id = null;
  if (host === "youtu.be") id = parsed.pathname.split("/")[1];
  else if (host === "youtube.com" || host === "music.youtube.com") {
    id = parsed.searchParams.get("v") || parsed.pathname.match(/^\/(?:shorts|embed|live)\/([^/?#]+)/)?.[1];
  }
  return id && /^[\w-]{6,20}$/.test(id) ? `https://i.ytimg.com/vi/${id}/hqdefault.jpg` : null;
}

function loadPreviewThumbnail(image, url, fallbackUrl = null) {
  const sourceKey = `${Date.now()}:${Math.random()}`;
  image.dataset.thumbnailSource = sourceKey;
  image.hidden = true;
  image.onload = null;
  image.onerror = null;
  image.removeAttribute("src");
  const candidates = [...new Set([url, fallbackUrl].filter(Boolean))];
  const next = async () => {
    while (candidates.length && image.dataset.thumbnailSource === sourceKey) {
      const candidate = candidates.shift();
      const resolved = await resolveCachedThumbnail(candidate);
      if (image.dataset.thumbnailSource !== sourceKey) return;
      if (!resolved) continue;
      image.onload = () => { if (image.dataset.thumbnailSource === sourceKey) image.hidden = false; };
      image.onerror = () => {
        if (image.dataset.thumbnailSource !== sourceKey) return;
        image.hidden = true;
        thumbnailDataCache.delete(candidate);
        thumbnailRetryAfter.set(candidate, Date.now() + 60_000);
        image.removeAttribute("src");
        void next();
      };
      image.src = resolved;
      return;
    }
  };
  void next();
}

let lastDownloadRenderSignature = "";

function renderDownloads(force = false) {
  const list = document.querySelector("#download-list");
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
  const previousScrollTop = list.scrollTop;
  const expandedTaskIds = new Set([...list.querySelectorAll(".task-details[open]")].map(node => node.closest(".download-row").dataset.taskId));
  list.replaceChildren();
  list.hidden = visible.length === 0;
  for (const task of visible) {
    const row = document.createElement("article");
    row.className = "download-row";
    row.dataset.taskId = task.id;
    row.dataset.state = stateKey(task.state);
    const reorderable = ["queued", "paused"].includes(stateKey(task.state))
      && activeFilter === "all"
      && !historyQuery;
    row.draggable = reorderable;
    row.classList.toggle("reorderable", reorderable);
    row.addEventListener("dragstart", (event) => {
      if (!reorderable || event.target.closest("button,input,select,a")) {
        event.preventDefault();
        return;
      }
      selectionPointerActive = true;
      row.classList.add("dragging");
      event.dataTransfer.effectAllowed = "move";
      event.dataTransfer.setData("text/plain", task.id);
    });
    row.addEventListener("dragend", () => {
      row.classList.remove("dragging");
      document.querySelectorAll(".download-row.drag-over").forEach((item) => item.classList.remove("drag-over"));
      selectionPointerActive = false;
    });
    row.addEventListener("dragover", (event) => {
      if (!reorderable) return;
      const sourceId = event.dataTransfer.getData("text/plain");
      if (!sourceId || sourceId === task.id) return;
      event.preventDefault();
      event.dataTransfer.dropEffect = "move";
      row.classList.add("drag-over");
    });
    row.addEventListener("dragleave", () => row.classList.remove("drag-over"));
    row.addEventListener("drop", async (event) => {
      event.preventDefault();
      row.classList.remove("drag-over");
      const sourceId = event.dataTransfer.getData("text/plain");
      if (!sourceId || sourceId === task.id) return;
      const movable = downloads.filter((item) => ["queued", "paused"].includes(stateKey(item.state)));
      const ids = movable.map((item) => item.id);
      const from = ids.indexOf(sourceId);
      const target = ids.indexOf(task.id);
      if (from < 0 || target < 0) return;
      const [moved] = ids.splice(from, 1);
      ids.splice(target, 0, moved);
      const byId = new Map(downloads.map((item) => [item.id, item]));
      const reordered = ids.map((id) => byId.get(id)).filter(Boolean);
      let cursor = 0;
      downloads = downloads.map((item) => ["queued", "paused"].includes(stateKey(item.state)) ? reordered[cursor++] : item);
      renderDownloads(true);
      try {
        await invoke("reorder_downloads", { ids });
        downloadListState.invalidate();
        await refreshDownloads();
      } catch (error) {
        reportUiError("main", "renderDownloads", error);
        await refreshDownloads();
      } finally {
        selectionPointerActive = false;
      }
    });
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
    const requestedThumbnail = task.thumbnail || youtubeThumbnailFallback(task.source);
    row.classList.toggle("has-thumbnail", Boolean(requestedThumbnail));
    if (requestedThumbnail) {
      const thumbnail = document.createElement("img");
      thumbnail.className = "download-thumbnail";
      thumbnail.alt = "";
      icon.replaceChildren(thumbnail);
      icon.classList.add("has-thumbnail");
      loadPreviewThumbnail(thumbnail, task.thumbnail, youtubeThumbnailFallback(task.source) || youtubeThumbnailFallback(task.referer));
    }
    const info = Object.assign(document.createElement("div"), {
      className: "download-info",
    });
    const name = document.createElement("strong");
    name.textContent = task.display_title || task.destination.split(/[\\/]/).pop();
    name.title = task.display_title || task.destination.split(/[\\/]/).pop();
    const source = document.createElement("small");
    source.textContent = task.source;
    source.title = task.source;
    const titleLine = document.createElement("div");
    titleLine.className = "task-title-line";
    titleLine.append(name);
    info.append(titleLine);
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
    const uploadSpeed = task.state === "downloading" ? Number(task.upload_speed) || 0 : 0;
    const progressText = hasReportedPercent && !task.total
      ? `${percent.toFixed(1)}%`
      : task.total
      ? `${formatBytes(task.received)} / ${formatBytes(task.total)} · ${percent.toFixed(1)}%`
      : formatBytes(task.received);
    const webSeedStats = task.torrent_web_seeds ? ` · +${task.torrent_web_seeds} ${t("webMirror")}` : "";
    const torrentStats = task.torrent_seeders !== null && task.torrent_seeders !== undefined
      ? ` · S:${task.torrent_seeders} L:${task.torrent_leechers || 0}${task.torrent_eta ? ` · ETA ${task.torrent_eta}` : ""}${webSeedStats}` : "";
    details.textContent = `${progressText}${torrentStats}`;
    details.title = details.textContent;
    const rates = document.createElement("span");
    rates.className = "task-transfer-rates";
    if (task.state === "downloading") {
      const down = document.createElement("span");
      down.textContent = `↓ ${formatBytes(speed)}/s`;
      rates.append(down);
      if (isTorrent(task) || uploadSpeed > 0) {
        const up = document.createElement("span");
        up.textContent = `↑ ${formatBytes(uploadSpeed)}/s`;
        rates.append(up);
      }
    }
    progress.append(bar);
    info.append(progress, details, rates);
    const resumeCapability = document.createElement("strong");
    resumeCapability.className = "resume-capability";
    const resumeValue = task.resume_supported === true
      ? t("resumeYes")
      : task.resume_supported === false
        ? t("resumeNo")
        : t("resumeChecking");
    resumeCapability.textContent = `${t("resumeCapability")} ${resumeValue}`;
    resumeCapability.dataset.supported = task.resume_supported === true ? "true" : task.resume_supported === false ? "false" : "unknown";

    const failureMessage = typeof task.state === "object" ? task.state.failed?.message || "" : "";
    const state = Object.assign(document.createElement("span"), {
      className: "download-state",
      textContent: failureMessage === "network_waiting_for_reconnect"
        ? t("networkWaiting")
        : /\.recording(?: \(\d+\))?\.webm$/i.test(task.destination) && stateKey(task.state) === "downloading"
          ? t("recordingActive")
          : stateName(task.state),
    });
    if (typeof task.state === "object") {
      state.title = failureMessage === "facebook_direct_download_unavailable_use_recording"
        ? t("facebookRecordingFallback")
        : failureMessage === "network_waiting_for_reconnect"
          ? t("networkWaitingHint")
          : failureMessage;
    }
    titleLine.append(state.cloneNode(true));
    const actions = document.createElement("div");
    actions.className = "task-actions";
    const addAction = (label, command) => {
      const button = document.createElement("button");
      button.className = "task-action";
      button.textContent = label;
      button.dataset.command = command;
      button.title = label;
      button.setAttribute("aria-label", label);
      const icons = {
        pause_download: '<path d="M8 5v14M16 5v14"/>',
        resume_download: '<path d="m8 5 11 7-11 7z"/>',
        stop_recording: '<path d="M6 6h12v12H6z"/>',
        reveal_download: '<path d="M3 7a2 2 0 0 1 2-2h4l2 2h8a2 2 0 0 1 2 2v8a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"/>',
      };
      if (icons[command]) {
        button.classList.add("task-icon-action");
        button.innerHTML = `<svg viewBox="0 0 24 24" aria-hidden="true" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">${icons[command]}</svg>`;
      }
      const execute = async () => {
        if (busyIds.has(task.id)) return;
        busyIds.add(task.id);
        button.disabled = true;
        try {
          await invoke(command, { id: task.id });
          await refreshDownloads();
        } catch (error) {
          reportUiError("main", "renderDownloads", error);
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
    const recording = /\.recording(?: \(\d+\))?\.webm$/i.test(task.destination);
    if (recording && key === "downloading") addAction(t("stopRecording"), "stop_recording");
    else if (key === "downloading" || key === "inspecting")
      addAction(t("pause"), "pause_download");
    if (key === "paused") addAction(t("resume"), "resume_download");
    if (key === "failed") addAction(t("retry"), "resume_download");
    if ((key === "paused" || key === "failed") && !isTorrent(task)) {
      const locateButton = document.createElement("button");
      locateButton.className = "task-action";
      locateButton.textContent = t("locateFile");
      locateButton.title = t("locateFileHint");
      locateButton.onclick = async () => {
        if (busyIds.has(task.id)) return;
        busyIds.add(task.id);
        locateButton.disabled = true;
        try {
          const selected = await invoke("pick_directory", { initialDirectory: null });
          if (selected) {
            await invoke("relocate_download", { id: task.id, newDirectory: selected });
            await refreshDownloads();
          }
        } catch (error) {
          reportUiError("main", "renderDownloads", error);
          window.alert(String(error));
        } finally {
          busyIds.delete(task.id);
          locateButton.disabled = false;
        }
      };
      actions.append(locateButton);
    }
    if (key === "completed" && /\.recording(?: \(\d+\))?\.webm$/i.test(task.destination)) {
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
    if (key === "completed" && !isTorrent(task)) {
      const convert = document.createElement("button");
      convert.className = "task-action";
      convert.textContent = t("convertFile");
      convert.onclick = () => fileConverter.open(task.destination).catch(error => reportUiError("main", "conversion-open", error));
      actions.append(convert);
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
          // Blank input means "calculate only": nothing was compared, so do
          // not claim the file was verified.
          window.alert(`${expectedSha256.trim() ? t("integrityOk") : "SHA-256"}: ${digest}`);
          await refreshDownloads();
        } catch (error) { window.alert(String(error)); }
        finally { verify.disabled = false; }
      };
      actions.append(verify);
    }
    addAction(t("preview"), "preview_torrent");
    const previewButton = actions.lastElementChild;
    previewButton.classList.add("task-preview-action");
    previewButton.disabled = true;
    invoke("can_preview_download", { id: task.id }).then((available) => {
      if (previewButton.isConnected) previewButton.disabled = available !== true;
    }).catch(() => {});
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
        try {
          await invoke("set_download_bandwidth_limit", {
            id: task.id,
            bandwidthLimit: Math.round(megabytes * 1024 * 1024),
          });
          await refreshDownloads();
        } catch (error) {
          window.alert(String(error));
        }
      };
      actions.append(bandwidth);
    }
    const removeButton = document.createElement("button");
    removeButton.className = "task-action task-icon-action task-remove-action";
    removeButton.title = t("removeSelected");
    removeButton.setAttribute("aria-label", t("removeSelected"));
    removeButton.innerHTML = '<svg viewBox="0 0 24 24" aria-hidden="true" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M4 7h16M10 11v6M14 11v6M6 7l1 12a2 2 0 0 0 2 2h6a2 2 0 0 0 2-2l1-12M9 7V4h6v3"/></svg>';
    removeButton.onclick = () => {
      selectedIds.clear();
      selectedIds.add(task.id);
      renderDownloads(true);
      clearDialog.showModal();
    };
    actions.append(removeButton);
    addAction(t("openFolder"), "reveal_download");
    const status = document.createElement("div");
    status.className = "task-status";
    const primaryActions = document.createElement("div");
    primaryActions.className = "task-primary-actions";
    for (const button of [...actions.querySelectorAll('.task-icon-action,.task-preview-action')]) primaryActions.append(button);
    row.append(primaryActions);
    const taskDetails = document.createElement("details");
    taskDetails.className = "task-details";
    taskDetails.open = expandedTaskIds.has(task.id);
    const detailsSummary = document.createElement("summary");
    detailsSummary.textContent = "⋯";
    detailsSummary.title = t("taskDetails");
    detailsSummary.setAttribute("aria-label", t("taskDetails"));
    const detailsBody = document.createElement("div");
    detailsBody.className = "task-details-body";
    detailsBody.append(state, source, resumeCapability, actions);
    taskDetails.append(detailsSummary, detailsBody);
    status.append(taskDetails);
    row.prepend(select, icon, info);
    row.append(status);
    const scenicDetails = document.createElement("div");
    scenicDetails.className = "task-scenic-details";
    const detailValues = [[t("taskDestination"), task.destination.replace(/[\\/][^\\/]*$/, "")],
      [t("taskEngine"), task.engine || ""],
      [t("taskConnectionsLabel"), task.connections_override || ""],
      [t("taskEta"), task.torrent_eta || (speed > 0 && task.total > task.received ? `${Math.ceil((task.total - task.received) / speed)} s` : "")]];
    for (const [label, value] of detailValues) {
      if (!value) continue;
      const entry = document.createElement("span"); entry.textContent = label;
      const text = document.createElement("b"); text.textContent = String(value);
      entry.append(text); scenicDetails.append(entry);
    }
    if (scenicDetails.childElementCount) detailsBody.append(scenicDetails);
    list.append(row);
  }
  list.scrollTop = previousScrollTop;
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
  document.body.dataset.hasSelection = selectedIds.size ? "true" : "false";
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
  document.querySelectorAll("[data-i18n-aria]").forEach((node) => node.setAttribute("aria-label", t(node.dataset.i18nAria)));
  document.querySelectorAll("[data-language-choice]").forEach((button) =>
    button.classList.toggle("active", button.dataset.languageChoice === locale));
  if (!taskConnectionsManuallyChanged) {
    document.querySelector("#task-connections-value").value = document.querySelector("#task-connections").value;
  }
  setSiteThreadsNote();
  const activeNavigation = document.querySelector(`nav [data-page="${activePage}"]`);
  if (activeNavigation) document.querySelector("main > header h1").textContent = activeNavigation.querySelector("b")?.textContent || t("downloads");
  document.querySelector("#page-description").textContent = t(descriptions[activePage] || "downloadsDescription");
  renderDownloads(true);
}

const warnedFacebookRecordingFallbacks = new Set();
const aiNotifiedFailures = new Set();
async function refreshDownloads() {
  try {
    const ticket = downloadListState.beginRead();
    const refreshed = await invoke("list_downloads");
    if (selectionPointerActive) return;
    const accepted = downloadListState.acceptRead(ticket, refreshed);
    if (!accepted) return;
    downloads = accepted;
    for (const task of downloads) {
      const failure = typeof task.state === "object" ? task.state.failed?.message : null;
      if (!failure) continue;
      if (failure === "facebook_direct_download_unavailable_use_recording"
          && !warnedFacebookRecordingFallbacks.has(task.id)) {
        warnedFacebookRecordingFallbacks.add(task.id);
        window.alert(t("facebookRecordingFallback"));
      }
      if (!aiNotifiedFailures.has(task.id)) {
        aiNotifiedFailures.add(task.id);
        window.dispatchEvent(new CustomEvent("apocalipse-task-failed", {
          detail: { id: task.id, source: task.source, message: failure },
        }));
      }
    }
    const ids = new Set(downloads.map((task) => task.id));
    for (const id of selectedIds) if (!ids.has(id)) selectedIds.delete(id);
    updateSpeeds(downloads);
    renderDownloads();
  } catch (error) {
    reportUiError("main", "refreshDownloads", error);
  }
}

function acceptEnqueuedTask(task) {
  downloadListState.invalidate();
  downloads = downloadListState.visible([...downloads.filter((item) => item.id !== task.id), task]);
}

const dialog = document.querySelector("#add-dialog");
let downloadWindowResize = Promise.resolve();
function resizeDownloadWindow(expanded) {
  downloadWindowResize = downloadWindowResize.catch(() => {}).then(() => invoke("set_download_dialog_expanded", { expanded }));
  downloadWindowResize.catch(error => reportUiError("main", "download-window-size", error, "WARN"));
}
function showDownloadDialog() {
  if (dialog.open) return;
  dialog.showModal();
  resizeDownloadWindow(true);
}

const clearDialog = document.querySelector("#clear-dialog");
// Settings and Tools are pages now; these keep the old dialog call sites working.
const settingsDialog = { open: false, showModal() {}, close() { showPage("downloads"); } };
const toolsDialog = { open: false, showModal() {}, close() {} };
const logDialog = document.querySelector("#log-dialog");
const exportDialog = document.querySelector("#export-dialog");
const bandwidthDialog = document.querySelector("#bandwidth-dialog");
let exportTaskId = null;
document.querySelector("#history-search").oninput = (event) => {
  historyQuery = event.target.value.trim().toLocaleLowerCase();
  selectedIds.clear();
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
        acceptEnqueuedTask(await invoke("enqueue_download", { url, destinationDirectory, fileName, formatSelection: null, torrentSelection: null, mirrors: null, priority: 0, bandwidthLimit: null, connectionsOverride: null, context: {} }));
      } catch (error) { reportUiError("main", "import", error, "WARN"); }
    }
    renderDownloads();
  } catch (error) { reportUiError("main", "import-list", error); }
  finally { button.disabled = false; }
};
async function applyAboutMedia(media) {
  const panel = document.querySelector("#about-panel");
  const photo = document.querySelector("#about-creator-photo");
  const audio = document.querySelector("#about-audio");
  if (media?.backgroundDataUrl) panel.style.setProperty("--about-background", `url("${media.backgroundDataUrl}")`);
  if (media?.photoDataUrl) photo.src = media.photoDataUrl;
  if (media?.audioDataUrl && audio.src !== media.audioDataUrl) {
    audio.src = media.audioDataUrl;
    audio.load();
  }
}

async function loadAboutMedia() {
  const background = await invoke("get_about_background");
  if (background) {
    document.querySelector(".about-scene").style.backgroundImage = `url("${background}")`;
  }
  await applyAboutMedia(await invoke("get_about_media"));
}

const aboutAudio = document.querySelector("#about-audio");
const aboutPlayPause = document.querySelector("#about-play-pause");
aboutAudio.volume = Number(document.querySelector("#about-volume").value);
aboutPlayPause.onclick = () => {
  if (aboutAudio.paused) aboutAudio.play().catch(() => {});
  else aboutAudio.pause();
};
document.querySelector("#about-stop").onclick = () => {
  aboutAudio.pause();
  aboutAudio.currentTime = 0;
};
document.querySelector("#about-volume").oninput = (event) => {
  aboutAudio.volume = Number(event.target.value);
};
aboutAudio.onplay = () => { aboutPlayPause.textContent = t("aboutPause"); };
aboutAudio.onpause = () => { aboutPlayPause.textContent = t("aboutPlay"); };

loadAboutMedia().catch(error => reportUiError("main", "about-stop", error));

async function refreshTorrentStoreControls() {
  try {
    document.querySelector("#save-torrent-metadata").checked = await invoke("get_torrent_store_preference");
  } catch (error) { reportUiError("main", "refreshTorrentStoreControls", error); }
}
document.querySelector("#save-torrent-metadata").onchange = (event) => {
  invoke("set_torrent_store_preference", { enabled: event.target.checked }).catch((error) => {
    reportUiError("main", "save-torrent-metadata", error);
    event.target.checked = !event.target.checked;
  });
};
document.querySelector("#clear-torrent-store").onclick = async () => {
  if (!window.confirm(t("clearTorrentStoreConfirm"))) return;
  const button = document.querySelector("#clear-torrent-store");
  button.disabled = true;
  try {
    const removed = await invoke("clear_torrent_store");
    window.alert(t("torrentStoreCleared").replace("{count}", String(removed)));
  } catch (error) {
    reportUiError("main", "clear-torrent-store", error);
    window.alert(String(error));
  } finally {
    button.disabled = false;
  }
};

let activeSettingsTab = "general";
const settingsTabBusy = { value: false };
async function showSettingsTab(tab) {
  activeSettingsTab = tab;
  document.querySelectorAll("[data-settings-tab]").forEach((button) => {
    const on = button.dataset.settingsTab === tab;
    button.classList.toggle("active", on);
    button.setAttribute("aria-selected", String(on));
  });
  document.querySelectorAll("#settings-panel [data-pane]").forEach((pane) => { pane.hidden = pane.dataset.pane !== tab; });
  document.querySelector(".settings-footer").hidden = !["general", "network", "extension"].includes(tab);
  try {
    if (tab === "tools") await refreshToolStatuses();

  } catch (error) { reportUiError("main", "settings-tab", error); }
}
function showPage(page) {
  const openedAt = performance.now();
  // A selection made on another page must not survive into bulk actions here.
  if (page !== activePage) selectedIds.clear();
  activePage = page;
  document.body.dataset.page = page;
  const list = ["downloads", "recordings", "torrents"].includes(page);
  document.querySelectorAll("nav [data-page]").forEach((item) => item.classList.toggle("active", item.dataset.page === page));
  const button = document.querySelector(`nav [data-page="${page}"]`);
  document.querySelector("header h1").textContent = button?.querySelector("b")?.textContent || t("downloads");
  document.querySelector("#moq-capture-panel").hidden = page !== "recordings";
  document.querySelector("#settings-panel").hidden = page !== "settings";
  document.querySelector("#about-panel").hidden = page !== "about";
  document.querySelector("#logs-page-panel").hidden = page !== "logs";
  if (page === "logs") {
    refreshLogEvents().catch(error => reportUiError("main", "open-logs", error));
    invoke("get_log_editor").then(editor => {
      document.querySelector("#log-editor").value = editor;
      updateLogEditorControls();
    }).catch(error => reportUiError("main", "log-editor", error));
  }
  document.querySelector("#add").hidden = !list;
  document.querySelector("#torrent-store-controls").hidden = page !== "torrents";
  document.querySelector("#clear-torrent-store").hidden = page !== "torrents";
  if (page === "torrents") refreshTorrentStoreControls().catch(error => reportUiError("main", "clear-torrent-store", error));
  if (page === "about" && aboutAudio.src) {
    aboutAudio.currentTime = 0;
    aboutAudio.play().catch(() => {});
  } else {
    aboutAudio.pause();
    aboutAudio.currentTime = 0;
  }
  document.querySelector(".metrics").hidden = !list;
  document.querySelector(".panel").hidden = !list;
  renderDownloads();
  if (page === "settings") {
    openSettings(activeSettingsTab).then(() => showSettingsTab(activeSettingsTab))
      .catch(error => reportUiError("main", "open-settings", error));
  }
  invoke("record_ui_diagnostic", { level: "INFO", event: "page_opened", detail: `page=${page} panel_present=true duration_ms=${Math.round(performance.now() - openedAt)}` }).catch(() => {});
}
document.querySelectorAll("nav [data-page]").forEach((button) => {
  button.onclick = () => showPage(button.dataset.page);
});
document.querySelectorAll("[data-settings-tab]").forEach((button) => {
  button.onclick = () => showSettingsTab(button.dataset.settingsTab);
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

const darkModeToggle = document.querySelector("#dark-mode");
const applyColorMode = (dark) => {
  document.documentElement.dataset.mode = dark ? "dark" : "light";
  darkModeToggle.checked = Boolean(dark);
  try { localStorage.setItem("apocalipse.mode", dark ? "dark" : "light"); } catch {}
};
darkModeToggle.checked = document.documentElement.dataset.mode === "dark";
darkModeToggle.onchange = () => {
  applyColorMode(darkModeToggle.checked);
  invoke("set_dark_mode", { enabled: darkModeToggle.checked }).catch((error) => reportUiError("main", "dark-mode-save", error, "WARN"));
};
// settings.json is the source of truth; localStorage only avoids a flash.
invoke("get_dark_mode").then((saved) => applyColorMode(Boolean(saved))).catch((error) => reportUiError("main", "dark-mode-load", error, "WARN"));
document.querySelector("#open-link-window").onclick = () => {
  invoke("open_link_window").catch((error) => window.alert(String(error)));
  invoke("record_ui_diagnostic", { level: "INFO", event: "link_window_requested", detail: "source=navigation" }).catch(() => {});
};
document.querySelector("#open-ed2k-window").onclick = () => {
  invoke("open_ed2k_window").catch((error) => reportUiError("main", "ed2k-window-open", error));
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
  } catch (error) { reportUiError("main", "refreshToolStatuses", error); }
  finally { if (button) button.disabled = false; }
}
async function refreshDestinationHistory() {
  try {
    const destinations = await invoke("list_download_directories");
    const root = document.querySelector("#destination-list");
    root.replaceChildren();
    // Only the six most recent folders are shown (the default folder first),
    // as cards: folder name on top, full path underneath.
    const current = document.querySelector("#destination").value;
    for (const item of destinations.slice(0, 6)) {
      const row = document.createElement("div");
      row.className = "destination-card";
      row.classList.toggle("unavailable", !item.available);
      row.classList.toggle("selected", item.path === current);
      row.title = item.path;
      const select = document.createElement("button");
      select.type = "button";
      select.className = "destination-select";
      const folderName = item.path.split(/[\\/]/).filter(Boolean).pop() || item.path;
      const title = document.createElement("b");
      title.textContent = folderName;
      const badge = document.createElement("span");
      badge.className = "destination-badge";
      badge.textContent = item.isDefault ? t("defaultLocation") : (!item.available ? t("unavailableLocation") : "");
      badge.hidden = !badge.textContent;
      title.append(badge);
      const path = document.createElement("small");
      path.textContent = item.path;
      select.append(title, path);
      select.onclick = () => {
        document.querySelector("#destination").value = item.path;
        root.querySelectorAll(".destination-card").forEach((card) => card.classList.toggle("selected", card === row));
      };
      row.append(select);
      if (!item.isDefault) {
        const remove = document.createElement("button");
        remove.type = "button";
        remove.className = "destination-remove";
        remove.textContent = "×";
        remove.setAttribute("aria-label", t("clearLocations"));
        remove.onclick = async () => {
          await invoke("remove_download_directory", { path: item.path });
          await refreshDestinationHistory();
        };
        row.append(remove);
      }
      root.append(row);
    }
  } catch (error) {
    reportUiError("main", "refreshDestinationHistory", error);
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
      reportUiError("main", "refreshDestinationHistory", error);
    } finally {
      button.disabled = false;
    }
  };
});
document
  .querySelectorAll("[data-dialog-close]")
  .forEach((button) => (button.onclick = () => dialog.close()));

function resetAnalysisForNewRequest() {
  analysisGeneration += 1;
  const analyze = document.querySelector("#analyze");
  analyze.disabled = false;
  analyze.hidden = false;
  document.querySelector("#analysis").hidden = true;
  document.querySelector("#enqueue").hidden = true;
  return analysisGeneration;
}
const analysisIsCurrent = (generation) => generation === analysisGeneration;
dialog.addEventListener("close", () => {
  resizeDownloadWindow(false);
  analysisGeneration += 1;
  document.querySelector("#analyze").disabled = false;
});

function resetMediaInspection() {
  const panel = document.querySelector("#media-inspection");
  const thumbnail = document.querySelector("#media-thumbnail");
  panel.hidden = true;
  thumbnail.hidden = true;
  thumbnail.dataset.thumbnailSource = "";
  thumbnail.removeAttribute("src");
  document.querySelector("#media-title").textContent = "";
  document.querySelector("#media-duration").textContent = "";
  document.querySelector("#media-format").replaceChildren();
  document.querySelector("#media-format").hidden = false;
  document.querySelector("#hls-audio-conversion").hidden = true;
  document.querySelector("#hls-convert-audio").checked = false;
  delete document.querySelector("#hls-audio-conversion").dataset.originalName;
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
  loadPreviewThumbnail(image, thumbnail, youtubeThumbnailFallback(document.querySelector("#url").value) || youtubeThumbnailFallback(pendingReferer));
  panel.hidden = false;
}

async function showTorrentInspection(source, generation = analysisGeneration) {
  const torrent = await invoke("inspect_torrent_metadata", { source });
  if (!analysisIsCurrent(generation)) return false;
  document.querySelector("#torrent-title").textContent = torrent.name;
  document.querySelector("#file-name").value = torrent.name;
  knownArchiveName = "";
  refreshAutoExtractOption();
  document.querySelector("#torrent-total").textContent = formatBytes(torrent.totalSize);
  pendingExpectedSize = Number.isFinite(torrent.totalSize) ? torrent.totalSize : null;
  pendingTorrentMetadataPath = torrent.torrentPath || null;
  const root = document.querySelector("#torrent-files");
  root.replaceChildren();
  for (const file of torrent.files) {
    const row = document.createElement("label");
    const input = Object.assign(document.createElement("input"), { type: "checkbox", checked: true });
    input.dataset.torrentIndex = file.index;
    input.dataset.torrentSize = Number(file.size) || 0;
    row.append(input, Object.assign(document.createElement("span"), { textContent: file.path }), Object.assign(document.createElement("small"), { textContent: formatBytes(file.size) }));
    root.append(row);
  }
  document.querySelector("#torrent-inspection").hidden = false;
  return true;
}
// Site writes are serialized so rapid slider changes and unchecking cannot save out of order.
let siteThreads = { host: "", remembered: false };
let siteThreadsRequest = 0;
let siteThreadsSave = Promise.resolve();
function siteThreadsHost(source) {
  try {
    const url = new URL(source);
    return /^(https?|ftp):$/.test(url.protocol) ? url.hostname.toLowerCase().replace(/\.$/, "") : "";
  } catch { return ""; }
}
function setSiteThreadsNote() {
  const note = document.querySelector("#remember-site-threads-note");
  const box = document.querySelector("#remember-site-threads");
  if (siteThreads.host && box.checked) note.textContent = t("rememberSiteThreadsFor").replace("{host}", siteThreads.host);
  else if (siteThreads.host) note.textContent = t("rememberSiteThreadsOffer").replace("{host}", siteThreads.host);
  else note.textContent = t("rememberSiteThreadsHint");
  box.disabled = !siteThreadsHost(document.querySelector("#url").value.trim());
  document.querySelector("#task-connections").disabled = false;
}
async function applySiteThreadsChoice(source, forget = false) {
  source = source.trim();
  const host = siteThreadsHost(source);
  if (!host) return;
  const checked = document.querySelector("#remember-site-threads").checked;
  if (!checked && !siteThreads.remembered && !forget) return siteThreadsSave;
  const threads = checked ? Number(document.querySelector("#task-connections").value) || 8 : null;
  const request = ++siteThreadsRequest;
  const write = siteThreadsSave.catch(() => {}).then(() => invoke("set_site_threads", { url: source, threads }));
  siteThreadsSave = write;
  try {
    await write;
    if (request === siteThreadsRequest && host === siteThreadsHost(document.querySelector("#url").value.trim())) {
      siteThreads = { host, remembered: checked };
      setSiteThreadsNote();
    }
  } catch (error) {
    reportUiError("main", "siteThreads", error);
    throw error;
  }
}
function resetTaskConnections() {
  taskConnectionsManuallyChanged = false;
  siteThreadsRequest += 1;
  siteThreads = { host: "", remembered: false };
  document.querySelector("#task-connections").value = "8";
  document.querySelector("#task-connections-value").value = "8";
  document.querySelector("#remember-site-threads").checked = false;
  setSiteThreadsNote();
}
async function refreshSiteThreads() {
  const source = document.querySelector("#url").value.trim();
  const host = siteThreadsHost(source);
  if (!host) return;
  // An Analyze click must not replace an explicit choice with an older stored value.
  if (taskConnectionsManuallyChanged && siteThreads.host === host) return;
  const request = ++siteThreadsRequest;
  siteThreads.host = host;
  setSiteThreadsNote();
  let found;
  try {
    await siteThreadsSave.catch(() => {});
    found = await invoke("get_site_threads", { url: source });
  } catch { return; }
  if (request !== siteThreadsRequest || source !== document.querySelector("#url").value.trim()) return;
  siteThreads = { host: found.host, remembered: Boolean(found.remembered) };
  if (found.threads) {
    document.querySelector("#task-connections").value = String(found.threads);
    document.querySelector("#task-connections-value").value = String(found.threads);
  }
  document.querySelector("#remember-site-threads").checked = siteThreads.remembered;
  setSiteThreadsNote();
}
document.querySelectorAll("#add").forEach(
  (button) =>
    (button.onclick = () => {
      resetAnalysisForNewRequest();
      pendingBrowserAssistedPath = null;
      pendingDiagnosticTrace = null;
      pendingReferer = null;
      pendingDuration = null;
      pendingIsLive = false;
      pendingTitle = null;
      pendingThumbnail = null;
      pendingAudioUrl = null;
      pendingMediaKind = null;
      pendingExpectedSize = null;
      pendingCookieHeader = null;
      pendingUserAgent = null;
      pendingRequestMethod = null;
      pendingRequestBody = null;
      pendingRequestContentType = null;
      pendingTorrentMetadataPath = null;
      resetTaskConnections();
      resetMediaInspection();
      invoke("default_download_directory")
        .then((path) => {
          document.querySelector("#destination").value = path;
        })
        .catch(error => reportUiError("main", "resetTaskConnections", error));
      refreshDestinationHistory();
      showDownloadDialog();
    }),
);
const selectLanguage = (language) => {
  locale = ["en", "pt-BR", "zh-CN"].includes(language) ? language : "en";
  localStorage.setItem("apocalipse.language", locale);
  translate();
  window.dispatchEvent(new CustomEvent("apocalipse-language-changed", { detail: { language: locale } }));
  invoke("set_application_language", { language: locale }).catch(error => reportUiError("main", "resetTaskConnections", error));
};
document.querySelectorAll("[data-language-choice]").forEach((button) => {
  button.onclick = () => selectLanguage(button.dataset.languageChoice);
});
// settings.json is authoritative for the language: a missing or
// stale webview cache (localStorage is lost on some restarts) must not show a
// different language than the one the browser extension receives from the app.
// A backend still at the default "en" while the cache holds another language
// means the backend never saw the choice, so heal it instead of overriding it.
const applicationLanguageReady = invoke("get_application_language").then((saved) => {
  const supported = ["en", "pt-BR", "zh-CN"];
  if (!supported.includes(saved)) throw new Error("unsupported_saved_language");
  if (saved === "en" && locale !== "en" && supported.includes(locale)) {
    return invoke("set_application_language", { language: locale });
  }
  if (saved !== locale) selectLanguage(saved);
}).catch(error => reportUiError("main", "restoreApplicationLanguage", error));

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
    reportUiError("main", "redownload-selected", error);
  } finally {
    updateSelectionControls();
  }
};
document.querySelector("#clear-destinations").onclick = async () => {
  try {
    await invoke("clear_download_directories");
    await refreshDestinationHistory();
  } catch (error) {
    reportUiError("main", "clear-destinations", error);
  }
};
document
  .querySelectorAll("[data-clear-cancel]")
  .forEach((button) => (button.onclick = () => clearDialog.close()));
async function removeSelectedDownloads(button, deleteFiles) {
  button.disabled = true;
  const ids = [...selectedIds];
  // The explicit torrent-store control handles metadata deletion.
  const deleteTorrentMetadata = false;
  downloadListState.beginRemoval(ids);
  let removed = false;
  try {
    await invoke("remove_downloads", { ids, deleteFiles, deleteTorrentMetadata });
    removed = true;
    downloadListState.finishRemoval(ids, true);
    downloads = downloadListState.visible(downloads);
    for (const id of ids) selectedIds.delete(id);
    renderDownloads(true);
    clearDialog.close();
    await refreshDownloads();
  } catch (error) {
    reportUiError("main", "removeSelectedDownloads", error);
    window.alert(`${t("removeFailed")}: ${error}`);
  } finally {
    if (!removed) {
      downloadListState.finishRemoval(ids, false);
      await refreshDownloads();
    }
    button.disabled = false;
  }
}
document.querySelector("#clear-list-only").onclick = (event) =>
  removeSelectedDownloads(event.currentTarget, false);
document.querySelector("#clear-list-and-files").onclick = (event) =>
  removeSelectedDownloads(event.currentTarget, true);
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
function renderHostRules(rules) {
  const list = document.querySelector("#host-rule-list");
  list.replaceChildren();
  if (!rules.length) {
    const empty = document.createElement("small");
    empty.textContent = t("hostRulesEmpty");
    list.append(empty);
    return;
  }
  for (const rule of rules) {
    const row = document.createElement("div");
    const identity = document.createElement("span");
    const pattern = document.createElement("b");
    const details = document.createElement("small");
    const remove = document.createElement("button");
    pattern.textContent = rule.pattern;
    const parts = [];
    if (rule.username) parts.push(rule.username);
    if (rule.userAgent) parts.push("UA: " + rule.userAgent);
    if (rule.connections) parts.push(t("connections") + ": " + rule.connections);
    if (rule.bandwidthLimit) parts.push(t("hostRuleBandwidth") + ": " + (rule.bandwidthLimit / 1024 / 1024).toFixed(1));
    if (rule.hasPassword) parts.push("🔐");
    details.textContent = parts.join(" · ");
    identity.append(pattern, details);
    identity.onclick = () => {
      document.querySelector("#host-rule-pattern").value = rule.pattern;
      document.querySelector("#host-rule-username").value = rule.username || "";
      document.querySelector("#host-rule-password").value = "";
      document.querySelector("#host-rule-user-agent").value = rule.userAgent || "";
      document.querySelector("#host-rule-connections").value = rule.connections || "";
      document.querySelector("#host-rule-bandwidth").value = rule.bandwidthLimit ? (rule.bandwidthLimit / 1024 / 1024).toFixed(1) : "";
      document.querySelector("#host-rule-clear-password").checked = false;
      document.querySelector("#host-rule-pattern").focus();
    };
    remove.type = "button";
    remove.className = "danger-action";
    remove.textContent = t("hostRuleRemove");
    remove.onclick = async () => {
      if (!window.confirm(t("hostRuleRemoveConfirm").replace("{pattern}", rule.pattern))) return;
      remove.disabled = true;
      try {
        renderHostRules(await invoke("remove_host_rule", { pattern: rule.pattern }));
      } catch (error) {
        reportUiError("main", "renderHostRules", error);
        remove.disabled = false;
      }
    };
    row.append(identity, remove);
    list.append(row);
  }
}

document.querySelector("#save-host-rule").onclick = async (event) => {
  const button = event.currentTarget;
  const pattern = document.querySelector("#host-rule-pattern");
  const username = document.querySelector("#host-rule-username");
  const password = document.querySelector("#host-rule-password");
  const userAgent = document.querySelector("#host-rule-user-agent");
  const connections = document.querySelector("#host-rule-connections");
  const bandwidth = document.querySelector("#host-rule-bandwidth");
  if (![pattern, username, password, userAgent, connections, bandwidth].every((input) => input.reportValidity()) || !pattern.value.trim()) return;
  button.disabled = true;
  try {
    const connectionValue = connections.value ? Number(connections.value) : null;
    const bandwidthValue = Number(bandwidth.value) || 0;
    renderHostRules(await invoke("save_host_rule", {
      pattern: pattern.value,
      username: username.value,
      password: password.value,
      userAgent: userAgent.value,
      connections: connectionValue,
      bandwidthLimit: bandwidthValue > 0 ? Math.round(bandwidthValue * 1024 * 1024) : null,
      clearPassword: document.querySelector("#host-rule-clear-password").checked,
    }));
    pattern.value = "";
    username.value = "";
    password.value = "";
    userAgent.value = "";
    connections.value = "";
    bandwidth.value = "";
    document.querySelector("#host-rule-clear-password").checked = false;
  } catch (error) {
    reportUiError("main", "save-host-rule", error);
    window.alert(String(error));
  } finally {
    button.disabled = false;
  }
};

function renderXtremeRpcStatus(status = {}) {
  const target = document.querySelector("#xtreme-rpc-status");
  if (!target) return;
  if (!status.connected) {
    target.textContent = t("xtremeRpcDisconnected");
    return;
  }
  const details = [
    t("xtremeRpcConnected"),
    status.activePort ? `127.0.0.1:${status.activePort}` : "",
    status.version ? `nibiru-xtreme ${status.version}` : "",
  ].filter(Boolean);
  target.textContent = details.join(" · ");
}

const openSettings = async (target = "general") => {
  try {
    const [autostart, directory, clipboard, limits, pairing, userAgent, logEditor, proxy, dns, rpc, associations, hostRules] = await Promise.all([
      invoke("get_autostart"),
      invoke("default_download_directory"),
      invoke("get_clipboard_monitor"),
      invoke("get_transfer_limits"),
      invoke("get_bridge_pairing"),
      invoke("get_user_agent"),
      invoke("get_log_editor"),
      invoke("get_proxy_setting"),
      invoke("get_dns_setting"),
      invoke("get_xtreme_rpc_settings"),
      invoke("get_associations"),
      invoke("list_host_rules"),
    ]);
    document.querySelector("#autostart").checked = autostart.enabled;
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
    document.querySelector("#xtreme-rpc-enabled").checked = rpc.enabled;
    document.querySelector("#xtreme-rpc-auto-start").checked = rpc.autoStart;
    document.querySelector("#xtreme-rpc-port").value = rpc.configuredPort || "";
    renderXtremeRpcStatus(rpc);
    renderHostRules(hostRules);
    updateLogEditorControls();
    settingsDialog.showModal();
    const targetElement = {
      general: document.querySelector("#autostart"),
    }[target];
    targetElement?.scrollIntoView?.({ block: "center" });
    targetElement?.focus?.();
    invoke("record_ui_diagnostic", { level: "INFO", event: "settings_section_opened", detail: `section=${target} found=${Boolean(targetElement)}` }).catch(() => {});
  } catch (error) {
    reportUiError("main", "renderXtremeRpcStatus", error);
  }
};
document
  .querySelectorAll("[data-settings-close]")
  .forEach((button) => (button.onclick = () => settingsDialog.close()));
document.querySelector("#xtreme-rpc-test").onclick = async () => {
  const status = document.querySelector("#xtreme-rpc-status");
  const button = document.querySelector("#xtreme-rpc-test");
  button.disabled = true;
  status.textContent = t("xtremeRpcTesting");
  try {
    const rpcPortValue = Number(document.querySelector("#xtreme-rpc-port").value) || 0;
    await invoke("set_xtreme_rpc_settings", {
      enabled: document.querySelector("#xtreme-rpc-enabled").checked,
      autoStart: document.querySelector("#xtreme-rpc-auto-start").checked,
      port: rpcPortValue > 0 ? rpcPortValue : null,
    });
    renderXtremeRpcStatus(await invoke("test_xtreme_rpc"));
  } catch (error) {
    status.textContent = `${t("xtremeRpcDisconnected")} · ${error}`;
  } finally {
    button.disabled = false;
  }
};
document.querySelector("#xtreme-rpc-regenerate-token").onclick = async () => {
  const button = document.querySelector("#xtreme-rpc-regenerate-token");
  button.disabled = true;
  try {
    await invoke("regenerate_xtreme_rpc_token");
    document.querySelector("#xtreme-rpc-status").textContent = t("xtremeRpcTokenRegenerated");
  } catch (error) {
    reportUiError("main", "xtreme-rpc-regenerate-token", error);
    window.alert(String(error));
  } finally {
    button.disabled = false;
  }
};
for (const button of document.querySelectorAll('[data-window-action]')) {
  button.onclick = () => invoke("control_main_window", { action: button.dataset.windowAction })
    .catch(error => reportUiError("main", "window-control", error));
}
document.querySelector('#window-drag-region').onmousedown = event => {
  if (event.button === 0 && event.detail === 1) invoke("control_main_window", { action: "drag" }).catch(() => {});
};
document.querySelector('#window-drag-region').ondblclick = () => invoke("control_main_window", { action: "maximize" }).catch(() => {});
document.querySelector("#save-settings").onclick = async () => {
  const button = document.querySelector("#save-settings");
  const directory = document.querySelector("#default-directory");
  if (!directory.reportValidity()) return;
  button.disabled = true;
  try {
    localStorage.setItem("apocalipse.schedule.enabled", String(document.querySelector("#schedule-enabled").checked));
    localStorage.setItem("apocalipse.schedule.start", document.querySelector("#schedule-start").value);
    localStorage.setItem("apocalipse.schedule.end", document.querySelector("#schedule-end").value);
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
    const rpcPortValue = Number(document.querySelector("#xtreme-rpc-port").value) || 0;
    await invoke("set_xtreme_rpc_settings", {
      enabled: document.querySelector("#xtreme-rpc-enabled").checked,
      autoStart: document.querySelector("#xtreme-rpc-auto-start").checked,
      port: rpcPortValue > 0 ? rpcPortValue : null,
    });
    for (const input of document.querySelectorAll("[data-association]")) {
      if (!input.disabled && input.dataset.initial !== String(input.checked)) await invoke("set_association", {
        id: input.dataset.association,
        enabled: input.checked,
      });
    }
    settingsDialog.close();
  } catch (error) {
    reportUiError("main", "save-settings", error);
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
      xtreme: document.querySelector("#tool-xtreme").value,
      extractor: document.querySelector("#tool-extractor").value,
    });
    await invoke("set_media_player", { path: document.querySelector("#tool-player").value });
    toolsDialog.close();
  } catch (error) { reportUiError("main", "save-tools", error); }
  finally { button.disabled = false; }
};
document.querySelectorAll("[data-tool-download]").forEach((button) => {
  button.onclick = async () => {
    const id = button.dataset.toolDownload;
    const input = document.querySelector(`#tool-${id}`);
    const original = button.textContent;
    button.disabled = true;
    button.textContent = t("downloadingTool");
    try {
      const path = String(await invoke("download_tool", { id }));
      input.value = path;
      const status = document.querySelector(`[data-tool="${id}"] > span small`);
      if (status) {
        status.textContent = t("toolDownloaded");
        status.classList.add("tool-found");
      }
    } catch (error) {
      alert(String(error));
    } finally {
      button.disabled = false;
      button.textContent = original;
    }
  };
});
document.querySelectorAll("[data-tool-update]").forEach((button) => {
  button.onclick = async () => {
    button.disabled = true;
    try {
      const message = String(await invoke("update_tool", { id: button.dataset.toolUpdate }));
      await refreshToolStatuses();
      const updated = message.match(/^(.+) updated: (.+) → (.+)$/);
      const current = message.match(/^(.+) already current \((.+)\)$/);
      const installed = message.match(/^(\S+) installed: (.+)$/s);
      if (installed) {
        // No engine was at the path: the latest one was downloaded into the tools folder.
        // Fill the field so the only thing left is Save.
        const field = document.querySelector(`#tool-${button.dataset.toolUpdate}`);
        if (field) field.value = installed[2];
        alert(`${installed[1]} · ${t("toolDownloaded")}: ${installed[2]}`);
      } else if (updated) alert(`${updated[1]} · ${t("toolUpdated")}: ${updated[2]} → ${updated[3]}`);
      else if (current) alert(`${current[1]} · ${t("toolCurrent")} (${current[2]})`);
      else alert(message);
    } catch (error) {
      alert(String(error).replace("manual_update_required:", `${t("manualUpdateRequired")}:`));
    } finally {
      button.disabled = false;
    }
  };
});
document.querySelectorAll("[data-export-close]").forEach((button) => button.onclick = () => exportDialog.close());
document.querySelector("#donate-paypal").onclick = () => invoke("open_paypal_donation").catch(error => reportUiError("main", "donate-paypal", error));
document.querySelector("#export-format").onchange = (event) => {
  document.querySelector("#export-video-codec").disabled = ["mp3", "m4a", "opus", "flac", "wav"].includes(event.target.value);
};
document.querySelector("#export-recording").onclick = async (event) => {
  const button = event.currentTarget;
  const originalLabel = button.textContent;
  button.disabled = true;
  button.textContent = "…";
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
  } catch (error) {
    reportUiError("main", "export-recording", error);
    // Without a visible message the dialog looked frozen after a failed export.
    alert(String(error?.message || error || "export_failed"));
  } finally { button.disabled = false; button.textContent = originalLabel; }
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
  } catch (error) { reportUiError("main", "open-log", error); }
};
document.querySelector("#clear-log").onclick = async (event) => {
  const button = event.currentTarget;
  button.disabled = true;
  try {
    await invoke("clear_general_log");
    if (logDialog.open) await refreshDiagnosticLog();
  }
  catch (error) { reportUiError("main", "clear-log", error); }
  finally { button.disabled = false; }
};
document.querySelector("#refresh-log").onclick = () => refreshDiagnosticLog().catch(error => reportUiError("main", "refresh-log", error));
document.querySelector("#open-log-external").onclick = () => invoke("open_log_external").catch(error => reportUiError("main", "open-log-external", error));
document.querySelector("#pick-log-editor").onclick = async () => {
  const input = document.querySelector("#log-editor");
  try {
    const selected = await invoke("pick_executable", { initialPath: input.value });
    if (selected) {
      input.value = await invoke("set_log_editor", { path: selected });
      updateLogEditorControls();
    }
  } catch (error) { reportUiError("main", "pick-log-editor", error); }
};
document.querySelector("#remove-log-editor").onclick = async () => {
  try {
    document.querySelector("#log-editor").value = await invoke("set_log_editor", { path: "" });
    updateLogEditorControls();
  } catch (error) { reportUiError("main", "remove-log-editor", error); }
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
    } catch (error) { reportUiError("main", "remove-log-editor", error); }
    finally { button.disabled = false; }
  };
});
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
document.querySelector("#copy-pairing").onclick = () => invoke("copy_bridge_token").catch(error => reportUiError("main", "copy-pairing", error));
document.querySelector("#regenerate-pairing").onclick = async () => {
  try {
    const pairing = await invoke("regenerate_bridge_token");
    document.querySelector("#pairing-token").value = pairing.token;
  } catch (error) {
    reportUiError("main", "regenerate-pairing", error);
  }
};
function isArchiveFileName(name) {
  return /\.(zip|7z|rar|tar|tar\.gz|tgz|tar\.bz2|tbz2|tar\.xz|txz|tar\.zst|gz|bz2|xz|zst|cab|arj|lha|lzh)$/i.test(String(name || ""));
}
let knownArchiveName = "";
function refreshAutoExtractOption() {
  const option = document.querySelector("#auto-extract-option");
  const source = document.querySelector("#url").value.trim();
  const archive = /^https?:/i.test(source) && isArchiveFileName(knownArchiveName)
    && isArchiveFileName(document.querySelector("#file-name").value);
  option.hidden = !archive;
  if (!archive) document.querySelector("#auto-extract").checked = false;
}
document.querySelector("#file-name").addEventListener("input", refreshAutoExtractOption);
document.querySelector("#url").oninput = () => {
  // Once the user edits the source manually, any browser/extension metadata
  // belongs to the old URL and must not leak into the replacement request.
  // In particular this prevents stale expected sizes, cookies, POST bodies,
  // Referer/User-Agent values and browser-assisted local paths from being
  // reused for an unrelated direct download.
  pendingDiagnosticTrace = null;
  pendingReferer = null;
  pendingDuration = null;
  pendingIsLive = false;
  pendingTitle = null;
  pendingThumbnail = null;
  pendingAudioUrl = null;
  pendingMediaKind = null;
  pendingExpectedSize = null;
  pendingCookieHeader = null;
  pendingUserAgent = null;
  pendingRequestMethod = null;
  pendingRequestBody = null;
  pendingRequestContentType = null;
  pendingBrowserAssistedPath = null;
  pendingTorrentMetadataPath = null;
  resetTaskConnections();
  refreshSiteThreads();
  resetAnalysisForNewRequest();
  knownArchiveName = "";
  document.querySelector("#file-name").value = "";
  document.querySelector("#auto-extract").checked = false;
  document.querySelector("#auto-extract-option").hidden = true;
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

async function showMediaInspection(url, generation = analysisGeneration) {
  if (!analysisIsCurrent(generation)) return false;
  const panel = document.querySelector("#media-inspection");
  const select = document.querySelector("#media-format");
  select.replaceChildren();
  select.hidden = false;
  document.querySelector("#hls-audio-conversion").hidden = true;
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
    if (!analysisIsCurrent(generation)) return false;
    pendingTitle = media.title || pendingTitle;
    const ytFallback = youtubeThumbnailFallback(url);
    pendingThumbnail = pendingThumbnail || media.thumbnail || ytFallback;
    pendingDuration = Number.isFinite(media.duration) ? media.duration : pendingDuration;
    pendingIsLive = media.isLive === true;
    document.querySelector("#media-title").textContent = media.title;
    document.querySelector("#media-duration").textContent = media.duration ? `${t("duration")}: ${secondsLabel(media.duration)}` : "";
    const thumbnail = document.querySelector("#media-thumbnail");
    loadPreviewThumbnail(thumbnail, pendingThumbnail, ytFallback);
    for (const format of media.formats) option(select, format.selection, format.label);
    document.querySelector("#file-name").value = media.suggestedFileName;
    refreshAutoExtractOption();
    panel.hidden = false;
    return true;
  } catch (error) {
    if (!analysisIsCurrent(generation)) return false;
    reportUiError("main", "showMediaInspection", error, "WARN");
    // The extension may already have supplied trustworthy title/thumbnail
    // metadata. Preserve it when yt-dlp inspection is blocked by a VPN,
    // CAPTCHA or transient anti-bot response.
    showCapturedPreview({
      title: pendingTitle || t("mediaUnavailable"),
      thumbnail: (pendingThumbnail ||= youtubeThumbnailFallback(url)),
      kind: pendingMediaKind || "video",
      duration: pendingDuration,
      size: pendingExpectedSize,
      showFormats: true,
    });
    return true;
  }
}
function applyAudioFormatSelection(value) {
  const audio = value.match(/^audio:(.+)$/);
  if (!audio) return;
  const input = document.querySelector("#file-name");
  const base = input.value.replace(/\.[^.]+$/, "");
  input.value = `${base}.${audio[1]}`;
}
document.querySelector("#media-format").onchange = (event) => applyAudioFormatSelection(event.target.value);
function updateHlsAudioConversion() {
  const enabled = document.querySelector("#hls-convert-audio").checked;
  const format = document.querySelector("#hls-audio-format").value;
  const selection = enabled ? `audio:${format}` : "original";
  document.querySelector("#media-format").value = selection;
  document.querySelector("#hls-audio-format").disabled = !enabled;
  if (enabled) applyAudioFormatSelection(selection);
  else if (document.querySelector("#hls-audio-conversion").dataset.originalName) document.querySelector("#file-name").value = document.querySelector("#hls-audio-conversion").dataset.originalName;
}
document.querySelector("#hls-convert-audio").onchange = updateHlsAudioConversion;
document.querySelector("#hls-audio-format").onchange = updateHlsAudioConversion;
document.querySelector("#task-connections").oninput = (event) => {
  taskConnectionsManuallyChanged = true;
  siteThreadsRequest += 1;
  siteThreads.host = siteThreadsHost(document.querySelector("#url").value.trim());
  setSiteThreadsNote();
  document.querySelector("#task-connections-value").value = event.target.value;
};
document.querySelector("#url").addEventListener("change", refreshSiteThreads);
document.querySelector("#task-connections").addEventListener("change", () => {
  if (document.querySelector("#remember-site-threads").checked) {
    applySiteThreadsChoice(document.querySelector("#url").value).catch(() => {});
  }
});
document.querySelector("#remember-site-threads").addEventListener("change", () => {
  taskConnectionsManuallyChanged = true;
  siteThreads.host = siteThreadsHost(document.querySelector("#url").value.trim());
  setSiteThreadsNote();
  applySiteThreadsChoice(document.querySelector("#url").value, true).catch(() => {});
});
document.querySelector("#analyze").onclick = async () => {
  const url = document.querySelector("#url");
  if (!url.reportValidity()) return;
  refreshSiteThreads();
  const analyzeButton = document.querySelector("#analyze");
  if (analyzeButton.disabled) return;
  const generation = ++analysisGeneration;
  const current = () => analysisIsCurrent(generation);
  analyzeButton.disabled = true;
  const box = document.querySelector("#analysis");
  box.hidden = false;
  box.textContent = "…";
  let metadataTimer = null;
  const stopMetadataTimer = () => {
    if (metadataTimer) window.clearInterval(metadataTimer);
    metadataTimer = null;
  };
  try {
    if (pendingBrowserAssistedPath) {
      if (!current()) return;
      box.textContent = t("browserAssistedArchiveReady");
      refreshAutoExtractOption();
      document.querySelector("#analyze").hidden = true;
      document.querySelector("#enqueue").hidden = false;
      return;
    }
    try {
      const hostResolution = await invoke("resolve_file_host_url", { url: url.value });
      if (!current()) return;
      if (hostResolution?.adapted && hostResolution.url) url.value = hostResolution.url;
    } catch (error) {
      if (!current()) return;
      reportUiError("main", "file-host-adapter", error, "WARN");
    }
    const plan = await invoke("inspect_url", { url: url.value });
    if (!current()) return;
    const fileName = document.querySelector("#file-name");
    const suggestedFileName = await invoke("suggest_download_name", { url: url.value });
    if (!current()) return;
    knownArchiveName = knownArchiveName || (isArchiveFileName(suggestedFileName) ? suggestedFileName : "");
    const currentName = fileName.value.trim();
    const genericName = /^(?:watch|reel|video|download)(?:\.[a-z0-9]{1,10})?$/i.test(currentName);
    if (!currentName || (genericName && !pendingTitle)) fileName.value = suggestedFileName;
    box.textContent = `${plan.primary} · ${plan.reason}`;
    if (plan.primary === "YtDlp") {
      const shown = await showMediaInspection(url.value, generation);
      if (!shown || !current()) return;
    } else if (/^magnet:/i.test(url.value) || /\.torrent$/i.test(url.value.split(/[?#]/)[0])) {
      const startedAt = Date.now();
      const updateMetadataStatus = () => {
        if (!current()) {
          stopMetadataTimer();
          return;
        }
        const seconds = Math.floor((Date.now() - startedAt) / 1000);
        box.textContent = `${t("torrentMetadataSeeking")} · ${seconds}s`;
      };
      updateMetadataStatus();
      metadataTimer = window.setInterval(updateMetadataStatus, 1000);
      const shown = await showTorrentInspection(url.value, generation);
      stopMetadataTimer();
      if (!shown || !current()) return;
      box.textContent = "nibiru-xtreme";
    } else if (plan.reason === "hls_manifest") {
      const select = document.querySelector("#media-format");
      select.replaceChildren();
      option(select, "original", pendingMediaKind === "audio" ? "Original (MP4/M4A)" : t("bestQuality"));
      for (const format of ["mp3", "m4a", "opus", "flac", "wav"])
        option(select, `audio:${format}`, `${t("audioOnly")} · ${format.toUpperCase()}`);
      const audioHls = pendingMediaKind === "audio" && showDirectAudioConversion(url.value, fileName.value, pendingMediaKind);
      select.hidden = audioHls;
      document.querySelector("#hls-audio-conversion").hidden = !audioHls;
      document.querySelector("#hls-convert-audio").checked = false;
      updateHlsAudioConversion();
      showCapturedPreview({ title: pendingTitle || "HLS", thumbnail: pendingThumbnail, kind: "M3U8 / HLS", duration: pendingDuration, size: pendingExpectedSize, showFormats: true });
    } else if (showDirectAudioConversion(url.value, fileName.value, pendingMediaKind)) {
      const select = document.querySelector("#media-format");
      select.replaceChildren();
      option(select, "original", t("originalAudio"));
      for (const format of ["mp3", "m4a", "opus", "flac", "wav"])
        option(select, `audio:${format}`, `${t("audioOnly")} · ${format.toUpperCase()}`);
      select.hidden = true;
      const conversion = document.querySelector("#hls-audio-conversion");
      conversion.hidden = false;
      conversion.dataset.originalName = fileName.value;
      document.querySelector("#hls-convert-audio").checked = false;
      updateHlsAudioConversion();
      showCapturedPreview({ title: pendingTitle || fileName.value, thumbnail: pendingThumbnail, kind: "audio", duration: pendingDuration, size: pendingExpectedSize, showFormats: true });
    } else if (pendingMediaKind === "image" || /\.(?:avif|bmp|gif|jpe?g|png|svg|webp)(?:$|[?#])/i.test(url.value)) {
      showCapturedPreview({ title: pendingTitle || fileName.value, thumbnail: pendingThumbnail || url.value, kind: pendingMediaKind || "image", duration: null, size: pendingExpectedSize });
    }
    if (!current()) return;
    refreshAutoExtractOption();
    document.querySelector("#analyze").hidden = true;
    document.querySelector("#enqueue").hidden = false;
  } catch (error) {
    if (!current()) return;
    const message = String(error);
    box.textContent = /connections=0:seeders=0/.test(message)
      ? t("torrentMetadataNoPeers")
      : /xtreme_metadata_timeout:connections=[1-9][0-9]*/.test(message) ? t("torrentMetadataPeersStalled")
      : /timed? ?out|timeout/i.test(message) ? t("torrentMetadataTimeout") : message;
  } finally {
    stopMetadataTimer();
    if (current()) analyzeButton.disabled = false;
  }
};

// ---- File / extraction conflicts ------------------------------------------
const conflictRememberedMode = { download: null, extract: null };
const conflictDialog = document.querySelector("#conflict-dialog");
const formatConflictText = (key, values = {}) => Object.entries(values)
  .reduce((text, [name, value]) => text.replaceAll(`{${name}}`, value), t(key));
// Resolves with "rename" | "replace" | "delete" | "skip", or null when cancelled.
function showConflictDialog({ kind, folder, fileName, existing, incomingSize, suggestedName, archive, count, names, protectedApp = false }) {
  if (conflictRememberedMode[kind]) return Promise.resolve(conflictRememberedMode[kind]);
  return new Promise((resolve) => {
    const extract = kind === "extract";
    const $ = (selector) => conflictDialog.querySelector(selector);
    $("#conflict-eyebrow").textContent = t(extract ? "conflictExtractEyebrow" : "conflictFileEyebrow").toUpperCase();
    $("#conflict-title").textContent = t(extract ? "conflictExtractTitle" : "conflictFileTitle");
    $("#conflict-copy").textContent = extract
      ? formatConflictText("conflictExtractCopy", { archive, folder, count })
      : formatConflictText("conflictFileCopy", { folder });
    $("#conflict-compare").hidden = extract;
    $("#conflict-files").hidden = !extract;
    if (extract) {
      const list = $("#conflict-files");
      list.replaceChildren(...names.map((name) => Object.assign(document.createElement("div"), { textContent: name })));
      if (count > names.length) {
        list.append(Object.assign(document.createElement("div"), { className: "more", textContent: formatConflictText("conflictMore", { n: count - names.length }) }));
      }
    } else {
      $("#conflict-old-name").textContent = fileName;
      $("#conflict-old-meta").textContent = `${formatBytes(existing.size)} · ${existing.modifiedMs ? new Date(existing.modifiedMs).toLocaleString() : ""}`;
      $("#conflict-new-name").textContent = fileName;
      $("#conflict-new-meta").textContent = incomingSize > 0 ? formatBytes(incomingSize) : t("conflictNew");
    }
    $("#conflict-rename-hint").textContent = extract
      ? t("conflictRenameExtractHint")
      : formatConflictText("conflictRenameFileHint", { name: suggestedName });
    $("#conflict-replace-hint").textContent = t(extract ? "conflictReplaceExtractHint" : "conflictReplaceFileHint");
    // The running app is never replaced or deleted: only "rename" stays on offer.
    $('[data-conflict-mode="replace"]').hidden = protectedApp;
    $('[data-conflict-mode="delete"]').hidden = !extract || protectedApp;
    $("#conflict-protected").hidden = !protectedApp;
    $('[data-conflict-mode="skip"]').hidden = extract;
    $("#conflict-remember").checked = false;
    const continueButton = $("#conflict-continue");
    let selected = "rename";
    const refresh = () => {
      conflictDialog.querySelectorAll("[data-conflict-mode]").forEach((button) => {
        button.classList.toggle("selected", button.dataset.conflictMode === selected);
      });
      continueButton.textContent = selected === "delete" ? t("conflictConfirmDelete") : t("conflictContinue");
    };
    const finish = (value) => {
      conflictDialog.onclose = null;
      conflictDialog.close();
      // Never remember "delete": erasing a folder always needs an explicit confirmation.
      if (value && value !== "delete" && $("#conflict-remember").checked) conflictRememberedMode[kind] = value;
      resolve(value);
    };
    conflictDialog.querySelectorAll("[data-conflict-mode]").forEach((button) => {
      button.onclick = () => { selected = button.dataset.conflictMode; refresh(); };
    });
    continueButton.onclick = () => finish(selected);
    conflictDialog.querySelectorAll("[data-conflict-cancel]").forEach((button) => { button.onclick = () => finish(null); });
    conflictDialog.onclose = () => resolve(null);
    refresh();
    conflictDialog.showModal();
  });
}
// Returns the file name to use, or null when the user cancelled/skipped.
async function resolveFileConflict(directory, fileName, incomingSize) {
  if (!directory || !fileName) return fileName;
  let existing;
  try { existing = await invoke("check_file_conflict", { directory, fileName }); } catch { return fileName; }
  if (!existing.exists) return fileName;
  const mode = await showConflictDialog({ kind: "download", folder: directory, fileName, existing, incomingSize: incomingSize || 0, suggestedName: existing.suggestedName, protectedApp: existing.protectedApp });
  if (!mode || mode === "skip") return null;
  if (mode === "rename") return existing.suggestedName;
  try {
    await invoke("remove_existing_file", { directory, fileName });
  } catch (error) {
    const inUse = String(error).match(/file_in_use:(.*)$/);
    if (!inUse) throw error;
    window.alert(formatConflictText("conflictInUse", { files: inUse[1].trim() }));
    return null;
  }
  return fileName;
}
window.__TAURI__?.event?.listen?.("archive-conflict", async (event) => {
  const conflict = event.payload;
  const mode = await showConflictDialog({
    kind: "extract",
    folder: conflict.destination,
    archive: conflict.archive.split(/[\\/]/).pop(),
    count: conflict.count,
    names: conflict.names,
    protectedApp: conflict.protectedApp,
  });
  try {
    await invoke("resolve_archive_conflict", { id: conflict.taskId, mode: mode || "skip" });
  } catch (error) {
    const text = String(error);
    const inUse = text.match(/(?:archive_files_in_use|file_in_use):(.*)$/);
    if (inUse) window.alert(formatConflictText("conflictInUse", { files: inUse[1].trim() }));
    reportUiError("main", "resolve-archive-conflict", error, "WARN");
  }
}).catch(error => reportUiError("main", "archive-conflict-listener", error));

document.querySelector("#enqueue").onclick = async () => {
  const url = document.querySelector("#url");
  const button = document.querySelector("#enqueue");
  button.disabled = true;
  try {
    const torrentInputs = document.querySelector("#torrent-inspection").hidden
      ? null : [...document.querySelectorAll("[data-torrent-index]:checked")];
    const torrentSelection = torrentInputs
      ? torrentInputs.map((input) => Number(input.dataset.torrentIndex))
      : null;
    if (torrentSelection && !torrentSelection.length) throw new Error("Selecione pelo menos um arquivo do torrent.");
    const torrentExpectedSize = torrentInputs
      ? torrentInputs.reduce((total, input) => total + (Number(input.dataset.torrentSize) || 0), 0)
      : null;
    const autoExtract = document.querySelector("#auto-extract-option").hidden ? false : document.querySelector("#auto-extract").checked;
    let chosenFileName = document.querySelector("#file-name").value;
    if (!torrentSelection) {
      chosenFileName = await resolveFileConflict(document.querySelector("#destination").value, chosenFileName, pendingExpectedSize);
      if (chosenFileName === null) return;
    }
    await applySiteThreadsChoice(url.value);
    const acceptedTask = pendingBrowserAssistedPath
      ? await invoke("import_browser_assisted_download", {
          localPath: pendingBrowserAssistedPath,
          url: url.value,
          destinationDirectory: document.querySelector("#destination").value,
          fileName: chosenFileName,
          autoExtract,
        })
      : await invoke("enqueue_download", {
        url: url.value,
        destinationDirectory: document.querySelector("#destination").value,
        fileName: chosenFileName,
        formatSelection: document.querySelector("#media-inspection").hidden || document.querySelector("#media-format-control").hidden ? null : document.querySelector("#media-format").value,
        torrentSelection,
        mirrors: document.querySelector("#mirrors").value.split(/\r?\n/).map((value) => value.trim()).filter(Boolean),
        priority: Number(document.querySelector("#priority").value),
        bandwidthLimit: Math.round((Number(document.querySelector("#download-bandwidth-limit").value) || 0) * 1024 * 1024) || null,
        connectionsOverride: Number(document.querySelector("#task-connections").value) || 8,
        autoExtract,
        context: {
          traceId: pendingDiagnosticTrace,
          referer: pendingReferer,
          knownDuration: pendingDuration,
          isLive: pendingIsLive,
          title: pendingTitle,
          thumbnail: pendingThumbnail,
          audioUrl: pendingAudioUrl,
          expectedSize: torrentExpectedSize || pendingExpectedSize,
          cookieHeader: pendingCookieHeader,
          userAgent: pendingUserAgent,
          requestMethod: pendingRequestMethod,
          requestBody: pendingRequestBody,
          requestContentType: pendingRequestContentType,
          torrentMetadataPath: pendingTorrentMetadataPath,
        },
      });
    acceptEnqueuedTask(acceptedTask);
    renderDownloads();
    dialog.close();
    url.value = "";
    document.querySelector("#mirrors").value = "";
    document.querySelector("#priority").value = "0";
    document.querySelector("#download-bandwidth-limit").value = "0";
    document.querySelector("#auto-extract").checked = false;
    document.querySelector("#auto-extract-option").hidden = true;
    resetTaskConnections();
    pendingTitle = null;
    pendingThumbnail = null;
    pendingAudioUrl = null;
    pendingMediaKind = null;
    pendingExpectedSize = null;
    pendingBrowserAssistedPath = null;
    pendingTorrentMetadataPath = null;
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
      invoke("pause_download", { id: task.id }).catch(error => reportUiError("main", "enqueue", error));
    } else if (allowed && key === "paused" && schedulerPaused.delete(task.id)) {
      invoke("resume_download", { id: task.id }).catch(error => reportUiError("main", "enqueue", error));
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
    if (dialog.open || document.querySelector("#conversion-dialog").open) return;
    pendingDiagnosticTrace = null;
    pendingReferer = null;
    pendingDuration = null;
    pendingIsLive = false;
    pendingTitle = null;
    pendingThumbnail = null;
    pendingAudioUrl = null;
    pendingMediaKind = null;
    pendingExpectedSize = null;
    pendingCookieHeader = null;
    pendingUserAgent = null;
    pendingRequestMethod = null;
    pendingRequestBody = null;
    pendingRequestContentType = null;
    pendingTorrentMetadataPath = null;
    resetTaskConnections();
    const url = document.querySelector("#url");
    url.value = link;
    refreshSiteThreads();
    knownArchiveName = "";
    document.querySelector("#file-name").value = "";
    document.querySelector("#auto-extract").checked = false;
    document.querySelector("#auto-extract-option").hidden = true;
    resetAnalysisForNewRequest();
    resetMediaInspection();
    await invoke("activate_main_window");
    if (!dialog.open) showDownloadDialog();
    url.focus();
  } catch (error) {
    reportUiError("main", "enqueue", error);
  }
}, 750);
let consumingBrowserAssistedDownload = false;
async function consumeBrowserAssistedDownload() {
  if (consumingBrowserAssistedDownload || dialog.open || document.querySelector("#conversion-dialog").open) return;
  consumingBrowserAssistedDownload = true;
  try {
    const request = await invoke("take_browser_assisted_download");
    if (!request) return;
    pendingBrowserAssistedPath = request.fileName || null;
    pendingDiagnosticTrace = null;
    pendingReferer = null;
    pendingDuration = null;
    pendingIsLive = false;
    pendingTitle = null;
    pendingThumbnail = null;
    pendingAudioUrl = null;
    pendingMediaKind = null;
    pendingExpectedSize = Number.isFinite(request.total) ? request.total : null;
    pendingCookieHeader = null;
    pendingUserAgent = null;
    pendingRequestMethod = null;
    pendingRequestBody = null;
    pendingRequestContentType = null;
    resetTaskConnections();
    document.querySelector("#url").value = request.url;
    refreshSiteThreads();
    const sourceName = String(request.fileName || "").split(/[\\/]/).pop() || "download.zip";
    document.querySelector("#file-name").value = sourceName;
    knownArchiveName = isArchiveFileName(sourceName) ? sourceName : "";
    resetAnalysisForNewRequest();
    document.querySelector("#destination").value = await invoke("default_download_directory");
    resetMediaInspection();
    refreshAutoExtractOption();
    await refreshDestinationHistory();
    await invoke("activate_main_window");
    if (!dialog.open) showDownloadDialog();
    document.querySelector("#url").focus();
  } catch (error) {
    reportUiError("main", "consumeBrowserAssistedDownload", error);
  } finally {
    consumingBrowserAssistedDownload = false;
  }
}
setInterval(consumeBrowserAssistedDownload, 400);
window.__TAURI__?.event?.listen?.("browser-assisted-ready", consumeBrowserAssistedDownload).catch(error => reportUiError("main", "consumeBrowserAssistedDownload", error));

let consumingBridgeDownload = false;
async function consumeBridgeDownload() {
  if (consumingBridgeDownload || document.querySelector("#conversion-dialog").open) return;
  consumingBridgeDownload = true;
  try {
    const currentUrl = dialog.open ? document.querySelector("#url").value : null;
    const request = await invoke("take_bridge_download", { currentUrl });
    if (!request) return;
    lastClipboardLink = request.url;
    pendingBrowserAssistedPath = null;
    pendingDiagnosticTrace = request.traceId || null;
    pendingReferer = request.pageUrl || null;
    pendingDuration = Number.isFinite(request.duration) ? request.duration : null;
    pendingIsLive = false;
    pendingTitle = request.title || null;
    pendingThumbnail = request.thumbnail || youtubeThumbnailFallback(request.url) || youtubeThumbnailFallback(request.pageUrl);
    pendingAudioUrl = request.audioUrl || null;
    pendingMediaKind = request.mediaKind || null;
    pendingExpectedSize = Number.isFinite(request.expectedSize) ? request.expectedSize : null;
    pendingCookieHeader = request.cookieHeader || null;
    pendingUserAgent = request.userAgent || null;
    pendingRequestMethod = request.requestMethod || null;
    pendingRequestBody = request.requestBody || null;
    pendingRequestContentType = request.requestContentType || null;
    resetTaskConnections();
    document.querySelector("#url").value = request.url;
    refreshSiteThreads();
    const requestedName = request.fileName || "";
    const genericMediaName = /^(?:watch|reel|video|audio|track|download)(?:\.[a-z0-9]{1,10})?$/i.test(requestedName.trim());
    const titleName = String(pendingTitle || "")
      .replace(/[<>:\"/\\|?*\u0000-\u001f]/g, "_")
      .replace(/[. ]+$/g, "")
      .trim();
    // A resolved page/media title (e.g. a SoundCloud track name) is almost
    // always a better file name than the source URL's own path, which for a
    // signed CDN stream carries no useful name at all. This used to apply
    // only to video captures, so audio captures (like SoundCloud) fell back
    // to the raw, meaningless URL segment even when the real title was known.
    const titleExtension = pendingMediaKind === "audio" ? "m4a" : "mp4";
    document.querySelector("#file-name").value = (pendingMediaKind === "video" || pendingMediaKind === "audio") && titleName && (!requestedName || genericMediaName)
      ? `${[...titleName].slice(0, 110).join("")}.${titleExtension}`
      : requestedName;
    document.querySelector("#auto-extract").checked = false;
    knownArchiveName = /^https?:/i.test(request.url) && isArchiveFileName(requestedName) && !["video", "audio", "image"].includes(pendingMediaKind) ? requestedName : "";
    resetAnalysisForNewRequest();
    resetMediaInspection();
    refreshAutoExtractOption();
    if (pendingThumbnail || pendingTitle || pendingMediaKind === "image") {
      showCapturedPreview({ title: pendingTitle, thumbnail: pendingThumbnail || (pendingMediaKind === "image" ? request.url : null), kind: pendingMediaKind, duration: pendingDuration, size: pendingExpectedSize });
    }
    document.querySelector("#destination").value = await invoke("default_download_directory");
    await refreshDestinationHistory();
    await invoke("activate_main_window");
    if (!dialog.open) showDownloadDialog();
    invoke("record_ui_diagnostic", { level: "INFO", event: "save_dialog_opened", detail: `trace=${pendingDiagnosticTrace || "none"}` }).catch(() => {});
    document.querySelector("#url").focus();
  } catch (error) { reportUiError("main", "consumeBridgeDownload", error); }
  finally { consumingBridgeDownload = false; }
}
setInterval(consumeBridgeDownload, 400);
window.__TAURI__?.event?.listen?.("bridge-download-ready", consumeBridgeDownload).catch(error => reportUiError("main", "consumeBridgeDownload", error));
window.__TAURI__?.event?.listen?.("blob-upload-progress", (event) => {
  const progress = event.payload || {};
  const task = downloads.find((item) => item.id === progress.taskId);
  if (!task) {
    refreshDownloads();
    return;
  }
  task.received = Number(progress.received) || 0;
  task.total = progress.total ?? task.total;
  task.download_speed = Number(progress.downloadSpeed) || 0;
  if (task.total) task.progress_percent = task.received * 100 / task.total;
  updateSpeeds(downloads);
  renderDownloads(true);
}).catch(error => reportUiError("main", "consumeBridgeDownload", error));
window.__TAURI__?.event?.listen?.("media-preview-error", (event) => {
  const prefix = locale === "pt-BR" ? "Falha na pr\u00e9-visualiza\u00e7\u00e3o. Consulte Logs para os detalhes."
    : locale === "zh-CN" ? "\u9884\u89c8\u5931\u8d25\u3002\u8bf7\u67e5\u770b\u65e5\u5fd7\u3002" : "Preview failed. See Logs for details.";
  window.alert(`${prefix}\n${String(event.payload || "preview_failed")}`);
}).catch(error => reportUiError("main", "consumeBridgeDownload", error));
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
  } catch (error) { reportUiError("main", "consumeBridgeDownload", error); }
}).catch(error => reportUiError("main", "consumeBridgeDownload", error));
const bridgeStatusStartedAt = Date.now();
let bridgeEverConnected = false;
async function refreshBridgeStatus() {
  try {
    const status = await invoke("get_bridge_pairing");
    const root = document.querySelector("footer > span:first-child");
    root.classList.toggle("bridge-waiting", !status.connected);
    root.classList.toggle("bridge-connected", status.connected);
    if (status.connected) bridgeEverConnected = true;
    const initiallyWaiting = !bridgeEverConnected && Date.now() - bridgeStatusStartedAt < 6000;
    root.querySelector("b").textContent = status.connected
      ? t("bridgeConnected")
      : t(initiallyWaiting ? "bridgeWaiting" : "bridgeDisconnected");
  } catch (error) {
    reportUiError("main", "refreshBridgeStatus", error);
  }
}
refreshBridgeStatus();
setInterval(refreshBridgeStatus, 3000);

document.querySelector("#dictionary-download").onclick = async event => {
  const button = event.currentTarget, status = document.querySelector("#dictionary-status");
  button.disabled = true;
  status.textContent = t("transportWorking");
  try {
    const path = await invoke("download_with_dictionary", {
      dictionaryUrl: document.querySelector("#dictionary-url").value.trim(),
      artifactUrl: document.querySelector("#dictionary-artifact-url").value.trim(),
      sha256: document.querySelector("#dictionary-sha256").value.trim(),
    });
    status.textContent = t("transportSaved") + path;
  } catch (error) { status.textContent = String(error); }
  finally { button.disabled = false; }
};
let moqCaptureJob = null;
document.querySelector("#moq-start").onclick = async event => {
  const button = event.currentTarget, status = document.querySelector("#moq-status");
  button.disabled = true;
  moqCaptureJob = crypto.randomUUID();
  status.textContent = t("moqActive");
  document.querySelector("#moq-stop").disabled = false;
  try {
    const path = await invoke("capture_moq_objects", {
      relay: document.querySelector("#moq-relay").value.trim(),
      broadcast: document.querySelector("#moq-broadcast").value.trim(),
      track: document.querySelector("#moq-track").value.trim(),
      seconds: Number(document.querySelector("#moq-seconds").value), jobId: moqCaptureJob,
    });
    status.textContent = t("transportSaved") + path;
  } catch (error) { status.textContent = String(error); }
  finally { moqCaptureJob = null; button.disabled = false; document.querySelector("#moq-stop").disabled = true; }
};
document.querySelector("#moq-stop").onclick = async event => {
  if (!moqCaptureJob) return;
  event.currentTarget.disabled = true;
  try {
    await invoke("stop_transport_capture", {jobId: moqCaptureJob});
    document.querySelector("#moq-status").textContent = t("moqStopping");
  } catch (error) { document.querySelector("#moq-status").textContent = String(error); event.currentTarget.disabled = !moqCaptureJob; }
};

// Reveal the list scrollbar during wheel, touch and keyboard scrolling.
let listScrollTimer;
document.querySelector("#download-list").addEventListener("scroll", () => {
  const list = document.querySelector("#download-list");
  list.classList.add("is-scrolling");
  clearTimeout(listScrollTimer);
  listScrollTimer = setTimeout(() => list.classList.remove("is-scrolling"), 900);
}, { passive: true });

const fileConverter = window.createAdmFileConverter({ invoke, t, resize: resizeDownloadWindow, report: reportUiError });
for (const id of ["open-converter", "download-open-converter"]) document.getElementById(id).onclick = () => {
  if (dialog.open) dialog.close();
  fileConverter.open().catch(error => reportUiError("main", "conversion-open", error));
};
