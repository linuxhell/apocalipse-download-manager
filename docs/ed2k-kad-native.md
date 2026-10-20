# Apocalipse ED2K/Kad — native alpha integration

The sidebar entry directly below Apocalipse Link opens a separate, reusable
Tauri window. Its UI follows the saved ADM language (en, pt-BR, zh-CN), including
changes while open, and the selected colour mode. Opening it does not show or
resize the main window. The layout takes inspiration
from aMule's network/search/preferences organization and dense server tables.
It does not reuse aMule C++ code, launch aMule, or require an aMule daemon.

The backend embeds the native ED2K module from `linuxhell/nibiru-xtreme`, pinned
to an exact Git revision in Cargo.toml/Cargo.lock. Default Nibiru transport
features are disabled: Cargo metadata confirms no SSH or FTP stack is pulled
into this ED2K consumer. This is an integration foundation, not a finished aMule
replacement; performance/memory claims require measurements of the final build.

## Working surfaces and limits

- A seeded list is available offline at first launch: 16 ED2K servers and 192
  Kad2 nodes parsed by Nibiru from current metadata sources on 2026-10-19.
- HTTPS list update and native file import validate size, counts, types and
  truncation before persisting `server.met` or `nodes.dat`. Invalid updates keep
  the last good list. Credentials are not accepted in metadata URLs.
- ED2K connect waits for a real server ID acknowledgement, not merely TCP success.
  Automatic selection tries at most four initial servers. Disconnect invalidates
  in-flight attempts. Native server search displays file size, sources, hash and
  an ED2K link that can be copied.
- Connecting starts a bounded incoming TCP hello/probe listener (prefers port
  4662, falls back to an available port on collision). Login advertises that
  actual port. The server's assigned ID remains authoritative: a listener does
  not prove firewall/NAT reachability. The UI shows the port and handshake count.
- Kad now runs a persistent UDP session with a bounded 512-contact routing table,
  plain Kad2 HELLO, XOR-ordered iterative lookups, cancellation, periodic refresh
  and verified-contact persistence. Plain (`E4`) and zlib-packed (`E5`) UDP
  replies are supported. A matching HELLO introduces a candidate;
  only a separate random endpoint challenge can verify it. Stored contacts need
  that handshake again after restart. Direct high-ID sources discovered through
  Kad feed the download queue even without an ED2K server. This is not a full
  Kademlia routing zone: keyword search, publishing, obfuscation, firewall testing
  and buddy/UDP callbacks are not implemented. The UDP socket alone proves no
  public reachability. A diagnostic one-shot bootstrap command remains available.
- Direct high-ID peer downloads have a persistent transfer queue, pause/resume,
  root-checked ED2K hashsets, verified parts, 32/64-bit offsets and compressed
  block support. Unfinished tasks reopen paused. Resume rehashes saved data.
  The ADM download directory is used; existing destinations are preserved.
- **Sharing and uploads.** The *Shared* tab lists exactly what the user chose: single
  files or folders (optionally with subfolders), added through native pickers.
  Nothing is shared automatically. Choices are stored in `shared.json`; computed
  hashes are cached in `shared-cache.json` and reused only while path, size and
  modification time match and the hashes are internally consistent, so reopening
  does not rehash large libraries. Hashing runs in the background with progress,
  per-file problems are listed, and files already shared keep being served during a
  rescan. Each file has an upload priority (very low to release) and per-file
  request/upload statistics. Links can be copied and the containing folder opened;
  the folder is looked up by hash in the registry, never from a path sent by the UI.
- **Upload queue.** The *Uploads* tab configures simultaneous slots (1-64) and a total
  KiB/s limit, and shows active clients, the waiting queue (ordered by file priority,
  then waiting time) and the total uploaded. Clients that disconnect keep their place
  for 30 minutes.
- **Publishing.** After connecting, the shared list is offered to the server and
  offered again every 15 minutes. ED2K has no withdraw packet: files that were
  unshared leave the server index when the connection is renewed.
- Credits, AICH, obfuscation, messaging, Kad keyword search/publishing and full aMule feature
  parity remain required work.

An ED2K initialization error disables this feature rather than stopping the
main ADM from opening. Existing HTTP, media, torrent and Link engines are not
replaced by this integration. No release publication or main-branch merge is
part of this draft.

## Reference and seed provenance

Latest stable verified again on 2026-10-20:
[aMule 3.1.0](https://github.com/amule-org/amule/releases/tag/3.1.0),
commit `909d304d993ee07df6c6f6acf501a6d791d53666`.
The active repository is [amule-org/amule](https://github.com/amule-org/amule).
Reference paths include `src/Preferences.cpp`, `src/ServerList.cpp`,
`src/include/protocol`, `src/include/tags`, and
`src/kademlia/routing/RoutingZone.cpp`.

The seed sources match aMule 3.1.0 preferences. Both returned HTTPS 200:

| Source | Bytes | SHA-256 of downloaded binary |
| --- | ---: | --- |
| https://upd.emule-security.org/server.met | 2,952 | 932441d5c78c74014e8111aeb1268c12cacf37bcd5a1bcbfc3a09b114738d45f |
| https://upd.emule-security.org/nodes.dat | 6,540 | 66d33023e0f0fda090dae17a14c4a15b0a52bce0c3a24934daa633bc49becc81 |

`assets/ed2k-seeds.json` is the decoded output of Nibiru's `inspect_lists`
example for these exact samples. No assumption is made that all listed servers
or nodes remain reachable. Updating a list is distinct from connecting to peers.

## Validation

- 337 Node regression/behaviour tests pass, including live language changes,
  metadata URL rejection, native command routing, and text-only rendering of
  untrusted server/result names.
- A Chromium render at 1280x820 was visually inspected in Portuguese; light/dark
  captures and language switching were exercised. Bridge responses in this UI
  check are fixtures, not evidence of public ED2K connectivity.
- Nibiru's local TCP/UDP fixtures exercise ED2K login/search and Kad bootstrap
  wire packets. FTP/FTPS/SFTP/HTTP3 transfers are verified separately in Nibiru.
- This audit cross-compiled the complete desktop executable for Windows GNU x64.
  A Linux harness executes the actual production Kad, native RPC and CLI adapter
  modules without requiring GTK. This is not a Windows runtime test. Native
  Windows/Linux/macOS CI and public-network interoperability remain acceptance gates.
- Nibiru's real-socket tests cover Nibiru downloading from a Nibiru listener, queueing and
  promotion, cancellation, invalid ranges, a file changed on disk, the bandwidth limit and
  `OP_OFFERFILES` batches. Interoperability with eMule/aMule has not been tested yet.

## Window and source lookup update

The independent window opens maximized, including when reused, and uses the
saved ADM theme for both its content and native title bar. Theme events update
open content immediately. The extra header eyebrow was removed at user request.
Main surfaces are white in light mode and black in dark mode, using the shared
monochrome ADM palette for borders, tables and controls.

Search results can be filtered locally and sorted by source count, size or name.
Each result can query the connected server using native OP_GETSOURCES. Responses
are correlated by file hash, checked for exact bounded lengths and deduplicated.
Low IDs are explicitly marked as requiring callbacks and never displayed as IPs.
Source discovery also feeds the native direct-peer transfer queue. Native TCP fixture
coverage includes unrelated responses, malformed lengths, invalid hashes,
duplicate sources and request throttling. UI behaviour and live themes are tested.


## Verified peer transfers

Search results can be added directly to Transfers, or users can paste a standard
ED2K file link. Two tasks may run concurrently; each native transfer uses one
peer and bounded block requests, trying at most 16 direct sources per attempt.
Low-ID sources are never dialed as IP addresses. If the requester has a High-ID
server session and an incoming listener, it registers a bounded callback and
asks the server to arrange the peer connection. The incoming client ID must
match the requested source before it enters the verified transfer path. An incomplete
attempt keeps verified parts and can be resumed to refresh server sources.

Persisted task records are bounded and atomically saved. Partial files use a
hash/size filename and an OS file lock. Resume never trusts task progress alone:
it reads and hashes saved parts. A peer hashset must first match the requested
root hash. Files are synced and published without replacing an existing target.
Removing a record preserves completed and partial files. Queued work can be
paused, including during source lookup and peer network stalls.

Native local TCP fixtures cover multipart resume, compressed blocks split over
packets, raw fragments arriving out of order, corruption, hashset validation,
large offsets above 4 GiB, cancellation, and a finalization race with an existing
user file. UI tests cover add/resume/pause/remove command routing. Public-network
interoperability and uploads/Kad feature completeness remain separate gates.

An incoming TCP fixture covers hello identity and no-file responses. A server
callback fixture exercises OP_CALLBACKREQUEST, an unrelated incoming peer ID,
and a completed MD4-verified download from the matching Low-ID source. Pending
callback registrations are removed on cancellation/drop. This does not implement
Low-ID to Low-ID NAT traversal.

## Audit checkpoint, 2026-10-20

See [AUDITORIA-NIBIRU-ED2K-2026-10-20.md](AUDITORIA-NIBIRU-ED2K-2026-10-20.md)
for fixed integration defects, tested revisions and the remaining aMule gap.
The additions have real UDP fixtures for routing/source lookup, wrong-target
rejection, persisted contacts, and pending-query cleanup after cancellation.
The client remains experimental; this checkpoint is not a claim of full parity.

## Plain Kad2 HELLO and endpoint verification

Starting the persistent Kad session also starts or reuses the native incoming
TCP listener. `HELLO_REQ` (`0x11`) and `HELLO_RES` (`0x19`) report that listener's
actual bound TCP port, including its fallback port when 4662 is occupied. The
payload is the 16-byte node ID, little-endian TCP port, protocol version, tag
count and tags. The advertised Kad version is **5**. Advertising version 6 or
higher causes aMule to select obfuscated HELLO replies; version 8 also introduces
keyed HELLO acknowledgements, neither of which this socket implements.

HELLO parsing allows at most 512 payload bytes and 16 tags, checks truncation and
trailing bytes, rejects zero IDs/ports and duplicate control tags, and accepts
both named and compact integer tags. `TAG_KADMISCOPTIONS` (`F2`) bit 0 means UDP
firewalled: such a HELLO cannot admit or renew a verified routing contact. Bit 1
is TCP firewalled and does not exclude an otherwise reachable UDP router. A HELLO
response requiring the version-8 keyed ACK is unsupported. `TAG_SOURCEUPORT`
(`FC`) is an integer UDP port; it must match the datagram's observed source port.
This implementation does not follow a differing advertised NAT port, because a
HELLO has not yet proven authority to change the pending query's endpoint.

An outgoing handshake first requires a valid HELLO response from the exact IP
and UDP port, with the expected peer node ID. It then sends `KADEMLIA2_REQ`
(`0x21`), type 2, with a fresh random UUID-v4 target and the intended peer's node
ID. Only a strictly parsed `KADEMLIA2_RES` (`0x29`) from that endpoint echoing the
random target establishes verification. Ordinary routing and source replies use
predictable targets and never establish or extend this proof. The challenge
proves receipt at an endpoint; it is not cryptographic ownership of a node ID or
a test of this client's public reachability.

A peer performing its own legacy challenge must not wait for mutual verification:
requests addressed to our node ID from an unknown or expired peer get the echoed
target and an **empty contact list**. Only a currently verified endpoint can
receive routing contacts. Incoming HELLO packets enqueue candidates in the bounded
table; they do not create a task per packet. The existing worker refreshes at most
four contacts per 30-second pass. Queries retain a 32-slot global limit, one
pending query per endpoint, a three-second timeout and cleanup on cancellation.
Responses to incoming requests share a 20-per-second budget. Verification expires
after 20 minutes and is never loaded as trusted state from disk.

Kad's `E5` zlib framing is separate from obfuscation. The opcode remains outside
the compressed payload. Both encoded and decoded payloads are limited to 32 KiB;
the decoder uses a fixed output capacity, requires the end of a complete zlib
stream and rejects trailing bytes or concatenated streams. Untracked replies are
discarded before inflation, and incoming requests consume their rate budget before
inflation. The UDP receive buffer can hold a complete IPv4 datagram so an oversized
packet can be rejected without triggering Windows' short-buffer receive failure.
Recoverable receive errors, including a reset/refusal from an offline UDP peer,
use a 100 ms retry delay that can be cancelled immediately by shutdown. Structural
socket errors still stop the session.

Source discovery collects multiple `0x3b` datagrams from each queried peer. Every
packet must match the endpoint, peer node ID and file target; a callback-only
first batch does not prevent using a later batch of direct sources. Collection
stops after six valid datagrams, 300 raw source rows, 192 KiB of decoded payload,
or a fixed three-second window from the request, whichever comes first. These
budgets apply before unusable sources are filtered, and the channel itself has
six slots. Direct endpoints are deduplicated before returning them to downloads.
At most three peers are queried sequentially, so collecting their replies may
take up to nine seconds in addition to routing lookup. Background refresh yields
to foreground lookup/source collection instead of causing a busy contact to look
like an empty source result. Cancellation and timeout release the registration.

Local UDP fixtures cover the HELLO wire with a live TCP binding, malformed and
oversized tags, wrong IDs/endpoints/nonces, UDP-firewalled and keyed-ACK replies,
the empty unknown-peer challenge, expiry, persistence/restart, and cancellation
during HELLO, challenge and source collection. They also cover zlib corruption,
truncation, expansion limits and trailing data; large UDP packets; cancellable
receive backoff; first-batch callback sources followed by direct sources; source
packet/row limits; refresh contention; and MD4 byte order using a nonuniform hash.
Ordinary lookup success does not renew verification. These are independent packet fixtures, not
tests against a running aMule or the public Kad network. Protocol references are
aMule 3.1.0's `KademliaUDPListener.cpp` (`SendMyDetails`, `AddContact2`,
`Process2HelloRequest`, `Process2HelloResponse`, `ProcessKademlia2Request` and
`SendLegacyChallenge`), plus `include/tags/FileTags.h` and the Kad protocol
constant headers. Compression and source batching were checked against
`Packet.cpp`, `ClientUDPSocket.cpp` and `kademlia/kademlia/Indexed.cpp`.
The implementation uses no aMule C++ source.
