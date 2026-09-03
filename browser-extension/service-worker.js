chrome.runtime.onMessage.addListener((message, sender) => {
  if (message?.type === "APOCALIPSE_MEDIA" && sender.tab?.id) {
    chrome.storage.session.set({ [`media:${sender.tab.id}`]: message.media });
  }
});

