(() => {
  globalThis.ADM_DIAG?.register("tiktok-media-fix.js");
  const videoForButton = button => globalThis.ApocalipseTikTokIdentity?.videoFor(button) || null;
  const permalinkFor = video => globalThis.ApocalipseTikTokIdentity?.resolve(video) || null;

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

  document.addEventListener("click", async (event) => {
    const button = event.target?.closest?.(".apocalipse-media-download:not(.apocalipse-media-record)");
    if (!button) return;
    const video = videoForButton(button);
    if (!video) {
      void globalThis.ADM_DIAG?.emit("overlay.binding_missing", { reason: "no_video_for_button", identityAvailable: Boolean(globalThis.ApocalipseTikTokIdentity) }, null, "WARN");
      return;
    }
    const actionId = globalThis.ADM_DIAG?.begin("overlay.click", { ...globalThis.ADM_DIAG.player(video), handler: "tiktok" }) || crypto.randomUUID();
    event.preventDefault();
    event.stopImmediatePropagation();
    chrome.runtime.sendMessage({
      type: "APOCALIPSE_CAPTURE_TRACE",
      eventName: "tiktok_overlay_handler_claimed",
      mode: "download",
      traceId: actionId,
      pageUrl: location.href,
      at: Date.now(),
      detail: { topFrame: window === window.top, hasBoundVideo: true },
    }).catch(() => {});
    const original = button.textContent;
    const clickSource = String(video.currentSrc || video.src || "");
    const clickPage = location.href;
    button.textContent = "…";
    const url = permalinkFor(video);
    void globalThis.ADM_DIAG?.emit("identity.overlay_result", { ...globalThis.ADM_DIAG.player(video), resolved: Boolean(url), permalinkUrl: url || "" }, actionId, url ? "INFO" : "WARN");
    // A permalink is a complete extractor task, never a hint for selecting a
    // possibly audio-only/video-only CDN response from the same tab.
    const captured = !url
      ? await chrome.runtime.sendMessage({ type: "APOCALIPSE_RECENT_TAB_MEDIA" }).catch(() => null)
      : null;
    const capturedMedia = Array.isArray(captured?.media) ? captured.media : [];
    if (!video.isConnected || clickSource !== String(video.currentSrc || video.src || "") || clickPage !== location.href) {
      void globalThis.ADM_DIAG?.emit("overlay.player_changed", { reason: "player_changed_during_lookup" }, actionId, "WARN");
      button.textContent = "!";
      button.title = "O vídeo mudou. Clique novamente no vídeo atual.";
      setTimeout(() => { button.textContent = original; }, 2500);
      return;
    }
    const resourceKey = (value) => {
      if (!value || !/^https?:/i.test(value)) return "";
      try {
        const parsed = new URL(value);
        for (const name of [...parsed.searchParams.keys()]) {
          if (/^(?:bytestart|byteend|range|start|end)$/i.test(name)) parsed.searchParams.delete(name);
        }
        return parsed.href;
      } catch { return ""; }
    };
    // Never select a lone buffered response or an unrelated item in global
    // React state: neither establishes which video the user clicked.
    const liveHttpUrl = directVideoUrl(clickSource);
    const videoItem = liveHttpUrl ? capturedMedia.find((item) =>
      /^video\//i.test(item.contentType || "") && resourceKey(item.url) === resourceKey(liveHttpUrl)) : null;
    const audioCandidates = videoItem && Number.isFinite(videoItem.capturedAt) && videoItem.capturedAt > 0
      ? capturedMedia.filter((item) => /^https?:/i.test(item.url || "") && /^audio\//i.test(item.contentType || "")
        && Number.isFinite(item.capturedAt) && item.capturedAt > 0
        && (item.frameId === videoItem.frameId || (item.frameId == null && videoItem.frameId == null))
        && Math.abs(item.capturedAt - videoItem.capturedAt) <= 8_000)
        .filter((audio) => !capturedMedia.some((other) =>
          /^video\//i.test(other.contentType || "") && Number.isFinite(other.capturedAt) && other.capturedAt > 0
          && (other.frameId === audio.frameId || (other.frameId == null && audio.frameId == null))
          && resourceKey(other.url) !== resourceKey(videoItem.url)
          && Math.abs(other.capturedAt - audio.capturedAt) <= Math.abs(videoItem.capturedAt - audio.capturedAt)))
        .sort((left, right) => Math.abs(left.capturedAt - videoItem.capturedAt) - Math.abs(right.capturedAt - videoItem.capturedAt))
      : [];
    const selectedUrl = url || videoItem?.url || liveHttpUrl;
    void globalThis.ADM_DIAG?.emit("capture.overlay_inventory", { candidates: capturedMedia.length, selected: Boolean(selectedUrl), url: selectedUrl || "", matchedPlayer: Boolean(videoItem), pageExtractor: Boolean(url) }, actionId);
    const tiedAudio = audioCandidates.length > 1
      && Math.abs(audioCandidates[0].capturedAt - videoItem.capturedAt) === Math.abs(audioCandidates[1].capturedAt - videoItem.capturedAt);
    const audioUrl = url || tiedAudio ? null : audioCandidates[0]?.url || null;
    const ambiguousSocialTrack = Boolean(!url && !audioUrl && selectedUrl);
    chrome.runtime.sendMessage({
      type: "APOCALIPSE_CAPTURE_TRACE",
      eventName: "tiktok_browser_media_selection",
      mode: "download",
      traceId: actionId,
      pageUrl: location.href,
      at: Date.now(),
      detail: {
        selection: url ? "complete_page_extractor" : videoItem ? "matched_player_media" : liveHttpUrl ? "player_http_media" : "media_picker_required",
        selectedVideoId: url?.match(/\/video\/(\d+)/i)?.[1] || "none",
        browserCapturedMedia: Boolean(videoItem),
        browserCandidates: capturedMedia.length,
        browserAudioCaptured: Boolean(audioUrl),
        ambiguousSocialTrack,
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
      let context = video;
      for (let depth = 0; context && depth < 10; depth += 1, context = context.parentElement) {
        const text = String(context.innerText || "").trim();
        if (text.length >= 8 && text.length <= 1200) break;
      }
      const title = String(context?.innerText || document.title || "TikTok").trim().replace(/\s+/g, " ").slice(0, 240);
      chrome.runtime.sendMessage({
        type: "APOCALIPSE_OPEN_MEDIA_PICKER",
        context: { title, thumbnail: video.poster || "", kind: "video", duration: Number.isFinite(video.duration) ? video.duration : null },
      }).catch(() => {});
      setTimeout(() => { button.textContent = original; }, 2500);
      return;
    }
    chrome.runtime.sendMessage({
      type: "APOCALIPSE_DOWNLOAD",
      item: {
        traceId: actionId,
        url: selectedUrl,
        duration: Number.isFinite(video.duration) && video.duration > 0 ? video.duration : null,
        audioUrl,
        ambiguousSocialTrack,
        pageExtractor: Boolean(url),
        requestUrls: url ? [] : [selectedUrl, ...(audioUrl ? [audioUrl] : [])],
        userAgent: navigator.userAgent,
        kind: "video",
        title: document.title,
        thumbnail: video.poster || "",
      },
    }, (result) => {
      const failed = chrome.runtime.lastError || !result?.ok;
      void globalThis.ADM_DIAG?.emit("overlay.handoff_reply", { ok: !failed, errorRef: result?.error || "" }, actionId, failed ? "ERROR" : "INFO");
      button.textContent = failed ? "⚠" : "✓";
      button.title = failed
        ? result?.error || chrome.runtime.lastError?.message || "Apocalipse unavailable"
        : "Enviado ao Apocalipse";
      setTimeout(() => { button.textContent = original; }, 1800);
    });
  }, true);
})();
