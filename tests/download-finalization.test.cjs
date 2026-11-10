const test = require("node:test");
const assert = require("node:assert/strict");
const { isDownloadFinalizing } = require("../apps/desktop/ui/download-progress.js");

test("the last byte stays active until the backend confirms publication", () => {
  const task = { state: "downloading", received: 1024, total: 1024 };
  assert.equal(isDownloadFinalizing(task), true);
  assert.equal(isDownloadFinalizing({ ...task, state: "completed" }), false);
  assert.equal(isDownloadFinalizing({ ...task, state: "paused" }), false);
  assert.equal(isDownloadFinalizing({ ...task, state: { failed: { message: "disk full" } } }), false);
});

test("partial and unknown lengths do not show a finalization stage", () => {
  for (const [received, total] of [[999, 1000], [0, 0], [100, null], [100, undefined], [100, NaN], [100, Infinity]]) {
    assert.equal(isDownloadFinalizing({ state: "downloading", received, total }), false);
  }
});

test("live recordings and torrent seeding are not inferred as disk finalization", () => {
  const task = { state: "downloading", received: 1024, total: 1024 };
  assert.equal(isDownloadFinalizing({ ...task, is_live: true }), false);
  assert.equal(isDownloadFinalizing(task, true), false);
});
