#!/usr/bin/env python3
"""Reproduce the abi3 cost measurement behind `ags-wiki/concepts/abi3-perf.md`.

That page answers two questions with numbers — *what does the limited API cost
laterite?* and *does a higher abi3 floor buy anything?* — and the answers drive a
shipping decision (one `cp312-abi3` wheel for 3.12+ rather than a per-version
matrix). The numbers were measured once, by hand, and written down. This script is
how they get re-measured, so the page can be refreshed instead of aged.

It is a REPRODUCER, not a CI gate. The absolute nanoseconds move with machine and
interpreter; only the ratios are durable, and a threshold on a ~5 ns delta would be
a flake generator. The regression that *is* worth gating — that the boundary stays
off the hot path — is already covered by the criterion benches and the wheel suite.

WHAT MAKES THIS HARD TO GET RIGHT. abi3 is not a cargo feature you can flip from
the command line: it is baked into `laterite-py`'s `pyo3` dependency line. So each
build rewrites that line, and the original bytes are restored unconditionally —
including on Ctrl-C and on a build failure. The script refuses to start if that
manifest already has uncommitted changes, so a crashed earlier run can never be
mistaken for the baseline and silently committed.

AND THE TRAP THE PAGE ALREADY DOCUMENTS. maturin 1.13.3 tags an `abi3-py314` build
`cp312-abi3` — the wheel FILENAME lies. Anything that identified a build by its
wheel tag would benchmark, and compare, three copies of what it believed were
different things. So the artifacts are identified by content: the extension
module's sha256, its size, and its undefined-symbol set. If any two of the three
builds hash the same, the run ABORTS — that is the failure mode this whole
measurement is vulnerable to, and it must never be reported as a result.

Run:
    uv run --no-sync python tools/bench-abi3.py
    uv run --no-sync python tools/bench-abi3.py --reps 15 --json out.json

Needs `maturin` and `uv` on PATH. Three release builds of the PyO3 crate; expect
several minutes and a few hundred MB in the shared workspace target dir.
"""

from __future__ import annotations

import argparse
import atexit
import hashlib
import json
import re
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
MANIFEST = ROOT / "rust-packages" / "laterite-py" / "Cargo.toml"
WHEEL_PROJECT = ROOT / "packages" / "laterite"

#: The pyo3 feature list per build. `extension-module` + `chrono` are constant —
#: only the ABI changes, which is the whole point: benchmarked on ONE interpreter,
#: the delta between these three is attributable to the limited API and nothing else.
CONFIGS: list[tuple[str, list[str]]] = [
    ("non-abi3", ["extension-module", "chrono"]),
    ("abi3-py312", ["extension-module", "chrono", "abi3-py312"]),
    ("abi3-py314", ["extension-module", "chrono", "abi3-py314"]),
]

PYO3_LINE_RE = re.compile(
    r'^(pyo3 = \{ version = "[^"]+", features = )\[[^\]]*\](\s*\})', re.M
)

# The workloads, run inside each build's own venv. Deliberately the PUBLIC import
# surface (`laterite.groups`, `laterite.validate`) rather than `_laterite_native`:
# what the page prices is the cost a USER pays at the boundary, and the public path
# is the one that carries the descriptors and kwargs parsing.
#
# They span the range that makes the finding interpretable — from a bare
# `#[pyclass]` alloc (where abi3's tp_alloc/tp_new penalty is maximal and nothing
# else happens) through to a real validation (where Rust work dominates and any
# boundary cost should vanish). A single workload could not distinguish "abi3 is
# cheap" from "we measured the wrong thing".
HARNESS = '''
import json
import sys
import time

import laterite
from laterite.groups import LOCA, PROJ

FIXTURE = sys.argv[1]
REPS = int(sys.argv[2])


def micro_construct(n):
    for _ in range(n):
        PROJ()


def micro_attr(n):
    p = PROJ()
    for _ in range(n):
        p.bench = 1
        p.bench


def typed_construct(n):
    for _ in range(n):
        LOCA(loca_id="BH01", loca_gl=12.5, loca_type="CP", loca_fdep=30.0)


def validate(n):
    for _ in range(n):
        laterite.validate(FIXTURE)


WORKLOADS = {
    "micro_construct": micro_construct,
    "micro_attr": micro_attr,
    "typed_construct": typed_construct,
    "validate": validate,
}

TARGET_NS = 50_000_000  # ~50 ms per rep


def timed(fn, n):
    t0 = time.perf_counter_ns()
    fn(n)
    return time.perf_counter_ns() - t0


def calibrate(fn):
    """Iterations that make one rep ~TARGET_NS.

    A fixed iteration count cannot serve both `micro_construct` (~50 ns) and
    `validate` (~100 us) — one would be swamped by clock granularity, the other
    would take minutes.
    """
    n = 1
    while n < 50_000_000:
        dt = timed(fn, n)
        if dt >= TARGET_NS:
            break
        n = max(n * 2, int(n * TARGET_NS / max(dt, 1)))
    return n


out = {}
for name, fn in WORKLOADS.items():
    n = calibrate(fn)
    # min, not mean: the noise is one-sided — scheduling and cache effects only
    # ever ADD time — so the minimum is the best estimator of the true cost.
    best = min(timed(fn, n) for _ in range(REPS))
    out[name] = {"ns_per_iter": best / n, "iters": n, "reps": REPS}

print(
    json.dumps(
        {
            "workloads": out,
            "python": sys.version.split()[0],
            "native": laterite._laterite_native.__file__,
        }
    )
)
'''


def run(cmd: list[str], **kw: object) -> subprocess.CompletedProcess[str]:
    return subprocess.run(cmd, check=True, text=True, capture_output=True, **kw)  # type: ignore[arg-type]


def set_features(features: list[str]) -> None:
    """Rewrite laterite-py's pyo3 feature list in place."""
    txt = MANIFEST.read_text(encoding="utf-8")
    feats = ", ".join(f'"{f}"' for f in features)
    new, n = PYO3_LINE_RE.subn(rf"\g<1>[{feats}]\g<2>", txt)
    if n != 1:
        sys.exit(
            f"bench-abi3: expected exactly one pyo3 dependency line in {MANIFEST}, matched {n}. "
            "The manifest's shape changed — update PYO3_LINE_RE."
        )
    MANIFEST.write_text(new, encoding="utf-8")


def fingerprint(so: Path) -> dict:
    """Identify a built extension by CONTENT, never by its wheel tag.

    maturin mis-tags abi3-py314 as `cp312-abi3`, so the filename cannot be trusted
    to tell two builds apart. sha256 + size settle it; the undefined-symbol set is
    the human-readable evidence of *why* they differ (which CPython entry points
    each build actually reaches for).
    """
    data = so.read_bytes()
    undef: list[str] = []
    if shutil.which("nm"):
        for flags in (["-u"], ["-D", "-u"]):  # macOS, then GNU binutils
            try:
                out = subprocess.run(
                    ["nm", *flags, str(so)], text=True, capture_output=True, check=True
                )
            except subprocess.CalledProcessError:
                continue
            undef = sorted(
                {ln.split()[-1] for ln in out.stdout.splitlines() if ln.strip()}
            )
            if undef:
                break
    return {
        "sha256": hashlib.sha256(data).hexdigest(),
        "size": len(data),
        "undefined_symbols": undef,
    }


def build(name: str, features: list[str], python: str, workdir: Path) -> dict:
    out_dir = workdir / name
    out_dir.mkdir(parents=True)
    set_features(features)
    print(f"  building {name} ({', '.join(features)}) …", flush=True)
    run(
        [
            "maturin",
            "build",
            "--release",
            "--interpreter",
            python,
            "--out",
            str(out_dir),
        ],
        cwd=WHEEL_PROJECT,
    )
    wheels = list(out_dir.glob("*.whl"))
    if len(wheels) != 1:
        sys.exit(f"bench-abi3: expected one wheel for {name}, got {wheels}")
    wheel = wheels[0]

    venv = workdir / f"venv-{name}"
    run(["uv", "venv", "--python", python, str(venv)])
    run(["uv", "pip", "install", "--python", str(venv / "bin" / "python"), str(wheel)])
    # `.so`/`.pyd` only — the same directory also holds the generated
    # `_laterite_native.pyi` stub, which is not the artifact being fingerprinted.
    sos = [
        p
        for p in (venv / "lib").glob("python*/site-packages/laterite/_laterite_native*")
        if p.suffix in {".so", ".pyd", ".dylib"}
    ]
    if len(sos) != 1:
        sys.exit(f"bench-abi3: expected one native module for {name}, got {sos}")

    return {
        "config": name,
        "features": features,
        "wheel_tag": wheel.name,
        "python": venv / "bin" / "python",
        **fingerprint(sos[0]),
    }


def bench(build_info: dict, harness: Path, fixture: Path, reps: int) -> dict:
    out = run(
        [str(build_info["python"]), str(harness), str(fixture), str(reps)],
    )
    return json.loads(out.stdout)


def main() -> None:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument(
        "--python",
        default="3.14",
        help="the ONE interpreter all three builds are benchmarked on (default 3.14)",
    )
    ap.add_argument("--reps", type=int, default=9, help="timed repetitions, min wins")
    ap.add_argument(
        "--fixture",
        type=Path,
        default=ROOT / "packages/laterite/tests/fixtures/multi_finding.ags",
        help="the file the `validate` workload runs on",
    )
    ap.add_argument(
        "--json", type=Path, default=None, help="also write raw results here"
    )
    ap.add_argument(
        "--keep",
        action="store_true",
        help="keep the wheels + venvs (they land in a temp dir otherwise)",
    )
    args = ap.parse_args()

    for tool in ("maturin", "uv"):
        if not shutil.which(tool):
            sys.exit(f"bench-abi3: {tool} is not on PATH")
    if not args.fixture.exists():
        sys.exit(f"bench-abi3: fixture {args.fixture} does not exist")

    # The manifest is rewritten three times. Refuse to start against a dirty one:
    # otherwise a crashed earlier run's half-applied edit becomes the "original"
    # this run restores, and an abi3 change lands in the tree unnoticed.
    dirty = subprocess.run(
        ["git", "status", "--porcelain", "--", str(MANIFEST)],
        cwd=ROOT,
        text=True,
        capture_output=True,
        check=True,
    ).stdout.strip()
    if dirty:
        sys.exit(
            f"bench-abi3: {MANIFEST.relative_to(ROOT)} has uncommitted changes — "
            "commit or stash them first (this script rewrites and restores that file)"
        )

    original = MANIFEST.read_bytes()

    def restore() -> None:
        if MANIFEST.read_bytes() != original:
            MANIFEST.write_bytes(original)
            print(f"bench-abi3: restored {MANIFEST.relative_to(ROOT)}")

    atexit.register(restore)

    tmp = Path(tempfile.mkdtemp(prefix="bench-abi3-"))
    print(f"bench-abi3: workdir {tmp}\n")
    try:
        harness = tmp / "harness.py"
        harness.write_text(HARNESS, encoding="utf-8")

        builds = [build(name, feats, args.python, tmp) for name, feats in CONFIGS]
    finally:
        restore()

    # The abort that protects every number below it.
    by_hash: dict[str, list[str]] = {}
    for b in builds:
        by_hash.setdefault(b["sha256"], []).append(b["config"])
    collisions = [v for v in by_hash.values() if len(v) > 1]
    if collisions:
        sys.exit(
            "bench-abi3: IDENTICAL ARTIFACTS — "
            + "; ".join(" == ".join(c) for c in collisions)
            + ".\nThe feature rewrite did not take effect (a stale target dir, or a "
            "manifest shape PYO3_LINE_RE no longer matches). Benchmarking these would "
            "compare a build against itself."
        )

    print("\nartifacts (identified by content — the wheel tag is not trustworthy):")
    for b in builds:
        print(
            f"  {b['config']:12} {b['sha256'][:16]}  {b['size']:>9,} B  {b['wheel_tag']}"
        )
    tags = {b["wheel_tag"] for b in builds}
    if len(tags) < len(builds):
        print(
            "\n  NOTE: two builds share a wheel TAG while differing in content — the\n"
            "  maturin mis-tagging abi3-perf.md documents is still present. An\n"
            "  abi3-py314 build is not safely shippable with this toolchain."
        )

    base_syms = set(builds[0]["undefined_symbols"])
    for b in builds[1:]:
        syms = set(b["undefined_symbols"])
        only_base, only_b = sorted(base_syms - syms), sorted(syms - base_syms)
        print(
            f"\n  {b['config']} vs {builds[0]['config']}: "
            f"{len(only_base)} symbol(s) dropped, {len(only_b)} gained"
        )
        for label, syms_ in (("  dropped", only_base), ("  gained", only_b)):
            if syms_:
                shown = ", ".join(syms_[:8]) + (" …" if len(syms_) > 8 else "")
                print(f"  {label}: {shown}")

    print(f"\nbenchmarking on Python {args.python}, best of {args.reps} …\n")
    results = {b["config"]: bench(b, harness, args.fixture, args.reps) for b in builds}

    names = [b["config"] for b in builds]
    workloads = list(results[names[0]]["workloads"])
    width = max(len(w) for w in workloads) + 2
    print(f"{'workload (ns/iter)':<{width}}" + "".join(f"{n:>14}" for n in names))
    for w in workloads:
        row = f"{w:<{width}}"
        for n in names:
            row += f"{results[n]['workloads'][w]['ns_per_iter']:>14,.1f}"
        print(row)

    print(
        f"\nratios vs {names[0]} (the durable finding; absolute ns are machine-bound):"
    )
    for w in workloads:
        base = results[names[0]]["workloads"][w]["ns_per_iter"]
        parts = [
            f"{n} {results[n]['workloads'][w]['ns_per_iter'] / base:.2f}x"
            for n in names[1:]
        ]
        print(f"  {w:<{width}}" + "  ".join(parts))

    if args.json:
        args.json.write_text(
            json.dumps(
                {
                    "python": args.python,
                    "reps": args.reps,
                    "fixture": str(args.fixture),
                    "builds": [
                        {
                            k: (str(v) if isinstance(v, Path) else v)
                            for k, v in b.items()
                        }
                        for b in builds
                    ],
                    "results": results,
                },
                indent=2,
            )
            + "\n"
        )
        print(f"\nwrote {args.json}")

    if args.keep:
        print(f"\nkept builds + venvs in {tmp}")
    else:
        shutil.rmtree(tmp, ignore_errors=True)


if __name__ == "__main__":
    main()
