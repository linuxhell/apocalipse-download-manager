(() => {
  globalThis.ADM_DIAG?.register("content.js");
  let shortcutKeys = { force: "Shift", bypass: "Alt" };
  const heldShortcutKeys = new Set();
  let interfaceLanguage = "en";
  let interfaceTheme = "void";
  let refreshOverlayLanguages = () => {};
  const extensionContextActive = () => {
    try { return Boolean(chrome?.runtime?.id); } catch { return false; }
  };
  const extensionVersion = () => {
    try { return chrome?.runtime?.getManifest?.()?.version || "unknown"; } catch { return "unknown"; }
  };
  const sendRuntimeMessageQuietly = (message) => {
    try {
      if (!extensionContextActive()) return Promise.resolve(null);
      return Promise.resolve(chrome.runtime.sendMessage(message)).catch(() => null);
    } catch {
      return Promise.resolve(null);
    }
  };
  chrome.storage.local.get({ forceShortcut: "Shift", bypassShortcut: "Alt", language: "en", desktopTheme: "void" }, (value) => {
    shortcutKeys = { force: value.forceShortcut, bypass: value.bypassShortcut };
    interfaceLanguage = value.language || "en";
    interfaceTheme = value.desktopTheme || "void";
    refreshOverlayLanguages();
  });
  chrome.storage.onChanged.addListener((changes, area) => {
    if (area !== "local") return;
    if (changes.forceShortcut) shortcutKeys.force = changes.forceShortcut.newValue;
    if (changes.bypassShortcut) shortcutKeys.bypass = changes.bypassShortcut.newValue;
    if (changes.language) {
      interfaceLanguage = changes.language.newValue || "en";
      refreshOverlayLanguages();
    }
    if (changes.desktopTheme) {
      interfaceTheme = changes.desktopTheme.newValue || "void";
      refreshOverlayLanguages();
    }
  });
  let mainHookReady = false;
  const pingMainHook = () => {
    try {
      window.postMessage({
        source: "apocalipse-extension",
        type: "hook-ping",
        version: extensionVersion(),
        nonce: `${Date.now()}-${Math.random().toString(36).slice(2)}`,
      }, "*");
    } catch {}
  };
  chrome.runtime.onMessage.addListener((message, _sender, reply) => {
    if (message?.type === "APOCALIPSE_CONTENT_PING") {
      reply({
        ok: true,
        version: extensionVersion(),
        hookReady: mainHookReady,
        topFrame: window === window.top,
      });
      return;
    }
    if (message?.type === "APOCALIPSE_REQUEST_HOOK_PING") {
      pingMainHook();
      reply({ ok: true });
      return;
    }
    if (message?.type !== "APOCALIPSE_LANGUAGE_CHANGED") return;
    interfaceLanguage = message.language || "en";
    refreshOverlayLanguages();
  });
  void sendRuntimeMessageQuietly({
    type: "APOCALIPSE_CONTENT_READY",
    version: extensionVersion(),
    topFrame: window === window.top,
  });
  const modifierPressed = (event, key) => ({ Alt: event.altKey, Shift: event.shiftKey, Control: event.ctrlKey }[key] || false);
  const updateHeldShortcutKey = (event) => {
    const key = event.key === "Ctrl" ? "Control" : event.key;
    if (!["Alt", "Shift", "Control", "Insert"].includes(key)) return;
    if (event.type === "keydown") heldShortcutKeys.add(key);
    else if (event.type === "keyup") heldShortcutKeys.delete(key);
  };
  const shortcutPressed = (event, key) => modifierPressed(event, key) || heldShortcutKeys.has(key);
  const forcePressed = (event) => shortcutPressed(event, shortcutKeys.force) || shortcutPressed(event, "Insert");
  const chatgptDownloadGesture = (event) => {
    if (location.hostname.toLowerCase() !== "chatgpt.com") return false;
    const target = event.target instanceof Element ? event.target : null;
    const clickable = target?.closest?.("a[href],button,[role=button],[role=menuitem]");
    if (!clickable) return false;
    const label = String(clickable.getAttribute?.("aria-label") || clickable.innerText || clickable.textContent || "").trim();
    const href = String(clickable.getAttribute?.("href") || clickable.href || "");
    return /(?:\bdownload\b|\bbaixar\b|下载)/i.test(label)
      || /^sandbox:/i.test(href)
      || /\/backend-api\/estuary\/content(?:\?|$)/i.test(href);
  };
  const sendShortcutState = (event) => {
    updateHeldShortcutKey(event);
    return sendRuntimeMessageQuietly({
      type: "APOCALIPSE_SHORTCUT_STATE",
      bypassPressed: shortcutPressed(event, shortcutKeys.bypass),
      forcePressed: forcePressed(event),
    }).catch(() => {});
  };
  document.addEventListener("keydown", sendShortcutState, true);
  document.addEventListener("keyup", sendShortcutState, true);
  document.addEventListener("pointerdown", (event) => {
    const bypass = shortcutPressed(event, shortcutKeys.bypass);
    const force = forcePressed(event);
    if (bypass) {
      void sendRuntimeMessageQuietly({ type: "APOCALIPSE_BYPASS_NEXT", ttlMs: 4000 });
    } else if (force) {
      void sendRuntimeMessageQuietly({ type: "APOCALIPSE_FORCE_NEXT", ttlMs: 20000 });
    } else if (chatgptDownloadGesture(event)) {
      // ChatGPT can render generated-file controls without exposing the final
      // estuary URL in the clicked node. Treat that normal click as a short,
      // scoped force transaction so the MAIN-world hook can capture the final
      // authenticated file response before it becomes a navigation/download.
      void sendRuntimeMessageQuietly({ type: "APOCALIPSE_FORCE_NEXT", ttlMs: 8000 });
    }
  }, true);
  // A keyup can be missed by this frame when focus leaves the window or tab
  // (alt-tab, DevTools, another app) while a modifier is still physically
  // held. A "stuck" Alt/Shift then silently disables capture (Alt gates
  // bypass, which suppresses force capture) on pages such as claude.ai and
  // Rapidgator. Reset the held-key state whenever focus changes or the tab
  // is hidden so a stale modifier can never persist across those events.
  const clearHeldShortcutKeys = () => {
    heldShortcutKeys.clear();
    return sendRuntimeMessageQuietly({
      type: "APOCALIPSE_SHORTCUT_STATE",
      bypassPressed: false,
      forcePressed: false,
    }).catch(() => {});
  };
  window.addEventListener("blur", clearHeldShortcutKeys);
  window.addEventListener("focus", clearHeldShortcutKeys);
  document.addEventListener("visibilitychange", () => {
    if (document.hidden) clearHeldShortcutKeys();
  });
  const absolute = (value) => {
    // Missing src/poster values are not relative links: new URL("", base)
    // resolves to the page and would invent both a media URL and a thumbnail.
    if (typeof value !== "string" || !value.trim()) return null;
    try { return new URL(value, location.href).href; } catch { return null; }
  };
  const youtubeExtractorUrl = () => {
    try {
      const page = new URL(location.href);
      const host = page.hostname.toLowerCase();
      if (host === "youtu.be") {
        return /^\/[A-Za-z0-9_-]{6,}(?:\/|$)/.test(page.pathname) ? page.href : null;
      }
      if (!(host === "youtube.com" || host.endsWith(".youtube.com"))) return null;
      if (page.pathname === "/watch" && page.searchParams.get("v")) return page.href;
      if (/^\/(?:shorts|live)\/[A-Za-z0-9_-]{6,}(?:\/|$)/.test(page.pathname)) return page.href;
    } catch {}
    return null;
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
    // Bypass must keep the browser's native download untouched. The generic
    // pointerdown listener above also arms the worker lease for the ensuing
    // chrome.downloads event, which may not carry a tabId.
    if (shortcutPressed(event, shortcutKeys.bypass)) return;
    const found = chatgptLibraryLinkForEvent(event);
    if (!found) return;
    event.preventDefault();
    event.stopImmediatePropagation();
    chrome.runtime.sendMessage({
      type: "APOCALIPSE_DOWNLOAD",
      item: {
        url: found.url,
        requestUrls: [found.url],
        userAgent: navigator.userAgent,
        kind: "file",
        title: found.fileName || "chatgpt-download",
      },
    }, () => void chrome.runtime.lastError);
  };
  document.addEventListener("pointerdown", interceptChatgptLibrary, true);
  document.addEventListener("click", interceptChatgptLibrary, true);

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
  const compactMediaTitle = (value) => String(value || "").replace(/\s+/g, " ").trim();
  const normalizedMediaTitle = (value) => compactMediaTitle(value)
    .normalize("NFD").replace(/[\u0300-\u036f]/g, "").toLowerCase();
  const genericMediaTitle = (value) => {
    const title = normalizedMediaTitle(value).replace(/[._-]+/g, " ").trim();
    return !title || /^(?:video|audio|music|musica|media|midia|player|video player|audio player|media player|reprodutor(?: de)? video|reprodutor(?: de)? audio)(?: \(\d+\))?$/i.test(title);
  };
  const soundCloudSlugTitle = () => {
    try {
      const host = location.hostname.toLowerCase();
      if (!(host === "soundcloud.com" || host.endsWith(".soundcloud.com"))) return "";
      const slug = decodeURIComponent(location.pathname.split("/").filter(Boolean).at(-1) || "");
      return compactMediaTitle(slug.replace(/[-_]+/g, " "));
    } catch { return ""; }
  };
  const pageMediaTitle = () => {
    const candidates = [
      document.querySelector('meta[property="og:title"]')?.content,
      document.querySelector('meta[name="twitter:title"]')?.content,
      document.querySelector('meta[name="title"]')?.content,
      document.querySelector("h1")?.textContent,
      document.title,
      soundCloudSlugTitle(),
    ];
    for (let candidate of candidates) {
      candidate = compactMediaTitle(candidate);
      if (!candidate) continue;
      try {
        const host = location.hostname.toLowerCase();
        if (host === "soundcloud.com" || host.endsWith(".soundcloud.com")) {
          candidate = candidate
            .replace(/\s*[|–—]\s*(?:listen|stream).*?soundcloud.*$/i, "")
            .replace(/\s*[|–—-]\s*soundcloud.*$/i, "")
            .trim();
        }
      } catch {}
      if (!genericMediaTitle(candidate) && normalizedMediaTitle(candidate) !== "soundcloud") return candidate;
    }
    return "";
  };
  const titleInfoFor = (element) => {
    const labels = [
      element?.getAttribute?.("aria-label"),
      element?.title,
      element?.alt,
    ].map(compactMediaTitle).filter(Boolean);
    const elementTitle = labels.find((value) => !genericMediaTitle(value));
    if (elementTitle) return { title: elementTitle, source: "element" };
    const pageTitle = pageMediaTitle();
    if (pageTitle) return { title: pageTitle, source: soundCloudSlugTitle() === pageTitle ? "soundcloud_slug" : "page_title" };
    const fallback = labels[0] || (element?.tagName === "AUDIO" ? "audio" : "video");
    return { title: fallback, source: "generic_element" };
  };
  const titleFor = (element) => titleInfoFor(element).title;
  const facebookSponsoredEvidence = (element) => {
    if (!/(^|\.)facebook\.com$/i.test(location.hostname)) return null;
    // Sponsored filtering belongs to popup inventory, not Home overlay
    // eligibility. Facebook virtualizes/recycles feed DOM, so only trust an
    // explicit label in the same article header as the media.
    const sponsoredLabel = /^(?:Sponsored|Patrocinado|Patrocinada|Publicidad|Gesponsert|Sponsorisé|Sponsorizzato|赞助内容|贊助內容)$/iu;
    const visibleMarker = marker => {
      const rect = marker?.getBoundingClientRect?.();
      if (!rect || rect.width <= 0 || rect.height <= 0) return false;
      if (marker?.getAttribute?.("aria-hidden") === "true" || marker?.hidden) return false;
      const style = globalThis.getComputedStyle?.(marker);
      return !style || (style.display !== "none" && style.visibility !== "hidden" && Number(style.opacity || 1) > 0);
    };
    const compactLabel = value => String(value || "")
      .replace(/\s+/g, " ").trim()
      .replace(/\s*[·•|].*$/u, "").trim();
    const markerReason = marker => {
      if (!visibleMarker(marker)) return null;
      if (marker?.matches?.('[data-ad-preview],[data-testid*="sponsored" i]')) {
        return "explicit_attribute_same_article";
      }
      const aria = compactLabel(marker?.getAttribute?.("aria-label"));
      const text = compactLabel(marker?.innerText || marker?.textContent);
      return sponsoredLabel.test(aria) || sponsoredLabel.test(text)
        ? "exact_header_label_same_article" : null;
    };
    const article = element?.closest?.('article,[role="article"]');
    if (!article) return null;
    const player = element?.tagName === "VIDEO"
      ? element
      : article.querySelector?.("video");
    const playerRect = player?.getBoundingClientRect?.() || element?.getBoundingClientRect?.();
    if (!playerRect) return null;
    const markers = [
      ...(article.matches?.('[data-ad-preview],[data-testid*="sponsored" i],[aria-label]') ? [article] : []),
      ...(article.querySelectorAll?.('[data-ad-preview],[data-testid*="sponsored" i],[aria-label],span,a') || []),
    ];
    for (const marker of markers) {
      const reason = markerReason(marker);
      if (!reason) continue;
      const rect = marker.getBoundingClientRect?.();
      if (!rect) continue;
      const horizontallyAligned = rect.right >= playerRect.left - 32
        && rect.left <= playerRect.right + 32;
      const inPostHeader = rect.bottom <= playerRect.top + 18
        && rect.top >= playerRect.top - 280;
      if (horizontallyAligned && inPostHeader) return { sponsored: true, reason };
    }
    return null;
  };
  const isSponsoredFacebookPlayer = (element) => Boolean(facebookSponsoredEvidence(element));
  // Scoped to this document: closing the popup does not destroy the catalog.
  // A new document (including another site) creates a fresh isolated catalog.
  const mediaCatalog = new Map();
  const MAX_CATALOG_ITEMS = 300;
  const MAX_CATALOG_THUMBNAIL_BYTES = 6 * 1024 * 1024;
  const rememberMedia = (items) => {
    const live = new Set();
    for (const item of items) {
      if (item.visualOnly) continue; // A DOM player is not a durable file identity.
      const key = `${item.kind}:${item.url}`;
      live.add(key);
      const previous = mediaCatalog.get(key);
      mediaCatalog.set(key, { ...previous, ...item,
        thumbnail: item.thumbnail || previous?.thumbnail || "" });
    }
    while (mediaCatalog.size > MAX_CATALOG_ITEMS) mediaCatalog.delete(mediaCatalog.keys().next().value);
    let bytes = 0;
    for (const [key, item] of [...mediaCatalog.entries()].reverse()) {
      if (!item.thumbnail?.startsWith("data:")) continue;
      bytes += item.thumbnail.length;
      if (bytes > MAX_CATALOG_THUMBNAIL_BYTES) mediaCatalog.set(key, { ...item, thumbnail: "" });
    }
    return [...mediaCatalog.entries()].map(([key, item]) => live.has(key)
      ? { ...item, retained: false }
      : { ...item, retained: true, recommended: false, playerBound: false, rect: null, viewport: null })
      .concat(items.filter(item => item.visualOnly));
  };
  const playerIds = new WeakMap();
  const playerThumbnails = new WeakMap();
  let playerIdCounter = 0;
  const playerIdentity = (element) => {
    const source = String(element?.currentSrc || element?.src || "");
    const stream = element?.srcObject || null;
    const pageUrl = location.href;
    let state = playerIds.get(element);
    if (!state || state.source !== source || state.stream !== stream || state.pageUrl !== pageUrl || state.invalidated) {
      playerIdCounter += 1;
      const watching = state?.watching;
      const blockedThumbnail = state && (state.source !== source || state.stream !== stream || state.pageUrl !== pageUrl)
        ? state.lastThumbnail || state.blockedThumbnail : state?.blockedThumbnail;
      state = { id: `player-${playerIdCounter}`, source, stream, pageUrl, watching: true, blockedThumbnail };
      playerIds.set(element, state);
      playerThumbnails.delete(element);
      if (!watching) for (const event of ["emptied", "loadstart", "loadedmetadata"]) {
        element?.addEventListener?.(event, () => {
          const current = playerIds.get(element);
          if (current) current.invalidated = true;
        });
      }
    }
    return state.id;
  };
  const playerContext = (element) => {
    const rect = element?.getBoundingClientRect?.();
    if (!rect || rect.width < 80 || rect.height < 45) return {};
    const visible = rect.bottom > 0 && rect.right > 0 && rect.top < innerHeight
      && rect.left < (Number(globalThis.innerWidth) || document.documentElement?.clientWidth || rect.right);
    return {
      playerBound: true,
      playerId: playerIdentity(element),
      playerPageUrl: location.href,
      recommended: visible,
      rect: { left: rect.left, top: rect.top, width: rect.width, height: rect.height },
      viewport: {
        width: Number(globalThis.innerWidth) || document.documentElement?.clientWidth || rect.right,
        height: Number(globalThis.innerHeight) || document.documentElement?.clientHeight || rect.bottom,
      },
    };
  };
  const cssImageUrl = (value) => {
    const match = String(value || "").match(/url\(["']?([^"')]+)["']?\)/i);
    return match ? absolute(match[1]) : "";
  };
  const visualThumbnailFor = (element) => {
    if (!element?.getBoundingClientRect) return "";
    const target = element.getBoundingClientRect();
    const targetArea = Math.max(1, target.width * target.height);
    let best = null;
    let container = element.parentElement;
    for (let depth = 0; container && depth < 10; depth += 1, container = container.parentElement) {
      const videos = [...(container.querySelectorAll?.("video") || [])];
      if (videos.length > 1) break;
      const nodes = [container, ...(container.querySelectorAll?.("img") || [])];
      for (const node of nodes) {
        const rect = node.getBoundingClientRect?.();
        if (!rect || rect.width < 80 || rect.height < 45) continue;
        const overlapWidth = Math.max(0, Math.min(target.right, rect.right) - Math.max(target.left, rect.left));
        const overlapHeight = Math.max(0, Math.min(target.bottom, rect.bottom) - Math.max(target.top, rect.top));
        const overlap = overlapWidth * overlapHeight / Math.min(targetArea, Math.max(1, rect.width * rect.height));
        if (overlap < 0.45) continue;
        const source = node.tagName === "IMG"
          ? absolute(node.currentSrc || node.src || node.getAttribute?.("data-src"))
          : cssImageUrl(globalThis.getComputedStyle?.(node)?.backgroundImage);
        if (!source || !/^https?:/i.test(source)) continue;
        const score = overlap * 100 - depth * 2 + (node.tagName === "IMG" ? 5 : 0);
        if (!best || score > best.score) best = { source, score };
      }
    }
    return best?.source || "";
  };
  const pageThumbnail = (element) => {
    if (element) playerIdentity(element);
    const state = element && playerIds.get(element);
    const candidates = [
      element?.poster,
      element?.getAttribute?.("poster"),
      visualThumbnailFor(element),
    ];
    // Page-level metadata is safe only when the document has one player. On a
    // feed it commonly describes the site or the first card, not this video.
    // One visible player is not proof that page-wide artwork belongs to it:
    // virtualized feeds reuse that one element. Require an exact media URL.
    const source = absolute(element?.currentSrc || element?.src);
    const pageVideo = ["og:video", "og:video:url", "og:video:secure_url"]
      .map(name => absolute(document.querySelector(`meta[property="${name}"]`)?.content));
    const pageBound = Boolean(source && /^https?:/i.test(source) && pageVideo.includes(source)
      && document.querySelectorAll("video").length === 1);
    if (pageBound) candidates.push(
      document.querySelector('meta[property="og:image:secure_url"]')?.content,
      document.querySelector('meta[property="og:image"]')?.content,
      document.querySelector('meta[name="twitter:image"]')?.content,
      document.querySelector('meta[name="twitter:image:src"]')?.content,
      document.querySelector('link[rel="image_src"]')?.href,
    );
    for (const script of document.querySelectorAll('script[type="application/ld+json"]')) {
      if (document.querySelectorAll("video").length > 1) break;
      try {
        const data = JSON.parse(script.textContent || "null");
        const nodes = Array.isArray(data) ? data : [data];
        for (const node of nodes) {
          const value = Array.isArray(node?.thumbnailUrl) ? node.thumbnailUrl[0] : node?.thumbnailUrl;
          const type = Array.isArray(node?.["@type"]) ? node["@type"] : [node?.["@type"]];
          if (value && type.includes("VideoObject") && source
            && absolute(node?.contentUrl) === source) candidates.push(value);
        }
      } catch {}
    }
    for (const candidate of candidates) {
      const url = absolute(candidate);
      if (url && /^https?:/i.test(url) && url !== state?.blockedThumbnail) {
        if (state) state.lastThumbnail = url;
        return url;
      }
    }
    return "";
  };
  const thumbnailFor = (element, kind) => {
    if (kind === "audio") return "";
    if (element?.tagName === "IMG") return element.currentSrc || element.src || "";
    if (element) {
      const id = playerIdentity(element);
      const cached = playerThumbnails.get(element);
      if (cached?.id === id && cached.dataUrl) return cached.dataUrl;
    }
    return pageThumbnail(element);
  };
  const playerSnapshot = (element) => {
    const rect = element?.getBoundingClientRect?.();
    return { id: playerIdentity(element), source: String(element?.currentSrc || element?.src || ""),
      stream: element?.srcObject, poster: element?.poster, pageUrl: location.href,
      width: Number(globalThis.innerWidth), height: Number(globalThis.innerHeight),
      rect: rect && [rect.left, rect.top, rect.width, rect.height] };
  };
  const samePlayerSnapshot = (element, before) => {
    if (element?.isConnected === false) return false;
    const after = playerSnapshot(element);
    return before.id === after.id && before.source === after.source && before.stream === after.stream
      && before.poster === after.poster && before.pageUrl === after.pageUrl
      && before.width === after.width && before.height === after.height
      && before.rect?.every((value, index) => Math.abs(value - after.rect?.[index]) < 1);
  };
  const captureThumbnailFor = async (element, kind = "video", rejectedSource = null) => {
    const snapshot = playerSnapshot(element);
    const cached = playerThumbnails.get(element);
    if (cached?.id === snapshot.id && cached.dataUrl) return cached.dataUrl;
    const existing = thumbnailFor(element, kind);
    if (existing && existing !== rejectedSource) return existing;
    const rect = element?.getBoundingClientRect?.();
    if (!rect || rect.width < 80 || rect.height < 45 || rect.bottom <= 0 || rect.top >= innerHeight) return "";
    // Drawing the exact video is preferable to a viewport screenshot. Cross-origin
    // canvas restrictions can prevent it; those restrictions are never bypassed.
    try {
      if (element.readyState >= 2 && element.videoWidth > 0 && element.videoHeight > 0) {
        const canvas = document.createElement("canvas");
        const scale = Math.min(1, 320 / element.videoWidth, 320 / element.videoHeight);
        canvas.width = Math.max(1, Math.round(element.videoWidth * scale));
        canvas.height = Math.max(1, Math.round(element.videoHeight * scale));
        canvas.getContext("2d").drawImage(element, 0, 0, canvas.width, canvas.height);
        const dataUrl = canvas.toDataURL("image/jpeg", 0.7);
        if (samePlayerSnapshot(element, snapshot) && dataUrl.startsWith("data:image/jpeg")) {
          playerThumbnails.set(element, { id: snapshot.id, dataUrl });
          return dataUrl;
        }
      }
    } catch {}
    try {
      const result = await chrome.runtime.sendMessage({
        type: "APOCALIPSE_CAPTURE_VISIBLE_THUMBNAIL",
        rect: { left: rect.left, top: rect.top, width: rect.width, height: rect.height },
        viewport: { width: innerWidth, height: innerHeight },
      });
      if (!samePlayerSnapshot(element, snapshot)) {
        void globalThis.ADM_DIAG?.emit("thumbnail.stale_capture_rejected", { reason: "player_or_viewport_changed" });
        return "";
      }
      const dataUrl = result?.dataUrl || "";
      if (dataUrl) playerThumbnails.set(element, { id: snapshot.id, dataUrl });
      return dataUrl;
    } catch { return ""; }
  };
  const isFacebookMediaUrl = (url) => {
    try {
      const parsed = new URL(url, location.href);
      if (!/(^|\.)facebook\.com$/i.test(parsed.hostname)) return false;
      if (/\/(?:watch\/hashtag|hashtag)(?:\/|$)/i.test(parsed.pathname)) return false;
      // Facebook uses `fbid` for both photos and videos. A photo permalink must
      // stay in the Images tab; otherwise the popup offers the video player for
      // a JPEG and the desktop correctly rejects it as a non-video page.
      if (/\/(?:photo|photos)(?:\.php|\/|$)/i.test(parsed.pathname)) return false;
      return /(?:^|\/)(?:reel|reels|watch|videos|posts|share)(?:\/|$)/i.test(parsed.pathname)
        || /\/(?:permalink|story)\.php$/i.test(parsed.pathname)
        || parsed.searchParams.has("fbid")
        || parsed.searchParams.has("story_fbid");
    } catch { return false; }
  };
  const isSocialMediaPage = (url) => {
    try {
      return /(^|\.)(?:facebook|tiktok|instagram)\.com$/i.test(new URL(url, location.href).hostname);
    } catch { return false; }
  };
  const isTikTokVideoUrl = (url) => {
    try {
      const parsed = new URL(url, location.href);
      return /(^|\.)tiktok\.com$/i.test(parsed.hostname) && /\/@[^/]+\/video\/\d+/i.test(parsed.pathname);
    } catch { return false; }
  };
  const socialCardUrl = (value) => {
    const url = absolute(value);
    if (!url) return null;
    if (isFacebookMediaUrl(url) || isTikTokVideoUrl(url)) return url;
    try {
      const parsed = new URL(url);
      return /(^|\.)instagram\.com$/i.test(parsed.hostname)
        && /\/(?:reel|reels|p)\/[^/?#]+/i.test(parsed.pathname)
        ? parsed.href : null;
    } catch { return null; }
  };
  const cardThumbnailFor = (anchor) => {
    let container = anchor;
    for (let depth = 0; container && depth < 8; depth += 1, container = container.parentElement) {
      const images = [...(container.querySelectorAll?.("img") || [])]
        .map((image) => ({ image, rect: image.getBoundingClientRect?.() }))
        .filter(({ image, rect }) => rect && rect.width >= 120 && rect.height >= 90
          && /^https?:/i.test(image.currentSrc || image.src || image.getAttribute?.("data-src") || ""))
        .sort((left, right) => right.rect.width * right.rect.height - left.rect.width * left.rect.height);
      if (images[0]) return images[0].image.currentSrc || images[0].image.src || images[0].image.getAttribute("data-src") || "";
      if (container.matches?.("article,[role=article],[data-e2e*=feed-item]")) break;
    }
    return "";
  };
  const tikTokUrlFor = (element) => globalThis.ApocalipseTikTokIdentity?.resolve(element) || null;
  const facebookUrlFor = (element) => {
    if (!/(^|\.)facebook\.com$/i.test(location.hostname)) return null;
    if (isFacebookMediaUrl(location.href)) return location.href;
    const v3 = globalThis.ADM_SOCIAL_HOME_FEED_V3?.facebookDom?.(element);
    if (v3?.url && isFacebookMediaUrl(v3.url)) return v3.url;
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
  const mediaResourceIdentity = (value) => {
    if (!value || !/^https?:/i.test(value)) return "";
    try {
      const url = new URL(value, location.href);
      const query = [...url.searchParams.entries()].filter(([name]) =>
        !name.toLowerCase().match(/^(?:bytestart|byteend|range|start|end)$/));
      url.search = "";
      for (const [name, item] of query) url.searchParams.append(name, item);
      return url.href;
    } catch { return ""; }
  };
  const sameMediaResource = (left, right) => {
    const a = mediaResourceIdentity(left), b = mediaResourceIdentity(right);
    return Boolean(a && b && a === b);
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
    const youtubeUrl = youtubeExtractorUrl();
    const add = (url, kind, element, thumbnail, extra = {}) => {
      url = absolute(url);
      if (!url || !/^https?:/.test(url)) return;
      const resource = performance.getEntriesByName(url).at(-1);
      const measuredSize = Number(resource?.encodedBodySize || resource?.transferSize || 0);
      const duration = Number(element?.duration);
      const titleInfo = titleInfoFor(element);
      items.set(`${kind}:${url}`, {
        url,
        kind,
        thumbnail: thumbnail ?? thumbnailFor(element, kind),
        title: titleInfo.title,
        titleSource: titleInfo.source,
        size: measuredSize > 0 && !/\.m3u8(?:$|[?#])/i.test(url) ? measuredSize : null,
        duration: Number.isFinite(duration) && duration > 0 ? duration : null,
        ...extra,
      });
    };
    document.querySelectorAll("video").forEach((element) => {
      // YouTube uses several hidden/standby Blob players on watch, live and
      // Shorts pages. They are implementation details, not separate media.
      // The canonical page-extractor row is added once below.
      if (youtubeUrl) return;
      // Reject an explicitly sponsored Facebook card before any URL, Blob or
      // MediaStream path can turn it into a popup row.
      if (isSponsoredFacebookPlayer(element)) {
        const sponsoredUrl = facebookUrlFor(element);
        if (sponsoredUrl) mediaCatalog.delete(`video:${absolute(sponsoredUrl)}`);
        void globalThis.ADM_DIAG?.emit?.("facebook.popup_sponsored_filtered", {
          reason: facebookSponsoredEvidence(element)?.reason || "sponsored",
          retainedPurged: Boolean(sponsoredUrl),
        });
        return;
      }
      const candidateFacebookUrl = facebookUrlFor(element);
      const facebookUrl = isFacebookMediaUrl(candidateFacebookUrl) ? candidateFacebookUrl : null;
      const tikTokUrl = tikTokUrlFor(element);
      const context = playerContext(element);
      if (facebookUrl) {
        add(facebookUrl, "video", element, undefined, {
          pageExtractor: true,
          previewUrl: absolute(element.currentSrc || element.src),
          ...context,
        });
      } else if (tikTokUrl) {
        add(tikTokUrl, "video", element, undefined, {
          pageExtractor: true,
          previewUrl: absolute(element.currentSrc || element.src),
          ...context,
        });
      } else {
        const source = absolute(element.currentSrc || element.src);
        if (/^https?:/i.test(source || "")) {
          add(source, "video", element, undefined, context);
          element.querySelectorAll("source").forEach((child) => {
            if (absolute(child.src) !== source) add(child.src, "video", null, "");
          });
        } else if (source?.startsWith("blob:") || element.srcObject) {
          // A Blob/MSE player is still a real visual item. Preserve its stable
          // element identity and geometry instead of replacing it with every
          // anonymous CDN request observed in the tab.
          const identity = new URL(location.href);
          identity.hash = `apocalipse-${context.playerId || playerIdentity(element)}`;
          add(identity.href, "video", element, undefined, {
            ...context,
            visualOnly: true,
            pageExtractor: true,
            // Ordinary Facebook Reels and sponsored cards can both use
            // srcObject. Do not turn that implementation detail into an ad
            // classification or valid Reels disappear from the popup.
            recordingOnly: Boolean(element.srcObject && isSponsoredFacebookPlayer(element)),
          });
        }
      }
    });
    // Social feeds commonly expose the permalink and cover image before they
    // create a <video>. Treat that card as a video candidate so its thumbnail
    // is available without starting playback.
    document.querySelectorAll("a[href]").forEach((anchor) => {
      if (isSponsoredFacebookPlayer(anchor)) {
        const sponsoredUrl = socialCardUrl(anchor.href);
        if (sponsoredUrl) mediaCatalog.delete(`video:${absolute(sponsoredUrl)}`);
        return;
      }
      const url = socialCardUrl(anchor.href);
      if (!url || items.has(`video:${url}`)) return;
      const thumbnail = cardThumbnailFor(anchor);
      if (!thumbnail) return;
      const rect = anchor.getBoundingClientRect?.();
      add(url, "video", anchor, thumbnail, {
        pageExtractor: true,
        recommended: Boolean(rect && rect.width > 0 && rect.height > 0 && rect.bottom > 0 && rect.top < innerHeight),
        title: anchor.getAttribute("aria-label") || anchor.closest?.("article,[role=article]")?.innerText?.trim()?.slice(0, 240) || document.title,
      });
    });
    const facebookPageUrl = facebookUrlFor(document.querySelector("video"));
    if (facebookPageUrl && !items.has(`video:${facebookPageUrl}`)) {
      const video = document.querySelector("video");
      add(facebookPageUrl, "video", video, undefined, {
        pageExtractor: true,
        previewUrl: absolute(video?.currentSrc || video?.src),
        recommended: true,
      });
    }
    if (youtubeUrl) {
      const parsed = new URL(youtubeUrl);
      const videoId = parsed.searchParams.get("v") || parsed.pathname.match(/^\/(?:shorts|live)\/([^/]+)/)?.[1] || (parsed.hostname === "youtu.be" ? parsed.pathname.split("/")[1] : null);
      const videos = [...document.querySelectorAll("video")];
      const video = videos.sort((left, right) => {
        const a = left.getBoundingClientRect?.() || { width: 0, height: 0 };
        const b = right.getBoundingClientRect?.() || { width: 0, height: 0 };
        return (b.width * b.height) - (a.width * a.height);
      })[0] || null;
      const thumbnail = videoId
        ? `https://i.ytimg.com/vi/${videoId}/hqdefault.jpg`
        : (document.querySelector('meta[property="og:image"]')?.content || "");
      add(youtubeUrl, "video", video, thumbnail, {
        ...playerContext(video),
        pageExtractor: true,
        recommended: true,
      });
    }
    document.querySelectorAll("audio").forEach((element) => {
      add(element.currentSrc || element.src, "audio", element);
      element.querySelectorAll("source").forEach((source) => add(source.src, "audio", element));
    });
    document.querySelectorAll("img").forEach((element) => add(element.currentSrc || element.src, "image", element));
    performance.getEntriesByType("resource").forEach((entry) => {
      if (/\.m3u8(?:$|[?#])/i.test(entry.name) && !items.has(`video:${entry.name}`)) {
        add(entry.name, "video", null, "");
      }
    });
    const collected = [...items.values()];
    return rememberMedia(collected);
  };
  const downloadLabel = () => {
    const value = String(interfaceLanguage || "en").toLowerCase();
    return value.startsWith("zh") ? "下载" : value.startsWith("pt") ? "Baixar" : "Download";
  };
  const recordingLabels = () => {
    const value = String(interfaceLanguage || "en").toLowerCase();
    if (value.startsWith("zh")) return { record: "● 录制", stop: "■ 停止并保存", uploading: "正在发送…", done: "已保存", unavailable: "此视频无法由浏览器录制" };
    if (value.startsWith("pt")) return { record: "● Gravar", stop: "■ Parar e salvar", uploading: "Enviando…", done: "Gravação salva", unavailable: "Este vídeo não permite gravação pelo navegador" };
    return { record: "● Record", stop: "■ Stop and save", uploading: "Uploading…", done: "Recording saved", unavailable: "This video cannot be recorded by the browser" };
  };
  const overlayThemeColors = () => {
    const accents = { inferno:"#ff6a20",toxic:"#82f23d",synthwave:"#ff4fca",royal:"#9c83ff",crimson:"#ff4775",arctic:"#65e8ff",obsidian:"#59e3d1",monochrome:"#f1f1f1",midnight:"#4e9dff",forest:"#55cf8a",graphite:"#b3c7d6",deepsea:"#22c7c9",eclipse:"#b86cff",hazard:"#fca311",cyberstorm:"#94d2bd",ultraviolet:"#d05cff",emeraldgold:"#51e79b",scarletice:"#ff496c",coppernavy:"#f4a261",solarizednight:"#e7b84b",pearlblue:"#258fd1",whiteaurora:"#18a7b8",goldenivory:"#d69524",crystalrose:"#d75c91",polarmint:"#2da875",void:"#25d9ef" };
    return accents[interfaceTheme] || accents.void;
  };
  const clockLabel = (seconds) => {
    const value = Math.max(0, Math.floor(seconds));
    return `${String(Math.floor(value / 60)).padStart(2, "0")}:${String(value % 60).padStart(2, "0")}`;
  };
  const traceDiagnostic = (eventName, mode, detail = {}, actionId = null) => {
    const level = /failed|error/.test(eventName) ? "ERROR" : /unresolved|rejected|changed/.test(eventName) ? "WARN" : "INFO";
    void globalThis.ADM_DIAG?.emit(eventName, { mode, ...detail }, actionId, level);
    return chrome.runtime.sendMessage({
    type: "APOCALIPSE_CAPTURE_TRACE",
    eventName,
    mode,
    traceId: actionId || crypto.randomUUID(),
    pageUrl: location.href,
    at: Date.now(),
    detail,
  }).catch(() => {});
  };
  const trace = traceDiagnostic;
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
    if (youtubeExtractorUrl()) return false;
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
          duration: Number.isFinite(video.duration) ? video.duration : null,
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
    const bypass = shortcutPressed(event, shortcutKeys.bypass);
    const force = shortcutPressed(event, shortcutKeys.force);
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
    if (element.tagName === "VIDEO" && youtubeExtractorUrl()) return youtubeExtractorUrl();
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
    if (youtubeExtractorUrl()) return immediate;
    const hls = hlsForPage();
    if (!hls.candidates.length) return immediate;
    try {
      const selected = await chrome.runtime.sendMessage({
        type: "APOCALIPSE_SELECT_HLS",
        urls: hls.candidates,
        duration: Number.isFinite(element.duration) ? element.duration : null,
      });
      return selected || { url: hls.fallback || immediate, duration: null, requestUrls: hls.candidates };
    } catch { return { url: hls.fallback || immediate, duration: null, requestUrls: hls.candidates }; }
  };
  let overlayTimer;
  const activeOverlays = new Map();

  // Universal social-player diagnostics. Decisions are emitted only while the
  // existing opt-in diagnostics session is active. A per-player signature
  // suppresses identical repeats so long feeds remain readable.
  const socialDecisionCache = new WeakMap();
  let socialScanId = 0;
  const socialPlatform = () => {
    const host = String(location.hostname || "").toLowerCase();
    if (/(^|\.)facebook\.com$/.test(host)) return "facebook";
    if (/(^|\.)instagram\.com$/.test(host)) return "instagram";
    if (/(^|\.)tiktok\.com$/.test(host)) return "tiktok";
    if (/(^|\.)(?:x|twitter)\.com$/.test(host)) return "x";
    return "generic";
  };
  const socialPlayerVisible = element => {
    const rect = element?.getBoundingClientRect?.();
    return Boolean(rect && rect.width >= 100 && rect.height >= 55
      && rect.bottom > 0 && rect.right > 0 && rect.top < innerHeight
      && rect.left < (Number(globalThis.innerWidth) || document.documentElement?.clientWidth || rect.right));
  };
  const socialSourceType = element => {
    const source = String(element?.currentSrc || element?.src || "");
    if (element?.srcObject) return "srcObject";
    if (/^blob:/i.test(source)) return "blob";
    if (/^https?:/i.test(source)) return "http";
    return source ? "other" : "empty";
  };
  const emitSocialDecision = (element, decision, detail = {}, force = false) => {
    if (!element || element.tagName !== "VIDEO" || !globalThis.ADM_DIAG?.active?.()) return;
    const platform = socialPlatform();
    if (platform === "generic") return;
    const player = globalThis.ADM_DIAG?.player?.(element) || {};
    const rect = element.getBoundingClientRect?.() || {};
    const payload = {
      platform,
      scanId: socialScanId,
      decision,
      playerId: player.playerId || playerIdentity(element),
      revision: player.revision ?? null,
      visible: socialPlayerVisible(element),
      sourceType: socialSourceType(element),
      duration: Number.isFinite(element.duration) ? Number(element.duration.toFixed(3)) : null,
      readyState: element.readyState ?? null,
      paused: Boolean(element.paused),
      muted: Boolean(element.muted),
      volume: Number.isFinite(element.volume) ? element.volume : null,
      hasSrcObject: Boolean(element.srcObject),
      rect: {
        left: Math.round(rect.left || 0), top: Math.round(rect.top || 0),
        width: Math.round(rect.width || 0), height: Math.round(rect.height || 0),
      },
      datasetButton: Boolean(element.dataset?.apocalipseButton),
      overlayActive: activeOverlays.has(element),
      ...detail,
    };
    const signature = JSON.stringify({ ...payload, scanId: 0 });
    if (!force && socialDecisionCache.get(element) === signature) return;
    socialDecisionCache.set(element, signature);
    void globalThis.ADM_DIAG.emit("social.player_decision", payload, null,
      decision === "missing" ? "WARN" : "INFO");
    if (decision === "missing") {
      void globalThis.ADM_DIAG.emit("social.overlay_missing", payload, null, "WARN");
    }
  };
  const emitSocialSummary = summary => {
    if (!globalThis.ADM_DIAG?.active?.() || socialPlatform() === "generic") return;
    void globalThis.ADM_DIAG.emit("social.scan_summary", {
      platform: socialPlatform(), scanId: socialScanId, ...summary,
    });
  };

  const installOverlays = () => {
    if (/(^|\.)chatgpt\.com$/.test(location.hostname)) return;
    socialScanId += 1;
    const socialSummary = { players: 0, visible: 0, eligible: 0, overlays: 0, missing: 0,
      sponsored: 0, audioOnly: 0, inactive: 0, noAction: 0, kept: 0, installed: 0 };
    if (youtubeExtractorUrl()) {
      document.querySelectorAll(".apocalipse-media-record").forEach((button) => button.remove());
    }
    for (const overlay of [...activeOverlays.values()]) {
      if (!overlay.element.isConnected || overlay.pageUrl !== location.href) {
        overlay.cleanup();
        continue;
      }
      const facebookOverlay = overlay.element?.tagName === "VIDEO"
        && /(^|\.)facebook\.com$/i.test(location.hostname);
      if (!facebookOverlay) continue;
      const currentBindingId = playerIdentity(overlay.element);
      if (overlay.bindingId && overlay.bindingId !== currentBindingId) {
        trace("facebook.video_reused", "overlay", {
          previousBindingId: overlay.bindingId,
          currentBindingId,
          source: String(overlay.element.currentSrc || overlay.element.src || ""),
        });
        overlay.cleanup();
      }
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
      const facebookPage = /(^|\.)facebook\.com$/i.test(location.hostname);
      const isSocialVideo = element.tagName === "VIDEO" && socialPlatform() !== "generic";
      if (isSocialVideo) {
        socialSummary.players += 1;
        if (socialPlayerVisible(element)) socialSummary.visible += 1;
      }
      if (facebookPage && element.tagName === "AUDIO") {
        socialSummary.audioOnly += 1;
        trace("facebook.overlay_skipped_audio_only", "overlay", { reason: "audio_element" });
        return;
      }
      const isFacebookVideo = element.tagName === "VIDEO" && facebookPage;
      if (isFacebookVideo && isSponsoredFacebookPlayer(element)) {
        socialSummary.sponsored += 1;
        const evidence = facebookSponsoredEvidence(element);
        emitSocialDecision(element, "allow_sponsored_home", {
          reason: evidence?.reason || "sponsored_home_allowed",
          sponsored: true,
          permalinkFound: Boolean(facebookUrlFor(element)),
        });
        trace("facebook.sponsored_home_allowed", "overlay", {
          reason: evidence?.reason || "sponsored_home_allowed",
        });
      }
      if (element.dataset.apocalipseButton && !activeOverlays.has(element)) {
        delete element.dataset.apocalipseButton;
      }
      if (element.dataset.apocalipseButton) {
        if (isSocialVideo) {
          socialSummary.eligible += 1; socialSummary.overlays += 1; socialSummary.kept += 1;
          emitSocialDecision(element, "keep", { reason: "overlay_already_active", sponsored: false });
        }
        return;
      }
      const youtubeUrl = element.tagName === "VIDEO" ? youtubeExtractorUrl() : null;
      const isYouTubeVideo = Boolean(youtubeUrl);
      // Extractor-first pages already have a complete, higher-quality download
      // route. Recording would only duplicate yt-dlp with a less reliable path.
      const usesExtractorOnlyDownload = isYouTubeVideo;
      if (isFacebookReelsPage && isFacebookVideo && element !== activeFacebookReel) {
        socialSummary.inactive += 1;
        emitSocialDecision(element, "skip_inactive_player", { reason: "not_active_facebook_reel", sponsored: false });
        return;
      }
      if (isInstagramReelsPage && element.tagName === "VIDEO" && element !== activeInstagramReel) {
        socialSummary.inactive += 1;
        emitSocialDecision(element, "skip_inactive_player", { reason: "not_active_instagram_reel" });
        return;
      }
      if (isTikTokPage && element.tagName === "VIDEO" && element !== activeTikTokVideo) {
        socialSummary.inactive += 1;
        emitSocialDecision(element, "skip_inactive_player", { reason: "not_active_tiktok_player" });
        return;
      }
      const tikTokUrl = element.tagName === "VIDEO" ? tikTokUrlFor(element) : null;
      const isTikTokVideo = Boolean(tikTokUrl);
      const url = isFacebookVideo ? facebookUrlFor(element) || location.href : tikTokUrl || downloadUrlFor(element);
      const liveMediaUrl = element.currentSrc || element.src || "";
      const hasDirectHttpMedia = /^https?:/i.test(liveMediaUrl);
      const downloadReady = () => Boolean((downloadUrlFor(element) && /^https?:/.test(downloadUrlFor(element)))
        || /^blob:/i.test(String(element.currentSrc || element.src || '')));
      const canDownload = Boolean((url && /^https?:/.test(url)) || /^blob:/i.test(liveMediaUrl));
      // A direct HTTP media URL belongs in the download path. Cross-origin
      // players commonly reject captureStream(), so displaying Record there
      // offers an action that cannot succeed (and duplicates Download).
      // Facebook Home frequently exposes a direct HTTP video track even when
      // audio is separate or the durable post identity is still unresolved.
      // Keep Record available there and let Download fall back to recording
      // when no complete Facebook resource can be proven.
      const canRecord = !isYouTubeVideo && element.tagName === "VIDEO" && Boolean(globalThis.MediaRecorder)
        && Boolean(element.captureStream || element.webkitCaptureStream)
        && (!hasDirectHttpMedia || isFacebookVideo);
      if (!canDownload && !canRecord) {
        if (isSocialVideo) {
          socialSummary.noAction += 1;
          if (socialPlayerVisible(element)) {
            socialSummary.missing += 1;
            emitSocialDecision(element, "missing", {
              reason: "no_supported_action", sponsored: false,
              canDownload, canRecord, hasDirectHttpMedia,
              permalinkFound: Boolean(isFacebookVideo ? facebookUrlFor(element) : tikTokUrl),
              downloadCandidate: Boolean(url),
            }, true);
          } else {
            emitSocialDecision(element, "skip_no_action", {
              reason: "no_supported_action_offscreen", sponsored: false,
              canDownload, canRecord, hasDirectHttpMedia,
            });
          }
        }
        return;
      }
      if (isSocialVideo) socialSummary.eligible += 1;
      element.dataset.apocalipseButton = "1";
      const button = document.createElement("button");
      globalThis.ApocalipseTikTokIdentity?.bind(button, element);
      button.type = "button";
      button.className = "apocalipse-media-download";
      button.textContent = `⇩ ${downloadLabel()}`;
      button.title = "Apocalipse Download Manager";
      button.hidden = !canDownload || Boolean(isFacebookVideo && facebookSponsoredEvidence(element));
      let recordButton = null;
      let refreshRecordLabels = () => {};
      button.addEventListener("click", async (event) => {
        const actionId = globalThis.ADM_DIAG?.begin("overlay.click", { ...globalThis.ADM_DIAG.player(element), handler: "generic" }) || crypto.randomUUID();
        const trace = (name, mode, detail = {}) => traceDiagnostic(name, mode, detail, actionId);
        event.preventDefault();
        event.stopPropagation();
        const restoreDownloadLabel = () => { button.textContent = `⇩ ${downloadLabel()}`; };
        const clickSource = String(element.currentSrc || element.src || "");
        const clickPage = location.href;
        const socialVideo = isFacebookVideo || (isTikTokPage && element.tagName === "VIDEO");
        button.textContent = "…";
        trace("overlay_download_clicked", "download", { tag: element.tagName, facebook: isFacebookVideo, tiktokPage: isTikTokPage, tiktokPermalink: isTikTokVideo, overlays: activeOverlays.size });
        const visibleFacebookUrl = isFacebookVideo && isFacebookMediaUrl(location.href) ? location.href : null;
        const immediateFacebookUrl = isFacebookVideo ? facebookUrlFor(element) : null;
        // Normal Facebook Reels must attempt permalink/extractor resolution
        // even when the visible player is backed by srcObject. Sponsored posts
        // never reach this handler because their Download button is hidden.
        const resolved = visibleFacebookUrl || immediateFacebookUrl
          || (isFacebookVideo ? await revealFacebookUrl(element) : tikTokUrlFor(element) || await resolveDownloadUrl(element));
        // The Reel/post permalink represents the complete video. A recent CDN
        // response can be only one DASH track (even when labelled video/mp4).
        // Use the same page-extractor route as the popup, before blobs or CDN URLs.
        const facebookPageUrl = isFacebookVideo
          ? [typeof resolved === "string" ? resolved : resolved?.url, visibleFacebookUrl]
            .find((value) => value && isFacebookMediaUrl(value)) || null
          : null;
        const tikTokPageUrl = isTikTokPage
          ? [typeof resolved === "string" ? resolved : resolved?.url, tikTokUrlFor(element)]
            .find((value) => value && isTikTokVideoUrl(value)) || null
          : null;
        const socialPageUrl = facebookPageUrl || tikTokPageUrl;
        const liveSource = String(element.currentSrc || element.src || "");
        const liveBlobUrl = /^blob:/i.test(liveSource) ? liveSource : null;
        const liveHttpUrl = /^https?:/i.test(liveSource) ? liveSource : null;
        const networkMediaUrl = socialVideo ? null : recentNetworkMediaUrl();
        const capturedSocialMedia = socialVideo && !socialPageUrl
          ? await chrome.runtime.sendMessage({ type: "APOCALIPSE_RECENT_TAB_MEDIA" }).catch(() => null)
          : null;
        const inspectedSocialMedia = capturedSocialMedia?.media?.length
          ? await chrome.runtime.sendMessage({ type: "APOCALIPSE_INSPECT_MEDIA_TRACKS", media: capturedSocialMedia.media }).catch(() => null)
          : null;
        const socialTrackInfo = new Map((inspectedSocialMedia?.media || []).map((item) => [item.url, item]));
        if (socialVideo && (!element.isConnected || clickSource !== String(element.currentSrc || element.src || "")
          || clickPage !== location.href)) {
          trace("overlay_download_player_changed", "download", { facebook: isFacebookVideo, tiktokPage: isTikTokPage });
          button.textContent = "!";
          button.title = "O vídeo mudou. Clique novamente no vídeo atual.";
          setTimeout(restoreDownloadLabel, 2500);
          return;
        }
        // Bind the overlay to this exact player. Never use an arbitrary recent
        // response from the tab: feeds may keep several videos buffered.
        let browserVideoItem = capturedSocialMedia?.media?.find((item) =>
          /^https?:/i.test(item.url || "")
          && !/^audio\//i.test(item.contentType || "")
          && sameMediaResource(item.url, liveHttpUrl)) || null;
        if (!browserVideoItem && (liveBlobUrl || element.srcObject) && Number.isFinite(element.duration) && element.duration > 0) {
          const durationMatches = (capturedSocialMedia?.media || [])
            .map((item) => ({ item, info: socialTrackInfo.get(item.url) }))
            .filter(({ info }) => (info?.kind === "video" || info?.kind === "muxed") && Number.isFinite(info.duration))
            .map((entry) => ({ ...entry, delta: Math.abs(entry.info.duration - element.duration) }))
            .filter(({ delta }) => delta <= 1.5)
            .sort((left, right) => left.delta - right.delta || (right.item.capturedAt || 0) - (left.item.capturedAt || 0));
          if (durationMatches[0] && !(durationMatches[1] && Math.abs(durationMatches[0].delta - durationMatches[1].delta) < 0.05)) {
            browserVideoItem = durationMatches[0].item;
          }
        }
        const browserAudioCandidates = (capturedSocialMedia?.media || [])
          .filter((item) => /^https?:/i.test(item.url || "")
            && (socialTrackInfo.get(item.url)?.kind === "audio" || /^audio\//i.test(item.contentType || ""))
            && Number.isFinite(item.capturedAt) && item.capturedAt > 0
            && Number.isFinite(browserVideoItem?.capturedAt) && browserVideoItem.capturedAt > 0
            && (item.frameId === browserVideoItem.frameId || (item.frameId == null && browserVideoItem.frameId == null)))
          .filter((audio) => !(capturedSocialMedia?.media || []).some((other) =>
            (socialTrackInfo.get(other.url)?.kind === "video" || socialTrackInfo.get(other.url)?.kind === "muxed"
              || (!socialTrackInfo.get(other.url)?.kind && /^video\//i.test(other.contentType || "")))
            && Number.isFinite(other.capturedAt) && other.capturedAt > 0
            && (other.frameId === audio.frameId || (other.frameId == null && audio.frameId == null))
            && !sameMediaResource(other.url, browserVideoItem.url)
            && Math.abs(other.capturedAt - audio.capturedAt) <= Math.abs(browserVideoItem.capturedAt - audio.capturedAt)))
          .sort((left, right) => Math.abs((left.capturedAt || 0) - (browserVideoItem?.capturedAt || 0))
            - Math.abs((right.capturedAt || 0) - (browserVideoItem?.capturedAt || 0)));
        const closestAudioItem = browserAudioCandidates[0] || null;
        const browserAudioItem = closestAudioItem
          && browserVideoItem
          && !(browserAudioCandidates[1] && Math.abs(browserAudioCandidates[1].capturedAt - browserVideoItem.capturedAt)
            === Math.abs(closestAudioItem.capturedAt - browserVideoItem.capturedAt))
          && Math.abs((closestAudioItem.capturedAt || 0) - (browserVideoItem.capturedAt || 0)) <= 8_000
          ? closestAudioItem
          : null;
        const browserVideoMedia = browserVideoItem?.url || null;
        const browserAudioMedia = socialTrackInfo.get(browserVideoItem?.url)?.kind === "muxed" ? null : browserAudioItem?.url || null;

        // For a real <video>, the source feeding the player is more authoritative
        // than location.href. Try readable blob first; for MSE blobs, fall through
        // to the most recent underlying media request captured by Performance API.
        if (liveBlobUrl && !socialVideo && !isYouTubeVideo) {
          try {
            const fallbackName = isFacebookVideo ? facebookDownloadTitle(location.href) : null;
            await uploadBlobUrl(liveBlobUrl, fallbackName);
            button.textContent = "✓";
            button.title = "Enviado ao Apocalipse";
            setTimeout(restoreDownloadLabel, 1500);
            return;
          } catch (error) {
            console.debug("Apocalipse live blob is MSE/unreadable; trying network media", error);
          }
        }

        // A resolved HLS manifest is more authoritative than incidental network
        // traffic or a generic HTTP source exposed by the player.
        let currentUrl = socialVideo
          ? (socialPageUrl || browserVideoMedia || liveHttpUrl)
          : (resolved?.url || resolved || liveHttpUrl || networkMediaUrl || (isYouTubeVideo ? location.href : null));
        const facebookPlayableUrl = isFacebookVideo && currentUrl && (
          isFacebookMediaUrl(currentUrl)
          || /\.(?:mp4|webm|m3u8|mpd)(?:[?#]|$)/i.test(currentUrl)
          || (/(?:fbcdn|fbsbx|video)/i.test(currentUrl)
            && !/\.(?:avif|bmp|gif|ico|jpe?g|png|svg|webp)(?:[?#]|$)/i.test(currentUrl))
        );
        if (!currentUrl || (isFacebookVideo && !facebookPlayableUrl)) {
          // Facebook Home players may expose only a MediaStream/blob or an
          // incomplete DASH track. Preserve the historical behavior: clicking
          // Download automatically starts recording when no complete resource
          // can be proven for this exact player.
          if (isFacebookVideo && canRecord && recordButton) {
            trace("overlay_download_stream_capture", "download", {
              reason: element.srcObject ? "facebook_srcobject_without_complete_resource" : "facebook_unresolved_recording_fallback",
              duration: Number.isFinite(element.duration) ? element.duration : null,
            });
            button.textContent = "●";
            button.title = recordingLabels().record;
            recordButton.click();
            setTimeout(() => { restoreDownloadLabel(); button.title = "Apocalipse Download Manager"; }, 1800);
            return;
          }
          trace("overlay_download_unresolved", "download", { liveBlob: Boolean(liveBlobUrl), liveHttp: Boolean(liveHttpUrl), networkMedia: Boolean(networkMediaUrl), facebook: isFacebookVideo });
          button.textContent = "⚠";
          button.title = "Abra o vídeo ou use os três pontos e Copiar link";
          setTimeout(restoreDownloadLabel, 2500);
          return;
        }
        if (/^blob:/i.test(String(currentUrl || ""))) {
          try {
            await uploadBlobUrl(currentUrl);
            button.textContent = "✓";
            button.title = "Enviado ao Apocalipse";
            setTimeout(restoreDownloadLabel, 1500);
            return;
          } catch (error) {
            console.debug("Apocalipse direct blob failed", error);
          }
        }
        const directFacebookMedia = isFacebookVideo ? absolute(element.currentSrc || element.src) : null;
        // Do not attach unrelated CDN audio to an extractor page task.
        const extractorPageSelected = Boolean(socialPageUrl);
        const companionAudioUrl = !extractorPageSelected && socialVideo ? browserAudioMedia : null;
        const ambiguousSocialTrack = Boolean(socialVideo && !extractorPageSelected && !companionAudioUrl
          && !/\.(?:m3u8|mpd)(?:$|[?#])/i.test(currentUrl));
        const requestUrls = extractorPageSelected ? [] : socialVideo ? [currentUrl] : [...new Set([
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
          browserAudioCaptured: Boolean(browserAudioMedia),
          browserAudioAgeMs: browserAudioItem?.ageMs ?? -1,
          browserAudioType: browserAudioItem?.contentType || "none",
          browserAudioCandidates: browserAudioCandidates.length,
          browserAudioDeltaMs: browserAudioItem && browserVideoItem
            ? Math.abs((browserAudioItem.capturedAt || 0) - (browserVideoItem.capturedAt || 0))
            : -1,
          directPlayer: Boolean(liveHttpUrl && currentUrl === liveHttpUrl),
          networkMedia: Boolean(networkMediaUrl && currentUrl === networkMediaUrl),
          pageFallback: Boolean(isFacebookVideo && isFacebookMediaUrl(currentUrl)),
          facebookPageExtractorPreferred: Boolean(facebookPageUrl),
          candidate: currentUrl,
        });
        const thumbnail = await captureThumbnailFor(element, "video");
        chrome.runtime.sendMessage({ type: "APOCALIPSE_DOWNLOAD", item: { traceId: actionId, url: currentUrl, audioUrl: companionAudioUrl, ambiguousSocialTrack, duration: resolved?.duration || null, requestUrls: [...requestUrls, ...(companionAudioUrl ? [companionAudioUrl] : [])], userAgent: navigator.userAgent, kind: resolved?.mediaKind || element.tagName.toLowerCase(), title: facebookPageUrl ? titleFor(element) : (isFacebookVideo ? facebookDownloadTitle(currentUrl) : document.title), thumbnail } }, (result) => {
          const failed = chrome.runtime.lastError || !result?.ok;
          trace(failed ? "overlay_download_failed" : "overlay_download_handed_off", "download", { target: result?.target || "none", error: result?.error || chrome.runtime.lastError?.message || "none", candidates: requestUrls.length });
          button.textContent = failed ? "⚠" : "✓";
          if (failed) button.title = result?.error || chrome.runtime.lastError?.message || "Apocalipse unavailable";
          setTimeout(restoreDownloadLabel, 1500);
        });
      });
      if (element.tagName === "VIDEO" && canRecord && !usesExtractorOnlyDownload) {
        const record = document.createElement("button");
        recordButton = record;
        try { button[Symbol.for("apocalipse.recordButton")] = record; } catch {}
        record.type = "button";
        record.className = "apocalipse-media-download apocalipse-media-record";
        record.textContent = recordingLabels().record;
        record.title = recordingLabels().record;
        let recorder = null;
        let recordPhase = "idle";
        let previousLoop = false;
        let startedAt = 0;
        let clockTimer = null;
        let stopPoll = null;
        let playbackWatch = null;
        refreshRecordLabels = () => {
          const labels = recordingLabels();
          if (recordPhase === "recording") record.textContent = `${labels.stop} · ${clockLabel((Date.now() - startedAt) / 1000)}`;
          else if (recordPhase === "uploading") record.textContent = labels.uploading;
          else if (recordPhase === "done") record.textContent = `✓ ${labels.done}`;
          else if (recordPhase === "error") record.textContent = "⚠";
          else record.textContent = labels.record;
          record.title = recordPhase === "error" ? labels.unavailable : labels.record;
        };
        record.addEventListener("click", async (event) => {
          event.preventDefault();
          event.stopPropagation();
          if (recorder && recorder.state !== "inactive") {
            trace("recording_stop_clicked", "record", { elapsedMs: Date.now() - startedAt });
            recorder.stop();
            record.disabled = true;
            recordPhase = "uploading";
            refreshRecordLabels();
            return;
          }
          try {
            trace("recording_start_clicked", "record", { captureStream: Boolean(element.captureStream || element.webkitCaptureStream), mediaRecorder: Boolean(globalThis.MediaRecorder) });
            const capture = element.captureStream?.bind(element) || element.webkitCaptureStream?.bind(element);
            if (!capture || !globalThis.MediaRecorder) throw new Error("capture_not_supported");
            if (Number.isFinite(element.duration)) element.currentTime = 0;
            previousLoop = element.loop;
            element.loop = false;
            await element.play();
            const stream = capture();
            for (let attempt = 0; attempt < 20 && !stream.getTracks().length; attempt += 1) {
              await new Promise(resolve => setTimeout(resolve, 100));
            }
            if (!stream.getTracks().length) throw new Error("capture_stream_has_no_tracks");
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
            let pausedAtMediaTime = null;
            const syncRecorderWithPlayer = () => {
              if (!recorder || recorder.state === "inactive") return;
              if (element.paused && recorder.state === "recording") {
                recorder.requestData?.();
                recorder.pause();
                pausedAtMediaTime = Number(element.currentTime) || 0;
                trace("recording_paused_with_player", "record", { currentTime: element.currentTime });
              }
            };
            const resumeRecorderOnRealProgress = () => {
              if (!recorder || recorder.state !== "paused" || element.paused) return;
              const currentTime = Number(element.currentTime);
              if (!Number.isFinite(currentTime) || pausedAtMediaTime == null || currentTime <= pausedAtMediaTime + 0.04) return;
              try {
                recorder.resume();
                pausedAtMediaTime = null;
                trace("recording_resumed_with_player", "record", { currentTime: element.currentTime });
              } catch (error) {
                trace("recording_resume_failed", "record", { currentTime, error: String(error) });
              }
            };
            element.addEventListener("pause", syncRecorderWithPlayer);
            element.addEventListener("timeupdate", resumeRecorderOnRealProgress);
            stream.getTracks().forEach((track) => track.addEventListener("ended", () => {
              trace("recording_track_ended", "record", { kind: track.kind, readyState: track.readyState });
            }, { once: true }));
            recorder.ondataavailable = ({ data }) => {
              if (!data.size || uploadError) return;
              uploadQueue = uploadQueue.then(() => appendBlob(begin.uploadId, data)).catch((error) => { uploadError = error; });
            };
            recorder.onstop = async () => {
              if (clockTimer) clearInterval(clockTimer);
              if (stopPoll) clearInterval(stopPoll);
              if (playbackWatch) clearInterval(playbackWatch);
              element.removeEventListener("pause", syncRecorderWithPlayer);
              element.removeEventListener("timeupdate", resumeRecorderOnRealProgress);
              stream.getTracks().forEach((track) => track.stop());
              element.loop = previousLoop;
              recordPhase = "uploading";
              refreshRecordLabels();
              try {
                await uploadQueue;
                if (uploadError) throw uploadError;
                const result = await chrome.runtime.sendMessage({ type: "APOCALIPSE_BLOB_END", request: { uploadId: begin.uploadId } });
                if (result?.error) throw new Error(result.error);
                recordPhase = "done";
                refreshRecordLabels();
                trace("recording_completed", "record", { uploadId: begin.uploadId, elapsedMs: Date.now() - startedAt });
              } catch (error) {
                trace("recording_upload_failed", "record", { uploadId: begin.uploadId, error: String(error) });
                console.error("Apocalipse recorder upload", error);
                recordPhase = "error";
                refreshRecordLabels();
              } finally {
                recorder = null;
                setTimeout(() => { record.disabled = false; recordPhase = "idle"; refreshRecordLabels(); }, 2200);
              }
            };
            element.addEventListener("ended", () => {
              if (recorder && recorder.state !== "inactive") recorder.stop();
            }, { once: true });
            recorder.start(1000);
            startedAt = Date.now();
            recordPhase = "recording";
            refreshRecordLabels();
            clockTimer = setInterval(() => {
              if (recorder && recorder.state !== "inactive") refreshRecordLabels();
            }, 1000);
            playbackWatch = setInterval(() => {
              if (!recorder || recorder.state === "inactive") return;
              const duration = Number(element.duration);
              const currentTime = Number(element.currentTime);
              if (Number.isFinite(duration) && duration > 0 && Number.isFinite(currentTime) && currentTime >= duration - 0.25) {
                trace("recording_reached_media_end", "record", { currentTime, duration });
                recorder.stop();
                return;
              }
              resumeRecorderOnRealProgress();
            }, 750);
            stopPoll = setInterval(async () => {
              if (!recorder || recorder.state === "inactive") return;
              const status = await chrome.runtime.sendMessage({ type: "APOCALIPSE_BLOB_STATUS", request: { uploadId: begin.uploadId } }).catch(() => null);
              if (status?.stop && recorder && recorder.state !== "inactive") recorder.stop();
            }, 1000);
          } catch (error) {
            trace("recording_start_failed", "record", { error: String(error) });
            console.error("Apocalipse recorder", error);
            recordPhase = "error";
            refreshRecordLabels();
            setTimeout(() => { recordPhase = "idle"; refreshRecordLabels(); }, 2000);
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
        const liveCanDownload = canDownload || downloadReady();
        const sponsoredHomeVideo = Boolean(isFacebookVideo && facebookSponsoredEvidence(element));
        button.hidden = sponsoredHomeVideo || !liveCanDownload || rect.width < 100 || rect.height < 55;
        if (recordButton) {
          const recordLeft = !sponsoredHomeVideo && liveCanDownload && !button.hidden
            ? left + button.offsetWidth + 8
            : left;
          recordButton.style.left = `${recordLeft}px`;
          recordButton.style.top = `${Math.max(6, top)}px`;
          recordButton.hidden = sponsoredHomeVideo || rect.width < 100 || rect.height < 55;
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
      const refreshOverlayLanguage = () => {
        if (button.textContent.startsWith("⇩")) button.textContent = `⇩ ${downloadLabel()}`;
        button.style.setProperty("--apocalipse-accent", overlayThemeColors());
        if (recordButton) refreshRecordLabels();
        recordButton?.style.setProperty("--apocalipse-accent", overlayThemeColors());
      };
      activeOverlays.set(element, {
        element,
        pageUrl: location.href,
        bindingId: isFacebookVideo ? playerIdentity(element) : null,
        cleanup: cleanupOverlay,
        refreshLabels: refreshOverlayLanguage,
      });
      const duplicateButtons = document.querySelectorAll(".apocalipse-media-download").length - activeOverlays.size * 2;
      trace("overlay_installed", "overlay", { tag: element.tagName, canDownload, canRecord, sponsoredRecordOnly: Boolean(isFacebookVideo && facebookSponsoredEvidence(element)), active: activeOverlays.size, duplicateDelta: duplicateButtons });
      if (isSocialVideo) {
        socialSummary.overlays += 1;
        socialSummary.installed += 1;
        emitSocialDecision(element, "install", {
          reason: "eligible_overlay_installed", sponsored: false,
          canDownload, canRecord, hasDirectHttpMedia,
          permalinkFound: Boolean(isFacebookVideo ? facebookUrlFor(element) : tikTokUrl),
          downloadCandidate: Boolean(url),
        }, true);
      }
      position();
      addEventListener("scroll", position, { passive: true });
      addEventListener("resize", position, { passive: true });
      if (isYouTubeVideo) positionTimer = setInterval(position, 1000);
    });

    // Final reconciliation catches the exact class of bug where a visible
    // social player passed through scanning but still ended the cycle without
    // a live overlay. This is the highest-value event for post-mortem analysis.
    for (const video of document.querySelectorAll("video")) {
      if (socialPlatform() === "generic" || !socialPlayerVisible(video)) continue;
      const active = activeOverlays.has(video);
      const cached = socialDecisionCache.get(video) || "";
      if (!active && !/"decision":"skip_inactive_player"/.test(cached)
        && !/"decision":"missing"/.test(cached)) {
        socialSummary.missing += 1;
        emitSocialDecision(video, "missing", {
          reason: "visible_player_without_overlay_after_reconcile",
          sponsored: socialPlatform() === "facebook" ? isSponsoredFacebookPlayer(video) : false,
          datasetButton: Boolean(video.dataset?.apocalipseButton),
        }, true);
      }
    }
    emitSocialSummary(socialSummary);
  };
  refreshOverlayLanguages = () => {
    for (const overlay of activeOverlays.values()) overlay.refreshLabels?.();
  };
  let catalogTimer = null;
  const scheduleCatalog = () => {
    // Throttle rather than indefinitely debounce a continuously mutating feed.
    if (catalogTimer !== null) return;
    catalogTimer = setTimeout(() => {
      catalogTimer = null;
      if (document.visibilityState === "hidden") return;
      collect();
    }, 350);
  };
  const scheduleOverlays = () => {
    // Busy players can mutate their controls on every frame. Throttle the
    // installer so continuous DOM activity cannot postpone it forever.
    if (overlayTimer !== null && overlayTimer !== undefined) return;
    overlayTimer = setTimeout(() => {
      overlayTimer = null;
      installOverlays();
    }, 250);
    scheduleCatalog();
  };
  // Facebook virtualizes the Home feed and can reuse an existing <video>
  // without inserting a fresh node. Re-run overlay validation while scrolling
  // and on player lifecycle changes so a recycled player cannot keep stale state.
  addEventListener("scroll", scheduleOverlays, { passive: true, capture: true });
  for (const event of ["loadedmetadata", "loadstart", "load", "emptied"]) {
    document.addEventListener(event, scheduleOverlays, true);
  }
  const style = document.createElement("style");
  style.textContent = ".apocalipse-media-download{position:absolute!important;z-index:2147483647!important;border:2px solid var(--apocalipse-accent,#25d9ef)!important;border-radius:8px!important;padding:8px 11px!important;background:#111a20f2!important;color:#fff!important;font:700 13px system-ui!important;box-shadow:0 3px 12px #0008!important;backdrop-filter:blur(5px)!important;cursor:pointer!important;overflow:hidden!important;isolation:isolate!important;transition:border-color .15s,background .15s,box-shadow .15s!important}.apocalipse-media-download::after{content:\"\"!important;position:absolute!important;inset:50%!important;border-radius:999px!important;background:color-mix(in srgb,var(--apocalipse-accent,#25d9ef) 42%,transparent)!important;opacity:0!important;pointer-events:none!important;transform:translate(-50%,-50%) scale(0)!important}.apocalipse-media-download.apocalipse-click-feedback{animation:apocalipse-overlay-press .34s cubic-bezier(.2,.8,.2,1)!important}.apocalipse-media-download.apocalipse-click-feedback::after{animation:apocalipse-overlay-wave .34s ease-out!important}@keyframes apocalipse-overlay-press{0%{transform:scale(1)}42%{transform:scale(.92);filter:brightness(1.3)}100%{transform:scale(1)}}@keyframes apocalipse-overlay-wave{0%{opacity:.85;transform:translate(-50%,-50%) scale(0)}100%{opacity:0;transform:translate(-50%,-50%) scale(5)}}.apocalipse-media-download:hover{background:#15262ef8!important;box-shadow:0 3px 14px var(--apocalipse-accent,#25d9ef)!important}.apocalipse-media-download:disabled{cursor:wait!important;opacity:.85!important}.apocalipse-media-record{color:#fff!important;background:#35151cf2!important}.apocalipse-media-record:hover{background:#4a1922f8!important}@media (prefers-reduced-motion:reduce){.apocalipse-media-download.apocalipse-click-feedback{animation-duration:.12s!important}.apocalipse-media-download.apocalipse-click-feedback::after{animation:none!important}}";
  document.documentElement.append(style);
  const restartOverlayButtonFeedback = (button) => {
    if (!button || button.disabled) return;
    button.classList.remove("apocalipse-click-feedback");
    void button.offsetWidth;
    button.classList.add("apocalipse-click-feedback");
    setTimeout(() => {
      try { if (button.isConnected) button.classList.remove("apocalipse-click-feedback"); } catch {}
    }, 360);
  };
  document.addEventListener("pointerdown", (event) => {
    restartOverlayButtonFeedback(event.target?.closest?.(".apocalipse-media-download"));
  }, true);
  document.addEventListener("keydown", (event) => {
    if (event.key !== "Enter" && event.key !== " ") return;
    restartOverlayButtonFeedback(event.target?.closest?.(".apocalipse-media-download"));
  }, true);
  new MutationObserver(scheduleOverlays).observe(document.documentElement, { childList: true, subtree: true, attributes: true, attributeFilter: ["src", "poster"] });
  scheduleOverlays();
  const overlayRefreshTimer = setInterval(() => {
    if (!extensionContextActive()) return clearInterval(overlayRefreshTimer);
    try { scheduleOverlays(); } catch {}
  }, 2000);
  const appearanceSyncTimer = setInterval(() => {
    if (!extensionContextActive()) return clearInterval(appearanceSyncTimer);
    if (!activeOverlays.size) return;
    sendRuntimeMessageQuietly({ type: "APOCALIPSE_SYNC_DESKTOP_APPEARANCE" }).then((result) => {
      if (!result?.language) return;
      const changed = result.language !== interfaceLanguage || result.theme !== interfaceTheme;
      interfaceLanguage = result.language;
      interfaceTheme = result.theme || "void";
      if (changed) refreshOverlayLanguages();
    }).catch(() => {});
  }, 2000);
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
    if (message?.type === "APOCALIPSE_CAPTURE_PLAYER_THUMBNAIL") {
      const element = [...document.querySelectorAll("video")]
        .find(video => playerIdentity(video) === message.playerId);
      const current = element && collect().find(item => item.playerId === message.playerId
        && item.url === message.url && !item.retained);
      if (!current || (message.pageUrl && message.pageUrl !== location.href)) {
        reply({ dataUrl: "", error: "stale_player_binding" });
        return;
      }
      const snapshot = playerSnapshot(element);
      captureThumbnailFor(element, "video", message.rejectedSource).then(dataUrl => {
        reply({ dataUrl: samePlayerSnapshot(element, snapshot) ? dataUrl : "" });
      }).catch(() => reply({ dataUrl: "" }));
      return true;
    }
    if (message?.type !== "APOCALIPSE_SCAN") return;
    const scanTrace = globalThis.ADM_DIAG?.begin("popup.scan_started", { stage: "content" });
    const found = collect();
    void globalThis.ADM_DIAG?.emit("popup.dom_inventory", { count: found.length, videos: found.filter(v => v.kind === "video").length,
      audio: found.filter(v => v.kind === "audio").length, images: found.filter(v => v.kind === "image").length }, scanTrace);
    (async () => {
      let selectedItems = found;
      const hls = found.filter((item) => /\.m3u8(?:$|[?#])/i.test(item.url));
      if (hls.length > 0) {
        const visibleMedia = document.querySelector("video,audio");
        const analyzed = await chrome.runtime.sendMessage({ type: "APOCALIPSE_ANALYZE_HLS", urls: hls.map((item) => item.url), duration: Number.isFinite(visibleMedia?.duration) ? visibleMedia.duration : null });
        const details = new Map((analyzed || []).map((item) => [item.url, item]));
        selectedItems = found.map((item) => {
          const detail = details.get(item.url);
          return detail ? { ...item, ...detail, kind: detail.mediaKind || item.kind } : item;
        });
      }
      return Promise.all(selectedItems.map(async (item) => {
      try {
        if (item.visualOnly || item.retained) return item;
        return { ...item, ...(await chrome.runtime.sendMessage({ type: "APOCALIPSE_PROBE", url: item.url })) };
      } catch {
        return item;
      }
      }));
    })().then((media) => {
      void globalThis.ADM_DIAG?.emit("popup.scan_reply", { count: media.length }, scanTrace);
      reply({ pageUrl: location.href, media });
    }).catch(error => {
      void globalThis.ADM_DIAG?.emit("popup.scan_failed", { errorRef: String(error) }, scanTrace, "ERROR");
      reply({ pageUrl: location.href, media: found });
    });
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
    if (!extensionContextActive()) return;
    try {
      window.postMessage({
        source: "apocalipse-extension",
        type: "shortcut-config",
        bypass: shortcutKeys.bypass,
        force: shortcutKeys.force,
      }, "*");
    } catch {}
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
    if (data?.source === "apocalipse-page-hook" && data.type === "hook-pong") {
      mainHookReady = true;
      void sendRuntimeMessageQuietly({
        type: "APOCALIPSE_MAIN_HOOK_READY",
        version: data.version || extensionVersion(),
        topFrame: window === window.top,
      });
      return;
    }
    if (data?.source === "apocalipse-page-hook" && data.type === "capture-trace") {
      void sendRuntimeMessageQuietly({ type: "APOCALIPSE_CAPTURE_TRACE", eventName: data.eventName, mode: data.mode, detail: data.detail || {}, traceId: data.traceId, pageUrl: location.href, at: data.at || Date.now() });
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
  pingMainHook();

})();
