#!/usr/bin/env python3
"""Enforce the python-ags4 parity contract BY IDENTITY, not by count (laterite-dev#556).

The gate this replaces was:

    passed=$(grep -Eo '[0-9]+ passed' parity.log | tail -n1 | awk '{print $1}')
    if [ "$passed" -lt "$PARITY_MIN_PASSING" ]; then exit 1

A count is blind to a SWAP. One test regressing while another starts passing holds
at 121 and the job goes green — the regression is invisible precisely because
something else improved at the same time. The number also said nothing about WHICH
divergences we accept: the ten deliberate non-closures lived only in prose that no
job read, so the contract the CI enforced and the contract we'd actually agreed
were different objects, and nothing compared them. (laterite-dev#549's Shape 1, in its plainest
form.)

So: pin the failing SET. Three outcomes, and the third is the interesting one.

  a failure NOT in the fixture     -> regression. Red.
  a fixture entry that now PASSES  -> ALSO red, deliberately. A divergence closed
                                      and the record must follow; otherwise the
                                      fixture rots into the stale prose it replaced
                                      and starts excusing failures nobody chose.
  the sets match                   -> green.

`total_tests` is pinned too, guarding a failure mode this workflow has ALREADY
eaten: parity.yml's own comment records that a broken clone made pytest collect 0
tests, print "0 passed", and read as a false regression. The mirror-image — 0
collected, 0 failures, "no unexpected failures!" — would be a false CLEAN, which
is worse. An empty run must never look like a passing one.

Usage:
    python tools/check_parity.py parity.log            # enforce
    python tools/check_parity.py parity.log --write    # re-vendor after a
                                                       # DELIBERATE change
"""

from __future__ import annotations

import json
import re
import sys
from pathlib import Path

_REPO = Path(__file__).resolve().parents[1]
_FIXTURE = _REPO / "parity-known-failures.json"

# pytest's short summary: `FAILED tests/test_check.py::test_rule_2 - AssertionError:
# ...` — the paths in that output are python-ags4's own suite (not in this repo);
# this parses their run, not ours.
_FAILED = re.compile(r"^FAILED (\S+::\S+)", re.M)
# `10 failed, 121 passed, 3 warnings in 2.49s`. Scanned INDEPENDENTLY and
# tail-most, mirroring the shell this replaces (`grep -Eo '[0-9]+ passed' | tail
# -n1`), because a single combined regex quietly mis-binds: a `\d+ failed` group
# made optional will happily match a later bare `121 passed` with failed unset,
# and the count silently becomes the passed count. It did exactly that on the
# first real run here.
_N_FAILED = re.compile(r"(\d+) failed")
_N_PASSED = re.compile(r"(\d+) passed")


def _parse(log: str) -> tuple[set[str], int, int]:
    failed_ids = set(_FAILED.findall(log))
    f = _N_FAILED.findall(log)
    p = _N_PASSED.findall(log)
    return failed_ids, int(f[-1]) if f else 0, int(p[-1]) if p else 0


def main() -> int:
    args = [a for a in sys.argv[1:] if not a.startswith("--")]
    write = "--write" in sys.argv
    if not args:
        print("usage: check_parity.py <parity.log> [--write]", file=sys.stderr)
        return 2

    log = Path(args[0]).read_text(encoding="utf-8", errors="replace")
    failed_ids, n_failed, n_passed = _parse(log)
    total = n_failed + n_passed

    fixture = json.loads(_FIXTURE.read_text(encoding="utf-8"))
    known = fixture["known_failures"]
    pinned_total = fixture["total_tests"]

    # Guard the guard, FIRST — before any set comparison. A collection failure
    # yields zero FAILED lines, which set-compares as "everything we expected to
    # fail now passes": alarming-but-wrong, and if the fixture were ever empty it
    # would read as a clean run instead. Check the run happened before trusting
    # what it says.
    if total == 0:
        print(
            "BROKEN: the log records no tests at all (0 passed, 0 failed).\n"
            "  pytest collected nothing — a broken clone or an import error, not a\n"
            "  parity result. This workflow has eaten exactly that before (see\n"
            "  parity.yml's maturin-develop step comment). Refusing to report a\n"
            "  verdict on a run that did not happen.",
            file=sys.stderr,
        )
        return 1
    if not write and total != pinned_total:
        print(
            f"BROKEN: the suite collected {total} tests, the fixture pins "
            f"{pinned_total}.\n"
            f"  The test COUNT moving means upstream changed its suite — the "
            f"identity comparison below would be comparing against a different "
            f"population. Re-vendor deliberately (--write) once you have looked at "
            f"what upstream did.",
            file=sys.stderr,
        )
        return 1

    if write:
        # Preserve the reasons we already have; a new entry is stubbed loudly so it
        # cannot be vendored in silently as though it were considered.
        merged = {}
        for tid in sorted(failed_ids):
            merged[tid] = known.get(
                tid,
                {
                    "category": "UNTRIAGED",
                    "reason": "TODO: this failure was vendored in without a reason. "
                    "Say why it is acceptable, or fix it.",
                },
            )
        fixture["known_failures"] = merged
        fixture["total_tests"] = total
        _FIXTURE.write_text(json.dumps(fixture, indent=2) + "\n", encoding="utf-8")
        untriaged = [t for t, v in merged.items() if v.get("category") == "UNTRIAGED"]
        print(f"wrote {_FIXTURE.name}: {len(merged)} known failures, {total} tests")
        if untriaged:
            print("  UNTRIAGED (give these a reason before committing):")
            for t in untriaged:
                print(f"    {t}")
        return 0

    expected = set(known)
    regressions = sorted(failed_ids - expected)
    fixed = sorted(expected - failed_ids)

    if not regressions and not fixed:
        print(
            f"parity contract met by IDENTITY — {n_passed} passed, {n_failed} failed."
        )
        print(f"The {len(expected)} failures are exactly the accepted non-closures:")
        for tid in sorted(expected):
            e = known[tid]
            obs = f" [{e['observation']}]" if "observation" in e else ""
            print(f"  {e['category']:22}{obs:10} {tid}")
        return 0

    if regressions:
        print(
            f"PARITY REGRESSION — {len(regressions)} failure(s) we have not accepted:\n"
        )
        for tid in regressions:
            reason = re.search(rf"^FAILED {re.escape(tid)} - (.*)$", log, re.M)
            print(f"  {tid}")
            if reason:
                print(f"      {reason.group(1)[:120]}")
        print(
            "\nA count-based gate could have missed this entirely: if a divergence\n"
            "closed in the same run, the total would not have moved."
        )
    if fixed:
        print(f"\nACCEPTED FAILURES THAT NOW PASS — {len(fixed)}:\n")
        for tid in fixed:
            print(f"  {tid}  ({known[tid]['category']})")
        print(
            "\nThis is good news, and it still fails the gate on purpose: the record\n"
            "must follow the code. Re-vendor:\n"
            "    python tools/check_parity.py parity.log --write\n"
            "(the fixture's `reason` fields ARE the record; cross-link OBSERVATIONS.md)."
        )
    return 1


if __name__ == "__main__":
    sys.exit(main())
