#!/usr/bin/env python3
"""Hold every prose speed claim to the benchmark table it summarises.

`tools/bench-vs-python-ags4.py` generates the speedup tables in
`packages/laterite/README.md`: it verifies each fixture against a pinned
SHA-256 and prints the rungs. That much was already rigorous. What had no link
was the prose that QUOTES those tables — three docs-site pages and a
`pyproject.toml` comment each restate a multiple by hand, and nothing read the
table when either side moved.

So re-running the benchmark after a perf pass updated the README and silently
falsified four other files. That is exactly how #326's claim came to sit below
every rung of its own table for about a month, on the first page a python-ags4
user reads. #326 corrected the value; this closes the blind spot that let it
stand — the same split as #316 -> #323, and the same shape as
`check_support_matrix.py` next to it in ci.yml.

The failure is one-directional in the worst way. A table moving UP leaves the
prose understating the product, and nobody files a bug about being too modest.
So this is deliberately BIDIRECTIONAL: a claim outside its table's measured
band fails whichever side it falls.

## Why a band and not equality

The predicate is "the prose must not misdescribe the table", NOT "the prose
must equal the table". A quoted multiple is a summary of five rungs that
genuinely differ, and the benchmark is an instrument with run-to-run drift. An
equality check would flake on noise and get switched off, which leaves the
prose staler than having no gate at all. Membership of [min rung, max rung] is
the assertion that survives re-measurement.

This is also what keeps the multiples legal in prose under the house rule
against writing measured values there: they may stay ONLY because this gate
reads them. Cite this script when explaining why -- never the numbers, which
are the table's to state.

## Which table a claim refers to

"~3x faster" is the compat table; validation and read->typed are different
axes with different rungs, so a claim that names none of them cannot be
checked. Rather than guess, this requires an axis marker near the claim
(`compat`/`AGS4_to_dataframe`, `laterite.validate`/`check_file`,
`laterite.read`/`convert_to_numeric`) and fails asking for one when absent.
Requiring the axis is better writing anyway.

## The wiki is deliberately out of scope

`ags-wiki/` quotes its own multiple from `bench_compat_dataframe.py`, a
different fixture in the dev satellite -- a different measurement, not a stale
digit. Sweeping it into this range check would assert that two benchmarks must
agree, which is false. #326 made each name its fixture; this scans the
reader-facing set only.

Usage:
    uv run --no-sync python tools/check_speed_claims.py

Exit 0 when every claim sits inside the band of the table it names, 1 otherwise.
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
TABLES = ROOT / "packages" / "laterite" / "README.md"

# Reader-facing only. The wiki is excluded on purpose (see the module docstring).
SCANNED = [
    ROOT / "README.md",
    ROOT / "packages" / "laterite" / "README.md",
    ROOT / "packages" / "laterite" / "pyproject.toml",
    ROOT / "rust-packages" / "laterite-node" / "README.md",
    *sorted((ROOT / "web" / "docs-site" / "docs").rglob("*.md")),
]

# The axis is named by the table header's own third column — the API being
# measured IS the axis, so there is nothing to keep in sync here.
AXES = {
    "compat": ("laterite.compat", ("compat", "AGS4_to_dataframe")),
    "validation": ("laterite.validate", ("laterite.validate", "check_file")),
    "read-typed": ("laterite.read", ("laterite.read", "convert_to_numeric")),
}

# `3×` in the docs, `3x` in a TOML comment — both spellings are in the tree.
CLAIM_RE = re.compile(
    r"~?\s*(\d+(?:\.\d+)?)\s*[×x]\s*\*{0,2}\s*faster\s+than\s+python-ags4"
)
# The #826 promotion's memory twin: "~2× less (peak) memory than python-ags4".
# Banded against the peak-RSS tables the same way time claims are banded
# against the time tables.
CLAIM_MEM_RE = re.compile(
    r"~?\s*(\d+(?:\.\d+)?)\s*[×x]\s*\*{0,2}\s*less\s+(?:peak\s+)?memory\s+"
    r"than\s+python-ags4"
)
RUNG_RE = re.compile(r"\*\*(\d+(?:\.\d+)?)×\*\*")
# What tells a memory table's header row from the time table sharing the same
# API name — `bench-vs-python-ags4.py` prints it into every memory header.
MEM_MARKER = "peak RSS"
# How far from the claim an axis marker may sit. The claims in the tree today
# carry theirs within a sentence or two; a wider window starts finding the
# wrong axis in a neighbouring paragraph.
WINDOW = 5


def _fail(msg: str) -> None:
    print(f"[speed-claims] FAIL: {msg}", file=sys.stderr)


def read_bands(text: str, memory: bool = False) -> dict[str, tuple[float, float]]:
    """Each table's measured band, keyed by axis.

    A markdown table is found by its HEADER row naming the API, then its rungs
    are the bolded multiples in the rows below it until the table ends. The
    time and memory tables name the same APIs, so the MEM_MARKER in the header
    is what separates them — `memory=True` reads the peak-RSS tables, the
    default reads the time tables and skips the memory ones.
    """
    lines = text.splitlines()
    bands: dict[str, tuple[float, float]] = {}
    for axis, (header_token, _) in AXES.items():
        rungs: list[float] = []
        for i, line in enumerate(lines):
            if not (line.startswith("|") and header_token in line):
                continue
            if (MEM_MARKER in line) != memory:
                continue
            for row in lines[i + 1 :]:
                if not row.startswith("|"):
                    break
                rungs.extend(float(m) for m in RUNG_RE.findall(row))
            break
        if rungs:
            bands[axis] = (min(rungs), max(rungs))
    return bands


def axis_for(lines: list[str], idx: int) -> str | None:
    """The axis a claim at `idx` names, or None when it names none."""
    lo, hi = max(0, idx - WINDOW), min(len(lines), idx + WINDOW + 1)
    context = "\n".join(lines[lo:hi])
    hits = {
        axis
        for axis, (_, markers) in AXES.items()
        if any(marker in context for marker in markers)
    }
    return hits.pop() if len(hits) == 1 else None


def main() -> int:
    if not TABLES.is_file():
        _fail(f"no benchmark tables at {TABLES.relative_to(ROOT)}")
        return 1
    tables_text = TABLES.read_text(encoding="utf-8")
    bands = read_bands(tables_text)
    mem_bands = read_bands(tables_text, memory=True)
    missing = set(AXES) - set(bands)
    if missing:
        _fail(
            f"{TABLES.relative_to(ROOT)} has no speedup table for "
            f"{', '.join(sorted(missing))} — the tables moved or were renamed, "
            f"so this gate is checking prose against nothing. Re-run "
            f"tools/bench-vs-python-ags4.py and update the axis headers here."
        )
        return 1
    # The memory tables (the #826 promotion) are checked the same way, but
    # their absence only fails once a prose claim needs them — the memory
    # claim vocabulary may legitimately go unused.

    ok = True
    checked = 0
    mem_checked = 0
    for path in SCANNED:
        if not path.is_file():
            continue
        text = path.read_text(encoding="utf-8")
        lines = text.splitlines()
        # Matched over the WHOLE text, not line by line: prose wraps, and
        # `cookbook/compat.md` splits one of these claims across a line break
        # mid-phrase. A per-line scan silently skipped it — a gate that misses
        # the claim it exists to check passes for the worst possible reason.
        for kind, regex, kind_bands in (
            ("speed", CLAIM_RE, bands),
            ("memory", CLAIM_MEM_RE, mem_bands),
        ):
            for m in regex.finditer(text):
                i = text.count("\n", 0, m.start())
                if kind == "memory":
                    mem_checked += 1
                else:
                    checked += 1
                where = f"{path.relative_to(ROOT)}:{i + 1}"
                claimed = float(m.group(1))
                axis = axis_for(lines, i)
                if axis is None:
                    _fail(
                        f"{where} claims {m.group(0).strip()} without naming "
                        f"which benchmark it summarises. Name the axis near the "
                        f"claim (compat / laterite.validate / laterite.read) so "
                        f"it can be checked — the tables measure different "
                        f"things."
                    )
                    ok = False
                    continue
                if axis not in kind_bands:
                    _fail(
                        f"{where} makes a {kind} claim on the {axis} axis but "
                        f"{TABLES.relative_to(ROOT)} has no {kind} table for it "
                        f"— re-run tools/bench-vs-python-ags4.py and paste the "
                        f"table the claim summarises."
                    )
                    ok = False
                    continue
                lo, hi = kind_bands[axis]
                if not lo <= claimed <= hi:
                    side = "above" if claimed > hi else "below"
                    _fail(
                        f"{where} claims {claimed:g}× ({kind}) on the {axis} "
                        f"axis, {side} the {lo:g}–{hi:g}× the table in "
                        f"{TABLES.relative_to(ROOT)} measures. Quote a multiple "
                        f"inside the band, or re-run "
                        f"tools/bench-vs-python-ags4.py if the table is the "
                        f"stale one."
                    )
                    ok = False

    if not checked:
        _fail(
            "no speed claims found in the reader-facing set — either the claims "
            "were reworded past this gate's pattern (`~N× faster than "
            "python-ags4`) or the scanned paths moved. A gate that checks "
            "nothing passes for the wrong reason."
        )
        return 1

    if ok:
        print(
            f"[speed-claims] OK: {checked} speed claim(s) across "
            f"{len(bands)} benchmark axes, each inside its table's band; "
            f"{mem_checked} memory claim(s) against {len(mem_bands)} peak-RSS "
            f"table(s). Zero memory claims is fine — zero SCANNED claims "
            f"overall is not, and fails above."
        )
    return 0 if ok else 1


if __name__ == "__main__":
    raise SystemExit(main())
