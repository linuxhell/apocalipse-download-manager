(() => {
  const mediaUrl = (value) => {
    try {
      const url = new URL(value, location.href);
      return /(^|\.)instagram\.com$/i.test(url.hostname)
        && /\/(?:reel|reels|p)\/[^/?#]+/i.test(url.pathname)
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

  const permalinkFor = (video) => {
    if (mediaUrl(location.href)) return location.href;
    const selectors = 'a[href*="/reel/"],a[href*="/reels/"],a[href*="/p/"]';
    let container = video;
    for (let depth = 0; container && depth < 24; depth += 1, container = container.parentElement) {
      for (const anchor of container.querySelectorAll?.(selectors) || []) {
        const url = mediaUrl(anchor.href);
        if (url) return url;
      }
    }
    const rect = video?.getBoundingClientRect?.();
    if (!rect) return null;
    return [...document.querySelectorAll(selectors)]
      .map((anchor) => ({ url: mediaUrl(anchor.href), rect: anchor.getBoundingClientRect() }))
      .filter((item) => item.url && item.rect.width > 0 && item.rect.height > 0
        && item.rect.bottom >= rect.top && item.rect.top <= rect.bottom)
      .sort((left, right) => {
        const center = (rect.top + rect.bottom) / 2;
        return Math.abs((left.rect.top + left.rect.bottom) / 2 - center)
          - Math.abs((right.rect.top + right.rect.bottom) / 2 - center);
      })[0]?.url || null;
  };

  const thumbnailFor = (video) => {
    if (video?.poster) return video.poster;
    const rect = video?.getBoundingClientRect?.();
    if (!rect) return document.querySelector('meta[property="og:image"]')?.content || "";
    return [...document.images]
      .map((image) => ({ url: image.currentSrc || image.src, rect: image.getBoundingClientRect() }))
      .filter((item) => item.url && item.rect.width >= 120 && item.rect.height >= 120
        && item.rect.bottom >= rect.top && item.rect.top <= rect.bottom)
      .sort((left, right) => right.rect.width * right.rect.height - left.rect.width * left.rect.height)[0]?.url
      || document.querySelector('meta[property="og:image"]')?.content
      || "";
  };

  const playableUrlFor = (video) => {
    for (const value of [video?.currentSrc, video?.src, ...[...(video?.querySelectorAll?.("source") || [])].map((source) => source.src)]) {
      try {
        const url = new URL(value, location.href);
        if (/^https?:$/i.test(url.protocol)) return url.href;
      } catch {}
    }
    return null;
  };

  document.addEventListener("click", (event) => {
    const button = event.target?.closest?.(".apocalipse-media-download:not(.apocalipse-media-record)");
    if (!button) return;
    const video = videoForButton(button);
    const url = playableUrlFor(video) || permalinkFor(video);
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
        thumbnail: thumbnailFor(video),
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
