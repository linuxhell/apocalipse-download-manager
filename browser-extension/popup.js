let media = [], selected = "video", locale = "en";
const messages = {
  en: { mediaIntelligence: "Media intelligence", video: "Video", audio: "Audio", images: "Images", download: "Download", empty: "No media detected in this tab." },
  pt_BR: { mediaIntelligence: "Inteligência de mídia", video: "Vídeo", audio: "Áudio", images: "Imagens", download: "Baixar", empty: "Nenhuma mídia detectada nesta aba." },
  zh_CN: { mediaIntelligence: "媒体智能", video: "视频", audio: "音频", images: "图片", download: "下载", empty: "此标签页未检测到媒体。" }
};
const t = key => messages[locale]?.[key] || messages.en[key] || key;
const translate = () => { document.querySelectorAll("[data-i18n]").forEach(el => el.textContent = t(el.dataset.i18n)); document.querySelectorAll("[data-i18n-title]").forEach(el => el.title = t(el.dataset.i18nTitle)); };
const render = () => {
  const root = document.querySelector("#items"); root.textContent = "";
  const matches = media.filter(x => x.kind === selected);
  if (!matches.length) { root.innerHTML = `<div id="empty">${t("empty")}</div>`; return; }
  for (const item of matches) {
    const row = document.querySelector("#row").content.cloneNode(true);
    row.querySelector("img").src = item.thumbnail || "";
    row.querySelector("b").textContent = item.title || item.url.split("/").pop();
    row.querySelector("small").textContent = item.size ? `${item.size} bytes` : new URL(item.url).hostname;
    row.querySelector("button").onclick = () => chrome.runtime.sendMessage({ type: "APOCALIPSE_DOWNLOAD", item });
    root.append(row);
  }
};
document.querySelectorAll("nav button").forEach(button => button.onclick = () => { selected = button.dataset.kind; document.querySelectorAll("nav button").forEach(x => x.classList.toggle("active", x === button)); render(); });
chrome.storage.local.get({ language: "en" }, ({ language }) => { locale = language; document.querySelector("#language").value = locale; translate(); chrome.tabs.query({ active: true, currentWindow: true }, tabs => chrome.tabs.sendMessage(tabs[0].id, { type: "APOCALIPSE_SCAN" }, response => { media = response?.media || []; render(); })); });
document.querySelector("#language").onchange = event => { locale = event.target.value; chrome.storage.local.set({ language: locale }); translate(); render(); };
