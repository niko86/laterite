---
type: decision
title: "One repo, many published artifacts — stay monorepo, don't split docling-style"
status: accepted
tags: [design, decision, architecture]
decided: 2026-06-16
supersedes: []
from_gap: []
related: [crate-map, dec-rust-drives-python, pyo3-boundary, dec-python-imports-rust-library, dec-laterite-ags4-types-leaf, design/_README]
sources: []
---

# One repo, many published artifacts — stay monorepo, don't split docling-style

## Context
An honest recurring question: is this hybrid Rust + Python project better as the
current **monorepo**, or should its components be split into separate repos the
way the [docling](https://github.com/docling-project) project does it
(`docling` / `docling-core` / `docling-parse` / `docling-ibm-models` as
independent repositories, each with its own release cadence)?

The repo today is **one Cargo workspace of thirteen crates feeding three Python
wheels** (`repo:rust-packages/Cargo.toml`, `repo:packages/`; see [[crate-map]]),
plus the `lat-db` / `lat` binaries, a wasm validator, and a napi addon —
all shipped from a single tree. The dictionary is single-sourced at
`repo:rust-packages/laterite-ags4-reference/data/ags_dictionary.json` and everything else
generates from it.

## Options considered
1. **Split docling-style** — per-component repos (e.g. `laterite-ags4-types`,
   `laterite-ags4-core`, `laterite-validator`, `laterite-ags5`), each released
   independently.
2. **One repo, one package** — collapse everything into a single wheel.
3. **One repo, many published artifacts** *(current)* — a single workspace that
   publishes the `laterite` + `laterite-ags5` wheels and the `lat-db` /
   `lat` binaries, with heavy capability gated behind the `[ags5]` extra.
4. **Split a single outlier later, only if it earns it** — keep the monorepo,
   carve out one component to its own repo *only* when a concrete trigger fires
   (see Revisit triggers).

## Decision
**Option 3 now; Option 4 as the named future escape hatch.** The project stays a
monorepo. A repo split is explicitly *not* pursued, and is revisited only
against the triggers below — never as a speculative "cleaner structure" move.

## Why
1. **Single source of truth is load-bearing here.**
   `repo:rust-packages/laterite-ags4-reference/data/ags_dictionary.json` drives the Rust
   `#[pyclass]` codegen (`build.rs`), the `.pyi` stubs (`tools/generate_pyi.py`),
   the DuckDB DDL, and the XML codec — with `repo:packages/laterite/tests/test_pyi_stubs_match_generator.py`
   gating drift. Splitting repos breaks the one-file-edit-then-rebuild-everything
   property: a one-line dictionary change would become a cross-repo publish dance.
2. **Cross-language changes stay atomic.** A [[pyo3-boundary]] change touches the
   Rust crate and the Python package in *one* commit / PR / CI run that tests
   both together; `bump-my-version` stamps ~8 sites to a single workspace version.
   Polyrepo forces land-Rust → publish → chase-the-Python-side, with a version
   matrix to keep aligned.
3. **The project already deliberately consolidated.** The F2c arc (May 2026)
   *deleted* `ags5-models` / `ags5-db` / `ags5-ags4`, folding them into Rust + <!-- historical -->
   `laterite` (`repo:ags-wiki/log.md`, F2c entries). Splitting now reverses
   recent, intentional work.
4. **The dependency-weight win is already had without repos.**
   `pip install laterite` is the lean AGS4 base; `pip install "laterite[ags5]"`
   pulls the ~50 MB DuckDB-bundled companion wheel. Docling's main motive for its
   split — don't make every user carry heavy ML deps — is solved here by an
   *extra*, not a repo boundary.
5. **Monorepo ≠ mono-package.** The repo already ships multiple independent
   artifacts from one tree (the two wheels, two binaries, wasm, napi) — the same
   model used by polars / ruff / pydantic-core. Docling's polyrepo is the less
   common pattern and trades coordination cost for independence this project
   does not currently need.

## Consequences
Commits the toolkit to: a single Cargo workspace + uv workspace; lockstep
versioning across Rust and Python; capability/weight separation via wheel extras
([[crate-map]] "wheels split by weight, not repo"). Rules out: per-component
repos, cross-repo version matrices, and N parallel release pipelines. The
public/private split is handled at *file* level by `.github/workflows/public-tree-gate.yml`,
not by a repo boundary — which keeps that concern orthogonal to this decision.

> [!todo] **Revisit triggers — when a split (Option 4) earns itself**
> Carve a single component out to its own repo only if one of these becomes
> concretely true (not before):
> 1. A component gains **genuine external consumers on a faster cadence** than
>    the rest — the realistic candidates are [[dec-laterite-ags4-types-leaf|laterite-ags4-types]]
>    or the wasm validator becoming a standalone library others depend on directly.
> 2. A **separate maintainer** should own one component without commit rights to
>    the whole tree.
> 3. An **open-source subset** must ship while the rest stays private *and* the
>    file-level `public-tree-gate` proves insufficient.
>
> **First fire (2026-06-17): the `laterite_ags4` DuckDB extension** — triggers #1
> (distinct audience + distribution channel) and #3 (community-extensions builds
> from a public *extension-repo* shape with a root `Cargo.toml` and a stable ref,
> which the force-pushed whole-workspace mirror can't provide, so the file-level
> gate is insufficient *for that consumer*). Resolved by the Option-4 carve-out: a
> **dedicated public repo** that submodules the mirror for its lib deps. It began
> distribution-only (glue developed in-monorepo, the repo a generated artifact), but
> the hand-sync **drifted**, so on **Path B (2026-06-20)** the dedicated repo became
> the **canonical source** for the extension glue and the in-workspace copy was
> retired — a full *component* carve-out, still narrow (only the thin glue leaves;
> the library crates + dictionary single-source stay). It confirms the rule:
> exceptions are permitted when a trigger genuinely fires. See
> [[dec-duckdb-extension]] → Distribution.

## Related
[[crate-map]] · [[dec-rust-drives-python|Rust ships, Python is a library]] ·
[[pyo3-boundary]] · [[dec-python-imports-rust-library|laterite: scoped Python→Rust exception]] ·
[[dec-laterite-ags4-types-leaf|laterite-ags4-types wasm-safe leaf]] ·
AGS5 decoupled out of this tree ·
[[design/_README|AGS5 design register]]
