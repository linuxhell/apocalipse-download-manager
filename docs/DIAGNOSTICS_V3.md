# Diagnostics v3 - test candidate, not a release

Desktop 0.4.29 / extension 0.3.102. Built on PR58 0.3.101. This instrumentation does NOT claim to fix TikTok.

## Reproduce a browser problem
1. Update both the desktop candidate and extension; reload the target tab.
2. Open the extension on that tab and choose **Diagnosticar esta aba (10 min)** / **Diagnose this tab**.
3. Reproduce the failed popup, click or preview. Choose **Marcar problema agora**.
4. Keep ADM open for the diagnostic deliveries. Stop the session in the extension.
5. In ADM > Logs, use the existing ZIP export or **Copiar relatorio para IA**.

Desktop-only sessions can be started in ADM > Logs, for queue/UI problems. Starting a new session replaces the current session; export before starting another investigation. Detailed collection expires after ten minutes. Normal legacy logging remains available.

## Evidence
The existing v2 export files remain. New files: RELATORIO_PARA_IA.txt, logs/diagnostics-v3.jsonl, traces/actions.jsonl, capture/media-decisions.jsonl, state/players-popup.json, health/collectors.json.

An action trace follows a click through extension and bridge to the Save prompt, task binding and native media preparation. Frame context, player generation/source references, popup row enablement reasons, network filter decisions and handler ownership are observations. They are not automatically declared root causes. Last-stage and absent-event interpretations must account for collector health.

## Privacy and bounds
Advanced mode is explicit and scoped to one browser tab. No page text, HTML, raw credential headers or media bytes are collected. URL paths/query values and free text become session-salted SHA256 references before extension storage. Native ingestion sanitizes again. The session salt is not exported. Hostnames, numeric fields, allowed event codes, and UUIDs remain useful for correlation. The local session metadata is restricted to the current OS user on Unix.

Content buffers hold at most 200 records; worker buffers hold at most 1200 records and 2 MiB of JSON, with reported drops/errors. Desktop detailed files rotate at 4 MiB each (two files). Summaries are bounded. Close/crash/cache and inaccessible-frame gaps remain possible. The worker persists its queue before attempting delivery; acknowledgements/deduplication make normal retries idempotent. This is not a guarantee against power loss or every browser lifecycle edge case. Export includes legacy logs, which may retain sanitized URL paths and filenames: review before sharing publicly.

## Test-only policy
No release publication. release.yml is a read-only disabled notice. validate-portable.yml creates Actions artifacts only. PR58 remains draft because owner acceptance failed; this candidate is for collecting better evidence, not an approved TikTok fix.
