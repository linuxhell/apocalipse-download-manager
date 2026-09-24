const test = require("node:test");
const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const root = path.resolve(__dirname, "..");
const popupJs = fs.readFileSync(path.join(root, "browser-extension/popup.js"), "utf8");
const popupCss = fs.readFileSync(path.join(root, "browser-extension/popup.css"), "utf8");
const contentJs = fs.readFileSync(path.join(root, "browser-extension/content.js"), "utf8");
const appJs = fs.readFileSync(path.join(root, "apps/desktop/ui/app.js"), "utf8");

// The desktop app's canonical theme list ("valid" in app.js). Any file below
// that decides whether a theme name from the desktop is trusted must agree
// with this list, or a real theme choice silently falls back to "void".
const currentThemes = [
  "void", "nebula", "ember", "jade", "plasma", "glacier", "amber", "abyss",
  "rust", "venom", "wine", "linen", "sky", "blossom", "sage", "sand",
  "lilac", "mist", "citrus", "coral", "frost",
];

test("app.js still defines exactly the theme list this test locks in", () => {
  const match = appJs.match(/const valid = \[([^\]]+)\];/);
  assert.ok(match, "expected to find app.js's `valid` theme array");
  const declared = match[1].match(/"([a-z]+)"/g).map((s) => s.slice(1, -1));
  assert.deepEqual(declared, currentThemes);
});

test("browser extension popup accepts every current desktop theme, not a retired palette", () => {
  // Regression: popupThemes previously listed a pre-rename 26-theme palette
  // (inferno, toxic, synthwave, ...) that no longer exists anywhere in the
  // app. Every non-void theme choice failed the popupThemes.has() check and
  // silently fell back to "void", so only the language ever reached the
  // extension, never the theme.
  const setMatch = popupJs.match(/const popupThemes = new Set\(\[([^\]]+)\]\);/);
  assert.ok(setMatch, "expected to find popup.js's popupThemes Set");
  const declared = setMatch[1].match(/"([a-z]+)"/g).map((s) => s.slice(1, -1));
  assert.deepEqual(new Set(declared), new Set(currentThemes));
  assert.doesNotMatch(popupJs, /"inferno"|"synthwave"|"whiteaurora"|"pearlblue"/);
});

test("popup.css has a rendered palette for every current non-void theme", () => {
  for (const theme of currentThemes) {
    if (theme === "void") continue; // void is the base :root palette, no selector needed
    assert.match(
      popupCss,
      new RegExp(`:root\\[data-theme="${theme}"\\]`),
      `popup.css is missing a --p-* palette block for theme "${theme}"`,
    );
  }
  assert.doesNotMatch(popupCss, /\[data-theme="(inferno|synthwave|whiteaurora|pearlblue|polarmint)"\]/);
});

test("content.js overlay accent colors track the current theme names", () => {
  const accentsMatch = contentJs.match(/const accents = \{([^}]+)\};/);
  assert.ok(accentsMatch, "expected to find content.js's overlayThemeColors accents map");
  for (const theme of currentThemes) {
    assert.match(accentsMatch[1], new RegExp(`\\b${theme}:`));
  }
  assert.doesNotMatch(accentsMatch[1], /\binferno:|\bsynthwave:|\bwhiteaurora:/);
});
