#!/usr/bin/env python3
"""Cross-check `modality.json`'s duckdb cells against the extension's own manifest.

`modality.json` names duckdb verbs by hand — `read_ags`, `read_ags_text`,
`read_ags (<path>.idx autodiscovery)`. The extension declares those functions in
`functions.json`, which its own `tests/functions_manifest.rs` gates against the
`register_table()` calls, so the manifest cannot drift from the real SQL surface.
Nothing compared the two, which left two drifts invisible:

  * a function RENAMED or REMOVED upstream leaves a cell naming a verb that no
    longer exists, and
  * a function ADDED upstream appears in no cell — the same silence #717 was
    filed about, reappearing one release later.

Both are mechanically detectable, so this is a tripwire in both directions. It is
NOT a way to derive the duckdb row: mapping `read_ags` to the `read` capability
is a human judgement, and #734 owns that question.

The manifest it reads is a PIN — `tools/vendor/laterite-duckdb-functions.json`,
whose `_pin` block carries the refresh procedure. What it catches is our register
drifting from a known-good surface, never the extension moving.

Its reach is "at least one cell", per FUNCTION, not per cell — the two lists have
no common key finer than the function name, because which capability `read_ags`
belongs to is the human judgement #734 owns. So a verb dropped from ONE cell
while another still names it is invisible here, and so is a cell that names a
real function under the wrong capability. Both are register-shape questions this
gate does not ask; it asks only whether the two lists cover each other.

Deliberately not enrolled: seven of the nine functions belong to categories this
register has no capability for at all (see UNENROLLED). That is a real finding
about the register's scope, not a shortfall of this gate — so the set is named,
reasoned and PRINTED on every run rather than filtered away, which is the failure
mode CLAUDE.md records `check_doc_refs.py` paying for.

Usage:
    uv run --no-project python tools/check_duckdb_manifest.py

Exit 0 when both directions hold, 1 when either does not.
"""

from __future__ import annotations

import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
MODALITY = ROOT / "modality.json"
PIN = ROOT / "tools" / "vendor" / "laterite-duckdb-functions.json"

# A duckdb verb is a SQL identifier, optionally followed by a parenthesised
# qualifier that says which shape of the call the cell means:
#   "read_ags"                          -> read_ags
#   "read_ags (<path>.idx autodiscovery)" -> read_ags
# Matched, not substring-searched: `in` against the cell text would pass for the
# wrong reasons (it cannot tell `read_ags` from `read_ags_text`, and it would
# accept an identifier buried in a qualifier).
_VERB = re.compile(r"^([A-Za-z_][A-Za-z0-9_]*)(?:\s*\(.*\))?$")

# Functions that no duckdb cell names, and why. Each reason is about the
# REGISTER's axes, not about the function: this register's capabilities are the
# ones an AGS4 *file* passes through, and these three categories are not among
# them. Enrolling them means adding a capability, which is a register-scope
# decision (#717 / #734), not something a drift gate may take.
UNENROLLED: dict[str, str] = {
    "ags_groups": (
        "structure introspection — the register has no capability for 'what "
        "groups/headings does this file have', on any surface; it is carried "
        "only as the `handle` output form of `read`"
    ),
    "ags_headings": (
        "structure introspection — as `ags_groups`, and the dictionary "
        "enrichment it adds does not make it a capability of its own"
    ),
    "ags_dictionary": (
        "reference-data query — takes no AGS4 file, so it sits outside a "
        "register whose axis is the form a file enters and leaves in"
    ),
    "ags_relationships": "reference-data query — as `ags_dictionary`",
    "ags_rules": (
        "reference-data query — as `ags_dictionary`. It LISTS the numbered "
        "rules; running them is the `validate` capability, which the extension "
        "does not offer (`read_only: true`) and whose cell already says so"
    ),
    "load_ags": (
        "persistence DDL — emits CREATE TABLE statements for the caller to "
        "run. The register has no capability for materialising a read into a "
        "store; `emit` is AGS4 back out, not SQL"
    ),
    "to_duckdb": (
        "persistence DDL — as `load_ags`. The py/rust surfaces DO ship a "
        "`to_duckdb()` and the register carries no capability for it either, "
        "so this absence is the register's, not the extension's"
    ),
}


def _fail(msg: str) -> None:
    print(f"FAIL: {msg}", file=sys.stderr)


def main() -> int:
    pin = json.loads(PIN.read_text(encoding="utf-8"))
    if "_pin" not in pin:
        _fail(
            f"{PIN.relative_to(ROOT)} has no `_pin` block — a refresh that "
            "copies upstream over it drops the note saying this file is a "
            "snapshot and that new functions must be reconciled into cells. "
            "Re-add it (git show HEAD -- the file) after copying."
        )
        return 1

    declared = {f["name"]: f.get("category", "?") for f in pin["functions"]}

    modality = json.loads(MODALITY.read_text(encoding="utf-8"))
    named: dict[str, list[str]] = {}
    unparsed: list[str] = []
    cells = 0
    verbless = 0
    for cap in modality["capabilities"]:
        for cell in cap["cells"]:
            if cell["surface"] != "duckdb":
                continue
            cells += 1
            if not cell.get("verbs"):
                verbless += 1
            for verb in cell.get("verbs", []):
                raw = verb["name"]
                m = _VERB.match(raw)
                if m is None:
                    unparsed.append(f"{cap['capability']}: {raw!r}")
                    continue
                named.setdefault(m.group(1), []).append(cap["capability"])

    ok = True

    # Direction 1 — every verb a cell names is a function that exists.
    for ident, caps in sorted(named.items()):
        if ident not in declared:
            _fail(
                f"modality.json's duckdb cell for `{', '.join(caps)}` names "
                f"`{ident}`, which the pinned manifest "
                f"({pin['extension']} {pin['version']}) does not declare — "
                "either the function was renamed/removed upstream and the cell "
                "must follow, or the cell has a typo"
            )
            ok = False

    # Direction 2 — every function that exists is accounted for by a cell or by
    # a reasoned non-enrolment. A brand-new function is in neither, and fails.
    for name, category in sorted(declared.items()):
        if name in named or name in UNENROLLED:
            continue
        _fail(
            f"the pinned manifest declares `{name}` (category {category}) and "
            "no duckdb cell names it — reconcile it into a cell in "
            "modality.json, or, if this register has no capability for it, "
            "into UNENROLLED in this file with the reason why"
        )
        ok = False

    # A non-enrolment for a function that is gone is stale bookkeeping: it
    # reads as a live decision and defends nothing.
    for name in sorted(UNENROLLED):
        if name not in declared:
            _fail(
                f"UNENROLLED still excuses `{name}`, which the pinned manifest "
                "no longer declares — drop the entry"
            )
            ok = False
        elif name in named:
            _fail(
                f"`{name}` is both named by a duckdb cell and excused in "
                "UNENROLLED — the cell wins; drop the entry"
            )
            ok = False

    if unparsed:
        _fail(
            "these duckdb verb strings do not lead with a SQL identifier, so "
            "neither direction could ask about them: " + "; ".join(unparsed)
        )
        ok = False

    # Always, pass or fail: what was asked, and what was deliberately not.
    print(
        f"[duckdb-manifest] pin: {pin['extension']} {pin['version']}, "
        f"{len(declared)} function(s). Read {cells} duckdb cell(s) naming "
        f"{len(named)} distinct verb(s); {len(unparsed)} unreadable."
    )
    print(
        f"[duckdb-manifest] no cell names {len(UNENROLLED)} of them, by "
        "recorded decision — this gate is blind to whether that stays right: "
        + ", ".join(sorted(UNENROLLED))
    )
    print(
        f"[duckdb-manifest] {verbless} of the {cells} cell(s) name no verb "
        "(the capability is absent on duckdb), so they are outside both "
        "directions; and coverage is counted per function, not per cell — a "
        "verb dropped from one cell while another still names it passes here."
    )
    if ok:
        print("[duckdb-manifest] OK: cells and manifest agree in both directions.")
    return 0 if ok else 1


if __name__ == "__main__":
    raise SystemExit(main())
