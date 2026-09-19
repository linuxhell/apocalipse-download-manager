# NetTrace RS 0.1.0

Portable Windows x64 network monitor written in Rust.

## Forced-family targets

- `ipv4:google.com` resolves/monitors Google using IPv4 only.
- `ipv6:google.com` resolves/monitors Google using IPv6 only.
- `google.com` defaults to IPv4.
- The **IPv4 + IPv6** button creates both monitors side by side.

## Included in 0.1.0

- Continuous 1-second probes
- Forced IPv4 / forced IPv6
- Live latency graph
- Min/average/max latency
- Packet loss
- Jitter
- Current resolved IP
- On-demand traceroute forced to the same IP family
- Multiple simultaneous targets
- Portable target persistence in `data/targets.txt`
- Dark native desktop UI
- No installer

This first build uses the Windows `ping.exe` and `tracert.exe` networking tools as subprocesses while all monitoring, statistics, persistence and UI orchestration are implemented in Rust. This avoids requiring administrator/raw-socket privileges and makes IPv4/IPv6 behavior predictable on stock Windows systems.
