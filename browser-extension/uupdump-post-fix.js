(() => {
  if (window.__apocalipsePostDownloadFormFix) return;
  window.__apocalipsePostDownloadFormFix = true;

  const replaying = new WeakSet();
  const pending = new Map();

  const isDownloadPost = (form) => {
    if (!(form instanceof HTMLFormElement)) return false;
    if (String(form.method || "get").toUpperCase() !== "POST") return false;
    try {
      const action = new URL(form.action || location.href, location.href);
      return action.origin === location.origin && action.pathname.endsWith("/get.php");
    } catch {
      return false;
    }
  };

  const replay = (form, submitter) => {
    replaying.add(form);
    try {
      if (typeof form.requestSubmit === "function") form.requestSubmit(submitter || undefined);
      else form.submit();
    } finally {
      queueMicrotask(() => replaying.delete(form));
    }
  };

  addEventListener("message", (event) => {
    if (event.source !== window || event.data?.source !== "apocalipse-extension") return;
    if (event.data.type !== "pre-download-result") return;
    const saved = pending.get(event.data.requestId);
    if (!saved) return;
    pending.delete(event.data.requestId);
    if (!event.data.result?.ok && saved.form.isConnected) replay(saved.form, saved.submitter);
  });

  document.addEventListener("submit", (event) => {
    const form = event.target;
    if (!isDownloadPost(form) || replaying.has(form)) return;

    const submitter = event.submitter instanceof HTMLElement ? event.submitter : null;
    const data = new FormData(form, submitter || undefined);
    if ([...data.values()].some((value) => value instanceof File && value.size > 0)) return;

    const body = new URLSearchParams();
    for (const [name, value] of data.entries()) body.append(name, String(value));

    event.preventDefault();
    event.stopImmediatePropagation();

    const requestId = `${Date.now()}-${Math.random().toString(36).slice(2)}`;
    pending.set(requestId, { form, submitter });
    window.postMessage({
      source: "apocalipse-page-hook",
      type: "pre-download-url",
      requestId,
      url: new URL(form.action || location.href, location.href).href,
      kind: "post-download-form",
      primitive: "form-submit",
      fileName: "",
      force: true,
      method: "POST",
      body: body.toString(),
      contentType: "application/x-www-form-urlencoded;charset=UTF-8",
    }, "*");
  }, true);
})();
