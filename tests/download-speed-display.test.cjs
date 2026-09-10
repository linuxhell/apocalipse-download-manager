const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const test = require("node:test");

const root = path.resolve(__dirname, "..");
const ui = fs.readFileSync(path.join(root, "apps/desktop/ui/app.js"), "utf8");
const desktop = fs.readFileSync(
  path.join(root, "apps/desktop/src-tauri/src/main.rs"),
  "utf8",
);

test("active downloads keep the engine-reported speed visible", () => {
  assert.match(
    ui,
    /const externalSpeed = active \? Number\(task\.download_speed\) \|\| 0 : 0;/,
  );
  assert.doesNotMatch(
    ui,
    /const externalSpeed = now - changedAt < 2000/,
  );
});

test("quiet clipboard polling does not flood diagnostics", () => {
  assert.match(
    ui,
    /if \(!quiet\.has\(command\) && command !== "record_ui_diagnostic"\)/,
  );
  assert.match(
    desktop,
    /let Ok\(value\) = app\.clipboard\(\)\.read_text\(\) else \{\s*return Ok\(None\);/,
  );
});
