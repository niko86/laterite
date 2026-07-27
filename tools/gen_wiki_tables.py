#!/usr/bin/env python3
"""Render marker-delimited wiki tables from their JSON source of truth.

The register/ledger family in the vault (`concepts/mutation-sweep.md`'s sweep
ledger, `design/reliquary.md`'s machine-checked register) are append-per-batch
*tables* wrapped in hand-written prose. Their rows are the structured, SSOT-worthy
part; the prose is not. So — mirroring `observations.json → OBSERVATIONS.md` and
`ags_dictionary.json → models` — each table lives in a JSON file and is rendered
back into a `<!-- BEGIN/END GENERATED: <marker> -->` region in its `.md`. The
prose outside the markers stays native Markdown, edited by hand.

This is a *structural* generator: it owns table shape and pipe-escaping, nothing
else. It carries **no** content/leak gate — these are internal design/process
pages that reference dead-code relics (including AGS5-strand ones) by name on
purpose; the public-facing `CHANGELOG.md` is where the AGS5-as-concept gate lives
(`tools/gen_changelog.py`), not here.

Usage:
  uv run --no-project python tools/gen_wiki_tables.py            # write every table
  uv run --no-project python tools/gen_wiki_tables.py --check    # CI drift gate
"""

from __future__ import annotations

import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

# The JSON sources this generator owns. Add a line to extend the family: a new
# JSON (declaring its own `target`, `marker`, `fields`, `rows`) plus the matching
# BEGIN/END markers in the target `.md`. No new code needed.
SOURCES = [
    "ags-wiki/concepts/mutation-sweep.json",
    "ags-wiki/design/reliquary.json",
]


def _render_table(fields: list[dict], rows: list[dict]) -> list[str]:
    """The Markdown table body (header, separator, one line per row). Cells carry
    Markdown verbatim; the lone transform is escaping a literal `|` to `\\|` so it
    can't be read as a column break."""
    headers = [f["header"] for f in fields]
    keys = [f["key"] for f in fields]
    out = [
        "| " + " | ".join(headers) + " |",
        "|" + "|".join("---" for _ in fields) + "|",
    ]
    for row in rows:
        cells = [str(row.get(k, "")).replace("|", "\\|") for k in keys]
        out.append("| " + " | ".join(cells) + " |")
    return out


def _splice(md: str, marker: str, table: list[str]) -> str:
    """Replace the lines between the `<marker>` BEGIN/END comments with `table`."""
    begin = f"<!-- BEGIN GENERATED: {marker}"
    end = f"<!-- END GENERATED: {marker}"
    lines = md.split("\n")
    bi = next((i for i, ln in enumerate(lines) if ln.startswith(begin)), None)
    ei = next((i for i, ln in enumerate(lines) if ln.startswith(end)), None)
    if bi is None or ei is None:
        raise SystemExit(f"gen_wiki_tables: {marker!r} markers not found in target")
    return "\n".join(lines[: bi + 1] + table + lines[ei:])


def _render_target(src_rel: str) -> tuple[Path, str]:
    """(target path, its full text with the generated region refreshed)."""
    data = json.loads((ROOT / src_rel).read_text(encoding="utf-8"))
    target = ROOT / data["target"]
    table = _render_table(data["fields"], data["rows"])
    return target, _splice(target.read_text(encoding="utf-8"), data["marker"], table)


def main(argv: list[str]) -> int:
    check = "--check" in argv
    stale: list[str] = []
    for src_rel in SOURCES:
        target, rendered = _render_target(src_rel)
        current = target.read_text(encoding="utf-8")
        if check:
            if current != rendered:
                stale.append(f"{target.relative_to(ROOT)} (from {src_rel})")
        elif current != rendered:
            target.write_text(rendered, encoding="utf-8")
            print(f"gen_wiki_tables: wrote {target.relative_to(ROOT)}")
        else:
            print(f"gen_wiki_tables: {target.relative_to(ROOT)} already current")
    if check and stale:
        print(
            "gen_wiki_tables: generated table drifted from its JSON source — "
            "run `uv run --no-project python tools/gen_wiki_tables.py`:",
            file=sys.stderr,
        )
        for s in stale:
            print(f"  - {s}", file=sys.stderr)
        return 1
    if check:
        print("gen_wiki_tables: all generated tables up to date")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
