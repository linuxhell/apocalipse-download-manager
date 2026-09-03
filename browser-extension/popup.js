let media = [], selected = "video", locale = "en";
const messages = {
  en: { mediaIntelligence: "Media intelligence", video: "Video", audio: "Audio", images: "Images", download: "Download", empty: "No media detected in this tab.", unknownSize: "Size unavailable" },
  pt_BR: { mediaIntelligence: "Inteligência de mídia", video: "Vídeo", audio: "Áudio", images: "Imagens", download: "Download", empty: "Nenhuma mídia detectada nesta aba.", unknownSize: "Tamanho indisponível" },
  zh_CN: { mediaIntelligence: "媒体智能", video: "视频", audio: "音频", images: "图片", download: "下载", empty: "此标签页未检测到媒体。", unknownSize: "大小未知" }
};
const t = (key) => messages[locale]?.[key] || messages.en[key] || key;
const formatBytes = (bytes) => {
  if (!bytes) return t("unknownSize");
  const units = ["B", "KB", "MB", "GB"];
  const index = Math.min(Math.floor(Math.log(bytes) / Math.log(1024)), units.length - 1);
  return `${(bytes / 1024 ** index).toFixed(index ? 1 : 0)} ${units[index]}`;
};
const translate = () => document.querySelectorAll("[data-i18n]").forEach((element) => {
  element.textContent = t(element.dataset.i18n);
});
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
      image.src = item.thumbnail || item.url;
    }
    row.querySelector("b").textContent = item.title || item.url.split("/").pop();
    row.querySelector("small").textContent = `${formatBytes(item.size)} · ${new URL(item.url).hostname}`;
    const button = row.querySelector("button");
    button.textContent = t("download");
    button.onclick = () => chrome.runtime.sendMessage({ type: "APOCALIPSE_DOWNLOAD", item });
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
document.querySelector("#language").onchange = (event) => {
  locale = event.target.value;
  chrome.storage.local.set({ language: locale });
  translate();
  render();
};
