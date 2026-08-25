---
type: decision
title: "Rust ships, Python is a library, Rust drives Python"
status: accepted
tags: [design, decision]
decided: 2026-05-18
supersedes: []
from_gap: []
related: [build-rust, design/_README, dec-python-imports-rust-library, dec-laterite-ags4-types-leaf, dec-monorepo-structure, dec-duckdb-extension]
sources: []
---

# Rust ships, Python is a library, Rust drives Python

## Context
The repo grew two parallel CLI implementations: the Python
`ags5db-py` (`packages/ags5-db`) and the Rust `ags5db` <!-- retired: ags5-db -->
(`rust-packages/ags5db`). laterite-ags5-db reached **read+write parity**
(every documented command implemented, NDJSON parity-verified against
the Python reference). Maintaining a *shipped* Python binary alongside
it — via PyInstaller / Nuitka / PyApp / make-dist — meant two
distribution code paths for one product surface, and the Python
packagers each had real defects (PyInstaller 7.6 s cold-start, Nuitka
3.14 `_uuid` wall, PyApp dropped exit codes). The distribution +
language strategy was implicit and the docs had drifted (e.g.
`tools/README.md` claimed the Rust write commands were "stubbed exit
9" when they are fully implemented).

## Options considered
1. **Ship both binaries** — redundant; doubles the packaging matrix,
   keeps the broken Python packagers on life support.
2. **Ship Python only** — abandons the 9 ms cold-start, single-file
   Rust binary for a 118 MB / 7.6 s bundle.
3. **Ship Rust only; Python = library + parity oracle** — one
   distribution; Python keeps its value as the reference
   implementation the Rust NDJSON parity tests check against and as a
   `uv run` dev entrypoint.
4. **+ a future Rust→Python boundary** — when a Python-only capability
   is needed from the shipped binary, embed Python (PyO3 / embedded
   interpreter) so **Rust drives Python**, never the reverse.

## Decision
Option 3 now, Option 4 as the named future boundary. The Rust
laterite-ags5-db binary (built by [[build-rust]]) is the **sole shipped
artefact**. The Python `ags5-*` packages are a **library + parity
oracle**, never packaged into a binary, never a driver of Rust. The
PyInstaller / Nuitka / PyApp / make-dist apparatus is deleted. The
intended future Rust↔Python boundary is **Rust drives Python** (PyO3
or an embedded interpreter); status: `decided, unimplemented`.

## Why
One product surface, one distribution path. The Rust binary already
has parity, a 9 ms warm cold-start, and propagates exit codes (the
agent contract the Python packagers broke). Python's enduring value is
*not* as a shipped CLI — it is the clean-room parity reference
([[parity-model]]: a divergence is by construction Rust-vs-python) and
a fast iteration surface via `uv run`. A Rust→Python embedding (never
the reverse) keeps the single-binary distribution intact even if a
future capability is Python-only, while never making Python a runtime
prerequisite of the shipped tool.

## Consequences
Commits AGS5 tooling to: a single `tools/build-rust` ship path;
`ags5db-py` as a dev/oracle entrypoint only; docs that state this
plainly. Rules out: shipping a Python binary; Python ever invoking the
Rust binary as its execution engine.

> [!todo] **Known gaps — roadmap follow-ups (not in the cleanup pass)**
> 1. `ags4-to-db --append` is declared but Phase E v1 always writes a
>    fresh dst (`laterite-ags5-db`'s `ags4_to_db` command, not in this tree).
>    Highest priority — Rust is now the only distribution.
> 2. Stale comment at `…ags4_to_db.rs:19` claims passthrough errors
>    out; it is in fact implemented (`build_passthrough_descriptors`,
>    lines 304-351). Comment-only fix; bundle with (1).
> 3. ~~`query` (the deprecated `sql` alias the Python CLI carries) is
>    absent in Rust by design — decide: add the alias, or drop it
>    from Python for symmetry.~~ **RESOLVED (gate B.3, 2026-05-22):**
>    not added to Rust. The shipped binary's canonical name is `sql`;
>    `laterite.ags5db.sql` uses that. The Python CLI's deprecated
>    `query` alias retires with the Python CLI in Stage C — no Rust
>    change. See dec-rust-engine-staged-adoption PR-B3.
> 4. ~~**PyO3 spike** to validate the Rust→Python boundary~~ **DONE** —
>    discharged by [[dec-python-imports-rust-library]]: the `laterite`
>    package realised the boundary in the library direction (PyO3
>    cdylib over the engine). Stage A of the staged-adoption roadmap
>    (codec.py → `laterite.compat`, python-ags4 → dev-only oracle)
>    builds on it.
> 5. ~~Folder renames `rust-cli`→`ags5db`, `rust-laterite-ags4-validator`→
>    `laterite-ags4-validator`~~ **DONE** (see `ags-wiki/log.md` 2026-05-18):
>    both crate dirs renamed to match their artefacts; ~125 refs
>    rewritten (Cargo path-deps, `.bootstrap/*.py` generators, wiki
>    `repo:` authority pointers, LICENSE, tools); cargo 241✓ / pytest
>    209✓ / laterite 60✓ / LINT CLEAN. `rust-packages/README.md` added.
> 6. ~~`test_packaging.py` asserts the standalone `uv tool install
>    ./packages/ags5-db` → `ags5db-py` contract~~ **RETIRED (F2c-6, <!-- historical -->
>    2026-05-25).** `ags5-db` and `ags5-ags4` were deleted entirely in <!-- historical -->
>    F2c-6, and the standalone-install probe this bullet named
>    (`test_ags5_db_standalone_install_runs_lazy_imports`) was retired
>    with them — its docstring now records that explicitly. The test
>    file moved out of this tree with the
>    rest of the dormant AGS5 strand (#177, 2026-06-21) and
>    today asserts only `test_workspace_packages_declare_their_sibling_imports`
>    — a package-agnostic static check that every AGS5-strand package's
>    sibling import is declared as a dep, unrelated to any shipped-binary
>    or `uv tool install` contract. AGS5 is a dormant concept, not a
>    shipped package.

## Related
laterite-ags5-db · [[build-rust]] · [[design/_README\|AGS5 register]] ·
[[dec-python-imports-rust-library|laterite: the scoped Python→Rust exception]] ·
staged adoption roadmap (A→F) ·
Stage F1: msgspec kernel retirement path ·
[[dec-laterite-ags4-types-leaf|laterite-ags4-types wasm-safe leaf crate]] ·
[[dec-monorepo-structure|one repo, many artifacts — stay monorepo]] ·
[[dec-duckdb-extension|laterite-duckdb: AGS4 read surface for the DuckDB host]] ·
#177: AGS5 decoupled into the dormant AGS5 strand
