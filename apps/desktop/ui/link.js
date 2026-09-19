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
    linkConnect: "Connect",
    linkDelete: "Delete",
    linkDownload: "← Download",
    linkSend: "Send →",
    linkDrives: "Shares",
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
    linkConnect: "Conectar",
    linkDelete: "Apagar",
    linkDownload: "← Baixar",
    linkSend: "Enviar →",
    linkDrives: "Compartilhamentos",
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
    linkConnect: "连接",
    linkDelete: "删除",
    linkDownload: "← 下载",
    linkSend: "发送 →",
    linkDrives: "共享",
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
}

const rawInvoke = window.__TAURI__?.core?.invoke;
const invoke = async (command, args = {}) => {
  if (!rawInvoke) throw new Error("Desktop bridge unavailable");
  const started = performance.now();
  if (command !== "record_ui_diagnostic") rawInvoke("record_ui_diagnostic", { level: "DEBUG", event: "command_started", detail: `command=${command} window=link` }).catch(() => {});
  try {
    const result = await rawInvoke(command, args);
    if (command !== "record_ui_diagnostic") rawInvoke("record_ui_diagnostic", { level: "DEBUG", event: "command_completed", detail: `command=${command} window=link duration_ms=${Math.round(performance.now() - started)}` }).catch(() => {});
    return result;
  } catch (error) {
    if (command !== "record_ui_diagnostic") rawInvoke("record_ui_diagnostic", { level: "ERROR", event: "command_failed", detail: `command=${command} window=link duration_ms=${Math.round(performance.now() - started)} error=${String(error)}` }).catch(() => {});
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
  document.querySelector("#link-upload-local").disabled = !linkSelectedLocal || !linkRemoteId || !linkRemotePath || !linkRemoteAllowWrite;
  document.querySelector("#link-download-remote").disabled = !linkSelectedRemote;
  document.querySelector("#link-delete-remote").disabled = !linkSelectedRemote || !linkRemoteAllowWrite;
}

function renderLinkFiles(target, entries, open, select) {
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
  renderLinkFiles("#link-remote-files", entries, openRemoteLink, (entry) => {
    linkSelectedRemote = entry;
    updateLinkTransferButtons();
  });
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
  if (!linkSelectedRemote) return;
  const status = document.querySelector("#link-status");
  status.textContent = t("linkTransferring");
  try {
    const destination = linkLocalAccountSession
      ? await invoke("download_local_shared_link_item", { path: linkSelectedRemote.path, directory: linkSelectedRemote.directory, fileName: linkSelectedRemote.name })
      : await invoke("download_remote_link_file", { id: linkRemoteId, password: linkRemoteTransportToken, path: linkSelectedRemote.path, directory: linkSelectedRemote.directory, fileName: linkSelectedRemote.name });
    status.textContent = `${t("linkCompleted")}: ${destination}`;
  } catch (error) {
    if (String(error) !== "cancelled") status.textContent = `${t("linkTransferFailed")}: ${error}`;
  }
};

document.querySelector("#link-upload-local").onclick = async () => {
  if (!linkSelectedLocal || !linkRemoteId || !linkRemotePath) return;
  const status = document.querySelector("#link-status");
  const button = document.querySelector("#link-upload-local");
  status.textContent = t("linkSending");
  button.disabled = true;
  try {
    const remotePath = linkLocalAccountSession
      ? await invoke("upload_local_shared_link_item", { remoteDirectory: linkRemotePath, localPath: linkSelectedLocal.path })
      : await invoke("upload_remote_link_file", { id: linkRemoteId, password: linkRemoteTransportToken, remoteDirectory: linkRemotePath, localPath: linkSelectedLocal.path });
    status.textContent = `${t("linkCompleted")}: ${remotePath}`;
    await openRemoteLink(linkRemotePath);
  } catch (error) {
    status.textContent = `${t("linkUploadFailed")}: ${error}`;
  } finally {
    updateLinkTransferButtons();
  }
};

syncPresentation();
window.addEventListener("storage", syncPresentation);
invoke("record_ui_diagnostic", { level: "INFO", event: "link_window_opened", detail: "dedicated=true maximized=true" }).catch(() => {});
loadLinkIdentity().catch((error) => {
  document.querySelector("#link-status").textContent = String(error);
});
