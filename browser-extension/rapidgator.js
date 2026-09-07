(() => {
  if (!/(^|\.)rapidgator\.net$/i.test(location.hostname)) return;

  let replaying = false;
  const transportFrames = new Map();

  const finalRapidgatorUrl = (value) => {
    try {
      const url = new URL(value, location.href);
      if (!/^s\d+\.rapidgator\.net$/i.test(url.hostname)) return null;
      if (!/^\/download\/[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}\/?$/i.test(url.pathname)) return null;
      return url.href;
    } catch {
      return null;
    }
  };

  const replayOriginalClick = async (anchor) => {
    replaying = true;
    try {
      await chrome.runtime.sendMessage({ type: "APOCALIPSE_BYPASS_NEXT", ttlMs: 15000 }).catch(() => {});
      anchor.click();
    } finally {
      setTimeout(() => { replaying = false; }, 0);
    }
  };

  chrome.runtime.onMessage.addListener((message, _sender, reply) => {
    if (message?.type === "APOCALIPSE_RAPIDGATOR_START_BROWSER_TRANSPORT" && message.url && message.transportId) {
      try {
        const url = finalRapidgatorUrl(message.url);
        if (!url) throw new Error("invalid_rapidgator_transport_url");
        const frame = document.createElement("iframe");
        frame.hidden = true;
        frame.setAttribute("aria-hidden", "true");
        frame.style.cssText = "display:none!important;width:0!important;height:0!important;border:0!important";
        frame.src = url;
        transportFrames.set(message.transportId, frame);
        (document.documentElement || document.body).append(frame);
        reply({ started: true });
      } catch (error) {
        reply({ started: false, error: String(error) });
      }
      return;
    }
    if (message?.type === "APOCALIPSE_RAPIDGATOR_BROWSER_TRANSPORT_DONE" && message.transportId) {
      const frame = transportFrames.get(message.transportId);
      if (frame) frame.remove();
      transportFrames.delete(message.transportId);
      return;
    }
  });

  document.addEventListener("click", (event) => {
    if (replaying || event.defaultPrevented || event.button !== 0) return;
    if (event.altKey || event.ctrlKey || event.metaKey || event.shiftKey) return;

    const anchor = event.target?.closest?.("a[href]");
    if (!anchor) return;
    const url = finalRapidgatorUrl(anchor.href);
    if (!url) return;

    event.preventDefault();
    event.stopImmediatePropagation();

    chrome.runtime.sendMessage({
      type: "APOCALIPSE_RAPIDGATOR_DOWNLOAD",
      item: {
        url,
        userAgent: navigator.userAgent,
        kind: "file",
        title: anchor.getAttribute("download") || null,
      },
    }, (result) => {
      const failedBeforeBrowserRequest = Boolean(chrome.runtime.lastError) || result?.target !== "apocalipse";
      if (failedBeforeBrowserRequest) void replayOriginalClick(anchor);
      // Once the browser-authenticated request starts, never replay this one-shot
      // URL. If the stream later fails the token has already been consumed.
    });
  }, true);
})();
