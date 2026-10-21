const test = require("node:test");
const assert = require("node:assert/strict");
const crypto = require("node:crypto");
const fs = require("node:fs");
const os = require("node:os");
const path = require("node:path");
const { spawnSync } = require("node:child_process");

const root = path.join(__dirname, "..");
const script = path.join(root, "scripts", "build_evidence.py");
const manifest = JSON.parse(fs.readFileSync(path.join(root, "scripts", "nibiru-engines.json"), "utf8"));
const commit = "0123456789abcdef0123456789abcdef01234567";
const sha = (bytes) => crypto.createHash("sha256").update(bytes).digest("hex");

function fixture(mutate = () => {}) {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), "adm-evidence-"));
  const engines = path.join(dir, "tools");
  const records = [];
  for (const entry of manifest.engines) {
    fs.mkdirSync(path.join(engines, entry.name), { recursive: true });
    const bytes = Buffer.from(`binary of ${entry.name}`);
    fs.writeFileSync(path.join(engines, entry.name, entry.name), bytes);
    records.push({ engine: entry.name, available: true, revision: entry.revision, sha256: sha(bytes) });
  }
  mutate(records, engines);
  fs.writeFileSync(path.join(engines, "build.json"), JSON.stringify(records));
  const adm = path.join(dir, "adm");
  fs.writeFileSync(adm, "adm executable");
  return { dir, engines, adm, output: path.join(dir, "build-evidence.json") };
}

const run = ({ engines, adm, output }, sha1 = commit) =>
  spawnSync("python3", ["-I", script, "--adm-binary", adm, "--engines", engines, "--commit", sha1, "--output", output], { encoding: "utf8" });

test("evidence names the ADM commit, each engine revision and every executable hash", () => {
  const f = fixture();
  const result = run(f);
  assert.equal(result.status, 0, result.stderr);
  const evidence = JSON.parse(fs.readFileSync(f.output, "utf8"));
  assert.equal(evidence.adm.commit, commit);
  assert.equal(evidence.adm.executable_sha256, sha("adm executable"));
  assert.deepEqual(evidence.engines.map((e) => e.engine), manifest.engines.map((e) => e.name));
  for (const entry of manifest.engines) {
    const record = evidence.engines.find((e) => e.engine === entry.name);
    assert.equal(record.revision, entry.revision);
    assert.equal(record.sha256, sha(`binary of ${entry.name}`));
  }
});

test("evidence is refused when an engine drifted from the pin, changed after build, or is missing", () => {
  const drifted = fixture((records) => { records[0].revision = "f".repeat(40); });
  assert.notEqual(run(drifted).status, 0);
  assert.match(run(drifted).stderr, /manifest pins/);

  const swapped = fixture((records, engines) => {
    fs.writeFileSync(path.join(engines, records[1].engine, records[1].engine), "replaced after the build");
  });
  assert.match(run(swapped).stderr, /changed after the build/);

  const missing = fixture((records) => { records.pop(); });
  assert.match(run(missing).stderr, /was not built/);
  assert.equal(fs.existsSync(missing.output), false, "no evidence file for a package that cannot be described");

  assert.match(run(fixture(), "not-a-sha").stderr, /40-character/);
});
