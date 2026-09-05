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
| `rust-packages/` | the **AGS4 Rust workspace** — every shipped crate (the engine, validator, emit, the PyO3 + napi glue, wasm, the forge/parity/perf harnesses) + the `lat` binary | the source of truth for the toolchain; mapped in [[crate-map]]. The experimental `.ags5db` crates were decoupled out of this tree |
| `packages/` | the **Python package**: `laterite` (the AGS4 base wheel) | thin Python over the Rust wheel; the AGS5 packages were decoupled out of this tree |
| `web/` | the **web validator app** (browser SQL-over-AGS via the wasm build) | its own front-end project |
| `ags-wiki/` | the **knowledge base** (this vault) — rules, groups, the O-N divergence catalogue, design decisions, concepts | LLM-maintained; see `AGS-WIKI.md` |
| `docs/` | `docs/agents/` — the agent-skill configuration docs (issue tracker, triage labels, domain layout) — plus the parity coverage map | small on purpose; the knowledge base itself is `ags-wiki/`, and the dated-report archive this row once promised (`docs/history/`) never existed in this repo |
| `tools/` | dev/build/release/perf scripts; **`tools/release/`** is the publish machinery — version bumping, the wasm artifact gates, the package-contents manifest, trusted publishing, the public-API snapshots | every script is CI-wired or a documented utility |
| `examples/` | runnable example scripts | the AGS5 examples (`create_ags5db`, `create_agsx`, `benchmark_scale`, …) went with the decouple, out of this tree |
| `assets/` | brand icons (the curated/derived set the README + release pipeline use) | raw art exports are *not* kept here |
| `tests/` | the **root pytest suite** — `test_dictionary_faithful.py` (the AGS4 dictionary-union faithfulness gate) | runs in CI's python job; the AGS5x/packaging/xml tests went with the decouple, out of this tree |
| `.github/` | CI workflows + composite actions (`build-accel`) + Dependabot | self-hosted-runner-aware; see ci-and-runners |

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

Each terse directory (`tests/`, `examples/`, `assets/`) is a distinct,
justified unit, and the paths are **encoded across the repo** — CI workflows,
wiki `repo_refs`, and the release manifests under `tools/release/`. A top-level
move would touch many files (the root `tests/`
alone is referenced in ~47) to break references for a cosmetic gain. The fix for
"feels busy" is this map — *legibility over relocation*. If a specific consolidation
is ever worth it, do it surgically with its blast radius on the table (and steer
clear of `tests/`).
