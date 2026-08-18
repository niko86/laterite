"""Band colour never reaches prose on the docs site (#401).

The site direction keys each top-level docs section to a band from the strata
ramp — the same ramp the apex's borehole rail runs. That keying is only safe
because of one constraint: **band colour appears in the navigation swatches and
rules, the table-of-contents rail, the masthead's closing hairline and the
catalogue's strata cap, and nowhere else.**

This is not a tidiness rule. On a documentation site for a *validator*, the warm
ramp and the severity palette are drawn from the same family, so a rust-tinted
callout in the middle of a page is indistinguishable from an error. Group
identity and severity have to stay in different places on the page or neither
means anything.

The constraint is one a reviewer cannot hold: the check is "does any of ~700
lines of CSS across three stylesheets paint a band somewhere it shouldn't", and
the failure looks like a design choice rather than a bug. So it is a gate.

Read the allowlist below as the definition of "a band-bearing surface". Adding to
it is a design decision about where group identity may appear — which is exactly
the decision that should require an edit here rather than happening by accident.

ALIASES ARE DELIBERATELY NOT FOLLOWED, and the reason has to be stated or it
reads as a hole. `--accent`, `--cta` and `--selection` all resolve INTO the ramp
(`--accent` is literally `var(--laterite-900)`), so a check that chased aliases
would flag every maroon headline and every rust button on the site — including
the ones the direction mandates. That is the tell that the constraint is not
"no colour cut from the ramp". It is "no band used AS GROUP IDENTITY": the ramp
indexed by position, where band N means the Nth group. A role token means one
fixed thing everywhere — read, act, select — and cannot be mistaken for a group,
because it does not vary with one. Hence the direct-reference check: reaching for
`--laterite-500` or `--band` in a rule is the act of picking a band out of the
ramp, and that is what may not happen in prose.
"""

from __future__ import annotations

import re
from pathlib import Path

import pytest

_STYLESHEETS = Path(__file__).resolve().parents[1] / "web/docs-site/docs/stylesheets"

# `--laterite-*` is the seven-band strata ramp; `--band` is the per-section
# variable the left nav sets from it. Either one paints group identity.
_BAND_TOKEN = re.compile(r"--laterite-\d|var\(\s*--band\b")

# A selector may carry band colour when it is one of these surfaces. Matched as
# substrings against the whole selector, so a compound or grouped selector needs
# only to name the surface it belongs to.
_BAND_BEARING = (
    "md-header__strata",  # the hairline closing the masthead
    "md-nav--primary",  # the left nav's per-section swatches
    "md-nav__link--active",  # the active item's inset section rule
    "md-rail",  # the table-of-contents strata rail
    "group-table",  # the catalogue spotlight's border + strata cap
)

_COMMENT = re.compile(r"/\*.*?\*/", re.DOTALL)


def _rules(css: str) -> list[tuple[str, str]]:
    """Every (selector, declarations) pair, including inside at-rules.

    A hand-rolled walk rather than a CSS parser: the dependency would be a new
    one for a single gate, and the shape here is small — brace depth, with
    at-rule blocks transparent so a rule nested in `@media` is still checked
    against its own selector.
    """
    css = _COMMENT.sub("", css)
    out: list[tuple[str, str]] = []
    depth = 0
    start = 0
    stack: list[str] = []
    for m in re.finditer(r"[{}]", css):
        if m.group() == "{":
            prelude = css[start : m.start()].strip()
            stack.append(prelude)
            depth += 1
            start = m.end()
        else:
            prelude = stack.pop() if stack else ""
            depth -= 1
            # An at-rule's block holds rules, not declarations; a plain
            # selector's block holds the declarations we care about.
            if not prelude.startswith("@"):
                out.append((prelude, css[start : m.start()]))
            start = m.end()
    assert depth == 0, "unbalanced braces — the stylesheet does not parse"
    return out


# The generated bundle DEFINES the ramp — `--laterite-500: #ce5640`, and the
# aliases cut from it — so every line of it names a band by construction and
# scanning it for band references would only ever say so. It is excluded, and
# the test below pins that the excluded file really is the generated one, so the
# exclusion cannot be widened into a place to hide a hand-written rule.
_GENERATED = "tokens.css"


def _sheets() -> list[Path]:
    return sorted(p for p in _STYLESHEETS.glob("*.css") if p.name != _GENERATED)


def test_the_stylesheets_are_where_this_gate_thinks_they_are() -> None:
    """A moved or renamed stylesheet must fail loudly, not silently pass.

    A glob that matches nothing is a green test that checks nothing, which is the
    failure mode every gate in this repo is written against.
    """
    names = {p.name for p in _sheets()}
    assert {"laterite.css", "catalogue.css"} <= names, names


@pytest.mark.parametrize("sheet", _sheets(), ids=lambda p: p.name)
def test_band_colour_stays_on_band_bearing_surfaces(sheet: Path) -> None:
    offenders = [
        selector
        for selector, body in _rules(sheet.read_text(encoding="utf-8"))
        if _BAND_TOKEN.search(body)
        and not any(surface in selector for surface in _BAND_BEARING)
    ]
    assert not offenders, (
        f"{sheet.name} paints band colour outside a band-bearing surface:\n"
        + "\n".join(f"  {s}" for s in offenders)
        + "\n\nBand colour encodes GROUP IDENTITY. In prose or an admonition it is "
        "indistinguishable from severity. Either move the rule onto one of "
        f"{list(_BAND_BEARING)}, or use the status/accent tokens instead."
    )


def test_the_excluded_sheet_is_the_generated_one() -> None:
    """Guards the exclusion, not the CSS.

    `tokens.css` is skipped above because it is a machine-written copy of the
    shared token layer and defining the ramp is its whole job. That reasoning
    only holds while the file really is generated — so this pins the banner
    sync-docs-tokens.mjs writes. Hand-authoring a stylesheet under that name
    would otherwise be a way past the gate.
    """
    bundle = _STYLESHEETS / _GENERATED
    assert bundle.is_file(), f"{_GENERATED} is missing — run `npm run sync-docs-tokens`"
    head = bundle.read_text(encoding="utf-8")[:400]
    assert "GENERATED FILE — DO NOT EDIT" in head, (
        f"{_GENERATED} is exempt from the band scan because it is generated; "
        "this copy carries no generated banner."
    )
