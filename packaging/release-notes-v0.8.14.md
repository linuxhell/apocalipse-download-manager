# Apocalipse Download Manager 0.8.14

- Bundle Xtreme 0.2.6: announce the actual listening TCP port in incoming ED2K HELLOANSWER packets instead of zero.
- Discover UPnP gateways across local IPv4 addresses and retry lost SSDP requests within a shared bounded deadline; retain gateway URL trust checks.
- Separate accepted TCP connections from completed ED2K handshakes in the status header and login diagnostics. Low ID means ED2K access was not confirmed by the server; do not infer its precise cause from a TCP port test alone.
- Retain the bounded UPnP-before-login/relogin recovery from 0.8.13 and Chrome/Edge/Firefox 0.3.209, including the unchanged Mozilla-signed Firefox XPI.

Public-server High ID must be retested on the user's network; local protocol regressions do not establish that result.
