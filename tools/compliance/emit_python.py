#!/usr/bin/env python3
"""The python-laterite leg of the cross-surface findings gate (#1170).

Runs `laterite.validate` over a fixtures dir and writes `python-laterite.json`
in the schema `laterite-ags4-compliance` reads, beside what `emit-rust` and
`emit_js.mjs` write for the other three surfaces. It runs in ci.yml's `python`
job, against the cdylib that job already loaded, so the gate tests the build CI
tests rather than a second one.

Ported from the dev satellite's `_worker.py`, minus its python-ags4 leg. That leg
compares against the incumbent library and needs no rebuild per PR, so it stays
with the scheduled report; this one exists because the four laterite surfaces
must agree on every change that could move a finding.

The harness invariants are the other emitters': warnings and FYI on,
`check_files` off (the fixtures do not ship the files they reference), encoding
left to its default so the latin-1 and cp1252 fixtures fire Rule 1 the same way
everywhere.

    uv run --no-sync python tools/compliance/emit_python.py \
        --fixtures <dir> --out <dir>
"""

from __future__ import annotations

import argparse
import json
import re
from pathlib import Path
from typing import TYPE_CHECKING, Any

if TYPE_CHECKING:
    from collections.abc import Iterable

RULE = re.compile(r"^AGS Format Rule ")
FYI = re.compile(r"^FYI")


def split(keys: Iterable[str]) -> tuple[list[str], list[str]]:
    """Partition finding labels into the rule floor and the FYI labels."""
    floor = sorted(k for k in keys if RULE.match(k))
    fyi = sorted(k for k in keys if FYI.match(k))
    return floor, fyi


def floor_tuples(ndjson: str) -> list[dict[str, Any]]:
    """The floor as full tuples (rule, line, group, desc, field_index), parsed
    from the report's NDJSON: the value the four-surface check compares. FYI
    lines are excluded, like the label floor; `field_index` is absent on a
    whole-group finding and recorded as null, as every surface does."""
    out = []
    for line in ndjson.splitlines():
        if not line.strip():
            continue
        o = json.loads(line)
        if not RULE.match(o.get("rule", "")):
            continue
        out.append(
            {
                "rule": o["rule"],
                "line": o.get("line"),
                "group": o.get("group", ""),
                "desc": o.get("desc", ""),
                "field_index": o.get("field_index"),
            }
        )
    return out


def surface(fixtures: list[Path]) -> dict[str, Any]:
    import laterite

    results = []
    for p in fixtures:
        try:
            rep = laterite.validate(p, warnings=True, fyi=True)
        except Exception as e:
            # An un-validatable input raises a typed error (`NotAgs4Error`,
            # `UnsupportedEditionError`, …). Stripping `Error` recovers the
            # sentinel every other surface reports for the same file.
            results.append(
                {
                    "fixture": p.name,
                    "rules": [],
                    "findings": [],
                    "fyi": [],
                    "hard_error": type(e).__name__.removesuffix("Error"),
                    "error": None,
                }
            )
            continue
        floor, fyi = split(rep.by_rule().keys())
        results.append(
            {
                "fixture": p.name,
                "rules": floor,
                "findings": floor_tuples(rep.to_ndjson()),
                "fyi": fyi,
                "hard_error": None,
                "error": None,
            }
        )
    # The engine's own version and build fingerprint, not the wheel's product
    # number: the comparator holds the four surfaces to one engine, and the
    # product number is not what `emit-rust` (the engine itself) can report.
    return {
        "schema": 1,
        "surface": "python-laterite",
        "version": laterite.engine_version(),
        "engine_sha": laterite.engine_fingerprint(),
        "results": results,
    }


def main() -> None:
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("--fixtures", required=True, type=Path)
    ap.add_argument("--out", required=True, type=Path)
    a = ap.parse_args()
    fixtures = sorted(a.fixtures.glob("*.ags"))
    if not fixtures:
        raise SystemExit(f"no *.ags fixtures in {a.fixtures}")
    a.out.mkdir(parents=True, exist_ok=True)
    payload = surface(fixtures)
    path = a.out / f"{payload['surface']}.json"
    path.write_text(json.dumps(payload), encoding="utf-8")
    print(
        f"{payload['surface']} engine {payload['version']} "
        f"({payload['engine_sha']}): {len(payload['results'])} fixtures -> {path}"
    )


if __name__ == "__main__":
    main()
