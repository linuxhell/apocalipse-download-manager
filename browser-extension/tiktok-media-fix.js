(() => {
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

  const reactPayloadsFor = (video) => {
    const payloads = [];
    const fibers = [];
    let element = video;
    for (let depth = 0; element && depth < 12; depth += 1, element = element.parentElement) {
      for (const key of Object.getOwnPropertyNames(element)) {
        if (/^__reactFiber/i.test(key) && element[key]) fibers.push(element[key]);
        if (/^__reactProps/i.test(key) && element[key]) payloads.push(element[key]);
      }
    }
    const seen = new WeakSet();
    for (const initial of fibers) {
      let fiber = initial;
      for (let depth = 0; fiber && depth < 120 && !seen.has(fiber); depth += 1) {
        seen.add(fiber);
        for (const value of [fiber.memoizedProps, fiber.pendingProps, fiber.memoizedState]) {
          if (value && typeof value === "object") payloads.push(value);
        }
        fiber = fiber.return;
      }
    }
    return payloads;
  };

  const permalinkInObject = (root) => {
    const queue = [root];
    const visited = new WeakSet();
    let inspected = 0;
    while (queue.length && inspected < 12_000) {
      const value = queue.shift();
      inspected += 1;
      if (typeof value === "string") {
        const path = value.replaceAll("\\/", "/").match(/\/@[^/"'<>&\s]+\/video\/\d+/i)?.[0];
        const url = path && validUrl(path);
        if (url) return url;
        continue;
      }
      if (!value || typeof value !== "object" || visited.has(value)) continue;
      visited.add(value);
      const id = String(value.id || value.itemId || value.aweme_id || value.awemeId || value.videoId || "");
      const author = value.author?.uniqueId || value.author?.unique_id || value.author?.unique_id_str
        || value.authorInfo?.uniqueId || value.authorInfo?.unique_id || value.uniqueId || value.authorName;
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
    for (const payload of reactPayloadsFor(video)) {
      const url = permalinkInObject(payload);
      if (url) return url;
    }
    return urlInState(video) || urlInPageJson(video);
  };

  const copiedPermalinkFor = async (video) => {
    let container = video;
    let share = null;
    for (let depth = 0; container && depth < 18 && !share; depth += 1, container = container.parentElement) {
      share = [...(container.querySelectorAll?.('button,[role="button"]') || [])].find((item) =>
        /(?:compartilhar|share|分享)/i.test(`${item.getAttribute("aria-label") || ""} ${item.title || ""} ${item.textContent || ""}`));
    }
    if (!share) return null;
    const shield = document.createElement("style");
    shield.textContent = `
      html[data-apocalipse-tiktok-resolving] [role="dialog"],
      html[data-apocalipse-tiktok-resolving] [data-e2e*="share"],
      html[data-apocalipse-tiktok-resolving] [class*="ShareModal"],
      html[data-apocalipse-tiktok-resolving] [class*="share-modal"] {
        opacity: 0 !important;
        visibility: hidden !important;
        transition: none !important;
      }
    `;
    document.documentElement.dataset.apocalipseTiktokResolving = "1";
    document.documentElement.append(shield);
    try {
      share.click();
      let copyItem = null;
      for (let attempt = 0; attempt < 20 && !copyItem; attempt += 1) {
        await new Promise((resolve) => setTimeout(resolve, 75));
        copyItem = [...document.querySelectorAll('[role="menuitem"],button,[role="button"]')].find((item) =>
          /(?:copiar link|copy link|复制链接|複製連結)/i.test(`${item.getAttribute("aria-label") || ""} ${item.textContent || ""}`));
      }
      if (!copyItem) return null;
      copyItem.click();
      await new Promise((resolve) => setTimeout(resolve, 100));
      try { return validUrl(await navigator.clipboard.readText()); } catch { return null; }
    } finally {
      document.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape", code: "Escape", bubbles: true }));
      delete document.documentElement.dataset.apocalipseTiktokResolving;
      shield.remove();
    }
  };

  const directVideoUrl = (value) => {
    try {
      const url = new URL(String(value || "").replaceAll("\\/", "/"), location.href);
      const host = url.hostname.toLowerCase();
      if (!/(?:^|\.)(?:tiktok\.com|tiktokcdn(?:-us)?\.com|tiktokv\.com|byteoversea\.com|ibytedtos\.com|muscdn\.com)$/i.test(host)) return null;
      const normalized = url.href.toLowerCase();
      if (/mime_type=(?:audio|image)/i.test(normalized)) return null;
      if (!/(?:\/video\/tos\/|\/aweme\/v1\/(?:play|download)\/|mime_type=video|\.mp4(?:$|[?#]))/i.test(normalized)) return null;
      return url.href;
    } catch { return null; }
  };

  const mediaUrlInObject = (root) => {
    const queue = [{ value: root, path: "" }];
    const visited = new WeakSet();
    const candidates = [];
    let inspected = 0;
    while (queue.length && inspected < 5000) {
      const { value, path } = queue.shift();
      inspected += 1;
      if (typeof value === "string") {
        const url = directVideoUrl(value);
        if (url) {
          const score = (/playaddr|downloadaddr|urllist|bitrate/i.test(path) ? 8 : 0)
            + (/mime_type=video/i.test(url) ? 4 : 0)
            + (/\/video\/tos\//i.test(url) ? 2 : 0);
          candidates.push({ url, score });
        }
        continue;
      }
      if (!value || typeof value !== "object" || visited.has(value)) continue;
      visited.add(value);
      for (const [key, child] of Object.entries(value)) {
        if ((child && typeof child === "object") || typeof child === "string") {
          queue.push({ value: child, path: `${path}.${key}` });
        }
      }
    }
    candidates.sort((left, right) => right.score - left.score);
    return candidates[0]?.url || null;
  };

  const mediaUrlInTree = (roots, targetId) => {
    const queue = [...roots];
    const visited = new WeakSet();
    let inspected = 0;
    while (queue.length && inspected < 30000) {
      const value = queue.shift();
      inspected += 1;
      if (!value || typeof value !== "object" || visited.has(value)) continue;
      visited.add(value);
      const id = String(value.id || value.itemId || value.aweme_id || value.awemeId || "");
      if (id === targetId) {
        const url = mediaUrlInObject(value.video || value.videoInfo || value);
        if (url) return url;
      }
      for (const child of Object.values(value)) {
        if (child && typeof child === "object") queue.push(child);
      }
    }
    return null;
  };

  const directMediaFor = (video, permalink) => {
    const targetId = permalink.match(/\/video\/(\d+)/i)?.[1];
    if (!targetId) return null;
    const roots = [];
    let element = video;
    for (let depth = 0; element && depth < 20; depth += 1, element = element.parentElement) {
      for (const key of Object.getOwnPropertyNames(element)) {
        if (/^__(?:react|next|vue)/i.test(key)) roots.push(element[key]);
      }
    }
    const fromState = mediaUrlInTree(roots, targetId);
    if (fromState) return fromState;
    const pageRoots = [];
    for (const script of document.querySelectorAll('script[type="application/json"],script[id*="DATA"],script[id*="STATE"]')) {
      const text = script.textContent || "";
      if (!text.includes(targetId) || text.length > 15_000_000) continue;
      try { pageRoots.push(JSON.parse(text)); } catch {}
    }
    return mediaUrlInTree(pageRoots, targetId);
  };

  const directMediaForPlayer = (video) => {
    for (const payload of reactPayloadsFor(video)) {
      const url = mediaUrlInObject(payload);
      if (url) return url;
    }
    let element = video;
    for (let depth = 0; element && depth < 8; depth += 1, element = element.parentElement) {
      const keys = Object.getOwnPropertyNames(element)
        .filter((key) => /^__(?:react|next|vue)/i.test(key))
        .sort((left, right) => Number(/props/i.test(right)) - Number(/props/i.test(left)));
      for (const key of keys) {
        const url = mediaUrlInObject(element[key]);
        if (url) return url;
      }
    }
    return null;
  };

  document.addEventListener("click", async (event) => {
    const button = event.target?.closest?.(".apocalipse-media-download:not(.apocalipse-media-record)");
    if (!button) return;
    const video = videoForButton(button);
    if (!video) return;
    event.preventDefault();
    event.stopImmediatePropagation();
    const original = button.textContent;
    const url = permalinkFor(video) || await copiedPermalinkFor(video);
    button.textContent = "…";
    const stateMediaUrl = url ? directMediaFor(video, url) : directMediaForPlayer(video);
    const captured = await chrome.runtime.sendMessage({ type: "APOCALIPSE_RECENT_TAB_MEDIA" }).catch(() => null);
    const capturedMedia = Array.isArray(captured?.media) ? captured.media : [];
    const freshVideoCandidates = capturedMedia.filter((item) =>
      /^https?:/i.test(item.url || "")
      && /^video\//i.test(item.contentType || "")
      && Number.isFinite(item.ageMs)
      && item.ageMs >= 0
      && item.ageMs <= 8_000);

    const liveHttpUrl = directVideoUrl(video.currentSrc || video.src);
    const soleFreshCandidate = freshVideoCandidates.length === 1 ? freshVideoCandidates[0]?.url : null;
    const selectedUrl = stateMediaUrl || liveHttpUrl || soleFreshCandidate;
    const selectedId = url?.match(/\/video\/(\d+)/i)?.[1] || "none";
    const newestCaptured = capturedMedia[0] || null;
    if (!url) {
      chrome.runtime.sendMessage({
        type: "APOCALIPSE_CAPTURE_TRACE",
        eventName: "tiktok_video_identity_unresolved",
        mode: "download",
        traceId: crypto.randomUUID(),
        pageUrl: location.href,
        at: Date.now(),
        detail: {
          fallbackBlocked: !selectedUrl,
          reason: selectedUrl ? "player_media_without_permalink" : "no_verified_player_media",
        },
      }).catch(() => {});
    }
    chrome.runtime.sendMessage({
      type: "APOCALIPSE_CAPTURE_TRACE",
      eventName: "tiktok_browser_media_selection",
      mode: "download",
      traceId: crypto.randomUUID(),
      pageUrl: location.href,
      at: Date.now(),
      detail: {
        selection: stateMediaUrl ? "matched_state_media" : liveHttpUrl ? "player_http_media" : soleFreshCandidate ? "sole_fresh_media" : "media_picker_required",
        selectedVideoId: selectedId,
        browserCapturedMedia: Boolean(soleFreshCandidate && selectedUrl === soleFreshCandidate),
        browserCandidates: capturedMedia.length,
        browserFreshVideoCandidates: freshVideoCandidates.length,
        browserCandidatesRejected: capturedMedia.length - (soleFreshCandidate ? 1 : 0),
        browserRejectionReason: capturedMedia.length ? "media_picker_required" : "uncorrelated_feed_media",
        browserCandidateAgeMs: newestCaptured?.ageMs ?? -1,
        browserCandidateType: newestCaptured?.contentType || "none",
        browserCandidateBytes: newestCaptured?.contentLength || 0,
        browserCandidateHost: (() => { try { return new URL(newestCaptured?.url || "").hostname; } catch { return "none"; } })(),
      },
    }).catch(() => {});
    if (!selectedUrl) {
      const language = (await chrome.storage.local.get({ language: "en" })).language;
      const notice = language === "pt_BR"
        ? "Há recursos de vídeo disponíveis na extensão. Escolha o arquivo."
        : language === "zh_CN"
          ? "扩展中有可用的视频资源。请选择文件。"
          : "Video resources are available in the extension. Choose a file.";
      button.textContent = "!";
      button.title = notice;
      const previousNotice = document.querySelector("#apocalipse-media-picker-notice");
      previousNotice?.remove();
      const toast = document.createElement("div");
      toast.id = "apocalipse-media-picker-notice";
      toast.textContent = notice;
      toast.style.cssText = "position:fixed;left:50%;bottom:32px;transform:translateX(-50%);z-index:2147483647;max-width:560px;padding:13px 18px;border:1px solid #31d9ee;border-radius:10px;background:#111a20f2;color:#f3fbff;font:600 14px system-ui;box-shadow:0 6px 24px #000a;text-align:center";
      document.documentElement.append(toast);
      setTimeout(() => toast.remove(), 5000);
      chrome.runtime.sendMessage({ type: "APOCALIPSE_OPEN_MEDIA_PICKER" }).catch(() => {});
      setTimeout(() => { button.textContent = original; }, 2500);
      return;
    }
    chrome.runtime.sendMessage({
      type: "APOCALIPSE_DOWNLOAD",
      item: {
        url: selectedUrl,
        duration: Number.isFinite(video.duration) && video.duration > 0 ? video.duration : null,
        requestUrls: [selectedUrl, ...(url ? [url] : [])],
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
