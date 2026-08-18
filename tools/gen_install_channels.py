"""Emit the landing page's install cards from the records that decide what this
project publishes, and refuse to emit one the repo contradicts (#395).

The install grid is the most concretely useful part of laterite.dev: five
surfaces exist and none of them is discoverable from a landing page. Every card
names a package a stranger is about to type, which makes it the one region of the
page where being wrong is worse than being absent — a reader who types a name the
registry does not have concludes the project is broken, and nothing in this repo
fails when that name goes stale, because markup has no gate.

## No name is written here

Each channel names TWO independent records and reads the package name out of
them. The emitted card carries record A's value; record B has to agree.

    python    pyproject `[project].name`        README surfaces table
    node      laterite-node package.json        README surfaces table
    cli       pyproject `[project].scripts`     README surfaces table + the crate manifest
    duckdb    README surfaces table             the docs site's INSTALL statement
    browser   prepare-wasm-package.sh           release.yml's published-name assertion

That is ten records for five cards and no third statement of any name. What IS
written here is the command template — `pip install {}` — because the verb is
editorial and the registry has no opinion about it. One place, and not markup.

## The refusal that already bit

#395's own text specified "the CLI on crates.io". `laterite-cli` is
`publish = false` and has never been on crates.io; `cargo install laterite-cli`
is a 404. What ships `lat` is the wheel's `[project.scripts]` entry and the
GitHub release, which is what the wheel's own README has said all along. The card
says so because `unpublished_claims()` would not let it say otherwise — see
`CLI_ON_CRATES_IO_FOR_THE_TEST`, which is that rejected card, kept as the
fixture that proves the refusal still fires.

Usage:
    uv run --no-project python tools/gen_install_channels.py           # write
    uv run --no-project python tools/gen_install_channels.py --check   # gate
"""

from __future__ import annotations

import json
import re
import sys
import tomllib
from dataclasses import dataclass
from pathlib import Path

_REPO = Path(__file__).resolve().parents[1]

#: Repo-relative, and the whole set the resolver may read. Tests hand this back
#: as {path: text} with one entry mutated, so every read goes through it rather
#: than touching disk — a check that can only be exercised against the tree as it
#: happens to stand is a check nobody has falsified.
SOURCES = (
    "packages/laterite/pyproject.toml",
    "packages/laterite/README.md",
    "rust-packages/laterite-node/package.json",
    "rust-packages/laterite-cli/Cargo.toml",
    "tools/release/prepare-wasm-package.sh",
    "web/docs-site/docs/duckdb/index.md",
    ".github/workflows/release.yml",
)

OUT = "web/landing/installChannels.ts"


@dataclass(frozen=True)
class Card:
    id: str
    label: str
    #: The registry a reader would go looking on. "" where there isn't one —
    #: the CLI rides the wheel, and saying "crates.io" there is the bug.
    registry: str
    package: str
    command: str
    href: str
    note: str
    #: True for the one card the grid highlights.
    primary: bool = False


def _text(sources: dict[str, str], rel: str) -> str:
    try:
        return sources[rel]
    except KeyError as err:  # pragma: no cover - a programming error, not drift
        raise SystemExit(f"gen_install_channels: {rel} is not in SOURCES") from err


# --- reading each record --------------------------------------------------


def _surfaces_table(sources: dict[str, str]) -> dict[str, str]:
    """{surface label: row text} from the wheel README's "One engine, every
    stack" table — the project's shipped statement of what it publishes."""
    rows: dict[str, str] = {}
    for line in _text(sources, "packages/laterite/README.md").splitlines():
        m = re.match(r"\|\s*\*\*(?P<label>[^*]+)\*\*\s*\|(?P<rest>.*)$", line.strip())
        if m:
            rows[m.group("label").strip()] = m.group("rest")
    return rows


def _pyproject(sources: dict[str, str]) -> dict:
    return tomllib.loads(_text(sources, "packages/laterite/pyproject.toml"))["project"]


def _wasm_published_name(sources: dict[str, str]) -> str:
    m = re.search(
        r'^PUBLISHED_NAME="([^"]+)"',
        _text(sources, "tools/release/prepare-wasm-package.sh"),
        re.MULTILINE,
    )
    return m.group(1) if m else ""


def _duckdb_extension(sources: dict[str, str]) -> str:
    m = re.search(
        r"INSTALL\s+(\w+)\s+FROM\s+community",
        _text(sources, "packages/laterite/README.md"),
    )
    return m.group(1) if m else ""


def resolve(sources: dict[str, str]) -> list[Card]:
    """The five cards, each name read out of its primary record."""
    project = _pyproject(sources)
    wheel = project["name"]
    node = json.loads(_text(sources, "rust-packages/laterite-node/package.json"))[
        "name"
    ]
    wasm = _wasm_published_name(sources)
    duckdb = _duckdb_extension(sources)
    cli = next(iter(project.get("scripts", {})), "")

    return [
        Card(
            id="python",
            label="Python",
            registry="PyPI",
            package=wheel,
            command=f"pip install {wheel}",
            href=f"https://pypi.org/project/{wheel}/",
            note="polars + duckdb, pyarrow-free",
            primary=True,
        ),
        Card(
            id="node",
            label="Node.js",
            registry="npm",
            package=node,
            command=f"npm install {node}",
            href=f"https://www.npmjs.com/package/{node}",
            note="native addon, four platforms",
        ),
        Card(
            id="cli",
            label="CLI",
            # Deliberately blank. The crate is `publish = false`; the binary
            # rides the wheel and the GitHub release, and a card that named a
            # registry here would name one that 404s.
            registry="",
            package=cli,
            command=f"pip install {wheel}",
            href="https://github.com/niko86/laterite/releases",
            note=f"{cli} ships with the wheel — or take the standalone binary",
        ),
        Card(
            id="duckdb",
            label="DuckDB",
            registry="community extension",
            package=duckdb,
            command=f"INSTALL {duckdb} FROM community;",
            href=f"https://community-extensions.duckdb.org/extensions/{duckdb}.html",
            note="read AGS4 in place, as SQL table functions",
        ),
        Card(
            id="browser",
            label="Browser",
            registry="npm",
            package=wasm,
            command=f"npm install {wasm}",
            href=f"https://www.npmjs.com/package/{wasm}",
            note="the same engine as wasm — or open the web app",
        ),
    ]


#: The card #395 specified and this module refused: `laterite-cli` on crates.io.
#: Kept as the fixture the refusal is falsified against, so "we check for that"
#: stays a fact rather than a claim in a docstring.
CLI_ON_CRATES_IO_FOR_THE_TEST = Card(
    id="cli",
    label="CLI",
    registry="crates.io",
    package="laterite-cli",
    command="cargo install laterite-cli",
    href="https://crates.io/crates/laterite-cli",
    note="",
)


# --- the refusals ---------------------------------------------------------


def _crate_publishes(sources: dict[str, str], crate: str) -> bool:
    rel = f"rust-packages/{crate}/Cargo.toml"
    if rel not in sources:
        return False
    manifest = tomllib.loads(_text(sources, rel))
    return manifest.get("package", {}).get("publish", True) is not False


def unpublished_claims(
    sources: dict[str, str], cards: list[Card] | None = None
) -> list[str]:
    """Every reason a card would advertise something this repo contradicts.

    Empty means the grid may be emitted. Each string names the card and the
    record that disagrees, because a reader chasing this failure needs to know
    which of the two records to fix, not that they differ.
    """
    cards = resolve(sources) if cards is None else cards
    table = _surfaces_table(sources)
    claims: list[str] = []

    for card in cards:
        if not card.package:
            claims.append(
                f"{card.id}: its record no longer carries a package name — the "
                "card would render blank"
            )
            continue

        # A card may not send a reader to crates.io for a crate held back from it.
        if card.registry == "crates.io" and not _crate_publishes(sources, card.package):
            claims.append(
                f"{card.id}: claims crates.io for `{card.package}`, whose manifest "
                "says `publish = false` — the card would link to a 404"
            )

        # Record B: the wheel's own README table, for the three it lists.
        row = table.get(card.label, "")
        if row and card.package not in row:
            claims.append(
                f"{card.id}: reads `{card.package}` from its manifest, but the "
                f"wheel README's surfaces table row for {card.label} does not "
                "name it — the two records of what this project publishes disagree"
            )

    by_id = {c.id: c for c in cards}

    if "cli" in by_id:
        project = _pyproject(sources)
        if "lat" not in project.get("scripts", {}):
            claims.append(
                "cli: the wheel's `[project.scripts]` no longer declares `lat`, so "
                "the card's claim that the CLI ships with the wheel is false"
            )
        if _crate_publishes(sources, "laterite-cli") and not by_id["cli"].registry:
            claims.append(
                "cli: `laterite-cli` is now published — the card should name "
                "crates.io rather than sending readers to the wheel alone"
            )

    if "browser" in by_id:
        release = _text(sources, ".github/workflows/release.yml")
        if f'= "{by_id["browser"].package}"' not in release:
            claims.append(
                f"browser: prepare-wasm-package.sh publishes "
                f"`{by_id['browser'].package}`, which release.yml's published-name "
                "assertion does not expect — the release would fail its own check"
            )

    if "duckdb" in by_id:
        docs = _text(sources, "web/docs-site/docs/duckdb/index.md")
        if by_id["duckdb"].package not in docs:
            claims.append(
                f"duckdb: the README names `{by_id['duckdb'].package}` but the docs "
                "site's INSTALL statement does not — one of them is telling readers "
                "to install an extension that is not there"
            )

    return claims


# --- rendering ------------------------------------------------------------


def render(cards: list[Card]) -> str:
    rows = "\n".join(
        "  {\n"
        + f'    id: "{c.id}",\n'
        + f'    label: "{c.label}",\n'
        + f"    registry: {json.dumps(c.registry)},\n"
        + f"    package: {json.dumps(c.package)},\n"
        + f"    command: {json.dumps(c.command)},\n"
        + f"    href: {json.dumps(c.href)},\n"
        + f"    note: {json.dumps(c.note)},\n"
        + f"    primary: {'true' if c.primary else 'false'},\n"
        + "  },"
        for c in cards
    )
    return f"""// GENERATED by tools/gen_install_channels.py — DO NOT EDIT.
//
// Every package name here is read out of the file that decides it (the wheel's
// pyproject, laterite-node's package.json, prepare-wasm-package.sh, the wheel
// README's surfaces table), and cross-checked against a second record before
// this file is written. Editing it by hand puts a name on laterite.dev that
// nothing in the repo can keep true.
//
// Regenerate: uv run --no-project python tools/gen_install_channels.py

export type InstallChannel = {{
  readonly id: string;
  readonly label: string;
  /** The registry a reader would look on, or "" where there isn't one. */
  readonly registry: string;
  readonly package: string;
  readonly command: string;
  readonly href: string;
  readonly note: string;
  /** The one card the grid highlights. */
  readonly primary: boolean;
}};

export const INSTALL_CHANNELS: readonly InstallChannel[] = [
{rows}
];
"""


def main(argv: list[str]) -> int:
    sources = {rel: (_REPO / rel).read_text(encoding="utf-8") for rel in SOURCES}

    claims = unpublished_claims(sources)
    if claims:
        print("REFUSED — the install grid would advertise this:", file=sys.stderr)
        for c in claims:
            print(f"  - {c}", file=sys.stderr)
        return 1

    emitted = render(resolve(sources))
    out = _REPO / OUT

    if "--check" in argv:
        if not out.exists() or out.read_text(encoding="utf-8") != emitted:
            print(
                f"DRIFT: {OUT} is stale — run "
                "`uv run --no-project python tools/gen_install_channels.py`",
                file=sys.stderr,
            )
            return 1
        print("gen_install_channels: the install grid matches every publish record")
        return 0

    out.write_text(emitted, encoding="utf-8")
    print(f"wrote {OUT}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
