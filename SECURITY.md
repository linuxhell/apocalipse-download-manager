# Security model

- The browser extension sends downloads only after a user action.
- The browser bridge binds to loopback and requires a per-install secret.
- Apocalipse Link listens on the local network for authenticated transfers. Its current transport is not encrypted; use it only on trusted networks or through a trusted VPN.
- URLs, filenames and response headers are untrusted input.
- External tools are executed without a shell and with explicit arguments.
- Tool updates use HTTPS and replace only the selected executable. Signed update manifests and mandatory SHA-256 verification remain hardening work.
- Partial downloads are written to `*.part` and atomically renamed only after completion.
- Diagnostic logs redact cookies, URL credentials and configured proxy passwords. Secure-vault storage for every persisted secret remains hardening work.
- Credentials are scoped to an exact URL origin and are never forwarded across origins during redirects.
- DRM and access-control circumvention is out of scope.

Please report vulnerabilities privately to the repository owner rather than opening a public exploit report.
