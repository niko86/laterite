---
type: decision
title: "laterite — the public Rust API surface, and publishing the engine to crates.io"
status: accepted
tags: [design, decision, api, rust, crates-io, versioning]
decided: 2026-07-29
supersedes: []
from_gap: []
related: [api-surface-1.0, dec-rust-drives-python, dec-laterite-ags4-types-leaf, crate-map, crate-dependency-graph, dec-monorepo-structure, dec-duckdb-extension, pyo3-boundary, reliquary]
sources:
  - "https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html"
  - "https://doc.rust-lang.org/cargo/reference/publishing.html"
repo_refs:
  workspace: "repo:rust-packages/Cargo.toml"
  fingerprint: "repo:rust-packages/laterite-ags4-validator/build.rs"
  duckdb_consumer: "ext:niko86/laterite-duckdb:Cargo.toml"
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
  **Done** — but not written at the 21 sites. They live once in
  `[workspace.dependencies]` and each site says `{ workspace = true }`, because
  `[tool.bumpversion]` stamps `version = "{current_version}"` in
  `rust-packages/Cargo.toml` and rewrites *every* occurrence in that file. Inline
  versions would sit at the old number after a release and publish crates pinned to
  a version that was not the engine they shipped with; there, they bump in lockstep.
  `test_workspace_dependency_versions_match` asserts it rather than trusting it.
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
- **laterite#159 — no `include` allowlist. DONE.** `cargo package` ships everything not
  excluded, permanently and immutably. On a repo with a private-corpus discipline this
  is a leak path the existing diff-grep habit does not cover. Each of the ten engine-tier
  crates now carries an explicit `include`, and `tools/check_package_contents.py`
  diffs `cargo package --list` against `tools/release/package-contents.json` in the
  `rust` CI job so a new file entering a tarball goes red rather than being discovered
  after it is immutable.
- **laterite#160 — rename `laterite-types` → `laterite-ags4-types`. DONE.** Its own header
  reads "AGS4 type system"; the name should say so. crates.io has no rename, so this was
  free then and irreversible later. Landed in #176, reverted by #178 (below), restored
  in #184. Regenerating [[crate-dependency-graph]] (`tools/gen_crate_graph.py`) is
  CI-gated and passes.
- **laterite#161 — DuckDB naming in a format-neutral leaf, and it is dead. DONE (twice).**
  `GroupDescriptor::table()`/`view()` returned `g_<code>`/`v_<code>`, documented as the
  DuckDB table and view names, from the reference-data leaf. Publishing would have frozen
  an unrelated product's schema conventions into a dictionary crate's public API — but the
  sharper finding was that **they had no callers**: the only occurrences repo-wide were
  the two functions and their own unit test, and `laterite-duckdb` never references
  `GroupDescriptor` or builds a `g_`/`v_` name at all. So the fix was deletion, not
  relocation.

> [!warning] #178 reverted two merged PRs, and the second went unnoticed for days
> #178 was branched before #175 and #176 and squash-merged after them, so its diff
> restored what both had deleted. The #176 casualty (the 104-file crate rename) was
> caught during the 0.9.0 cut and fixed in #184. The #175 casualty — these two
> functions — was **not**, because laterite#161 was already closed as completed, so
> the reverted state read as the finished state. It was re-deleted during the #159
> work, when the crate's `include` put it back under scrutiny.
>
> No gate could see either one. The tree compiled, every test passed, and the result
> was internally consistent — a self-consistent revert is invisible to CI by
> construction. What closed the class was requiring branches to be up to date before
> merge (`strict_required_status_checks_policy`); what would have caught THIS instance
> sooner is not trusting a closed issue as evidence about the tree.

**`laterite-transport` keeps its name.** Its header records that the age + zstd
envelope works on any file; it is the one engine crate that would carry over unchanged
to a non-AGS4 format, so `laterite-ags4-transport` would be actively wrong. This is the
counter-example to "prefix everything": the rule is *encode format-boundedness*, and
that rule splits these two crates in opposite directions.

**Gates that must predate the first publish. DONE.** The `cargo package --list`
diff landed in #189; the other four landed together as the `publish-gates` CI job.
Each turned out to be a slightly different shape than the one-line plan implied:

- **Public API — a checked-in snapshot, not a `--diff` invocation.**
  `tools/check_public_api.py` renders each crate with `cargo public-api
  --all-features --omit blanket-impls` and compares it to
  `tools/release/public-api/<crate>.txt` exactly. A diff-against-a-baseline run
  reports; a snapshot puts the whole surface **into the PR's own diff**, which is
  the property that matters here — see the #178 warning above, where nothing
  could see two public functions come back from the dead.
- **`Send`/`Sync` — mechanical for named types, asserted for opaque ones.**
  `cargo public-api` renders auto-trait impls as lines, so the snapshot already
  carries `impl Send for …` for every public type and the tool fails when one is
  missing. No hand-maintained list, so nothing to go stale. `impl Trait` returns
  are the exception the original note flagged: rustdoc renders the declared
  bounds and stops, so the leaked auto traits appear nowhere. Those four are
  asserted at compile time in `laterite-ags4-reference/tests/auto_traits.rs`, and
  the tool refuses any `-> impl` the test file does not name.
- **MSRV — the floor comes from the manifests.** `tools/check_msrv.py` reads each
  crate's `rust-version` and builds it on exactly that, so raising the floor
  cannot leave the gate testing the old one. **Libraries only**: `criterion`
  needs 1.86, and widening to `--all-targets` would fail on a dev-dependency that
  is not part of the promise. The first run found the promise already broken — a
  `let` chain (stable in 1.88) in `laterite-ags4-emit`.
- **`cargo semver-checks` — baselined on `main` until there is something better.**
  Nothing is on crates.io, so the registry baseline does not exist yet; at first
  publish it should switch to `--baseline-version`. Note the consequence of it
  being a *blocking* gate: a PR that deliberately breaks an engine crate's API
  now fails until the version bump that justifies it is in the same PR. That is
  the intended trade — the alternative is a gate that reports and never stops
  anything — but it makes a breaking reshape a louder, more deliberate event than
  it used to be.

The job is a **new required-status-check context** (`publish-gates`) and gates
nothing until branch protection is told about it.

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

## `laterite` 0.1.0 — the facade exists

Built 2026-08-01, after the engine tier was already published. Scope is **read,
validate, write**; diff, merge, typed cells and the indexed `scan()` path stay
0.2 as planned, and all four are additive so nothing here has to move to admit
them.

The Option-C rules held in practice, and the rendered API is the evidence:

- Everything under `laterite::ags4`; the crate root carries only `Error` and
  `ErrorKind`.
- Handles opaque, `Debug` written by hand rather than derived — a derived
  `Debug` on `Document` prints every cell of a delivery file, which turns a
  stray `dbg!` or a panic message into a data dump and quietly makes the
  engine's field names part of what consumers see.
- **Zero third-party types in the public API**, now MECHANICALLY ENFORCED rather
  than reviewed for: `check_public_api.py`'s `check_no_third_party` requires
  every path root in the facade's rendered surface to be `laterite`, `core`,
  `alloc` or `std`. Verified to catch both leak shapes (a `serde_json::Value`
  return, an `encoding_rs::Encoding` argument), and it exempts the engine
  crates, which traffic in those on purpose.
- `encoding_rs` is not even a dependency: resolving the encoding inline lets
  inference carry the type, so the crate is absent rather than merely
  absent-from-signatures.

Two things the build changed elsewhere. `ErrorKind` gained an `Other` variant —
the engine's `ValidatorError::kind()` is documented as the single producer of
that token domain, so the facade maps from the token rather than re-matching
variants, and an unmapped future token needs an honest home rather than being
filed under whichever kind looked closest. And `publish_crates.py` now reads
**per-crate** versions: one `workspace_version()` would have asked crates.io
about `laterite 0.9.0`, been told no, and tried to publish it.

### 0.1.1 — `validate_bytes`, and a justification that did not hold

0.1.0 shipped `validate` taking a **path only**. The reason written at the
function was that Rule 20 concerns files on disk beside the `.ags`, so a bytes
API "could only ever answer half of it" and would be added "when it can say
honestly which half it ran".

That reason was already satisfied when it was written. Rule 20 has lived in two
modules since the `cert-trust-v2` arc (2026-07-14): the data-level half in
`rules/references.rs`, a pure function of the bytes, and the on-disk half in
`world.rs`. [[cert-trust-v2|`WorldScope`]] exists precisely to say which one ran,
and its own doc rejects the `(bool, Option<&Path>)` shape because that pair has a
fourth state — *asked for the check, had nothing to check against* — which the
engine used to answer by quietly reporting Rule 20 clean. Ask for the world check
with no source and you get `WorldCheckRequiresSource`.

Every other surface was already using it: Python sniffs bytes, Node builds all
three arms explicitly, wasm is `WorldScope::None` always, and the emitter
re-validates its own output in memory. The facade was the only one that could
not, and its error mapping already handled the `world_check_requires_source`
token it never produced. The omission was mine, and the comment made it read as
principled.

So 0.1.1 adds `ags4::validate_bytes`. Both arms end at `check_parsed_with_dict`
— deliberately, because that door does four things (resolve `TRAN_AGS`, apply the
4.0.3→4.0.4 guard, run the rules, emit the transparency FYI) and the engine
records that **every** surface which hand-assembled them skipped the guard,
judging one file against two dictionaries depending on how it arrived. A test
asserts a path and its own bytes produce identical findings, which is that bug
stated as an assertion.

The motivating case is a service that validates uploads without exposing a
filesystem — the same shape as "merging a disk file with bytes off a network",
already on record as laterite#162.

## The first publish happened — 0.9.0, 2026-08-01

All eight went out: `laterite-ags4-parse`, `-types`, `laterite-transport`,
`-reference`, `-core`, `-validator`, `-emit`, `-trust`. `laterite-ags4-diff` and
`-merge` are **not** on crates.io, which is the `publish = false` guard doing
exactly its job rather than a decision anyone had to remember at the keyboard.

`tools/publish_crates.py` runs it: waves derived from the manifests, a wait for
each wave to become *resolvable* (not merely uploaded) before the next starts,
and idempotent so a failure is resumed rather than restarted.

### Two things crates.io does that only a publish reveals

Both were hit, neither is a defect, and neither left partial state — but both cost
a stop, and the 0.2 publish of diff/merge should not rediscover them.

- **A verified email address is required, and nothing says so until the upload.**
  crates.io fills the address in from GitHub and leaves it UNVERIFIED. Account
  creation, token scopes, `cargo login`, packaging and the full verification
  build all succeed; the rejection arrives at the upload itself, as
  `400 A verified email address is required to publish`. Verify at
  <https://crates.io/settings/profile> *first*.
- **New crates are rate-limited.** A burst allowance, then roughly one new crate
  per interval. Publishing eight tripped it on the eighth, with a `429` naming
  the retry time. This is the reason idempotency is not a nicety: the fix is to
  wait and re-run, and a re-run must not attempt to re-upload the seven that
  already went out.

### What the publish proved that no local gate could

`laterite-ags4-trust`'s verification build **downloaded `laterite-ags4-core 0.9.0`
from the registry** and compiled against it. Every earlier check ran against path
dependencies; this is the first evidence that the published artefacts resolve and
build as a stranger receives them — which is precisely the thing
[[dec-rust-api-crates-io|#158]] was about.

## The CLI is `laterite-cli`, and it cannot publish yet

Renamed from `laterite-ags4-check` on 2026-08-01. The binary it ships is still
`lat` — only the crate is renamed, so nothing a user types changes.

The rename happened **now** for one reason: crates.io has no rename. A crate name
is free to change until its first publish and irreversible after, which is the
[[reliquary|#160]] lesson learned somewhere cheaper. `laterite-ags4-check` named
a crate that stopped being a checker several verbs ago — it reads, fixes, diffs,
merges, certifies, packs and converts — and `-ags4-` marks the engine tier, which
this is not.

It was **not** added to the publish set, and the reason is worth writing down
because it looks like an oversight:

- **A binary's dependencies are still dependencies.** Publishing strips `path`,
  so every in-workspace dep must itself be on the registry. `laterite-cli` needs
  `laterite-ags4-diff` and `laterite-ags4-merge` — both deliberately held for 0.2
  — plus `laterite-cliutil` and `laterite-excel`, which carry `publish = false`
  and have never been considered for the registry at all. Publishing the CLI
  means publishing four more crates, two of them against a decision recorded
  above. That is the trade the crate split bought us, seen from the other side:
  the CLI is separate *precisely* so its `clap`/`ratatui`/`calamine` weight stays
  off every library consumer, and the same boundary means it drags its own tier
  along when it goes out.
- **Both publish gates assume a library.** `check_public_api.py` renders with
  `cargo public-api` and `check_semver.py` runs `cargo semver-checks`; a bin-only
  crate has no lib target for either to read. `PUBLISH_SET` is the single list
  both gates and the packaging gate share, so adding the CLI to it breaks two of
  the three. The fix is an exemption — a binary has no public API to freeze, so
  there is genuinely nothing for those gates to check — and **not** a `src/lib.rs`
  invented to satisfy a tool.

So the CLI ships with 0.2, alongside the diff/merge publish it already depends
on. Until then `lat` is distributed as it is today: per-target binaries from the
release workflow. `cargo install laterite-cli` is a convenience, not an API
promise, and it is the one part of this plan with a good reason to wait.

## Related

[[api-surface-1.0]] · [[dec-rust-drives-python]] · [[dec-laterite-ags4-types-leaf]] ·
[[crate-map]] · [[crate-dependency-graph]] · [[dec-duckdb-extension]]
