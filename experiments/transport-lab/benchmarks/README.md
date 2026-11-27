# Independent-interface QUIC Link measurements

This is an opt-in experimental transport, not an Internet speed guarantee. Two
loopback addresses, two IPs on one NIC, or two adapters behind one saturated WAN
do not demonstrate bandwidth aggregation. Use two independent uplinks and a
remote host with enough egress bandwidth; record adapters, routes and gateways.
The core QUIC implementation selects paths and retransmits on surviving paths.
ADM now retries a failed additional source up to three times during transfer,
without blocking the payload stream, and reports per-path UDP counters and RTT.
Counters include framing/retransmission and must not be called file goodput.

Build `cargo build --release --manifest-path experiments/transport-lab/Cargo.toml`.
Keep the random 32+ character `ADM_LINK_LAB_TOKEN` environment variable private
and identical on both machines. Distribute the generated DER certificate through
a trusted channel. Never disable TLS verification or publish the token.

On the remote host (one selected test file, nine transfers for three rounds):

```
apocalipse-transport-lab serve-many 0.0.0.0:4433 test.bin trusted.der 9
```

On the client, substitute real assigned local IPv4 addresses and the remote IP:

```
python experiments/transport-lab/benchmarks/interfaces.py --lab target/release/apocalipse-transport-lab --remote SERVER_IP:4433 --certificate trusted.der --primary-ip PRIMARY_IP --secondary-ip SECONDARY_IP --networks "adapter names, distinct gateways and uplinks" --output results
```

The order rotates among primary-only, secondary-only and both paths. Every
completed output is independently checked for size and SHA-256 before deletion;
JSON reports persist all samples, wall times, retries and path counters. The
server source must remain unchanged. Run without competing downloads or builds.

Repeat in a separate run while manually disconnecting only the secondary uplink
mid-transfer. Reconnect it within the bounded retry window (about 30 seconds) to
measure recovery. Keep the primary working. Record disconnect/reconnect times
and whether any file transfer fails. To test primary-path loss, first confirm a
validated secondary path and connectivity to the same remote host; the QUIC
implementation must retain the transfer on the surviving route. Compare hashes
and wall times against stable-network runs. Do not claim success until this test
has actually run on independent physical networks.

Loopback transfer, invalid-interface fallback, cancellation, authentication and
no-overwrite tests run in CI. Independent WAN results are not available from the
single-network development container; this harness is provided to collect them.
