# Apocalipse Download Manager 0.8.15

- Chrome/Edge/Firefox 0.3.210 keeps the successful response from a POST download form and streams its bytes into ADM's destination workflow. Fixes UUP Dump ZIP captures that discarded an HTTP 200 response and repeated the POST, triggering HTTP 429.
- Check ADM availability before consuming a POST. Browser fallback is allowed before sending; failures after sending never automatically resubmit the form, including uncertain worker communication failures.
- Count decoded bytes for HTTP-compressed responses, retain bounded streaming and pause handling, and release the response reader on success or failure.
- Include the user-provided Mozilla-signed Firefox 0.3.210 XPI, verified against the extension source, Mozilla production certificate chain, cryptographic signature and every internal file hash. Firefox now includes the POST fix as well. XPI SHA-256: `5b730b9ecdd6ba2c0f80447624ef8942672ee2e436f594856a99cb352a51c202`. Retain Nibiru Xtreme 0.2.6.

Regression tests use a local HTTP server that returns a ZIP once and rejects duplicate requests. The user should retest the original UUP Dump form with the updated extension; these tests do not confirm the live website result.
