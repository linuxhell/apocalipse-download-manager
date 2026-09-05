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
  const copyTextNow = (value) => {
    void navigator.clipboard.writeText(value).catch(() => {});
    const input = document.createElement("textarea");
    input.value = value;
    input.setAttribute("readonly", "");
    input.style.cssText = "position:fixed;left:-10000px;top:0;opacity:0";
    document.documentElement.append(input);
    input.select();
    input.setSelectionRange(0, input.value.length);
    let copied = false;
    try { copied = document.execCommand("copy"); } catch {}
    input.remove();
    return copied;
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
  const thumbnailFor = (element, kind) => {
    if (kind === "audio") return "";
    if (element?.tagName === "IMG") return element.currentSrc || element.src || "";
    return element?.poster || document.querySelector('meta[property="og:image"]')?.content || element?.closest?.("figure,article")?.querySelector?.("img")?.currentSrc || "";
  };
  const isFacebookMediaUrl = (url) => {
    try {
      const parsed = new URL(url, location.href);
      if (!/(^|\.)facebook\.com$/i.test(parsed.hostname)) return false;
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
    let container = element;
    for (let depth = 0; container && depth < 12; depth += 1, container = container.parentElement) {
      const anchors = container.querySelectorAll?.('a[href*="/video/"]') || [];
      for (const anchor of anchors) {
        const url = absolute(anchor.href);
        if (isTikTokVideoUrl(url)) return url;
      }
      const markup = (container.innerHTML || "").replaceAll("\\/", "/");
      const path = markup.match(/\/@[^/"'<>\\s]+\/video\/\d+/i)?.[0];
      if (path && isTikTokVideoUrl(path)) return absolute(path);
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
    copyItem.click();
    for (let attempt = 0; attempt < 15; attempt += 1) {
      await new Promise((resolve) => setTimeout(resolve, 100));
      try {
        const copied = (await navigator.clipboard.readText()).trim();
        if (isFacebookMediaUrl(copied)) return copied;
      } catch {}
    }
    return "clipboard-copied";
  };
  const revealFacebookUrl = async (element) => {
    const immediate = facebookUrlFor(element);
    if (immediate) return immediate;
    if (!/(^|\.)facebook\.com$/i.test(location.hostname)) return null;
    const postText = element.closest?.('[role="article"],article')?.textContent || "";
    if (/(?:patrocinado|sponsored)/i.test(postText)) {
      const menuUrl = await facebookUrlFromMenu(element);
      if (menuUrl) return menuUrl;
    }
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
      items.set(`${kind}:${url}`, { url, kind, thumbnail: thumbnail ?? thumbnailFor(element, kind), title: titleFor(element), size: null });
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
      if (/\.m3u8(?:$|[?#])/i.test(entry.name)) add(entry.name, "video", null, document.querySelector("video")?.poster || "");
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
      const isFacebookVideo = element.tagName === "VIDEO" && /(^|\.)facebook\.com$/i.test(location.hostname);
      const tikTokUrl = element.tagName === "VIDEO" ? tikTokUrlFor(element) : null;
      const isTikTokVideo = Boolean(tikTokUrl);
      const url = isFacebookVideo ? facebookUrlFor(element) || location.href : tikTokUrl || downloadUrlFor(element);
      const canDownload = Boolean(url && /^https?:/.test(url));
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
        const visibleFacebookUrl = isFacebookVideo && isFacebookMediaUrl(location.href) ? location.href : null;
        const resolved = visibleFacebookUrl || (isFacebookVideo ? await revealFacebookUrl(element) : tikTokUrlFor(element) || await resolveDownloadUrl(element));
        if (resolved === "clipboard-copied") {
          button.textContent = "✓";
          button.title = "Link copiado; o Apocalipse abrirá a janela de download";
          setTimeout(() => { button.textContent = originalText; }, 2500);
          return;
        }
        const currentUrl = resolved?.url || resolved || (isYouTubeVideo ? location.href : null);
        if (!currentUrl || (isFacebookVideo && !isFacebookMediaUrl(currentUrl))) {
          button.textContent = "⚠";
          button.title = "Abra o vídeo ou use os três pontos e Copiar link";
          setTimeout(() => { button.textContent = originalText; }, 2500);
          return;
        }
        const copiedToClipboard = isFacebookVideo ? copyTextNow(currentUrl) : false;
        chrome.runtime.sendMessage({ type: "APOCALIPSE_DOWNLOAD", item: { url: currentUrl, duration: resolved?.duration || null, requestUrls: resolved?.requestUrls || [], userAgent: navigator.userAgent, kind: element.tagName.toLowerCase(), title: document.title, thumbnail: thumbnailFor(element, "video") } }, (result) => {
          const failed = !copiedToClipboard && (chrome.runtime.lastError || result?.target !== "apocalipse");
          button.textContent = failed ? "⚠" : "✓";
          if (failed) button.title = result?.error || chrome.runtime.lastError?.message || "Apocalipse unavailable";
          setTimeout(() => { button.textContent = originalText; }, 1500);
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
            recorder.stop();
            record.disabled = true;
            record.textContent = labels.uploading;
            return;
          }
          try {
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
              request: { fileName: `${safeTitle}.recording.webm`, total: 0, source: location.href, streaming: true },
            });
            if (!begin?.uploadId) throw new Error(begin?.error || "recording_begin_failed");
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
              } catch (error) {
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
            console.error("Apocalipse recorder", error);
            record.textContent = "⚠";
            record.title = labels.unavailable;
            setTimeout(() => { record.textContent = labels.record; }, 2000);
          }
        });
      }
      let positionTimer = null;
      const position = () => {
        if (!element.isConnected) {
          button.remove();
          recordButton?.remove();
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
        button.hidden = !canDownload || rect.width < 100 || rect.height < 55;
        if (recordButton) {
          recordButton.style.left = `${Math.max(6, rect.left + scrollX + (canDownload ? 104 : 8))}px`;
          recordButton.style.top = `${Math.max(6, top)}px`;
          recordButton.hidden = rect.width < 100 || rect.height < 55;
        }
      };
      document.documentElement.append(button);
      if (recordButton) document.documentElement.append(recordButton);
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
    return true;
  });
})();
