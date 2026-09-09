const { test } = require('node:test');
const assert = require('node:assert/strict');
const { createTaskListState } = require('../apps/desktop/ui/task-list-state.js');
const old = { id: 'deleted', source: 'https://example.test/media.mp4' };
const fresh = { id: 'new', source: old.source };

test('a list request started before removal cannot restore the removed task', () => {
  const state = createTaskListState();
  const stale = state.beginRead();
  state.beginRemoval([old.id]);
  state.finishRemoval([old.id], true);
  assert.equal(state.acceptRead(stale, [old]), null);
  assert.deepEqual(state.acceptRead(state.beginRead(), [old, fresh]), [fresh]);
});
test('enqueueing a new download cannot revive a deleted id, even in a stale backend snapshot', () => {
  const state = createTaskListState();
  state.beginRemoval([old.id]);
  state.finishRemoval([old.id], true);
  state.invalidate();
  assert.deepEqual(state.visible([old, fresh]), [fresh]);
  assert.deepEqual(state.acceptRead(state.beginRead(), [old, fresh]), [fresh]);
});
test('newer list responses win when polling completes out of order', () => {
  const state = createTaskListState();
  const first = state.beginRead(), second = state.beginRead();
  assert.deepEqual(state.acceptRead(second, [fresh]), [fresh]);
  assert.equal(state.acceptRead(first, [old]), null);
});
test('a failed removal can be retried and does not hide the task permanently', () => {
  const state = createTaskListState();
  state.beginRemoval([old.id]);
  assert.deepEqual(state.visible([old]), []);
  state.finishRemoval([old.id], false);
  assert.deepEqual(state.acceptRead(state.beginRead(), [old]), [old]);
});
test('removal is by task id, not URL or destination, so intentional redownloads are visible', () => {
  const state = createTaskListState();
  state.finishRemoval([old.id], true);
  assert.deepEqual(state.visible([fresh]), [fresh]);
});
