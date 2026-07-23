---
type: decision
title: "A Python library MAY import the Rust engine (laterite) — scoped exception to Rust-drives-Python"
status: accepted
tags: [design, decision]
decided: 2026-05-18
supersedes: []
from_gap: []
related: [dec-rust-drives-python, design/_README, laterite-ags4-check, parity-model, dec-duckdb-extension]
sources: []
---

# A Python library MAY import the Rust engine (laterite) — scoped exception to Rust-drives-Python

## Context
[[dec-rust-drives-python]] settled the **shipped product**: the Rust
binary is the shipped CLI (now `lat`); the Python `laterite` package is
a library + parity oracle; Python must never invoke the shipped binary
as its execution engine; and *if* a future capability needs a
Rust↔Python boundary it must be **Rust drives Python** (embed Python
in Rust). That decision's known-gap **#4** explicitly deferred a
"PyO3 spike to validate the Rust→Python boundary … its own `/plan`".

A concrete need then arrived: downstream Python users want
`python-ags4`'s capability —
`AGS4_to_dataframe` / `dataframe_to_AGS4` / `check_file` — but on the
clean-room Rust engine, *as an importable library*, not by shelling
out to the `lat` binary. The repo already has a feature-
complete clean-room validator (`repo:rust-packages/laterite-ags4-validator`,
crate `laterite_ags4_validator`) parity-tested against `python-ags4==1.2.0`.
The open question: does exposing it to Python via PyO3 **contradict**
[[dec-rust-drives-python]] (which says "Rust drives Python, never the
reverse")?

## Options considered
1. **Refuse it** — treat any Python→Rust direction as forbidden by
   [[dec-rust-drives-python]]. Leaves Python users on `python-ags4`
   (LGPL, pandas-mandatory, slower) and blocks the codec from ever
   running on the clean-room engine.
2. **Re-implement the validator in Python** — a second
   implementation to keep in parity; defeats the single-engine,
   single-source-of-truth gain and doubles the divergence surface.
3. **Shell out to the `lat` binary from Python** — process
   per call, no dataframe API, brittle; not a library.
4. **A Python *library* (`laterite`) that imports the Rust engine via
   PyO3** — `rust-packages/laterite-py` (cdylib, depends *on*
   `laterite_ags4_validator`, never the reverse) compiled by maturin into
   `repo:packages/laterite`. Scoped, named exception recorded here.

## Decision
**Option 4.** A Python *library* MAY import the Rust engine in-process
via PyO3. The new `laterite` package
(`repo:packages/laterite/pyproject.toml`, crate
`repo:rust-packages/laterite-py/Cargo.toml`) is the **named, scoped
exception** to the "never the reverse" clause of
[[dec-rust-drives-python]]. It discharges that decision's known-gap #4
(the PyO3 boundary spike), in the **library** direction.

The exception is bounded by three invariants:

1. **Not the shipped product.** The shipped CLI is the Rust `lat`
   validator binary ([[build-rust]]). `laterite` is a library
   wheel; nothing about distribution changes.
2. **Library link, not binary drive.** Python links the Rust *library*
   in-process (`laterite._laterite_native`); it never invokes the
   shipped `lat` *binary* as its execution engine —
   exactly the thing [[dec-rust-drives-python]] ruled out.
3. **Engine is downstream-only.** `laterite-py` depends **on**
   `laterite_ags4_validator`; the validator never depends on pyo3. The lean
   `phf+thiserror+chrono` dep-graph guarantee is verified intact
   (`cargo tree -p laterite-ags4-validator` shows no pyo3/polars) — same
   wrapper pattern as `ags4-parity` / `laterite-cliutil`.

A further bounded deviation from the original design sketch: the
Rust↔Python boundary carries **plain primitives**, not `pyo3-polars`
Arrow-FFI handles — frames are assembled on the Python side
(`repo:packages/laterite/python/laterite/_frames.py`). This was the
design's documented fallback; chosen up-front to remove the
Rust/Python polars ABI version coupling and keep crate deps to
`pyo3 + serde_json`.

## Why
One engine, one source of truth. Re-implementing (opt 2) or refusing
(opt 1) either forks the parity model or strands Python users;
shelling out (opt 3) is not a library. The [[parity-model]] stays
intact precisely because `laterite` *is* the same `laterite_ags4_validator`
binary's engine: `laterite.compat.check_file` /
`laterite.validate().to_json()` are byte-identical to `[[laterite-ags4-check]]`
(verified across the validator fixture corpus), so a divergence is
still, by construction, Rust-vs-`python-ags4` — never a new
laterite-vs-Rust axis. The "never the reverse" clause exists to keep
the *shipped binary* free of a Python runtime prerequisite; an
independently-versioned library wheel that *embeds* the Rust engine
does not touch that property, so the spirit of
[[dec-rust-drives-python]] is preserved while its known-gap #4 is
closed. `pandas` is an *optional* extra (mandatory deps: `polars` +
`duckdb`; `narwhals` was removed); the python-ags4-shaped `check_file` dict and the Rust-CLI
`{file,findings}` JSON are **two deliberately distinct shapes**, both
reproduced exactly.

## Consequences
Commits the toolchain to: a maintained `laterite` library wheel as the
supported Python access to the clean-room engine; `laterite-py` as the
**only** pyo3-linking crate (the validator stays pyo3-free — Lint/CI
should keep asserting the lean dep-graph); `laterite.compat` tracking
`python-ags4`'s public surface with divergences pinned to the O-N
catalogue (`repo:OBSERVATIONS.md`);
the `lat` console script being byte-faithful to the Rust
binary. Rules out: a second Python re-implementation of the validator;
Python ever driving the shipped *binary*; `pandas` as a mandatory
dependency. Opens (not taken here): a downstream Python consumer
could later swap `from python_ags4 import AGS4` →
`from laterite import compat as AGS4` and drop `python-ags4` from its
*runtime* deps (kept as the dev oracle) — a ~1-line follow-up, its own
change.

## Related
[[dec-rust-drives-python]] · dec-rust-engine-staged-adoption · [[design/_README|AGS5 register]] ·
[[laterite-ags4-check]] · laterite-ags5-db · [[parity-model]] ·
[[dec-duckdb-extension|laterite-duckdb: the DuckDB-host read surface]]
