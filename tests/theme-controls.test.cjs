const test = require("node:test");
const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const css = fs.readFileSync(
  path.resolve(__dirname, "../apps/desktop/ui/styles.css"),
  "utf8",
);

test("About audio controls inherit theme colors", () => {
  const block = css.slice(
    css.indexOf(".about-audio-controls {"),
    css.indexOf(".about-scene {"),
  );
  assert.match(block, /color:\s*var\(--text\)/);
  assert.match(block, /border:\s*1px solid var\(--line\)/);
  assert.match(block, /background:\s*var\(--surface\)/);
  assert.match(block, /\.about-audio-controls label \{ color:\s*var\(--text\)/);
  assert.doesNotMatch(block, /color:\s*#fff/);
});

test("Apocalipse Link delete button follows the active theme", () => {
  const start = css.indexOf(".link-files header #link-delete-remote {");
  const end = css.indexOf(".link-files > small", start);
  const block = css.slice(start, end);
  assert.ok(start >= 0 && end > start);
  assert.match(block, /var\(--accent\)/);
  assert.match(block, /var\(--surface-2\)/);
  assert.match(block, /var\(--line\)/);
  assert.match(block, /var\(--muted\)/);
  assert.doesNotMatch(block, /#713443|#ff9aaa/i);
});
