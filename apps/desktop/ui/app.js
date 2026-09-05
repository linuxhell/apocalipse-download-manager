const catalogs = {
  en: {
    downloads: "Downloads",
    media: "Media",
    torrents: "Torrents",
    tools: "Tools",
    settings: "Settings",
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
  },
  "pt-BR": {
    downloads: "Downloads",
    media: "Mídia",
    torrents: "Torrents",
    tools: "Ferramentas",
    settings: "Configurações",
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
  },
  "zh-CN": {
    downloads: "下载",
    media: "媒体",
    torrents: "种子",
    tools: "工具",
    settings: "设置",
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
  },
};

let locale = localStorage.getItem("apocalipse.language") || "en";
const applyTheme = (theme) => {
  const valid = ["void", "inferno", "toxic", "synthwave", "royal", "crimson", "arctic", "obsidian", "monochrome", "midnight", "forest", "graphite", "deepsea", "eclipse"];
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
let lastClipboardLink = "";
let clipboardMonitorPrimed = false;
const busyIds = new Set();
const selectedIds = new Set();
const speedSamples = new Map();
let selectionPointerActive = false;
const t = (key) => catalogs[locale]?.[key] || catalogs.en[key] || key;
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
  for (const [id] of speedSamples)
    if (!currentIds.has(id)) speedSamples.delete(id);
  for (const task of tasks) {
    const previous = speedSamples.get(task.id);
    const active = stateKey(task.state) === "downloading";
    let speed = active ? previous?.speed || 0 : 0;
    if (previous && active) {
      const elapsed = Math.max(0.001, (now - previous.at) / 1000);
      const delta = Math.max(0, task.received - previous.bytes);
      if (delta > 0) {
        const instantaneous = delta / elapsed;
        speed = previous.speed ? instantaneous * 0.65 + previous.speed * 0.35 : instantaneous;
      } else if (now - previous.at >= 500) {
        speed = 0;
      }
    }
    speedSamples.set(task.id, {
      at: now,
      bytes: task.received,
      speed,
    });
    if (active) overallSpeed += speed;
  }
}

function visibleDownloads() {
  let visible = downloads;
  if (activePage === "torrents") visible = visible.filter((task) => /^(?:magnet:)|\.torrent(?:$|[?#])/i.test(task.source));
  if (activePage === "media") visible = visible.filter((task) => /(?:\.m3u8(?:$|[?#])|\.recording\.webm$|youtube\.com|youtu\.be|facebook\.com|fb\.watch|tiktok\.com|instagram\.com)/i.test(`${task.source} ${task.destination}`));
  if (activePage === "link") visible = visible.filter((task) => /^(?:ftp|sftp):/i.test(task.source));
  if (activeFilter === "completed") return visible.filter((task) => task.state === "completed");
  if (activeFilter === "active") return visible.filter((task) => task.state !== "completed");
  return visible;
}

function renderDownloads() {
  const list = document.querySelector("#download-list");
  const empty = document.querySelector("#empty");
  list.replaceChildren();
  list.hidden = downloads.length === 0;
  empty.hidden = downloads.length !== 0;
  const visible = visibleDownloads();
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
    const progressText = hasReportedPercent && !task.total
      ? `${percent.toFixed(1)}%`
      : task.total
      ? `${formatBytes(task.received)} / ${formatBytes(task.total)} · ${percent.toFixed(1)}%`
      : formatBytes(task.received);
    details.textContent =
      speed && task.state === "downloading"
        ? `${progressText} · ${formatBytes(speed)}/s`
        : progressText;
    progress.append(bar);
    info.append(progress, details);
    const state = Object.assign(document.createElement("span"), {
      className: "download-state",
      textContent: stateName(task.state),
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
    if (key === "downloading" || key === "inspecting")
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
        document.querySelector("#export-format").value = "mkv";
        document.querySelector("#export-video-codec").value = "copy";
        document.querySelector("#export-audio-codec").value = "copy";
        exportDialog.showModal();
      };
      actions.append(exportButton);
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
  renderDownloads();
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
const logDialog = document.querySelector("#log-dialog");
const siteRulesDialog = document.querySelector("#site-rules-dialog");
const exportDialog = document.querySelector("#export-dialog");
let exportTaskId = null;
document.querySelectorAll('nav [data-page]:not([data-page="settings"]):not([data-page="tools"])').forEach((button) => {
  button.onclick = () => {
    activePage = button.dataset.page;
    document.querySelectorAll("nav [data-page]").forEach((item) => item.classList.toggle("active", item === button));
    const heading = button.querySelector("b")?.textContent || t("downloads");
    document.querySelector("header h1").textContent = heading;
    renderDownloads();
  };
});
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
  button.disabled = true;
  try {
    const tools = await invoke("get_tool_statuses");
    for (const tool of tools) {
      document.querySelector(`#tool-${tool.id}`).value = tool.path;
      const status = document.querySelector(`[data-tool="${tool.id}"] > span small`);
      status.textContent = tool.found ? `${t("installed")} · ${tool.version}` : t("missing");
      status.classList.toggle("tool-found", tool.found);
    }
  } catch (error) { console.error(error); }
  finally { button.disabled = false; }
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
document.querySelector('[data-page="settings"]').onclick = async () => {
  try {
    const [autostart, directory, clipboard, limits, pairing, userAgent, logEditor, proxy, dns, associations] = await Promise.all([
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
    ]);
    document.querySelector("#autostart").checked = autostart.enabled;
    document.querySelector("#theme").value = document.documentElement.dataset.theme;
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
    updateLogEditorControls();
    await refreshToolStatuses();
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
    await invoke("set_tool_paths", {
      ffmpeg: document.querySelector("#tool-ffmpeg").value,
      ytDlp: document.querySelector("#tool-yt-dlp").value,
      nM3u8dlRe: document.querySelector("#tool-n-m3u8dl-re").value,
      aria2: document.querySelector("#tool-aria2").value,
    });
    settingsDialog.close();
  } catch (error) {
    console.error(error);
  } finally {
    button.disabled = false;
  }
};
document.querySelector("#check-tools").onclick = refreshToolStatuses;
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
    downloads.push(
      await invoke("enqueue_download", {
        url: url.value,
        destinationDirectory: document.querySelector("#destination").value,
        fileName: document.querySelector("#file-name").value,
        formatSelection: document.querySelector("#media-inspection").hidden ? null : document.querySelector("#media-format").value,
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
