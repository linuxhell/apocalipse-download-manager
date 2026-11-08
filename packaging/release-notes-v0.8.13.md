# Apocalipse Download Manager 0.8.13

- Wait for bounded UPnP forwarding before each ED2K server login, including discovery retries after an earlier failure.
- If TCP forwarding becomes confirmed only after a Low-ID login, retry login once so the server can retest the incoming port. Preserve the real server ID, cancellation and server failover; never infer High ID from UPnP alone.
- Add Portuguese, English and Simplified Chinese recovery messages and regression coverage with local SSDP/SOAP and ED2K wire servers.
- Retain Nibiru Xtreme 0.2.5 and Chrome/Edge/Firefox 0.3.209, including the unchanged Mozilla-signed Firefox XPI.
