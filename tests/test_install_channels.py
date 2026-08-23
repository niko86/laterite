"""The landing page's install grid must not advertise something unpublished.

#395 puts five install cards on laterite.dev — one per distribution channel —
and each names a package a stranger is about to type. A landing page naming a
package the registry does not have is worse than no landing page: the reader
concludes the project is broken, and nothing in the repo fails when the name
goes stale, because markup has no gate.

So the cards are generated from the manifests rather than typed, and this is the
half that bites. `gen_install_channels.py` resolves each card's package name from
the file that decides it, and `unpublished_claims()` refuses to emit a card whose
claim the repo contradicts. The refusals are exercised in both directions here —
a check written and never falsified is how the wiki's crate cards came to open
"internal implementation detail" over nine `publish = true` crates.

The live one, on the day this was written: #395's own text says "the CLI on
crates.io". `laterite-cli` is `publish = false` and has never been on crates.io;
what ships the `lat` binary is the wheel (`[project.scripts]`) and the GitHub
release. The card says so because this refused to let it say otherwise.
"""

from __future__ import annotations

import importlib.util
import json
import re
import sys
import tomllib
from pathlib import Path

REPO = Path(__file__).resolve().parents[1]


def _load(name: str, path: Path):
    spec = importlib.util.spec_from_file_location(name, path)
    assert spec and spec.loader
    mod = importlib.util.module_from_spec(spec)
    sys.modules[name] = mod
    spec.loader.exec_module(mod)
    return mod


gic = _load("gen_install_channels", REPO / "tools" / "gen_install_channels.py")


def _sources() -> dict[str, str]:
    """The source files, read once, as the {repo-relative path: text} the
    resolver takes — so a test can mutate one and re-run the real logic."""
    return {rel: (REPO / rel).read_text(encoding="utf-8") for rel in gic.SOURCES}


# --- the tree as it stands ------------------------------------------------


def test_the_tree_has_no_unpublished_claims() -> None:
    assert gic.unpublished_claims(_sources()) == []


def test_every_card_resolves_a_package_name() -> None:
    for card in gic.resolve(_sources()):
        assert card.package, f"{card.id} card resolved an empty package name"


def test_the_python_and_node_names_come_from_their_manifests() -> None:
    """Not a restatement of the constants — the point is that these two cards
    read the same files PyPI and npm publish from."""
    src = _sources()
    by_id = {c.id: c for c in gic.resolve(src)}
    pyproject = tomllib.loads(src["packages/laterite/pyproject.toml"])
    assert by_id["python"].package == pyproject["project"]["name"]
    node = json.loads(src["rust-packages/laterite-node/package.json"])
    assert by_id["node"].package == node["name"]


# --- the refusals, each falsified ----------------------------------------


def test_the_cli_card_may_not_claim_crates_io() -> None:
    """The live spec error. `laterite-cli` is `publish = false`; a card sending
    a reader to `cargo install laterite-cli` sends them to a 404."""
    claims = gic.unpublished_claims(
        _sources(), cards=[gic.CLI_ON_CRATES_IO_FOR_THE_TEST]
    )
    assert any("laterite-cli" in c and "publish = false" in c for c in claims), (
        "a card claiming crates.io for an unpublished crate went unnoticed"
    )


def test_a_card_naming_a_package_its_manifest_does_not_is_caught() -> None:
    """The drift that arrives without anyone editing the page: the manifest
    moves underneath it (the mechanism behind every 2026-08-04 audit finding)."""
    src = _sources()
    src["rust-packages/laterite-node/package.json"] = json.dumps(
        {**json.loads(src["rust-packages/laterite-node/package.json"]), "name": "lat3"}
    )
    claims = gic.unpublished_claims(src)
    assert any("node" in c and "lat3" in c for c in claims), (
        "renaming the npm package left the card advertising the old name"
    )


def test_the_cli_card_is_caught_when_the_wheel_stops_shipping_lat() -> None:
    """The card's whole claim is "bundled with the wheel". Drop the console
    script and the sentence is false while the card still prints it."""
    src = _sources()
    src["packages/laterite/pyproject.toml"] = src[
        "packages/laterite/pyproject.toml"
    ].replace('lat = "laterite._cli:main"', "")
    claims = gic.unpublished_claims(src)
    assert any("lat" in c and "scripts" in c for c in claims), (
        "the wheel dropping its `lat` entry point left the CLI card unchallenged"
    )


def test_the_browser_card_tracks_the_name_the_release_script_publishes() -> None:
    """`@laterite/ags4-wasm` is set in one place — prepare-wasm-package.sh
    rewrites wasm-pack's crate-derived name to it. That literal is the record."""
    src = _sources()
    src["tools/release/prepare-wasm-package.sh"] = src[
        "tools/release/prepare-wasm-package.sh"
    ].replace('PUBLISHED_NAME="@laterite/ags4-wasm"', 'PUBLISHED_NAME="@laterite/ags4"')
    claims = gic.unpublished_claims(src)
    assert any("browser" in c for c in claims), (
        "the published wasm name moved and the card kept the old one"
    )


def test_the_shipped_readme_and_the_grid_name_the_same_packages() -> None:
    """Two records of what this project publishes — the wheel's README surfaces
    table and the landing grid. They may differ in wording; they may not differ
    in package name."""
    src = _sources()
    src["packages/laterite/README.md"] = src["packages/laterite/README.md"].replace(
        "INSTALL laterite_ags4 FROM community", "INSTALL laterite_ags5 FROM community"
    )
    claims = gic.unpublished_claims(src)
    assert any("duckdb" in c for c in claims), (
        "the README and the install grid disagreed about the DuckDB extension "
        "and neither side failed"
    )


# --- the generated artefact ----------------------------------------------


def test_the_committed_typescript_is_what_the_generator_emits() -> None:
    """The drift gate's own assertion, so a stale artefact fails in the Python
    lane too and not only where `--check` is wired."""
    emitted = gic.render(gic.resolve(_sources()))
    committed = (REPO / gic.OUT).read_text(encoding="utf-8")
    assert committed == emitted, (
        f"{gic.OUT} is stale — run "
        "`uv run --no-project python tools/gen_install_channels.py`"
    )


def test_the_cli_card_leads_with_the_binary_not_pip() -> None:
    """#533: the standalone binary is THE CLI. No pip command as the card's
    headline — an empty command tells the grid to render the releases download
    as the card's action — and wheel/npm inclusion is the note, phrased as
    included, never as identical (three `lat` programs exist and only their
    scriptable output is gated; #509 tracks the Node launcher's gap)."""
    cards = {c.id: c for c in gic.resolve(_sources())}
    cli = cards["cli"]
    assert cli.command == ""
    assert cli.href == "https://github.com/niko86/laterite/releases"
    assert "wheel" in cli.note and "npm" in cli.note
    assert "identical" not in cli.note


def test_the_cli_card_is_caught_when_npm_stops_shipping_lat() -> None:
    """The #533 note's npm half is a claim with a record behind it: strip the
    `bin` entry from laterite-node's manifest and the refusal must fire."""
    src = _sources()
    manifest = json.loads(src["rust-packages/laterite-node/package.json"])
    manifest.pop("bin", None)
    src["rust-packages/laterite-node/package.json"] = json.dumps(manifest)
    claims = gic.unpublished_claims(src)
    assert any("npm" in c and c.startswith("cli:") for c in claims)


# --- the surface hues (#595) ----------------------------------------------


def test_every_card_carries_a_well_formed_hue_pair() -> None:
    """#595: each card borders and washes in its surface's own hue, one value
    per theme. A malformed hex would render as no border at all — CSS drops
    the declaration silently."""
    for card in gic.resolve(_sources()):
        assert re.fullmatch(r"#[0-9a-f]{6}", card.hue_light), card.id
        assert re.fullmatch(r"#[0-9a-f]{6}", card.hue_dark), card.id
        assert card.hue_light != card.hue_dark, (
            f"{card.id}: one hue for both themes — the dark value must be its "
            "own tuning, not a copy"
        )


def test_the_five_hues_are_five_hues() -> None:
    """The whole point of #595 is that no card reads as favoured or generic:
    a duplicated hue would put two surfaces in one dress."""
    cards = gic.resolve(_sources())
    assert len({c.hue_light for c in cards}) == len(cards)
    assert len({c.hue_dark for c in cards}) == len(cards)
