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

  const replayOriginalClick = async (anchor) => {
    replaying = true;
    try {
      // Tell the generic download interceptor to ignore the fallback browser
      // transfer, otherwise it could cancel the fresh Rapidgator request again.
      await chrome.runtime.sendMessage({ type: "APOCALIPSE_BYPASS_NEXT", ttlMs: 15000 }).catch(() => {});
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

    // Own the final Rapidgator link before Chrome can consume the one-shot URL.
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
      const failed = Boolean(chrome.runtime.lastError) || result?.target !== "apocalipse";
      if (failed) void replayOriginalClick(anchor);
    });
  }, true);
})();
