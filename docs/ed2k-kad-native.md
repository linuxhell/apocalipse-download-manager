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
- Kad runs as a real session ("Connect Kad"): the Nibiru module keeps a routing table, verifies
  contacts, and looks up sources for a file hash. The window shows UDP port, contacts and verified
  contacts. Kad keyword search and publishing are **not** implemented, and reachability from the
  public network is not verified.
- Direct high-ID peer downloads have a persistent transfer queue, pause/resume,
  root-checked ED2K hashsets, verified parts, 32/64-bit offsets and compressed
  block support. Unfinished tasks reopen paused. Resume rehashes saved data.
  The ADM download directory is used; existing destinations are preserved.
- Credits, AICH, messaging and full aMule feature parity remain required work.

An ED2K initialization error disables this feature rather than stopping the
main ADM from opening. Existing HTTP, media, torrent and Link engines are not
replaced by this integration. No release publication or main-branch merge is
part of this draft.

## Reference and seed provenance

Latest stable observed 2026-10-19:
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

- 332 Node regression/behaviour tests pass, including live language changes,
  metadata URL rejection, native command routing, and text-only rendering of
  untrusted server/result names.
- A Chromium render at 1280x820 was visually inspected in Portuguese; light/dark
  captures and language switching were exercised. Bridge responses in this UI
  check are fixtures, not evidence of public ED2K connectivity.
- Nibiru's local TCP/UDP fixtures exercise ED2K login/search and Kad bootstrap
  wire packets. FTP/FTPS/SFTP/HTTP3 transfers are verified separately in Nibiru.
- Local desktop compilation requires GTK/WebKit/pkg-config, unavailable in the
  current container. CI Windows/Linux/macOS builds remain the acceptance gate
  for the Tauri integration and its backend unit tests.

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

## Shared files (choose what to share)

The **Shared files** tab works like aMule's: nothing is shared until the user adds it.

- **Choices:** "Add folder…" (with or without subfolders) and "Add files…". Each shared file has an
  upload priority (very low, low, normal, high, release), request/uploaded counters, a copy-link
  button and "Stop sharing". Stopping a file found through a folder excludes just that file; removing
  a folder stops sharing everything that came only from it. Choices, priorities and limits are
  stored in `shares.json` next to the ED2K identity and restored at launch.
- **Never shared:** hidden files and folders (any path component starting with a dot), empty files,
  symbolic links, and anything under ADM's own data folder. A folder that contains ADM's data
  (for example the whole home directory) or a drive root is refused.
- **Hashing:** ED2K part hashes are computed in the background with progress shown in the tab;
  hashes are cached (`share-cache.json`) so unchanged files are not hashed again after a restart.
- **Serving and publishing:** uploads are served by the incoming TCP listener opened when the user
  connects to a server, so files are available only while this window is connected. After connecting
  and after each change the list is offered to the server (`OP_OFFERFILES`). ED2K has no withdraw
  packet: removed files leave the server's index on the next connection.
- **Limits:** upload slots (1-20, default 3) and an upload limit in KiB/s.
- **Dependency:** `nibiru-xtreme` is pinned to `f8f65b2` (0.2.0-alpha.4), which contains the uploader,
  the share registry, the upload queue, the Kad session and the UPnP module.

Validation: Rust tests cover choosing folders and files, subfolders, hidden/empty/excluded files,
removal, priority restore, persistence and the cache; one test serves a chosen 300 KB file to a real
ED2K client over the listener (MD4-verified) and confirms an unchosen file is refused. UI tests cover
the tab in en, pt-BR and zh-CN, plain-text rendering of names and the command routing, and the tab was
rendered in Chromium in all three languages. **Not** verified: serving on the public ED2K network,
reachability through a NAT/firewall (a Low-ID client cannot be reached by others), compatibility
with eMule/aMule clients as downloaders, or the Tauri window itself (native folder and file pickers).

## Network: Kad session, UPnP and obfuscation

- **Kad session:** "Connect Kad" starts the Nibiru Kad session on the listener's real TCP port and a UDP
  port (4672 when free), seeded from `nodes.dat`; contacts are kept in `kad-contacts.json`. Sources for a
  download come from the connected server **and** Kad, a download can run with only one of them
  connected, and Low-ID sources are reached by a server callback, a Kad buddy callback, or both.
  "Find sources" in the search tab also adds the Kad sources.
- **UPnP:** on by default (Preferences → Network). While a server or Kad is connected it asks the router
  to forward the incoming TCP port and, with Kad, the Kad UDP port; it renews the lease, and removes the
  forwarding when you disconnect, turn the option off or close ADM. The toolbar shows the state (looking
  for a router, forwarded with the external IP, partly failed, no router found).
- **Obfuscation:** off / preferred (default) / required, applied to server connections, peer
  connections, the listener and Kad UDP. "Required" refuses peers that do not obfuscate.
- **Stored in** `prefs.json` next to the ED2K identity; a damaged file falls back to the defaults.

Validation: the UPnP module is tested in Nibiru against an in-process router that speaks SSDP, device
and service XML and SOAP (map, remove, renew, release, permanent-lease router, no router, untrusted
location). ADM tests cover which ports are forwarded and when, preference persistence, Kad starting on
the listener's port and stopping, and the window behaviour in en, pt-BR and zh-CN. **Not** verified: any
real router (models differ a lot; CGNAT cannot be fixed by UPnP), Kad on the public network, and the
Tauri window.
