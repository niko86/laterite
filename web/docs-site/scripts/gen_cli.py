"""Render the shipped CLI guide (README-cli.md) into the docs as the generated
`lat` command reference (#430).

The command reference used to be hand-maintained here and it drifted — it silently
missed `read`, `transport`, and `excel` the moment those verbs shipped. README-cli.md
is the single source of truth: it is exactly what `lat --readme` prints, it ships in
both the binary and the wheel, and a test keeps its two copies byte-in-sync. Rendering
it at build time means a new verb updates one file and the docs follow — the reference
can no longer drift out from under the tool.
"""

from __future__ import annotations

from pathlib import Path

import mkdocs_gen_files

_REPO = Path(__file__).resolve().parents[3]
_README_CLI = _REPO / "rust-packages" / "laterite-cli" / "README-cli.md"

_guide = _README_CLI.read_text()

# Drop the guide's own "# lat" H1 — the page supplies its own title + the note that
# this is the shipped `--readme` guide, single-sourced.
_body = _guide.split("\n", 1)[1] if _guide.startswith("# ") else _guide

_page = f"""# `lat` command reference

!!! note "Generated from the shipped guide"
    This page **is** `lat --readme`: the guide bundled in the binary and the wheel,
    the single source of truth for every verb, flag, and exit code (so it can't drift
    from the tool). For how to run it (the native binary, `uvx --from laterite lat`,
    or `npx laterite`), see [CLI: one tool, three launchers](../surfaces/cli.md).
{_body}"""

with mkdocs_gen_files.open("reference/cli.md", "w") as fd:
    fd.write(_page)
