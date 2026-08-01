"""The census schema version is declared in four places and must agree.

`lat census` is the one machine door all three launchers implement, and the
version is how a reader tells "this launcher has no such table" from "this
launcher was built before the table existed". `tools/gen_census.py` refuses a
dump whose version is not exactly its own, so the four declarations agreeing is
what makes that refusal mean staleness rather than a forgotten edit.

Nothing enforced it until now. The constant is hand-copied — the source comments
say "All three launchers declare this; they must agree" and then rely on whoever
bumps one remembering the other three. A bump that reached the Rust census and
missed `_cli.py` would make the uvx launcher permanently "stale" to the
generator, which reads as a build problem rather than as the typo it is.

This reads the SOURCE files, not built launchers, so a half-finished bump fails
here without anything being compiled first.
"""

from __future__ import annotations

import re
from pathlib import Path

import pytest

REPO = Path(__file__).resolve().parents[1]

#: file -> the pattern capturing its census-schema declaration, and why that file
#: is a stamping site at all. The reason is part of the fixture deliberately: a
#: failure should say what breaks, not merely that four numbers differ.
SITES: dict[str, tuple[str, str]] = {
    "rust-packages/laterite-cli/src/commands/census.rs": (
        r"pub const CENSUS_VERSION: u32 = (\d+);",
        "the AUTHORITY dump — the column the other two launchers are diffed against",
    ),
    "packages/laterite/python/laterite/_cli.py": (
        r'"census_version": (\d+),',
        "the uvx launcher's dump",
    ),
    "rust-packages/laterite-node/ts/cli.ts": (
        r"census_version: (\d+),",
        "the npx launcher's dump",
    ),
    "tools/gen_census.py": (
        r"^CENSUS_VERSION = (\d+)$",
        "the generator, which REFUSES any dump not exactly equal to this",
    ),
}


def _declared(rel: str) -> int:
    pattern, _ = SITES[rel]
    text = (REPO / rel).read_text(encoding="utf-8")
    found = re.findall(pattern, text, flags=re.MULTILINE)
    assert found, f"no census version declaration matched in {rel}"
    assert len(found) == 1, f"{rel} declares the census version {len(found)} times"
    return int(found[0])


@pytest.mark.parametrize("rel", sorted(SITES))
def test_every_site_declares_a_census_version(rel: str) -> None:
    """Each stamping site is still findable — the regex has not been outrun."""
    assert _declared(rel) > 0


def test_the_four_declarations_agree() -> None:
    versions = {rel: _declared(rel) for rel in SITES}
    distinct = set(versions.values())
    assert len(distinct) == 1, (
        "the census schema version has drifted across its stamping sites:\n"
        + "\n".join(f"  {v}  {rel}  ({SITES[rel][1]})" for rel, v in versions.items())
        + "\n\nA launcher left behind is not merely inconsistent: `gen_census.py`"
        "\nrefuses a dump whose version is not exactly its own, so it will report"
        "\nthat launcher as built from older sources and tell you to rebuild it —"
        "\nadvice that cannot work, because the number is wrong in the source."
    )
