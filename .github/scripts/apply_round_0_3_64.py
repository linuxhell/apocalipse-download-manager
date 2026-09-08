from pathlib import Path
import json

# 0.3.64 evidence-driven fix:
# Rapidgator trace proves Shift reaches FORCE_ARMED, but the generic force path
# captures the visible /download/captcha page itself. emit() then suppresses the
# click, so the site never gets a chance to create the final sN.rapidgator.net
# attachment request. Keep force armed across the gesture, but only preempt a
# generic anchor when it already looks like a direct downloadable resource.
# Dynamic buttons/menuitems are allowed to run so fetch/window.open/location can
# expose the real downstream URL to the existing capture layer.

p = Path('browser-extension/page-hook.js')
s = p.read_text(encoding='utf-8')

needle = '''  const forceClassify = (value) => { if (!forceActive() || bypassPressed()) return null; try { const u = new URL(String(value || ""), location.href); return /^(https?):$/i.test(u.protocol) ? { url: u.href, kind: "forced" } : null; } catch { return null; } };\n'''
replacement = needle + '''  const forceDirectAnchorClassify = (anchor) => {
    if (!forceActive() || bypassPressed() || !anchor?.href) return null;
    try {
      const u = new URL(anchor.href, location.href);
      if (!/^(https?):$/i.test(u.protocol)) return null;
      const here = new URL(location.href);
      // A same-document/current-page target is an action trigger, not the file.
      // Let the page execute and capture the downstream primitive instead.
      if (u.origin === here.origin && u.pathname === here.pathname && u.search === here.search) return null;
      if (anchor.hasAttribute("download")) return { url: u.href, kind: "forced" };
      const path = u.pathname.toLowerCase();
      if (/\\.(?:7z|apk|avi|bin|bz2|csv|deb|dmg|docx?|epub|exe|flac|gz|iso|jpeg?|m4a|mkv|mov|mp3|mp4|msi|pdf|png|pptx?|rar|rpm|tar|tgz|torrent|txt|wav|webm|webp|xlsx?|xz|zip)(?:$|\\.)/i.test(path)) return { url: u.href, kind: "forced" };
      // Cross-host links are commonly CDN/object-storage handoffs. They are safe
      // to preempt under an explicit force gesture; same-host opaque action URLs
      // must execute so their real response/request can be observed.
      if (u.hostname !== here.hostname) return { url: u.href, kind: "forced" };
      return null;
    } catch { return null; }
  };\n'''
if needle not in s:
    raise SystemExit('forceClassify marker not found')
s = s.replace(needle, replacement, 1)

old = '''    const candidate = classify(this.href) || forceClassify(this.href);'''
new = '''    const candidate = classify(this.href) || forceDirectAnchorClassify(this);'''
if old not in s:
    raise SystemExit('HTMLAnchorElement.click force hook not found')
s = s.replace(old, new, 1)

old = '''    const candidate = classify(anchor?.href) || forceClassify(anchor?.href);'''
new = '''    const candidate = classify(anchor?.href) || forceDirectAnchorClassify(anchor);'''
if old not in s:
    raise SystemExit('document click force hook not found')
s = s.replace(old, new, 1)

# Extend force correlation. Five seconds was enough for a direct click but can
# expire during Rapidgator/React async handoff. This remains gesture-scoped and
# bypass still wins immediately.
s = s.replace('if (forcePressed()) forceGestureUntil = Date.now() + 5000;', 'if (forcePressed()) forceGestureUntil = Date.now() + 15000;', 1)

# Add explicit trace when a forced gesture intentionally passes through an
# opaque action target. This makes the next diagnostic distinguish pass-through
# from a missed listener without guessing.
trace_needle = '''    trace(forcePressed() ? "FORCE_ARMED" : (bypassPressed() ? "BYPASS_ARMED" : "AUTO_GESTURE"), { tag: clickable?.tagName || target?.tagName || "", role: clickable?.getAttribute?.("role") || "", text: String(clickable?.innerText || clickable?.textContent || "").trim().slice(0,120), href: safeUrl(clickable?.href || "") });\n'''
trace_repl = trace_needle + '''    if (forcePressed() && clickable && !classify(clickable.href || "") && !forceDirectAnchorClassify(clickable)) trace("FORCE_PASSTHROUGH", { tag: clickable.tagName || "", role: clickable.getAttribute?.("role") || "", href: safeUrl(clickable.href || "") });\n'''
if trace_needle not in s:
    raise SystemExit('FORCE_ARMED trace marker not found')
s = s.replace(trace_needle, trace_repl, 1)

p.write_text(s, encoding='utf-8')

manifest_path = Path('browser-extension/manifest.json')
manifest = json.loads(manifest_path.read_text(encoding='utf-8'))
manifest['version'] = '0.3.64'
manifest_path.write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
print('Applied 0.3.64 force downstream handoff fix')
