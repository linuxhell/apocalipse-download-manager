(() => {
  if (!/(^|\.)rapidgator\.net$/i.test(location.hostname)) return;

  let replaying = false;

  const finalRapidgatorUrl = (value) => {
    try {
      const url = new URL(value, location.href);
      // The real free-download handoff uses a numbered CDN host such as
      // s14.rapidgator.net or s107.rapidgator.net. Never intercept forms or
      // intermediate routes on rapidgator.net itself (for example
      // /download/captcha), because those must stay entirely in the browser.
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

    // Own only the final one-shot CDN URL, after CAPTCHA/submit has completed.
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
