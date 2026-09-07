(() => {
  if (!/(^|\.)rapidgator\.net$/i.test(location.hostname)) return;

  let armed = false;
  let arming = false;

  const arm = async () => {
    if (armed || arming) return armed;
    arming = true;
    try {
      const result = await chrome.runtime.sendMessage({
        type: "APOCALIPSE_RAPIDGATOR_ARM",
        pageUrl: location.href,
      });
      armed = Boolean(result?.armed);
      if (!armed) {
        console.warn("Apocalipse Rapidgator capture was not armed", result?.error || "unknown");
      }
      return armed;
    } catch (error) {
      console.warn("Apocalipse Rapidgator capture failed to arm", String(error));
      return false;
    } finally {
      arming = false;
    }
  };

  // Arm CDP as soon as the Rapidgator page is available. The final free-download
  // URL can be created by page JavaScript or navigation, so click interception
  // alone is not reliable. The original click/navigation is intentionally left
  // untouched; background.js captures the real 200 OK response before Chrome's
  // download manager sees it.
  void arm();
  document.addEventListener("visibilitychange", () => {
    if (document.visibilityState === "visible") void arm();
  });
  window.addEventListener("pageshow", () => void arm());

  // Re-arm immediately before likely download clicks as an extra guard if the
  // service worker/debugger was restarted while the countdown page stayed open.
  document.addEventListener("pointerdown", (event) => {
    if (event.button !== 0) return;
    const target = event.target?.closest?.("a,button,input[type=submit],input[type=button]");
    if (!target) return;
    const text = `${target.textContent || ""} ${target.value || ""} ${target.title || ""}`;
    const href = target.href || "";
    if (/descarr|download|baixar|clicar aqui/i.test(text) || /\/download\//i.test(href)) {
      void arm();
    }
  }, true);
})();
