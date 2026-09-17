(() => {
  const selector = "button, .list-select-button";

  const animate = (target) => {
    const control = target instanceof Element ? target.closest(selector) : null;
    if (!control) return;
    if (control.matches("button:disabled") || control.querySelector("input:disabled")) return;
    control.classList.remove("button-click-feedback");
    void control.offsetWidth;
    control.classList.add("button-click-feedback");
    window.setTimeout(() => control.classList.remove("button-click-feedback"), 360);
  };

  document.addEventListener("pointerdown", (event) => animate(event.target), true);
  document.addEventListener("keydown", (event) => {
    if (event.key === "Enter" || event.key === " ") animate(event.target);
  }, true);
})();
