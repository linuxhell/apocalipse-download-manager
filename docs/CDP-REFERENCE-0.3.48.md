# CDP capture reference (0.3.48)

This document preserves the architecture that was used before CDP/debugger support was removed from the shipped browser extension. It is documentation only and must not be loaded by the extension.

## Why it existed

CDP was introduced to capture downloads that Chrome otherwise owned before Apocalipse could take them over. It worked for the Rapidgator final one-use download response and was later also used while investigating ChatGPT Library downloads. Its major UX drawbacks were the Chrome "being debugged" infobar and competition with the browser Save As flow.

## Rapidgator recognition

The final response was considered a Rapidgator download only when:

- hostname matched `s<digits>.rapidgator.net`
- path matched `/download/<uuid>`
- response was HTTP 2xx
- response looked like a file by `Content-Type` or `Content-Disposition: attachment`

The debugger was attached to the Rapidgator tab and `Fetch.enable` was armed for:

`https://s*.rapidgator.net/download/*`

at response stage.

## Rapidgator CDP data flow

1. `chrome.debugger.attach({tabId}, "1.3")`
2. `Fetch.enable` with the final Rapidgator URL pattern at response stage.
3. Handle `chrome.debugger.onEvent` / `Fetch.requestPaused`.
4. Validate final URL, status, content type and disposition.
5. Ask the Apocalipse bridge `/v1/blob/begin` for a destination and upload id.
6. `Fetch.takeResponseBodyAsStream` to take ownership of the response body.
7. Repeated `IO.read` calls (later reduced to small chunks) and upload each chunk to `/v1/blob/chunk`.
8. `/v1/blob/end` when complete.
9. `Fetch.failRequest(..., errorReason: "Aborted")` so Chrome did not consume the same response.
10. Detach debugger.

Important lesson: the final Rapidgator URL/token is usable by Apocalipse if the browser is prevented from consuming it first.

## ChatGPT Library CDP experiments

During Library debugging the extension also tried:

- `Browser.setDownloadBehavior({behavior: "deny"})`
- `Page.setDownloadBehavior({behavior: "deny"})`
- `Page.downloadWillBegin`

The Library URL recognized by the extension was:

`https://chatgpt.com/backend-api/estuary/content...`

CDP was not ultimately necessary to identify the correct user-action interception point. Manual testing proved that capture-phase `preventDefault()` + `stopImmediatePropagation()` on the Library menu item whose text is `Baixar`/`Download` can stop the native browser download before Chrome creates its Save As flow.

## Modifier semantics to preserve

- Normal click: Apocalipse owns recognized downloads.
- Alt (or configured bypass shortcut): browser owns the download.
- Shift (or configured force shortcut): force Apocalipse takeover.

## Recovery

The pre-removal implementation remains available in Git history on branch `fix/rapidgator-preclick-0.3.29`, including the 0.3.48 patch chain. The executable CDP implementation previously lived primarily in `browser-extension/background.js` and `browser-extension/rapidgator.js`; the shipped manifest required the `debugger` permission.
