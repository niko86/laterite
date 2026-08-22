#!/usr/bin/env python3
"""The launcher-contract CONTENT gate: the same facts from every `lat` launcher.

The tiered contract (`ags-wiki/design/dec-launcher-contract.md`, #542) frees each
launcher's *human* layout but binds its *content*: a reader must not learn less
from one launcher than another. Byte gates cannot hold that tier — the layouts
differ by design — so this gate parses each launcher's human output into a FACTS
dict and compares the dicts. It found real holes on day one: `validate` stated
the dictionary edition on npx only, and `npx diff` dropped the header, the
heading-only groups, the group add/remove lines and the totals.

Reuses `emit_cli.py`'s launcher resolution (the three-launcher subprocess
harness) rather than growing a second one. A launcher whose executable is absent
is REPORTED on every run — never silently skipped — and `--require-legs all`
(CI) turns absence into failure, mirroring `xcheck` itself.

Scope — printed on every run, because a filter nobody can see is a blind spot
with a green tick on it: this gate reads the human forms of `validate`,
`diff` and `fix` only. Machine forms (`--json`/`--ndjson`/`--csv`) are the byte-exact
tier, held by `emit_cli.py` + `xcheck`; the other verbs' human facts have no
recorded divergence and no extractor yet.

    python tools/xcheck/check_cli_content.py [--repo-root <dir>] [--require-legs all|present]
"""

from __future__ import annotations

import argparse
import re
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from emit_cli import launchers

_CLEAN = "rust-packages/laterite-ags4-validator/tests/fixtures/clean_minimal.ags"
_DIRTY = (
    "rust-packages/laterite-ags4-validator/tests/fixtures/rule8_dp_wrong_precision.ags"
)
_DIFF_A = "rust-packages/laterite-ags4-xcheck/cases/inputs/diff_base.ags"
#: Carries every diff fact class at once: a heading-only group (PROJ — the one
#: npx's `added || removed || changed` filter dropped), a row-changed group
#: (LOCA), and a whole group present on one side only (SAMP). Run reversed, the
#: same pair exercises `groups removed`.
_DIFF_B = "rust-packages/laterite-ags4-xcheck/cases/inputs/diff_rev_facts.ags"


# --- extractors --------------------------------------------------------------
# One per launcher family per verb. An extractor never guesses: a fact its
# layout should carry but doesn't comes back as None / absent, and the compare
# fails showing both dicts — that IS the finding.


def validate_facts_binary(stdout: str) -> dict:
    """The `lat` binary and the wheel launcher (byte-identical layouts):
    `<file>: clean (0 findings) — dictionary <ed> (<res>)` or
    `<file>: <n> finding(s) — dictionary <ed> (<res>)` + a table."""
    head = stdout.splitlines()[0] if stdout else ""
    m = re.search(r"— dictionary (\S+) \((\w+)\)", head)
    n = None
    if "clean (0 findings)" in head:
        n = 0
    elif cm := re.search(r": (\d+) finding\(s\)", head):
        n = int(cm.group(1))
    return {
        "edition": m.group(1) if m else None,
        "resolution": m.group(2) if m else None,
        "count": n,
    }


def validate_facts_npx(stdout: str) -> dict:
    """npx: `<file> — <ed> (<res>)` head, then `  clean — no findings` or
    `  <n> finding(s)` + finding lines."""
    lines = stdout.splitlines()
    head = lines[0] if lines else ""
    m = re.search(r" — (\S+) \((\w+)\)$", head)
    body = lines[1] if len(lines) > 1 else ""
    n = None
    if "clean — no findings" in body:
        n = 0
    elif cm := re.search(r"(\d+) finding\(s\)", body):
        n = int(cm.group(1))
    return {
        "edition": m.group(1) if m else None,
        "resolution": m.group(2) if m else None,
        "count": n,
    }


def diff_facts_binary(stdout: str) -> dict:
    """The binary / wheel layout: `a → b` header, `  CODE   +x -y ~z` per delta
    group, `  groups added:   …` / `  groups removed: …`, and
    `  total: +A added · −R removed · ~C changed` (U+2212 minus)."""
    facts: dict = {
        "header": None,
        "groups": {},
        "groups_added": [],
        "groups_removed": [],
        "total": None,
    }
    for line in stdout.splitlines():
        if m := re.fullmatch(r"(\S+) → (\S+)", line.strip()):
            if facts["header"] is None:
                facts["header"] = [m.group(1), m.group(2)]
        elif m := re.fullmatch(r"\s+(\S+)\s+\+(\d+) -(\d+) ~(\d+)", line):
            facts["groups"][m.group(1)] = [
                int(m.group(2)),
                int(m.group(3)),
                int(m.group(4)),
            ]
        elif m := re.match(r"\s+groups added:\s+(.*)", line):
            facts["groups_added"] = [g.strip() for g in m.group(1).split(",")]
        elif m := re.match(r"\s+groups removed:\s+(.*)", line):
            facts["groups_removed"] = [g.strip() for g in m.group(1).split(",")]
        elif m := re.match(
            r"\s+total: \+(\d+) added · −(\d+) removed · ~(\d+) changed", line
        ):
            facts["total"] = [int(m.group(1)), int(m.group(2)), int(m.group(3))]
    return facts


def diff_facts_npx(stdout: str) -> dict:
    """npx: `a → b` header, `CODE: +x -y ~z` per delta group,
    `groups added: …` / `groups removed: …`, `total: +A -R ~C`."""
    facts: dict = {
        "header": None,
        "groups": {},
        "groups_added": [],
        "groups_removed": [],
        "total": None,
    }
    for line in stdout.splitlines():
        # `total:` and the group lines share a `<word>: +a -r ~c` shape, so the
        # specific matches run first — a generic-first chain filed the totals
        # under a group named "total" on this gate's own first run.
        if m := re.fullmatch(r"(\S+) → (\S+)", line.strip()):
            if facts["header"] is None:
                facts["header"] = [m.group(1), m.group(2)]
        elif m := re.match(r"groups added: (.*)", line):
            facts["groups_added"] = [g.strip() for g in m.group(1).split(",")]
        elif m := re.match(r"groups removed: (.*)", line):
            facts["groups_removed"] = [g.strip() for g in m.group(1).split(",")]
        elif m := re.match(r"total: \+(\d+) -(\d+) ~(\d+)", line):
            facts["total"] = [int(m.group(1)), int(m.group(2)), int(m.group(3))]
        elif m := re.fullmatch(r"(\S+): \+(\d+) -(\d+) ~(\d+)", line.strip()):
            facts["groups"][m.group(1)] = [
                int(m.group(2)),
                int(m.group(3)),
                int(m.group(4)),
            ]
    return facts


def fix_facts_binary(stdout: str) -> dict:
    """The binary / wheel layout:
    `applied N fix(es) [kind, kind] → dest` + `dest: M finding(s) remain (…)`."""
    facts: dict = {"applied": None, "kinds": None, "dest": None, "residual": None}
    if m := re.search(r"applied (\d+) fix\(es\) \[(.*?)\] → (\S+)", stdout):
        facts["applied"] = int(m.group(1))
        facts["kinds"] = sorted(k.strip() for k in m.group(2).split(","))
        facts["dest"] = m.group(3)
    if m := re.search(r"(\d+) finding\(s\) remain", stdout):
        facts["residual"] = int(m.group(1))
    return facts


def fix_facts_npx(stdout: str) -> dict:
    """npx: `<FixResult N bytes, N fix(es) applied, M residual finding(s)>
    [kind, kind] → dest` — its own one-liner, same facts (#542)."""
    facts: dict = {"applied": None, "kinds": None, "dest": None, "residual": None}
    if m := re.search(
        r"(\d+) fix\(es\) applied, (\d+) residual finding\(s\)>"
        r"(?: \[(.*?)\])? → (\S+)",
        stdout,
    ):
        facts["applied"] = int(m.group(1))
        facts["residual"] = int(m.group(2))
        if m.group(3) is not None:
            facts["kinds"] = sorted(k.strip() for k in m.group(3).split(","))
        facts["dest"] = m.group(4)
    return facts


_EXTRACTORS = {
    # (check, launcher family) → extractor. cli-native and cli-uvx share a
    # deliberately byte-identical human layout (test_cli_human_table_rust_binary
    # _byte_parity), so they share extractors; npx's layout is its own.
    "validate": {
        "cli-native": validate_facts_binary,
        "cli-uvx": validate_facts_binary,
        "cli-npx": validate_facts_npx,
    },
    "diff": {
        "cli-native": diff_facts_binary,
        "cli-uvx": diff_facts_binary,
        "cli-npx": diff_facts_npx,
    },
    "fix": {
        "cli-native": fix_facts_binary,
        "cli-uvx": fix_facts_binary,
        "cli-npx": fix_facts_npx,
    },
}

#: Facts a launcher's output must actually CARRY, per family. Guards the gate's
#: own blind spot: if every leg failed the same way (a bad fixture path, say),
#: the extractors would all return all-None dicts that AGREE, and equality alone
#: would tick green over three broken runs. The contract says the facts must be
#: there, not merely equal.
REQUIRED_FACTS = {
    "validate": ("edition", "resolution", "count"),
    "diff": ("header", "total"),
    "fix": ("applied", "kinds", "dest", "residual"),
}

#: check name → (verb argv, which extractor family). Validate runs a clean and a
#: findings fixture; diff runs its fact-rich pair both ways so `groups added`
#: and `groups removed` are each exercised.
CHECKS: list[tuple[str, str, list[str]]] = [
    ("validate.clean", "validate", ["validate", _CLEAN]),
    ("validate.findings", "validate", ["validate", _DIRTY]),
    ("diff.facts", "diff", ["diff", _DIFF_A, _DIFF_B]),
    ("diff.facts_reversed", "diff", ["diff", _DIFF_B, _DIFF_A]),
    # fix WRITES a sibling file, so it runs in a per-leg temp dir (see main) —
    # the fixture has one mechanical fix and four residual findings, so every
    # fact is non-trivial.
    ("fix.result_line", "fix", ["fix", "rule8_dp_wrong_precision.ags"]),
]

#: Fixtures a check needs copied beside it when it runs in a temp dir (the ops
#: that write files); path is repo-relative, the argv above names the basename.
TEMPDIR_FIXTURES = {"fix.result_line": [_DIRTY]}


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument(
        "--repo-root", type=Path, default=Path(__file__).resolve().parents[2]
    )
    ap.add_argument("--require-legs", choices=["all", "present"], default="present")
    args = ap.parse_args()
    root = args.repo_root

    legs = launchers(root)
    present = {name: argv for name, argv in legs.items() if argv is not None}
    absent = sorted(set(legs) - set(present))
    # Reported every run, pass or fail — an unavailable leg is a scope cut.
    for name in absent:
        print(f"leg {name}: UNAVAILABLE (executable not built) — not compared")
    print(
        f"scope: human `validate` + `diff` + `fix` facts across {len(present)} leg(s); "
        "machine forms and the other verbs' human output are NOT examined here "
        "(the byte tier is emit_cli.py + xcheck's)"
    )

    failures = 0
    if args.require_legs == "all" and absent:
        print(f"FAIL: --require-legs all, but absent: {', '.join(absent)}")
        failures += 1

    for check_id, family, argv in CHECKS:
        got: dict[str, dict] = {}
        missing: list[str] = []
        for name, argv0 in present.items():
            if fixtures := TEMPDIR_FIXTURES.get(check_id):
                with tempfile.TemporaryDirectory() as tmp:
                    for rel in fixtures:
                        shutil.copyfile(root / rel, Path(tmp) / Path(rel).name)
                    r = subprocess.run(
                        [*argv0, *argv], cwd=tmp, capture_output=True, text=True
                    )
            else:
                r = subprocess.run(
                    [*argv0, *argv], cwd=root, capture_output=True, text=True
                )
            facts = _EXTRACTORS[family][name](r.stdout)
            # The exit code is a fact too — the launchers must agree on the verdict.
            facts["exit"] = r.returncode
            got[name] = facts
            missing.extend(
                f"{name}.{k}" for k in REQUIRED_FACTS[family] if facts[k] is None
            )
        if missing:
            failures += 1
            print(
                f"[no-facts] {check_id}: {', '.join(missing)} — "
                "output carried no such fact"
            )
            for name, facts in got.items():
                print(f"  {name}: {facts}")
            continue
        vals = list(got.values())
        if any(v != vals[0] for v in vals[1:]):
            failures += 1
            print(f"[content-split] {check_id}:")
            for name, facts in got.items():
                print(f"  {name}: {facts}")
        else:
            print(f"ok {check_id} ({len(got)} legs agree)")

    if failures:
        print(f"check_cli_content: {failures} FAILURE(S)")
        sys.exit(1)
    print("check_cli_content: OK")


if __name__ == "__main__":
    main()
