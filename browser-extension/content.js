(() => {
  const absolute = (value) => { try { return new URL(value, location.href).href; } catch { return null; } };
  const collect = () => {
    const items = new Map();
    const add = (url, kind, element) => {
      url = absolute(url); if (!url || !/^https?:/.test(url)) return;
      const thumbnail = element?.poster || element?.closest?.("figure,article")?.querySelector?.("img")?.currentSrc || "";
      items.set(`${kind}:${url}`, { url, kind, thumbnail, title: element?.title || document.title, size: null });
    };
    document.querySelectorAll("video").forEach(el => { add(el.currentSrc || el.src, "video", el); el.querySelectorAll("source").forEach(s => add(s.src, "video", el)); });
    document.querySelectorAll("audio").forEach(el => { add(el.currentSrc || el.src, "audio", el); el.querySelectorAll("source").forEach(s => add(s.src, "audio", el)); });
    document.querySelectorAll("img").forEach(el => add(el.currentSrc || el.src, "image", el));
    return [...items.values()];
  };
  chrome.runtime.onMessage.addListener((message, _sender, reply) => {
    if (message?.type === "APOCALIPSE_SCAN") reply({ pageUrl: location.href, media: collect() });
  });
})();

