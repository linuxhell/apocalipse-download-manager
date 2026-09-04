let media = [], selected = "video", locale = "en";
const messages = {
  en: { mediaIntelligence: "Media intelligence", video: "Video", audio: "Audio", images: "Images", download: "Download", empty: "No media detected in this tab.", unknownSize: "Size unavailable", connected: "Connected to Apocalipse", disconnected: "Disconnected", pairingToken: "Pairing token", connect: "Connect", recommended: "Recommended" },
  pt_BR: { mediaIntelligence: "Inteligência de mídia", video: "Vídeo", audio: "Áudio", images: "Imagens", download: "Download", empty: "Nenhuma mídia detectada nesta aba.", unknownSize: "Tamanho indisponível", connected: "Conectada ao Apocalipse", disconnected: "Desconectada", pairingToken: "Token de pareamento", connect: "Conectar", recommended: "Recomendada" },
  zh_CN: { mediaIntelligence: "媒体智能", video: "视频", audio: "音频", images: "图片", download: "下载", empty: "此标签页未检测到媒体。", unknownSize: "大小未知", connected: "已连接到 Apocalipse", disconnected: "未连接", pairingToken: "配对令牌", connect: "连接", recommended: "推荐" }
};
const t = (key) => messages[locale]?.[key] || messages.en[key] || key;
const formatBytes = (bytes) => {
  if (!bytes) return t("unknownSize");
  const units = ["B", "KB", "MB", "GB"];
  const index = Math.min(Math.floor(Math.log(bytes) / Math.log(1024)), units.length - 1);
  return `${(bytes / 1024 ** index).toFixed(index ? 1 : 0)} ${units[index]}`;
};
const formatDuration = (seconds) => {
  if (!Number.isFinite(seconds)) return "";
  const hours = Math.floor(seconds / 3600), minutes = Math.floor((seconds % 3600) / 60), rest = Math.floor(seconds % 60);
  return [hours, minutes, rest].filter((_, index) => index || hours).map((value) => String(value).padStart(2, "0")).join(":");
};
const translate = () => document.querySelectorAll("[data-i18n]").forEach((element) => {
  element.textContent = t(element.dataset.i18n);
});
const setBridgeStatus = (connected) => {
  document.querySelector("#bridge-dot").classList.toggle("connected", connected);
  const label = document.querySelector("#bridge-label");
  label.dataset.i18n = connected ? "connected" : "disconnected";
  label.textContent = t(label.dataset.i18n);
};
const showBridgeError = (error) => {
  const label = document.querySelector("#bridge-label");
  label.removeAttribute("data-i18n");
  const invalid = String(error).includes("401");
  if (locale === "pt_BR") label.textContent = invalid ? "Token de pareamento inválido." : "Não foi possível encontrar o Apocalipse. Mantenha o programa aberto.";
  else if (locale === "zh_CN") label.textContent = invalid ? "配对令牌无效。" : "无法连接 Apocalipse。请保持桌面程序运行。";
  else label.textContent = invalid ? "Invalid pairing token." : "Apocalipse is not reachable. Keep the desktop app open.";
};
const render = () => {
  const root = document.querySelector("#items");
  root.textContent = "";
  const matches = media.filter((item) => item.kind === selected);
  if (!matches.length) {
    const empty = document.createElement("div");
    empty.id = "empty";
    empty.textContent = t("empty");
    root.append(empty);
    return;
  }
  for (const item of matches) {
    const row = document.querySelector("#row").content.cloneNode(true);
    const image = row.querySelector("img");
    const audio = row.querySelector(".audio-icon");
    if (selected === "audio") {
      image.hidden = true;
      audio.hidden = false;
    } else {
      image.src = item.thumbnail || (selected === "image" ? item.url : chrome.runtime.getURL("icons/alien-48.png"));
      image.onerror = () => {
        image.onerror = null;
        image.src = chrome.runtime.getURL("icons/alien-48.png");
      };
    }
    row.querySelector("b").textContent = item.title || item.url.split("/").pop();
    row.querySelector("small").textContent = [formatBytes(item.size), formatDuration(item.duration), item.recommended ? t("recommended") : "", new URL(item.url).hostname].filter(Boolean).join(" · ");
    const button = row.querySelector("button");
    button.textContent = t("download");
    button.onclick = () => chrome.runtime.sendMessage({ type: "APOCALIPSE_DOWNLOAD", item }, (result) => {
      if (result?.target === "error" || chrome.runtime.lastError) {
        showBridgeError(result?.error || chrome.runtime.lastError?.message || "unavailable");
      }
    });
    root.append(row);
  }
};
document.querySelectorAll("nav button").forEach((button) => {
  button.onclick = () => {
    selected = button.dataset.kind;
    document.querySelectorAll("nav button").forEach((item) => item.classList.toggle("active", item === button));
    render();
  };
});
chrome.storage.local.get({ language: "en" }, ({ language }) => {
  locale = language;
  document.querySelector("#language").value = locale;
  translate();
  chrome.tabs.query({ active: true, currentWindow: true }, (tabs) => {
    chrome.tabs.sendMessage(tabs[0].id, { type: "APOCALIPSE_SCAN" }, (response) => {
      media = response?.media || [];
      render();
    });
  });
});
chrome.storage.local.get({ pairingToken: "" }, ({ pairingToken }) => {
  document.querySelector("#pairing-token").value = pairingToken;
  chrome.runtime.sendMessage({ type: "APOCALIPSE_BRIDGE_STATUS" }, (status) => setBridgeStatus(Boolean(status?.connected)));
});
setInterval(() => {
  chrome.runtime.sendMessage({ type: "APOCALIPSE_BRIDGE_STATUS" }, (status) => {
    if (chrome.runtime.lastError) return;
    setBridgeStatus(Boolean(status?.connected));
  });
}, 5000);
document.querySelector("#connect").onclick = () => {
  const token = document.querySelector("#pairing-token").value;
  const button = document.querySelector("#connect");
  button.disabled = true;
  chrome.runtime.sendMessage({ type: "APOCALIPSE_PAIR", token }, (status) => {
    button.disabled = false;
    setBridgeStatus(Boolean(status?.connected));
    if (!status?.connected) showBridgeError(status?.error || chrome.runtime.lastError?.message || "unavailable");
  });
};
document.querySelector("#language").onchange = (event) => {
  locale = event.target.value;
  chrome.storage.local.set({ language: locale });
  translate();
  render();
};
