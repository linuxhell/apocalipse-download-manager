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
let previousSample = { at: performance.now(), bytes: 0 };
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

function renderDownloads() {
  const list = document.querySelector("#download-list");
  const empty = document.querySelector("#empty");
  list.replaceChildren();
  list.hidden = downloads.length === 0;
  empty.hidden = downloads.length !== 0;
  for (const task of downloads) {
    const row = document.createElement("article");
    row.className = "download-row";
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
    details.textContent = task.total
      ? `${formatBytes(task.received)} / ${formatBytes(task.total)} · ${percent.toFixed(1)}%`
      : formatBytes(task.received);
    progress.append(bar);
    info.append(progress, details);
    const state = Object.assign(document.createElement("span"), {
      className: "download-state",
      textContent: stateName(task.state),
    });
    if (typeof task.state === "object")
      state.title = task.state.failed?.message || "";
    row.append(icon, info, state);
    list.append(row);
  }
  document.querySelector(".metrics article:nth-child(4) strong").textContent =
    downloads.filter((task) => task.state === "queued").length;
  document.querySelector(".metrics article:nth-child(3) strong").textContent =
    downloads.filter((task) => task.state === "completed").length;
  const now = performance.now();
  const bytes = downloads.reduce((sum, task) => sum + task.received, 0);
  const elapsed = Math.max(0.001, (now - previousSample.at) / 1000);
  const speed = Math.max(0, bytes - previousSample.bytes) / elapsed;
  document.querySelector(".metrics article:first-child strong").textContent =
    `${formatBytes(speed)}/s`;
  previousSample = { at: now, bytes };
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
    renderDownloads();
  } catch (error) {
    console.error(error);
  }
}

const dialog = document.querySelector("#add-dialog");
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
setInterval(refreshDownloads, 750);
