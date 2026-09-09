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
    const bypass = modifierPressed(event, shortcutKeys.bypass);
    const force = modifierPressed(event, shortcutKeys.force);
    if (bypass) {
      chrome.runtime.sendMessage({ type: "APOCALIPSE_BYPASS_NEXT", ttlMs: 4000 }).catch(() => {});
    } else if (force) {
      chrome.runtime.sendMessage({ type: "APOCALIPSE_FORCE_NEXT", ttlMs: 20000 }).catch(() => {});
    }
  }, true);
  window.addEventListener("blur", () => chrome.runtime.sendMessage({
    type: "APOCALIPSE_SHORTCUT_STATE",
    bypassPressed: false,
    forcePressed: false,
  }).catch(() => {}));
  const absolute = (value) => {
    try { return new URL(value, location.href).href; } catch { return null; }
  };
  // Intercept ChatGPT Library links before Chrome creates its own download dialog.
  // Use composedPath + nearby link discovery because ChatGPT may wrap the visible
  // download control in buttons/spans instead of making the clicked node the anchor.
  const chatgptLibraryLinkForEvent = (event) => {
    const candidates = [];
    for (const node of event.composedPath?.() || []) {
      if (node?.href) candidates.push(node);
      const closest = node?.closest?.('a[href*="/backend-api/estuary/content"]');
      if (closest) candidates.push(closest);
      const nested = node?.querySelector?.('a[href*="/backend-api/estuary/content"]');
      if (nested) candidates.push(nested);
    }
    const target = event.target;
    for (let parent = target; parent && parent !== document.documentElement; parent = parent.parentElement) {
      const nested = parent.querySelector?.('a[href*="/backend-api/estuary/content"]');
      if (nested) candidates.push(nested);
      if (parent.matches?.('a[href*="/backend-api/estuary/content"]')) candidates.push(parent);
      if (candidates.length) break;
    }
    for (const candidate of candidates) {
      try {
        const parsed = new URL(candidate.href, location.href);
        if (parsed.hostname.toLowerCase() === "chatgpt.com" && parsed.pathname === "/backend-api/estuary/content") {
          return { url: parsed.href, fileName: candidate.getAttribute?.("download") || "" };
        }
      } catch {}
    }
    return null;
  };
  const interceptChatgptLibrary = (event) => {
    if (event.defaultPrevented || (typeof event.button === "number" && event.button !== 0)) return;
    const found = chatgptLibraryLinkForEvent(event);
    if (!found) return;
    event.preventDefault();
    event.stopImmediatePropagation();
    chrome.runtime.sendMessage({
      type: "APOCALIPSE_CHATGPT_LIBRARY_DIRECT",
      url: found.url,
      pageUrl: location.href,
      fileName: found.fileName,
    }).catch(() => {});
  };
  document.addEventListener("pointerdown", interceptChatgptLibrary, true);
  document.addEventListener("click", interceptChatgptLibrary, true);

  const looksLikeChatgptLibraryDownloadControl = (event) => {
    if (!/(^|\.)chatgpt\.com$/i.test(location.hostname)) return false;
    const path = location.pathname.toLowerCase();
    if (!path.includes("library")) return false;
    for (const node of event.composedPath?.() || []) {
      const label = `${node?.getAttribute?.("aria-label") || ""} ${node?.title || ""} ${node?.textContent || ""}`.trim();
      if (/(?:download|baixar|下载)/i.test(label)) return true;
      const href = node?.href || node?.closest?.('a[href*="/backend-api/estuary/content"]')?.href;
      if (href && /\/backend-api\/estuary\/content/i.test(href)) return true;
    }
    return false;
  };
  document.addEventListener("pointerdown", (event) => {
    if (!looksLikeChatgptLibraryDownloadControl(event)) return;
    chrome.runtime.sendMessage({ type: "APOCALIPSE_CHATGPT_LIBRARY_ARM_DENY" }).catch(() => {});
  }, true);

  const recentNetworkMediaUrl = () => {
    try {
      const entries = performance.getEntriesByType("resource");
      const facebookPage = /(^|\.)facebook\.com$/i.test(location.hostname);
      for (let i = entries.length - 1; i >= 0; i -= 1) {
        const name = String(entries[i]?.name || "");
        if (!/^https?:/i.test(name)) continue;
        if (/\.(?:avif|bmp|gif|ico|jpe?g|png|svg|webp)(?:[?#]|$)/i.test(name)) continue;
        // Host names and words such as "video" are not proof that a response is
        // media. DVIDS, for example, also exposes analytics and JSON APIs on
        // similarly named hosts. Passing one of those to the desktop incorrectly
        // selects NativeHttp/direct_http instead of the HLS pipeline.
        if (/\.(?:mp4|webm|m3u8|mpd)(?:[?#]|$)/i.test(name)) return name;
        // Facebook CDN paths frequently omit a file extension. initiatorType=video
        // is browser evidence that the response feeds the player, unlike a host
        // name or a loose "video" substring.
        if (facebookPage && entries[i]?.initiatorType === "video") {
          try {
            if (/(^|\.)fbcdn\.net$/i.test(new URL(name).hostname)) return name;
          } catch {}
        }
      }
    } catch {}
    return null;
  };

  const safeMediaFileName = (fallback = "video.mp4", blob = null) => {
    const title = (document.title || "video").replace(/[<>:\"/\\|?*]+/g, "_").trim().slice(0, 100) || "video";
    const type = String(blob?.type || "").toLowerCase();
    const ext = type.includes("webm") ? ".webm" : type.includes("ogg") ? ".ogv" : ".mp4";
    if (/\.[A-Za-z0-9]{2,5}$/.test(fallback)) return fallback;
    return `${title}${ext}`;
  };
  const uploadBlobUrl = async (url, fileName = null) => {
    if (!/^blob:/i.test(String(url || ""))) throw new Error("not_blob_url");
    const response = await fetch(url);
    if (!response.ok) throw new Error(`blob_http_${response.status}`);
    const blob = await response.blob();
    if (!blob.size) throw new Error("empty_blob_url");
    const name = safeMediaFileName(fileName || "video", blob);
    await uploadBlob(blob, name);
    return { ok: true, bytes: blob.size, fileName: name };
  };

  const uploadBlob = async (blob, fileName) => {
    const begin = await chrome.runtime.sendMessage({
      type: "APOCALIPSE_BLOB_BEGIN",
      request: { fileName, total: blob.size, source: location.href },
    });
    if (!begin?.uploadId) throw new Error(begin?.error || "blob_begin_failed");
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
  };
  const appendBlob = async (uploadId, blob) => {
    const chunkSize = 64 * 1024;
    for (let offset = 0; offset < blob.size; offset += chunkSize) {
      const bytes = new Uint8Array(await blob.slice(offset, offset + chunkSize).arrayBuffer());
      let data = "";
      for (const byte of bytes) data += byte.toString(16).padStart(2, "0");
      const result = await chrome.runtime.sendMessage({
        type: "APOCALIPSE_BLOB_CHUNK",
        request: { uploadId, data },
      });
      if (result?.error) throw new Error(result.error);
    }
  };
  const titleFor = (element) => element?.getAttribute?.("aria-label") || element?.title || element?.alt || document.title;
  const pageThumbnail = (element) => {
    const candidates = [
      element?.poster,
      element?.getAttribute?.("poster"),
      document.querySelector('meta[property="og:image:secure_url"]')?.content,
      document.querySelector('meta[property="og:image"]')?.content,
      document.querySelector('meta[name="twitter:image"]')?.content,
      document.querySelector('meta[name="twitter:image:src"]')?.content,
      document.querySelector('link[rel="image_src"]')?.href,
      element?.closest?.("figure,article,[class*=player],[class*=video]")?.querySelector?.("img")?.currentSrc,
    ];
    for (const script of document.querySelectorAll('script[type="application/ld+json"]')) {
      try {
        const data = JSON.parse(script.textContent || "null");
        const nodes = Array.isArray(data) ? data : [data];
        for (const node of nodes) {
          const value = Array.isArray(node?.thumbnailUrl) ? node.thumbnailUrl[0] : node?.thumbnailUrl;
          if (value) candidates.push(value);
        }
      } catch {}
    }
    for (const candidate of candidates) {
      const url = absolute(candidate);
      if (url && /^https?:/i.test(url)) return url;
    }
    return "";
  };
  const thumbnailFor = (element, kind) => {
    if (kind === "audio") return "";
    if (element?.tagName === "IMG") return element.currentSrc || element.src || "";
    return pageThumbnail(element);
  };
  const isFacebookMediaUrl = (url) => {
    try {
      const parsed = new URL(url, location.href);
      if (!/(^|\.)facebook\.com$/i.test(parsed.hostname)) return false;
      if (/\/(?:watch\/hashtag|hashtag)(?:\/|$)/i.test(parsed.pathname)) return false;
      return /(?:^|\/)(?:reel|reels|watch|videos|posts|share)(?:\/|$)/i.test(parsed.pathname)
        || /\/(?:permalink|story)\.php$/i.test(parsed.pathname)
        || parsed.searchParams.has("fbid")
        || parsed.searchParams.has("story_fbid");
    } catch { return false; }
  };
  const isTikTokVideoUrl = (url) => {
    try {
      const parsed = new URL(url, location.href);
      return /(^|\.)tiktok\.com$/i.test(parsed.hostname) && /\/@[^/]+\/video\/\d+/i.test(parsed.pathname);
    } catch { return false; }
  };
  const tikTokUrlFor = (element) => {
    if (!/(^|\.)tiktok\.com$/i.test(location.hostname)) return null;
    if (isTikTokVideoUrl(location.href)) return location.href;
    const videoRect = element?.getBoundingClientRect?.();
    const card = element?.closest?.([
      "article",
      '[data-e2e*="feed"]',
      '[data-e2e*="recommend"]',
      '[class*="DivItemContainer"]',
      '[class*="DivVideoContainer"]',
    ].join(","));
    const cardAnchors = [...(card?.querySelectorAll?.('a[href*="/video/"]') || [])];
    for (const anchor of cardAnchors) {
      const url = absolute(anchor.href);
      if (isTikTokVideoUrl(url)) return url;
    }
    let container = element;
    for (let depth = 0; container && depth < 28; depth += 1, container = container.parentElement) {
      const anchors = container.querySelectorAll?.('a[href*="/video/"]') || [];
      for (const anchor of anchors) {
        const url = absolute(anchor.href);
        if (isTikTokVideoUrl(url)) return url;
      }
      const markup = (container.innerHTML || "").replaceAll("\\/", "/");
      const path = markup.match(/\/@[^/"'<>\\s]+\/video\/\d+/i)?.[0];
      if (path && isTikTokVideoUrl(path)) return absolute(path);
    }
    if (videoRect) {
      const nearest = [...document.querySelectorAll('a[href*="/video/"]')]
        .map((anchor) => ({ anchor, url: absolute(anchor.href), rect: anchor.getBoundingClientRect() }))
        .filter(({ url, rect }) => isTikTokVideoUrl(url) && rect.width > 0 && rect.height > 0
          && rect.bottom >= videoRect.top && rect.top <= videoRect.bottom)
        .sort((left, right) => {
          const videoCenter = (videoRect.top + videoRect.bottom) / 2;
          return Math.abs((left.rect.top + left.rect.bottom) / 2 - videoCenter)
            - Math.abs((right.rect.top + right.rect.bottom) / 2 - videoCenter);
        })[0];
      if (nearest) return nearest.url;
    }
    return null;
  };
  const facebookUrlFor = (element) => {
    if (!/(^|\.)facebook\.com$/i.test(location.hostname)) return null;
    if (isFacebookMediaUrl(location.href)) return location.href;
    const selector = [
      'a[href*="/reel/"]',
      'a[href*="/reels/"]',
      'a[href*="/videos/"]',
      'a[href*="/posts/"]',
      'a[href*="/watch/"]',
      'a[href*="/watch?"]',
      'a[href*="/permalink.php"]',
      'a[href*="/story.php"]',
      'a[href*="/share/r/"]',
      'a[href*="/share/v/"]',
    ].join(",");
    let container = element;
    for (let depth = 0; container && depth < 10; depth += 1, container = container.parentElement) {
      const anchor = container.querySelector?.(selector);
      const url = absolute(anchor?.href);
      if (url && isFacebookMediaUrl(url)) return url;
      const markup = container.innerHTML || "";
      const path = markup.replaceAll("\\/", "/").match(/\/(?:reel|reels|videos|posts|share\/[rv])\/[A-Za-z0-9._-]+/i)?.[0];
      if (path && isFacebookMediaUrl(path)) return absolute(path);
    }
    return null;
  };
  const facebookMediaId = (url) => {
    try {
      const parsed = new URL(url, location.href);
      const numeric = parsed.pathname.match(/\/(?:reel|reels|videos|posts)\/(\d+)/i)?.[1]
        || parsed.searchParams.get("v")
        || parsed.searchParams.get("fbid")
        || parsed.searchParams.get("story_fbid");
      if (numeric) return numeric;
      const shared = parsed.pathname.match(/\/share\/[rv]\/([^/?#]+)/i)?.[1];
      if (shared) return shared.replace(/[^A-Za-z0-9_-]+/g, "");
    } catch {}
    return null;
  };
  const facebookDownloadTitle = (url) => {
    const id = facebookMediaId(url);
    return id ? `${id}.mp4` : "facebook-video.mp4";
  };

  const waitForFacebookUrl = async (element, attempts = 20) => {
    for (let attempt = 0; attempt < attempts; attempt += 1) {
      await new Promise((resolve) => setTimeout(resolve, 100));
      const revealed = facebookUrlFor(element);
      if (revealed) return revealed;
    }
    return null;
  };
  const facebookUrlFromMenu = async (element) => {
    const videoRect = element.getBoundingClientRect();
    const post = element.closest?.('[role="article"],article') || element.parentElement;
    let container = post;
    for (let depth = 0; container?.parentElement && depth < 6; depth += 1) {
      const rect = container.getBoundingClientRect();
      if (rect.top <= videoRect.top - 20 && rect.right >= videoRect.right - 20) break;
      container = container.parentElement;
    }
    const buttons = [...(container?.querySelectorAll?.('button,[role="button"]') || [])];
    const labeled = buttons.filter((button) => {
      const label = `${button.getAttribute("aria-label") || ""} ${button.title || ""} ${button.textContent || ""}`.trim();
      return /(?:ações|acoes|opções|opcoes|actions|options|more|menu|更多|更多选项)/i.test(label) || /^\s*(?:\.\.\.|…|⋯)\s*$/.test(label);
    });
    const candidates = labeled.length ? labeled : buttons;
    let menuButton = candidates.filter((button) => {
      const rect = button.getBoundingClientRect();
      return rect.width > 0 && rect.height > 0 && rect.top < videoRect.top + 80;
    }).sort((left, right) => {
      const score = (button) => {
        const rect = button.getBoundingClientRect();
        const label = `${button.getAttribute("aria-label") || ""} ${button.title || ""} ${button.textContent || ""}`;
        const postMenuBonus = /(?:publicação|publicacao|anúncio|anuncio|\bpost\b|\bad\b)/i.test(label) ? -1000 : 0;
        return postMenuBonus + Math.abs(rect.right - videoRect.right) + Math.abs(rect.bottom - videoRect.top);
      };
      return score(left) - score(right);
    })[0];
    if (!menuButton) {
      menuButton = buttons.filter((button) => {
        const rect = button.getBoundingClientRect();
        return rect.width > 0 && rect.height > 0 && rect.top >= videoRect.top - 180 && rect.bottom <= videoRect.top + 100;
      }).sort((left, right) => {
        const score = (button) => {
          const rect = button.getBoundingClientRect();
          return Math.abs(rect.right - videoRect.right) + Math.abs(rect.bottom - videoRect.top);
        };
        return score(left) - score(right);
      })[0];
    }
    if (!menuButton) return null;
    menuButton.click();
    let copyItem = null;
    for (let attempt = 0; attempt < 20 && !copyItem; attempt += 1) {
      await new Promise((resolve) => setTimeout(resolve, 100));
      copyItem = [...document.querySelectorAll('[role="menuitem"],[role="menuitemradio"]')].find((item) =>
        /(?:copiar link|copy link|复制链接|複製連結)/i.test(item.textContent || ""));
    }
    if (!copyItem) return null;
    let previousClipboard = "";
    try { previousClipboard = await navigator.clipboard.readText(); } catch {}
    copyItem.click();
    for (let attempt = 0; attempt < 20; attempt += 1) {
      await new Promise((resolve) => setTimeout(resolve, 100));
      try {
        const copied = await navigator.clipboard.readText();
        if (copied && copied !== previousClipboard && isFacebookMediaUrl(copied)) return copied;
      } catch {}
    }
    return null;
  };
  const revealFacebookUrl = async (element) => {
    if (!/(^|\.)facebook\.com$/i.test(location.hostname)) return null;
    // The site's own Copy link command is the authoritative association
    // between a feed card and its canonical Reel/post URL.
    const copiedUrl = await facebookUrlFromMenu(element);
    if (copiedUrl) return copiedUrl;
    const immediate = facebookUrlFor(element);
    if (immediate) return immediate;
    const rect = element.getBoundingClientRect();
    const target = document.elementFromPoint(rect.left + rect.width / 2, rect.top + rect.height / 2)
      || element.closest?.('a[href],[role="link"]') || element;
    target.click?.();
    return await waitForFacebookUrl(element) || await facebookUrlFromMenu(element);
  };
  const collect = () => {
    const items = new Map();
    const add = (url, kind, element, thumbnail) => {
      url = absolute(url);
      if (!url || !/^https?:/.test(url)) return;
      const resource = performance.getEntriesByName(url).at(-1);
      const measuredSize = Number(resource?.encodedBodySize || resource?.transferSize || 0);
      const duration = Number(element?.duration);
      items.set(`${kind}:${url}`, {
        url,
        kind,
        thumbnail: thumbnail ?? thumbnailFor(element, kind),
        title: titleFor(element),
        size: measuredSize > 0 && !/\.m3u8(?:$|[?#])/i.test(url) ? measuredSize : null,
        duration: Number.isFinite(duration) && duration > 0 ? duration : null,
      });
    };
    document.querySelectorAll("video").forEach((element) => {
      add(element.currentSrc || element.src, "video", element);
      element.querySelectorAll("source").forEach((source) => add(source.src, "video", element));
      const facebookUrl = facebookUrlFor(element);
      if (facebookUrl) add(facebookUrl, "video", element);
      const tikTokUrl = tikTokUrlFor(element);
      if (tikTokUrl) add(tikTokUrl, "video", element);
    });
    const facebookPageUrl = facebookUrlFor(document.querySelector("video"));
    if (facebookPageUrl) add(facebookPageUrl, "video", document.querySelector("video"));
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
      if (/\.m3u8(?:$|[?#])/i.test(entry.name)) add(entry.name, "video", document.querySelector("video"));
    });
    return [...items.values()];
  };
  const downloadLabel = () => {
    const value = (navigator.language || "en").toLowerCase();
    return value.startsWith("zh") ? "下载" : value.startsWith("pt") ? "Baixar" : "Download";
  };
  const recordingLabels = () => {
    const value = (navigator.language || "en").toLowerCase();
    if (value.startsWith("zh")) return { record: "● 录制", stop: "■ 停止并保存", uploading: "正在发送…", done: "已保存", unavailable: "此视频无法由浏览器录制" };
    if (value.startsWith("pt")) return { record: "● Gravar", stop: "■ Parar e salvar", uploading: "Enviando…", done: "Gravação salva", unavailable: "Este vídeo não permite gravação pelo navegador" };
    return { record: "● Record", stop: "■ Stop and save", uploading: "Uploading…", done: "Recording saved", unavailable: "This video cannot be recorded by the browser" };
  };
  const clockLabel = (seconds) => {
    const value = Math.max(0, Math.floor(seconds));
    return `${String(Math.floor(value / 60)).padStart(2, "0")}:${String(value % 60).padStart(2, "0")}`;
  };
  const trace = (eventName, mode, detail = {}) => chrome.runtime.sendMessage({
    type: "APOCALIPSE_CAPTURE_TRACE",
    eventName,
    mode,
    traceId: crypto.randomUUID(),
    pageUrl: location.href,
    at: Date.now(),
    detail,
  }).catch(() => {});
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
  const looksLikeDownloadControl = (event) => {
    for (const node of event.composedPath?.() || []) {
      const label = `${node?.getAttribute?.("aria-label") || ""} ${node?.getAttribute?.("data-title") || ""} ${node?.title || ""} ${node?.textContent || ""}`
        .replace(/\s+/g, " ").trim().slice(0, 240);
      if (/(?:download|baixar|descarregar|descargar|télécharger|下载)/i.test(label)) return { node, label };
      if (node?.hasAttribute?.("download")) return { node, label };
    }
    return null;
  };
  const forceKnownHlsDownload = (event) => {
    if (/^(?:www\.)?youtube\.com$/i.test(location.hostname) && location.pathname === "/watch") return false;
    const control = looksLikeDownloadControl(event);
    const video = document.querySelector("video");
    const hls = hlsForPage();
    if (!control || !video || !hls.candidates.length) return false;

    event.preventDefault();
    event.stopImmediatePropagation();
    trace("native_force_hls_candidate", "force", {
      control: control.label,
      candidates: hls.candidates.length,
      expectedDuration: Number.isFinite(video.duration) ? video.duration : null,
    });
    void (async () => {
      try {
        const selected = await chrome.runtime.sendMessage({
          type: "APOCALIPSE_SELECT_HLS",
          urls: hls.candidates,
          expectedDuration: Number.isFinite(video.duration) ? video.duration : null,
        });
        const url = selected?.url || hls.fallback;
        const requestUrls = [...new Set(selected?.requestUrls?.length ? selected.requestUrls : hls.candidates)];
        const result = await chrome.runtime.sendMessage({
          type: "APOCALIPSE_DOWNLOAD",
          item: {
            url,
            duration: selected?.duration || null,
            requestUrls,
            userAgent: navigator.userAgent,
            kind: "video",
            title: document.title,
            thumbnail: thumbnailFor(video, "video"),
          },
        });
        trace(result?.target === "apocalipse" ? "native_force_hls_handed_off" : "native_force_hls_failed", "force", {
          target: result?.target || "none",
          error: result?.error || "none",
          candidates: requestUrls.length,
        });
      } catch (error) {
        trace("native_force_hls_failed", "force", { error: String(error), candidates: hls.candidates.length });
      }
    })();
    return true;
  };
  document.addEventListener("click", (event) => {
    if (event.defaultPrevented || event.button !== 0 || event.metaKey) return;
    const bypass = modifierPressed(event, shortcutKeys.bypass);
    const force = modifierPressed(event, shortcutKeys.force);
    if (bypass) {
      chrome.runtime.sendMessage({ type: "APOCALIPSE_BYPASS_NEXT", ttlMs: 4000 }).catch(() => {});
      return;
    }
    if (force) {
      if (forceKnownHlsDownload(event)) return;
      // Force is a transaction, not an instruction to steal the visible href.
      // Let the page run and observe the real downstream file request/download.
      chrome.runtime.sendMessage({ type: "APOCALIPSE_FORCE_NEXT", ttlMs: 20000 }).catch(() => {});
      return;
    }
    if (event.ctrlKey || event.shiftKey || event.altKey) return;
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
    return { candidates: urls, fallback: masters.at(-1) || urls.at(-1) || null };
  };
  const downloadUrlFor = (element) => {
    if (element.tagName === "VIDEO" && /^(?:www\.)?youtube\.com$/.test(location.hostname) && location.pathname === "/watch") return location.href;
    if (element.tagName === "VIDEO") {
      const tikTokUrl = tikTokUrlFor(element);
      if (tikTokUrl) return tikTokUrl;
    }
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
    if (element.tagName !== "VIDEO") return immediate;
    // YouTube has its own format-selection pipeline (video + audio merging).
    // Keep both regular videos and live streams out of the generic HLS route.
    if (/^(?:www\.)?youtube\.com$/i.test(location.hostname) && location.pathname === "/watch") return immediate;
    const hls = hlsForPage();
    if (!hls.candidates.length) return immediate;
    try {
      const selected = await chrome.runtime.sendMessage({
        type: "APOCALIPSE_SELECT_HLS",
        urls: hls.candidates,
        expectedDuration: Number.isFinite(element.duration) ? element.duration : null,
      });
      return selected || { url: hls.fallback || immediate, duration: null, requestUrls: hls.candidates };
    } catch { return { url: hls.fallback || immediate, duration: null, requestUrls: hls.candidates }; }
  };
  let overlayTimer;
  const activeOverlays = new Map();
  const installOverlays = () => {
    if (/(^|\.)chatgpt\.com$/.test(location.hostname)) return;
    for (const overlay of activeOverlays.values()) {
      if (!overlay.element.isConnected || overlay.pageUrl !== location.href) overlay.cleanup();
    }
    const isFacebookReelsPage = /(^|\.)facebook\.com$/i.test(location.hostname)
      && /(?:^|\/)reels?(?:\/|$)/i.test(location.pathname);
    const isInstagramReelsPage = /(^|\.)instagram\.com$/i.test(location.hostname)
      && /(?:^|\/)reels?(?:\/|$)/i.test(location.pathname);
    const isTikTokPage = /(^|\.)tiktok\.com$/i.test(location.hostname);
    let activeFacebookReel = null;
    if (isFacebookReelsPage) {
      const viewportCenter = innerHeight / 2;
      const candidates = [...document.querySelectorAll("video")]
        .map((video) => ({ video, rect: video.getBoundingClientRect() }))
        .filter(({ rect }) => rect.width >= 100 && rect.height >= 55 && rect.bottom > 0 && rect.top < innerHeight)
        .sort((left, right) => {
          const leftCenter = Math.abs((left.rect.top + left.rect.bottom) / 2 - viewportCenter);
          const rightCenter = Math.abs((right.rect.top + right.rect.bottom) / 2 - viewportCenter);
          return leftCenter - rightCenter;
        });
      activeFacebookReel = candidates[0]?.video || null;
      for (const overlay of [...activeOverlays.values()]) {
        if (overlay.element?.tagName === "VIDEO" && overlay.element !== activeFacebookReel) overlay.cleanup();
      }
    }
    let activeInstagramReel = null;
    if (isInstagramReelsPage) {
      const viewportCenter = innerHeight / 2;
      activeInstagramReel = [...document.querySelectorAll("video")]
        .map((video) => ({ video, rect: video.getBoundingClientRect() }))
        .filter(({ rect }) => rect.width >= 100 && rect.height >= 55 && rect.bottom > 0 && rect.top < innerHeight)
        .sort((left, right) =>
          Math.abs((left.rect.top + left.rect.bottom) / 2 - viewportCenter)
          - Math.abs((right.rect.top + right.rect.bottom) / 2 - viewportCenter))[0]?.video || null;
      for (const overlay of [...activeOverlays.values()]) {
        if (overlay.element?.tagName === "VIDEO" && overlay.element !== activeInstagramReel) overlay.cleanup();
      }
    }
    let activeTikTokVideo = null;
    if (isTikTokPage) {
      const viewportCenter = innerHeight / 2;
      activeTikTokVideo = [...document.querySelectorAll("video")]
        .map((video) => ({ video, rect: video.getBoundingClientRect() }))
        .filter(({ rect }) => rect.width >= 100 && rect.height >= 55 && rect.bottom > 0 && rect.top < innerHeight)
        .sort((left, right) =>
          Math.abs((left.rect.top + left.rect.bottom) / 2 - viewportCenter)
          - Math.abs((right.rect.top + right.rect.bottom) / 2 - viewportCenter))[0]?.video || null;
      for (const overlay of [...activeOverlays.values()]) {
        if (overlay.element?.tagName === "VIDEO" && overlay.element !== activeTikTokVideo) overlay.cleanup();
      }
    }

    document.querySelectorAll("video,audio").forEach((element) => {
      if (element.dataset.apocalipseButton) return;
      const isYouTubeVideo = element.tagName === "VIDEO" && /^(?:www\.)?youtube\.com$/.test(location.hostname) && location.pathname === "/watch";
      const isFacebookVideo = element.tagName === "VIDEO" && /(^|\.)facebook\.com$/i.test(location.hostname);
      if (isFacebookReelsPage && isFacebookVideo && element !== activeFacebookReel) return;
      if (isInstagramReelsPage && element.tagName === "VIDEO" && element !== activeInstagramReel) return;
      if (isTikTokPage && element.tagName === "VIDEO" && element !== activeTikTokVideo) return;
      const tikTokUrl = element.tagName === "VIDEO" ? tikTokUrlFor(element) : null;
      const isTikTokVideo = Boolean(tikTokUrl);
      const url = isFacebookVideo ? facebookUrlFor(element) || location.href : tikTokUrl || downloadUrlFor(element);
      const liveMediaUrl = element.currentSrc || element.src || "";
      const canDownload = Boolean((url && /^https?:/.test(url)) || /^blob:/i.test(liveMediaUrl));
      const canRecord = element.tagName === "VIDEO" && Boolean(globalThis.MediaRecorder)
        && Boolean(element.captureStream || element.webkitCaptureStream);
      if (!canDownload && !canRecord) return;
      element.dataset.apocalipseButton = "1";
      const button = document.createElement("button");
      button.type = "button";
      button.className = "apocalipse-media-download";
      button.textContent = `⇩ ${downloadLabel()}`;
      button.title = "Apocalipse Download Manager";
      button.hidden = !canDownload;
      let recordButton = null;
      button.addEventListener("click", async (event) => {
        event.preventDefault();
        event.stopPropagation();
        const originalText = button.textContent;
        button.textContent = "…";
        trace("overlay_download_clicked", "download", { tag: element.tagName, facebook: isFacebookVideo, tiktokPage: isTikTokPage, tiktokPermalink: isTikTokVideo, overlays: activeOverlays.size });
        const visibleFacebookUrl = isFacebookVideo && isFacebookMediaUrl(location.href) ? location.href : null;
        let enteredTikTokFullscreen = false;
        if (isTikTokPage && !tikTokUrlFor(element) && document.fullscreenElement !== element && element.requestFullscreen) {
          try {
            await element.requestFullscreen();
            enteredTikTokFullscreen = true;
            await new Promise((resolve) => setTimeout(resolve, 500));
          } catch {}
        }
        const resolved = visibleFacebookUrl || (isFacebookVideo ? await revealFacebookUrl(element) : tikTokUrlFor(element) || await resolveDownloadUrl(element));
        const liveSource = String(element.currentSrc || element.src || "");
        const liveBlobUrl = /^blob:/i.test(liveSource) ? liveSource : null;
        const liveHttpUrl = /^https?:/i.test(liveSource) ? liveSource : null;
        const networkMediaUrl = recentNetworkMediaUrl();
        const capturedSocialMedia = (isTikTokPage || (isFacebookVideo && isFacebookMediaUrl(location.href)))
          ? await chrome.runtime.sendMessage({ type: "APOCALIPSE_RECENT_TAB_MEDIA" }).catch(() => null)
          : null;
        const browserVideoMedia = capturedSocialMedia?.media?.find((item) =>
          /^https?:/i.test(item.url || "") && /^(?:video|audio)\//i.test(item.contentType || ""))?.url
          || capturedSocialMedia?.media?.find((item) => /^https?:/i.test(item.url || ""))?.url
          || null;

        // For a real <video>, the source feeding the player is more authoritative
        // than location.href. Try readable blob first; for MSE blobs, fall through
        // to the most recent underlying media request captured by Performance API.
        if (liveBlobUrl) {
          try {
            const fallbackName = isFacebookVideo ? facebookDownloadTitle(location.href) : null;
            await uploadBlobUrl(liveBlobUrl, fallbackName);
            button.textContent = "✓";
            button.title = "Enviado ao Apocalipse";
            setTimeout(() => { button.textContent = originalText; }, 1500);
            return;
          } catch (error) {
            console.debug("Apocalipse live blob is MSE/unreadable; trying network media", error);
          }
        }

        // A resolved HLS manifest is more authoritative than incidental network
        // traffic or a generic HTTP source exposed by the player.
        let currentUrl = isFacebookVideo
          ? (browserVideoMedia || ((typeof resolved === "string" && isFacebookMediaUrl(resolved))
            ? resolved
            : (resolved?.url || liveHttpUrl || networkMediaUrl || resolved)))
          : (isTikTokPage
            ? (browserVideoMedia || liveHttpUrl || networkMediaUrl || resolved?.url || resolved)
            : (resolved?.url || resolved || liveHttpUrl || networkMediaUrl || (isYouTubeVideo ? location.href : null)));
        const facebookPlayableUrl = isFacebookVideo && currentUrl && (
          isFacebookMediaUrl(currentUrl)
          || /\.(?:mp4|webm|m3u8|mpd)(?:[?#]|$)/i.test(currentUrl)
          || (/(?:fbcdn|fbsbx|video)/i.test(currentUrl)
            && !/\.(?:avif|bmp|gif|ico|jpe?g|png|svg|webp)(?:[?#]|$)/i.test(currentUrl))
        );
        if (!currentUrl || (isFacebookVideo && !facebookPlayableUrl)) {
          trace("overlay_download_unresolved", "download", { liveBlob: Boolean(liveBlobUrl), liveHttp: Boolean(liveHttpUrl), networkMedia: Boolean(networkMediaUrl), facebook: isFacebookVideo });
          button.textContent = "⚠";
          button.title = "Abra o vídeo ou use os três pontos e Copiar link";
          setTimeout(() => { button.textContent = originalText; }, 2500);
          return;
        }
        if (/^blob:/i.test(String(currentUrl || ""))) {
          try {
            await uploadBlobUrl(currentUrl);
            button.textContent = "✓";
            button.title = "Enviado ao Apocalipse";
            setTimeout(() => { button.textContent = originalText; }, 1500);
            return;
          } catch (error) {
            console.debug("Apocalipse direct blob failed", error);
          }
        }
        const directFacebookMedia = isFacebookVideo ? absolute(element.currentSrc || element.src) : null;
        const requestUrls = [...new Set([
          ...(resolved?.requestUrls?.length ? resolved.requestUrls : (/\.m3u8(?:$|[?#])/i.test(String(currentUrl)) ? hlsForPage().candidates : [])),
          ...(directFacebookMedia && /^https?:/i.test(directFacebookMedia) ? [directFacebookMedia] : []),
        ])];
        trace("overlay_download_candidate_selected", "download", {
          facebook: isFacebookVideo,
          tiktokPage: isTikTokPage,
          tiktokPermalink: Boolean(isTikTokVideoUrl(currentUrl)),
          browserCapturedMedia: Boolean(browserVideoMedia && currentUrl === browserVideoMedia),
          browserCandidates: capturedSocialMedia?.media?.length || 0,
          browserCandidateAgeMs: capturedSocialMedia?.media?.[0]?.ageMs ?? -1,
          browserCandidateType: capturedSocialMedia?.media?.[0]?.contentType || "none",
          browserCandidateBytes: capturedSocialMedia?.media?.[0]?.contentLength || 0,
          directPlayer: Boolean(liveHttpUrl && currentUrl === liveHttpUrl),
          networkMedia: Boolean(networkMediaUrl && currentUrl === networkMediaUrl),
          pageFallback: Boolean(isFacebookVideo && isFacebookMediaUrl(currentUrl)),
          candidate: currentUrl,
        });
        chrome.runtime.sendMessage({ type: "APOCALIPSE_DOWNLOAD", item: { url: currentUrl, duration: resolved?.duration || null, requestUrls, userAgent: navigator.userAgent, kind: element.tagName.toLowerCase(), title: isFacebookVideo ? facebookDownloadTitle(currentUrl) : document.title, thumbnail: thumbnailFor(element, "video") } }, (result) => {
          const failed = chrome.runtime.lastError || !result?.ok;
          trace(failed ? "overlay_download_failed" : "overlay_download_handed_off", "download", { target: result?.target || "none", error: result?.error || chrome.runtime.lastError?.message || "none", candidates: requestUrls.length });
          button.textContent = failed ? "⚠" : "✓";
          if (failed) button.title = result?.error || chrome.runtime.lastError?.message || "Apocalipse unavailable";
          setTimeout(() => { button.textContent = originalText; }, 1500);
          if (enteredTikTokFullscreen && document.fullscreenElement) document.exitFullscreen?.().catch(() => {});
        });
      });
      if (element.tagName === "VIDEO") {
        const record = document.createElement("button");
        recordButton = record;
        record.type = "button";
        record.className = "apocalipse-media-download apocalipse-media-record";
        const labels = recordingLabels();
        record.textContent = labels.record;
        record.title = labels.record;
        let recorder = null;
        let previousLoop = false;
        let startedAt = 0;
        let clockTimer = null;
        let stopPoll = null;
        record.addEventListener("click", async (event) => {
          event.preventDefault();
          event.stopPropagation();
          if (recorder?.state === "recording") {
            trace("recording_stop_clicked", "record", { elapsedMs: Date.now() - startedAt });
            recorder.stop();
            record.disabled = true;
            record.textContent = labels.uploading;
            return;
          }
          try {
            trace("recording_start_clicked", "record", { captureStream: Boolean(element.captureStream || element.webkitCaptureStream), mediaRecorder: Boolean(globalThis.MediaRecorder) });
            const capture = element.captureStream?.bind(element) || element.webkitCaptureStream?.bind(element);
            if (!capture || !globalThis.MediaRecorder) throw new Error("capture_not_supported");
            if (Number.isFinite(element.duration)) element.currentTime = 0;
            previousLoop = element.loop;
            element.loop = false;
            const stream = capture();
            const mimeType = ["video/webm;codecs=vp9,opus", "video/webm;codecs=vp8,opus", "video/webm"]
              .find((type) => MediaRecorder.isTypeSupported(type)) || "";
            const safeTitle = (document.title || "recording").replace(/[<>:\"/\\|?*]+/g, "_").slice(0, 120);
            const begin = await chrome.runtime.sendMessage({
              type: "APOCALIPSE_BLOB_BEGIN",
              request: { fileName: `${safeTitle}.recording.webm`, total: 0, source: location.href, streaming: true, recording: true },
            });
            if (!begin?.uploadId) throw new Error(begin?.error || "recording_begin_failed");
            trace("recording_bridge_started", "record", { uploadId: begin.uploadId, mimeType });
            let uploadQueue = Promise.resolve();
            let uploadError = null;
            recorder = new MediaRecorder(stream, mimeType ? { mimeType } : undefined);
            recorder.ondataavailable = ({ data }) => {
              if (!data.size || uploadError) return;
              uploadQueue = uploadQueue.then(() => appendBlob(begin.uploadId, data)).catch((error) => { uploadError = error; });
            };
            recorder.onstop = async () => {
              if (clockTimer) clearInterval(clockTimer);
              if (stopPoll) clearInterval(stopPoll);
              stream.getTracks().forEach((track) => track.stop());
              element.loop = previousLoop;
              record.textContent = labels.uploading;
              try {
                await uploadQueue;
                if (uploadError) throw uploadError;
                const result = await chrome.runtime.sendMessage({ type: "APOCALIPSE_BLOB_END", request: { uploadId: begin.uploadId } });
                if (result?.error) throw new Error(result.error);
                record.textContent = `✓ ${labels.done}`;
                trace("recording_completed", "record", { uploadId: begin.uploadId, elapsedMs: Date.now() - startedAt });
              } catch (error) {
                trace("recording_upload_failed", "record", { uploadId: begin.uploadId, error: String(error) });
                console.error("Apocalipse recorder upload", error);
                record.textContent = "⚠";
              } finally {
                recorder = null;
                setTimeout(() => { record.disabled = false; record.textContent = labels.record; }, 2200);
              }
            };
            element.addEventListener("ended", () => {
              if (recorder?.state === "recording") recorder.stop();
            }, { once: true });
            recorder.start(1000);
            await element.play();
            startedAt = Date.now();
            record.textContent = `${labels.stop} · 00:00`;
            clockTimer = setInterval(() => {
              if (recorder?.state === "recording") record.textContent = `${labels.stop} · ${clockLabel((Date.now() - startedAt) / 1000)}`;
            }, 1000);
            stopPoll = setInterval(async () => {
              if (recorder?.state !== "recording") return;
              const status = await chrome.runtime.sendMessage({ type: "APOCALIPSE_BLOB_STATUS", request: { uploadId: begin.uploadId } }).catch(() => null);
              if (status?.stop && recorder?.state === "recording") recorder.stop();
            }, 1000);
          } catch (error) {
            trace("recording_start_failed", "record", { error: String(error) });
            console.error("Apocalipse recorder", error);
            record.textContent = "⚠";
            record.title = labels.unavailable;
            setTimeout(() => { record.textContent = labels.record; }, 2000);
          }
        });
      }
      let positionTimer = null;
      let cleanupOverlay = () => {};
      const position = () => {
        if (!element.isConnected) {
          cleanupOverlay();
          return;
        }
        const anchor = isYouTubeVideo
          ? document.querySelector("#movie_player") || element.closest("ytd-player") || element
          : element;
        const rect = anchor.getBoundingClientRect();
        const left = Math.max(6, rect.left + scrollX + 10);
        const top = rect.top + scrollY + 10;
        button.style.left = `${left}px`;
        button.style.top = `${Math.max(6, top)}px`;
        button.hidden = !canDownload || rect.width < 100 || rect.height < 55;
        if (recordButton) {
          const recordLeft = canDownload && !button.hidden
            ? left + button.offsetWidth + 8
            : left;
          recordButton.style.left = `${recordLeft}px`;
          recordButton.style.top = `${Math.max(6, top)}px`;
          recordButton.hidden = rect.width < 100 || rect.height < 55;
        }
      };
      document.documentElement.append(button);
      if (recordButton) document.documentElement.append(recordButton);
      cleanupOverlay = () => {
        button.remove();
        recordButton?.remove();
        if (positionTimer) clearInterval(positionTimer);
        removeEventListener("scroll", position);
        removeEventListener("resize", position);
        delete element.dataset.apocalipseButton;
        activeOverlays.delete(element);
      };
      activeOverlays.set(element, { element, pageUrl: location.href, cleanup: cleanupOverlay });
      const duplicateButtons = document.querySelectorAll(".apocalipse-media-download").length - activeOverlays.size * 2;
      trace("overlay_installed", "overlay", { tag: element.tagName, canDownload, canRecord, active: activeOverlays.size, duplicateDelta: duplicateButtons });
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
  style.textContent = ".apocalipse-media-download{position:absolute!important;z-index:2147483647!important;border:1px solid #4c6470!important;border-radius:8px!important;padding:8px 11px!important;background:#111a20e8!important;color:#f3fbff!important;font:700 13px system-ui!important;box-shadow:0 3px 12px #0008!important;backdrop-filter:blur(5px)!important;cursor:pointer!important;transition:border-color .15s,background .15s,box-shadow .15s!important}.apocalipse-media-download:hover{border-color:#31d9ee!important;background:#15262eef!important;box-shadow:0 3px 14px #00cce755!important}.apocalipse-media-download:disabled{cursor:wait!important;opacity:.85!important}.apocalipse-media-record{border-color:#c94a5e!important;color:#ffd8de!important;background:#35151ce8!important}.apocalipse-media-record:hover{border-color:#ff6078!important;background:#4a1922ef!important;box-shadow:0 3px 14px #ff405555!important}";
  document.documentElement.append(style);
  new MutationObserver(scheduleOverlays).observe(document.documentElement, { childList: true, subtree: true, attributes: true, attributeFilter: ["src", "poster"] });
  scheduleOverlays();
  setInterval(scheduleOverlays, 2000);
  chrome.runtime.onMessage.addListener((message, _sender, reply) => {
    if (message?.type === "APOCALIPSE_UPLOAD_BLOB" && /^blob:/i.test(message.url || "")) {
      (async () => {
        const response = await fetch(message.url);
        const blob = await response.blob();
        reply({ started: true });
        await uploadBlob(blob, message.fileName);
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
    trace("popup_scan_completed", "scan", {
      frame: window === window.top ? "top" : "child",
      detected: found.length,
      videos: found.filter((item) => item.kind === "video").length,
      audio: found.filter((item) => item.kind === "audio").length,
      images: found.filter((item) => item.kind === "image").length,
    });
    return true;
  });

  // MAIN-world pre-download relay (no CDP). The page hook traps the final URL
  // before Chrome creates a native download, while this isolated-world script
  // retains access to chrome.runtime and the existing Alt/Shift configuration.
  const postApocalipseShortcutConfig = () => {
    window.postMessage({
      source: "apocalipse-extension",
      type: "shortcut-config",
      bypass: shortcutKeys.bypass,
      force: shortcutKeys.force,
    }, "*");
  };
  postApocalipseShortcutConfig();
  chrome.storage.onChanged.addListener((changes, area) => {
    if (area === "local" && (changes.forceShortcut || changes.bypassShortcut)) {
      setTimeout(postApocalipseShortcutConfig, 0);
    }
  });
  window.addEventListener("message", (event) => {
    if (event.source !== window) return;
    const data = event.data;
    if (data?.source === "apocalipse-page-hook" && data.type === "capture-trace") {
      chrome.runtime.sendMessage({ type: "APOCALIPSE_CAPTURE_TRACE", eventName: data.eventName, mode: data.mode, detail: data.detail || {}, traceId: data.traceId, pageUrl: location.href, at: data.at || Date.now() }).catch(() => {});
      return;
    }
    if (!data || data.source !== "apocalipse-page-hook" || data.type !== "pre-download-url") return;
    chrome.runtime.sendMessage({
      type: "APOCALIPSE_PRE_DOWNLOAD_URL",
      url: data.url,
      pageUrl: location.href,
      fileName: data.fileName || "",
      source: data.kind || "main-world",
      force: Boolean(data.force),
      method: data.method || "GET",
      body: data.body || null,
      contentType: data.contentType || null,
    }).then((result) => {
      window.postMessage({
        source: "apocalipse-extension",
        type: "pre-download-result",
        requestId: data.requestId,
        result,
      }, "*");
    }).catch((error) => {
      window.postMessage({
        source: "apocalipse-extension",
        type: "pre-download-result",
        requestId: data.requestId,
        result: { ok: false, error: String(error) },
      }, "*");
    });
  });

})();
