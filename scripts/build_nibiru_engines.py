#!/usr/bin/env python3
"""Build the exact engine revisions recorded beside this script, with platform-native CI tools."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile


def run(args, **kwargs):
    return subprocess.run(args, check=True, **kwargs)


def copy_notices(source, destination):
    for path in source.iterdir():
        if path.is_file() and path.name.startswith(("LICENSE", "COPYRIGHT", "THIRD_PARTY", "NOTICE")):
            shutil.copy2(path, destination / path.name)
    # Power and Colapse preserve Rust notices in the packaging tree. A root
    # LICENSE glob alone silently drops them from the combined ADM package.
    notices = source / "packaging" / "licenses"
    if notices.is_dir():
        shutil.copytree(notices, destination / "licenses", dirs_exist_ok=True)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--require-all", action="store_true", help="Refuse a public release with any engine unavailable")
    parser.add_argument("--source-root", type=Path, help="Use local engine Git repositories, checking out the same pinned revisions")
    args = parser.parse_args()
    manifest = json.loads(Path(__file__).with_name("nibiru-engines.json").read_text())
    missing = [entry["name"] for entry in manifest["engines"] if entry["revision"] is None]
    if args.require_all and missing:
        raise SystemExit("Missing source revisions: " + ", ".join(missing))
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=True)
    records = []
    for entry in manifest["engines"]:
        name, revision = entry["name"], entry["revision"]
        if revision is None:
            if entry["required"] or args.require_all:
                raise SystemExit(f"Missing required source revision: {name}")
            records.append({"engine": name, "available": False, "reason": entry["reason"]})
            continue
        if len(revision) != 40 or any(c not in "0123456789abcdef" for c in revision):
            raise SystemExit(f"Unpinned source revision for {name}")
        with tempfile.TemporaryDirectory(prefix=f"{name}-build-") as scratch:
            source = Path(scratch) / "source"
            origin = entry["repository"]
            if args.source_root is not None:
                origin = str((args.source_root / name).resolve())
                if not Path(origin).is_dir():
                    raise SystemExit(f"Local source repository missing: {name}")
            run(["git", "init", str(source)])
            run(["git", "-C", str(source), "remote", "add", "origin", origin])
            run(["git", "-C", str(source), "fetch", "--depth=1", "origin", revision])
            run(["git", "-C", str(source), "checkout", "--detach", "FETCH_HEAD"])
            actual = subprocess.check_output(["git", "-C", str(source), "rev-parse", "HEAD"], text=True).strip()
            if actual != revision:
                raise SystemExit(f"Source revision mismatch for {name}")
            # Never reuse build paths owned by another checkout; Cargo fingerprints include source paths.
            env = dict(os.environ)
            env.pop("CARGO_TARGET_DIR", None)
            run(["cargo", "build", "--release", "--locked"], cwd=source, env=env)
            binary = source / "target" / "release" / (name + (".exe" if os.name == "nt" else ""))
            run([str(binary), "--version"])
            caps = json.loads(subprocess.check_output([str(binary), "capabilities", "--json"], text=True))
            if caps.get("engine") != name or caps.get("schema_version") != 1:
                raise SystemExit(f"Invalid native capabilities for {name}")
            destination = output / name
            destination.mkdir(exist_ok=True)
            shutil.copy2(binary, destination / binary.name)
            copy_notices(source, destination)
            records.append({"engine": name, "available": True, "revision": revision,
                            "sha256": hashlib.file_digest(binary.open("rb"), "sha256").hexdigest()})
    (output / "build.json").write_text(json.dumps(records, indent=2) + "\n")


if __name__ == "__main__":
    main()
