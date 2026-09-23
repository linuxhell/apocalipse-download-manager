const catalogs = {
  en: {
    archiveExtractor: "Archive extractor (7-Zip / RAR / UnRAR / unar / bsdtar / tar)",
    autoExtract: "Extract automatically after download",
    autoExtractHint: "Shown only for archive files. Loose root files are kept inside a folder named after the archive.",
    browserAssistedArchiveReady: "Archive received from the browser. Choose where to save it and whether to extract it automatically.",
    networkWaiting: "Waiting for network",
    networkWaitingHint: "The connection changed or went offline. This task will resume automatically when a network interface is available.",
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
    aiDescription: "A local assistant that explains problems, reads diagnostics and follows correction tests.",
    aiLocalPrivate: "Local, private and specialized in your Apocalipse",
    aiCorrectionHistory: "Correction history", aiCorrectionsFound: "Corrections found",
    aiCorrectionsHint: "Corrections remain local and are only confirmed after your manual test.",
    aiDeleteAll: "Delete all", aiNoCorrections: "No corrections found.", aiInputHint: "Ask about Apocalipse…", aiSend: "Send",
    aiPrivacy: "Works locally using application status and privacy-safe diagnostic records.",
    aiCopyName: "Copy name", aiDeleteCorrection: "Delete", aiApplyCorrection: "Apply for testing", aiUndoCorrection: "Undo",
    aiStatusProposed: "Awaiting approval", aiStatusTesting: "Testing", aiStatusSaved: "Saved", aiStatusConfirmed: "Confirmed", aiStatusRejected: "Did not work",
    toolsPageDescription: "Manage the engines used for media, transfers, conversion and preview.",
    settingsDescription: "Configure appearance, integrations, network and application behavior.",
    toolbox: "TOOLBOX", update: "Update", downloadTool: "Download", downloadingTool: "Downloading…", toolDownloaded: "downloaded", aria2Backend: "aria2 (HTTP/HTTPS, FTP, torrent and magnet)", toolUpdated: "updated", toolCurrent: "already current", manualUpdateRequired: "Manual update required", mediaPlayer: "mpv / media player",
    donatePaypal: "Donate via PayPal",
    about: "About", aboutDescription: "About the creator.", aboutCreator: "Creator: Juliano - Brazil", aboutPause: "Pause", aboutPlay: "Play", aboutStop: "Stop", aboutVolume: "Volume", facebookRecordingFallback: "Facebook could not provide this Reel for direct download. Use Record on the video while it is playing.",
    overview: "OVERVIEW",
    engineReady: "Engine ready",
    addDownload: "Add download",
    downloadSpeed: "DOWNLOAD SPEED",
    uploadSpeed: "UPLOAD SPEED",
    whySlow: "Why is this slow?",
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
    bridgeStatus: "Extension bridge not configured",
    newTask: "NEW TASK",
    sourceUrl: "Source URL",
    cancel: "Cancel",
    analyze: "Analyze",
    torrentMetadataSeeking: "Finding peers and receiving torrent metadata",
    torrentMetadataPeersStalled: "Peers were reached, but none delivered the torrent metadata in time. This may be specific to this swarm or its peers. Try again or use a torrent file if available.",
    torrentMetadataTimeout: "No peers responded with metadata for this torrent/magnet in time. Trackers or DHT may be slow or the swarm may have no active seeds. You can try again, or check the link.",
    torrentMetadataNoPeers: "No peers could be reached at all for this torrent/magnet, even through DHT. This usually means outbound BitTorrent traffic (UDP) is being blocked by a firewall, VPN, or the network you're on, rather than the swarm being empty.",
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
    openFolder: "Open folder",
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
    linkConnected: "Connected",
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
    linkWriteDenied: "The remote computer has not enabled Accept writing.",
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
    websiteCredentialsLocalWarning: "Passwords are stored in the operating system credential vault and are not written to settings.json.",
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
    dnsScopeHint: "Applied to the native HTTP engine. aria2 uses the system resolver; SOCKS5H continues resolving through the proxy.",
    aria2RpcTitle: "aria2 RPC",
    aria2RpcEnabled: "Use aria2 RPC",
    aria2RpcEnabledHint: "Control accelerated HTTP/HTTPS and FTP transfers through the local aria2 engine.",
    aria2RpcAutoStart: "Start aria2 automatically when needed",
    aria2RpcAutoStartHint: "Keeps one local aria2 backend for the current Apocalipse session.",
    aria2RpcPort: "RPC port",
    aria2RpcPortHint: "Automatic",
    aria2RpcStatus: "RPC status",
    aria2RpcConnected: "Connected",
    aria2RpcDisconnected: "Disconnected",
    aria2RpcTesting: "Testing…",
    aria2RpcTest: "Test RPC connection",
    aria2RpcRegenerateToken: "Regenerate RPC token",
    aria2RpcTokenRegenerated: "RPC token regenerated",
    maxTasks: "Maximum simultaneous tasks",
    connections: "Connections per download",
    automatic: "Automatic",
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
    bridgeDisconnected: "Extension disconnected",
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
    toolsHint: "Browse for an existing executable or download the latest compatible version automatically into the portable tools folder. Downloaded paths are applied after you click Save.",
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
    archiveExtractor: "Extrator de arquivos (7-Zip / RAR / UnRAR / unar / bsdtar / tar)",
    autoExtract: "Extrair automaticamente após o download",
    autoExtractHint: "Aparece somente para arquivos compactados. Arquivos soltos ficam dentro de uma pasta com o nome do arquivo compactado.",
    browserAssistedArchiveReady: "Arquivo compactado recebido do navegador. Escolha onde salvar e se deseja extrair automaticamente.",
    networkWaiting: "Aguardando rede",
    networkWaitingHint: "A conexão mudou ou ficou offline. Esta tarefa será retomada automaticamente quando uma interface de rede estiver disponível.",
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
    aiDescription: "Assistente local que explica problemas, analisa diagnósticos e acompanha testes de correções.",
    aiLocalPrivate: "Local, privada e especializada no seu Apocalipse",
    aiCorrectionHistory: "Histórico de correções", aiCorrectionsFound: "Correções encontradas",
    aiCorrectionsHint: "As correções permanecem locais e só são confirmadas depois do seu teste manual.",
    aiDeleteAll: "Apagar todas", aiNoCorrections: "Nenhuma correção encontrada.", aiInputHint: "Pergunte sobre o Apocalipse…", aiSend: "Enviar",
    aiPrivacy: "Funciona localmente usando o estado do programa e registros de diagnóstico protegidos.",
    aiCopyName: "Copiar nome", aiDeleteCorrection: "Apagar", aiApplyCorrection: "Aplicar para teste", aiUndoCorrection: "Desfazer",
    aiStatusProposed: "Aguardando aprovação", aiStatusTesting: "Em teste", aiStatusSaved: "Guardada", aiStatusConfirmed: "Confirmada", aiStatusRejected: "Não funcionou",
    toolsPageDescription: "Gerencie os motores usados para mídia, transferências, conversão e pré-visualização.",
    settingsDescription: "Configure aparência, integrações, rede e comportamento do aplicativo.",
    toolbox: "CAIXA DE FERRAMENTAS", update: "Atualizar", downloadTool: "Baixar", downloadingTool: "Baixando…", toolDownloaded: "baixado", aria2Backend: "aria2 (HTTP/HTTPS, FTP, torrent e magnet)", toolUpdated: "atualizado", toolCurrent: "já está atualizado", manualUpdateRequired: "Atualização manual necessária", mediaPlayer: "mpv / reprodutor de mídia",
    donatePaypal: "Faça uma doação pelo PayPal",
    about: "Sobre", aboutDescription: "Sobre o criador.", aboutCreator: "Criador: Juliano - Brasil", aboutPause: "Pausar", aboutPlay: "Tocar", aboutStop: "Parar", aboutVolume: "Volume", facebookRecordingFallback: "O Facebook não disponibilizou este Reel para download direto. Use Gravar no vídeo enquanto ele estiver em reprodução.",
    overview: "VISÃO GERAL",
    engineReady: "Motor pronto",
    addDownload: "Adicionar download",
    downloadSpeed: "VELOCIDADE DE DOWNLOAD",
    uploadSpeed: "VELOCIDADE DE ENVIO",
    whySlow: "Por que está lento?",
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
    bridgeStatus: "Ponte da extensão não configurada",
    newTask: "NOVA TAREFA",
    sourceUrl: "URL de origem",
    cancel: "Cancelar",
    analyze: "Analisar",
    torrentMetadataSeeking: "Procurando pares e recebendo metadados do torrent",
    torrentMetadataPeersStalled: "Pares foram alcançados, mas nenhum entregou os metadados do torrent a tempo. Isso pode ser específico deste swarm ou de seus pares. Tente novamente ou use um arquivo .torrent, se disponível.",
    torrentMetadataTimeout: "Nenhum par respondeu com os metadados deste torrent/magnet a tempo. Os trackers ou o DHT podem estar lentos, ou a rede pode não ter seeds ativos no momento. Você pode tentar novamente ou verificar o link.",
    torrentMetadataNoPeers: "Nenhum par foi alcançado para este torrent/magnet, nem mesmo pelo DHT. Isso geralmente indica que o tráfego BitTorrent (UDP) está sendo bloqueado por um firewall, VPN ou pela rede em que você está, e não que o torrent está sem seeds.",
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
    openFolder: "Abrir pasta",
    preview: "Pré-visualizar",
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
    linkConnected: "Conectado",
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
    linkWriteDenied: "O computador remoto não ativou Aceitar gravação.",
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
    websiteCredentialsLocalWarning: "As senhas ficam no cofre de credenciais do sistema operacional e não são gravadas no settings.json.",
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
    dnsScopeHint: "Aplicado ao motor HTTP nativo. O aria2 usa a resolução do sistema; o SOCKS5H continua resolvendo pelo proxy.",
    aria2RpcTitle: "aria2 RPC",
    aria2RpcEnabled: "Usar aria2 RPC",
    aria2RpcEnabledHint: "Controla HTTP/HTTPS acelerado e FTP pelo motor aria2 local.",
    aria2RpcAutoStart: "Iniciar o aria2 automaticamente quando necessário",
    aria2RpcAutoStartHint: "Mantém um único backend aria2 local durante a sessão atual do Apocalipse.",
    aria2RpcPort: "Porta RPC",
    aria2RpcPortHint: "Automática",
    aria2RpcStatus: "Status RPC",
    aria2RpcConnected: "Conectado",
    aria2RpcDisconnected: "Desconectado",
    aria2RpcTesting: "Testando…",
    aria2RpcTest: "Testar conexão RPC",
    aria2RpcRegenerateToken: "Regenerar token RPC",
    aria2RpcTokenRegenerated: "Token RPC regenerado",
    maxTasks: "Máximo de tarefas simultâneas",
    connections: "Conexões por download",
    automatic: "Automático",
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
    bridgeDisconnected: "Extensão desconectada",
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
    toolsHint: "Procure um executável existente ou baixe automaticamente a versão compatível mais recente para a pasta portátil tools. Os caminhos baixados passam a valer depois de clicar em Salvar.",
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
    archiveExtractor: "压缩文件解压工具（7-Zip / RAR / UnRAR / unar / bsdtar / tar）",
    autoExtract: "下载完成后自动解压",
    autoExtractHint: "仅在压缩文件时显示。根目录中的零散文件会解压到以压缩文件命名的文件夹中。",
    browserAssistedArchiveReady: "已从浏览器接收压缩文件。请选择保存位置以及是否自动解压。",
    networkWaiting: "等待网络",
    networkWaitingHint: "网络连接已更改或断开。可用网络接口恢复后，此任务会自动继续。",
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
    aiDescription: "本地助手，可解释问题、分析诊断并跟踪修正测试。",
    aiLocalPrivate: "本地、私密，专用于你的 Apocalipse",
    aiCorrectionHistory: "修正历史", aiCorrectionsFound: "发现的修正",
    aiCorrectionsHint: "修正保存在本地，只有在你手动测试后才会被确认。",
    aiDeleteAll: "全部删除", aiNoCorrections: "没有发现修正。", aiInputHint: "询问有关 Apocalipse 的问题…", aiSend: "发送",
    aiPrivacy: "使用应用状态和经过隐私保护的诊断记录在本地运行。",
    aiCopyName: "复制名称", aiDeleteCorrection: "删除", aiApplyCorrection: "应用测试", aiUndoCorrection: "撤销",
    aiStatusProposed: "等待批准", aiStatusTesting: "测试中", aiStatusSaved: "已保存", aiStatusConfirmed: "已确认", aiStatusRejected: "未解决",
    toolsPageDescription: "管理媒体、传输、转换和预览所使用的引擎。",
    settingsDescription: "配置外观、集成、网络和应用行为。",
    toolbox: "工具箱", update: "更新", downloadTool: "下载", downloadingTool: "正在下载…", toolDownloaded: "已下载", aria2Backend: "aria2（HTTP/HTTPS、FTP、种子和磁力链接）", toolUpdated: "已更新", toolCurrent: "已是最新版本", manualUpdateRequired: "需要手动更新", mediaPlayer: "mpv / 媒体播放器",
    donatePaypal: "通过 PayPal 捐赠",
    about: "关于", aboutDescription: "关于创作者。", aboutCreator: "创作者：Juliano - 巴西", aboutPause: "暂停", aboutPlay: "播放", aboutStop: "停止", aboutVolume: "音量", facebookRecordingFallback: "Facebook 无法提供此 Reel 的直接下载。请在视频播放时使用“录制”。",
    overview: "概览",
    engineReady: "引擎已就绪",
    addDownload: "添加下载",
    downloadSpeed: "下载速度",
    uploadSpeed: "上传速度",
    whySlow: "为什么这么慢？",
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
    bridgeStatus: "扩展桥接尚未配置",
    newTask: "新任务",
    sourceUrl: "来源网址",
    cancel: "取消",
    analyze: "分析",
    torrentMetadataSeeking: "正在查找节点并接收种子元数据",
    torrentMetadataPeersStalled: "已连接到节点，但没有节点及时传送种子元数据。这可能与该种子群或其节点有关。请重试，或使用可用的 .torrent 文件。",
    torrentMetadataTimeout: "没有节点及时返回这个种子/磁力链接的元数据。Tracker 或 DHT 可能响应缓慢，或者该网络目前没有活跃的做种者。您可以重试，或检查链接是否正确。",
    torrentMetadataNoPeers: "完全没有连接到任何节点，DHT 也没有。这通常意味着 BitTorrent 流量（UDP）被防火墙、VPN 或您所在的网络屏蔽了，而不是没有做种者。",
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
    openFolder: "打开文件夹",
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
    linkConnected: "已连接",
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
    linkWriteDenied: "远程电脑尚未启用接受写入。",
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
    websiteCredentialsLocalWarning: "密码保存在操作系统凭据保险库中，不会写入 settings.json。",
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
    dnsScopeHint: "应用于原生 HTTP 引擎。aria2 使用系统解析器；SOCKS5H 仍通过代理解析。",
    aria2RpcTitle: "aria2 RPC",
    aria2RpcEnabled: "使用 aria2 RPC",
    aria2RpcEnabledHint: "通过本地 aria2 引擎控制加速 HTTP/HTTPS 和 FTP 传输。",
    aria2RpcAutoStart: "需要时自动启动 aria2",
    aria2RpcAutoStartHint: "当前 Apocalipse 会话只保留一个本地 aria2 后端。",
    aria2RpcPort: "RPC 端口",
    aria2RpcPortHint: "自动",
    aria2RpcStatus: "RPC 状态",
    aria2RpcConnected: "已连接",
    aria2RpcDisconnected: "未连接",
    aria2RpcTesting: "测试中…",
    aria2RpcTest: "测试 RPC 连接",
    aria2RpcRegenerateToken: "重新生成 RPC 令牌",
    aria2RpcTokenRegenerated: "RPC 令牌已重新生成",
    maxTasks: "最大同时任务数",
    connections: "每个下载的连接数",
    automatic: "自动",
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
    bridgeDisconnected: "扩展已断开连接",
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
    toolsHint: "可选择现有可执行文件，或自动将最新兼容版本下载到便携式 tools 文件夹。下载后的路径会在点击“保存”后生效。",
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
window.apocalipseCatalogs = catalogs;

let locale = localStorage.getItem("apocalipse.language") || "en";
const valid = ["void", "nebula", "ember", "jade", "plasma", "glacier", "amber", "abyss", "rust", "venom", "wine", "linen", "sky", "blossom", "sage", "sand", "lilac", "mist", "citrus", "coral", "frost"];
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
let taskConnectionsManuallyChanged = false;
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
const t = (key) => catalogs[locale]?.[key] || catalogs.en[key] || key;
const tf = (key, values) => Object.entries(values).reduce((text, [name, value]) => text.replaceAll(`{${name}}`, value), t(key));
const descriptions = { downloads: "downloadsDescription", recordings: "recordingsDescription", torrents: "torrentsDescription", link: "linkDescription", ai: "aiDescription", logs: "logsDescription", themes: "themesDescription", language: "languageDescription", about: "aboutDescription", settings: "settingsDescription", tools: "toolsPageDescription" };
let lastUiInteractionTrace = null;
const freshUiTrace = () => {
  const now = performance.now();
  if (lastUiInteractionTrace && now - lastUiInteractionTrace.at < 2000) return lastUiInteractionTrace.id;
  return crypto.randomUUID();
};
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
invoke("set_application_theme", { theme: localStorage.getItem("apocalipse.theme") || "void" }).catch(console.error);
invoke("get_app_version").then((version) => {
  document.querySelector("#app-version").textContent = `v${version}`;
}).catch(() => {});
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
    // External engines can report byte counters at a different cadence from
    // their live throughput. Prefer the explicit engine speed while active and
    // use byte deltas as a second signal for a responsive ADM display.
    const externalSpeed = active ? Number(task.download_speed) || 0 : 0;
    let speed = active ? previous?.speed || 0 : 0;
    if (previous && active) {
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
    speedSamples.set(task.id, {
      at: now,
      bytes: task.received,
      speed: externalSpeed || speed,
      changedAt,
    });
    if (active) {
      overallSpeed += externalSpeed || speed;
      overallUploadSpeed += Number(task.upload_speed) || 0;
    }
  }
}

function visibleDownloads() {
  const isRecording = (task) => /\.recording\.webm$/i.test(`${task.source} ${task.destination}`);
  const isTorrent = (task) => /^(?:magnet:)|\.torrent(?:$|[?#])/i.test(task.source);
  let visible = activePage === "recordings"
    ? downloads.filter(isRecording)
    : activePage === "torrents"
      ? downloads.filter(isTorrent)
      : downloads.filter((task) => !isRecording(task) && !isTorrent(task));
  if (activePage === "link") visible = visible.filter((task) => /^(?:ftp|sftp):/i.test(task.source));
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
      console.warn("thumbnail-cache", error);
      thumbnailRetryAfter.set(url, Date.now() + 60_000);
      return null;
    })
    .finally(() => thumbnailPending.delete(url));
  thumbnailPending.set(url, pending);
  return pending;
}

function loadPreviewThumbnail(image, url) {
  image.dataset.thumbnailSource = url || "";
  image.hidden = true;
  image.removeAttribute("src");
  if (!url) return;
  resolveCachedThumbnail(url).then((resolved) => {
    if (!resolved || image.dataset.thumbnailSource !== url) return;
    image.src = resolved;
    image.hidden = false;
    image.onerror = () => {
      if (image.dataset.thumbnailSource !== url) return;
      image.hidden = true;
      image.removeAttribute("src");
      thumbnailDataCache.delete(url);
      thumbnailRetryAfter.set(url, Date.now() + 60_000);
    };
  });
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
  list.replaceChildren();
  list.hidden = visible.length === 0;
  for (const task of visible) {
    const row = document.createElement("article");
    row.className = "download-row";
    row.dataset.taskId = task.id;
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
        console.error(error);
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
    if (task.thumbnail) {
      const requestedThumbnail = task.thumbnail;
      resolveCachedThumbnail(requestedThumbnail).then((resolved) => {
        if (!resolved || !row.isConnected) return;
        const thumbnail = document.createElement("img");
        thumbnail.className = "download-thumbnail";
        thumbnail.alt = "";
        thumbnail.src = resolved;
        thumbnail.onerror = () => {
          thumbnailDataCache.delete(requestedThumbnail);
          thumbnailRetryAfter.set(requestedThumbnail, Date.now() + 60_000);
          icon.replaceChildren(document.createTextNode("⇩"));
          icon.classList.remove("has-thumbnail");
        };
        icon.replaceChildren(thumbnail);
        icon.classList.add("has-thumbnail");
      });
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
    const uploadSpeed = task.state === "downloading" ? Number(task.upload_speed) || 0 : 0;
    const progressText = hasReportedPercent && !task.total
      ? `${percent.toFixed(1)}%`
      : task.total
      ? `${formatBytes(task.received)} / ${formatBytes(task.total)} · ${percent.toFixed(1)}%`
      : formatBytes(task.received);
    const webSeedStats = task.torrent_web_seeds ? ` · +${task.torrent_web_seeds} ${t("webMirror")}` : "";
    const torrentStats = task.torrent_seeders !== null && task.torrent_seeders !== undefined
      ? ` · S:${task.torrent_seeders} L:${task.torrent_leechers || 0}${task.torrent_eta ? ` · ETA ${task.torrent_eta}` : ""}${webSeedStats}` : "";
    details.textContent =
      speed && task.state === "downloading"
        ? `${progressText} · ↓ ${formatBytes(speed)}/s · ↑ ${formatBytes(uploadSpeed)}/s${torrentStats}`
        : `${progressText}${torrentStats}`;
    progress.append(bar);
    info.append(progress, details);
    if (task.state === "downloading") {
      const whySlowLink = document.createElement("a");
      whySlowLink.href = "#";
      whySlowLink.className = "why-slow-link";
      whySlowLink.textContent = t("whySlow");
      const explanation = document.createElement("small");
      explanation.className = "why-slow-explanation";
      explanation.hidden = true;
      whySlowLink.onclick = async (event) => {
        event.preventDefault();
        if (!explanation.hidden) {
          explanation.hidden = true;
          return;
        }
        explanation.hidden = false;
        explanation.textContent = "…";
        try {
          const engineEvents = await invoke("read_ai_diagnostics");
          explanation.textContent = window.ApocalipseAI.performanceDiagnosis({ engineEvents }, locale, task.id);
        } catch (error) {
          explanation.textContent = String(error);
        }
      };
      info.append(whySlowLink, explanation);
    }
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
    const failureMessage = typeof task.state === "object" ? task.state.failed?.message || "" : "";
    const state = Object.assign(document.createElement("span"), {
      className: "download-state",
      textContent: failureMessage === "network_waiting_for_reconnect"
        ? t("networkWaiting")
        : /\.recording\.webm$/i.test(task.destination) && stateKey(task.state) === "downloading"
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
          console.error(error);
          window.alert(String(error));
        } finally {
          busyIds.delete(task.id);
          locateButton.disabled = false;
        }
      };
      actions.append(locateButton);
    }
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
  if (!taskConnectionsManuallyChanged) {
    document.querySelector("#task-connections-value").value = t("automatic");
  }
  const activeNavigation = document.querySelector(`nav [data-page="${activePage}"]`);
  if (activeNavigation) document.querySelector("main > header h1").textContent = activeNavigation.querySelector("b")?.textContent || t("downloads");
  document.querySelector("#page-description").textContent = t(descriptions[activePage] || "downloadsDescription");
  renderDownloads(true);
  if (activePage === "link") {
    document.querySelector("#link-local-path").textContent = linkLocalPath || t("linkDrives");
    document.querySelector("#link-remote-path").textContent = linkRemotePath || t("linkDrives");
  }
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
    console.error(error);
  }
}

function acceptEnqueuedTask(task) {
  downloadListState.invalidate();
  downloads = downloadListState.visible([...downloads.filter((item) => item.id !== task.id), task]);
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
        acceptEnqueuedTask(await invoke("enqueue_download", { url, destinationDirectory, fileName, formatSelection: null, torrentSelection: null, mirrors: null, priority: 0, bandwidthLimit: null, connectionsOverride: null, context: {} }));
      } catch (error) { console.warn("import", url, error); }
    }
    renderDownloads();
  } catch (error) { console.error(error); }
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

loadAboutMedia().catch(console.error);

document.querySelectorAll('nav [data-page]:not([data-page="settings"]):not([data-page="tools"])').forEach((button) => {
  button.onclick = () => {
    const openedAt = performance.now();
    if (button.dataset.page === "link") {
      invoke("open_link_window").catch((error) => window.alert(String(error)));
      invoke("record_ui_diagnostic", { level: "INFO", event: "link_window_requested", detail: "source=main_navigation" }).catch(() => {});
      return;
    }
    activePage = button.dataset.page;
    document.querySelectorAll("nav [data-page]").forEach((item) => item.classList.toggle("active", item === button));
    const heading = button.querySelector("b")?.textContent || t("downloads");
    document.querySelector("header h1").textContent = heading;
    document.querySelector("#page-description").textContent = t(descriptions[activePage] || "downloadsDescription");
    document.querySelector("#apocalipse-link-panel").hidden = activePage !== "link";
    document.querySelector("#ai-panel").hidden = activePage !== "ai";
    document.querySelector("#logs-panel").hidden = activePage !== "logs";
    document.querySelector("#themes-panel").hidden = activePage !== "themes";
    document.querySelector("#language-panel").hidden = activePage !== "language";
    document.querySelector("#about-panel").hidden = activePage !== "about";
    document.querySelector("#add").hidden = activePage === "about";
    if (activePage === "about" && aboutAudio.src) {
      aboutAudio.currentTime = 0;
      aboutAudio.play().catch(() => {});
    } else {
      aboutAudio.pause();
      aboutAudio.currentTime = 0;
    }
    document.querySelector(".metrics").hidden = ["link", "ai", "logs", "themes", "language", "about"].includes(activePage);
    document.querySelector(".panel").hidden = ["link", "ai", "logs", "themes", "language", "about"].includes(activePage);
    renderDownloads();
    invoke("record_ui_diagnostic", { level: "INFO", event: "page_opened", detail: `page=${activePage} panel_present=${activePage === "link" ? Boolean(document.querySelector("#apocalipse-link-panel")) : activePage === "logs" ? Boolean(document.querySelector("#logs-panel")) : true} duration_ms=${Math.round(performance.now() - openedAt)}` }).catch(() => {});
    if (activePage === "logs") refreshLogEvents().catch(console.error);
    if (activePage === "ai") window.dispatchEvent(new CustomEvent("apocalipse-ai-opened"));
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
let linkRemoteTransportToken = "";
let linkLocalIdentity = "";
let linkLocalAccountSession = false;
let linkSelectedLocal = null;
let linkSelectedRemote = null;
let linkRemoteAllowWrite = false;
// Lets a remote file listed here stand in for a local paused/failed/queued
// download with the same file name, so the user can fill it from a paired
// Link peer instead of waiting on the internet.
function linkMatchingDownload(fileName) {
  const lower = String(fileName || "").toLowerCase();
  if (!lower) return null;
  return downloads.find((task) => {
    const key = typeof task.state === "string" ? task.state : Object.keys(task.state || {})[0];
    if (!["paused", "failed", "queued"].includes(key) || isTorrent(task)) return false;
    const base = String(task.destination || "").split(/[\\/]/).pop();
    return base && base.toLowerCase() === lower;
  }) || null;
}
const linkParent = (path) => /^[A-Za-z]:[\\/]?$/.test(path) || /^\/shares\/[^/]+\/?$/.test(path) ? "" : path.replace(/[\\/]+$/, "").replace(/[\\/][^\\/]*$/, "");
function linkHost(value) {
  const authority = String(value || "").trim().replace(/^https?:\/\//i, "").split(/[/?#]/)[0];
  if (!authority) return "";
  if (authority.startsWith("[")) {
    const end = authority.indexOf("]");
    return (end > 0 ? authority.slice(1, end) : authority).toLowerCase();
  }
  if (authority === "::1" || (authority.match(/:/g) || []).length > 1) return authority.toLowerCase();
  return authority.split(":")[0].toLowerCase();
}
async function isLocalLinkTarget(value) {
  const host = linkHost(value);
  const ownHost = linkHost(linkLocalIdentity);
  if (host === "127.0.0.1" || host === "localhost" || host === "::1" || Boolean(ownHost && host === ownHost)) return true;
  return invoke("is_local_link_target", { id: value });
}
function updateLinkTransferButtons() {
  document.querySelector("#link-upload-local").disabled = !linkSelectedLocal || !linkRemoteId || !linkRemotePath || !linkRemoteAllowWrite;
  document.querySelector("#link-download-remote").disabled = !linkSelectedRemote;
  document.querySelector("#link-delete-remote").disabled = !linkSelectedRemote || !linkRemoteAllowWrite;
  document.querySelector("#link-disconnect").disabled = !linkRemoteId;
}
function disconnectLink() {
  linkRemoteId = ""; linkRemoteTransportToken = ""; linkLocalAccountSession = false;
  linkRemotePath = ""; linkSelectedRemote = null; linkRemoteAllowWrite = false;
  document.querySelector("#link-remote-password").value = "";
  document.querySelector("#link-remote-path").textContent = "/";
  document.querySelector("#link-remote-files").replaceChildren();
  document.querySelector("#link-status").textContent = t("linkDisconnected");
  updateLinkTransferButtons();
}
function renderLinkFiles(target, entries, open, select, matchDownloads = false) {
  const root = document.querySelector(target);
  root.replaceChildren();
  if (!entries.length) {
    const empty = document.createElement("small");
    empty.className = "link-empty";
    empty.textContent = t("linkNoShares");
    root.append(empty);
    return;
  }
  for (const entry of entries) {
    const row = document.createElement("div");
    row.className = "link-file";
    row.tabIndex = 0;
    row.setAttribute("role", "button");
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
    row.onkeydown = (event) => {
      if (event.key === "Enter" || event.key === " ") {
        event.preventDefault();
        row.onclick();
      }
    };
    if (matchDownloads && !entry.directory) {
      const match = linkMatchingDownload(entry.name);
      if (match) {
        const useButton = document.createElement("button");
        useButton.type = "button";
        useButton.className = "link-file-use-for-download";
        useButton.textContent = t("linkUseForDownload");
        useButton.title = tf("linkUseForDownloadHint", { name: match.display_title || entry.name });
        useButton.onclick = async (event) => {
          event.stopPropagation();
          useButton.disabled = true;
          const status = document.querySelector("#link-status");
          status.textContent = t("linkTransferring");
          try {
            await invoke("use_remote_link_file_for_download", {
              taskId: match.id,
              id: linkRemoteId,
              password: linkRemoteTransportToken,
              path: entry.path,
            });
            status.textContent = tf("linkUseForDownloadCompleted", { name: match.display_title || entry.name });
            await refreshDownloads();
            renderLinkFiles(target, entries, open, select, matchDownloads);
          } catch (error) {
            if (`${error}` !== "cancelled") status.textContent = `${t("linkTransferFailed")}: ${error}`;
            useButton.disabled = false;
          }
        };
        row.append(useButton);
      }
    }
    root.append(row);
  }
}
async function openLocalLink(path = "") {
  linkLocalPath = path;
  linkSelectedLocal = null;
  updateLinkTransferButtons();
  document.querySelector("#link-local-path").textContent = path || t("linkDrives");
  renderLinkFiles("#link-local-files", await invoke("list_local_link_files", { path }), openLocalLink, (entry) => {
    linkSelectedLocal = entry;
    updateLinkTransferButtons();
  });
}
async function openRemoteLink(path = "") {
  linkRemotePath = path;
  linkSelectedRemote = null;
  updateLinkTransferButtons();
  document.querySelector("#link-remote-path").textContent = path || t("linkDrives");
  const capabilities = linkLocalAccountSession
    ? await invoke("get_local_link_share_capabilities", { path })
    : await invoke("get_remote_link_capabilities", { id: linkRemoteId, password: linkRemoteTransportToken, path });
  linkRemoteAllowWrite = Boolean(capabilities.allowWrite);
  updateLinkTransferButtons();
  const entries = linkLocalAccountSession
    ? await invoke("list_local_link_files", { path })
    : await invoke("list_remote_link_files", { id: linkRemoteId, password: linkRemoteTransportToken, path });
  renderLinkFiles("#link-remote-files", entries, openRemoteLink, (entry) => {
    linkSelectedRemote = entry;
    updateLinkTransferButtons();
  }, !linkLocalAccountSession);
}
async function loadLinkIdentity() {
  const identity = await invoke("get_link_identity");
  linkLocalIdentity = identity.id;
  document.querySelector("#link-own-id").value = identity.id;
  renderLinkShares(await invoke("list_link_shares"));
  await openLocalLink();
}
// Link now opens in its own maximized window. The embedded panel stays available
// as a fallback for tests and future recovery paths, but main navigation does not load it.
async function refreshVisibleLinkPanels({ resetToRoot = false } = {}) {
  const localPath = resetToRoot ? "" : linkLocalPath;
  const remotePath = resetToRoot ? "" : linkRemotePath;
  await openLocalLink(localPath).catch(() => openLocalLink(""));
  if (linkRemoteId) await openRemoteLink(remotePath).catch(() => openRemoteLink(""));
}
function renderLinkShares(shares) {
  const root = document.querySelector("#link-share-list"); root.replaceChildren();
  for (const share of shares) {
    const row = document.createElement("div");
    const name = Object.assign(document.createElement("b"), { textContent: share.name });
    const permission = document.createElement("select");
    permission.append(new Option(t("linkReadOnly"), "false"), new Option(t("linkReadWrite"), "true"));
    permission.value = String(Boolean(share.allowWrite));
    permission.onchange = async () => { renderLinkShares(await invoke("update_link_share", { id: share.id, allowWrite: permission.value === "true" })); await refreshVisibleLinkPanels({ resetToRoot: true }); };
    const remove = Object.assign(document.createElement("button"), { type: "button", textContent: t("linkStopSharing") });
    remove.onclick = async () => { renderLinkShares(await invoke("remove_link_share", { id: share.id })); await refreshVisibleLinkPanels({ resetToRoot: true }); };
    row.append(name, permission, remove); root.append(row);
  }
}
document.querySelector("#link-share-file").onclick = async () => { try { renderLinkShares(await invoke("add_link_file_share")); await refreshVisibleLinkPanels({ resetToRoot: true }); } catch (error) { if (`${error}` !== "cancelled") window.alert(String(error)); } };
document.querySelector("#link-share-folder").onclick = async () => { try { renderLinkShares(await invoke("add_link_share")); await refreshVisibleLinkPanels({ resetToRoot: true }); } catch (error) { if (`${error}` !== "cancelled") window.alert(String(error)); } };
document.querySelector("#link-connect").onclick = async () => {
  const id = document.querySelector("#link-remote-id").value.trim();
  const username = document.querySelector("#link-remote-username").value.trim();
  const passwordField = document.querySelector("#link-remote-password");
  const status = document.querySelector("#link-status");
  if (!id) {
    status.textContent = t("linkCredentialsRequired");
    return;
  }
  linkRemoteId = "";
  linkRemoteTransportToken = "";
  linkLocalAccountSession = false;
  try {
    if (await isLocalLinkTarget(id)) {
      linkRemoteId = id;
      linkLocalAccountSession = true;
      await openRemoteLink("");
      status.textContent = t("linkLocalSessionReady");
      return;
    }
    if (!username || !passwordField.value) {
      status.textContent = t("linkCredentialsRequired");
      return;
    }
    status.textContent = t("linkAuthenticating");
    const session = await invoke("authenticate_remote_link_account", {
      id,
      username,
      password: passwordField.value,
    });
    linkRemoteId = id;
    linkRemoteTransportToken = session.token;
    linkLocalAccountSession = false;
    await openRemoteLink("");
    status.textContent = session.firstTrust
      ? `${t("linkRemoteSessionReady")} ${t("linkRemoteFirstTrust")} ${session.fingerprint}`
      : t("linkRemoteSessionReady");
  } catch (error) {
    linkRemoteId = "";
    linkRemoteTransportToken = "";
    linkLocalAccountSession = false;
    const value = String(error);
    status.textContent = value.includes("remote_system_auth_failed")
      ? t("linkAuthenticationInvalidCredentials")
      : value.includes("link_tls_certificate_changed")
        ? `${t("linkConnectionFailed")}: TLS certificate changed`
        : `${t("linkConnectionFailed")}: ${value}`;
  } finally {
    passwordField.value = "";
    updateLinkTransferButtons();
  }
};
document.querySelector("#link-local-up").onclick = () => openLocalLink(linkParent(linkLocalPath)).catch(console.error);
document.querySelector("#link-disconnect").onclick = disconnectLink;
document.querySelector("#link-remote-up").onclick = () => openRemoteLink(linkParent(linkRemotePath)).catch(console.error);
document.querySelector("#link-delete-remote").onclick = async () => {
  if (!linkSelectedRemote || !window.confirm(t("linkDeleteConfirm").replace("{name}", linkSelectedRemote.name))) return;
  if (linkLocalAccountSession) {
    await invoke("delete_local_shared_link_item", { path: linkSelectedRemote.path });
  } else {
    await invoke("delete_remote_link_item", { id: linkRemoteId, password: linkRemoteTransportToken, path: linkSelectedRemote.path });
  }
  await openRemoteLink(linkRemotePath);
};
document.querySelector("#link-download-remote").onclick = async () => {
  if (!linkSelectedRemote) return;
  const status = document.querySelector("#link-status");
  status.textContent = t("linkTransferring");
  try {
    const destination = linkLocalAccountSession
      ? await invoke("download_local_shared_link_item", {
          path: linkSelectedRemote.path,
          directory: linkSelectedRemote.directory,
          fileName: linkSelectedRemote.name,
        })
      : await invoke("download_remote_link_file", {
          id: linkRemoteId,
          password: linkRemoteTransportToken,
          path: linkSelectedRemote.path,
          directory: linkSelectedRemote.directory,
          fileName: linkSelectedRemote.name,
        });
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
    const remotePath = linkLocalAccountSession
      ? await invoke("upload_local_shared_link_item", {
          remoteDirectory: linkRemotePath,
          localPath: linkSelectedLocal.path,
        })
      : await invoke("upload_remote_link_file", {
          id: linkRemoteId,
          password: linkRemoteTransportToken,
          remoteDirectory: linkRemotePath,
          localPath: linkSelectedLocal.path,
        });
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
  thumbnail.dataset.thumbnailSource = "";
  thumbnail.removeAttribute("src");
  document.querySelector("#media-title").textContent = "";
  document.querySelector("#media-duration").textContent = "";
  document.querySelector("#media-format").replaceChildren();
  document.querySelector("#media-format").hidden = false;
  document.querySelector("#hls-audio-conversion").hidden = true;
  document.querySelector("#hls-convert-audio").checked = false;
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
  loadPreviewThumbnail(image, thumbnail);
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
function resetTaskConnections() {
  taskConnectionsManuallyChanged = false;
  document.querySelector("#task-connections").value = "16";
  document.querySelector("#task-connections-value").value = t("automatic");
}
document.querySelectorAll("#add").forEach(
  (button) =>
    (button.onclick = () => {
      document.querySelector("#analysis").hidden = true;
      document.querySelector("#enqueue").hidden = true;
      document.querySelector("#analyze").hidden = false;
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
      resetTaskConnections();
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
  window.dispatchEvent(new CustomEvent("apocalipse-language-changed", { detail: { language: locale } }));
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
async function removeSelectedDownloads(button, deleteFiles) {
  button.disabled = true;
  const ids = [...selectedIds];
  downloadListState.beginRemoval(ids);
  let removed = false;
  try {
    await invoke("remove_downloads", { ids, deleteFiles });
    removed = true;
    downloadListState.finishRemoval(ids, true);
    downloads = downloadListState.visible(downloads);
    for (const id of ids) selectedIds.delete(id);
    renderDownloads(true);
    clearDialog.close();
    await refreshDownloads();
  } catch (error) {
    console.error(error);
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
        console.error(error);
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
    console.error(error);
    window.alert(String(error));
  } finally {
    button.disabled = false;
  }
};

function renderAria2RpcStatus(status = {}) {
  const target = document.querySelector("#aria2-rpc-status");
  if (!target) return;
  if (!status.connected) {
    target.textContent = t("aria2RpcDisconnected");
    return;
  }
  const details = [
    t("aria2RpcConnected"),
    status.activePort ? `127.0.0.1:${status.activePort}` : "",
    status.version ? `aria2 ${status.version}` : "",
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
      invoke("get_aria2_rpc_settings"),
      invoke("get_associations"),
      invoke("list_host_rules"),
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
    document.querySelector("#aria2-rpc-enabled").checked = rpc.enabled;
    document.querySelector("#aria2-rpc-auto-start").checked = rpc.autoStart;
    document.querySelector("#aria2-rpc-port").value = rpc.configuredPort || "";
    renderAria2RpcStatus(rpc);
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
document.querySelector("#aria2-rpc-test").onclick = async () => {
  const status = document.querySelector("#aria2-rpc-status");
  const button = document.querySelector("#aria2-rpc-test");
  button.disabled = true;
  status.textContent = t("aria2RpcTesting");
  try {
    const rpcPortValue = Number(document.querySelector("#aria2-rpc-port").value) || 0;
    await invoke("set_aria2_rpc_settings", {
      enabled: document.querySelector("#aria2-rpc-enabled").checked,
      autoStart: document.querySelector("#aria2-rpc-auto-start").checked,
      port: rpcPortValue > 0 ? rpcPortValue : null,
    });
    renderAria2RpcStatus(await invoke("test_aria2_rpc"));
  } catch (error) {
    status.textContent = `${t("aria2RpcDisconnected")} · ${error}`;
  } finally {
    button.disabled = false;
  }
};
document.querySelector("#aria2-rpc-regenerate-token").onclick = async () => {
  const button = document.querySelector("#aria2-rpc-regenerate-token");
  button.disabled = true;
  try {
    await invoke("regenerate_aria2_rpc_token");
    document.querySelector("#aria2-rpc-status").textContent = t("aria2RpcTokenRegenerated");
  } catch (error) {
    console.error(error);
    window.alert(String(error));
  } finally {
    button.disabled = false;
  }
};
document.querySelector("#theme").onchange = (event) => {
  localStorage.setItem("apocalipse.theme", event.target.value);
  applyTheme(event.target.value);
  invoke("set_application_theme", { theme: event.target.value }).catch(console.error);
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
    await invoke("set_application_theme", { theme });
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
    const rpcPortValue = Number(document.querySelector("#aria2-rpc-port").value) || 0;
    await invoke("set_aria2_rpc_settings", {
      enabled: document.querySelector("#aria2-rpc-enabled").checked,
      autoStart: document.querySelector("#aria2-rpc-auto-start").checked,
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
      extractor: document.querySelector("#tool-extractor").value,
    });
    await invoke("set_media_player", { path: document.querySelector("#tool-player").value });
    toolsDialog.close();
  } catch (error) { console.error(error); }
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
      if (updated) alert(`${updated[1]} · ${t("toolUpdated")}: ${updated[2]} → ${updated[3]}`);
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
function updateLimitLabels() {
  document.querySelector("#max-tasks-value").value = document.querySelector("#max-tasks").value;
  document.querySelector("#connections-value").value = document.querySelector("#connections").value;
}
document.querySelector("#max-tasks").oninput = updateLimitLabels;
document.querySelector("#connections").oninput = updateLimitLabels;
document.querySelector("#default-limits").onclick = () => {
  document.querySelector("#max-tasks").value = 3;
  document.querySelector("#connections").value = 16;
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
function isArchiveFileName(name) {
  return /\.(zip|7z|rar|tar|tar\.gz|tgz|tar\.bz2|tbz2|tar\.xz|txz|tar\.zst|gz|bz2|xz|zst|cab|arj|lha|lzh)$/i.test(String(name || ""));
}
function refreshAutoExtractOption() {
  const option = document.querySelector("#auto-extract-option");
  const archive = isArchiveFileName(document.querySelector("#file-name").value);
  option.hidden = !archive;
  if (!archive) document.querySelector("#auto-extract").checked = false;
}
document.querySelector("#file-name").addEventListener("input", refreshAutoExtractOption);
document.querySelector("#url").oninput = () => {
  document.querySelector("#analysis").hidden = true;
  document.querySelector("#enqueue").hidden = true;
  document.querySelector("#analyze").hidden = false;
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

async function showMediaInspection(url) {
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
    pendingTitle = media.title || pendingTitle;
    pendingThumbnail = media.thumbnail || pendingThumbnail;
    pendingDuration = Number.isFinite(media.duration) ? media.duration : pendingDuration;
    pendingIsLive = media.isLive === true;
    document.querySelector("#media-title").textContent = media.title;
    document.querySelector("#media-duration").textContent = media.duration ? `${t("duration")}: ${secondsLabel(media.duration)}` : "";
    const thumbnail = document.querySelector("#media-thumbnail");
    thumbnail.hidden = !media.thumbnail;
    if (media.thumbnail) thumbnail.src = media.thumbnail;
    else thumbnail.removeAttribute("src");
    for (const format of media.formats) option(select, format.selection, format.label);
    document.querySelector("#file-name").value = media.suggestedFileName;
    refreshAutoExtractOption();
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
}
document.querySelector("#hls-convert-audio").onchange = updateHlsAudioConversion;
document.querySelector("#hls-audio-format").onchange = updateHlsAudioConversion;
document.querySelector("#task-connections").oninput = (event) => {
  taskConnectionsManuallyChanged = true;
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
    if (pendingBrowserAssistedPath) {
      const fileName = document.querySelector("#file-name");
      box.textContent = t("browserAssistedArchiveReady");
      refreshAutoExtractOption();
      document.querySelector("#analyze").hidden = true;
      document.querySelector("#enqueue").hidden = false;
      return;
    }
    try {
      const hostResolution = await invoke("resolve_file_host_url", { url: url.value });
      if (hostResolution?.adapted && hostResolution.url) {
        url.value = hostResolution.url;
      }
    } catch (error) {
      console.warn("file-host-adapter", error);
    }
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
    else if (/^magnet:/i.test(url.value) || /\.torrent$/i.test(url.value.split(/[?#]/)[0])) {
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
    else if (plan.reason === "hls_manifest") {
      const select = document.querySelector("#media-format");
      select.replaceChildren();
      option(select, "original", pendingMediaKind === "audio" ? "Original (MP4/M4A)" : t("bestQuality"));
      for (const format of ["mp3", "m4a", "opus", "flac", "wav"])
        option(select, `audio:${format}`, `${t("audioOnly")} · ${format.toUpperCase()}`);
      const audioHls = pendingMediaKind === "audio";
      select.hidden = audioHls;
      document.querySelector("#hls-audio-conversion").hidden = !audioHls;
      document.querySelector("#hls-convert-audio").checked = false;
      updateHlsAudioConversion();
      showCapturedPreview({ title: pendingTitle || "HLS", thumbnail: pendingThumbnail, kind: "M3U8 / HLS", duration: pendingDuration, size: pendingExpectedSize, showFormats: true });
    } else if (pendingMediaKind === "image" || /\.(?:avif|bmp|gif|jpe?g|png|svg|webp)(?:$|[?#])/i.test(url.value)) {
      showCapturedPreview({ title: pendingTitle || fileName.value, thumbnail: pendingThumbnail || url.value, kind: pendingMediaKind || "image", duration: null, size: pendingExpectedSize });
    }
    refreshAutoExtractOption();
    document.querySelector("#analyze").hidden = true;
    document.querySelector("#enqueue").hidden = false;
  } catch (error) {
    const message = String(error);
    box.textContent = /connections=0:seeders=0/.test(message)
      ? t("torrentMetadataNoPeers")
      : /aria2_metadata_timeout:connections=[1-9][0-9]*/.test(message) ? t("torrentMetadataPeersStalled")
      : /timed? ?out|timeout/i.test(message) ? t("torrentMetadataTimeout") : message;
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
    const autoExtract = document.querySelector("#auto-extract-option").hidden ? false : document.querySelector("#auto-extract").checked;
    const acceptedTask = pendingBrowserAssistedPath
      ? await invoke("import_browser_assisted_download", {
          localPath: pendingBrowserAssistedPath,
          url: url.value,
          destinationDirectory: document.querySelector("#destination").value,
          fileName: document.querySelector("#file-name").value,
          autoExtract,
        })
      : await invoke("enqueue_download", {
        url: url.value,
        destinationDirectory: document.querySelector("#destination").value,
        fileName: document.querySelector("#file-name").value,
        formatSelection: document.querySelector("#media-inspection").hidden || document.querySelector("#media-format-control").hidden ? null : document.querySelector("#media-format").value,
        torrentSelection,
        mirrors: document.querySelector("#mirrors").value.split(/\r?\n/).map((value) => value.trim()).filter(Boolean),
        priority: Number(document.querySelector("#priority").value),
        bandwidthLimit: Math.round((Number(document.querySelector("#download-bandwidth-limit").value) || 0) * 1024 * 1024) || null,
        connectionsOverride: taskConnectionsManuallyChanged
          ? Number(document.querySelector("#task-connections").value) || 16
          : null,
        autoExtract,
        context: {
          traceId: pendingDiagnosticTrace,
          referer: pendingReferer,
          knownDuration: pendingDuration,
          isLive: pendingIsLive,
          title: pendingTitle,
          thumbnail: pendingThumbnail,
          audioUrl: pendingAudioUrl,
          expectedSize: pendingExpectedSize,
          cookieHeader: pendingCookieHeader,
          userAgent: pendingUserAgent,
          requestMethod: pendingRequestMethod,
          requestBody: pendingRequestBody,
          requestContentType: pendingRequestContentType,
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
    resetTaskConnections();
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
let consumingBrowserAssistedDownload = false;
async function consumeBrowserAssistedDownload() {
  if (consumingBrowserAssistedDownload || dialog.open) return;
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
    const sourceName = String(request.fileName || "").split(/[\\/]/).pop() || "download.zip";
    document.querySelector("#file-name").value = sourceName;
    document.querySelector("#analysis").hidden = true;
    document.querySelector("#enqueue").hidden = true;
    document.querySelector("#analyze").hidden = false;
    document.querySelector("#destination").value = await invoke("default_download_directory");
    resetMediaInspection();
    refreshAutoExtractOption();
    await refreshDestinationHistory();
    await invoke("activate_main_window");
    if (!dialog.open) dialog.showModal();
    document.querySelector("#url").focus();
  } catch (error) {
    console.error(error);
  } finally {
    consumingBrowserAssistedDownload = false;
  }
}
setInterval(consumeBrowserAssistedDownload, 400);
window.__TAURI__?.event?.listen?.("browser-assisted-ready", consumeBrowserAssistedDownload).catch(console.error);

let consumingBridgeDownload = false;
async function consumeBridgeDownload() {
  if (consumingBridgeDownload) return;
  consumingBridgeDownload = true;
  try {
    const currentUrl = dialog.open ? document.querySelector("#url").value : null;
    const request = await invoke("take_bridge_download", { currentUrl });
    if (!request) return;
    lastClipboardLink = request.url;
    pendingDiagnosticTrace = request.traceId || null;
    pendingReferer = request.pageUrl || null;
    pendingDuration = Number.isFinite(request.duration) ? request.duration : null;
    pendingIsLive = false;
    pendingTitle = request.title || null;
    pendingThumbnail = request.thumbnail || null;
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
    invoke("record_ui_diagnostic", { level: "INFO", event: "save_dialog_opened", detail: `trace=${pendingDiagnosticTrace || "none"}` }).catch(() => {});
    document.querySelector("#url").focus();
  } catch (error) { console.error(error); }
  finally { consumingBridgeDownload = false; }
}
setInterval(consumeBridgeDownload, 400);
window.__TAURI__?.event?.listen?.("bridge-download-ready", consumeBridgeDownload).catch(console.error);
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
}).catch(console.error);
window.__TAURI__?.event?.listen?.("media-preview-error", (event) => {
  const prefix = locale === "pt-BR" ? "Falha na pr\u00e9-visualiza\u00e7\u00e3o. Consulte Logs para os detalhes."
    : locale === "zh-CN" ? "\u9884\u89c8\u5931\u8d25\u3002\u8bf7\u67e5\u770b\u65e5\u5fd7\u3002" : "Preview failed. See Logs for details.";
  window.alert(`${prefix}\n${String(event.payload || "preview_failed")}`);
}).catch(console.error);
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
    console.error(error);
  }
}
refreshBridgeStatus();
setInterval(refreshBridgeStatus, 3000);
