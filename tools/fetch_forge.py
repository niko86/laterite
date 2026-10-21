#!/usr/bin/env python3
"""Fetch the pinned `laterite-ags4-forge` release binary and print its path.

Forge — the synthetic AGS4 generator behind every pinned bench fixture — lives
in its own repo now (https://github.com/niko86/laterite-forge), and this repo
never builds it. Every workflow and tool here that needs it comes through this
one helper, so the version below is the ONLY place the choice is made. Bumping
it is a deliberate PR: a new forge may mint different bytes, and then the
fixture pins in `tools/readme-bench-fixtures.json` fail loudly rather than the
benches silently measuring other data.

The release asset is verified against that release's own `SHA256SUMS` before
it is unpacked, and the unpacked binary is cached under
`output/forge/<version>/<target>/` (gitignored), so a second call is offline.
A partial download never lands at the cached path: the binary is written
beside it and renamed into place.

Platforms are the release's three targets — Linux x86_64 (the CI pools),
macOS arm64 (local work) and Windows x86_64 (the hosted Windows perf legs).
Anything else is refused by name; there is no source-build fallback.

Usage:
    python tools/fetch_forge.py          # prints the binary's absolute path
    python tools/fetch_forge.py --force  # re-download even if cached

As a module: `fetch_forge.forge_path()` returns the same path.
"""

from __future__ import annotations

import argparse
import hashlib
import io
import platform
import subprocess
import sys
import tarfile
import zipfile
from pathlib import Path
from typing import NoReturn

#: The pinned forge release. One line, on purpose — see the module docstring.
FORGE_VERSION = "v0.1.0"

REPO_URL = "https://github.com/niko86/laterite-forge"
BIN = "laterite-ags4-forge"
CACHE = Path(__file__).resolve().parent.parent / "output" / "forge"

#: (platform.system(), platform.machine()) -> (Rust target, archive suffix).
TARGETS = {
    ("Linux", "x86_64"): ("x86_64-unknown-linux-gnu", ".tar.gz"),
    ("Darwin", "arm64"): ("aarch64-apple-darwin", ".tar.gz"),
    ("Windows", "AMD64"): ("x86_64-pc-windows-msvc", ".zip"),
}


def die(msg: str) -> NoReturn:
    print(f"fetch_forge: error: {msg}", file=sys.stderr)
    raise SystemExit(1)


def target() -> tuple[str, str]:
    key = (platform.system(), platform.machine())
    if key not in TARGETS:
        supported = ", ".join(f"{s}/{m}" for s, m in TARGETS)
        die(
            f"no forge {FORGE_VERSION} release binary for {key[0]}/{key[1]} "
            f"(published: {supported})"
        )
    return TARGETS[key]


def download(url: str) -> bytes:
    # Plain curl rather than urllib: every runner and the Mac carry it, and it
    # uses the system trust store, which a standalone Python build may not.
    # Public release assets — no token, no extra headers.
    try:
        proc = subprocess.run(
            ["curl", "-fsSL", "--retry", "3", url],
            check=False,
            capture_output=True,
        )
    except FileNotFoundError:
        die("curl is not on PATH")
    if proc.returncode != 0:
        die(f"curl failed ({proc.returncode}) on {url}: {proc.stderr.decode().strip()}")
    return proc.stdout


def expected_sha(sums: str, asset: str) -> str:
    for line in sums.splitlines():
        parts = line.split()
        if len(parts) == 2 and parts[1].lstrip("*") == asset:
            return parts[0].lower()
    die(f"{asset} is not listed in the {FORGE_VERSION} SHA256SUMS")


def extract(archive: bytes, suffix: str, name: str) -> bytes:
    if suffix == ".zip":
        with zipfile.ZipFile(io.BytesIO(archive)) as zf:
            for info in zf.infolist():
                if Path(info.filename).name == name:
                    return zf.read(info)
    else:
        with tarfile.open(fileobj=io.BytesIO(archive), mode="r:gz") as tf:
            for member in tf.getmembers():
                if member.isfile() and Path(member.name).name == name:
                    f = tf.extractfile(member)
                    if f is not None:
                        return f.read()
    die(f"{name} not found in the {FORGE_VERSION} archive")


def forge_path(force: bool = False) -> Path:
    """The pinned forge binary, downloaded and verified on first use."""
    triple, suffix = target()
    name = BIN + (".exe" if suffix == ".zip" else "")
    dest = CACHE / FORGE_VERSION / triple / name
    if dest.exists() and not force:
        return dest

    asset = f"{BIN}-{FORGE_VERSION}-{triple}{suffix}"
    base = f"{REPO_URL}/releases/download/{FORGE_VERSION}"
    print(f"fetch_forge: downloading {asset}", file=sys.stderr)
    sums = download(f"{base}/SHA256SUMS").decode("utf-8")
    archive = download(f"{base}/{asset}")
    want = expected_sha(sums, asset)
    got = hashlib.sha256(archive).hexdigest()
    if got != want:
        die(f"{asset} sha256 {got} does not match SHA256SUMS ({want})")

    dest.parent.mkdir(parents=True, exist_ok=True)
    partial = dest.with_name(dest.name + ".partial")
    partial.write_bytes(extract(archive, suffix, name))
    partial.chmod(0o755)
    partial.replace(dest)
    return dest


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("--force", action="store_true", help="re-download")
    args = ap.parse_args()
    print(forge_path(force=args.force))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
