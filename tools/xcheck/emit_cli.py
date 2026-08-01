#!/usr/bin/env python3
"""The three `lat` LAUNCHER legs of the cross-surface OUTPUT-VALUE gate (plan
`output/output-value-gate-plan.md` §2).

Runs the native binary, uvx (`python -m laterite._cli`), and npx
(`node bin.mjs`) as **subprocesses in a fresh temp dir** — because the drift this
leg exists to catch (#3) is a *filesystem side-effect*: the NAME of the file
`lat fix` writes, which no existing gate compares. One dispatch table, three
`argv[0]`s; each writes its own observation file (`cli-native.json` /
`cli-uvx.json` / `cli-npx.json`) in the three-variant envelope `xcheck` reads.

A launcher whose executable is missing self-skips (writes no file); in CI
`xcheck --require-legs all` turns that skip into a failure.

    python tools/xcheck/emit_cli.py --out <dir> [--cases <dir>] [--repo-root <dir>]
"""

from __future__ import annotations

import argparse
import hashlib
import json
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path


def launchers(repo_root: Path) -> dict[str, list[str] | None]:
    """`argv[0..]` per launcher, or None when its executable is absent."""
    native = (repo_root / "rust-packages/target/release/lat").resolve()
    npx_bin = (repo_root / "rust-packages/laterite-node/bin.mjs").resolve()
    return {
        # The compiled Rust binary — the reference the launchers converge on.
        "cli-native": [str(native)] if native.exists() else None,
        # uvx: the SAME interpreter this script runs under (it has `laterite`
        # installed), driving the argparse launcher as a module.
        "cli-uvx": [sys.executable, "-m", "laterite._cli"],
        # npx: the thin `bin.mjs` shim over the tsup-built CLI.
        "cli-npx": ["node", str(npx_bin)] if npx_bin.exists() else None,
    }


def load_cases(cases_dir: Path) -> list[dict]:
    cases: list[dict] = []
    for path in sorted(cases_dir.glob("*.json")):
        cases.extend(json.loads(path.read_text())["cases"])
    return cases


#: Which `lat <verb>` each CLI op exercises. The authority for xcheck VERB
#: coverage (`tests/test_xcheck_verb_coverage.py`): a SUBCOMMAND is covered iff some
#: case uses an op listed here. Kept beside the op functions — each already hard-codes
#: its verb in the argv it builds — so the map cannot drift from what actually runs
#: (a test asserts every op in `observe()` appears here). Ops NOT in this map are
#: library-level (emit/build/reemit/excel-roundtrip), not a `lat <verb>` invocation,
#: so they contribute no verb coverage.
OP_VERBS: dict[str, str] = {
    "fix_dest": "fix",
    "fix_json": "fix",
    "validate_json": "validate",
    "read_json": "read",
    "read_csv": "read",
    "rules_json": "rules",
    "validate_ndjson": "validate",
    "diff_json": "diff",
    "merge_out": "merge",
    "pack_out": "pack",
    "unpack_roundtrip": "unpack",
}


def fix_dest(argv0: list[str], case: dict, repo_root: Path) -> dict:
    """Copy the fixture into a temp dir under the case's target name, run
    `fix <name>` with no --fix-out, and observe BOTH the CREATED filename AND the
    repaired CONTENT.

    `created` + `exit` stay verbatim — the created NAME is the whole trigger for
    drift #3 (a filesystem side-effect no other gate compares) and it CAUGHT that
    drift, so it is load-bearing and must not be replaced (#548). The addition is
    `repaired`: the bytes `lat fix` — the one verb that REWRITES the user's data —
    actually wrote, which no cross-launcher check has ever compared. The output is
    deterministic (the fixture carries a fixed TRAN date, no wall-clock injection),
    so a launcher whose repair diverges shows as a text split, not flakiness.

    Read by the OBSERVED name (`created[0]`), never an assumed one — the name is
    the very thing that drifts (#3), so the content is read from whatever file fix
    wrote; zero-or-many created files means there is no single repaired output to
    compare (the `created` split already tells that story) → `repaired` is None.
    `newline=""` keeps the AGS4 CRLF the emitter writes intact: universal-newline
    translation would fold ``\\r\\n``→``\\n`` and blind this leg to a line-ending
    drift BETWEEN launchers — the exact 'normalise for robustness and lose the
    drift you exist to catch' trap the xcheck comparator forbids."""
    fixture = (repo_root / case["input"]["fixture"]).resolve()
    name = case["input"]["as"]
    with tempfile.TemporaryDirectory() as tmp:
        d = Path(tmp)
        shutil.copyfile(fixture, d / name)
        r = subprocess.run(
            [*argv0, "fix", name],
            cwd=d,
            capture_output=True,
            text=True,
        )
        created = sorted(p.name for p in d.iterdir() if p.name != name)
        repaired = None
        if len(created) == 1:
            # `Path.read_text(newline=...)` is 3.13+-only (ty's 3.12 floor check for
            # the shipped wheel would flag it even though this dev-only script always
            # runs on the pinned 3.14 interpreter); `Path.open(newline=...)` has
            # always forwarded to `io.open`, so it's the portable way to keep CRLF
            # intact (see docstring).
            with (d / created[0]).open(encoding="utf-8", newline="") as f:
                repaired = f.read()
        return {"ok": {"created": created, "repaired": repaired, "exit": r.returncode}}


def fix_json(argv0: list[str], case: dict, repo_root: Path) -> dict:
    """`lat fix <name> --json` — the machine-readable repair report #545 gave the one
    verb that REWRITES the user's data. Before #545 `fix` ACCEPTED the (global) `--json`
    but ignored it — a silent no-op that fell through to the human summary; now it emits
    `{file, dest, applied, residual}`, a report hand-synced across the three launchers
    (native serde_json, uvx json.dumps, npx JSON.stringify). Three hand-written copies of
    a JSON shape is the exact drift shape this gate exists to pin, and this verb had NO
    output-value case until now — validate/read/diff/rules/merge did, fix did not.

    Run like `fix_dest`: copy the fixture into a temp dir under the case's name and invoke
    with a RELATIVE path, so the report's `file`/`dest` (all three emit the arg verbatim +
    a sibling, never a resolved absolute) are launcher-independent — each leg gets its own
    temp dir, so an absolute path would false-split. Raw stdout + exit is the value: a
    launcher whose report structure, key order, or formatting diverges shows as a split,
    and the exit code is part of the contract (0 clean · 1 residual)."""
    fixture = (repo_root / case["input"]["fixture"]).resolve()
    name = case["input"]["as"]
    with tempfile.TemporaryDirectory() as tmp:
        d = Path(tmp)
        shutil.copyfile(fixture, d / name)
        r = subprocess.run(
            [*argv0, "fix", name, "--json"],
            cwd=d,
            capture_output=True,
            text=True,
        )
        return {"ok": {"stdout": r.stdout, "exit": r.returncode}}


def validate_json(argv0: list[str], case: dict, repo_root: Path) -> dict:
    """`lat validate <fixture> --json` — the triplicated findings-JSON renderer
    (laterite-py/src/lib.rs, laterite-node/src/lib.rs "ported verbatim",
    render.rs "byte-identical" by hand-discipline). Store the RAW stdout so any
    drift — structural OR formatting — is a split; the exit code is part of the
    value (a launcher that renders findings but exits 0 is a bug). Run from the
    repo root with the repo-relative path so the render's `file` field is
    identical across launchers rather than an absolute machine path."""
    fixture = case["input"]["fixture"]
    r = subprocess.run(
        [*argv0, "validate", fixture, "--json"],
        cwd=repo_root,
        capture_output=True,
        text=True,
    )
    return {"ok": {"stdout": r.stdout, "exit": r.returncode}}


def read_render(argv0: list[str], case: dict, repo_root: Path) -> dict:
    """`lat read <fixture> [GROUP] --json|--csv` — the read renderers, which
    #530 family B converged onto core's `read_render` after they had been written
    three times in three languages with no gate over them at all.

    Captured as RAW BYTES, decoded strict UTF-8: what a launcher puts on stdout
    is the value, and a leg whose stdout is not UTF-8 (a mis-set stdout encoding
    mangling the non-ASCII cell) is a real drift — it must fail loudly here, not
    be normalised away by an `errors=` policy."""
    argv = [*argv0, "read", case["input"]["fixture"]]
    group = case["input"].get("group")
    if group is not None:
        argv.append(group)
    argv.append("--json" if case["op"] == "read_json" else "--csv")
    r = subprocess.run(argv, cwd=repo_root, capture_output=True)
    try:
        stdout = r.stdout.decode("utf-8")
    except UnicodeDecodeError:
        return {"err": "stdout is not utf-8"}
    return {"ok": {"stdout": stdout, "exit": r.returncode}}


def rules_json(argv0: list[str], case: dict, repo_root: Path) -> dict:
    """`lat rules --json` — the rules catalogue, documented byte-identical to the
    Rust binary across all three launchers. Takes no fixture. This is the verb that
    CRASHED on npx (#508: it iterated `{schema_version, rules: [...]}` as if it were
    an array, and only `--json`, the one path the tests then covered, worked). It had
    no xcheck case at all until #555 — the exact 'a verb every launcher HAS but one
    cannot RUN is invisible to a name-only diff' shape. Raw stdout + exit, so any
    drift (structure or formatting) is a split."""
    r = subprocess.run(
        [*argv0, "rules", "--json"],
        cwd=repo_root,
        capture_output=True,
        text=True,
    )
    return {"ok": {"stdout": r.stdout, "exit": r.returncode}}


def validate_ndjson(argv0: list[str], case: dict, repo_root: Path) -> dict:
    """`lat validate <fixture> --ndjson` — one flat JSON object per finding per line.
    This is the ONLY verb that honours `--ndjson`: it is a global flag every verb
    ACCEPTS but only validate READS, so on diff / read / merge / rules it is a silent
    no-op (falls through to the human render). `--ndjson` is a DIFFERENT renderer from
    `--json` (findings_ndjson, hand-synced across the three surfaces), so it is worth
    pinning cross-surface in its own right rather than assuming the `--json` case covers
    it. Raw stdout + exit: any structural or formatting drift is a split."""
    fixture = case["input"]["fixture"]
    r = subprocess.run(
        [*argv0, "validate", fixture, "--ndjson"],
        cwd=repo_root,
        capture_output=True,
        text=True,
    )
    return {"ok": {"stdout": r.stdout, "exit": r.returncode}}


def diff_json(argv0: list[str], case: dict, repo_root: Path) -> dict:
    """`lat diff <base> <rev> --json` — the KEY/type-aware delta, serialized. The
    JSON is pure group/row/cell structure (no file paths in the body), so it is
    byte-identical across launchers; run from the repo root with repo-relative
    inputs. Note `--ndjson` is a SILENT NO-OP on diff — a global output flag `diff`
    never reads, so it falls through to the human render (which differs per launcher
    by design). `--json` is therefore diff's only machine contract, and it is what
    this pins. Raw stdout + exit: any structural OR formatting drift is a split."""
    base = case["input"]["base"]
    rev = case["input"]["rev"]
    r = subprocess.run(
        [*argv0, "diff", base, rev, "--json"],
        cwd=repo_root,
        capture_output=True,
        text=True,
    )
    return {"ok": {"stdout": r.stdout, "exit": r.returncode}}


def merge_out(argv0: list[str], case: dict, repo_root: Path) -> dict:
    """`lat merge <files...> --out merged.ags` — the verb whose ABSENCE from npx
    STARTED this arc (#494/#508) and still had no cross-surface output-value case.
    Copy the inputs into a temp dir, merge to a sibling, and observe the merged
    BYTES (the AGS4 text) + exit. The merged content is deterministic — a synthesised
    TRAN carries the FIXED `1900-01-01` placeholder, not a wall-clock date, and no
    input path leaks into the body — so a launcher whose merge diverges shows here as
    a text split, not flakiness."""
    inputs = case["input"]["files"]
    with tempfile.TemporaryDirectory() as tmp:
        d = Path(tmp)
        names: list[str] = []
        for rel in inputs:
            shutil.copyfile((repo_root / rel).resolve(), d / Path(rel).name)
            names.append(Path(rel).name)
        argv = [*argv0, "merge", *names, "--out", "merged.ags"]
        clash = case["input"].get("on_type_clash")
        if clash is not None:
            argv += ["--on-type-clash", clash]
        r = subprocess.run(argv, cwd=d, capture_output=True, text=True)
        out = d / "merged.ags"
        merged = out.read_text(encoding="utf-8") if out.exists() else None
        return {"ok": {"merged": merged, "exit": r.returncode}}


def pack_out(argv0: list[str], case: dict, repo_root: Path) -> dict:
    """`lat pack <input> out.zst` — the zstd transport envelope. zstd is deterministic
    for a fixed input + level, so the packed bytes are identical across launchers;
    observe their SHA-256 + size (raw binary cannot live in the JSON observation) +
    exit. A launcher packing at a different level, or wrapping the payload differently,
    shows as a hash split."""
    src = (repo_root / case["input"]["fixture"]).resolve()
    with tempfile.TemporaryDirectory() as tmp:
        out = Path(tmp) / "packed.zst"
        r = subprocess.run(
            [*argv0, "pack", str(src), str(out)],
            cwd=tmp,
            capture_output=True,
            text=True,
        )
        blob = out.read_bytes() if out.exists() else b""
        return {
            "ok": {
                "sha256": hashlib.sha256(blob).hexdigest(),
                "size": len(blob),
                "exit": r.returncode,
            }
        }


def unpack_roundtrip(argv0: list[str], case: dict, repo_root: Path) -> dict:
    """`lat unpack <p.zst> out` — inverts `pack`. Pack the fixture first (setup, itself
    deterministic), then unpack and observe the RECOVERED text + exit. The value under
    test is unpack's output: a launcher whose unpack corrupts or truncates the payload
    shows as a text split against the others' faithful round-trip."""
    src = (repo_root / case["input"]["fixture"]).resolve()
    with tempfile.TemporaryDirectory() as tmp:
        d = Path(tmp)
        packed = d / "p.zst"
        out = d / "restored.ags"
        subprocess.run(
            [*argv0, "pack", str(src), str(packed)],
            cwd=d,
            capture_output=True,
            text=True,
        )
        r = subprocess.run(
            [*argv0, "unpack", str(packed), str(out)],
            cwd=d,
            capture_output=True,
            text=True,
        )
        recovered = out.read_text(encoding="utf-8") if out.exists() else None
        return {"ok": {"recovered": recovered, "exit": r.returncode}}


def observe(argv0: list[str], case: dict, repo_root: Path) -> dict | None:
    op = case["op"]
    if op == "fix_dest":
        return fix_dest(argv0, case, repo_root)
    if op == "fix_json":
        return fix_json(argv0, case, repo_root)
    if op == "validate_json":
        return validate_json(argv0, case, repo_root)
    if op in ("read_json", "read_csv"):
        return read_render(argv0, case, repo_root)
    if op == "rules_json":
        return rules_json(argv0, case, repo_root)
    if op == "validate_ndjson":
        return validate_ndjson(argv0, case, repo_root)
    if op == "diff_json":
        return diff_json(argv0, case, repo_root)
    if op == "merge_out":
        return merge_out(argv0, case, repo_root)
    if op == "pack_out":
        return pack_out(argv0, case, repo_root)
    if op == "unpack_roundtrip":
        return unpack_roundtrip(argv0, case, repo_root)
    return None


def engine_of(argv0: list[str], repo_root: Path) -> str | None:
    """The engine digest this launcher is ACTUALLY carrying, via `census`.

    Asked of the launcher rather than read from the tree, because that is the whole
    point: each of these three drives a BUILT artefact — a release binary, an
    installed wheel, a tsup dist — any of which can be stale while every case still
    matches, since a stale engine and a current one usually agree.

    `None` when the launcher cannot answer, which `xcheck` reports as an unchecked
    leg by name rather than as agreement. A launcher built before census schema 6
    lands here too, and that is correct: it has no engine to report, not an engine
    that happens to match.
    """
    try:
        out = subprocess.run(
            # Bare `census` — the dump IS JSON, and the native binary rejects a
            # `--json` the other two tolerate. Same argv `tools/gen_census.py` uses.
            [*argv0, "census"],
            capture_output=True,
            text=True,
            timeout=120,
            cwd=repo_root,
        )
    except (OSError, subprocess.TimeoutExpired):
        return None
    if out.returncode != 0 or not out.stdout.strip():
        return None
    try:
        census = json.loads(out.stdout)
    except json.JSONDecodeError:
        return None
    engine = census.get("engine")
    return engine if isinstance(engine, str) and engine else None


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--out", default="output/xcheck")
    ap.add_argument("--cases", default="rust-packages/laterite-ags4-xcheck/cases")
    ap.add_argument("--repo-root", default=".")
    args = ap.parse_args()

    repo_root = Path(args.repo_root)
    cases = load_cases(Path(args.cases))
    out_dir = Path(args.out)
    out_dir.mkdir(parents=True, exist_ok=True)

    for leg, argv0 in launchers(repo_root).items():
        if argv0 is None:
            print(f"{leg}: launcher unavailable — skipping", flush=True)
            continue
        engine = engine_of(argv0, repo_root)
        observations: dict[str, dict] = {}
        for case in cases:
            if leg not in case["legs"]:
                continue
            obs = observe(argv0, case, repo_root)
            if obs is not None:
                observations[case["id"]] = obs
        path = out_dir / f"{leg}.json"
        path.write_text(
            json.dumps(
                {"schema": 1, "leg": leg, "engine": engine, "cases": observations},
                indent=2,
            )
        )
        engine_note = engine or "NOT REPORTED"
        print(
            f"{leg}: {len(observations)} cases, engine {engine_note} -> {path}",
            flush=True,
        )


if __name__ == "__main__":
    main()
