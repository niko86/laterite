#!/usr/bin/env python3
"""Hold the rule catalogue's O-N citations to the OBSERVATIONS canon.

There are two bodies of divergence prose in this repo and only one of them is
gated. `observations.json` is the canon, and `tools/gen_observations.py` holds
its rendered views — OBSERVATIONS.md, the wiki's coverage map, the docs site's
divergence page — in step with it. The rule catalogue
(`rust-packages/laterite-ags4-reference/data/rules_meta.json`) carries a SECOND
set of divergence notes, hand-written in practitioner voice, which the webapp's
rule explainer renders beside each rule. Nothing checked those against the canon
at all, and the two drift silently because they are read by different people in
different places.

**Prose is not the subject.** The two deliberately differ: the canon is written
observed / spec / assessment / upstream-reportable / our decision, and the
catalogue is written for someone looking at a failing file. Comparing wording
would fail constantly and correctly. What is checked here is IDENTITY — that the
record a note cites exists, and that it is still the record it was citing.

## What a red run means

Two shapes, both real:

* **`O-N does not exist`** — a note cites a record that was never written, or was
  renumbered. The number resolves to nothing, or worse, to a later record about
  something else entirely.
* **`O-N is superseded`** — the canon marked the record `status` with a
  `resolved_by` pointing at what replaced it, and the catalogue is still telling
  a reader the retired story. This is the failure that motivated the gate: a note
  on Rule 9 cited O-10 (superseded by O-30) while saying so in its own text, and
  a note on Rule 14 cited O-20 (also superseded by O-30) with content that had
  outlived its record.
* **`O-N cited twice`** — one rule carrying two notes about the same record. This
  is what REPOINTING produces: fixing the Rule 9 note above landed it on O-30
  beside a note that already cited O-30, and a reader gets the same divergence
  told twice in two voices. The fix that this gate demands is the one that
  creates it, so it catches it.

Fix by repointing the note at the `resolved_by` record and rewriting it to what
that record says, never by dropping the citation.

## Scope, stated rather than implied

The canon records no RULE per observation, so this gate cannot check that a note
is attached to the right rule — and a title-derived guess would be wrong, because
some citations are deliberate cross-references (O-3 is titled for Rule 5 and is
correctly cited on Rule 4, since the divergence IS the 4-vs-5 attribution). That
blind spot is counted and printed on every run, pass or fail, rather than left to
be inferred from a green tick: a gate that drops input says what it dropped
(CLAUDE.md, Conventions).

Usage:
    uv run --no-project python tools/check_rule_catalogue_refs.py

Exit 0 when every citation resolves to a live record, 1 otherwise.
"""

from __future__ import annotations

import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
CATALOGUE = ROOT / "rust-packages/laterite-ags4-reference/data/rules_meta.json"
CANON = ROOT / "observations.json"


def citations() -> list[tuple[str, str]]:
    """(rule, O-N) for every divergence note in the catalogue, in file order."""
    doc = json.loads(CATALOGUE.read_text(encoding="utf-8"))
    return [
        (rule["rule"], note["id"])
        for rule in doc["rules"]
        for note in rule.get("observations", [])
    ]


def records() -> dict[str, dict]:
    """O-N -> the canon record, flattened across the canon's sections."""
    doc = json.loads(CANON.read_text(encoding="utf-8"))
    return {
        rec["id"]: rec for section in doc["sections"] for rec in section["observations"]
    }


def main() -> int:
    cited = citations()
    canon = records()

    problems: list[str] = []
    seen: dict[tuple[str, str], int] = {}
    for rule, obs in cited:
        seen[(rule, obs)] = seen.get((rule, obs), 0) + 1
        if seen[(rule, obs)] == 2:
            problems.append(
                f"  rule {rule}: cites {obs} more than once — one divergence "
                f"told twice, in two voices. Merge the notes."
            )
        rec = canon.get(obs)
        if rec is None:
            problems.append(
                f"  rule {rule}: cites {obs}, which is not in the canon — it was "
                f"never written, or it was renumbered"
            )
            continue
        if rec.get("status"):
            problems.append(
                f"  rule {rule}: cites {obs}, which the canon marks "
                f"`{rec['status']}` (resolved by {rec.get('resolved_by', '?')}) — "
                f"the note is telling the retired story"
            )

    # The scope statement, printed pass or fail — see the module docstring.
    print(
        f"check_rule_catalogue_refs: {len(cited)} catalogue citation(s) checked "
        f"against {len(canon)} canon record(s)"
    )
    print(
        f"check_rule_catalogue_refs: rule ATTACHMENT unchecked for all "
        f"{len(cited)} — the canon records no rule per observation, and a "
        f"title-derived guess would fail the deliberate cross-references"
    )

    if problems:
        print(
            f"\ncheck_rule_catalogue_refs: {len(problems)} citation(s) do not "
            f"resolve to a live record:\n" + "\n".join(problems),
            file=sys.stderr,
        )
        print(
            "\nRepoint each note at the record that replaced it and rewrite it to "
            "what that record says. Dropping the citation instead leaves the note "
            "unsourced, which is the state this gate exists to end.",
            file=sys.stderr,
        )
        return 1

    print("check_rule_catalogue_refs: OK — every citation resolves to a live record")
    return 0


if __name__ == "__main__":
    sys.exit(main())
