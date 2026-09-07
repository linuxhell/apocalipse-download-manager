(() => {
  if (!/(^|\.)rapidgator\.net$/i.test(location.hostname)) return;

  let replaying = false;

  const finalRapidgatorUrl = (value) => {
    try {
      const url = new URL(value, location.href);
      if (!/(^|\.)rapidgator\.net$/i.test(url.hostname)) return null;
      if (!/^\/download\//i.test(url.pathname)) return null;
      return url.href;
    } catch {
      return null;
    }
  };

  const replayOriginalClick = (anchor) => {
    replaying = true;
    try {
      anchor.click();
    } finally {
      setTimeout(() => { replaying = false; }, 0);
    }
  };

  document.addEventListener("click", (event) => {
    if (replaying || event.defaultPrevented || event.button !== 0) return;
    if (event.altKey || event.ctrlKey || event.metaKey || event.shiftKey) return;

    const anchor = event.target?.closest?.("a[href]");
    if (!anchor) return;
    const url = finalRapidgatorUrl(anchor.href);
    if (!url) return;

    // Rapidgator's /download/<token> URL can be single-use/session-sensitive.
    // Stop the browser before it consumes the URL and let Apocalipse perform
    // the first request with the browser session context. If the bridge cannot
    // accept the handoff, replay the original click so the download is not lost.
    event.preventDefault();
    event.stopImmediatePropagation();

    chrome.runtime.sendMessage({
      type: "APOCALIPSE_DOWNLOAD",
      item: {
        url,
        requestUrls: [url],
        userAgent: navigator.userAgent,
        kind: "file",
        title: anchor.getAttribute("download") || null,
      },
    }, (result) => {
      const failed = Boolean(chrome.runtime.lastError) || result?.target !== "apocalipse";
      if (failed) replayOriginalClick(anchor);
    });
  }, true);
})();
