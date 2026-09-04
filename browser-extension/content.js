(() => {
  let shortcutKeys = { force: "Shift", bypass: "Alt" };
  chrome.storage.local.get({ forceShortcut: "Shift", bypassShortcut: "Alt" }, (value) => {
    shortcutKeys = { force: value.forceShortcut, bypass: value.bypassShortcut };
  });
  chrome.storage.onChanged.addListener((changes, area) => {
    if (area !== "local") return;
    if (changes.forceShortcut) shortcutKeys.force = changes.forceShortcut.newValue;
    if (changes.bypassShortcut) shortcutKeys.bypass = changes.bypassShortcut.newValue;
  });
  const modifierPressed = (event, key) => ({ Alt: event.altKey, Shift: event.shiftKey, Control: event.ctrlKey }[key] || false);
  const sendShortcutState = (event) => chrome.runtime.sendMessage({
    type: "APOCALIPSE_SHORTCUT_STATE",
    bypassPressed: modifierPressed(event, shortcutKeys.bypass),
    forcePressed: modifierPressed(event, shortcutKeys.force),
  }).catch(() => {});
  document.addEventListener("keydown", sendShortcutState, true);
  document.addEventListener("keyup", sendShortcutState, true);
  document.addEventListener("pointerdown", (event) => {
    if (!modifierPressed(event, shortcutKeys.bypass)) return;
    chrome.runtime.sendMessage({ type: "APOCALIPSE_BYPASS_NEXT", ttlMs: 15000 }).catch(() => {});
  }, true);
  window.addEventListener("blur", () => chrome.runtime.sendMessage({
    type: "APOCALIPSE_SHORTCUT_STATE",
    bypassPressed: false,
    forcePressed: false,
  }).catch(() => {}));
  document.addEventListener("submit", (event) => {
    const form = event.target;
    if (!(form instanceof HTMLFormElement) || String(form.method || "get").toLowerCase() !== "post") return;
    const data = new FormData(form, event.submitter || undefined);
    const body = new URLSearchParams();
    for (const [name, value] of data) if (typeof value === "string") body.append(name, value);
    chrome.runtime.sendMessage({
      type: "APOCALIPSE_FORM_SUBMIT",
      request: {
        url: absolute(form.action || location.href),
        pageUrl: location.href,
        method: "POST",
        body: body.toString(),
        contentType: "application/x-www-form-urlencoded",
        capturedAt: Date.now(),
      },
    }).catch(() => {});
  }, true);

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
      const facebookUrl = facebookUrlFor(element);
      if (facebookUrl) add(facebookUrl, "video", element);
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
  const downloadableLink = (anchor) => {
    const url = absolute(anchor?.href);
    if (!url || !/^https?:/i.test(url)) return null;
    // A same-origin URL that looks like a file can still be a generator/landing
    // page (Filespayouts is one example). Let the site's click handler run so
    // downloads.onDeterminingFilename receives the final CDN URL and headers.
    // The configured force shortcut intentionally bypasses this safeguard.
    try {
      if (new URL(url).origin === location.origin) return null;
    } catch {
      return null;
    }
    if (anchor.hasAttribute("download")) return url;
    return /\.(?:7z|apk|bin|bz2|cab|deb|dmg|exe|gz|img|iso|msi|msix|pkg|rar|rpm|tar|tbz2|tgz|txz|xz|zip)(?:$|[?#])/i.test(url) ? url : null;
  };
  const fileNameForUrl = (url) => {
    const value = new URL(url).pathname.split("/").pop() || "download";
    try { return decodeURIComponent(value); } catch { return value; }
  };
  document.addEventListener("click", (event) => {
    if (event.defaultPrevented || event.button !== 0 || event.metaKey) return;
    const bypass = modifierPressed(event, shortcutKeys.bypass);
    const force = modifierPressed(event, shortcutKeys.force);
    if (bypass) {
      chrome.runtime.sendMessage({ type: "APOCALIPSE_BYPASS_NEXT", ttlMs: 15000 }).catch(() => {});
      return;
    }
    if ((event.ctrlKey || event.shiftKey || event.altKey) && !force) return;
    const anchor = event.target.closest?.("a[href]");
    const anchorUrl = absolute(anchor?.href);
    if (!anchorUrl || /\/undefined(?:$|[?#])/i.test(anchorUrl)) return;
    let anchorHost = "";
    try { anchorHost = new URL(anchorUrl).hostname; } catch {}
    const onFilespayouts = /(^|\.)filespayouts\.com$/i.test(location.hostname);
    const entersFilespayouts = /(^|\.)filespayouts\.com$/i.test(anchorHost);
    if (!force && anchorUrl && entersFilespayouts && !onFilespayouts
      && !/\/undefined(?:$|[?#])/i.test(new URL(anchorUrl).pathname)) {
      // A Filespayouts URL ending in a file extension is still a generator
      // page. Navigation (the same behavior as Open link in new tab) lets it
      // resolve the temporary CDN address before interception.
      event.preventDefault();
      event.stopImmediatePropagation();
      window.open(anchorUrl, "_blank", "noopener");
      return;
    }
    const url = force ? anchorUrl : downloadableLink(anchor);
    if (!url) return;
    event.preventDefault();
    event.stopImmediatePropagation();
    chrome.runtime.sendMessage({
      type: "APOCALIPSE_DOWNLOAD",
      item: {
        url,
        requestUrls: [url],
        userAgent: navigator.userAgent,
        kind: "file",
        title: fileNameForUrl(url),
      },
    }, (result) => {
      if (result?.target !== "apocalipse" || chrome.runtime.lastError) location.assign(url);
    });
  }, true);
  const hlsForPage = () => {
    const urls = [...new Set(performance.getEntriesByType("resource").map((entry) => entry.name)
      .filter((url) => /\.m3u8(?:$|[?#])/i.test(url)))];
    const masters = urls.filter((url) => /(?:\/master\/|master\.m3u8)/i.test(url));
    return { candidates: urls, fallback: urls.at(-1) || masters.at(-1) || null };
  };
  const facebookUrlFor = (element) => {
    if (!/(^|\.)facebook\.com$/i.test(location.hostname)) return null;
    const container = element.closest?.('[role="dialog"],article,[data-pagelet]') || element.parentElement;
    const anchors = container?.querySelectorAll?.('a[href]') || [];
    for (const anchor of anchors) {
      const url = absolute(anchor.href);
      if (!url) continue;
      try {
        const parsed = new URL(url);
        if (/(?:^|\/)(?:reel|reels|watch|videos|share)(?:\/|$)/i.test(parsed.pathname)) return url;
      } catch {}
    }
    if (/(?:^|\/)(?:reel|reels|watch|videos|share)(?:\/|$)/i.test(location.pathname)) return location.href;
    return null;
  };
  const downloadUrlFor = (element) => {
    if (element.tagName === "VIDEO" && /^(?:www\.)?youtube\.com$/.test(location.hostname) && location.pathname === "/watch") return location.href;
    if (element.tagName === "VIDEO") {
      const facebookUrl = facebookUrlFor(element);
      if (facebookUrl) return facebookUrl;
    }
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
        chrome.runtime.sendMessage({ type: "APOCALIPSE_DOWNLOAD", item: { url: currentUrl, duration: resolved?.duration || null, requestUrls: resolved?.requestUrls || [], userAgent: navigator.userAgent, kind: element.tagName.toLowerCase(), title: document.title, thumbnail: thumbnailFor(element, "video") } });
      });
      let positionTimer = null;
      const position = () => {
        if (!element.isConnected) {
          button.remove();
          if (positionTimer) clearInterval(positionTimer);
          return;
        }
        const anchor = isYouTubeVideo
          ? document.querySelector("#movie_player") || element.closest("ytd-player") || element
          : element;
        const rect = anchor.getBoundingClientRect();
        button.style.left = `${Math.max(6, rect.left + scrollX + 8)}px`;
        const top = isYouTubeVideo
          ? rect.top + scrollY - button.offsetHeight - 8
          : rect.top + scrollY + 8;
        button.style.top = `${Math.max(6, top)}px`;
        button.hidden = rect.width < 100 || rect.height < 55;
      };
      document.documentElement.append(button);
      position();
      addEventListener("scroll", position, { passive: true });
      addEventListener("resize", position, { passive: true });
      if (isYouTubeVideo) positionTimer = setInterval(position, 1000);
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
    if (message?.type === "APOCALIPSE_UPLOAD_BLOB" && /^blob:/i.test(message.url || "")) {
      (async () => {
        const response = await fetch(message.url);
        const blob = await response.blob();
        const begin = await chrome.runtime.sendMessage({
          type: "APOCALIPSE_BLOB_BEGIN",
          request: { fileName: message.fileName, total: blob.size, source: location.href },
        });
        if (!begin?.uploadId) throw new Error(begin?.error || "blob_begin_failed");
        reply({ started: true });
        const chunkSize = 64 * 1024;
        for (let offset = 0; offset < blob.size; offset += chunkSize) {
          const bytes = new Uint8Array(await blob.slice(offset, offset + chunkSize).arrayBuffer());
          let data = "";
          for (const byte of bytes) data += byte.toString(16).padStart(2, "0");
          const result = await chrome.runtime.sendMessage({
            type: "APOCALIPSE_BLOB_CHUNK",
            request: { uploadId: begin.uploadId, data },
          });
          if (result?.error) throw new Error(result.error);
        }
        const result = await chrome.runtime.sendMessage({
          type: "APOCALIPSE_BLOB_END",
          request: { uploadId: begin.uploadId },
        });
        if (result?.error) throw new Error(result.error);
      })().catch((error) => {
        console.error("Apocalipse Telegram adapter", error);
        reply({ started: false, error: String(error) });
      });
      return true;
    }
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
