// Applied before the first paint: the saved colour mode (light is the default).
try { document.documentElement.dataset.mode = localStorage.getItem("apocalipse.mode") === "dark" ? "dark" : "light"; } catch { document.documentElement.dataset.mode = "light"; }
