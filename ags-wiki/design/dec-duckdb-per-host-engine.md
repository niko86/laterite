---
type: decision
title: "laterite query & materialise: one Rust+Arrow core, a per-host DuckDB (incl. duckdb-wasm)"
status: accepted
tags: [design, decision, architecture, duckdb, wasm]
decided: 2026-06-20
owns: [duckdb-host-engine]
supersedes: []
from_gap: []
related: [dec-duckdb-perf-architecture, dec-duckdb-extension, dec-rust-drives-python, dec-laterite-types-leaf, pyo3-boundary, crate-map, tech-stack-wasm, api-surface-1.0]
sources: []
---

# laterite query & materialise: one Rust+Arrow core, a per-host DuckDB

## Context

The same question keeps recurring: *"why isn't DuckDB one shared library across all
of laterite — why per-host?"* This page is the canonical answer so it is not
re-derived. It governs the **query** surface (`ags.sql()` / `at()` / `.filter()` /
`ags[code]`) and the **materialise** surface (write AGS4 → a queryable `.duckdb`).
It does **not** cover the `.ags5db` prototype identity (that is
dec-ags5db-submarine) nor the loadable extension (that is
[[dec-duckdb-extension]]).

## The model (one spine, read and write)

```
Rust (parse → typed Arrow [+ UUIDv8 keys])
   → Arrow capsule (Arrow C Data Interface, zero-copy)
   → ┊ language/FFI boundary ┊
   → the HOST's DuckDB does the I/O
   → host query API  /  a written .duckdb file
```

Rust supplies the data (and, for materialise, the deterministic content-addressed
keys); the **host's own DuckDB** does the I/O. Read and write are the *same* spine —
only the direction differs. The genuinely **shared** library is the pure-Rust,
**DuckDB-free** core (typing leaf [[laterite-types]] + the codec in `laterite-ags4-core`
+ the keychain in [[laterite-ags4-reference]] (re-exported at the historical
`laterite-ags4-core::keychain` path so every consumer is unchanged) + the validator)
**plus typed Apache Arrow as the one contract**
([[pyo3-boundary]], [[dec-laterite-types-leaf]]). DuckDB is a per-host detail bolted
on top.

The host DuckDB per surface — **peers, not special cases**:

| Surface | The host DuckDB (the "calculator") | Status |
|---|---|---|
| Python | pip `duckdb` package | hard base dep (`repo:packages/laterite/pyproject.toml`) |
| node | `@duckdb/node-api` | optional peer (`repo:rust-packages/laterite-node/package.json`) |
| browser | **`duckdb-wasm`** | lazy-loaded asset ([[tech-stack-wasm]]) |

> [!note] duckdb-wasm is the browser's host DuckDB — for reads **and** writes
> `duckdb-wasm` plays the exact same role pip-`duckdb`/`@duckdb/node-api` play
> natively. It ingests the Rust crate's Arrow IPC to query **and can persist**
> (OPFS-backed databases / `COPY`/`EXPORT` to a downloadable file). The browser is
> **not** a special case for materialise. What is genuinely native-only is a
> different thing — Rust-side persistence accelerators (the `.ags.idx` byte-index
> sidecar, the parse cache) need Rust filesystem access, which wasm-Rust lacks
> ([[dec-duckdb-perf-architecture]]). Those are perf helpers, not the materialise.

## Options considered

1. **One shared native Rust DuckDB engine across ALL surfaces** (Rust drives a
   bundled libduckdb everywhere). **Infeasible**, four independent ways: (a) native
   libduckdb cannot compile into a wasm module — the extension's wasm attempt proved
   only a reduced stable subset works ([[dec-duckdb-extension]]); (b) it forces a
   ~14 MB-stripped bundled engine into every base wheel + npm package; (c) an engine
   behind the FFI destroys the interop contract — callers want a live
   `duckdb.DuckDBPyRelation` / connection / `register()`, not Arrow-across-FFI;
   (d) bundled `duckdb-rs` and the loadable extension path (originally
   `quack-rs`, migrated to the official `duckdb` crate 2026-07-08) were <!-- retired: quack-rs -->
   mutually-exclusive `libduckdb-sys` configs that couldn't co-build at the
   time of this decision — whether that constraint still holds post-migration
   isn't verified here ([[dec-duckdb-extension]]).
2. **Per-host DuckDB, shared Rust+Arrow core** (chosen).
3. **Make the calculator optional everywhere** (node already does). A live
   dependency-shape choice for [[api-surface-1.0]] (#115), *orthogonal* to this
   decision — owner leans hard-everywhere (capability-first).

## Decision

**Per-host DuckDB on a shared Rust+Arrow core.** The Rust core (DuckDB-free) parses,
types, keys, validates and emits once; Arrow is the universal handoff; each surface's
own DuckDB does the query/write I/O. **Materialise to `.duckdb`** is the same spine:
Rust supplies typed Arrow + the deterministic **UUIDv8 keychain** (lifted into a
shared leaf crate so the libraries and the extension single-source it —
`repo:rust-packages/laterite-ags4-reference/src/keychain.rs` (the row-identity
consolidation that also gave `laterite-ags4-merge` its shared KEY definition;
`laterite-ags4-core::keychain` re-exports it unchanged), exp-uuid7-surrogate-keys),
and the host DuckDB writes the file (a plain queryable `.duckdb`, *not* `.ags5db`).

## Why

- **Interop is the point.** Per-host DuckDB lets callers get back native, composable
  objects (`ags.sql()` → a real relation; `ags.connection`; `ags.register()` to join
  their own frames). An in-Rust engine would hand back Arrow and forfeit that.
- **Read and write share the spine, so the materialise needs no second engine and no
  per-surface re-implementation** — the heavy logic (Arrow + keychain) is shared
  Rust; the host DuckDB I/O is thin, already-present glue, reused both directions.
  In Python, `ags[code]` *already* funnels through the host engine
  (`repo:packages/laterite/python/laterite/__init__.py:357-387` — `con.register` the
  Rust Arrow → CTAS → `.pl()`/`.df()`); write is the same call, opposite direction.
- **DuckDB multiplexes the output backend** (`.pl()`/`.df()`/`.arrow()`), so polars is
  a convenient *materialisation of DuckDB's result*, not the engine — and pandas
  comes out pyarrow-free via DuckDB's NumPy `.df()` ([[pyo3-boundary]],
  [[api-surface-1.0]]). DuckDB is **load-bearing in the base**, not a swappable
  convenience.

## Consequences

- The "per-host calculator isn't shared" worry is resolved: the *Rust* part is shared
  once; only thin host-DuckDB glue is per-surface. This is **not** a reason to split
  the monorepo ([[dec-monorepo-structure]] — the extension is the sole justified split).
- **wasm is uniform**: it reads and materialises via `duckdb-wasm` (Arrow IPC in,
  OPFS/export out). Only the Rust-side accelerators (sidecar/cache) stay native-only.
- The shared **UUIDv8 keychain** crate is the one piece the extension flows back into
  the libraries — **done (#303)**: every read surface (the Python wheel via
  `Reading::table_for`, the Node addon via `table_ipc`, and the browser via wasm
  `arrow_ipc(code, keys=true)`) now prepends `_id`/`_parent_id` through the SAME
  `keychain::group_row_ids`, so the ids are byte-identical to the extension's. The
  relational layer is always-keyed (joins work); frame accessors strip by default
  (`keys=True`/`{keys:true}` to keep); emit strips. A golden-UUID test in
  `keychain.rs` (`content_id_pins_the_cross_surface_golden`) is the single source
  the Python/Node/wasm surface tests all match. To stay wasm-safe, core's
  `transport` module is behind a default-on feature (its `age`→getrandom won't
  build on wasm32); `laterite-ags4-wasm` takes `default-features = false`.
- The hard-vs-optional base-`duckdb` dep is a separate [[api-surface-1.0]] (#115)
  decision, not reopened here.

## Related

[[dec-duckdb-perf-architecture]] · [[dec-duckdb-extension]] · [[dec-rust-drives-python]] · [[dec-laterite-types-leaf]] · dec-ags5db-submarine · [[pyo3-boundary]] · [[crate-map]] · [[tech-stack-wasm]] · exp-uuid7-surrogate-keys · [[api-surface-1.0]]
