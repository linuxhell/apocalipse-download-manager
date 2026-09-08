(() => {
  const validUrl = (value) => {
    try {
      const url = new URL(String(value || "").replaceAll("\\/", "/"), location.href);
      return /(^|\.)tiktok\.com$/i.test(url.hostname) && /\/@[^/]+\/video\/\d+/i.test(url.pathname)
        ? url.href
        : null;
    } catch { return null; }
  };

  const videoForButton = (button) => {
    const point = button.getBoundingClientRect();
    const x = point.left + point.width / 2;
    const y = point.top + point.height / 2;
    return [...document.querySelectorAll("video")]
      .map((video) => ({ video, rect: video.getBoundingClientRect() }))
      .filter(({ rect }) => rect.width >= 100 && rect.height >= 55 && rect.bottom > 0 && rect.top < innerHeight)
      .sort((left, right) => {
        const distance = (rect) => Math.hypot(
          Math.max(rect.left - x, 0, x - rect.right),
          Math.max(rect.top - y, 0, y - rect.bottom),
        );
        return distance(left.rect) - distance(right.rect);
      })[0]?.video || null;
  };

  const urlInside = (root) => {
    for (const anchor of root?.querySelectorAll?.('a[href*="/video/"]') || []) {
      const url = validUrl(anchor.href);
      if (url) return url;
    }
    const markup = String(root?.innerHTML || "").replaceAll("\\/", "/");
    const path = markup.match(/\/@[^/"'<>\s]+\/video\/\d+/i)?.[0];
    return path ? validUrl(path) : null;
  };

  const urlInState = (root) => {
    const queue = [];
    const visited = new WeakSet();
    let element = root;
    for (let depth = 0; element && depth < 18; depth += 1, element = element.parentElement) {
      for (const key of Object.getOwnPropertyNames(element)) {
        if (/^__(?:react|next|vue)/i.test(key)) queue.push(element[key]);
      }
    }
    let inspected = 0;
    while (queue.length && inspected < 6000) {
      const value = queue.shift();
      inspected += 1;
      if (typeof value === "string") {
        const match = value.replaceAll("\\/", "/").match(/\/@[^/"'<>\s]+\/video\/\d+/i)?.[0];
        if (match) return validUrl(match);
        continue;
      }
      if (!value || typeof value !== "object" || visited.has(value)) continue;
      visited.add(value);
      const id = String(value.id || value.itemId || value.aweme_id || "");
      const author = value.author?.uniqueId || value.author?.unique_id || value.authorName || value.uniqueId;
      if (/^\d{15,}$/.test(id) && author) {
        const url = validUrl(`https://www.tiktok.com/@${author}/video/${id}`);
        if (url) return url;
      }
      for (const child of Object.values(value)) {
        if ((child && typeof child === "object") || typeof child === "string") queue.push(child);
      }
    }
    return null;
  };

  const urlInPageJson = (video) => {
    let context = "";
    let container = video;
    for (let depth = 0; container && depth < 10; depth += 1, container = container.parentElement) {
      const text = String(container.innerText || "").trim();
      if (text.length >= 12 && text.length <= 4000) context = text.toLowerCase();
    }
    const queue = [];
    const candidates = [];
    for (const script of document.querySelectorAll('script[type="application/json"],script[id*="DATA"],script[id*="STATE"]')) {
      const text = script.textContent || "";
      if (!text.includes("video") || text.length > 15_000_000) continue;
      try { queue.push(JSON.parse(text)); } catch {}
    }
    const visited = new WeakSet();
    let inspected = 0;
    while (queue.length && inspected < 25000) {
      const value = queue.shift();
      inspected += 1;
      if (!value || typeof value !== "object" || visited.has(value)) continue;
      visited.add(value);
      const id = String(value.id || value.itemId || value.aweme_id || "");
      const author = value.author?.uniqueId || value.author?.unique_id || value.authorName || value.uniqueId;
      if (/^\d{15,}$/.test(id) && author) {
        const url = validUrl(`https://www.tiktok.com/@${author}/video/${id}`);
        const description = String(value.desc || value.description || value.title || "").trim().toLowerCase();
        const authorText = String(author).trim().toLowerCase();
        const score = (description && context.includes(description.slice(0, Math.min(48, description.length))) ? 4 : 0)
          + (authorText && context.includes(authorText) ? 2 : 0);
        if (url) candidates.push({ url, score });
      }
      for (const child of Object.values(value)) {
        if (child && typeof child === "object") queue.push(child);
      }
    }
    candidates.sort((left, right) => right.score - left.score);
    if (candidates[0]?.score > 0) return candidates[0].url;
    return candidates.length === 1 ? candidates[0].url : null;
  };

  const permalinkFor = (video) => {
    if (validUrl(location.href)) return location.href;
    let container = video;
    for (let depth = 0; container && depth < 28; depth += 1, container = container.parentElement) {
      const url = urlInside(container);
      if (url) return url;
    }
    return urlInState(video) || urlInPageJson(video);
  };

  document.addEventListener("click", (event) => {
    const button = event.target?.closest?.(".apocalipse-media-download:not(.apocalipse-media-record)");
    if (!button) return;
    const video = videoForButton(button);
    const url = permalinkFor(video);
    if (!video || !url) return;
    event.preventDefault();
    event.stopImmediatePropagation();
    const original = button.textContent;
    button.textContent = "…";
    chrome.runtime.sendMessage({
      type: "APOCALIPSE_DOWNLOAD",
      item: {
        url,
        duration: Number.isFinite(video.duration) && video.duration > 0 ? video.duration : null,
        requestUrls: [],
        userAgent: navigator.userAgent,
        kind: "video",
        title: document.title,
        thumbnail: video.poster || "",
      },
    }, (result) => {
      const failed = chrome.runtime.lastError || !result?.ok;
      button.textContent = failed ? "⚠" : "✓";
      button.title = failed
        ? result?.error || chrome.runtime.lastError?.message || "Apocalipse unavailable"
        : "Enviado ao Apocalipse";
      setTimeout(() => { button.textContent = original; }, 1800);
    });
  }, true);
})();
