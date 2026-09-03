chrome.runtime.onMessage.addListener((message, sender, reply) => {
  if (message?.type === "APOCALIPSE_MEDIA" && sender.tab?.id) {
    chrome.storage.session.set({ [`media:${sender.tab.id}`]: message.media });
  }
  if (message?.type === "APOCALIPSE_PROBE") {
    fetch(message.url, { method: "HEAD", credentials: "include", redirect: "follow" })
      .then((response) => reply({
        size: Number(response.headers.get("content-length")) || null,
        contentType: response.headers.get("content-type") || "",
      }))
      .catch(() => reply({ size: null }));
    return true;
  }
  if (message?.type === "APOCALIPSE_DOWNLOAD" && message.item?.url) {
    chrome.downloads.download({ url: message.item.url, saveAs: true }, (downloadId) => {
      reply({ downloadId, error: chrome.runtime.lastError?.message });
    });
    return true;
  }
});
