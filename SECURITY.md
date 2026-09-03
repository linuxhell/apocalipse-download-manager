# Security model

- The browser extension sends downloads only after a user action.
- The local bridge will bind to loopback only and require a per-install secret.
- URLs, filenames and response headers are untrusted input.
- External tools are executed without a shell and with explicit arguments.
- Tool updates must use HTTPS, pinned release sources and SHA-256 verification before replacement.
- Partial downloads are written to `*.part` and atomically renamed only after completion.
- DRM and access-control circumvention is out of scope.

Please report vulnerabilities privately to the repository owner rather than opening a public exploit report.

