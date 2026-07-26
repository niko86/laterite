---
type: concept
title: repository layout
status: drafted
tags: [concept, architecture]
ags_editions: []
repo_refs:
  root: "repo:."
  workspace: "repo:rust-packages/Cargo.toml"
  packages: "repo:packages/"
related: [start-here, crate-map, dec-monorepo-structure, dec-duckdb-extension]
sources: []
---
# repository layout

## Definition

The map of the **top-level directories** — what each one is for, so the structure
reads as deliberate rather than busy. [[crate-map]] is the zoom-in on the *crates*;
this is the zoom-out on the *repo*. A few directories hold only a handful of files,
but each is a distinct, single-purpose unit (a deployable, a data set, a tool group)
— terse, not disordered.

## Top-level directories

| dir | what's in it | notes |
|---|---|---|
| `rust-packages/` | the **AGS4 Rust workspace** — every shipped crate (the engine, validator, emit, the PyO3 + napi glue, wasm, the forge/parity/perf harnesses) + the `lat` binary | the source of truth for the toolchain; mapped in [[crate-map]]. The experimental `.ags5db` crates moved to `ags5/` (dec-ags5-decouple) |
| `packages/` | the **Python package**: `laterite` (the AGS4 base wheel) | thin Python over the Rust wheel; the AGS5 packages moved to `ags5/` (dec-ags5-decouple) |
| `web/` | the **web validator app** (browser SQL-over-AGS via the wasm build) | its own front-end project |
| `ags-wiki/` | the **knowledge base** (this vault) — rules, groups, the O-N divergence catalogue, design decisions, concepts | LLM-maintained; see `AGS-WIKI.md` |
| `docs/` | engineering docs (parity map, merge semantics, design reviews, release runbooks) and **`docs/history/`** — the dated-report / retired-pipeline archive (perf matrices, benchmarks) | `docs/history/` is the *chronological record*, not a relic graveyard |
| `tools/` | dev/build/release/perf scripts; **`tools/release/`** is the public-mirror machinery (allowlist/blocklist + leak gate + ref rewriter) | every script is CI-wired or a documented utility |
| `examples/` | runnable example scripts | the AGS5 examples (`create_ags5db`, `create_agsx`, `benchmark_scale`, …) moved to `ags5/examples/` with the decouple (dec-ags5-decouple) |
| `assets/` | brand icons (the curated/derived set the README + release pipeline use) | raw art exports are *not* kept here |
| `demo/` | the **GitHub Pages HTTP demo** (`.nojekyll` + `index.html`) | a self-contained deployable — correct as its own dir |
| `reports/` | the AGS-L `AGSL4_2_*.xlsx` dictionary references + the `.agsx` compression decision trail | **immutable provenance** the dictionary scaffolder reads and the wiki cites as authorities |
| `experiments/` | the **non-production** dictionary scaffolders (`scaffold_ags4_dict`, `merge_ags4_into_dict`, `backfill_dict_units`) | deliberately separate from shipped code |
| `ags5/` | the **dormant experimental-AGS5 holding folder** — the decoupled `.ags5db`/`.agsx` crates, packages, tests, examples + the retained `ags5_dictionary.json` AGS5 record | preserved intact, out of the workspace; a future AGS5 strand re-links it (dec-ags5-decouple); blocklisted from the public mirror |
| `tests/` | the **root pytest suite** — `test_dictionary_faithful.py` (the AGS4 dictionary-union faithfulness gate) | runs in CI's python job; the AGS5x/packaging/xml tests moved to `ags5/tests/` (dec-ags5-decouple) |
| `.github/` | CI workflows + composite actions (`build-accel`) + Dependabot | self-hosted-runner-aware; see ci-and-runners |
| `.agents/` | vendored agent skills (`skills/`) | dev-tooling, blocklisted from the public tree |

## Root files

The repo root carries the conventional project files (`README.md`, `LICENSE`,
`CHANGELOG.md`, `CONTRIBUTING.md`, `pyproject.toml`, `uv.lock`, `.gitignore`) plus
the **authority docs**: `OBSERVATIONS.md` (the canonical O-N divergence catalogue), `COMPAT.md` (python-ags4
parity decisions), and the `RELEASING*.md` runbooks. These are deliberately at root
because they're heavily cross-referenced — moving them would ripple through the
allowlist, CI, and wiki `repo_refs` for a purely cosmetic gain.

## Gitignored working space (not in git)

| dir | what it is |
|---|---|
| `output/` | scratch/working space — generated artifacts, **and the `laterite-duckdb` ship-repo working tree** (its own git repo; see [[dec-duckdb-extension]]). Regenerable; safe to clear except the ship repo |
| `testingeverything/` | a vendored third-party AGS validator (Java via IKVM DLLs) — left untouched, used only as an oracle |
| `target/`, `.venv/`, `__pycache__/`, caches | standard build/test artifacts |

## Why not reorganise the sparse dirs?

Each terse directory (`reports/`, `experiments/`, `tests/`, `demo/`) is a distinct,
justified unit, and the paths are **encoded across the repo** — the public
allowlist (`tools/release/public-allowlist.txt`), CI workflows, and wiki `repo_refs`. A top-level move would touch many files (the root `tests/`
alone is referenced in ~47) to break references for a cosmetic gain. The fix for
"feels busy" is this map — *legibility over relocation*. If a specific consolidation
is ever worth it, do it surgically with its blast radius on the table (and steer
clear of `tests/`).
