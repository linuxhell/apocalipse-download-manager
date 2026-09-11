(() => {
  const mediaUrl = (value) => {
    try {
      const url = new URL(value, location.href);
      if (!/(^|\.)facebook\.com$/i.test(url.hostname)) return null;
      return /\/(?:reel|reels|videos|posts)\/[A-Za-z0-9._-]+/i.test(url.pathname)
        || /\/(?:watch|permalink|story)\.php/i.test(url.pathname)
        || /[?&](?:v|fbid|story_fbid)=/i.test(url.href)
        ? url.href : null;
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
    const page = mediaUrl(location.href);
    if (page) return page;
    const selectors = [
      'a[href*="/reel/"]', 'a[href*="/reels/"]', 'a[href*="/videos/"]',
      'a[href*="/posts/"]', 'a[href*="watch.php"]', 'a[href*="permalink.php"]',
      'a[href*="story.php"]',
    ].join(",");
    let container = video;
    for (let depth = 0; container && depth < 20; depth += 1, container = container.parentElement) {
      for (const anchor of container.querySelectorAll?.(selectors) || []) {
        const url = mediaUrl(anchor.href);
        if (url) return url;
      }
      const path = String(container.innerHTML || "").replaceAll("\\/", "/")
        .match(/\/(?:reel|reels|videos|posts)\/[A-Za-z0-9._-]+/i)?.[0];
      if (path) return mediaUrl(path);
    }
    return null;
  };

  const thumbnailFor = (video) => {
    if (video?.poster) return video.poster;
    const target = video?.getBoundingClientRect?.();
    let container = video?.parentElement;
    for (let depth = 0; target && container && depth < 10; depth += 1, container = container.parentElement) {
      if (container.querySelectorAll?.("video").length > 1) break;
      for (const image of container.querySelectorAll?.("img") || []) {
        const rect = image.getBoundingClientRect();
        const width = Math.max(0, Math.min(target.right, rect.right) - Math.max(target.left, rect.left));
        const height = Math.max(0, Math.min(target.bottom, rect.bottom) - Math.max(target.top, rect.top));
        const overlap = width * height / Math.min(Math.max(1, target.width * target.height), Math.max(1, rect.width * rect.height));
        const source = image.currentSrc || image.src || image.getAttribute("data-src") || "";
        if (overlap >= 0.45 && /^https?:/i.test(source)) return source;
      }
    }
    return document.querySelectorAll("video").length <= 1
      ? document.querySelector('meta[property="og:image"]')?.content || ""
      : "";
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
    const id = new URL(url).pathname.match(/\/(?:reel|reels|videos|posts)\/([^/?#]+)/i)?.[1] || "video";
    chrome.runtime.sendMessage({
      type: "APOCALIPSE_DOWNLOAD",
      item: {
        url,
        duration: Number.isFinite(video.duration) && video.duration > 0 ? video.duration : null,
        requestUrls: [],
        userAgent: navigator.userAgent,
        kind: "video",
        title: `Facebook-${id}`,
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
