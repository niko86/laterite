#!/usr/bin/env python3
"""Generate `ags-wiki/concepts/crate-dependency-graph.md` from the workspace
Cargo manifests — the complete, always-true crate dependency graph.

SSOT = `rust-packages/*/Cargo.toml`. The page is fully generated and
faithfulness-gated (`tests/test_crate_graph_faithful.py`: committed == render()),
the same shape as `gen_reference_groups` / `gen_observations`. Two things keep it
tied into the wiki's rigid structure rather than floating beside it:

  * **One coupling truth.** `crate_deps()` here mirrors `lint.py::_crate_deps()`
    exactly (workspace-members list, all dep sections, dep names). An agreement
    test asserts the two Cargo readers are byte-identical, so the generator can
    never drift from the crate-map edge-correctness hard check that reads the
    same data. (lint.py is deliberately self-contained / stdlib-only, so a shared
    import is avoided in favour of a gate — the established multi-source pattern.)
  * **Complement, not duplicate.** `crate-map.md` stays the hand-curated keystone
    (a readable flowchart that omits edges for clarity); THIS page is the complete
    machine-derived view. They cross-link.

It also owns two SPLICED regions elsewhere, both rendered from the same
manifests this file already parses:

  * **`crate-card`** in each `ags-wiki/tools/<crate>.md` — distribution, version
    (own line vs workspace-inherited) and consumers. Fifteen of those pages once
    opened "Internal implementation detail — not a public API" while nine of the
    crates were `publish = true`.
  * **`availability`** in each publishable crate's `README.md` — the `cargo add`
    line. Eleven of them shipped to crates.io saying nothing about installation.
    For the ten that are not the facade it also carries the engine tier's
    **anti-promise** (`_engine_banner`): machinery, not a door.

Both come from `distribution()`: computed once, rendered twice, so the wiki and
the shipped README cannot state different things about the same crate. A README
is frozen at publish time, which is why the check has to fail before an upload
rather than after one.

Run: `uv run --no-project python tools/gen_crate_graph.py` (stdlib only — tomllib).
`--check` exits non-zero if the page, any card or any availability block is
stale (the CI gate).
"""

from __future__ import annotations

import re
import sys
import tomllib
from pathlib import Path

REPO = Path(__file__).resolve().parents[1]
RUST = REPO / "rust-packages"
PAGE = REPO / "ags-wiki" / "concepts" / "crate-dependency-graph.md"

# Intended layering (curated architecture SSOT: lower = more foundational). A
# crate missing here renders as "?" and is flagged, forcing a deliberate
# assignment when a crate is added.
LAYER = {
    "laterite-ags4-types": 0,
    "laterite-ags4-parse": 0,
    "laterite-ags4-reference": 0,
    "laterite-transport": 0,
    "laterite-cliutil": 0,
    "laterite-ags4-core": 1,
    "laterite-ags4-emit": 2,
    "laterite-ags4-validator": 2,
    "laterite-ags4-excel": 2,
    "laterite-ags4-diff": 2,
    "laterite-ags4-merge": 2,
    "laterite-ags4-censor": 2,
    # The trust model composes core (the .ags.idx format) + the validator (the engine)
    # into ONE certificate decision. It sits above both and below every surface, which
    # is the same slot the tools occupy — it is not itself a tool.
    "laterite-ags4-trust": 3,
    "laterite-ags4-parity": 3,
    "laterite-ags4-forge": 3,
    "laterite-ags4-corpus-qa": 3,
    "laterite-ags4-perf": 3,
    "laterite-ags4-compliance": 3,
    "laterite-ags4-xcheck": 3,
    "laterite-py": 4,
    "laterite-node": 4,
    "laterite-ags4-wasm": 4,
    "laterite-cli": 4,
    "laterite-ags4-tokenizer-wasm": 4,
}
LAYER_NAME = {
    # NOT "leaves (no internal deps)" — that was false from the moment
    # `laterite-ags4-reference` took its `laterite-ags4-types` edge, and it was the
    # SECOND copy of the claim (the first sat in reference's own Cargo.toml
    # header). Both were prose; nothing compared either to the manifests. What
    # is actually true — and what the inversion check below enforces — is that
    # an L0 crate depends on nothing above L0.
    0: "L0 · foundations (depend on nothing above L0)",
    1: "L1 · core (data foundation)",
    2: "L2 · writer / validator / excel / diff",
    3: "L3 · trust model + tools",
    4: "L4 · surfaces + CLI",
}

# Reviewed structural notes — the verified output of the multi-lens architectural
# sweep (crate-graph-analysis workflow). Editorial SSOT: edit + regenerate. The
# *computed* findings below (inversions / dev-cycle / hubs) come from the graph
# itself and update automatically; these are the interpretation.
NOTES = [
    (
        "`core`'s `laterite-ags4-types` re-export is dead weight",
        "minor · cheap",
        "`core/src/lib.rs`'s `pub use laterite_ags4_types as ags_types;` is used "
        "internally nowhere — only by `laterite-py`, which already depends on "
        "`laterite-ags4-types` directly. Cuttable with a one-line import change.",
    ),
    (
        "#441 (`core → emit`) — CUT 2026-07-11",
        "resolved",
        "The edge is gone: `core` depended on `emit` solely for `impl From<EmitError> "
        "for CliError`, whose one consumer (`laterite-ags4-excel`) now owns the mapping. "
        "Cutting that shim fully severed the edge (an earlier worry that it was "
        "'heavier than the shim framing' was misplaced — the shim was core's only use "
        "of `emit`) and, as a bonus, broke the former "
        "`validator ⇄ core → emit → validator` dev-dep cycle. See "
        "[[core-emit-layering-inversion]].",
    ),
    (
        "`emit → validator` is a legitimate same-layer edge",
        "informational",
        "Within L2, `validator` (dictionary + rules) is architecturally the lowest "
        "member; `emit`/`diff`/`excel` sit above it with no ship-direction cycle. "
        "Named explicitly so the L2 list order is never misread as an inversion.",
    ),
    (
        "wasm `getrandom` is a proc-macro build artifact, not a leak",
        "known-benign",
        "The only path to `getrandom` under `wasm32-unknown-unknown` is via "
        "`const-random-macro` (a host-compiled proc-macro) through `ahash ← arrow`, "
        "so it never reaches the wasm target at runtime.",
    ),
]

RELATED = ["crate-map", "pyo3-boundary", "tech-stack-wasm", "reliquary"]


def _members() -> list[str]:
    ws = tomllib.loads((RUST / "Cargo.toml").read_text())
    return ws.get("workspace", {}).get("members", [])


def _manifests() -> dict[str, dict]:
    """name -> parsed Cargo.toml for each workspace member that has a package."""
    out: dict[str, dict] = {}
    for m in _members():
        mct = RUST / m / "Cargo.toml"
        if not mct.exists():
            continue
        d = tomllib.loads(mct.read_text())
        nm = d.get("package", {}).get("name")
        if nm:
            out[nm] = d
    return out


def crate_deps() -> dict[str, set[str]]:
    """MIRRORS `lint.py::_crate_deps()` EXACTLY — {name: {all dep names across
    dependencies + build-dependencies + dev-dependencies}}. The agreement-gate
    anchor: `test_crate_graph_faithful` asserts this equals lint's copy, so the
    two Cargo readers cannot drift."""
    out: dict[str, set[str]] = {}
    for nm, d in _manifests().items():
        deps: set[str] = set()
        for sec in ("dependencies", "build-dependencies", "dev-dependencies"):
            deps |= set(d.get(sec, {}).keys())
        out[nm] = deps
    return out


def graph() -> tuple[set[str], dict[str, list[str]], dict[str, list[str]]]:
    """(crate names, ship-edges, dev-only-edges) — internal (workspace) edges
    only. Ship = normal + build deps (what a consumer compiles); dev = test-only
    edges (don't ship) that aren't already ship edges."""
    man = _manifests()
    names = set(man)

    def internal(d: dict, sec: str) -> set[str]:
        return {k for k in d.get(sec, {}) if k in names}

    ship: dict[str, list[str]] = {}
    dev: dict[str, list[str]] = {}
    for nm, d in man.items():
        s = internal(d, "dependencies") | internal(d, "build-dependencies")
        ship[nm] = sorted(s)
        dev[nm] = sorted(internal(d, "dev-dependencies") - s)
    return names, ship, dev


def _closure(start: str, ship: dict[str, list[str]]) -> set[str]:
    seen, stack = set(), [start]
    while stack:
        for y in ship.get(stack.pop(), ()):
            if y not in seen:
                seen.add(y)
                stack.append(y)
    return seen


def _mermaid(
    names: set[str], ship: dict[str, list[str]], dev: dict[str, list[str]]
) -> str:
    def nid(n: str) -> str:
        return n.replace("-", "_")

    lines = ["```mermaid", "flowchart TD"]
    # nodes grouped into a subgraph per layer (foundations last = lowest)
    for lyr in sorted(LAYER_NAME, reverse=True):
        members = sorted(n for n in names if LAYER.get(n) == lyr)
        if not members:
            continue
        lines.append(
            f'  subgraph {LAYER_NAME[lyr].split(" · ")[0].replace(" ", "")}["{LAYER_NAME[lyr]}"]'
        )
        lines.extend(f"    {nid(n)}[{n}]" for n in members)
        lines.append("  end")
    # unlayered crates (a new crate with no LAYER entry) still get a node
    lines.extend(
        f"  {nid(n)}[{n} ?]" for n in sorted(n for n in names if n not in LAYER)
    )
    # ship edges
    for a in sorted(ship):
        lines.extend(f"  {nid(a)} --> {nid(b)}" for b in ship[a])
    # dev-only edges, dashed
    for a in sorted(dev):
        lines.extend(f"  {nid(a)} -.dev.-> {nid(b)}" for b in dev[a])
    lines.append("```")
    return "\n".join(lines)


def _table(names: set[str], ship: dict[str, list[str]]) -> str:
    indeg = dict.fromkeys(names, 0)
    for a in ship:
        for b in ship[a]:
            indeg[b] += 1
    trans = {n: _closure(n, ship) for n in names}
    rows = [
        "| crate | layer | ship-deps (out) | dependents (in) | transitive |",
        "|---|---|--:|--:|--:|",
    ]
    # order: layer asc, then in-degree desc, then name
    for n in sorted(names, key=lambda n: (LAYER.get(n, 99), -indeg[n], n)):
        ly = LAYER_NAME.get(LAYER.get(n), "L? · unlayered").split(" · ")[0]
        rows.append(f"| `{n}` | {ly} | {len(ship[n])} | {indeg[n]} | {len(trans[n])} |")
    return "\n".join(rows)


def _inversions(ship: dict[str, list[str]]) -> list[str]:
    """Ship edges pointing at a HIGHER layer.

    Hoisted out of `_computed_findings` so the "Structural notes" prose can be
    DERIVED from this list rather than assert its own copy: the prose used to
    hard-code "layering respected" directly beneath the section that computes
    these, so a real inversion would have been listed and denied on one page.
    """
    out = []
    for a in sorted(ship):
        for b in ship[a]:
            la, lb = LAYER.get(a, 99), LAYER.get(b, 99)
            if la < 99 and lb < 99 and lb > la:
                out.append(f"`{a}` (L{la}) → `{b}` (L{lb})")
    return out


def _layering_verdict(ship: dict[str, list[str]]) -> str:
    """The layering claim, worded from the graph itself."""
    inv = _inversions(ship)
    if not inv:
        return "the graph respects its layering (no ship edge points at a higher layer)"
    return (
        f"**{len(inv)} layering inversion(s)** — the graph does NOT currently "
        f"respect its layering; see the computed findings above"
    )


def _computed_findings(
    names: set[str], ship: dict[str, list[str]], dev: dict[str, list[str]]
) -> str:
    inversions = _inversions(ship)
    dev_cycles = []
    for a in sorted(dev):
        # b ships back to a → a→b(dev)→…→a cycle
        dev_cycles.extend(
            f"`{a}` –dev→ `{b}`, and `{b}` ships back to `{a}` "
            f"(a Cargo-legal dev-only cycle)"
            for b in dev[a]
            if a in _closure(b, ship)
        )
    indeg = dict.fromkeys(names, 0)
    for a in ship:
        for b in ship[a]:
            indeg[b] += 1
    # Tie-break on the name. `names` is a SET, and sorting a set by in-degree
    # alone leaves ties in hash-iteration order — which differs BETWEEN processes
    # (PYTHONHASHSEED), so the page rendered differently run to run and the
    # `--check` gate became a coin flip. Latent until the reference leaf's
    # laterite-ags4-types edge (#448) created the graph's first in-degree tie. The
    # table at the top of this function already tie-breaks this way.
    hubs = [
        f"`{n}` (in-degree {indeg[n]})"
        for n in sorted(names, key=lambda n: (-indeg[n], n))
        if indeg[n] >= 6
    ]
    unlayered = sorted(n for n in names if n not in LAYER)

    def block(title: str, items: list[str], empty: str) -> str:
        if not items:
            return f"- **{title}:** {empty}"
        return f"- **{title}:**\n" + "\n".join(f"  - {i}" for i in items)

    parts = [
        block(
            "Layering inversions (ship edge to a higher layer)",
            inversions,
            "none — the graph respects its layering.",
        ),
        block(
            "Dev-only cycles (latent — would cycle if promoted to a ship dep)",
            dev_cycles,
            "none.",
        ),
        block("Hubs (in-degree ≥ 6)", hubs, "none."),
    ]
    if unlayered:
        parts.append(
            block(
                "Crates with no layer assignment (add one to `gen_crate_graph.py`)",
                [f"`{n}`" for n in unlayered],
                "",
            )
        )
    return "\n".join(parts)


def render() -> str:
    names, ship, dev = graph()
    fm = [
        "---",
        "type: concept",
        "title: crate dependency graph",
        "status: reviewed",
        "tags: [concept, architecture]",
        "ags_editions: []",
        "repo_refs:",
        '  workspace: "repo:rust-packages/Cargo.toml"',
        '  generator: "repo:tools/gen_crate_graph.py"',
        f"related: [{', '.join(RELATED)}]",
        "sources: []",
        "---",
    ]
    body = [
        "",
        "# Crate dependency graph",
        "",
        "<!-- GENERATED by tools/gen_crate_graph.py from the workspace Cargo",
        "     manifests. Do NOT hand-edit — run",
        "     `uv run --no-project python tools/gen_crate_graph.py`.",
        "     Faithfulness-gated by tests/test_crate_graph_faithful.py. -->",
        "",
        "> [!note] Generated from `repo:rust-packages/Cargo.toml` (+ each member's",
        "> manifest). The complete machine-derived internal dependency graph — the",
        "> counterpart to [[crate-map]], which is the hand-curated, readability-first",
        "> keystone view. This page can't drift: a faithfulness gate re-renders it",
        "> from the manifests, and its coupling read is asserted byte-identical to",
        "> the `lint.py` crate-map edge check's.",
        "",
        "Solid arrows are **ship** dependencies (normal + build — what a consumer",
        "compiles); the dashed `dev` arrow is a **test-only** edge that doesn't",
        "ship. Every edge here is a real Cargo coupling by construction.",
        "",
        "## The graph",
        "",
        _mermaid(names, ship, dev),
        "",
        "## Crates by layer",
        "",
        _table(names, ship),
        "",
        "## Structural findings (computed from the manifests)",
        "",
        _computed_findings(names, ship, dev),
        "",
        "## Structural notes (reviewed)",
        "",
        # This sentence used to hard-code "layering respected" one section below
        # the code that COMPUTES the inversions — so an inversion would have been
        # listed above and denied here, on the same generated page. The layering
        # clause is now derived from that same list. The two claims it also made
        # are attributed rather than asserted: a shipped cycle is not something
        # this generator checks (cargo rejects one at resolve time, so the graph
        # cannot express it), and the wasm boundary is a REVIEWED judgement here —
        # what is mechanically enforced is per-crate leaf purity, by
        # `laterite-ags4-parse`'s `dep_graph.rs` and the validator's
        # `lean_dep_graph.rs`, not anything global.
        f"From the multi-lens architectural sweep. Computed from the manifests: "
        f"{_layering_verdict(ship)}. A shipped cycle is impossible by construction "
        f"(cargo rejects circular normal deps). The wasm boundary is reviewed, not "
        f"computed here — leaf purity is pinned per-crate by `dep_graph.rs` "
        f"(parse) and `lean_dep_graph.rs` (validator). These are the interpretation:",
        "",
    ]
    for i, (title, sev, detail) in enumerate(NOTES, 1):
        body.append(f"{i}. **{title}** — *{sev}*.  {detail}")
    body += [
        "",
        "## Related",
        "",
        " · ".join(f"[[{r}]]" for r in RELATED),
        "",
    ]
    return "\n".join(fm) + "\n" + "\n".join(body)


# ---------------------------------------------------------------------------
# Crate cards — the facts a crate's page states ABOUT ITSELF, printed from its
# manifest instead of typed.
#
# The 2026-08-04 audit found fifteen tool pages opening "Internal implementation
# detail — a workspace crate, not a public API". Nine of them were `publish =
# true`. Nobody edited those pages; the manifests moved underneath them, which is
# why a wiki-diff-triggered check would have caught none of it and why this rides
# `rust-packages/**` in ci.yml's `code` filter instead.
#
# ## Why the card says "cleared for crates.io", not "published"
#
# Whether a version is actually ON the registry is not knowable offline, and a
# gate that needs the network is a gate that fails on a flaky DNS. Worse, the two
# states genuinely differ right now: `laterite-ags4-diff` and `-merge` are armed
# `publish = true` but their first upload has not happened. Asserting "published"
# for them would replace one false claim with another — the decided-but-not-yet-
# executed trap. So the card states what the manifest says, because the manifest
# is the thing this repo controls and the thing this gate can verify.
#
# ## Why module lists are NOT here
#
# Several pages annotate their module lists with prose that is the page's actual
# value. Generating a bare list beside that annotated prose would leave the wrong
# text in place with a green gate — strictly worse than today. Round-tripping the
# annotations is a separate change with its own idempotence proof.
# ---------------------------------------------------------------------------
CARD = "crate-card"
TOOLS = REPO / "ags-wiki" / "tools"

#: `repo_refs.root` pointing at a workspace member, inside a page's frontmatter.
_ROOT_REF = re.compile(
    r'^\s+root:\s*"repo:rust-packages/([A-Za-z0-9_-]+)/?"\s*$', re.MULTILINE
)


def _page_for_crate() -> dict[str, Path]:
    """crate directory -> the tool page rooted at it.

    Keyed on `repo_refs.root`, not on the filename: three different things in
    this repo are called `laterite` (the wheel, the facade crate, the import
    root), so a stem lookup silently lands on the wrong artifact.
    """
    out: dict[str, Path] = {}
    for page in sorted(TOOLS.glob("*.md")):
        head = page.read_text(encoding="utf-8").split("---", 2)
        if len(head) < 3:
            continue
        m = _ROOT_REF.search(head[1])
        if m:
            out.setdefault(m.group(1), page)
    return out


def _version_of(d: dict) -> tuple[str, bool]:
    """(version, inherited) — resolving `version.workspace = true`."""
    v = d.get("package", {}).get("version")
    if isinstance(v, str):
        return v, False
    ws = tomllib.loads((RUST / "Cargo.toml").read_text())
    return ws["workspace"]["package"]["version"], True


def distribution(name: str, man: dict[str, dict]) -> dict:
    """Where `name` is installable from, and at what version.

    Computed ONCE and rendered TWICE — into the crate's wiki card and into its
    shipped README. Half the audited defects lived outside `ags-wiki/`, and a
    README that ships to a registry is frozen at publish time: a wrong install
    line on crates.io cannot be corrected retroactively, only superseded by
    another release. Two renderings of one computation is what stops the wiki
    and the README disagreeing about the same crate.
    """
    d = man[name]
    version, inherited = _version_of(d)
    return {
        "name": name,
        "version": version,
        "inherited": inherited,
        "crates_io": d.get("package", {}).get("publish") is not False,
    }


def _card_lines(name: str, man: dict[str, dict], consumers: list[str]) -> list[str]:
    dist = distribution(name, man)
    version, inherited = dist["version"], dist["inherited"]
    tier = "inherited from the workspace" if inherited else "its own line"
    if not dist["crates_io"]:
        head = (
            f"> [!note] **Not published** — `{name}` is a workspace crate, "
            f"internal to this repo, at v{version} ({tier})."
        )
    else:
        head = (
            f"> [!note] **Cleared for crates.io** — `{name}` v{version} "
            f"({tier}) declares `publish = true`, so it is a public API under "
            f"semver, not an internal detail."
        )
    used = (
        ", ".join(f"[[{c}]]" for c in consumers)
        if consumers
        else "nothing else in this workspace"
    )
    return [head, f"> **Used by** — {used}."]


def _splice(md: str, marker: str, body: list[str]) -> str | None:
    """`md` with the region between the markers replaced, or None if it has none.

    None rather than raising: a page opts INTO a card by carrying the markers, so
    a page without them is a normal state to skip, not an error to stop on.
    """
    begin, end = f"<!-- BEGIN GENERATED: {marker}", f"<!-- END GENERATED: {marker}"
    lines = md.split("\n")
    bi = next((i for i, ln in enumerate(lines) if ln.startswith(begin)), None)
    ei = next((i for i, ln in enumerate(lines) if ln.startswith(end)), None)
    if bi is None or ei is None:
        return None
    return "\n".join(lines[: bi + 1] + body + lines[ei:])


def _name_of(crate_dir: str) -> str | None:
    """Package name declared by `rust-packages/<crate_dir>/Cargo.toml`.

    The directory and the package name usually match but are not the same thing,
    and the manifest is the authority for which crate a page is actually about.
    """
    ct = RUST / crate_dir / "Cargo.toml"
    if not ct.exists():
        return None
    return tomllib.loads(ct.read_text()).get("package", {}).get("name")


AVAIL = "availability"

#: The one crate on crates.io that is a DOOR. Everything else this workspace
#: publishes is the engine underneath it — named here rather than inferred from
#: `inherited`, because "is this the door" is a fact about the architecture and
#: not about which version line a crate happens to sit on today (the facade folds
#: onto the product line at `dec-facade-parity` phase 8 and is still the door).
FACADE = "laterite"


def _engine_banner(name: str) -> list[str]:
    """The engine tier's anti-promise, for a crate that is not the facade.

    The facade's stated purpose is to absorb engine reshaping — which says
    nothing about the engine crates themselves holding still, and they don't. A
    reader who found one on crates.io had nothing telling them that, and ten
    hand-written sentences would have been ten things to drift.

    It rides the availability region rather than a marker of its own so that the
    reader about to run `cargo add` meets it ABOVE the command, and so that a
    crate cannot acquire an install line without it.

    `laterite` is skipped: it is the door, and what it has to say — its own
    clock, its own churn — is a different claim in kind, hand-written in its
    README rather than stamped in one shape across eleven crates.
    """
    return [
        f"> **Engine crate, not a door.** `{name}` is machinery inside the laterite",
        "> toolchain, reshaped whenever the toolchain needs it. The Rust door is",
        "> [`laterite`](https://crates.io/crates/laterite); depend on this one directly",
        "> only if that suits you, and expect it to move.",
        "",
    ]


def _availability_lines(dist: dict) -> list[str]:
    """The install block for a crate's shipped README.

    Only crates.io is rendered here. The npm and PyPI surfaces are deliberately
    left hand-written: `packages/laterite/README.md` documents three extras with
    their dependency trade-offs, and `laterite-node/README.md` its own CLI — both
    richer than anything derivable from a manifest, and neither has ever been
    wrong. Generating over them would destroy information to prevent a defect
    that has not occurred.
    """
    banner = [] if dist["name"] == FACADE else _engine_banner(dist["name"])
    return [
        *banner,
        "## Install it",
        "",
        "```bash",
        f"cargo add {dist['name']}",
        "```",
        "",
        f"Currently v{dist['version']}"
        + (
            " — the engine crates move in lockstep on the workspace version."
            if dist["inherited"]
            else " — this crate versions independently of the engine."
        ),
    ]


def readmes() -> dict[Path, str]:
    """shipped README -> its text with the availability region refreshed.

    Keyed on the manifest's `publish`, so a crate armed for the registry without
    its README being regenerated fails the same `--check` as a stale card.
    """
    man = _manifests()
    out: dict[Path, str] = {}
    for crate_dir in sorted(set(_members())):
        name = _name_of(crate_dir)
        if name is None or name not in man:
            continue
        dist = distribution(name, man)
        if not dist["crates_io"]:
            continue
        readme = RUST / crate_dir / "README.md"
        if not readme.exists():
            continue
        spliced = _splice(
            readme.read_text(encoding="utf-8"), AVAIL, _availability_lines(dist)
        )
        if spliced is not None:
            out[readme] = spliced
    return out


def cards() -> dict[Path, str]:
    """page -> its full text with the crate-card region refreshed.

    Pages with no card region are skipped rather than rewritten: a page opts in
    by carrying the markers, so adding one is a deliberate edit, not something
    this tool does to every file it can reach.
    """
    man = _manifests()
    ship = graph()[1]
    out: dict[Path, str] = {}
    for crate_dir, page in _page_for_crate().items():
        name = _name_of(crate_dir)
        if name is None or name not in man:
            continue
        consumers = sorted(c for c, deps in ship.items() if name in deps)
        spliced = _splice(
            page.read_text(encoding="utf-8"), CARD, _card_lines(name, man, consumers)
        )
        if spliced is not None:
            out[page] = spliced
    return out


def main(argv: list[str]) -> int:
    out = render()
    targets = {**cards(), **readmes()}
    kinds = {
        p: (
            "crate-card"
            if p.suffix == ".md" and "ags-wiki" in p.parts
            else "availability"
        )
        for p in targets
    }
    if "--check" in argv:
        rc = 0
        cur = PAGE.read_text() if PAGE.exists() else ""
        if cur != out:
            print(
                "crate-dependency-graph.md is STALE — run "
                "`uv run --no-project python tools/gen_crate_graph.py`",
                file=sys.stderr,
            )
            rc = 1
        stale = [
            p for p, txt in targets.items() if p.read_text(encoding="utf-8") != txt
        ]
        if stale:
            for p in sorted(stale):
                print(
                    f"{p.relative_to(REPO)}: {kinds[p]} region is STALE — its "
                    "manifest says something the file does not",
                    file=sys.stderr,
                )
            print(
                "run `uv run --no-project python tools/gen_crate_graph.py`",
                file=sys.stderr,
            )
            rc = 1
        if rc == 0:
            n_cards = sum(1 for k in kinds.values() if k == "crate-card")
            print(
                f"crate dependency graph OK: page matches render(); "
                f"{n_cards} crate card(s) and {len(targets) - n_cards} README "
                f"availability block(s) current"
            )
        return rc
    PAGE.write_text(out)
    print(f"wrote {PAGE.relative_to(REPO)}")
    for p, txt in sorted(targets.items()):
        if p.read_text(encoding="utf-8") != txt:
            p.write_text(txt, encoding="utf-8")
            print(f"wrote {p.relative_to(REPO)} ({kinds[p]})")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
