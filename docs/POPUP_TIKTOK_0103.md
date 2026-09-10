# Extension 0.3.103 test candidate (ADM 0.4.29 compatible)

No desktop source or scrollbar code is changed by this follow-up. It is based
on diagnostic candidate a744da7 (PR #60). No releases, tags or automatic merge.

## Reproduced cause: one script resource in two execution worlds
The manifest in 0.3.102 declares tiktok-identity.js in MAIN and ISOLATED static
content-script entries. In an actual installed Chromium extension, the MAIN
resolver exists but the ISOLATED resolver is missing. The overlay is installed
without its binding and falls through to the old fullscreen path.

The installed-browser test now compares three clean profiles on the SAME
synthetic TikTok DOM, with real Chrome extension APIs and a local ADM stub:

1. Exact a744da7 / 0.3.102: MAIN identity=true, ISOLATED identity=false,
   fullscreenChanges=2 (entry and exit), zero download handoffs.
2. Exact baseline with ONLY a distinct filename for the MAIN resolver (its
   bytes are unchanged): both identities=true, sequential 111 and 222 reel
   handoffs correct, fullscreenChanges=0. No other popup/native change.
3. Full 0.3.103 candidate: both identities=true; video appears in popup BEFORE
   the overlay is clicked; sequential 111/222 handoffs correct; no fullscreen;
   unidentified blobs are not sent; a real webRequest response reaches the
   popup; select-all/navigation/language work; guarded batches remain guarded;
   a deliberately silent message receiver times out; no popup page errors.

Run proving this comparison: 34425595571, source da5064d074829d962107cde81a95781b50670246.
The test uses synthetic URLs/media and an ADM stub. It does NOT prove playback
in Windows VLC or every current logged-in TikTok layout. It does reproduce an
actual packaged-extension defect without deliberately breaking the baseline.

Chromium source inspected during investigation is consistent with this result:
user_script_injector.cc checks executing_scripts by script_url.GetPath();
script_injection.cc keys that executed-script set by extension host ID, not by
execution world. The filename-only comparison is the decisive empirical check.

## Packaging invariant
MAIN now uses tiktok-identity-main.js; ISOLATED uses tiktok-identity.js.
They must remain byte-identical. After editing the resolver, copy its contents
to the MAIN resource. tests/tiktok-world-packaging.test.cjs enforces equality
and prevents shared static JavaScript resource paths across worlds.

## Relation to Save scrolling
The 0.3.98 manifest had no dual-world TikTok resolver. PR #57 (0.3.99) introduced
that resolver and other media guards in the same release as the custom Save
scrollbar. The scrollbar is loaded only by the desktop document. The experiment
above restores handoff without changing any desktop code: there is a release
chronology link, but no demonstrated causal link to the scrollbar itself.

The latest private owner diagnostic had only a desktop session with two events,
zero browser contexts and an inactive bridge at export. It cannot prove the
exact user's click path or explain every control described as locked. Do not
interpret missing records as absence of network activity or blame installation.

## Additional changes
- Remove the actual requestFullscreen/exitFullscreen fallback from content.js.
- Call the TikTok handler directly with the button's exact video element.
- Reuse the same content snapshot/resolver for popup discovery across frames;
  remove nearest-link guessing and independent timers appending old reels.
- Bound IPC waits and restore controls after errors. No automatic write retry.
- Keep unverified rows selectable, with analysis/explanations instead of silent
  disabled controls. Never hand off unknown incomplete media as verified video.
- Preserve selection atomically; contain URL decode/render errors.
- Keep controls within narrow popup bounds.
- Add independent Copy extension diagnostics (no desktop required), exporting
  version, safe errors and control states, not tokens, URLs or page text.

## Acceptance
88 Node tests passed. Browser DOM tests additionally exercise actual rendered
controls with mocked transport. Installed-browser tests described above use
real extension messaging and worlds but synthetic site/desktop endpoints.
This verifies the specific injection failure and guarded popup behavior, not
all possible causes of the owner's report that every control appeared locked.

Update only the extension; retain ADM 0.4.29 and its engines/settings. Check
popup interaction, current-reel download, scroll to another reel and no
fullscreen flash. If still failing, Copy extension diagnostics works even
without an active desktop bridge. A browser v3 session must be started from the
target tab's extension popup, not from desktop-only diagnostics. Keep PR draft
until owner acceptance. No private diagnostic archive is committed.
