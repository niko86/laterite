"""docs/parity-coverage-map.md restates parity-known-failures.json — hold them level.

The #950 class, third sighting (week-35 curation): the page hand-restates the
parity contract, and the O-47 entry moved under it — the floor read 122 against
the script's 121, the "10 deliberate non-closures" table held 9 rows, and a
module bucket said 30/30 while `test_version` sat in the known-failures set.
Three findings, one root cause: a doc copying a machine-readable file by hand.

The JSON is the authority (`tools/check_parity.py` enforces it BY IDENTITY in
CI), so every number and row the page restates is derivable:

  * the headline `X / Y` and its percentage — Y is the JSON's `total_tests`,
    X is Y minus the known-failure count;
  * "The N deliberate non-closures" heading and the row set of the table under
    it — the failures themselves, compared by test name, not by count;
  * the "X passed, N failed" expected-result line;
  * the python-ags4 version the page anchors against.

A parse miss is a FINDING, not a silent pass: if the page (or this gate) is
reworded until an anchor can't be found, the run says which anchor died
rather than greening over a page it no longer reads.

NOT judged, and said so on every run: the per-row reason prose (editorial,
richer than the JSON's `reason` fields) and the approximate per-module bucket
counts (their denominators live in python-ags4's suite, not in this repo).
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "docs" / "parity-coverage-map.md"
FAILURES = ROOT / "parity-known-failures.json"


def short_names(failures: dict) -> set[str]:
    """The bare test name after `::` — the table's spelling.

    The JSON's keys are full ids, e.g. `tests/test_check.py::test_rule_2`
    -> `test_rule_2` — files in python-ags4's own suite, not in this repo.
    """
    return {k.split("::")[-1] for k in failures["known_failures"]}


def table_rows(doc: str) -> tuple[set[str], int]:
    """(backticked test names in the non-closures table, reason cells skipped).

    Rows are read from the heading to the next `##`; the first backticked
    token of a `|` row is the test, the rest of the row is editorial and
    counted as skipped rather than judged.
    """
    m = re.search(
        r"^## The (\d+) deliberate non-closures$(.*?)(?=^## )", doc, re.M | re.S
    )
    if not m:
        return set(), 0
    names: set[str] = set()
    skipped = 0
    for line in m.group(2).splitlines():
        row = re.match(r"\s*\|\s*`([^`]+)`\s*\|(.*)", line)
        if row:
            names.add(row.group(1))
            skipped += row.group(2).count("|")
    return names, skipped


def check(doc: str, failures: dict) -> tuple[list[str], list[str]]:
    """(findings, skip notes) — pure, so the tests can feed doctored pairs."""
    findings: list[str] = []
    notes: list[str] = []
    expected = short_names(failures)
    total = failures["total_tests"]
    n_fail = len(failures["known_failures"])
    passing = total - n_fail

    m = re.search(r"^## The (\d+) deliberate non-closures$", doc, re.M)
    if not m:
        findings.append(
            "anchor missing: no '## The N deliberate non-closures' heading — "
            "the page or this gate moved; update whichever did"
        )
    elif int(m.group(1)) != n_fail:
        findings.append(
            f"heading says {m.group(1)} deliberate non-closures; "
            f"parity-known-failures.json holds {n_fail}"
        )

    rows, skipped_cells = table_rows(doc)
    if not rows:
        findings.append(
            "anchor missing: no rows under the non-closures heading — "
            "the page or this gate moved; update whichever did"
        )
    else:
        findings.extend(
            f"table is missing `{name}` (in parity-known-failures.json)"
            for name in sorted(expected - rows)
        )
        findings.extend(
            f"table row `{name}` is not in parity-known-failures.json"
            for name in sorted(rows - expected)
        )
        notes.append(
            f"{skipped_cells} reason cell(s) in the table are editorial and NOT judged"
        )

    m = re.search(r"\*\*(\d+) / (\d+)[^*]*\((\d+)%\)", doc)
    if not m:
        findings.append("anchor missing: no '**X / Y … (P%)' headline")
    else:
        x, y, pct = (int(g) for g in m.groups())
        if (x, y) != (passing, total):
            findings.append(
                f"headline says {x} / {y}; the JSON derives {passing} / {total}"
            )
        if pct != round(100 * x / y):
            findings.append(f"headline percentage {pct}% is not {x}/{y} rounded")

    m = re.search(r"remaining (\d+) are deliberate", doc)
    if m and int(m.group(1)) != n_fail:
        findings.append(
            f"'remaining {m.group(1)}' contradicts the JSON's {n_fail} known failures"
        )

    m = re.search(r"\*\*(\d+) passed, (\d+) failed\*\*", doc)
    if not m:
        findings.append(
            "anchor missing: no '**X passed, N failed**' expected-result line"
        )
    elif (int(m.group(1)), int(m.group(2))) != (passing, n_fail):
        findings.append(
            f"expected-result line says {m.group(1)} passed, {m.group(2)} failed; "
            f"the JSON derives {passing} passed, {n_fail} failed"
        )

    version = failures["python_ags4_version"]
    if f"python-ags4 {version}" not in doc and f"**{version}**" not in doc:
        findings.append(
            f"the page never anchors against python-ags4 {version} "
            f"(the JSON's python_ags4_version)"
        )

    notes.append(
        "per-module bucket counts are approximate (denominators live in "
        "python-ags4's suite) and NOT judged"
    )
    return findings, notes


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("--doc", type=Path, default=DOC)
    ap.add_argument("--failures", type=Path, default=FAILURES)
    args = ap.parse_args()

    findings, notes = check(
        args.doc.read_text(encoding="utf-8"),
        json.loads(args.failures.read_text(encoding="utf-8")),
    )
    for n in notes:
        print(f"check_parity_coverage_map: {n}")
    for f in findings:
        print(f"  - {f}")
    if findings:
        print("check_parity_coverage_map: FAIL")
        return 1
    print(
        "check_parity_coverage_map: OK — the page's restated numbers and row "
        "set match parity-known-failures.json"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
