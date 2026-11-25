#!/usr/bin/env python3
"""Render the validator README's rule table from the rule catalogue.

`rust-packages/laterite-ags4-validator/README.md` ships to crates.io with a
table of what each rule covers. It was hand-written, and by the time anyone read
it against the engine about ten rows described a different rule from the one
numbered and two named rules the catalogue does not have — on the page a
newcomer to the crate reads first. Nothing had failed, because nothing read it.

The catalogue (`laterite-ags4-reference/data/rules_meta.json`) already holds a
gated id and title per rule: the validator's own tests hold it to exactly the
rules the engine emits. So the table is rendered from it, between
`<!-- BEGIN/END GENERATED: rules -->` markers, and the prose around the markers
stays hand-written.

Only id and title are rendered. The catalogue's longer `checks` text is written
for someone looking at a failing file, and `lat rules --json` is where it is
read; severity and fixability change with the engine and `lat rules` prints them.

Usage:
  uv run --no-project python tools/gen_rule_table.py            # write the table
  uv run --no-project python tools/gen_rule_table.py --check    # CI drift gate
"""

from __future__ import annotations

import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CATALOGUE = ROOT / "rust-packages/laterite-ags4-reference/data/rules_meta.json"
README = ROOT / "rust-packages/laterite-ags4-validator/README.md"
MARKER = "rules"
REGEN = "uv run --no-project python tools/gen_rule_table.py"


def render_table(rules: list[dict]) -> list[str]:
    """The table body, one row per catalogue rule, in catalogue order."""
    out = ["| Rule | Covers |", "|---|---|"]
    for r in rules:
        title = str(r["title"]).replace("|", "\\|")
        out.append(f"| {r['rule']} | {title} |")
    return out


def splice(md: str, table: list[str]) -> str:
    """Replace what lies between the BEGIN/END markers with `table`.

    Exactly one of each, in order, or it raises: a README that lost a marker
    must fail the gate rather than quietly keep whatever table it has.
    """
    begin, end = f"<!-- BEGIN GENERATED: {MARKER}", f"<!-- END GENERATED: {MARKER}"
    lines = md.split("\n")
    b = [i for i, ln in enumerate(lines) if ln.startswith(begin)]
    e = [i for i, ln in enumerate(lines) if ln.startswith(end)]
    if len(b) != 1 or len(e) != 1 or b[0] > e[0]:
        raise ValueError(
            f"expected one `{begin} …` line followed by one `{end} …` line; "
            f"found {len(b)} and {len(e)}"
        )
    return "\n".join(lines[: b[0] + 1] + table + lines[e[0] :])


def render() -> str:
    rules = json.loads(CATALOGUE.read_text(encoding="utf-8"))["rules"]
    return splice(README.read_text(encoding="utf-8"), render_table(rules))


def main(argv: list[str]) -> int:
    rel = README.relative_to(ROOT)
    try:
        want = render()
    except ValueError as exc:
        print(f"gen_rule_table: {rel}: {exc}", file=sys.stderr)
        return 1
    if "--check" in argv:
        if README.read_text(encoding="utf-8") != want:
            print(
                f"gen_rule_table: {rel} does not match rules_meta.json — "
                f"regenerate: {REGEN}",
                file=sys.stderr,
            )
            return 1
        print(f"gen_rule_table: {rel} matches rules_meta.json")
        return 0
    README.write_text(want, encoding="utf-8")
    print(f"gen_rule_table: wrote {rel}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
