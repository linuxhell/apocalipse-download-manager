(() => {
  const replaying = new WeakSet();
  const validUrl = (value) => {
    try {
      const url = new URL(String(value || "").replaceAll("\\/", "/"), location.href);
      const match = url.pathname.match(/^\/@([^/]+)\/video\/(\d+)/i);
      if (!/(^|\.)tiktok\.com$/i.test(url.hostname) || !match) return null;
      let author = null;
      try {
        const decoded = decodeURIComponent(match[1]);
        if (/^[A-Za-z0-9._-]+$/.test(decoded)) author = decoded;
      } catch {}
      if (!author) return null;
      return `https://www.tiktok.com/@${author}/video/${match[2]}`;
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
      const author = value.author?.uniqueId || value.author?.unique_id || value.uniqueId;
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
      const author = value.author?.uniqueId || value.author?.unique_id || value.uniqueId;
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

  const copiedPermalinkFor = async (video) => {
    let container = video;
    let share = null;
    for (let depth = 0; container && depth < 16 && !share; depth += 1, container = container.parentElement) {
      share = [...(container.querySelectorAll?.('button,[role="button"]') || [])].find((item) =>
        /(?:compartilhar|share|分享)/i.test(`${item.getAttribute("aria-label") || ""} ${item.title || ""} ${item.textContent || ""}`));
    }
    if (!share) return null;
    share.click();
    let copyItem = null;
    for (let attempt = 0; attempt < 15 && !copyItem; attempt += 1) {
      await new Promise((resolve) => setTimeout(resolve, 100));
      copyItem = [...document.querySelectorAll('[role="menuitem"],button,[role="button"]')].find((item) =>
        /(?:copiar link|copy link|复制链接|複製連結)/i.test(item.textContent || ""));
    }
    if (!copyItem) return null;
    copyItem.click();
    await new Promise((resolve) => setTimeout(resolve, 80));
    try { return validUrl(await navigator.clipboard.readText()); } catch { return null; }
  };

  document.addEventListener("click", async (event) => {
    const button = event.target?.closest?.(".apocalipse-media-download:not(.apocalipse-media-record)");
    if (!button || replaying.has(button)) {
      if (button) replaying.delete(button);
      return;
    }
    const video = videoForButton(button);
    if (!video) return;
    event.preventDefault();
    event.stopImmediatePropagation();
    let enteredFullscreen = false;
    let url = permalinkFor(video) || await copiedPermalinkFor(video);
    if (!url && video.requestFullscreen) {
      try {
        await video.requestFullscreen();
        enteredFullscreen = true;
        await new Promise((resolve) => setTimeout(resolve, 500));
        url = permalinkFor(video);
      } catch {}
    }
    if (!url) {
      replaying.add(button);
      button.click();
      return;
    }
    const original = button.textContent;
    button.textContent = "…";
    const captured = await chrome.runtime.sendMessage({ type: "APOCALIPSE_RECENT_TAB_MEDIA" }).catch(() => null);
    const browserMedia = captured?.media?.find((item) =>
      /^https?:/i.test(item.url || "") && /^video\//i.test(item.contentType || ""))
      || captured?.media?.find((item) =>
        /^https?:/i.test(item.url || "") && !/^audio\//i.test(item.contentType || ""))
      || null;
    const selectedUrl = browserMedia?.url || url;
    chrome.runtime.sendMessage({
      type: "APOCALIPSE_CAPTURE_TRACE",
      eventName: "tiktok_browser_media_selection",
      mode: "download",
      traceId: crypto.randomUUID(),
      pageUrl: location.href,
      at: Date.now(),
      detail: {
        browserCapturedMedia: Boolean(browserMedia),
        browserCandidates: captured?.media?.length || 0,
        browserCandidateAgeMs: browserMedia?.ageMs ?? -1,
        browserCandidateType: browserMedia?.contentType || "none",
        browserCandidateBytes: browserMedia?.contentLength || 0,
        browserCandidateHost: (() => { try { return new URL(browserMedia?.url || "").hostname; } catch { return "none"; } })(),
      },
    }).catch(() => {});
    chrome.runtime.sendMessage({
      type: "APOCALIPSE_DOWNLOAD",
      item: {
        url: selectedUrl,
        duration: Number.isFinite(video.duration) && video.duration > 0 ? video.duration : null,
        requestUrls: captured?.media?.map((item) => item.url).filter(Boolean) || [],
        userAgent: navigator.userAgent,
        kind: "video",
        title: document.title,
        thumbnail: video.poster || "",
      },
    }, (result) => {
      const failed = chrome.runtime.lastError || !result?.ok;
      button.textContent = failed ? "⚠" : "✓";
      if (enteredFullscreen && document.fullscreenElement) document.exitFullscreen?.().catch(() => {});
      button.title = failed
        ? result?.error || chrome.runtime.lastError?.message || "Apocalipse unavailable"
        : "Enviado ao Apocalipse";
      setTimeout(() => { button.textContent = original; }, 1800);
    });
  }, true);
})();
