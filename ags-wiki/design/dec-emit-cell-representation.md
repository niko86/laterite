---
type: decision
title: "emit's cell is its own enum, and Arrow streams past it (the RowData split, ten weeks late)"
status: accepted
tags: [design, decision]
decided: 2026-08-29
supersedes: []
from_gap: []
related: [ags4-output, laterite-ags4-emit, laterite-ags4-types, dec-laterite-ags4-types-leaf, dec-rust-api-crates-io, dec-facade-parity, crate-map, reliquary]
sources: []
---

# emit's cell is its own enum, and Arrow streams past it

Issue: #790. The measurements that price this decision — the heap composition
table, the per-cell byte ladder, the A/B/A — live on #788, #789 and #790, by
design: the instrument is `repo:rust-packages/laterite-ags4-emit/examples/heap_profile.rs`,
and no tracked page carries a figure nothing recomputes.

## Context

`laterite-ags4-emit` carried every input cell as a `serde_json::Value`
(`GroupInput.rows: Vec<Vec<Value>>`). Ten workspace crates enable serde_json's
`preserve_order` feature, Cargo unifies features across a build graph, and the
swollen `Map` variant that feature buys inflates **every** `Value` in any binary
linking those crates — including millions of cells that are never maps. On the
workload profiled in #790 the input transpose was the single largest slice of
`build_ags4`'s peak, and most of each cell was container, not content: the
enums outweighed the string bytes they pointed at by well over an order of
magnitude (the exact composition is on the issue).

`preserve_order` itself cannot move: it is load-bearing for the validator's
NDJSON key order (`repo:rust-packages/laterite-ags4-validator/Cargo.toml` says
so in as many words), and unification cannot be broken while the crates link
into one binary. So the fix has to be a cell representation that is not a JSON
value.

There is history here. The original emit design
([[ags4-output]], Phase 1) specified a `GroupInput` / `RowData`
(typed-columns | string-cells) model. `RowData` was never built: the page
judged "performance is a non-issue at site scale", and the *As built* record
replaced the split with one universal cell type plus one shared
Arrow→`Value` transpose (`repo:rust-packages/laterite-ags4-emit/src/arrow_in.rs`),
bought deliberately for drift-prevention — both hosts feed one type, so they
provably cannot diverge after the input. A downstream consumer then built
multi-million-cell files, and the recorded scale assumption stopped holding.
The irony is that the page also said the boundary cost was "minimised by Arrow
for columnar input": the Arrow door became the expensive one, because it is
the only door that manufactures a representation the caller did not already
have — the JSON door gets its `Value`s from serde because JSON genuinely *is*
values.

## Options considered

1. **Drop or gate `preserve_order`.** Ruled out in #790 itself: the feature is
   load-bearing elsewhere and unified in regardless.
2. **A slim cell enum only** (the issue's shape 2). Recovers the unification
   tax — the enum lands at the size a de-unified `Value` would — and leaves
   the transpose itself standing.
3. **`RowData` as an enum inside `GroupInput`**, Arrow variant behind the
   `arrow` feature. Recovers everything, but a `#[cfg]`-gated variant makes a
   public enum's shape feature-dependent.
4. **`RowData`'s split as two entry points** — the JSON/document door keeps
   materialised cells, a new Arrow door streams batches straight into
   formatting — both converging on the internal formatted representation
   (`OwnedGroup`).

## Decision

**Options 2 and 4 together, staged as two PRs.**

- **PR1 — the cell swap.** A `Cell` enum
  (`Null | Text(String) | Int(i64) | Float(f64) | Bool(bool)`) lives in
  `laterite-ags4-types`, beside `ags4_str`, its consumer — `ags4_str` now
  takes it, keeping exactly one authority for AGS4 numeric spelling.
  `GroupInput.rows` becomes `Vec<Vec<Cell>>`. `Cell` implements `Deserialize`,
  so the browser's JSON door deserialises straight into it and never builds a
  `Value` at all. The facade's document door builds `Cell::Text` instead of
  wrapping its own strings in `Value::String` for a `format_cell` that cloned
  them straight back out.
- **PR2 — the Arrow door.** `emit_ags4_from_arrow` streams record batches
  into `OwnedGroup` with no row-major intermediate; the #695 DT-precision
  rendering moves into it unchanged. The transpose
  exports (`cell_value`, `group_from_arrow`, `group_from_arrow_with_meta`,
  `group_from_arrow_with_meta_at_edition`) retire, inventoried through
  [[reliquary]] first. `serde_json` leaves emit's `[dependencies]` — a
  compile-time ratchet: the crate becomes structurally unable to grow a
  `Value` cell again.

### Why two `Cell` types

The facade already has `laterite::ags4::Cell`, built precisely so that
`serde_json`'s major version cannot dictate the facade's
(`repo:rust-packages/laterite/src/ags4/build.rs`). It cannot re-export the
engine's enum: `tools/check_public_api.py` (`check_no_third_party`) permits
only `laterite`/`core`/`alloc`/`std` path roots in the facade's rendered
surface, so an engine type in a facade signature fails the build. The two
types are the same concept — one cell's value, before AGS4 formatting — held
apart by a gate, converted by a consuming `into_engine(self)` so the
conversion is a move, not a clone-per-cell (the same lesson #788/#789 applied
at the emit door, applied at the builder door).

### Why the doors still cannot drift

The DRY property that displaced `RowData` survives; the **join point moves
later**. Today the doors join at `GroupInput` (the input cells); after PR2
they join at `OwnedGroup` (the formatted strings), and everything after that
point is one code path, exactly as before. What can drift is confined to two
short input functions, and a differential test sits on the join: the same
logical data through both doors must produce equal `OwnedGroup`s.

That test has a known trap: the doors are deliberately *not* identical for
temporal columns. The Arrow door renders a typed instant at its heading's
declared UNIT precision (#695); the JSON door emits a caller's string
verbatim. The differential test therefore asserts equality on data where the
doors *should* agree, and pins the temporal divergence separately as
intended behaviour — weakening the equality assertion to make red go away is
the failure mode.

## Why

The cost was the container, not the text: the profiled workload's cells were
overwhelmingly numeric, and a numeric cell in a slim enum allocates nothing.
Option 2 alone only recovers the unification tax; the transpose it leaves
standing is pure overhead for Arrow hosts, whose batches are already resident
in memory. Option 4 alone leaves the JSON and document doors carrying the
swollen `Value` when a slim enum holds the same scalars. Together they take
both, and the staging
puts the mechanical, everywhere-at-once type swap (PR1) in a separate,
individually shippable PR from the design-bearing streaming door (PR2).

## Consequences

- `laterite-ags4-emit` and `laterite-ags4-types` take breaking API changes on
  crates.io (`GroupInput.rows`, `ags4_str`'s signature). `check_semver`
  baselines against crates.io and renders the verdict; the retiring transpose
  exports had no in-repo caller outside the profiling instrument, but they
  are published, so "no in-repo caller" was checked and is not claimed as
  "no caller".
- `preserve_order` does **not** move, and nothing is de-unified: binaries
  linking the ten enabling crates still get the swollen `Value`. Emit simply
  stops holding one per cell.
- The facade's public API is unchanged — `GroupInput` was never in it, which
  is `laterite::ags4::Cell` doing its job.
- The scale assumption in [[ags4-output]] ("performance is a non-issue at
  site scale") is superseded by this page; an update section there points
  here so nobody reasons from it again.
- Future emit-side representations answer to the ratchet: no `serde_json` in
  emit's dependency list to reach for.

## Related

[[ags4-output]] · [[laterite-ags4-emit]] · [[laterite-ags4-types]] ·
[[dec-laterite-ags4-types-leaf]] · [[dec-rust-api-crates-io]] ·
[[dec-facade-parity]] · [[reliquary]] ·
repo: rust-packages/laterite-ags4-emit/src/emit.rs ·
repo: rust-packages/laterite-ags4-emit/src/arrow_in.rs ·
repo: rust-packages/laterite-ags4-types/src/lib.rs ·
repo: rust-packages/laterite/src/ags4/build.rs
