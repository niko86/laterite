---
type: concept
title: "abi3 performance: what the limited API costs laterite"
status: drafted
tags: [concept, architecture, pyo3, performance, benchmark]
volatile: [timings]
volatile_asof: 2026-08-03
ags_editions: []
repo_refs:
  latpy: "repo:rust-packages/laterite-py/Cargo.toml"
related: [pyo3-boundary, crate-map, laterite-py, core-perf-baseline]
sources: []
---
# abi3 performance: what the limited API costs laterite

## Definition

The shipped wheels build **abi3-py312** (one `cp312-abi3` wheel per platform,
runs on 3.12+ — see [[pyo3-boundary]]). abi3 (the CPython *limited* / stable
ABI) restricts which C-API entry points the extension may use, which *can* cost
runtime vs a full-API, version-specific build. This page answers two questions
empirically: **(1)** how much does abi3 cost laterite, and **(2)** does a higher
abi3 floor (`abi3-py313`/`abi3-py314`) buy any speed?

## Method

Three builds of the **base wheel** (`laterite-py`), benchmarked on the **same**
CPython 3.14 (so only the ABI differs), best-of-9 min timing:

- **non-abi3** — full API, version-specific (`cp314-cp314`).
- **abi3-py312** — what we ship.
- **abi3-py314** — the higher floor.

Reproduce with **`repo:tools/bench-abi3.py`** — it makes all three builds, proves
they are genuinely three different binaries (see the caveat below on why that
needs proving), and prints the table and the ratios. The measurement was hand-run
once and written down; the script is what lets it be re-run rather than aged.

Four workloads, spanning pure-boundary (where abi3 is maximal) to real Rust-heavy:

- `micro_construct` — `PROJ()` in a tight loop (bare `#[pyclass]` alloc/dealloc).
- `micro_attr` — set+get on the per-instance `__dict__` (the `dict` member).
- `typed_construct` — `LOCA(loca_id=…, loca_gl=…, …)` (kwargs `__new__` + typed
  field descriptors) — realistic object building.
- `validate` — `laterite.validate(fixture)` (Rust validation + marshalling 51
  findings across the boundary) — a real operation.

## Results

> [!stale-risk] timings · as-of 2026-08-03 · Apple M5, macOS 26.4.1, CPython 3.14.3, maturin 1.14.1, release builds
> Absolute ns drift with machine/Python; the **ratios** are the durable finding.
> Re-measured with `repo:tools/bench-abi3.py` (`--reps 9`); the `validate` figures
> are not comparable to the 2026-06-08 run, which used a different (larger) input.

| workload (ns/iter) | non-abi3 | abi3-py312 (shipped) | abi3-py314 |
|---|---:|---:|---:|
| `micro_construct` (bare alloc) | **43.9** | 48.5 | 47.2 |
| `micro_attr` (dict set/get) | 20.1 | 20.2 | 20.0 |
| `typed_construct` (kwargs + fields) | 692 | 683 | 678 |
| `validate` (per call) | 94 958 | 95 213 | 95 498 |

Ratios vs non-abi3 — the part that survives a change of machine:

| workload | abi3-py312 | abi3-py314 |
|---|---:|---:|
| `micro_construct` | 1.10× | 1.08× |
| `micro_attr` | 1.00× | 0.99× |
| `typed_construct` | 0.99× | 0.98× |
| `validate` | 1.00× | 1.01× |

## What it means

1. **abi3's only measurable cost is bare object construction** — ~44 → ~49 ns,
   about **+10%** (~5 ns/object). That's the `tp_alloc`/`tp_new` path, which the
   limited API can't inline as tightly. The symbol evidence lines up: the abi3
   builds give up `_Py_Dealloc`, `_PyType_Freeze` and the `_PyUnicodeWriter_*`
   family and reach for `PyType_GetSlot` / `PyObject_Vectorcall` instead.
2. **Everywhere else abi3 is free.** Dict attribute access is identical (the
   `PyDict` C-API is the same under abi3); `typed_construct` and the real
   `validate` op show no difference beyond run-to-run noise — the ~5 ns
   construction delta is swamped by kwargs parsing / Rust work. Both are *under*
   1.00× here, which is noise, not a win.
3. **abi3-py312 ≈ abi3-py314.** No meaningful difference on any workload.
   **Raising the abi3 floor buys no performance** — it only costs 3.12/3.13 reach.

So the boundary is never the bottleneck: on `validate` — the actual workload —
all three builds are within **<1%**, because the Rust validation dominates and
PyO3 just marshals the result once (consistent with the README benchmark's
"~0% extra over the bare validation pass").

Two re-measurements apart, on different hardware and after two months of engine
work, every conclusion held and only the absolute numbers moved. That is the
evidence for treating the ratios — not the nanoseconds — as the finding.

## Decision

Ship **abi3-py312**: one wheel for 3.12+, at a cost (≤5 ns/object) invisible in
real use. Non-abi3 would reclaim that micro-cost but reimpose a per-version wheel
matrix (3 platforms × N Pythons instead of 3 total); abi3-py314 would drop
3.12/3.13 users for zero gain. See [[pyo3-boundary]] for the floor rationale (the
174 `#[pyclass(dict)]` classes pin the abi3 minimum at 3.12).

## Toolchain caveats (found while measuring)

- **maturin mis-tagged `abi3-py314` — fixed by 1.14.1.** On 1.13.3 an
  `abi3-py314` build produced a *distinct* binary (it genuinely used the 3.14
  limited API — the `.so` sizes differed) while maturin still named the wheel
  `cp312-abi3`, so `abi3-py314` was not safely shippable regardless of the
  (non-existent) perf win. On 1.14.1 the tag is `cp314-abi3` and the three builds
  are correctly distinguished. `repo:tools/bench-abi3.py` re-checks this every run
  rather than trusting this note: it identifies each build by the extension's
  **sha256**, not by its wheel filename, aborts if any two hash the same, and says
  so if two builds ever share a tag again.
- **abi3 is not a command-line switch.** It is baked into `laterite-py`'s `pyo3`
  dependency line, so measuring it means rewriting that manifest three times.
  Anything automating this has to restore the original bytes unconditionally —
  including on a failed build — or an ABI change lands in the tree unnoticed.
- **Workspace feature unification (resolver v2)** would bite if a *second*
  pyo3-linking crate ever joined the workspace: both would share the one `pyo3`
  dependency, building either resolves the whole workspace, and the abi3 feature
  unifies across them — you could not ship one wheel abi3 and the other not.
  `laterite-py` is currently the only crate here that links pyo3, so this is
  latent, not live.

## Related

[[core-perf-baseline]] — the other half of the picture: this page prices the
*binding*, that one prices the *engine* (parse / rules / type / emit).

[[pyo3-boundary]] · [[crate-map]] · [[laterite-py]] · [[core-perf-baseline]]
