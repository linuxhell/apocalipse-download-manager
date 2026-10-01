(function (root) {
  function showDirectAudioConversion(url, fileName, mediaKind) {
    const names = [fileName || ""];
    try { names.push(decodeURIComponent(new URL(url).pathname)); }
    catch { names.push(String(url || "").split(/[?#]/)[0]); }
    if (names.some(name => /\.mp4$/i.test(name))) return false;
    return mediaKind === "audio" || names.some(name => /\.(?:mp3|m4a|aac|ogg|oga|wav|flac|opus|aiff?|wma|alac)$/i.test(name));
  }
  root.showDirectAudioConversion = showDirectAudioConversion;
  if (typeof module !== "undefined") module.exports = { showDirectAudioConversion };
})(typeof window !== "undefined" ? window : globalThis);
