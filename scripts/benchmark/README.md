# HTTP engine benchmark

This benchmark compares **Apocalipse Native HTTP**, **aria2c**, and **Hydra** with the same requested connection counts.

## Windows

Prerequisites:

- Rust toolchain / `cargo`
- `aria2c` available on PATH (or pass `-Aria2`)
- Hydra available on PATH (or pass `-Hydra`)

Example:

```powershell
powershell -ExecutionPolicy Bypass -File scripts/benchmark/http-engine.ps1 `
  -Url "https://example.org/1GB.bin" `
  -Connections 1,2,4,8,16 `
  -Repeats 3
```

The script writes raw files plus `http-engine-benchmark.csv` under `benchmark-output`.

## Fair-test rules

1. Use the exact same URL for all engines.
2. Run one client at a time.
3. Use at least three repetitions.
4. Keep connection counts matched.
5. Prefer a large static file (1 GiB or larger) served with HTTP byte ranges.
6. Verify hashes separately when benchmarking a public origin.
7. Compare both throughput and completion time; do not infer universal superiority from one origin.

The Apocalipse CLI benchmark command enables the native engine's adaptive admission. The requested connection count is a ceiling: it starts conservatively and admits additional workers only when measured marginal goodput is useful. Torrent and magnet downloads are not part of this benchmark and continue to use aria2c.
