# Extension 0.3.103 test candidate (ADM 0.4.29 compatible)

No desktop source or scrollbar code is changed by this follow-up. It is based
on the diagnostic candidate a744da7 (PR #60), not an approved TikTok release.

## Evidence and limits
The latest owner-supplied diagnostic contains a desktop-only session with two
events, no browser contexts, and no active bridge at export time. It cannot
establish which extension code handled that click. Do not interpret missing
browser records as a root cause or as proof that no network request occurred.
The Save scrollbar runs only in the desktop document. Several capture/preview
changes shipped alongside it; temporal association is not a shared CSS path.

## Changes
- Remove the actual requestFullscreen/exitFullscreen fallback from content.js.
- The TikTok overlay calls the handler directly with its own video element.
  Losing a delegate lookup can no longer send the click into the generic path.
- Popup discovery reuses the same content snapshot/resolver across frames.
  No nearest-link heuristic and no independent timers that append old reels.
- Bound IPC waits and restore controls after errors. Never retry a write merely
  because its acknowledgement timed out.
- Unverified rows remain selectable. Analyze/preview provide an explanation and
  attempt a safe identity refresh; they do not send unknown incomplete media.
  Batch download sends only verified selected rows and reports skipped items.
- Atomic refresh preserves selection; URL decode errors cannot kill rendering.
- Responsive layout keeps the player/download controls within popup width.
- Independent Copy extension diagnostics works without the desktop bridge or a
  functioning worker, showing version, bootstrap phases, safe errors and control
  states. It never exports token values, URLs, cookies or page text.

## Verification
Node regression tests and browser DOM fixtures are required. The installed
Chromium test uses the real extension worlds, messages and webRequest, but a
synthetic page and a localhost ADM stub, not a logged-in TikTok session or VLC.
It compares an intentionally missing delegate binding in 0.3.102 with this
candidate. That demonstrates a fallback defect; it does not prove the exact
reason the owner's binding was unavailable.

Do not merge, create tags or publish releases until manual acceptance. Update
only the extension for this test; keep ADM 0.4.29 and its settings/engines. If the
popup still fails, use Copy extension diagnostics even if Start diagnostics
cannot contact ADM. Browser collection must be started from the target tab's
extension popup; the desktop-only session does not include the browser.
