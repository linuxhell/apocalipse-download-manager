# Signed tool updates

The app downloads and runs third-party binaries (yt-dlp, FFmpeg, aria2,
N_m3u8DL-RE, QuickJS). Signed manifests let the maintainer pin exactly which
release assets are accepted.

## How it works

- `download_tool` and `update_tool` fetch `tools-manifest.json` from
  `TOOL_UPDATE_MANIFEST_URL` (latest ADM release asset).
- The manifest is an ed25519-signed `UpdateManifest`
  (`crates/apocalipse-core/src/signed_update.rs`). Verification checks the
  signature against `TRUSTED_TOOL_UPDATE_KEYS`, expiry, and an anti-rollback
  `sequence` (highest accepted value is stored in `tools/.update-manifest-sequence`).
- Each artifact entry has `target = "<tool>/<platform>-<arch>"`
  (e.g. `yt-dlp/windows-x86_64`), the asset `length` and its `sha256`. The hash
  is of the exact downloaded asset (the archive for zipped tools).
- Verification failure aborts the install (`tool_update_verification_failed:*`).
- With signing on, `yt-dlp -U` is not used: yt-dlp updates through the same
  verified download path.

While `TRUSTED_TOOL_UPDATE_KEYS` is empty nothing changes: the previous checks
(aria2's `.sha256` sidecar, `--version` validation) still apply.

## Enabling it

1. `cargo run -p apocalipse-core --example tools_manifest -- keygen`
   Keep the private seed offline. Put the public key in `TRUSTED_TOOL_UPDATE_KEYS`.
2. For each tool release you approve, download the asset and run
   `... -- hash <file>` to get `length` and `sha256`.
3. Write an unsigned manifest (`schema: 1`, increasing `sequence`, `expires_at`
   in epoch seconds, one `artifacts` entry per target) and sign it:
   `ADM_UPDATE_SIGNING_SEED=<seed> cargo run -p apocalipse-core --example tools_manifest -- sign unsigned.json > tools-manifest.json`
4. Upload `tools-manifest.json` to the ADM release.

Because manifests pin hashes, a tool only updates after the maintainer signs the
new version. Re-sign and bump `sequence` for each tool update you want to allow.
