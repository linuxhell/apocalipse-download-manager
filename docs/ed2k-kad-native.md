# Apocalipse ED2K/Kad — native alpha integration

The sidebar entry directly below Apocalipse Link opens a separate, reusable
Tauri window. Its UI follows the saved ADM language (en, pt-BR, zh-CN), including
changes while open, and the selected colour mode. The layout takes inspiration
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
- No incoming peer listener exists yet. Server login currently uses port zero
  and may receive Low ID or be rejected. The UI explicitly displays Low ID.
- Kad currently implements a UDP bootstrap probe that validates a responding
  node and returns contacts. It **does not** keep a routing table/session active,
  perform Kad lookup/publishing or download through Kad. The UI labels this as
  bootstrap rather than a connected Kad session.
- Peer downloads/uploads, queues, shared files, credits, AICH, obfuscation,
  messaging and full aMule feature parity remain required work. Search results
  are not presented as downloadable tasks in this alpha.

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

- 328 Node regression/behaviour tests pass, including live language changes,
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
