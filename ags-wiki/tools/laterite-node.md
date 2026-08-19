---
type: tool
title: laterite-node
status: drafted
tags: [tool, internal, napi, node]
tool_kind: crate
language: rust
artifact: "laterite-node (.node addon + TS laterite package)"
ags_editions: [4.0.3, 4.0.4, 4.1, 4.1.1, 4.2]
volatile: [phase]
volatile_asof: 2026-08-03
repo_refs:
  root: "repo:rust-packages/laterite-node"
  lib: "repo:rust-packages/laterite-node/src/lib.rs"
  ts: "repo:rust-packages/laterite-node/ts/index.ts"
related: [crate-map, laterite-ags4-validator, laterite-ags4-types, laterite-py, laterite-ags4-wasm, pyo3-boundary, dec-rust-drives-python, ags4-output, dec-ags4-merge-semantics, edition-resolution, laterite-ags4-reference, surface-census, data-single-source-audit, dec-ags-idx-certificate]
sources: []
---
# laterite-node

<!-- BEGIN GENERATED: crate-card — DO NOT EDIT BY HAND. Regenerate: uv run --no-project python tools/gen_crate_graph.py -->
> [!note] **Not published** — `laterite-node` is a workspace crate, internal to this repo, at v0.11.0 (its own line).
> **Used by** — nothing else in this workspace.
<!-- END GENERATED: crate-card -->

> [!note] the napi-rs cdylib + co-located TS package: the **Node.js** host
> binding, the direct analog of [[laterite-py]] for the JS/TS half of the
> toolchain.

> [!important] **Published.** `laterite` shipped to npm at **0.1.0 (2026-06-15)**
> This page claimed "not yet published to npm (that is P4)" for six weeks after it
> stopped being true — `laterite@0.8.0` is current, alongside the three
> `@laterite/native-*` platform packages, and releases run from CI via OIDC with
> provenance. The npm package is a **released surface** on its own `node-v*` tag
> train. See `repo:RELEASING-node.md`.

## What it is

The **Node-API (napi-rs) bindings** exposing the clean-room
[[laterite-ags4-validator]] engine to Node.js — the Node analog of [[laterite-py]]
(PyO3). A `cdylib` `.node` addon re-expresses the **DuckDB-free** engine
through `#[napi]` instead of `#[pyclass]`/`#[pyfunction]`, returning each
group as a typed **Arrow IPC `Buffer`** — exactly the marshalling
[[laterite-ags4-wasm]] already frames for the browser (the napi boundary has no
pyo3-arrow-capsule analog, so IPC `Buffer`s are the boundary). A
co-located TypeScript `laterite` package layers the high-level API on top
(the `__init__.py` analog).

The typing is **byte-identical across hosts by construction**: the addon,
[[laterite-py]] and [[laterite-ags4-wasm]] all call the one shared
`laterite_ags4_types::arrow_cols::build_record_batch` (a 2DP heading → Float64, YN →
Bool, DT → Timestamp(µs), ID/X → Utf8). Engine crates reused unchanged:
[[laterite-ags4-validator]], [[laterite-ags4-types]] (`arrow`), `laterite-ags4-emit` (`arrow`).

## Two layers

- **Native addon** (`src/lib.rs`): `parseArrow` → a `Reading` handle
  (`tableIpc(code)` typed Arrow IPC, `meta`, `groupCodes`, `tranAgs`,
  `emit`); `runCheck` (mirrors laterite-py's `run_check` dict —
  `{ok, file, count, exitCode, findings, json, ndjson}` + the
  `{ok:false, errorKind, exitCode}` failure shape, with byte-faithful
  `findings_json`/`findings_ndjson`); `emitAgs4FromIpc` (data → AGS4 via
  `group_from_ipc` + `laterite_ags4_emit::emit_ags4`). Auto-camelCased; auto
  `index.d.ts`.
  Plus `canonicalType`/`displayHint`/`parseValue` (over `laterite_ags4_types`) and
  `transportPack`/`Unpack`/`Lock`/`Unlock` (zstd + age, reimplemented on
  `zstd`/`age` directly — NOT laterite-ags4-core, which would drag in Excel/csv for a
  niche feature; the age envelope stays pyrage-interoperable). Also
  `editions()`/`fallbackEdition()` (2026-07-14), both projecting
  [[laterite-ags4-reference]]'s generated `DictVersion::ALL`/`FALLBACK` — so no
  JS-side list of AGS4 editions is hand-written; see [[edition-resolution]]. And
  `resolveEncodingLabel(label)` (2026-07-14) — what THIS surface's own
  `resolve_encoding` wrapper turns a label into, `null` if it refuses. That wrapper
  used to fall back to UTF-8 on an unresolved label (`.unwrap_or(UTF_8)`); every
  encoding-consuming call (`runCheck`/`fixFile`/`diff`/`merge`/`parseArrow`) now
  raises instead (`bad_args`, exit 5), matching Python. The npx `lat` shim
  (`ts/cli.ts`) had its own, separate bug: its flag parser is global (one
  valued-flag set for every verb, unlike clap's per-verb declarations), so
  `--encoding` was silently accepted and dropped on `validate`/`fix`/`diff`/
  `certify` — now threaded through; `read`/`excel` genuinely cannot honour one and
  now refuse the flag instead of swallowing it. [[surface-census]] gained a third
  table (`encodings`) that calls `resolveEncodingLabel` to check this launcher
  against the others. See [[data-single-source-audit]].

  **The `ts/cli.ts` flag parser was rebuilt outright (2026-07-14, `CENSUS_VERSION`
  3→4).** The encoding fix above plugged one flag; the root cause was that this
  launcher had **one global valued-flag set for every verb and no per-verb notion
  of "accepted" at all**, where clap declares flags per verb — so it silently
  swallowed any flag on any verb. `HANDLERS` → `SPECS`: one table carrying
  dispatch + per-verb `flags` / `valued` / `positionals`; `pickVerb` finds the verb
  *before* parsing (so a valued flag on an unrelated verb can no longer be
  mistaken for the verb token itself); `rejectUnknownFlags` refuses anything the
  verb doesn't declare (exit 5). The flags census's per-verb table (a ✅/❌ diff of
  that declaration against clap's) surfaced three real bugs at once: `--dict
  <custom.ags>` was accepted, ignored, and validated against the *bundled*
  dictionary anyway (exit 0, "clean" — the binary and uvx both refuse it, exit 5);
  a typo'd `--no-warnigs` was silently swallowed rather than rejected; and
  `--index <cert>` was accepted and dropped entirely (`ValidateOptions extends
  ReadOptions`, so `tsc` was happy while the certificate went nowhere — a cert
  minted for a *completely different file* changed nothing). `--index` is now
  `validateWithCert`: `read(file, {index})` freshness-checks it and
  `Ags4File.validate()` decides whether it may skip the engine — the SAME library
  cert policy Python already had, not a second hand-written copy; the CLI adds
  only the recovery posture (an untrustworthy cert is a stderr note, not an
  error). `lat certify`'s result also moved from stderr to **stdout**, matching
  the binary and uvx — it had been the one launcher where `CERT=$(lat certify
  f.ags)` captured an empty string. See [[surface-census]] · [[data-single-source-audit]]
  row 6. **Open, not fixed here:** `--index` now parses identically on all three
  launchers but is *honoured* differently — see [[dec-ags-idx-certificate]] →
  "Known divergence".
- **TS `laterite` package** (`ts/`): `read` → `Ags4File` (`table(code)`
  decodes the IPC straight to an arrow-js Table — **Arrow-direct, no
  engine**; metadata; `toAgs4Text`/`write`; optional `sql`/`at`/`connection`;
  the **fluent chained verbs** `Ags4File.validate()` [→ this, `.report`] /
  `.fix()` [→ new repaired handle, `.fixReport`] / `.diff(other)` [→
  `RevisionDelta`] — the handle retains its read source so they re-run against the
  true bytes/encoding; #294 Batch E, owner-chosen 2026-07-02 to build the fluent
  layer; plus the **certificate lifecycle** `Ags4File.certify()` [mint `.ags.idx`]
  / `read(f, {index})` [consume + freshness-check, else `StaleCertError`] / the
  `.validate()` cert-skip fast-path, over the ONE core `Sidecar` class — a
  Node-minted cert is byte-/checker-compatible with Python + `lat`; #294
  Batch E/#14);
  `validate` → `Report` (`isValid`/`count`/`byRule`/`toJson`/`toNdjson`);
  `diff(a,b)` → `RevisionDelta` (the KEY-aware/type-aware revision diff over the
  shared `laterite-ags4-diff` leaf — byte-identical to Python/wasm/`lat diff`;
  #294 Batch E/#4);
  `merge(sources[], opts)` → `MergeResult` (N-ary reconciliation of AGS4
  deliveries over the shared `laterite-ags4-merge` leaf, 2026-07-12 — union
  semantics, later-argument wins a KEY conflict, `opts.onTypeClash` (typed
  `"error" | "widen" | "promote"`, default `"error"`) settles
  a TYPE disagreement: `"widen"` falls back to `X`, `"promote"` keeps the
  greatest `nDP` precision (zero-padding the coarser values); see
  [[dec-ags4-merge-semantics]]);
  `emitAgs4` → `EmitResult` (frames **or** a typed-graph tree); `errors.ts`
  (the `Ags4Error` hierarchy + the kind→exception map, the `_errors.py`
  analog); the `agsTypes`/`registry`/`transport` namespaces; the 174
  TS-generated typed-graph classes (`import { PROJ, LOCA } from "laterite"`) —
  the full AGS4 union (#173 repointed `generate-typed-graph.mjs` at
  `ags_dictionary.json`; 92 was the pre-#173/AGS5-only count).

**`registry`'s tree walks are now native, not re-implemented (2026-07-17,
#532, part of the #527 leaf-convergence arc).** `ts/registry.ts`'s
`ancestorChain`/`inheritedKeyNames` used to walk `.parent` pointers and
KEY-intersect against the direct parent in TypeScript — a hand-kept-in-sync
copy of `laterite_ags4_core::registry::{ancestor_chain, inherited_key_names}`,
the same leaf [[laterite-py]] already binds (`registry_fns.rs::
registry_ancestor_chain`/`registry_inherited_key_names`, the D2 step of
dec-rust-engine-staged-adoption) — its own doc comment said "matches the
Rust/Python check", i.e. a copy, not a binding. Two new `#[napi]` fns of the
same names (camelCased `registryAncestorChain`/`registryInheritedKeyNames` in
the generated `index.d.ts`) call that leaf directly
(`repo:rust-packages/laterite-node/src/lib.rs`); the two TS functions are now
a pure native passthrough, with the native unknown-code error re-typed to
`Ags4Error` so the facade's contract is unchanged (message byte-identical:
`unknown group code: "NOPE"`). Deliberately **not** total convergence —
`isKeyStatus`/`keyHeadings`/`nonKeyHeadings`/`childGroups` stay local TS views
over the drift-tested generated `registry.generated.ts`, exactly as Python's
own `registry.py` facade keeps `is_key`/`key_headings`/`child_groups`
Python-side (it too binds only the same two walks) — only the drift-prone
logic that has a core-leaf authority gets bound. Values unchanged from the old
TS implementation: `ancestorChain("SAMP") = ["SAMP","LOCA","PROJ"]`;
`inheritedKeyNames("SAMP") = {"LOCA_ID"}`. See
`repo:rust-packages/laterite-node/test/p3-registry.test.ts` and
[[data-single-source-audit]] (the #181 audit's "relationship logic
reimplemented per surface... logic-duplication only, no action" call — #532
closes that for these two walks on this surface).

## Why Arrow-direct (no DuckDB by default)

`read`/`table`/`validate`/`emit` need no engine: the IPC → arrow-js Table
path is faster + lighter, and Python's reason to route reads through DuckDB
(a pyarrow-free *pandas* path) does not exist in Node. DuckDB
(`@duckdb/node-api`) is an **optional peer** gated behind `sql()`/`at()`
only. See [[dec-rust-drives-python]] for the host-binding direction.

**The peer range must carry a prerelease component, and there is a gate.**
`@duckdb/node-api` publishes ONLY prereleases — all 56 versions are `X.Y.Z-r.N`
— and semver excludes a prerelease from a range unless some comparator shares
its major.minor.patch *and* carries a prerelease itself. So `>=1.5.0` matched
**nothing**; neither did `>=1.5.0-0`, nor `*`. That range shipped in
`laterite@0.10.1`, which made `sql()` and `at()` unreachable for anyone
installing from npm: `npm i laterite` then `npm i @duckdb/node-api` — the exact
command `ts/duckdb.ts`'s own error prints — fails with **ETARGET**; installing
both at once drops the peer silently; installing the peer first has it
*removed* when laterite arrives. No gate saw it because `npm ci` installs from
the lockfile, so the range is never consulted. It is now `>=1.5.5-0`, pinned to
the line the devDependency tracks, and `repo:rust-packages/laterite-node/test/peer-range.test.ts`
asserts the published range admits the version we actually build against —
offline, no registry. A duckdb minor bump means updating both together; that is
inherent to a dependency that never ships a final release.

**DuckDB-in-Node reality (P3):** the modern `@duckdb/node-api` ("Neo")
client has **no *built-in* Arrow bridge** — no `register`; results come
back as JS rows/cols, not Arrow. (The legacy `duckdb` package has a
built-in Arrow API but **segfaults** on current Node + ships high-severity
vulns.) So each group's born-typed arrow-js Table is ingested via a typed
`CREATE TABLE` (from `meta.sqlTypes`) + the **Appender** — type-faithful
(typed nulls, timestamps, booleans), zero-network, no temp files. `sql()`/
`at()` return **plain JS row objects** by default (`getRowObjectsJS`), are
**async** (Neo is promise-based), and lazy-import the optional peer (absent
→ a helpful install error). `table()` stays arrow-js columnar.

DuckDB's `arrow` **community** extension *does* provide the bridge
(`read_arrow` in, `to_arrow_ipc` out — `INSTALL arrow FROM community`, not
the core repo). But it's **network-fetched on first use** (breaks
offline / air-gapped / fresh-CI), and input via it still needs temp files
or pointers JS can't supply — so the Appender stays the default. Arrow-js
*output* is exposed as **opt-in**: `sql(q, { arrow: true })` (and
`at(…).table/frames({ arrow: true })`) lazy-loads the extension and decodes
`to_arrow_ipc` blobs into an arrow-js `Table`; without the flag, nothing
touches the network.

## Build & test

Built with **`@napi-rs/cli` 3** (on napi-rs 3 since PR #315), NOT plain `cargo
build` (the cdylib links Node's symbols at load time, like the PyO3 cdylibs):
from `rust-packages/laterite-node`, `./node_modules/.bin/napi build --platform
--release`. Bumping the CLI requires bumping the Rust `napi`/`napi-derive`
crates in **lockstep** — CLI 3 can't read napi-derive 2.x's type-def metadata,
so a mismatch emits an *empty* `index.d.ts` (every `./native` type silently
becomes `any`). The `fmt`/`clippy` cargo gates **do** compile the crate (kept
fmt + clippy clean), but it is **excluded from the `coverage` job**: under
napi 3 its standalone lib-test can't even link (napi's `Error::drop`
references N-API symbols the Node host only provides at load — see
ci-and-runners → Coverage gate). TS: `npm test`
(vitest) + `npm run typecheck` (`tsc --noEmit`) + `npm run smoke:pack` (the
**packaging smoke**: `npm pack`, then `read`/`table`/`validate`/`emitAgs4`
against the *packed* dist + native binary through **both** the CJS and ESM
entries — the artifact `npm install laterite` actually ships). The CI `node`
job (`ci.yml`, paths-filtered) runs all four on the linux self-hosted runner.

The napi loader `index.js` + types `index.d.ts` are **committed**, not just
gitignored build output: the `npm-publish` job consumes prebuilt `.node`s and
never runs `napi build`, so it needs them in the checkout (and the tarball's
`files` ships them). A **drift guard** keeps them honest — the `node` job
re-runs `napi build` then `git diff --exit-code` (they regenerate
byte-identically), so a `#[napi]`-surface edit can't ship a stale loader/types.

## Distribution (P4)

Ships as **`laterite`** on npm (unscoped); the native binary ships as three
scoped platform packages `@laterite/native-{darwin-arm64,linux-x64-gnu,
win32-x64-msvc}` (set via napi `packageName`), auto-selected through
`optionalDependencies`. The TS layer is a **dual ESM+CJS** bundle (`tsup` →
`dist/index.{mjs,cjs}` + `.d.ts`), and the napi loader is reached through a Node
subpath-import (`#native`, the package.json `imports` field) so the seam
survives bundling. `release.yml`'s `build-node` (a 3-runner matrix reusing the
wheels/binaries runners) + `npm-publish` ride their **own** tag namespace
`node-v*` — fully decoupled from the Python `v*` tags — gated like
`pypi-publish` (final tags only, no `rc`/`dev`; the `npm` environment + the
`NPM_TOKEN` secret for the first release, then npm trusted publishers). The
owner npm setup + release flow are in `RELEASING-node.md`.

## Status

**Complete (P1–P4), and long past first publish.** The npm package is on
**0.10.0** as of 2026-08-03 — it tracks the shared PRODUCT number, so
`npm i laterite@X` and `pip install laterite==X` are the same release (see
[[dec-rust-api-crates-io]] for the engine/product split). Its own coverage floors
are 98 lines / 91 branches, at 95.4% on the strict measure.

The phase history below is kept because the first-publish gotchas it records are
still the ones that bite. P1+P2 merged (PR #120); P3 merged (PR #121: DuckDB
`sql`/`at` Appender + row output, `agsTypes`, the 174 typed-graph classes +
`registry` + the typed-tree→`emitAgs4` bridge, `transport`, opt-in arrow-js
output via the `arrow` community extension's `to_arrow_ipc`); **P4** = the npm
package + platform packages + the `node` CI job + the `build-node`/`npm-publish`
release jobs. **`0.1.0` shipped to npm 2026-06-15** — `laterite` + the three
`@laterite/native-*` platform packages, provenance-signed (Sigstore); a real
`npm install laterite` reads + validates against the published binary. The
first-publish shakedown surfaced three gotchas, each its own re-cut and all now
guarded/documented (`RELEASING-node.md`): (1) the napi loader/types had to be
**committed** (the publish job never runs `napi build`) → drift guard + packaging
smoke (see Build & test); (2) **`EOTP`** — `NPM_TOKEN` must be a classic
**Automation** token to bypass account 2FA (granular/publish tokens don't); (3)
**`E402`** — scoped packages default to private, fixed with
`publishConfig.access=public` on the root package.json (propagated into the
platform packages by `napi create-npm-dir`). Born out of the [[ags4-output]]
Arrow-IPC producer work.

## Related

[[crate-map]] · [[laterite-py]] · [[laterite-ags4-wasm]] · [[laterite-ags4-validator]] · [[laterite-ags4-types]] · `laterite-ags4-emit` · [[pyo3-boundary]] · [[dec-rust-drives-python]] · [[ags4-output]] · [[dec-ags4-merge-semantics]] · [[edition-resolution]] · [[laterite-ags4-reference]] · [[surface-census]] · [[data-single-source-audit]] · [[dec-ags-idx-certificate]] · dec-rust-engine-staged-adoption
