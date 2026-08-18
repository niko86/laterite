"""Hold the TOC rail's breakpoint to Material's own (#402).

The strata rail runs down the left edge of the table of contents, so it has to
appear and disappear exactly with it. There is no way to say that in CSS — the
rail cannot ask whether `.md-sidebar--secondary` is displayed — so its media
query carries a COPY of Material's breakpoint, and a copy of somebody else's
internal number is the definition of a value that rots.

It rotted immediately: the first cut used 76.1875em, Material's *layout*
breakpoint, where the sidebar actually turns on at 60em. Between the two the
reader got a table of contents with no strip beside it, on every window from
960px to 1219px. Nothing failed, nothing looked broken, and the mismatch was
found only by sweeping widths in a browser.

So the number is checked against the source it was copied from, on every build,
by reading the CSS mkdocs-material actually shipped.

CALIBRATION: this logs a WARNING rather than raising. The PR gate builds with
`--strict`, which promotes warnings to errors, so a Material bump that moves the
breakpoint fails there. The deploy build is deliberately non-strict (a docs nit
must never block the app deploy), so the same drift is loud but not fatal on the
way out. That is the split the docs site already uses everywhere else.
"""

from __future__ import annotations

import logging
import re
from pathlib import Path
from typing import Any

log = logging.getLogger("mkdocs.hooks.rail_breakpoint")

# The rule that turns the table of contents on, in Material's shipped stylesheet.
_TOC_RULE = ".md-sidebar--secondary:not([hidden]){display:block}"
# Ours, in the override layer.
_RAIL_QUERY = re.compile(
    r"@media screen and \(max-width:\s*([\d.]+)em\)\s*\{\s*\.md-rail\s*\{\s*display:\s*none",
)
_MIN_WIDTH = re.compile(r"min-width:\s*([\d.]+)em")

# Media queries step in 1/16em: Material's `min-width: 60em` turns the sidebar on,
# so ours must turn the rail off at the largest width below it.
_STEP = 0.0625


def _enclosing_prelude(css: str, index: int) -> str | None:
    """The prelude of the block containing `index` — e.g. `@media …`.

    Walks back counting braces rather than regexing, because the stylesheet is
    minified onto one line and nesting is the only structure left.
    """
    depth = 0
    i = index
    while i >= 0:
        ch = css[i]
        if ch == "}":
            depth += 1
        elif ch == "{":
            if depth == 0:
                start = max(css.rfind("}", 0, i), css.rfind("{", 0, i)) + 1
                return css[start:i].strip()
            depth -= 1
        i -= 1
    return None


def _material_toc_breakpoint() -> float:
    import material  # installed alongside mkdocs-material; the docs build needs it

    sheets = sorted(
        (Path(material.__file__).parent / "templates/assets/stylesheets").glob(
            "main.*.min.css"
        )
    )
    if not sheets:
        raise FileNotFoundError(
            "mkdocs-material's main.*.min.css is missing — the rail breakpoint "
            "cannot be checked, and a check that cannot run must not pass silently."
        )
    css = sheets[-1].read_text(encoding="utf-8")
    index = css.find(_TOC_RULE)
    if index == -1:
        raise ValueError(
            f"mkdocs-material no longer contains {_TOC_RULE!r} — the rule that "
            "turns the table of contents on has been renamed or restructured, so "
            "the rail's breakpoint has nothing to track. Re-derive it by hand."
        )
    prelude = _enclosing_prelude(css, index) or ""
    found = _MIN_WIDTH.search(prelude)
    if not found:
        raise ValueError(
            f"the table-of-contents rule is no longer inside a min-width media "
            f"query (found prelude {prelude[:80]!r})."
        )
    return float(found.group(1))


def on_config(config: Any, **_: Any) -> Any:
    ours = Path(config.docs_dir) / "stylesheets/laterite.css"
    match = _RAIL_QUERY.search(ours.read_text(encoding="utf-8"))
    if not match:
        log.warning(
            "rail_breakpoint: no `@media (max-width: …em) { .md-rail { display: none",
            " rule found in stylesheets/laterite.css — the rail may now outlive the"
            " table of contents at narrow widths.",
        )
        return config

    rail_max = float(match.group(1))
    toc_min = _material_toc_breakpoint()
    expected = round(toc_min - _STEP, 4)
    if rail_max != expected:
        log.warning(
            "rail_breakpoint: the TOC rail hides below %sem but mkdocs-material "
            "shows the table of contents from %sem, so a reader between them gets "
            "a table of contents with no rail beside it. Set the rail's media "
            "query to max-width: %sem in docs/stylesheets/laterite.css.",
            rail_max,
            toc_min,
            expected,
        )
    return config
