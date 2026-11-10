# Apocalipse Download Manager 0.8.15

- Chrome/Edge 0.3.210 keeps the successful response from a POST download form and streams its bytes into ADM's destination workflow. Fixes UUP Dump ZIP captures that discarded an HTTP 200 response and repeated the POST, triggering HTTP 429.
- Check ADM availability before consuming a POST. Browser fallback is allowed before sending; failures after sending never automatically resubmit the form, including uncertain worker communication failures.
- Count decoded bytes for HTTP-compressed responses, retain bounded streaming and pause handling, and release the response reader on success or failure.
- Retain Nibiru Xtreme 0.2.6 and the unchanged Mozilla-signed Firefox 0.3.209 XPI. Firefox 0.3.209 does not include the new POST fix; that requires a signed 0.3.210 package.

Regression tests use a local HTTP server that returns a ZIP once and rejects duplicate requests. The user should retest the original UUP Dump form with the updated extension; these tests do not confirm the live website result.
