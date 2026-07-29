---
type: decision
title: "laterite — the public Rust API surface, and publishing the engine to crates.io"
status: accepted
tags: [design, decision, api, rust, crates-io, versioning]
decided: 2026-07-29
supersedes: []
from_gap: []
related: [api-surface-1.0, dec-rust-drives-python, dec-laterite-types-leaf, crate-map, crate-dependency-graph, dec-monorepo-structure, dec-duckdb-extension, pyo3-boundary, reliquary]
sources:
  - "https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html"
  - "https://doc.rust-lang.org/cargo/reference/publishing.html"
repo_refs:
  workspace: "repo:rust-packages/Cargo.toml"
  fingerprint: "repo:rust-packages/laterite-ags4-validator/build.rs"
  duckdb_consumer: "repo:../laterite-duckdb/Cargo.toml"
---

# laterite — the public Rust API surface, and publishing the engine to crates.io

> **DECIDED 2026-07-29.** The API design is settled; nothing is published yet.
> Sibling to [[api-surface-1.0]], which is the *Python* surface — this page is the
> Rust one. Prerequisite work is tracked as laterite#158–#162.

## Context

There is no unified Rust API. `laterite-py` (PyO3), `laterite-node` (napi) and
`laterite-ags4-wasm` each independently assemble their own surface out of the engine
crates; a Rust user faces 24 crates, all `publish = false`, and no front door. The
Python side exposes ~30 items (`read`, `validate`, `build_ags4`, `diff`, `fix`,
`from_excel`, `Report`, `Ags4File`, `AgsQuery`, …) with no Rust equivalent.

The question arrived as "should we publish to crates.io", motivated by wanting to
**roll back to an older engine version** and to **reproduce an old build**. Both
motives turned out to be already satisfied, which reframed the decision:

- **Rollback.** `laterite-duckdb` — the one external Rust consumer — pins the engine
  by git submodule SHA. That is exact and content-addressed. End users roll back via
  `pip install laterite==0.7.0` / `npm i laterite@0.7.0`, which already works.
- **Reproducibility.** A tag plus `Cargo.lock` reproduces the source; GitHub release
  assets and the published wheels/packages are the artifact record.

What crates.io actually adds is **durability and reach**, not exactness: a SHA needs
the repo to exist, stay public and keep the commit reachable; a published version is
append-only on mirrored infrastructure, vendors offline, and is the only realistic
door for a third-party Rust user. Note also that **a crate with a git dependency
cannot itself be published**, so submodule pinning is a dead end for anything
downstream that might want to publish.

**So publishing is a distribution decision, not an engineering-hygiene one.**

### The mechanic that governs everything

When a dependency carries both `path` and `version`, Cargo uses the **path** for
in-workspace builds and the **version** only for outside consumers.

| Declaration | In-workspace | Published consumer |
|---|---|---|
| `{ path = "../x", version = "0.8" }` | path — the tree | registry `0.8` |
| `{ version = "0.8" }` | registry `0.8` | registry `0.8` |
| `{ path = "../x" }` | path | *cannot be published* |

Consequences, verified against the workspace:

- Publishing does **not** change how `laterite-py`/`-node`/`-wasm`/`lat` build. They
  keep compiling the tree. A version field on their deps is a claim nothing verifies.
- `laterite-py` is `crate-type = ["cdylib"]` + `publish = false`; a cdylib can never
  be a crates.io library dependency, so its dep versions would be read by nobody.
- Only deps **of publishable crates** need version fields: **21 sites**, not the 84
  across the whole workspace. The 3 dev-only path deps are stripped at publish.
- Making a surface genuinely build from the registry requires *deleting* the `path`
  key, which costs the dev loop entirely (every engine edit needs a publish first).
  That trade only makes sense across a repo boundary — which is what
  `laterite-duckdb` already is.

### The engine tier

Ten crates, verified **dependency-closed** (nothing in the set reaches a crate left
unpublished), sorting into five publish waves:

| Wave | Crates |
|---|---|
| 1 | `laterite-ags4-parse` · `laterite-transport` · `laterite-ags4-types` |
| 2 | `laterite-ags4-reference` |
| 3 | `laterite-ags4-core` · `laterite-ags4-diff` · `laterite-ags4-validator` |
| 4 | `laterite-ags4-emit` · `laterite-ags4-trust` |
| 5 | `laterite-ags4-merge` |

`laterite-duckdb` needs only `laterite-ags4-core`, `laterite-ags4-types` and
`laterite-ags4-reference` — whose closure is waves 1–3 minus diff/validator.

Names: every `laterite*` name is free on crates.io; **`lat` is already taken** by an
unrelated crate, so `cargo install lat` can never be the story (the *binary* may still
be named `lat` from a differently-named crate).

## Options considered

Three independent API proposals were developed and adversarially reviewed.

**A — cross-ecosystem familiarity.** Mirror the Python surface so one vocabulary spans
PyPI, npm and crates.io. *Rejected*: its thesis ("the engine reshapes freely") is
contradicted by its own design, which re-exports the dictionary-derived
`GroupDescriptor`/`Heading` pub-field structs verbatim while refusing to add
`#[non_exhaustive]` upstream. That freezes the projected schema of
`ags_dictionary.json` — the one file the architecture exists to keep editable — at
first publish. It also puts verbs at the crate root (`laterite::read`), which fails
the AGS4→AGS5 naming question permanently.

**B — Rust-native ergonomics.** Borrow-friendly, typestate, builders, `impl Trait`
returns. *Rejected despite being the best-engineered of the three*: a
`Provenance` typestate on the central document type propagates into **downstream**
signatures, so removing it later edits user code, not just ours. It also forces all
documents in a merge to share one provenance — ruling out merging a disk file with
bytes off a network (recorded as laterite#162). Its eight verbatim engine-enum
re-exports can only be made safe by adding `#[non_exhaustive]` upstream, which would
force wildcard match arms into `laterite-py`/`-node`/`-wasm`.

**C — minimal commitment surface.** Treat the semver commitment as the dominant cost;
publish only what is already frozen by a contract we cannot break anyway.

## Decision

**C wins**, with grafts. The consolidated shape:

- **Everything under `laterite::ags4`.** The crate root stays format-neutral — this is
  the entire AGS5 insurance policy, and it is why A's root-level verbs were rejected.
- **Opaque handles with private fields** (`Document`, `Report`, `Certificate`, `Fixed`,
  all `*Options`). No engine type appears in any default-feature public signature.
- **Opaque `Error` struct** with a coarse `#[non_exhaustive] ErrorKind`, plus
  `kind_str()` (the frozen wire token shared with Python/Node/`lat`) and
  `exit_code()`. `source()` returns a *private* newtype so the anyhow/eyre chain
  renders but downcasting can never reach an engine type.
- **Zero third-party types in public signatures** — no `serde_json::Value` (→ `Cell`),
  no `uuid::Uuid` (→ `RowId([u8;16])`), no `chrono` (→ ISO strings), no `encoding_rs`
  (→ WHATWG label strings), no `arrow`. This is the highest-leverage rule: no
  dependency's major can ever force ours.
- **`unstable-engine` feature** as the only escape hatch — chosen over
  `#[doc(hidden)]` because it appears in the downstream's own `Cargo.toml`, whereas
  hiding from rustdoc hides nothing from the compiler.
- **Grafted from B**: caret version requirements; a `Row<'a>` handle with an amortised
  per-`Group` column index (without it, cell lookup is O(headings) per cell);
  `Cell::Verbatim`, so `format(parse(x))` stays an identity. `scan()`/`Indexed` are
  reserved as the flagship 0.2 addition.
- **Grafted from A**: option setter names match the Python kwarg names (`warnings`,
  `fyi`, `risky`, `synthesise_metadata`, `on_type_clash`) — already frozen by two
  ecosystems, so adopting them costs nothing and makes the Python docs readable as
  Rust docs.

### Versioning

**`laterite` starts at 0.1.0 on its own clock**, not the workspace 0.8.x — which would
advertise eight minors of API history that do not exist.

**The engine tier keeps lockstep 0.8.x with caret requirements (`"0.8"`), and any
breaking reshape bumps the 0.x minor (0.8 → 0.9), never a patch.** Exact `=` pins were
explicitly rejected: for a `0.x` crate Cargo unifies to one version within a
compatibility range, so the moment `laterite` pins `=0.8.1` and `laterite-duckdb`
still pins `=0.8.0`, resolution is **unsatisfiable** with no user-side remedy. Caret
plus minor-for-breaking gives the same protection without the diamond.

This is a step toward [[reliquary|per-crate versioning]] (laterite#153) without
deciding that question now.

## Consequences

**Blocking prerequisites** (all latent defects today, all worth fixing regardless):

- **laterite#158 — the engine fingerprint does not survive packaging.**
  `laterite-ags4-validator/build.rs` derives its covered set by recursing into `path`
  deps read from `Cargo.toml`; `cargo publish` strips that key, so the recursion
  silently stops and the fingerprint covers only the validator's own files. That
  regresses to exactly the stale-`Vouched` bug #550 fixed. The recommended API exposes
  `engine_fingerprint()` publicly, which turns an internal narrowing into a published
  contract that misreports itself.
- **laterite#159 — no `include` allowlist.** `cargo package` ships everything not
  excluded, permanently and immutably. On a repo with a private-corpus discipline this
  is a leak path the existing diff-grep habit does not cover.
- **laterite#160 — rename `laterite-types` → `laterite-ags4-types`.** Its own header
  reads "AGS4 type system"; the name should say so. crates.io has no rename, so this is
  free now and irreversible later. Requires regenerating [[crate-dependency-graph]]
  (`tools/gen_crate_graph.py`), which is CI-gated.
- **laterite#161 — DuckDB naming in a format-neutral leaf, and it is dead.**
  `GroupDescriptor::table()`/`view()` return `g_<code>`/`v_<code>`, documented as the
  DuckDB table and view names, from the reference-data leaf. Publishing would freeze an
  unrelated product's schema conventions into a dictionary crate's public API — but the
  sharper finding is that **they have no callers**: the only occurrences repo-wide are
  the two functions and their own unit test, and `laterite-duckdb` never references
  `GroupDescriptor` or builds a `g_`/`v_` name at all. So the fix is deletion, not
  relocation. Inventoried in [[reliquary]] as `spotted`.

**`laterite-transport` keeps its name.** Its header records that the age + zstd
envelope works on any file; it is the one engine crate that would carry over unchanged
to a non-AGS4 format, so `laterite-ags4-transport` would be actively wrong. This is the
counter-example to "prefix everything": the rule is *encode format-boundedness*, and
that rule splits these two crates in opposite directions.

**Gates that must predate the first publish**: `cargo public-api --diff`,
`cargo semver-checks`, a `cargo package --list` diff per crate, an MSRV job, and
`Send`/`Sync` static assertions on every public handle (auto-traits leak through
`impl Trait` returns and become a silent major bump otherwise).

**Deliberately deferred**: diff and merge ship in 0.2, not 0.1 — keeping two crates off
crates.io and the frozen surface smaller. `laterite-duckdb` migrates off its submodule
**one cycle after** first publish, so a first-ever publish cannot break a shipped
extension. `laterite-ags5` will **not** be squatted: the name is brand-prefixed so the
risk is negligible, and a placeholder crate would publicly signal intent on a repo
whose stated posture is that AGS5 is a dormant concept.

**Round one is 8 crates**, not 10 (diff and merge deferred) and not 7 —
`laterite-transport` is `optional = true` but `default = ["transport"]` in
`laterite-ags4-core`, so it cannot be dropped without a breaking default-feature
change.

## Related

[[api-surface-1.0]] · [[dec-rust-drives-python]] · [[dec-laterite-types-leaf]] ·
[[crate-map]] · [[crate-dependency-graph]] · [[dec-duckdb-extension]]
