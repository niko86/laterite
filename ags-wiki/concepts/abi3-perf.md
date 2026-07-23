---
type: concept
title: "abi3 performance: what the limited API costs laterite"
status: drafted
tags: [concept, architecture, pyo3, performance, benchmark]
volatile: [timings]
volatile_asof: 2026-06-08
ags_editions: []
repo_refs:
  latpy: "repo:rust-packages/laterite-py/Cargo.toml"
related: [pyo3-boundary, crate-map, laterite-py]
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

Four workloads, spanning pure-boundary (where abi3 is maximal) to real Rust-heavy:

- `micro_construct` — `PROJ()` in a tight loop (bare `#[pyclass]` alloc/dealloc).
- `micro_attr` — set+get on the per-instance `__dict__` (the `dict` member).
- `typed_construct` — `LOCA(loca_id=…, loca_gl=…, …)` (kwargs `__new__` + typed
  field descriptors) — realistic object building.
- `validate` — `laterite.validate(fixture)` (Rust validation + marshalling 51
  findings across the boundary) — a real operation.

## Results

> [!stale-risk] timings · as-of 2026-06-08 · macOS arm64, CPython 3.14.3, release builds
> Absolute ns drift with machine/Python; the **ratios** are the durable finding.

| workload (ns/iter) | non-abi3 | abi3-py312 (shipped) | abi3-py314 |
|---|---:|---:|---:|
| `micro_construct` (bare alloc) | **44** | 49 | 51 |
| `micro_attr` (dict set/get) | 19 | 19 | 19 |
| `typed_construct` (kwargs + fields) | 553 | 540 | 552 |
| `validate` (per call) | 322 000 | 322 000 | 325 000 |

## What it means

1. **abi3's only measurable cost is bare object construction** — ~44 → ~49 ns,
   about **+12%** (~5 ns/object). That's the `tp_alloc`/`tp_new` path, which the
   limited API can't inline as tightly.
2. **Everywhere else abi3 is free.** Dict attribute access is identical (the
   `PyDict` C-API is the same under abi3); `typed_construct` and the real
   `validate` op show no difference beyond run-to-run noise — the ~5 ns
   construction delta is swamped by kwargs parsing / Rust work.
3. **abi3-py312 ≈ abi3-py314.** No meaningful difference on any workload (py314
   is if anything a hair *slower* on construct, within noise). **Raising the abi3
   floor buys no performance** — it only costs 3.12/3.13 reach.

So the boundary is never the bottleneck: on `validate` — the actual workload —
all three builds are within **<1%**, because the Rust validation dominates and
PyO3 just marshals the result once (consistent with the README benchmark's
"~0% extra over the bare validation pass").

## Decision

Ship **abi3-py312**: one wheel for 3.12+, at a cost (≤5 ns/object) invisible in
real use. Non-abi3 would reclaim that micro-cost but reimpose a per-version wheel
matrix (3 platforms × N Pythons instead of 3 total); abi3-py314 would drop
3.12/3.13 users for zero gain. See [[pyo3-boundary]] for the floor rationale (the
174 `#[pyclass(dict)]` classes pin the abi3 minimum at 3.12).

## Toolchain caveats (found while measuring)

- **maturin 1.13.3 mis-tags `abi3-py314`.** An `abi3-py314` build produces a
  *distinct* binary (it genuinely uses the 3.14 limited API — the `.so` sizes
  differ) but maturin still names the wheel `cp312-abi3`. So `abi3-py314` isn't
  safely shippable with this toolchain regardless of the (non-existent) perf win.
- **Workspace feature unification (resolver v2).** `laterite-py` and
  `laterite-py-ags5` share the `pyo3` dependency; building either resolves the
  whole workspace and *unifies* the abi3 feature. You cannot ship one wheel abi3
  and the other not — the abi3 setting must match across both glue crates.

## Related

[[pyo3-boundary]] · [[crate-map]] · [[laterite-py]]
