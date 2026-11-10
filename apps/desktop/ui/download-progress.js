// Receiving the last byte does not confirm that disk sync/publication succeeded.
(function (root) {
  function isDownloadFinalizing(task, torrent = false) {
    if (task.state !== "downloading" || task.is_live || torrent) return false;
    const total = Number(task.total), received = Number(task.received);
    return Number.isFinite(total) && total > 0
      && Number.isFinite(received) && received >= total;
  }
  root.isDownloadFinalizing = isDownloadFinalizing;
  if (typeof module === "object" && module.exports) module.exports = { isDownloadFinalizing };
})(globalThis);
