---
type: tool
title: laterite-ags4-compliance
status: drafted
tags: [tool]
tool_kind: crate
language: rust
artifact: laterite-ags4-compliance
ags_editions: []
repo_refs:
  root: "repo:rust-packages/laterite-ags4-compliance"
related: [laterite-ags4-parity, laterite-ags4-xcheck, parity-model, crate-map]
sources: []
---
# laterite-ags4-compliance

<!-- BEGIN GENERATED: crate-card — DO NOT EDIT BY HAND. Regenerate: uv run --no-project python tools/gen_crate_graph.py -->
> [!note] **Not published** — `laterite-ags4-compliance` is a workspace crate, internal to this repo, at v0.9.0 (inherited from the workspace).
> **Used by** — nothing else in this workspace.
<!-- END GENERATED: crate-card -->

## What it is
> [!quote] **Implemented** (`repo:rust-packages/laterite-ags4-compliance`, #169).
> The cross-surface **findings-agreement** harness: it reads one finding-set
> JSON per validation surface (emitted by the per-surface runners) and checks
> the findings-emitting AGS4 surfaces agree over python-ags4's fixtures.

Two invariants:
1. the four laterite surfaces (rust / python-laterite / node / wasm) wrap
   **one** engine → they must be byte-identical on the numbered-rule floor;
2. python-ags4 is the O-N-aware reference, compared via [[laterite-ags4-parity]]'s
   `classify` **verbatim** — no parity logic re-derived, so no clean-room drift.

## Inputs / outputs
> [!quote] In: one findings JSON per surface. Out: a pass/fail agreement
> report; non-zero exit on any unexplained divergence.

Three bins:
- `laterite-ags4-compliance` — the comparator (the primary bin).
- `emit-rust` — the rust-surface runner; drives the validator engine directly
  and emits `rust.json` in the schema the comparator reads.
- `duckdb-parse-check` (laterite-dev#458) — the duckdb *read/parse-agreement* leg: proves
  the read-only extension's `read_ags` produces the same content-addressed key
  set as the core reference (it compares a pre-computed JSON against the core
  reader — it does **not** link the duckdb crate).

## Relationship to laterite-ags4-xcheck
Distinct concern, deliberately a **separate** crate. The **output-value gate**
(`xcheck` / `emit-cases` + the case manifest) lives in the lean
[[laterite-ags4-xcheck]] crate so the gate builds on just `validator + emit +
parse`. This harness is about *findings agreement*, not output values, and its
kept bins never touch the case manifest — so the two stay decoupled and the
gate stays slim.

## Where it lives
`repo:rust-packages/laterite-ags4-compliance`. Deps only [[laterite-ags4-parity]]
+ `laterite-ags4-validator` + `laterite-ags4-core` + serde — no heavy dep, no
duckdb crate. Depends **on** parity/validator, never the reverse.

## Relationship to other components
```mermaid
flowchart LR
  val[laterite-ags4-validator lib] --> comp[laterite-ags4-compliance]
  core[laterite-ags4-core] --> comp
  parity[laterite-ags4-parity] --> comp
  comp -->|agreement report| ci[compliance CI gate]
```

See [[parity-model]] for the verdict semantics the reference comparison carries.

## Related
[[laterite-ags4-parity]] · [[laterite-ags4-xcheck]] · [[parity-model]] · [[crate-map]]
