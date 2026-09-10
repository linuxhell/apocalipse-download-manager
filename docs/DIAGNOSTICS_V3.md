# Diagnostic v3 (test candidate)

This is instrumentation, **not a claim that TikTok downloads are fixed**.
Baseline: main a03e399 (release publication disabled) plus the experimental 0.3.101 extension behavior from draft PR #58. The diagnostic candidate is desktop 0.4.29-diag.1 / extension 0.3.102. PR #58 stays draft. No tags/releases are created.

## Reproduce and export

1. Run the new desktop candidate and update the paired extension. Reload the affected web tab.
2. In that tab's extension popup choose **Capture this tab (5 min)**. This enables detailed observation for that tab only. No page navigation, fullscreen or download is triggered by the diagnostic collector.
3. Reproduce the original problem, then choose **Mark problem now**. Keep the desktop open. The popup reports pending, dropped and delivery errors. Stop collection when finished.
4. Before exporting, wait until Pending is zero. An older ADM does not implement `/v1/diagnostic-v3`, so events remain queued and delivery errors increase.
5. In ADM > Logs use Export log (one ZIP), or Copy report for AI (plain text). Mark problem now is also available in this panel.

## Package

The previous v2 entries remain available. Format version is now 3, with:

- `RELATORIO_PARA_IA.txt`: recent action chains, warnings, explicit uncertainty and collector limits.
- `traces/acoes.jsonl`: shared action IDs across popup/overlay, bridge and native task processing.
- `capture/decisoes-de-midia.jsonl`: network observation vs actual downloader filtering, identity results and popup input/final counts.
- `state/players-e-popup.json`: bounded snapshots of player state and popup decisions.
- `health/coletores.json`: collector readiness, outbox loss/error counters and aggregated repetitive queries.
- `logs/v3/events-*.jsonl`: three rotating generations, 4 MiB each. The in-memory report window is 4,000 events; evictions are reported.

## Privacy and constraints

The new structured events are sanitized in the extension before local outbox persistence/transmission, and again on the desktop. They contain no request/response bodies, cookies, authorization values or full DOM. Resource URLs become a hostname/scheme and a salted identifier. The salt is local to the capture and is not exported. Page titles/captions and sensitive object fields are removed. Native free-text paths/credential patterns are scrubbed. Existing v2 entries retain the legacy format and redaction behavior.

Detailed network logging observes media-like responses before the existing downloader host filter. It **does not change** that filter or select a resource. The existing volatile download cache is identified as such in logs; v3 persists only the sanitized diagnostic outbox, not active download credentials/media buffers.

The extension outbox is serialized and capped at 600 records / approximately 1.8 million serialized characters. Old debug records are evicted first. It survives worker recreation. Delivery is acknowledged by eventId; the desktop deduplicates within its retained window. It does not provide exactly-once delivery beyond retention or a power-loss durability guarantee.

Page-world probes are explicitly untrusted observations. In-memory browser cache and inaccessible frames may hide activity. No event is evidence that activity did not occur. Successful process launch is not proof of visible image/audible sound. Producer sequence and clientTimestamp must be considered for delayed batches; desktop timestamp is receipt time.

## Validation

`node --test tests/*.test.cjs` exercises original routing regressions plus privacy canaries, worker recreation/offline delivery, sender metadata, tab scope/expiry, action identity, and an observed-but-rejected CDN response without changing routing.

Rust tests in `diagnostics_v3.rs` cover secret scrubbing, bounded retention, ID validation, severity, persistence, duplicate batches and generated report entries. The existing portable validation workflow builds test artifacts only. Chromium smoke tests additionally check actual extension loading, controls and collection.

## Remaining work deliberately outside this patch

No automatic TikTok correction, unrestricted network capture, DevTools debugger attachment, full-DOM dump, cookie export, or automatic public upload. The legacy fullscreen fallback remains in the experimental download path and is now explicitly logged if reached. Use the resulting evidence to fix it in a separate behavior patch.
