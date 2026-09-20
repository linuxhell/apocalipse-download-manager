const test = require("node:test");
const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const ROOT = path.resolve(__dirname, "..");
const FORBIDDEN = ("go" + "peed").toLowerCase();
const TEXT_EXTENSIONS = new Set([
  ".rs", ".js", ".cjs", ".json", ".html", ".css", ".md", ".yml", ".yaml",
  ".toml", ".txt", ".ps1", ".sh", ".plist"
]);
const ROOT_FILES = new Set(["Cargo.toml", "Cargo.lock", "README.md", "ROADMAP.md", "SECURITY.md"]);
const ROOT_DIRECTORIES = [
  ".github", "apps", "browser-extension", "crates", "docs", "packaging", "scripts", "tests"
];

function walk(target, results = []) {
  if (!fs.existsSync(target)) return results;
  const stat = fs.statSync(target);
  if (stat.isDirectory()) {
    for (const name of fs.readdirSync(target)) {
      if ([".git", "target", "node_modules", "dist"].includes(name)) continue;
      walk(path.join(target, name), results);
    }
  } else {
    results.push(target);
  }
  return results;
}

test("obsolete transfer engine is completely absent from product sources", () => {
  const files = [
    ...ROOT_DIRECTORIES.flatMap((directory) => walk(path.join(ROOT, directory))),
    ...[...ROOT_FILES].map((name) => path.join(ROOT, name)).filter(fs.existsSync),
  ];

  const residue = [];
  for (const file of files) {
    const relative = path.relative(ROOT, file).replaceAll("\\", "/");
    if (relative.toLowerCase().includes(FORBIDDEN)) {
      residue.push(`path:${relative}`);
      continue;
    }
    const extension = path.extname(file).toLowerCase();
    if (!TEXT_EXTENSIONS.has(extension) && !ROOT_FILES.has(relative)) continue;
    const content = fs.readFileSync(file, "utf8");
    if (content.toLowerCase().includes(FORBIDDEN)) {
      residue.push(`content:${relative}`);
    }
  }

  assert.deepEqual(residue, []);
});
