(() => {
  if (window.__apocalipsePreDownloadHook) return;
  window.__apocalipsePreDownloadHook = true;

  let shortcuts = { bypass: "Alt", force: "Shift" };
  const held = new Set();
  let activeLibraryFileName = "";
  let forceGestureUntil = 0;
  let bypassGestureUntil = 0;
  let activeTraceId = "";
  const pendingNavigationFallbacks = new Map();

  const canonicalKey = (event) => {
    if (event.altKey) held.add("Alt"); else held.delete("Alt");
    if (event.shiftKey) held.add("Shift"); else held.delete("Shift");
    if (event.ctrlKey) held.add("Control"); else held.delete("Control");
  };
  addEventListener("keydown", canonicalKey, true);
  addEventListener("keyup", canonicalKey, true);
  addEventListener("blur", () => held.clear(), true);

  addEventListener("message", (event) => {
    if (event.source !== window || event.data?.source !== "apocalipse-extension") return;
    if (event.data.type === "shortcut-config") {
      shortcuts = {
        bypass: event.data.bypass || "Alt",
        force: event.data.force || "Shift",
      };
    }
  });

  const bypassPressed = () => held.has(shortcuts.bypass || "Alt");
  const bypassActive = () => bypassPressed() || Date.now() < bypassGestureUntil;
  const forcePressed = () => held.has(shortcuts.force || "Shift");
  const forceActive = () => !bypassActive() && (forcePressed() || Date.now() < forceGestureUntil);
  const safeUrl = (value) => { try { const u = new URL(String(value || ""), location.href); u.search = ""; u.hash = ""; return u.href; } catch { return ""; } };
  const trace = (eventName, detail = {}) => window.postMessage({ source: "apocalipse-page-hook", type: "capture-trace", eventName, mode: bypassActive() ? "bypass" : (forceActive() ? "force" : "normal"), detail, traceId: activeTraceId, at: Date.now() }, "*");
  const forceClassify = (value) => { if (!forceActive() || bypassPressed()) return null; try { const u = new URL(String(value || ""), location.href); return /^(https?):$/i.test(u.protocol) ? { url: u.href, kind: "forced" } : null; } catch { return null; } };
  const forceDirectAnchorClassify = (anchor) => {
    if (!forceActive() || bypassActive() || !anchor?.href) return null;
    try {
      const u = new URL(anchor.href, location.href);
      if (!/^(https?):$/i.test(u.protocol)) return null;
      const here = new URL(location.href);
      // Explicit download actions often point back to the current route. The
      // authenticated GET itself can return the one-use file response.
      if (u.origin === here.origin && u.pathname === here.pathname && u.search === here.search) {
        const label = `${anchor.getAttribute?.("aria-label") || ""} ${anchor.title || ""} ${anchor.textContent || ""}`;
        return /download|baixar|descarregar|descargar|télécharger|scarica|herunterladen|下载/i.test(label)
          ? { url: u.href, kind: "forced-action", method: "GET" }
          : null;
      }
      if (anchor.hasAttribute("download")) return { url: u.href, kind: "forced" };
      const path = u.pathname.toLowerCase();
      if (/\.(?:7z|apk|avi|bin|bz2|csv|deb|dmg|docx?|epub|exe|flac|gz|iso|jpeg?|m4a|mkv|mov|mp3|mp4|msi|pdf|png|pptx?|rar|rpm|tar|tgz|torrent|txt|wav|webm|webp|xlsx?|xz|zip)(?:$|\.)/i.test(path)) return { url: u.href, kind: "forced" };
      // Cross-host links are commonly CDN/object-storage handoffs. They are safe
      // to preempt under an explicit force gesture; same-host opaque action URLs
      // must execute so their real response/request can be observed.
      if (u.hostname !== here.hostname) return { url: u.href, kind: "forced" };
      return null;
    } catch { return null; }
  };
  const classify = (value) => {
    try {
      const url = new URL(String(value || ""), location.href);
      if (url.hostname.toLowerCase() === "chatgpt.com" && url.pathname === "/backend-api/estuary/content") {
        return { url: url.href, kind: "chatgpt-library" };
      }
    } catch {}
    return null;
  };

  const emit = (candidate, primitive, fallbackOnFailure = false) => {
    if (!candidate || bypassActive()) { if (candidate && bypassActive()) trace("BYPASS", { primitive, url: safeUrl(candidate.url) }); return false; }
    const requestId = `${Date.now()}-${Math.random().toString(36).slice(2)}`;
    trace(forceActive() ? "FORCE_CAPTURE" : "AUTO_ACCEPT", { primitive, kind: candidate.kind, url: safeUrl(candidate.url), requestId });
    window.postMessage({
      source: "apocalipse-page-hook",
      type: "pre-download-url",
      requestId,
      url: candidate.url,
      kind: candidate.kind,
      primitive,
      fileName: candidate.kind === "chatgpt-library" ? activeLibraryFileName : "",
      force: forceActive(),
      method: candidate.method || "GET",
      body: candidate.body || null,
      contentType: candidate.contentType || null,
    }, "*");
    if (fallbackOnFailure) pendingNavigationFallbacks.set(requestId, candidate.url);
    return true;
  };

  addEventListener("message", (event) => {
    if (event.source !== window || event.data?.source !== "apocalipse-extension" || event.data.type !== "pre-download-result") return;
    const fallback = pendingNavigationFallbacks.get(event.data.requestId);
    pendingNavigationFallbacks.delete(event.data.requestId);
    if (!fallback || event.data.result?.ok) return;
    bypassGestureUntil = Date.now() + 5000;
    trace("FORCE_CAPTURE_FALLBACK", { url: safeUrl(fallback), error: String(event.data.result?.error || "takeover_failed") });
    location.assign(fallback);
  });

  // Remember which Library row opened the Radix menu. The menu itself is portaled
  // under <body>, so this association must be captured before the menu item is clicked.
  document.addEventListener("pointerdown", (event) => {
    activeTraceId = `${Date.now()}-${Math.random().toString(36).slice(2)}`;
    if (bypassPressed()) bypassGestureUntil = Date.now() + 4000;
    else if (forcePressed()) forceGestureUntil = Date.now() + 20000;
    const target = event.target instanceof Element ? event.target : null;
    const clickable = target?.closest?.("a[href],button,[role=button],[role=menuitem]");
    trace(forcePressed() ? "FORCE_ARMED" : (bypassActive() ? "BYPASS_ARMED" : "AUTO_GESTURE"), { tag: clickable?.tagName || target?.tagName || "", role: clickable?.getAttribute?.("role") || "", text: String(clickable?.innerText || clickable?.textContent || "").trim().slice(0,120), href: safeUrl(clickable?.href || "") });
    if (forcePressed() && clickable && !classify(clickable.href || "")) {
      const forcedAction = forceDirectAnchorClassify(clickable);
      trace(forcedAction ? "FORCE_ACTION_CLASSIFIED" : "FORCE_PASSTHROUGH", {
        tag: clickable.tagName || "",
        role: clickable.getAttribute?.("role") || "",
        kind: forcedAction?.kind || "none",
        text: String(clickable.innerText || clickable.textContent || "").trim().slice(0,120),
        href: safeUrl(clickable.href || ""),
      });
    }
    const button = event.target?.closest?.('button[data-testid^="file-row-actions-"]');
    if (!button || location.hostname !== "chatgpt.com") return;
    const label = button.getAttribute("aria-label") || "";
    activeLibraryFileName = label
      .replace(/^.*?(?:ações de|acoes de|actions for|actions of)\s+/i, "")
      .trim();
  }, true);

  const originalAnchorClick = HTMLAnchorElement.prototype.click;
  HTMLAnchorElement.prototype.click = function(...args) {
    const candidate = classify(this.href) || forceDirectAnchorClassify(this);
    if (emit(candidate, "anchor.click", true)) return;
    return originalAnchorClick.apply(this, args);
  };

  const originalOpen = window.open;
  window.open = function(url, ...args) {
    const candidate = classify(url) || forceClassify(url);
    if (emit(candidate, "window.open", true)) return null;
    return originalOpen.call(this, url, ...args);
  };

  const originalFetch = window.fetch;
  window.fetch = function(input, init) {
    const value = typeof input === "string" || input instanceof URL ? String(input) : input?.url;
    const candidate = classify(value);
    if (candidate?.kind === "chatgpt-library") {
      trace(forceActive() ? "FORCE_PASSTHROUGH" : "AUTO_PASSTHROUGH", { primitive: "window.fetch", kind: candidate.kind, url: safeUrl(candidate.url) });
    } else if (emit(candidate, "window.fetch")) {
      return Promise.resolve(new Response("", { status: 204, statusText: "Handled by Apocalipse" }));
    }
    const forcedAtCall = forceActive() && !bypassActive();
    return originalFetch.call(this, input, init).then((response) => {
      if (forcedAtCall) {
        const disposition = response.headers?.get?.("content-disposition") || "";
        const type = response.headers?.get?.("content-type") || "";
        const fileLike = /attachment|filename=/i.test(disposition) || /application\/(octet-stream|zip|x-rar|pdf)|video\//i.test(type);
        if (fileLike) emit(forceClassify(response.url || value), "window.fetch.response");
        else trace("FORCE_OBSERVED_RESPONSE", { status: response.status, type: type.slice(0,80), url: safeUrl(response.url || value) });
      }
      return response;
    });
  };

  // Catch ordinary anchors after page/React handlers had a chance to update href,
  // but before the browser performs the default navigation/download action.
  document.addEventListener("click", (event) => {
    const anchor = event.target?.closest?.("a[href]");
    const candidate = classify(anchor?.href) || forceDirectAnchorClassify(anchor);
    if (!candidate || bypassActive()) return;
    event.preventDefault();
    event.stopImmediatePropagation();
    emit(candidate, "document.click", true);
  }, true);

  // Some pages call Location.assign/replace instead of clicking an anchor.
  for (const method of ["assign", "replace"]) {
    try {
      const original = Location.prototype[method];
      Location.prototype[method] = function(url) {
        const candidate = classify(url) || forceClassify(url);
        if (emit(candidate, `location.${method}`, true)) return;
        return original.call(this, url);
      };
    } catch {}
  }
})();
