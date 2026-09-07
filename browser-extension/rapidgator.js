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
      // Rapidgator free URLs are one-shot. If debugger/bridge setup fails before
      // the browser request begins, keep the original page intact instead of
      // replaying the click and wasting the token. The user can click again
      // after the transport problem is corrected.
      if (chrome.runtime.lastError || result?.target !== "apocalipse") {
        console.warn("Apocalipse Rapidgator transport did not start", result?.error || chrome.runtime.lastError?.message || "unknown");
      }
      // Once the browser-authenticated request starts, never replay this URL.
    });
  }, true);
})();
