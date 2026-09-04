(() => {
  const absolute = (value) => {
    try { return new URL(value, location.href).href; } catch { return null; }
  };
  const titleFor = (element) => element?.getAttribute?.("aria-label") || element?.title || element?.alt || document.title;
  const thumbnailFor = (element, kind) => {
    if (kind === "audio") return "";
    if (element?.tagName === "IMG") return element.currentSrc || element.src || "";
    return element?.poster || document.querySelector('meta[property="og:image"]')?.content || element?.closest?.("figure,article")?.querySelector?.("img")?.currentSrc || "";
  };
  const collect = () => {
    const items = new Map();
    const add = (url, kind, element, thumbnail) => {
      url = absolute(url);
      if (!url || !/^https?:/.test(url)) return;
      items.set(`${kind}:${url}`, { url, kind, thumbnail: thumbnail ?? thumbnailFor(element, kind), title: titleFor(element), size: null });
    };
    document.querySelectorAll("video").forEach((element) => {
      add(element.currentSrc || element.src, "video", element);
      element.querySelectorAll("source").forEach((source) => add(source.src, "video", element));
    });
    if (/^(?:www\.)?youtube\.com$/.test(location.hostname) && location.pathname === "/watch") {
      const videoId = new URL(location.href).searchParams.get("v");
      add(location.href, "video", document.querySelector("video"), document.querySelector('meta[property="og:image"]')?.content || (videoId ? `https://i.ytimg.com/vi/${videoId}/hqdefault.jpg` : ""));
    }
    document.querySelectorAll("audio").forEach((element) => {
      add(element.currentSrc || element.src, "audio", element);
      element.querySelectorAll("source").forEach((source) => add(source.src, "audio", element));
    });
    document.querySelectorAll("img").forEach((element) => add(element.currentSrc || element.src, "image", element));
    performance.getEntriesByType("resource").forEach((entry) => {
      if (/\.m3u8(?:$|[?#])/i.test(entry.name)) add(entry.name, "video", null, document.querySelector("video")?.poster || "");
    });
    return [...items.values()];
  };
  const downloadLabel = () => {
    const value = (navigator.language || "en").toLowerCase();
    return value.startsWith("zh") ? "下载" : value.startsWith("pt") ? "Baixar" : "Download";
  };
  const hlsForPage = () => {
    const urls = [...new Set(performance.getEntriesByType("resource").map((entry) => entry.name)
      .filter((url) => /\.m3u8(?:$|[?#])/i.test(url)))];
    const masters = urls.filter((url) => /(?:\/master\/|master\.m3u8)/i.test(url));
    return { candidates: urls, fallback: urls.at(-1) || masters.at(-1) || null };
  };
  const downloadUrlFor = (element) => {
    if (element.tagName === "VIDEO" && /^(?:www\.)?youtube\.com$/.test(location.hostname) && location.pathname === "/watch") return location.href;
    const direct = absolute(element.currentSrc || element.src);
    if (direct && /^https?:/.test(direct)) return direct;
    if (element.tagName === "VIDEO") return hlsForPage().fallback;
    return null;
  };
  const resolveDownloadUrl = async (element) => {
    const immediate = downloadUrlFor(element);
    if (element.tagName !== "VIDEO" || !immediate || !/\.m3u8(?:$|[?#])/i.test(immediate)) return immediate;
    const hls = hlsForPage();
    try {
      const selected = await chrome.runtime.sendMessage({
        type: "APOCALIPSE_SELECT_HLS",
        urls: hls.candidates,
        expectedDuration: Number.isFinite(element.duration) ? element.duration : null,
      });
      return selected || { url: immediate, duration: null };
    } catch { return { url: immediate, duration: null }; }
  };
  let overlayTimer;
  const installOverlays = () => {
    if (/(^|\.)chatgpt\.com$/.test(location.hostname)) return;
    document.querySelectorAll("video,audio").forEach((element) => {
      if (element.dataset.apocalipseButton) return;
      const isYouTubeVideo = element.tagName === "VIDEO" && /^(?:www\.)?youtube\.com$/.test(location.hostname) && location.pathname === "/watch";
      const url = downloadUrlFor(element);
      if (!url || !/^https?:/.test(url)) return;
      element.dataset.apocalipseButton = "1";
      const button = document.createElement("button");
      button.type = "button";
      button.className = "apocalipse-media-download";
      button.textContent = `⇩ ${downloadLabel()}`;
      button.title = "Apocalipse Download Manager";
      button.addEventListener("click", async (event) => {
        event.preventDefault();
        event.stopPropagation();
        const resolved = await resolveDownloadUrl(element);
        const currentUrl = resolved?.url || resolved || (isYouTubeVideo ? location.href : url);
        if (!currentUrl) return;
        chrome.runtime.sendMessage({ type: "APOCALIPSE_DOWNLOAD", item: { url: currentUrl, duration: resolved?.duration || null, kind: element.tagName.toLowerCase(), title: document.title, thumbnail: thumbnailFor(element, "video") } });
      });
      const position = () => {
        if (!element.isConnected) { button.remove(); return; }
        const rect = element.getBoundingClientRect();
        button.style.left = `${Math.max(6, rect.left + scrollX + 8)}px`;
        button.style.top = `${Math.max(6, rect.top + scrollY + 8)}px`;
        button.hidden = rect.width < 100 || rect.height < 55;
      };
      document.documentElement.append(button);
      position();
      addEventListener("scroll", position, { passive: true });
      addEventListener("resize", position, { passive: true });
    });
  };
  const scheduleOverlays = () => {
    clearTimeout(overlayTimer);
    overlayTimer = setTimeout(installOverlays, 250);
  };
  const style = document.createElement("style");
  style.textContent = ".apocalipse-media-download{position:absolute!important;z-index:2147483647!important;border:1px solid #73efff!important;border-radius:8px!important;padding:7px 11px!important;background:linear-gradient(135deg,#18d5ec,#348fff)!important;color:#031219!important;font:700 12px system-ui!important;box-shadow:0 4px 18px #0009!important;cursor:pointer!important}";
  document.documentElement.append(style);
  new MutationObserver(scheduleOverlays).observe(document.documentElement, { childList: true, subtree: true, attributes: true, attributeFilter: ["src", "poster"] });
  scheduleOverlays();
  setInterval(scheduleOverlays, 2000);
  chrome.runtime.onMessage.addListener((message, _sender, reply) => {
    if (message?.type !== "APOCALIPSE_SCAN") return;
    const found = collect();
    (async () => {
      let selectedItems = found;
      const hls = found.filter((item) => item.kind === "video" && /\.m3u8(?:$|[?#])/i.test(item.url));
      if (hls.length > 1) {
        const analyzed = await chrome.runtime.sendMessage({ type: "APOCALIPSE_ANALYZE_HLS", urls: hls.map((item) => item.url), expectedDuration: Number.isFinite(document.querySelector("video")?.duration) ? document.querySelector("video").duration : null });
        const details = new Map((analyzed || []).map((item) => [item.url, item]));
        selectedItems = found.map((item) => details.has(item.url) ? { ...item, ...details.get(item.url) } : item);
      }
      return Promise.all(selectedItems.map(async (item) => {
      try {
        return { ...item, ...(await chrome.runtime.sendMessage({ type: "APOCALIPSE_PROBE", url: item.url })) };
      } catch {
        return item;
      }
      }));
    })().then((media) => reply({ pageUrl: location.href, media })).catch(() => reply({ pageUrl: location.href, media: found }));
    return true;
  });
})();
