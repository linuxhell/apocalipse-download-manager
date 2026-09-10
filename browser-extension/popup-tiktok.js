// Reuse the exact resolver that owns the overlay. Never guess from the nearest
// anchor, viewport centre, address bar, or a previously scanned reel.
(() => {
  globalThis.ADM_TIKTOK_SCAN = async tabId => {
    if (!chrome.scripting?.executeScript) throw new Error('scripting_unavailable');
    const results = await ADM_POPUP.deadline(chrome.scripting.executeScript({
      target: { tabId, allFrames: true },
      func: () => {
        if (!/(^|\.)tiktok\.com$/i.test(location.hostname)) return { media: [], skipped: true };
        if (!globalThis.ADM_MEDIA_SCAN?.snapshot) return { media: [], error: 'content_script_unavailable' };
        try { return globalThis.ADM_MEDIA_SCAN.snapshot(); }
        catch (error) { return { media: [], error: 'frame_scan_failed', errorName: error?.name || 'Error' }; }
      },
    }), 4500);
    const media = [];
    for (const frame of results) {
      const result = frame.result;
      if (result?.error) {
        ADM_POPUP.record('tiktok_frame_scan', { code: result.error });
        void globalThis.ADM_DIAG?.emit('popup.frame_scan_error', { frameId: frame.frameId, reason: result.error }, null, 'WARN');
      }
      for (const item of result?.media || []) media.push({ ...item, frameId: frame.frameId });
    }
    return media;
  };
})();
