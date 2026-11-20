// Shared, allowlisted palettes. Keep the desktop copy byte-identical.
(function (root) {
  "use strict";
  const palettes = [
  {
    "id": "white",
    "dark": false,
    "names": {
      "en": "White",
      "pt-BR": "Branco",
      "zh-CN": "白色"
    },
    "colors": {
      "bg": "#ffffff",
      "panel": "#f6f6f6",
      "raised": "#eeeeee",
      "ink": "#111111",
      "muted": "#555555",
      "line": "#cccccc",
      "accent": "#111111",
      "on": "#ffffff",
      "soft": "#e8e8e8",
      "done": "#1b713c",
      "warning": "#835700",
      "error": "#b32839",
      "track": "#dddddd"
    }
  },
  {
    "id": "black",
    "dark": true,
    "names": {
      "en": "Black",
      "pt-BR": "Preto",
      "zh-CN": "黑色"
    },
    "colors": {
      "bg": "#141414",
      "panel": "#1c1c1c",
      "raised": "#282828",
      "ink": "#ececec",
      "muted": "#b4b4b4",
      "line": "#444444",
      "accent": "#f2f2f2",
      "on": "#111111",
      "soft": "#303030",
      "done": "#8cddb3",
      "warning": "#f8cf80",
      "error": "#ffacb7",
      "track": "#444444"
    }
  },
  {
    "id": "obsidian",
    "dark": true,
    "names": {
      "en": "Obsidian",
      "pt-BR": "Obsidiana",
      "zh-CN": "黑曜石"
    },
    "colors": {
      "accent": "#79dbec",
      "bg": "#10191d",
      "panel": "#17252b",
      "raised": "#20343c",
      "ink": "#eaf6f8",
      "muted": "#abc3cc",
      "line": "#36505c",
      "on": "#08262e",
      "soft": "#1e3e48",
      "done": "#a6ddba",
      "warning": "#f2ce87",
      "error": "#ffb1b8",
      "track": "#38505a"
    }
  },
  {
    "id": "sapphire",
    "dark": true,
    "names": {
      "en": "Sapphire",
      "pt-BR": "Safira",
      "zh-CN": "蓝宝石"
    },
    "colors": {
      "accent": "#83b5ff",
      "bg": "#0d1423",
      "panel": "#141e30",
      "raised": "#1b2940",
      "ink": "#edf3ff",
      "muted": "#a9bad4",
      "line": "#30415c",
      "on": "#0b2141",
      "soft": "#233c5f",
      "done": "#8cddb3",
      "warning": "#f8cf80",
      "error": "#ffacb7",
      "track": "#34465f"
    }
  },
  {
    "id": "amethyst",
    "dark": true,
    "names": {
      "en": "Amethyst",
      "pt-BR": "Ametista",
      "zh-CN": "紫水晶"
    },
    "colors": {
      "accent": "#c5a7ff",
      "bg": "#181322",
      "panel": "#211b30",
      "raised": "#2d2540",
      "ink": "#f5efff",
      "muted": "#c0afd4",
      "line": "#493c5e",
      "on": "#281448",
      "soft": "#3a2b51",
      "done": "#a5ddc3",
      "warning": "#f2cf93",
      "error": "#ffb2bd",
      "track": "#493b5e"
    }
  },
  {
    "id": "emerald",
    "dark": true,
    "names": {
      "en": "Emerald",
      "pt-BR": "Esmeralda",
      "zh-CN": "翡翠"
    },
    "colors": {
      "accent": "#7fdbb6",
      "bg": "#101c19",
      "panel": "#172a24",
      "raised": "#20392f",
      "ink": "#eaf7f1",
      "muted": "#aac7ba",
      "line": "#365347",
      "on": "#092c20",
      "soft": "#244637",
      "done": "#b5dfa1",
      "warning": "#f1cd83",
      "error": "#ffb1b2",
      "track": "#3b584b"
    }
  },
  {
    "id": "copper",
    "dark": true,
    "names": {
      "en": "Copper",
      "pt-BR": "Cobre",
      "zh-CN": "铜色"
    },
    "colors": {
      "accent": "#edb482",
      "bg": "#201812",
      "panel": "#2c211a",
      "raised": "#3a2c22",
      "ink": "#fff2e8",
      "muted": "#d0b5a0",
      "line": "#5a4535",
      "on": "#3b200c",
      "soft": "#4a3221",
      "done": "#b6d3a7",
      "warning": "#f4d495",
      "error": "#ffaaa3",
      "track": "#5b4535"
    }
  },
  {
    "id": "porcelain",
    "dark": false,
    "names": {
      "en": "Porcelain",
      "pt-BR": "Porcelana",
      "zh-CN": "瓷白"
    },
    "colors": {
      "accent": "#285bd4",
      "bg": "#edf1f7",
      "panel": "#ffffff",
      "raised": "#f1f4fa",
      "ink": "#18223a",
      "muted": "#52607b",
      "line": "#cdd6e5",
      "on": "#ffffff",
      "soft": "#e6edfd",
      "done": "#227346",
      "warning": "#8a5905",
      "error": "#b32943",
      "track": "#d6dfef"
    }
  },
  {
    "id": "pearl",
    "dark": false,
    "names": {
      "en": "Pearl",
      "pt-BR": "Pérola",
      "zh-CN": "珍珠"
    },
    "colors": {
      "accent": "#7651b8",
      "bg": "#f1edf5",
      "panel": "#fffcff",
      "raised": "#f3eef8",
      "ink": "#2b213a",
      "muted": "#6a597b",
      "line": "#d9cce5",
      "on": "#ffffff",
      "soft": "#eee5fa",
      "done": "#286b50",
      "warning": "#87550a",
      "error": "#ae2948",
      "track": "#ded3eb"
    }
  },
  {
    "id": "sand",
    "dark": false,
    "names": {
      "en": "Sand",
      "pt-BR": "Areia",
      "zh-CN": "沙色"
    },
    "colors": {
      "accent": "#995027",
      "bg": "#f2ede4",
      "panel": "#fffdf8",
      "raised": "#f5efe5",
      "ink": "#35291f",
      "muted": "#71604c",
      "line": "#d9cdbd",
      "on": "#ffffff",
      "soft": "#f4e4d2",
      "done": "#376b42",
      "warning": "#7d5c04",
      "error": "#b12e3f",
      "track": "#e1d5c3"
    }
  }
];
  const byId = new Map(palettes.map(palette => [palette.id, Object.freeze(palette)]));
  function resolve(id, dark = false) {
    return byId.get(id) || byId.get(dark ? "black" : "white");
  }
  function apply(element, id, popup = false) {
    const palette = resolve(id);
    const c = palette.colors;
    const variables = popup ? {
      "p-bg": c.bg, "p-head": c.panel, "p-surface": c.panel,
      "p-surface-2": c.raised, "p-input": c.bg, "p-text": c.ink,
      "p-muted": c.muted, "p-line": c.line, "p-accent": c.accent,
      "p-on-accent": c.on, "p-success": c.done, "p-error": c.error,
      "p-track": c.track,
    } : {
      surface: c.bg, "surface-2": c.panel, text: c.ink, muted: c.muted,
      faint: c.muted, line: c.line, "line-strong": c.line, strong: c.accent,
      "on-strong": c.on, "strong-hover": c.accent, hover: c.raised,
      track: c.track, thumb: c.muted, "thumb-hover": c.ink, blue: c.accent,
      green: c.done, red: c.error, yellow: c.warning, orange: c.warning,
      "theme-soft": c.soft, "theme-panel": c.panel,
    };
    for (const [key, value] of Object.entries(variables)) element.style.setProperty(`--${key}`, value);
    element.dataset.theme = palette.id;
    element.dataset.mode = palette.dark ? "dark" : "light";
    element.style.colorScheme = palette.dark ? "dark" : "light";
    return palette;
  }
  root.ADM_THEMES = Object.freeze({ palettes: Object.freeze(palettes), resolve, apply });
  if (typeof module === "object" && module.exports) module.exports = root.ADM_THEMES;
})(globalThis);
