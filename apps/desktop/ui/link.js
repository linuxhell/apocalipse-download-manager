const catalogs = {
  en: {
    linkDescription: "Transfer files securely between this computer and a remote Apocalipse.",
    linkThisComputer: "This computer",
    linkRemoteComputer: "Remote computer",
    linkRemoteControl: "Remote connection",
    linkRemoteId: "Remote IP / host",
    linkRemoteAddressExamples: "Use this PC, a local-network IP or a public Internet IP/host.",
    linkAccessNotice: "Only explicitly shared files, folders and drives are exposed.",
    linkShareNotice: "Share a file, folder or mapped drive here. Windows and Linux SMB shared folders are also discovered automatically.",
    linkRemoteShareNotice: "Only files, folders and drives shared by the other user appear below.",
    linkNoShares: "No shared files, folders or drives yet.",
    linkUseForDownload: "Use for download",
    linkUseForDownloadHint: "This matches your paused/failed download “{name}”. Fill it from here instead of downloading over the internet.",
    linkUseForDownloadCompleted: "“{name}” filled from Apocalipse Link.",
    linkRemoteUsername: "Operating-system username",
    linkRemoteSystemPassword: "System account password",
    linkCredentialsRequired: "Enter the remote IP/host, operating-system username and account password.",
    linkAuthenticating: "Authenticating system account…",
    linkAuthenticationFailed: "System account authentication failed",
    linkAuthenticationInvalidCredentials: "Windows rejected the credentials. For a Microsoft account, enter the e-mail address and the account password; Windows Hello PIN is not accepted.",
    linkLocalSessionReady: "Connected to this Apocalipse. Shared items are available below.",
    linkRemoteSessionReady: "Encrypted TLS connection established. Remote shares are available below.",
    linkRemoteFirstTrust: "First connection: this Apocalipse TLS certificate was trusted for this address.",
    linkConnectionFailed: "Connection failed",
    linkRemoteAuthPlan: "Remote access uses one login: IP/host + operating-system username + account password.",
    linkRemoteAccountFormats: "Windows: local, domain, Microsoft or AzureAD account. Linux/macOS: local system username.",
    linkRemoteSecurityNotice: "The system password is never saved and is sent only inside the encrypted TLS channel.",
    linkShareFile: "Share file",
    linkShareFolder: "Share folder or drive",
    linkReadOnly: "Read only",
    linkReadWrite: "Read and write",
    linkStopSharing: "Stop sharing",
    linkConnect: "Connect", linkDisconnect: "Disconnect", linkDisconnected: "Disconnected.",
    linkDelete: "Delete",
    linkDownload: "← Download",
    linkSend: "Send →",
    linkDrives: "Shares",
    linkTransfer: "Transfer",
    linkTransferDownload: "Receiving",
    linkTransferUpload: "Sending",
    linkTransferPreparing: "Preparing transfer…",
    linkPause: "Pause",
    linkContinue: "Continue",
    linkCancel: "Cancel",
    linkPaused: "Paused",
    linkTransferCancelled: "Transfer cancelled",
    linkTransferring: "Transferring…",
    linkSending: "Sending…",
    linkCompleted: "Completed",
    linkTransferFailed: "Transfer failed",
    linkUploadFailed: "Upload failed",
    linkDeleteConfirm: "Permanently delete {name}?"
  },
  "pt-BR": {
    linkDescription: "Transfira arquivos com segurança entre este computador e um Apocalipse remoto.",
    linkThisComputer: "Este computador",
    linkRemoteComputer: "Computador remoto",
    linkRemoteControl: "Conexão remota",
    linkRemoteId: "IP / host remoto",
    linkRemoteAddressExamples: "Use este PC, um IP da rede local ou um IP/host público da Internet.",
    linkAccessNotice: "Somente arquivos, pastas e unidades compartilhados explicitamente ficam expostos.",
    linkShareNotice: "Compartilhe um arquivo, pasta ou unidade por aqui. Pastas compartilhadas pelo Windows ou Linux via SMB também aparecem automaticamente.",
    linkRemoteShareNotice: "Abaixo aparecem somente arquivos, pastas e unidades compartilhados pelo outro usuário.",
    linkNoShares: "Nenhum arquivo, pasta ou unidade compartilhado ainda.",
    linkUseForDownload: "Usar para download",
    linkUseForDownloadHint: "Este arquivo corresponde ao seu download pausado/com falha “{name}”. Preencha a partir daqui em vez de baixar pela internet.",
    linkUseForDownloadCompleted: "“{name}” preenchido pelo Apocalipse Link.",
    linkRemoteUsername: "Usuário do sistema operacional",
    linkRemoteSystemPassword: "Senha da conta do sistema",
    linkCredentialsRequired: "Informe o IP/host remoto, o usuário do sistema operacional e a senha da conta.",
    linkAuthenticating: "Autenticando conta do sistema…",
    linkAuthenticationFailed: "Falha na autenticação da conta do sistema",
    linkAuthenticationInvalidCredentials: "O Windows rejeitou as credenciais. Em conta Microsoft, informe o e-mail e a senha da conta; o PIN do Windows Hello não é aceito.",
    linkLocalSessionReady: "Conectado a este Apocalipse. Os compartilhamentos estão disponíveis abaixo.",
    linkRemoteSessionReady: "Conexão TLS criptografada estabelecida. Os compartilhamentos remotos estão disponíveis abaixo.",
    linkRemoteFirstTrust: "Primeira conexão: o certificado TLS deste Apocalipse foi confiado para este endereço.",
    linkConnectionFailed: "Falha na conexão",
    linkRemoteAuthPlan: "O acesso remoto usa um único login: IP/host + usuário do sistema operacional + senha da conta.",
    linkRemoteAccountFormats: "Windows: conta local, domínio, Microsoft ou AzureAD. Linux/macOS: usuário local do sistema.",
    linkRemoteSecurityNotice: "A senha do sistema nunca é salva e só é enviada dentro do canal TLS criptografado.",
    linkShareFile: "Compartilhar arquivo",
    linkShareFolder: "Compartilhar pasta ou unidade",
    linkReadOnly: "Somente leitura",
    linkReadWrite: "Leitura e gravação",
    linkStopSharing: "Parar de compartilhar",
    linkConnect: "Conectar", linkDisconnect: "Desconectar", linkDisconnected: "Desconectado.",
    linkDelete: "Apagar",
    linkDownload: "← Baixar",
    linkSend: "Enviar →",
    linkDrives: "Compartilhamentos",
    linkTransfer: "Transferência",
    linkTransferDownload: "Recebendo",
    linkTransferUpload: "Enviando",
    linkTransferPreparing: "Preparando transferência…",
    linkPause: "Pausar",
    linkContinue: "Continuar",
    linkCancel: "Cancelar",
    linkPaused: "Pausado",
    linkTransferCancelled: "Transferência cancelada",
    linkTransferring: "Transferindo…",
    linkSending: "Enviando…",
    linkCompleted: "Concluído",
    linkTransferFailed: "Falha na transferência",
    linkUploadFailed: "Falha no envio",
    linkDeleteConfirm: "Apagar permanentemente {name}?"
  },
  "zh-CN": {
    linkDescription: "在本机与远程 Apocalipse 之间安全传输文件。",
    linkThisComputer: "此电脑",
    linkRemoteComputer: "远程电脑",
    linkRemoteControl: "远程连接",
    linkRemoteId: "远程 IP / 主机",
    linkRemoteAddressExamples: "可使用本机、局域网 IP 或公网 IP/主机。",
    linkAccessNotice: "仅公开明确共享的文件、文件夹和驱动器。",
    linkShareNotice: "可在此共享文件、文件夹或映射驱动器；Windows 和 Linux 的 SMB 共享文件夹也会自动显示。",
    linkRemoteShareNotice: "下方仅显示对方共享的文件、文件夹和驱动器。",
    linkNoShares: "尚无共享文件、文件夹或驱动器。",
    linkUseForDownload: "用于此下载",
    linkUseForDownloadHint: "此文件与您暂停/失败的下载“{name}”匹配。可从这里直接填充，而不必通过互联网下载。",
    linkUseForDownloadCompleted: "“{name}”已通过 Apocalipse Link 填充完成。",
    linkRemoteUsername: "操作系统用户名",
    linkRemoteSystemPassword: "系统账户密码",
    linkCredentialsRequired: "请输入远程 IP/主机、操作系统用户名和账户密码。",
    linkAuthenticating: "正在验证系统账户…",
    linkAuthenticationFailed: "系统账户身份验证失败",
    linkAuthenticationInvalidCredentials: "Windows 拒绝了凭据。Microsoft 账户请填写电子邮件和账户密码；不支持 Windows Hello PIN。",
    linkLocalSessionReady: "已连接到本机 Apocalipse。共享项目显示在下方。",
    linkRemoteSessionReady: "TLS 加密连接已建立。远程共享项目显示在下方。",
    linkRemoteFirstTrust: "首次连接：已信任此地址的 Apocalipse TLS 证书。",
    linkConnectionFailed: "连接失败",
    linkRemoteAuthPlan: "远程访问使用一个登录：IP/主机 + 操作系统用户名 + 账户密码。",
    linkRemoteAccountFormats: "Windows：本地、域、Microsoft 或 AzureAD 账户。Linux/macOS：本地系统用户名。",
    linkRemoteSecurityNotice: "系统密码永不保存，并且只会通过 TLS 加密通道发送。",
    linkShareFile: "共享文件",
    linkShareFolder: "共享文件夹或驱动器",
    linkReadOnly: "只读",
    linkReadWrite: "读写",
    linkStopSharing: "停止共享",
    linkConnect: "连接", linkDisconnect: "断开连接", linkDisconnected: "已断开连接。",
    linkDelete: "删除",
    linkDownload: "← 下载",
    linkSend: "发送 →",
    linkDrives: "共享",
    linkTransfer: "传输",
    linkTransferDownload: "正在接收",
    linkTransferUpload: "正在发送",
    linkTransferPreparing: "正在准备传输…",
    linkPause: "暂停",
    linkContinue: "继续",
    linkCancel: "取消",
    linkPaused: "已暂停",
    linkTransferCancelled: "传输已取消",
    linkTransferring: "正在传输…",
    linkSending: "正在发送…",
    linkCompleted: "已完成",
    linkTransferFailed: "传输失败",
    linkUploadFailed: "发送失败",
    linkDeleteConfirm: "永久删除 {name}？"
  }
};

let locale = localStorage.getItem("apocalipse.language") || "en";
const t = (key) => catalogs[locale]?.[key] || catalogs.en[key] || key;
const tf = (key, values) => Object.entries(values).reduce((text, [name, value]) => text.replaceAll(`{${name}}`, value), t(key));

const validThemes = ["void", "inferno", "toxic", "synthwave", "royal", "crimson", "arctic", "obsidian", "monochrome", "midnight", "forest", "graphite", "deepsea", "eclipse", "hazard", "cyberstorm", "ultraviolet", "emeraldgold", "scarletice", "coppernavy", "solarizednight", "pearlblue", "whiteaurora", "goldenivory", "crystalrose", "polarmint"];
const appearanceDefaults = { transparencyEnabled: false, transparencyLevel: 30, roundedEnabled: true, cornerRadius: 10, interfaceSize: "normal" };

function syncPresentation() {
  locale = localStorage.getItem("apocalipse.language") || "en";
  document.documentElement.lang = locale === "pt-BR" ? "pt-BR" : locale === "zh-CN" ? "zh-CN" : "en";
  const theme = localStorage.getItem("apocalipse.theme") || "void";
  document.documentElement.dataset.theme = validThemes.includes(theme) ? theme : "void";
  let appearance = { ...appearanceDefaults };
  try { appearance = { ...appearance, ...JSON.parse(localStorage.getItem("apocalipse.appearance") || "{}") }; } catch {}
  const transparency = Math.max(0, Math.min(70, Number(appearance.transparencyLevel) || 0));
  const radius = Math.max(0, Math.min(28, Number(appearance.cornerRadius) || 0));
  document.documentElement.dataset.transparency = appearance.transparencyEnabled ? "on" : "off";
  document.documentElement.dataset.rounded = appearance.roundedEnabled ? "on" : "off";
  document.documentElement.dataset.uiSize = ["compact", "normal", "large"].includes(appearance.interfaceSize) ? appearance.interfaceSize : "normal";
  document.documentElement.style.setProperty("--window-opacity-percent", appearance.transparencyEnabled ? `${100 - transparency}%` : "100%");
  document.documentElement.style.setProperty("--corner-radius", appearance.roundedEnabled ? `${radius}px` : "0px");
  document.querySelectorAll("[data-i18n]").forEach((node) => { node.textContent = t(node.dataset.i18n); });
  document.querySelector("#link-window-description").textContent = t("linkDescription");
  syncLinkTransferLanguage();
}

const rawInvoke = window.__TAURI__?.core?.invoke;
let lastLinkInteractionTrace = null;
const freshLinkTrace = () => {
  const now = performance.now();
  if (lastLinkInteractionTrace && now - lastLinkInteractionTrace.at < 2000) return lastLinkInteractionTrace.id;
  return crypto.randomUUID();
};
const recordLinkUi = (event, detail = {}) =>
  rawInvoke?.("record_diagnostics_ui", { event, detail }).catch(() => {});
document.addEventListener("click", (event) => {
  const control = event.target?.closest?.("button,[role='button'],a,input[type='button'],input[type='submit']");
  if (!control || !rawInvoke) return;
  const traceId = crypto.randomUUID();
  lastLinkInteractionTrace = { id: traceId, at: performance.now() };
  recordLinkUi("control_clicked", {
    traceId,
    level: "INFO",
    window: "link",
    controlTag: control.tagName?.toLowerCase() || "unknown",
    controlType: control.getAttribute?.("type") || control.getAttribute?.("role") || "default",
    controlId: control.id || null,
    action: control.dataset?.action || null,
  });
}, true);
const invoke = async (command, args = {}) => {
  if (!rawInvoke) throw new Error("Desktop bridge unavailable");
  const started = performance.now();
  const traceId = freshLinkTrace();
  const taskId = typeof args?.id === "string" ? args.id : null;
  const detailBase = {
    traceId,
    taskId,
    window: "link",
    command,
    argKeys: Object.keys(args || {}).sort(),
  };
  if (command !== "record_ui_diagnostic" && command !== "record_diagnostics_ui") {
    recordLinkUi("command_started", { ...detailBase, level: "DEBUG" });
  }
  try {
    const result = await rawInvoke(command, args);
    if (command !== "record_ui_diagnostic" && command !== "record_diagnostics_ui") {
      recordLinkUi("command_completed", {
        ...detailBase,
        level: "DEBUG",
        durationMs: Math.round(performance.now() - started),
        resultType: result == null ? "null" : Array.isArray(result) ? "array" : typeof result,
      });
    }
    return result;
  } catch (error) {
    if (command !== "record_ui_diagnostic" && command !== "record_diagnostics_ui") {
      recordLinkUi("command_failed", {
        ...detailBase,
        level: "ERROR",
        durationMs: Math.round(performance.now() - started),
        errorName: String(error?.name || "command_error"),
      });
    }
    throw error;
  }
};

function formatBytes(bytes) {
  if (!bytes) return "0 B";
  const units = ["B", "KB", "MB", "GB", "TB"];
  const index = Math.min(Math.floor(Math.log(bytes) / Math.log(1024)), units.length - 1);
  return `${(bytes / 1024 ** index).toFixed(index ? 1 : 0)} ${units[index]}`;
}

let linkLocalPath = "";
let linkRemotePath = "";
let linkRemoteId = "";
let linkRemoteTransportToken = "";
let linkLocalIdentity = "";
let linkLocalAccountSession = false;
let linkSelectedLocal = null;
let linkSelectedRemote = null;
let linkRemoteAllowWrite = false;
let linkTransferActive = false;
let linkTransferPaused = false;
let linkTransferCancelRequested = false;
let linkActiveTransferId = "";
let linkActiveTransferDirection = "";
let linkActiveTransferName = "";
let linkTransferPollTimer = null;
let linkLastProgressEventAt = 0;

// Lets a remote file listed here stand in for a local paused/failed/queued
// download with the same file name, so the user can fill it from a paired
// Link peer instead of waiting on the internet.
let linkMatchableDownloads = [];
async function refreshLinkMatchableDownloads() {
  try {
    const downloads = await invoke("list_downloads");
    linkMatchableDownloads = downloads.filter((task) => {
      const key = typeof task.state === "string" ? task.state : Object.keys(task.state || {})[0];
      return ["paused", "failed", "queued"].includes(key)
        && !/^(?:magnet:)|\.torrent(?:$|[?#])/i.test(task.source || "");
    });
  } catch {
    linkMatchableDownloads = [];
  }
}
function linkMatchingDownload(fileName) {
  const lower = String(fileName || "").toLowerCase();
  if (!lower) return null;
  return linkMatchableDownloads.find((task) => {
    const base = String(task.destination || "").split(/[\\/]/).pop();
    return base && base.toLowerCase() === lower;
  }) || null;
}

function stopLinkTransferProgressPolling() {
  if (linkTransferPollTimer !== null) {
    clearInterval(linkTransferPollTimer);
    linkTransferPollTimer = null;
  }
}

function startLinkTransferProgressPolling(transferId) {
  stopLinkTransferProgressPolling();
  const poll = async () => {
    if (!linkTransferActive || !transferId || transferId !== linkActiveTransferId || !rawInvoke) {
      if (!linkTransferActive || transferId !== linkActiveTransferId) stopLinkTransferProgressPolling();
      return;
    }
    // Tauri events remain the primary path. Poll only when no fresh event was
    // observed recently, which keeps this as a fallback without duplicating work.
    if (performance.now() - linkLastProgressEventAt < 450) return;
    try {
      const progress = await rawInvoke("get_link_transfer_progress", { transferId });
      if (progress && linkTransferActive && transferId === linkActiveTransferId) {
        renderLinkTransferProgress(progress);
      }
    } catch (error) {
      if (String(error) !== "link_transfer_not_found") {
        console.debug("Apocalipse Link progress poll", error);
      }
    }
  };
  void poll();
  linkTransferPollTimer = setInterval(poll, 250);
}

const linkParent = (path) => /^[A-Za-z]:[\\/]?$/.test(path) || /^\/shares\/[^/]+\/?$/.test(path)
  ? ""
  : path.replace(/[\\/]+$/, "").replace(/[\\/][^\\/]*$/, "");

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
  document.querySelector("#link-upload-local").disabled = linkTransferActive || !linkSelectedLocal || !linkRemoteId || !linkRemotePath || !linkRemoteAllowWrite;
  document.querySelector("#link-download-remote").disabled = linkTransferActive || !linkSelectedRemote;
  document.querySelector("#link-delete-remote").disabled = linkTransferActive || !linkSelectedRemote || !linkRemoteAllowWrite;
  document.querySelector("#link-disconnect").disabled = linkTransferActive || !linkRemoteId;
  const pause = document.querySelector("#link-transfer-pause");
  const cancel = document.querySelector("#link-transfer-cancel");
  if (pause) {
    pause.disabled = !linkTransferActive || linkTransferCancelRequested;
    pause.textContent = linkTransferPaused ? t("linkContinue") : t("linkPause");
  }
  if (cancel) cancel.disabled = !linkTransferActive || linkTransferCancelRequested;
}

function createLinkTransferId() {
  return globalThis.crypto?.randomUUID?.() || `link-${Date.now()}-${Math.random().toString(16).slice(2)}`;
}

function syncLinkTransferLanguage() {
  const title = document.querySelector("#link-transfer-title");
  if (title && linkTransferActive) {
    const label = linkActiveTransferDirection === "upload" ? t("linkTransferUpload") : t("linkTransferDownload");
    title.textContent = linkActiveTransferName ? `${label} · ${linkActiveTransferName}` : label;
  }
  updateLinkTransferButtons();
}

function beginLinkTransfer(direction, name = "") {
  linkTransferActive = true;
  linkTransferPaused = false;
  linkTransferCancelRequested = false;
  linkActiveTransferId = createLinkTransferId();
  linkActiveTransferDirection = direction;
  linkActiveTransferName = name;
  const panel = document.querySelector("#link-transfer-panel");
  const percent = document.querySelector("#link-transfer-percent");
  const detail = document.querySelector("#link-transfer-detail");
  const track = panel.querySelector(".link-transfer-track");
  panel.hidden = false;
  document.querySelector("#link-transfer-fill").style.width = "0%";
  percent.textContent = "0%";
  detail.textContent = t("linkTransferPreparing");
  track.setAttribute("aria-valuenow", "0");
  syncLinkTransferLanguage();
  linkLastProgressEventAt = 0;
  startLinkTransferProgressPolling(linkActiveTransferId);
  return linkActiveTransferId;
}

function renderLinkTransferProgress(progress) {
  if (!linkTransferActive || progress?.transferId !== linkActiveTransferId) return;
  const transferred = Math.max(0, Number(progress.transferred) || 0);
  const total = Math.max(0, Number(progress.total) || 0);
  const speed = Math.max(0, Number(progress.bytesPerSecond) || 0);
  const value = total > 0
    ? Math.max(0, Math.min(100, Number(progress.percent) || (transferred * 100 / total)))
    : 0;
  document.querySelector("#link-transfer-fill").style.width = `${value}%`;
  document.querySelector("#link-transfer-percent").textContent =
    total > 0 ? `${value.toFixed(value >= 10 ? 0 : 1)}%` : "…";
  document.querySelector("#link-transfer-detail").textContent = linkTransferPaused
    ? t("linkPaused")
    : total > 0
      ? `${formatBytes(transferred)} / ${formatBytes(total)} · ${formatBytes(speed)}/s`
      : `${formatBytes(transferred)} · ${formatBytes(speed)}/s`;
  document.querySelector("#link-transfer-panel .link-transfer-track").setAttribute("aria-valuenow", String(value));
}

function finishLinkTransfer({ success = false, cancelled = false } = {}) {
  stopLinkTransferProgressPolling();
  if (success) {
    document.querySelector("#link-transfer-fill").style.width = "100%";
    document.querySelector("#link-transfer-percent").textContent = "100%";
    document.querySelector("#link-transfer-panel .link-transfer-track").setAttribute("aria-valuenow", "100");
    document.querySelector("#link-transfer-detail").textContent = t("linkCompleted");
  } else if (cancelled) {
    document.querySelector("#link-transfer-detail").textContent = t("linkTransferCancelled");
  }
  linkTransferActive = false;
  linkTransferPaused = false;
  linkTransferCancelRequested = false;
  linkActiveTransferId = "";
  linkActiveTransferDirection = "";
  linkActiveTransferName = "";
  updateLinkTransferButtons();
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
          if (linkTransferActive) return;
          useButton.disabled = true;
          const status = document.querySelector("#link-status");
          const transferId = beginLinkTransfer("download", entry.name);
          status.textContent = t("linkTransferring");
          try {
            await invoke("use_remote_link_file_for_download", {
              taskId: match.id,
              id: linkRemoteId,
              password: linkRemoteTransportToken,
              path: entry.path,
              transferId,
            });
            finishLinkTransfer({ success: true });
            status.textContent = tf("linkUseForDownloadCompleted", { name: match.display_title || entry.name });
            await refreshLinkMatchableDownloads();
            renderLinkFiles(target, entries, open, select, matchDownloads);
          } catch (error) {
            const cancelled = linkTransferCancelRequested || String(error) === "cancelled";
            finishLinkTransfer({ cancelled });
            if (!cancelled) {
              status.textContent = `${t("linkTransferFailed")}: ${error}`;
            }
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
  const entries = await invoke("list_local_link_files", { path });
  renderLinkFiles("#link-local-files", entries, openLocalLink, (entry) => {
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
  if (!linkLocalAccountSession) await refreshLinkMatchableDownloads();
  renderLinkFiles("#link-remote-files", entries, openRemoteLink, (entry) => {
    linkSelectedRemote = entry;
    updateLinkTransferButtons();
  }, !linkLocalAccountSession);
}

async function refreshVisibleLinkPanels({ resetToRoot = false } = {}) {
  const localPath = resetToRoot ? "" : linkLocalPath;
  const remotePath = resetToRoot ? "" : linkRemotePath;
  await openLocalLink(localPath).catch(() => openLocalLink(""));
  if (linkRemoteId) await openRemoteLink(remotePath).catch(() => openRemoteLink(""));
}

function renderLinkShares(shares) {
  const root = document.querySelector("#link-share-list");
  root.replaceChildren();
  for (const share of shares) {
    const row = document.createElement("div");
    const name = Object.assign(document.createElement("b"), { textContent: share.name });
    const permission = document.createElement("select");
    permission.append(new Option(t("linkReadOnly"), "false"), new Option(t("linkReadWrite"), "true"));
    permission.value = String(Boolean(share.allowWrite));
    permission.onchange = async () => {
      renderLinkShares(await invoke("update_link_share", { id: share.id, allowWrite: permission.value === "true" }));
      await refreshVisibleLinkPanels({ resetToRoot: true });
    };
    const remove = Object.assign(document.createElement("button"), { type: "button", textContent: t("linkStopSharing") });
    remove.onclick = async () => {
      renderLinkShares(await invoke("remove_link_share", { id: share.id }));
      await refreshVisibleLinkPanels({ resetToRoot: true });
    };
    row.append(name, permission, remove);
    root.append(row);
  }
}

async function loadLinkIdentity() {
  const identity = await invoke("get_link_identity");
  linkLocalIdentity = identity.id;
  document.querySelector("#link-own-id").value = identity.id;
  renderLinkShares(await invoke("list_link_shares"));
  await openLocalLink();
}

function linkAuthenticationMessage(error) {
  const value = String(error);
  if (value.includes("system_auth_failed:1326")) return t("linkAuthenticationInvalidCredentials");
  return `${t("linkAuthenticationFailed")}: ${value}`;
}

document.querySelector("#link-share-file").onclick = async () => {
  try {
    renderLinkShares(await invoke("add_link_file_share"));
    await refreshVisibleLinkPanels({ resetToRoot: true });
  } catch (error) {
    if (String(error) !== "cancelled") window.alert(String(error));
  }
};

document.querySelector("#link-share-folder").onclick = async () => {
  try {
    renderLinkShares(await invoke("add_link_share"));
    await refreshVisibleLinkPanels({ resetToRoot: true });
  } catch (error) {
    if (String(error) !== "cancelled") window.alert(String(error));
  }
};

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
  if (!linkSelectedRemote || !window.confirm(tf("linkDeleteConfirm", { name: linkSelectedRemote.name }))) return;
  if (linkLocalAccountSession) {
    await invoke("delete_local_shared_link_item", { path: linkSelectedRemote.path });
  } else {
    await invoke("delete_remote_link_item", { id: linkRemoteId, password: linkRemoteTransportToken, path: linkSelectedRemote.path });
  }
  await openRemoteLink(linkRemotePath);
};

document.querySelector("#link-download-remote").onclick = async () => {
  if (!linkSelectedRemote || linkTransferActive) return;
  const status = document.querySelector("#link-status");
  const selected = linkSelectedRemote;
  const transferId = beginLinkTransfer("download", selected.name);
  status.textContent = t("linkTransferring");
  try {
    const destination = linkLocalAccountSession
      ? await invoke("download_local_shared_link_item", {
          path: selected.path,
          directory: selected.directory,
          fileName: selected.name,
          transferId,
        })
      : await invoke("download_remote_link_file", {
          id: linkRemoteId,
          password: linkRemoteTransportToken,
          path: selected.path,
          directory: selected.directory,
          fileName: selected.name,
          transferId,
        });
    finishLinkTransfer({ success: true });
    status.textContent = `${t("linkCompleted")}: ${destination}`;
  } catch (error) {
    const cancelled = linkTransferCancelRequested || String(error) === "cancelled";
    finishLinkTransfer({ cancelled });
    if (cancelled) {
      status.textContent = t("linkTransferCancelled");
    } else if (String(error) !== "cancelled") {
      document.querySelector("#link-transfer-detail").textContent = `${t("linkTransferFailed")}: ${error}`;
      status.textContent = `${t("linkTransferFailed")}: ${error}`;
    }
  }
};

document.querySelector("#link-upload-local").onclick = async () => {
  if (!linkSelectedLocal || !linkRemoteId || !linkRemotePath || linkTransferActive) return;
  const status = document.querySelector("#link-status");
  const selected = linkSelectedLocal;
  const transferId = beginLinkTransfer("upload", selected.name);
  status.textContent = t("linkSending");
  try {
    const remotePath = linkLocalAccountSession
      ? await invoke("upload_local_shared_link_item", {
          remoteDirectory: linkRemotePath,
          localPath: selected.path,
          transferId,
        })
      : await invoke("upload_remote_link_file", {
          id: linkRemoteId,
          password: linkRemoteTransportToken,
          remoteDirectory: linkRemotePath,
          localPath: selected.path,
          transferId,
        });
    finishLinkTransfer({ success: true });
    status.textContent = `${t("linkCompleted")}: ${remotePath}`;
    await openRemoteLink(linkRemotePath);
  } catch (error) {
    const cancelled = linkTransferCancelRequested || String(error) === "cancelled";
    finishLinkTransfer({ cancelled });
    if (cancelled) {
      status.textContent = t("linkTransferCancelled");
    } else {
      document.querySelector("#link-transfer-detail").textContent = `${t("linkUploadFailed")}: ${error}`;
      status.textContent = `${t("linkUploadFailed")}: ${error}`;
    }
  }
};

document.querySelector("#link-transfer-pause").onclick = async () => {
  if (!linkTransferActive || !linkActiveTransferId || linkTransferCancelRequested) return;
  const nextPaused = !linkTransferPaused;
  try {
    await invoke("pause_link_transfer", { transferId: linkActiveTransferId, paused: nextPaused });
    linkTransferPaused = nextPaused;
    document.querySelector("#link-transfer-detail").textContent =
      linkTransferPaused ? t("linkPaused") : t("linkTransferring");
    updateLinkTransferButtons();
  } catch (error) {
    if (String(error) !== "link_transfer_not_found") console.error(error);
  }
};

document.querySelector("#link-transfer-cancel").onclick = async () => {
  if (!linkTransferActive || !linkActiveTransferId || linkTransferCancelRequested) return;
  linkTransferCancelRequested = true;
  linkTransferPaused = false;
  document.querySelector("#link-transfer-detail").textContent = t("linkTransferCancelled");
  updateLinkTransferButtons();
  try {
    await invoke("cancel_link_transfer", { transferId: linkActiveTransferId });
  } catch (error) {
    if (String(error) !== "link_transfer_not_found") console.error(error);
  }
};

window.__TAURI__?.event?.listen?.("link-transfer-progress", (event) => {
  linkLastProgressEventAt = performance.now();
  renderLinkTransferProgress(event.payload || {});
}).catch(console.error);

syncPresentation();
window.addEventListener("storage", syncPresentation);
invoke("record_ui_diagnostic", { level: "INFO", event: "link_window_opened", detail: "dedicated=true maximized=true" }).catch(() => {});
loadLinkIdentity().catch((error) => {
  document.querySelector("#link-status").textContent = String(error);
});
