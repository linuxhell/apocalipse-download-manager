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
    queued: "Queued",
    inspecting: "Inspecting",
    downloading: "Downloading",
    failed: "Failed",
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
    queued: "Na fila",
    inspecting: "Analisando",
    downloading: "Baixando",
    failed: "Falhou",
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
    queued: "已排队",
    inspecting: "正在检查",
    downloading: "正在下载",
    failed: "失败",
  },
};

let locale = localStorage.getItem("apocalipse.language") || "en";
let downloads = [];
let activeFilter = "all";
let overallSpeed = 0;
const selectedIds = new Set();
const speedSamples = new Map();
const t = (key) => catalogs[locale]?.[key] || catalogs.en[key] || key;
const invoke = (command, args = {}) => {
  const bridge = window.__TAURI__?.core?.invoke;
  if (!bridge) throw new Error("Desktop bridge unavailable in preview");
  return bridge(command, args);
};

function stateName(state) {
  return typeof state === "string" ? t(state) : Object.keys(state)[0];
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
    let speed = previous?.speed || 0;
    let activeAt = previous?.activeAt || 0;
    if (previous) {
      const elapsed = Math.max(0.001, (now - previous.at) / 1000);
      const delta = Math.max(0, task.received - previous.bytes);
      if (delta > 0) {
        speed = delta / elapsed;
        activeAt = now;
      } else if (now - activeAt > 2000) {
        speed = 0;
      }
    }
    speedSamples.set(task.id, {
      at: now,
      bytes: task.received,
      speed,
      activeAt,
    });
    if (task.state === "downloading") overallSpeed += speed;
  }
}

function visibleDownloads() {
  if (activeFilter === "completed")
    return downloads.filter((task) => task.state === "completed");
  if (activeFilter === "active")
    return downloads.filter((task) => task.state !== "completed");
  return downloads;
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
    const percent = task.total
      ? Math.min(100, (task.received / task.total) * 100)
      : 0;
    bar.style.width = `${percent}%`;
    const details = document.createElement("small");
    const speed = speedSamples.get(task.id)?.speed || 0;
    const progressText = task.total
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
    row.append(select, icon, info, state);
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
}

function translate() {
  document.documentElement.lang = locale;
  document
    .querySelectorAll("[data-i18n]")
    .forEach((element) => (element.textContent = t(element.dataset.i18n)));
  document.querySelector("#language").value = locale;
  renderDownloads();
}

async function refreshDownloads() {
  try {
    downloads = await invoke("list_downloads");
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
document
  .querySelectorAll("[data-dialog-close]")
  .forEach((button) => (button.onclick = () => dialog.close()));
document.querySelectorAll("#add,#empty-add").forEach(
  (button) =>
    (button.onclick = () => {
      document.querySelector("#analysis").hidden = true;
      document.querySelector("#enqueue").hidden = true;
      document.querySelector("#analyze").hidden = false;
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
    } finally {
      button.disabled = false;
    }
  };
});
document.querySelector("#url").oninput = () => {
  document.querySelector("#analysis").hidden = true;
  document.querySelector("#enqueue").hidden = true;
  document.querySelector("#analyze").hidden = false;
};
document.querySelector("#analyze").onclick = async () => {
  const url = document.querySelector("#url");
  if (!url.reportValidity()) return;
  const box = document.querySelector("#analysis");
  box.hidden = false;
  box.textContent = "…";
  try {
    const plan = await invoke("inspect_url", { url: url.value });
    box.textContent = `${plan.primary} · ${plan.reason}`;
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
    downloads.push(await invoke("enqueue_download", { url: url.value }));
    renderDownloads();
    dialog.close();
    url.value = "";
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
