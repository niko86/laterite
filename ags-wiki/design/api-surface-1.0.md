---
type: decision
title: "laterite — the modern Python API surface (DuckDB-engine redesign, 0.3)"
status: accepted
tags: [design, decision, api, duckdb, polars]
decided: 2026-06-12
supersedes: []
from_gap: []
related: [dec-rust-drives-python, dec-laterite-ags4-types-leaf, crate-map, pyo3-boundary, abi3-perf, tech-stack-wasm, ags4-output, reliquary, dec-duckdb-extension, dec-rust-api-crates-io]
sources:
  - "https://duckdb.org/docs/stable/internals/storage"
  - "https://duckdb.org/docs/stable/sql/statements/attach"
  - "https://docs.pola.rs/user-guide/lazy/using/"
---

# laterite — the modern Python API surface (DuckDB-engine redesign, 0.3)

> **BUILT 2026-06-12** on branch `feat/0.3-duckdb-engine` (commit `e686da0` + polish).
> This page is the as-built reference; the full design dialogue (the narwhals →
> DuckDB-backed-scan → everything-through-DuckDB → drop-narwhals arc) is in the session
> plan + `log.md`, and the prior 1.0 design review is in this file's git history.

> **UPDATED 2026-06-19** — cross-surface vocabulary alignment (branch
> `feat/api-vocab-python`). Load verb is **`read`** (`source` later re-added as a
> fluent-chain alias — see the 2026-06-21 note below);
> the file terminal is **`.save()`** (was `.write()`); in-memory AGS4 is the memoised
> **`.text`/`.bytes`** pair (was `to_ags4_text()`); data→AGS4 is **`build_ags4()` →
> `BuildResult`** (was `emit_ags4`/`EmitResult`). The module-level `write()` is dropped.
> Plus the PR-B fluent chain (`.query()`/`AgsQuery`, `.validate()→self`, `.pipe()`).
> node / wasm / duckdb mirror this vocab where it makes sense — retired names tracked
> in [[reliquary]].

> **UPDATED 2026-06-21** — toward a **0.5.0** release. (a) **`source`** is back as a
> fluent-chain alias of `read` (same callable). (b) **`read(..., xn="numeric")`** — an
> opt-in Float64 read view of `XN` columns (qualifiers → null; write-back byte-faithful);
> a fuller bidirectional XN treatment is future work ([[#Still open / future]]). (c) The
> dictionary is now the multi-edition **union** (`ags_dictionary.json`, 4.0.3 → 4.2). (d)
> The experimental AGS5 (`.ags5db`/`.agsx`) surface is **decoupled** from the shipped
> package — the `[ags5]` extra is gone. (e) The release now
> publishes a **source distribution** alongside the wheels.

> **UPDATED 2026-07-20** — `compat.AGS4_to_dataframe`'s common path moved off
> per-cell primitives onto a Rust-built all-`Utf8` Arrow table (same "no
> per-cell boxing" trick as the typed read path, python-ags4's raw-string
> frame *shape*): **~2× faster than python-ags4** (was ~2–7× slower). pyarrow
> is now a *provable* optional accelerator rather than an untested claim — a
> new `string_dtype` knob (`object` default / `string` opt-in, needs pyarrow)
> mirrors `backend`, and two isolated-install CI smokes cover both the
> pyarrow-free default and the pyarrow-present accelerator. See Dependencies
> below and [[pyo3-boundary]].

## The as-built 0.3 surface

A parsed AGS4 file is a handle over a Python-owned in-memory **DuckDB** engine; each group
is a born-typed DuckDB table, loaded lazily on first touch (CTAS from the Rust-built Arrow).
One door per task:

| call | returns | notes |
|---|---|---|
| `read(path, backend="polars"\|"pandas")` | `Ags4File` | the door; engine is lazy |
| `ags["LOCA"]` / `ags.table("LOCA")` | eager **polars** (default) / **pandas** frame | born-typed; pyarrow-free |
| `ags.sql("SELECT … WHERE …")` | a **DuckDB relation** | cross-group joins + filter pushdown; finish with `.df()` / `pl.from_arrow(rel)` or chain SQL |
| `ags.at("LOCA", ids)` / `ags.query("SELECT …")` | an `AgsQuery` (lazy) | fan-out: `sub[code]` / `.frames()` / `.groups`; single-result: `.filter(sql)` / `.select()` → `.frame()` / `.to_polars()` / `.to_pandas()` / `.relation()` |
| `ags.connection` | the raw `duckdb` connection | every engine feature (parquet, Arrow via `.arrow()`, …) |
| `ags.register(name, frame)` | — | join your own frames in `sql()` |
| `ags.headings/units/types/line_numbers(code)` | metadata | no engine spin-up |
| `ags.text` / `ags.bytes` / `ags.save(path)` | byte-faithful AGS4 (text / UTF-8 bytes / file) | memoised; re-emitted from the retained Rust parse |
| `build_ags4(groups)` → `BuildResult` | valid AGS4 from frames (`{code: df}` / `(code, df)` list) **or a typed PROJ graph** | the data→AGS4 door (construct + autofix + validate); `.text`/`.bytes`/`.save(path)`. The typed-graph form is walked depth-first like Node's `buildAgs4` (#214) — emits the PROJ-rooted subtree, **only the headings you set** (unset ones pruned, except KEY — matches the frames door); under autofix the missing UNIT/TYPE/TRAN metadata groups (and ABBR when PA codes are used) are synthesized so a sparse graph builds valid at the default edition (2026-06-25) |
| `validate(...)` → `Report.findings` | polars frame | text-level; `ags.validate()` chains + caches on `.report` |
| `compat.*` | python-ags4 drop-in | same engine + backend switch, pandas default; `AGS4_to_dataframe`'s common path is Arrow-native since 2026-07-20 — see Dependencies below |
| `transport.{pack,unpack,lock,unlock}` | — | content-agnostic (any file) |

Read handles are context managers (`with read(p) as ags:` + `close()`); there is **no
`__del__` close**, so a relation from `sql()` survives a one-liner like
`read(p).sql(q).df()`.

## The engine + the boundary

Rust parses → typed Arrow per group (`laterite-ags4-types::arrow_cols`, the *same* emitter the wasm
explorer frames as IPC) → handed to Python zero-copy as an **Arrow PyCapsule** (`pyo3-arrow`,
abi3-safe) → loaded into DuckDB as a **native CTAS table** (NOT a view over external Arrow —
that tripped pyarrow's `string_view` `is_in` kernel on joins). Group frames come back out
**pyarrow-free**: polars via `pl.from_arrow(rel)` (the capsule), pandas via DuckDB's NumPy
`rel.df()`. Born-typed dtypes survive the round-trip identically (String / Float64 / Int64,
dirty cell → null). Writes are byte-faithful from the retained parse, independent of which
groups were read. `laterite-ags4-wasm` is unaffected — it links no DuckDB; the same Arrow emitter
feeds it as IPC.

## Dependencies (verified pyarrow-blocked)

- **base = `polars + duckdb`.** No narwhals, no pyarrow.
- **`[compat]` = `pandas<3`** — pyarrow-free by default. `AGS4_to_dataframe`'s common
  path (2026-07-20 perf pass) reads a **Rust-built all-`Utf8` Arrow table**
  (`build_record_batch_compat` in `laterite-ags4-types::arrow_cols` → `parse_compat_arrow`,
  no per-cell `PyObject` boxing —
  `repo:rust-packages/laterite-ags4-types/src/arrow_cols.rs`,
  `repo:rust-packages/laterite-py/src/lib.rs`) and hands pandas an object-dtype frame
  via DuckDB's NumPy `rel.df()` (the same trick as the core) — already **~2× faster
  than python-ags4** on that bench's 3-group fixture (was ~2–7× slower before the
  Arrow move; reproducible via `tools/bench_compat_dataframe.py`). The **published**
  figure is **~3×** — the same call, measured across five file sizes on forge's
  123-group `wide` scaffold (`repo:packages/laterite/README.md`). Two fixtures, not
  two claims; quote the published one outside the wiki.
- **pyarrow is an OPTIONAL accelerator, never a hard dependency.** `[compat,pyarrow]`
  (or `[all]` / the bare `[pyarrow]` extra) adds it; `_frames.py::compat_materializer`
  auto-detects it at runtime and swaps the pandas hop to pyarrow's `to_pandas` (a
  touch faster) *and* unlocks the `string_dtype="string"` knob (pandas' Arrow-backed
  `str`, the pandas-3 baseline — `object` stays the default, numpy, today's
  python-ags4-compatible dtype). Absent, the DuckDB `.df()` object hop above still
  runs; the explicit `pyarrow` backend / `ags.connection` route is unaffected. CI
  proves both isolated shapes (`ci.yml` wheel-smoke: `tools/compat_smoke.py` for
  `[compat]`, `tools/compat_pyarrow_smoke.py` for `[compat,pyarrow]`).
- **No `[ags5]` extra.** The experimental `.ags5db` companion was decoupled from the
  shipped package (#177); `laterite` is AGS4-only.

## Key decisions (and why)

- **No narwhals.** DuckDB itself multiplexes backends (`.pl()` / `.df()` / `.arrow()`), so
  narwhals was redundant indirection — and the source of four live frictions: it needs
  pyarrow (its duckdb-collect calls `rel.arrow()`); a `string_view` crash on joins over
  registered Arrow views; `.collect()` defaulting to a pyarrow frame; and a DuckDB-backed
  lazyframe dying on a one-liner (GC closes the connection). Dropping it dissolved all four.
  Verified the compat migration held: python-ags4 parity unchanged at 122/131.
- **Frames go *through* the engine.** Eager `ags[code]` materialises via DuckDB (not direct
  Arrow → polars) because that is the only way to give **pandas pyarrow-free** (`rel.df()`);
  `polars.to_pandas()` would pull pyarrow. The ~1.7× round-trip on a whole-group read is the
  price of one uniform, pyarrow-free path.
- **`sql()` is the lazy/pushdown path**, not a `scan()` / lazy-frame layer — the relation's
  `WHERE` runs in the engine, materialising only matching rows. `at()` is the ergonomic
  sugar over that for the location-subset case (filter by `{group}_ID`, chainable,
  `frames()` pulls all related groups at once).
- **0.3, not 1.0.** The surface pivoted repeatedly during design; 0.x keeps it free to
  evolve until the model proves out in real use.

## Still open / future

- **Comprehensive XN.** 0.5.0 ships only a read-side `read(xn="numeric")` view. The fuller
  treatment — Rust-level born-typed XN on the Arrow boundary preserving the original
  qualifier token for dual (numeric + faithful) access, bidirectional so a numeric edit
  re-emits valid AGS4, opt-in across read/typed-graph/node/wasm — is future work (#178).
- **Experimental AGS5 is decoupled, not converging here.** The `.ags5db`/`.agsx` surface
  moved to a dormant holding folder outside this tree (#177) — it no longer
  ships with `laterite`. A future AGS5 strand re-links it against the shared libs and
  would publish it separately; the planner-split convergence idea lives with that strand.
- **The DuckDB community extension shipped.** `laterite_ags4` (`read_ags()` across CLI /
  Python / wasm) is no longer parked — it's released from its own repo
  `niko86/laterite-duckdb` via DuckDB Community Extensions ([[dec-duckdb-extension]]).

## The invariant

> Every group of AGS data a user touches arrives as a born-typed frame in the backend they
> chose, from one obvious door; the cost was paid once, in Rust at parse time and once more
> as a native DuckDB table; writing it back unchanged is byte-identical — proven by a
> property test, not a promise.

## Related

The **Rust** counterpart to this page is [[dec-rust-api-crates-io]] — the public Rust
API surface and the crates.io publishing decision. It borrows this page's verb and
option vocabulary deliberately (`read`, `validate`, `build_ags4`, `warnings`, `fyi`,
`risky`, `on_type_clash`), so the two surfaces read alike where the languages allow.
