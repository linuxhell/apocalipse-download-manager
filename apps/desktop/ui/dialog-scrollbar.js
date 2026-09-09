// A persistent, keyboard-accessible rail for the Save form. Native scrolling
// remains the fallback when this script is unavailable; OS overlay preferences
// cannot hide this rail.
(function (root) {
  function attachDialogScrollbar(viewport, rail) {
    if (!viewport || !rail) return;
    const area = rail.querySelector('[data-scroll-area]');
    const thumb = rail.querySelector('[data-scroll-thumb]');
    const up = rail.querySelector('[data-scroll-up]');
    const down = rail.querySelector('[data-scroll-down]');
    let drag = null;
    const range = () => Math.max(0, viewport.scrollHeight - viewport.clientHeight);
    const move = (value) => { viewport.scrollTop = Math.max(0, Math.min(range(), value)); update(); };
    function update() {
      const max = range(), height = area.clientHeight;
      const thumbHeight = max ? Math.min(height, Math.max(32, height * viewport.clientHeight / viewport.scrollHeight)) : height;
      thumb.style.height = `${thumbHeight}px`;
      thumb.style.top = `${max ? viewport.scrollTop / max * (height - thumbHeight) : 0}px`;
      rail.setAttribute('aria-valuemin', '0');
      rail.setAttribute('aria-valuemax', String(Math.round(max)));
      rail.setAttribute('aria-valuenow', String(Math.round(viewport.scrollTop)));
      rail.setAttribute('aria-disabled', String(!max));
      up.disabled = viewport.scrollTop <= 0;
      down.disabled = viewport.scrollTop >= max;
      const language = document.documentElement.lang;
      const labels = language === 'pt-BR' ? ['Rolar op\u00e7\u00f5es de download', 'Rolar para cima', 'Rolar para baixo']
        : language === 'zh-CN' ? ['\u6eda\u52a8\u4e0b\u8f7d\u9009\u9879', '\u5411\u4e0a\u6eda\u52a8', '\u5411\u4e0b\u6eda\u52a8']
        : ['Scroll download options', 'Scroll up', 'Scroll down'];
      rail.setAttribute('aria-label', labels[0]); up.setAttribute('aria-label', labels[1]); down.setAttribute('aria-label', labels[2]);
    }
    up.onclick = () => move(viewport.scrollTop - 80);
    down.onclick = () => move(viewport.scrollTop + 80);
    rail.onkeydown = (event) => {
      const distances = { ArrowUp: -40, ArrowDown: 40, PageUp: -viewport.clientHeight, PageDown: viewport.clientHeight };
      if (event.key in distances) move(viewport.scrollTop + distances[event.key]);
      else if (event.key === 'Home') move(0);
      else if (event.key === 'End') move(range());
      else return;
      event.preventDefault();
    };
    area.onpointerdown = (event) => {
      if (event.button !== 0) return;
      event.preventDefault(); rail.focus();
      if (event.target === thumb) {
        drag = { y: event.clientY, top: viewport.scrollTop, id: event.pointerId };
        area.setPointerCapture(event.pointerId);
      } else {
        move(viewport.scrollTop + (event.clientY < thumb.getBoundingClientRect().top ? -1 : 1) * viewport.clientHeight);
      }
    };
    area.onpointermove = (event) => {
      if (!drag || event.pointerId !== drag.id) return;
      const travel = area.clientHeight - thumb.offsetHeight;
      if (travel > 0) move(drag.top + (event.clientY - drag.y) / travel * range());
    };
    area.onpointerup = area.onpointercancel = area.onlostpointercapture = () => { drag = null; };
    viewport.addEventListener('scroll', update, { passive: true });
    const observer = new ResizeObserver(update);
    observer.observe(viewport);
    if (viewport.firstElementChild) observer.observe(viewport.firstElementChild);
    new MutationObserver(update).observe(document.documentElement, { attributes: true, attributeFilter: ['lang'] });
    viewport.classList.add('custom-scroll-ready');
    rail.hidden = false;
    update();
  }
  root.attachDialogScrollbar = attachDialogScrollbar;
  if (typeof document !== 'undefined') attachDialogScrollbar(document.querySelector('#add-dialog-scroll'), document.querySelector('#add-dialog-scrollbar'));
})(globalThis);
