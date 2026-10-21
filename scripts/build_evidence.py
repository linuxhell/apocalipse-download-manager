#!/usr/bin/env python3
"""Write build-evidence.json: which ADM commit and which engine revisions a package contains.

Fails (non-zero) instead of writing evidence that cannot be true: every engine
recorded by build_nibiru_engines.py must still match the pinned manifest and the
SHA-256 of the executable that is about to be packaged.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import subprocess
import sys
import tomllib

ROOT = Path(__file__).resolve().parent.parent


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1 << 20), b""):
            digest.update(chunk)
    return digest.hexdigest()


def engine_binary(engines: Path, name: str) -> Path:
    suffix = ".exe" if (engines / name / f"{name}.exe").is_file() else ""
    return engines / name / f"{name}{suffix}"


def build(adm_binary: Path, engines: Path, commit: str, source_root: Path = ROOT) -> dict:
    if len(commit) != 40 or any(c not in "0123456789abcdef" for c in commit.lower()):
        raise SystemExit("ADM commit must be a full 40-character SHA-1")
    manifest = json.loads((source_root / "scripts" / "nibiru-engines.json").read_text())
    pinned = {entry["name"]: entry["revision"] for entry in manifest["engines"]}
    built = {record["engine"]: record for record in json.loads((engines / "build.json").read_text())}
    records = []
    for name, revision in pinned.items():
        record = built.get(name)
        if not record or not record.get("available"):
            raise SystemExit(f"Engine {name} was not built")
        if record["revision"] != revision:
            raise SystemExit(f"Engine {name} was built from {record['revision']}, manifest pins {revision}")
        actual = sha256(engine_binary(engines, name))
        if actual != record["sha256"]:
            raise SystemExit(f"Engine {name} executable changed after the build ({actual} != {record['sha256']})")
        records.append({"engine": name, "revision": revision, "sha256": actual})
    workspace = tomllib.loads((source_root / "Cargo.toml").read_text())["workspace"]["package"]
    return {
        "schema_version": 1,
        "adm": {
            "repository": os.environ.get("GITHUB_REPOSITORY"),
            "commit": commit.lower(),
            "version": workspace["version"],
            "executable_sha256": sha256(adm_binary),
        },
        "engines": records,
        "builder": {
            "os": platform.system(),
            "machine": platform.machine(),
            "rustc": subprocess.run(["rustc", "--version"], capture_output=True, text=True).stdout.strip(),
            "run_id": os.environ.get("GITHUB_RUN_ID"),
        },
    }


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--adm-binary", type=Path, required=True)
    parser.add_argument("--engines", type=Path, required=True)
    parser.add_argument("--commit", required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    evidence = build(args.adm_binary, args.engines, args.commit)
    args.output.write_text(json.dumps(evidence, indent=2) + "\n", encoding="utf-8")
    print(json.dumps(evidence))


if __name__ == "__main__":
    main()
