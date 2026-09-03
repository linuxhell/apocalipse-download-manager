(() => {
  const absolute = (value) => {
    try { return new URL(value, location.href).href; } catch { return null; }
  };
  const titleFor = (element) => element?.getAttribute?.("aria-label") || element?.title || element?.alt || document.title;
  const thumbnailFor = (element, kind) => {
    if (kind === "audio") return "";
    return element?.poster || element?.currentSrc || element?.src || element?.closest?.("figure,article")?.querySelector?.("img")?.currentSrc || "";
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
    return value.startsWith("zh") ? "下载" : "Download";
  };
  let overlayTimer;
  const installOverlays = () => {
    document.querySelectorAll("video,audio,img").forEach((element) => {
      if (element.dataset.apocalipseButton || (element.tagName === "IMG" && (element.clientWidth < 120 || element.clientHeight < 70))) return;
      const url = absolute(element.currentSrc || element.src);
      if (!url || !/^https?:/.test(url)) return;
      element.dataset.apocalipseButton = "1";
      const button = document.createElement("button");
      button.type = "button";
      button.className = "apocalipse-media-download";
      button.textContent = `⇩ ${downloadLabel()}`;
      button.title = "Apocalipse Download Manager";
      button.addEventListener("click", (event) => {
        event.preventDefault();
        event.stopPropagation();
        chrome.runtime.sendMessage({ type: "APOCALIPSE_DOWNLOAD", item: { url, kind: element.tagName.toLowerCase(), title: titleFor(element), thumbnail: thumbnailFor(element, "video") } });
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
  chrome.runtime.onMessage.addListener((message, _sender, reply) => {
    if (message?.type !== "APOCALIPSE_SCAN") return;
    const found = collect();
    Promise.all(found.map(async (item) => {
      try {
        return { ...item, ...(await chrome.runtime.sendMessage({ type: "APOCALIPSE_PROBE", url: item.url })) };
      } catch {
        return item;
      }
    })).then((media) => reply({ pageUrl: location.href, media }));
    return true;
  });
})();
