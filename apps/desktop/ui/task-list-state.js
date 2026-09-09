// A delayed list response must never undo a confirmed removal or newer snapshot.
(function (root) {
  function createTaskListState() {
    let revision = 0, requested = 0, applied = 0;
    const removed = new Set(), removing = new Set();
    const visible = (tasks) => tasks.filter((task) => !removed.has(task.id) && !removing.has(task.id));
    return {
      visible,
      invalidate() { revision += 1; },
      beginRead() { return { revision, sequence: ++requested }; },
      acceptRead(ticket, tasks) {
        if (ticket.revision !== revision || ticket.sequence < applied) return null;
        applied = ticket.sequence;
        return visible(tasks);
      },
      beginRemoval(ids) {
        revision += 1;
        for (const id of ids) removing.add(id);
      },
      finishRemoval(ids, succeeded) {
        revision += 1;
        for (const id of ids) {
          removing.delete(id);
          if (succeeded) removed.add(id);
        }
      },
    };
  }
  root.createTaskListState = createTaskListState;
  if (typeof module === 'object' && module.exports) module.exports = { createTaskListState };
})(globalThis);
