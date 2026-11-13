(() => {
  // Microsoft Update file IDs are stable until their signed URL expires.
  // A UUID alone does not make this endpoint a consumed, single-use link.
  const isMicrosoftUpdateFile = (value) => {
    try {
      const url = new URL(value);
      return /^https?:$/.test(url.protocol) && !url.username && !url.password && !url.port
        && /(^|\.)dl\.delivery\.mp\.microsoft\.com$/i.test(url.hostname)
        && /^\/filestreamingservice\/files\/[0-9a-f]{8}(?:-[0-9a-f]{4}){3}-[0-9a-f]{12}\/?$/i.test(url.pathname);
    } catch { return false; }
  };
  const uupFileName = (pageUrl, downloadUrl) => {
    if (!isMicrosoftUpdateFile(downloadUrl)) return null;
    try {
      const page = new URL(pageUrl);
      if (page.protocol !== "https:" || page.username || page.password || page.port
        || !/^(?:www\.)?uupdump\.net$/i.test(page.hostname) || page.pathname !== "/getfile.php") return null;
      const name = page.searchParams.get("file") || "";
      if (name.length > 240 || /[\x00-\x1f\x7f<>:"/\\|?*]/.test(name)
        || /^(?:con|prn|aux|nul|com[1-9]|lpt[1-9])\./i.test(name)) return null;
      return /\.(?:msu|cab|esd|wim|psf)$/i.test(name) ? name : null;
    } catch { return null; }
  };
  globalThis.ADM_DOWNLOAD_URLS = Object.freeze({ isMicrosoftUpdateFile, uupFileName });
})();
