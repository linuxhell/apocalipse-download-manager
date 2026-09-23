const catalogs = {
  en: {
    ed2kConnect: "Connect", ed2kDisconnect: "Disconnect",
    ed2kDisconnected: "Disconnected", ed2kServersApplied: "Servers configured",
    ed2kTabTransfer: "Transfer", ed2kTabServers: "Servers", ed2kTabSearch: "Search",
    ed2kLinkPlaceholder: "ed2k://|file|...|/", ed2kAddLink: "Download",
    ed2kDownloads: "Downloads", ed2kNoDownloads: "No ED2K downloads yet.",
    ed2kServerHost: "server.example.org", ed2kAddServer: "Add server",
    ed2kNoServers: "No servers configured yet.", ed2kRemoveServer: "Remove",
    ed2kSearchPlaceholder: "Search keyword…", ed2kSearch: "Search",
    ed2kNoResults: "No results yet.", ed2kSearchDownload: "Download",
    ed2kNoServersConfigured: "Add at least one server before connecting.",
    ed2kConnectFailed: "Could not apply the server list",
  },
  "pt-BR": {
    ed2kConnect: "Conectar", ed2kDisconnect: "Desconectar",
    ed2kDisconnected: "Desconectado", ed2kServersApplied: "Servidores configurados",
    ed2kTabTransfer: "Transferência", ed2kTabServers: "Servidores", ed2kTabSearch: "Buscar",
    ed2kLinkPlaceholder: "ed2k://|file|...|/", ed2kAddLink: "Baixar",
    ed2kDownloads: "Downloads", ed2kNoDownloads: "Nenhum download ED2K ainda.",
    ed2kServerHost: "servidor.exemplo.org", ed2kAddServer: "Adicionar servidor",
    ed2kNoServers: "Nenhum servidor configurado ainda.", ed2kRemoveServer: "Remover",
    ed2kSearchPlaceholder: "Palavra-chave de busca…", ed2kSearch: "Buscar",
    ed2kNoResults: "Nenhum resultado ainda.", ed2kSearchDownload: "Baixar",
    ed2kNoServersConfigured: "Adicione pelo menos um servidor antes de conectar.",
    ed2kConnectFailed: "Não foi possível aplicar a lista de servidores",
  },
  "zh-CN": {
    ed2kConnect: "连接", ed2kDisconnect: "断开",
    ed2kDisconnected: "未连接", ed2kServersApplied: "服务器已配置",
    ed2kTabTransfer: "传输", ed2kTabServers: "服务器", ed2kTabSearch: "搜索",
    ed2kLinkPlaceholder: "ed2k://|file|...|/", ed2kAddLink: "下载",
    ed2kDownloads: "下载", ed2kNoDownloads: "还没有 ED2K 下载任务。",
    ed2kServerHost: "server.example.org", ed2kAddServer: "添加服务器",
    ed2kNoServers: "还没有配置服务器。", ed2kRemoveServer: "移除",
    ed2kSearchPlaceholder: "搜索关键词…", ed2kSearch: "搜索",
    ed2kNoResults: "还没有结果。", ed2kSearchDownload: "下载",
    ed2kNoServersConfigured: "连接前请至少添加一个服务器。",
    ed2kConnectFailed: "无法应用服务器列表",
  },
};

let locale = localStorage.getItem("apocalipse.language") || "en";
const t = (key) => catalogs[locale]?.[key] || catalogs.en[key] || key;

// Must stay in sync with the THEMES list in set_application_theme
// (main.rs) and the data-theme selectors in styles.css.
const validThemes = ["void", "nebula", "ember", "jade", "plasma", "glacier", "amber", "abyss", "rust", "venom", "wine", "linen", "sky", "blossom", "sage", "sand", "lilac", "mist", "citrus", "coral", "frost"];
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
  document.querySelectorAll("[data-i18n-placeholder]").forEach((node) => { node.placeholder = t(node.dataset.i18nPlaceholder); });
  if (!connected) document.querySelector("#ed2k-connection-status").textContent = t("ed2kDisconnected");
}

const rawInvoke = window.__TAURI__?.core?.invoke;
const invoke = async (command, args = {}) => {
  if (!rawInvoke) throw new Error("Desktop bridge unavailable");
  try {
    return await rawInvoke(command, args);
  } finally {
    if (command !== "record_ui_diagnostic") {
      rawInvoke("record_ui_diagnostic", { level: "INFO", event: "ed2k_ui_command", detail: `command=${command}` }).catch(() => {});
    }
  }
};

let connected = false;

function setTab(name) {
  document.querySelectorAll(".ed2k-tab").forEach((button) => button.classList.toggle("active", button.dataset.ed2kTab === name));
  document.querySelectorAll(".ed2k-panel").forEach((panel) => { panel.hidden = panel.id !== `ed2k-panel-${name}`; });
}
document.querySelectorAll(".ed2k-tab").forEach((button) => {
  button.onclick = () => setTab(button.dataset.ed2kTab);
});

function formatBytes(value) {
  const bytes = Number(value) || 0;
  if (bytes <= 0) return "0 B";
  const units = ["B", "KB", "MB", "GB", "TB"];
  let size = bytes;
  let index = 0;
  while (size >= 1024 && index < units.length - 1) { size /= 1024; index += 1; }
  return `${size >= 100 ? size.toFixed(0) : size.toFixed(1)} ${units[index]}`;
}

async function renderServers() {
  const servers = await invoke("ed2k_list_servers");
  const list = document.querySelector("#ed2k-server-list");
  list.replaceChildren();
  document.querySelector("#ed2k-server-empty").hidden = servers.length > 0;
  for (const server of servers) {
    const row = document.createElement("div");
    row.className = "ed2k-row";
    const name = document.createElement("span");
    name.className = "ed2k-row-name";
    name.textContent = server;
    const remove = document.createElement("button");
    remove.type = "button";
    remove.textContent = t("ed2kRemoveServer");
    remove.onclick = async () => {
      remove.disabled = true;
      try { await invoke("ed2k_remove_server", { server }); await renderServers(); }
      catch (error) { window.alert(String(error)); remove.disabled = false; }
    };
    row.append(name, remove);
    list.append(row);
  }
}

document.querySelector("#ed2k-server-add").onclick = async () => {
  const host = document.querySelector("#ed2k-server-host");
  const port = document.querySelector("#ed2k-server-port");
  const value = `${host.value.trim()}:${port.value.trim() || "4661"}`;
  const button = document.querySelector("#ed2k-server-add");
  button.disabled = true;
  try {
    await invoke("ed2k_add_server", { server: value });
    host.value = "";
    port.value = "";
    await renderServers();
  } catch (error) {
    window.alert(String(error));
  } finally {
    button.disabled = false;
  }
};

function setConnectionState(state) {
  connected = state === "connected";
  const pill = document.querySelector("#ed2k-connection-status");
  pill.dataset.state = state;
  pill.textContent = state === "connected" ? t("ed2kServersApplied") : t("ed2kDisconnected");
  document.querySelector("#ed2k-connect").disabled = connected;
  document.querySelector("#ed2k-disconnect").disabled = !connected;
}

document.querySelector("#ed2k-connect").onclick = async () => {
  const button = document.querySelector("#ed2k-connect");
  button.disabled = true;
  try {
    await invoke("ed2k_connect");
    setConnectionState("connected");
  } catch (error) {
    window.alert(String(error) === "ed2k_no_servers_configured" ? t("ed2kNoServersConfigured") : `${t("ed2kConnectFailed")}: ${error}`);
    button.disabled = false;
  }
};
document.querySelector("#ed2k-disconnect").onclick = async () => {
  const button = document.querySelector("#ed2k-disconnect");
  button.disabled = true;
  try { await invoke("ed2k_disconnect"); }
  catch (error) { console.error(error); }
  setConnectionState("disconnected");
};

async function renderDownloads() {
  const tasks = await invoke("list_downloads");
  const ed2kTasks = tasks.filter((task) => /^ed2k:\/\//i.test(task.source));
  const list = document.querySelector("#ed2k-downloads-list");
  list.replaceChildren();
  document.querySelector("#ed2k-downloads-empty").hidden = ed2kTasks.length > 0;
  for (const task of ed2kTasks) {
    const row = document.createElement("div");
    row.className = "ed2k-row";
    const name = document.createElement("span");
    name.className = "ed2k-row-name";
    name.textContent = task.destination.split(/[\\/]/).pop();
    const progress = document.createElement("span");
    progress.className = "ed2k-row-meta";
    const percent = task.progress_percent != null ? `${task.progress_percent.toFixed(1)}%` : "…";
    progress.textContent = `${percent} · ${formatBytes(task.received)} / ${formatBytes(task.total)}`;
    const speed = document.createElement("span");
    speed.className = "ed2k-row-meta";
    speed.textContent = task.download_speed ? `${formatBytes(task.download_speed)}/s` : "";
    const state = document.createElement("span");
    state.className = "ed2k-row-meta";
    state.textContent = typeof task.state === "string" ? task.state : Object.keys(task.state || {})[0] || "";
    row.append(name, progress, speed, state);
    list.append(row);
  }
}

document.querySelector("#ed2k-link-add").onclick = async () => {
  const input = document.querySelector("#ed2k-link-input");
  const link = input.value.trim();
  if (!link.startsWith("ed2k://")) { window.alert("ed2k://…"); return; }
  const button = document.querySelector("#ed2k-link-add");
  button.disabled = true;
  try {
    const destinationDirectory = await invoke("default_download_directory");
    await invoke("enqueue_download", {
      url: link, destinationDirectory, fileName: null, formatSelection: null,
      torrentSelection: null, mirrors: null, priority: 0, bandwidthLimit: null,
      connectionsOverride: null, context: {},
    });
    input.value = "";
    await renderDownloads();
  } catch (error) {
    window.alert(String(error));
  } finally {
    button.disabled = false;
  }
};

document.querySelector("#ed2k-search-form").onsubmit = async (event) => {
  event.preventDefault();
  const keyword = document.querySelector("#ed2k-search-keyword").value.trim();
  if (!keyword) return;
  const results = document.querySelector("#ed2k-search-results");
  const empty = document.querySelector("#ed2k-search-empty");
  results.replaceChildren();
  empty.hidden = false;
  empty.textContent = "…";
  try {
    const gid = await invoke("ed2k_search", { keyword });
    // eD2K search results stream in over a few seconds; poll briefly.
    let payload = null;
    for (let attempt = 0; attempt < 12; attempt += 1) {
      await new Promise((resolve) => setTimeout(resolve, 1000));
      payload = await invoke("ed2k_search_results", { gid });
      if (Array.isArray(payload?.results) && payload.results.length) break;
    }
    const items = Array.isArray(payload?.results) ? payload.results : [];
    if (!items.length) {
      empty.hidden = false;
      empty.textContent = t("ed2kNoResults");
      return;
    }
    empty.hidden = true;
    for (const item of items) {
      const row = document.createElement("div");
      row.className = "ed2k-row";
      const name = document.createElement("span");
      name.className = "ed2k-row-name";
      name.textContent = item.name || item.fileName || "?";
      const size = document.createElement("span");
      size.className = "ed2k-row-meta";
      size.textContent = formatBytes(item.size || item.length);
      const sources = document.createElement("span");
      sources.className = "ed2k-row-meta";
      sources.textContent = item.sources != null ? `${item.sources}` : "";
      const download = document.createElement("button");
      download.type = "button";
      download.className = "primary";
      download.textContent = t("ed2kSearchDownload");
      download.onclick = async () => {
        if (!item.link) return;
        download.disabled = true;
        try {
          const destinationDirectory = await invoke("default_download_directory");
          await invoke("enqueue_download", {
            url: item.link, destinationDirectory, fileName: null, formatSelection: null,
            torrentSelection: null, mirrors: null, priority: 0, bandwidthLimit: null,
            connectionsOverride: null, context: {},
          });
          setTab("transfer");
          await renderDownloads();
        } catch (error) {
          window.alert(String(error));
          download.disabled = false;
        }
      };
      row.append(name, size, sources, download);
      results.append(row);
    }
  } catch (error) {
    empty.hidden = false;
    empty.textContent = String(error);
  }
};

syncPresentation();
window.addEventListener("storage", syncPresentation);
invoke("get_application_theme")
  .then((theme) => {
    if (typeof theme === "string" && validThemes.includes(theme)) {
      localStorage.setItem("apocalipse.theme", theme);
      syncPresentation();
    }
  })
  .catch(console.error);
window.__TAURI__?.event?.listen?.("theme-changed", (event) => {
  const theme = event.payload;
  if (typeof theme === "string" && validThemes.includes(theme)) {
    localStorage.setItem("apocalipse.theme", theme);
    syncPresentation();
  }
}).catch(console.error);

renderServers().catch(console.error);
renderDownloads().catch(console.error);
setInterval(() => renderDownloads().catch(() => {}), 3000);
invoke("record_ui_diagnostic", { level: "INFO", event: "ed2k_window_opened", detail: "" }).catch(() => {});
