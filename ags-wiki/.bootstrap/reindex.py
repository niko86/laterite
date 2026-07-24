#!/usr/bin/env python
"""Content-safe index.md regenerator (AGS-WIKI.md §8).

Scans the vault filesystem and rebuilds ONLY index.md — never touches
content pages (unlike generate.py, which rebuilds bootstrap stubs).
Run at the end of every Ingest phase:
    python ags-wiki/.bootstrap/reindex.py

`render()` is a pure function (filesystem in, string out — same shape as
`tools/gen_observations.py::render`), so the committed `index.md` ==
`render()` comparison needs no subprocess. `--check` runs exactly that from
the CLI (diff vs committed, nonzero exit on mismatch) instead of writing;
the wiki-lint workflow runs it as a step, which is what stops a page being
added/removed/reclassified without regenerating the index.
"""

from __future__ import annotations

import datetime
import json
import re
import sys
from pathlib import Path

WIKI = Path(__file__).resolve().parent.parent

# The page classes are single-sourced in wiki-classes.json (D5) — reindex and
# lint.py both read it, so the list can't drift between the index generator and
# the linter (which used to hand-duplicate it).
_CLASSES = json.loads(
    (Path(__file__).resolve().parent / "wiki-classes.json").read_text(encoding="utf-8")
)["classes"]
CLASSES = [
    (c["dir"], c["label"], c["extra"]) for c in _CLASSES
]  # dir, label, extra column
CAMPAIGN = {c["dir"] for c in _CLASSES if c["campaign"]}


def fmval(txt: str, key: str):
    m = re.search(rf"^{re.escape(key)}:\s*(.+?)\s*$", txt, re.M)
    return m.group(1).strip() if m else ""


def render(today: str | None = None) -> str:
    """Scan the vault and return the full index.md text (frontmatter +
    body). Pure w.r.t. the filesystem snapshot at call time — the only
    non-derived input is `today` (defaults to the real date), threaded
    through rather than read from `datetime` internally so a
    faithfulness check can pin it to the committed file's own stamp and
    compare the *derived content* without every run past the original
    generation day looking like drift (see `_normalize_generated`)."""
    today = today or datetime.date.today().isoformat()
    counts: dict[str, int] = {}
    status_dist: dict[str, int] = {}
    sections: list[str] = []
    # D4: superseded pages are lifted out of their live class table into a
    # dedicated Retired section — they're read-time tombstones (redirect
    # stubs), not live references, so they shouldn't sit indexed among live
    # pages or inflate a class count. (stem, class label, raw superseded_by).
    retired: list[tuple[str, str, str]] = []

    for d, label, extra in CLASSES:
        dd = WIKI / d
        # rglob, not glob (D5): pages under a subfolder — e.g.
        # sources/repo-authorities/*.md — were invisible to the catalog. lint.py
        # already rglob'd its page scan, so those pages resolved as wikilink
        # targets but never appeared in index.md.
        pages = sorted(dd.rglob("*.md")) if dd.exists() else []
        real = [p for p in pages if p.stem != "_README"]
        sections.append(f"\n## {label}\n")
        sections.append("| page | status |" + (f" {extra} |" if extra else " |"))
        sections.append("|---|---|" + ("---|" if extra else ""))
        if (dd / "_README.md").exists():
            sections.append(
                f"| [[{d}/_README\\|{label} register]] | register |"
                + (" — |" if extra else "")
            )
        live = 0
        for p in real:
            t = p.read_text(encoding="utf-8")
            st = fmval(t, "status") or "?"
            if st == "superseded":
                retired.append((p.stem, label, fmval(t, "superseded_by")))
                continue
            live += 1
            status_dist[st] = status_dist.get(st, 0) + 1
            row = f"| [[{p.stem}]] | {st} |"
            if extra:
                row += f" {fmval(t, extra) or '—'} |"
            sections.append(row)
        counts[d] = live
        if live == 0 and d in CAMPAIGN:
            sections.append(
                "| _(none yet — campaign-authored)_ | — |" + (" — |" if extra else "")
            )

    if retired:
        sections.append("\n## Retired / Superseded\n")
        sections.append(
            "> Pages for removed packages / superseded decisions, "
            "kept as read-time tombstones (redirect stubs) — not "
            "live references. Excluded from the class tables and "
            "counts above; each links to what replaced it (D4)."
        )
        sections.append("\n| page | class | superseded by |")
        sections.append("|---|---|---|")
        for stem, label, sb in sorted(retired):
            raw = sb.strip()
            if raw.startswith("[") and raw.endswith("]"):
                raw = raw[1:-1]
            tgts = [x.strip().split("/")[-1] for x in raw.split(",") if x.strip()]
            disp = " · ".join(f"[[{x}]]" for x in tgts) if tgts else "—"
            sections.append(f"| [[{stem}]] | {label} | {disp} |")

    total = sum(counts.values())
    cov = ["## Coverage\n", "| Class | Pages |", "|---|---|"]
    for d, label, _ in CLASSES:
        cov.append(f"| {label} | {counts[d]} |")
    cov.append(f"| **Total (live)** | **{total}** |")
    if retired:
        cov.append(f"| _Retired / superseded_ | {len(retired)} |")
    cov.append(
        "\n**Status distribution:** "
        + " · ".join(f"`{k}` {v}" for k, v in sorted(status_dist.items()))
    )

    # Gaps: stub content pages remaining + rules missing fixtures
    stub_pages = [
        p.stem
        for d, *_ in CLASSES
        if d not in CAMPAIGN
        for p in (WIKI / d).rglob("*.md")
        if p.stem != "_README"
        and re.search(r"^status:\s*stub\s*$", p.read_text(encoding="utf-8"), re.M)
    ]
    gaps = [
        "\n## Gaps\n",
        f"- Content pages still `stub` (await later Ingest phases): "
        f"**{len(stub_pages)}**.",
        "- Insights with `proposes_observation: true` await user "
        "ratification before any `OBSERVATIONS.md` change (AGS-WIKI §12.5).",
        "- See `.bootstrap/INGEST-PLAN.md` for phase status; `log.md` for activity.",
    ]

    fm = (
        f"---\ntype: index\ngenerated: {today}\n"
        f"counts: {{{', '.join(f'{k}: {v}' for k, v in counts.items())}}}\n"
        f"---\n"
    )
    body = (
        "# AGS Wiki — Content Catalog\n\n"
        "> Regenerated by `.bootstrap/reindex.py` every Ingest phase "
        "(AGS-WIKI.md §8). Filesystem-scanned; never hand-edited.\n\n"
        + "\n".join(cov)
        + "\n"
        + "\n".join(sections)
        + "\n"
        + "\n".join(gaps)
        + "\n\n## Related\n[[start-here]] · [[log]] · [[AGS-WIKI]]\n"
    )
    return fm + body


def _normalize_generated(text: str) -> str:
    """Blank the `generated: YYYY-MM-DD` stamp before comparing two
    renders — it's a regeneration timestamp, not derived content, so a
    faithfulness check that ran on a different day than the last real
    `reindex.py` run must not treat the date alone as drift."""
    return re.sub(
        r"^generated: \d{4}-\d{2}-\d{2}$", "generated: <date>", text, flags=re.M
    )


def main() -> None:
    content = render()
    index_path = WIKI / "index.md"
    if "--check" in sys.argv:
        committed = (
            index_path.read_text(encoding="utf-8") if index_path.exists() else ""
        )
        if _normalize_generated(content) != _normalize_generated(committed):
            print(
                "index.md is STALE (content differs from render()) — run "
                "`uv run --no-sync python ags-wiki/.bootstrap/reindex.py` "
                "to regenerate",
                file=sys.stderr,
            )
            sys.exit(1)
        print("index.md OK: matches render()")
        return
    index_path.write_text(content, encoding="utf-8")
    m = re.search(r"^counts: (\{.*\})$", content, re.M)
    print(f"reindex OK: wrote index.md {m.group(1) if m else ''}")


if __name__ == "__main__":
    main()
