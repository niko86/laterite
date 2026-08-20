---
type: concept
title: "tech stack: the browser wasm path"
status: drafted
tags: [concept, architecture, wasm]
ags_editions: []
volatile: [sizes]
volatile_asof: 2026-08-16
repo_refs:
  wasm: "repo:rust-packages/laterite-ags4-wasm/src"
  manifest: "repo:rust-packages/laterite-ags4-wasm/Cargo.toml"
  deploy: "repo:.github/workflows/deploy-validator.yml"
  release: "repo:.github/workflows/release.yml"
  slim_gate: "repo:tools/release/check-wasm-slim.mjs"
  tier1_gate: "repo:tools/release/check-wasm-tier1.mjs"
  artifact_gate: "repo:tools/release/wasm-artifact-gate.mjs"
  web: "repo:web/"
  gitignore: "repo:.gitignore"
  types_leaf: "repo:rust-packages/laterite-ags4-types/src/lib.rs"
related: [crate-map, laterite-ags4-tokenizer-wasm, validator-site, playwright-e2e, dec-laterite-ags4-types-leaf, dec-rust-drives-python, dec-duckdb-per-host-engine, laterite-ags4-types, ags4-output, DT, 0DP, YN, surface-census, data-single-source-audit]
sources: []
---
# tech stack: the browser wasm path

> [!note] Two browser wasm modules, not one
> This page describes the **engine** wasm (`laterite-ags4-wasm`, 5.1 MiB raw /
> 1.71 MiB gzipped for the full build — re-measured 2026-08-16, post-#336,
> post-#330 and post-#342; [[dec-engine-tiering]] splits what the browser
> actually loads into four tiers, of which this full build is the heaviest) —
> `validate`/`parse`/`diff`/`merge`/`to_ags4`, run in a Web Worker. Since
> #533 (part of the #527 convergence arc) the browser also loads a SEPARATE,
> deliberately tiny sibling — `laterite-ags4-tokenizer-wasm` (~30 KB / ~13 KB
> gzipped) — instantiated once on the **main thread** at boot, purely for the
> inline line editor/preview's synchronous tokenize/quote calls
> (`web/src/lib/tokenizer.ts`). It shares no code with this crate; both
> depend on `laterite-ags4-types`, and the tokenizer additionally depends on
> `laterite-ags4-parse`. See [[crate-map]] and [[dec-laterite-ags4-types-leaf]]
> for the full picture — this page's build/worker/Arrow-IPC detail is about
> the engine wasm only.

## Definition

The **fully client-side** AGS4 path: file bytes never leave the browser.
A dropped AGS4 file is handed to the `laterite-ags4-wasm` crate — a
`wasm32-unknown-unknown` cdylib built by `wasm-pack build … --target web`
(`repo:.github/workflows/deploy-validator.yml`) and loaded by the SolidJS
app in `repo:web/`. The release build's `wasm-opt` pass needs
`wasm-opt = ['-O', '-all']` in the crate's
`[package.metadata.wasm-pack.profile.release]`
(`repo:rust-packages/laterite-ags4-wasm/Cargo.toml`): modern Rust/LLVM emits
`bulk-memory` + `nontrapping-fptoint` by default, which the older binaryen
wasm-pack downloads rejects unless every feature is enabled.

> [!important] The generated bindings are **gitignored** — regenerate on a
> fresh clone. `wasm-pack` writes the JS glue, the `.d.ts`, and `_bg.wasm`
> into `web/src/wasm/`, which `.gitignore` excludes (the directory
> also carries its own `*` ignore). They are **never committed, only
> built** — so a fresh checkout has *no* bindings and `web/` won't
> typecheck, build, or run until they're regenerated:
> `wasm-pack build rust-packages/laterite-ags4-wasm --target web --release --out-dir web/src/wasm`
> (the exact command CI's `deploy-validator.yml` runs; do it once locally
> after cloning). **Corollary:** a Rust-side `validate`/`parse` signature
> change is invisible to `web/` — and to the `.d.ts` `tsc` checks against —
> until the bindings are rebuilt. The Rust crate is the source of truth;
> the checked-out `.d.ts` can be stale or absent (this is exactly why
> `compute_fixes`/`apply_fixes` — live in the crate — were once missing from
> an old checked-in `.d.ts` until it was regenerated).

The crate exposes these `#[wasm_bindgen]` entry points — **ten** in every
build, plus seven more behind the features described below
(one module per verb under `repo:rust-packages/laterite-ags4-wasm/src`):

- `validate(bytes, dict_version, include_fyi, encoding, max_per_rule)` → a
  `ValidationReport` JS object — the whole clean-room rule engine run
  in-browser with `source = None` (no filesystem). `max_per_rule` caps how
  many findings per rule are **serialized** (every rule still runs over
  every line); `finding_count`/`RuleGroup.total` keep the true totals and
  `shown_count` reports what crossed the boundary. This is a UI safety
  ceiling — a pathologically dirty file can yield millions of findings
  (hundreds of MB of JSON) — and is **wasm-only**: the CLI's `--json` is
  built independently in `repo:rust-packages/laterite-cli/src/main.rs`.
- `parse(bytes, encoding)` → a `ParsedDataset` exposing `group_codes()`,
  `meta(code)` (`{headings, units, types, sql_types}`), and
  `arrow_ipc(code)` — one **typed Apache Arrow IPC stream per group**,
  built lazily so peak residency is a single batch.
- `compute_fixes(bytes, dict_version, encoding)` → the safe/risky `Fix[]` for
  a file (the Fix tab's oracle; a **separate** surface from the findings — see
  [[tools/laterite-ags4-wasm]]).
- `apply_fixes(bytes, encoding, fixes)` → `Result<Vec<u8>, JsError>` (the patched
  file as **UTF-8 bytes**; thrown as a JS exception on failure since 2026-07-14 —
  it used to be infallible). Decodes with the given encoding then re-encodes UTF-8,
  so applying a fix to a cp1252 file also normalises its encoding.
- `diff(a_bytes, b_bytes, { encoding, maxRowsPerGroup })` → a type-aware,
  KEY-matched `RevisionDelta` between two files (the Tools revision-diff),
  engine-consistent because it parses both to the typed graph via [[laterite-ags4-types]].
  The dictionary edition is derived **internally** from the new (B) file's
  `TRAN_AGS` (not an option); `maxRowsPerGroup` caps how many per-row
  deltas are serialized per group (the counts stay true totals).
- `to_ags4(groups_json, edition, mode)` → `{ text, findings, fixes_applied }` —
  the **AGS4 *producer*** (the read path reversed): a JSON array of
  `{code, headings, units?, types?, rows}` → valid AGS4 `text` (UTF-8, CRLF) for
  a `Blob` download, via the shared `laterite-ags4-emit` orchestrator (dict UNIT/TYPE
  fill + AutoFix). The browser/offline half of the data→AGS4 feature
  ([[ags4-output]]).
- `to_ags4_ipc(groups, edition, mode)` → same `{ text, findings, fixes_applied }`,
  but **columnar**: `groups` is a JS array of `{code, ipc: Uint8Array}` Arrow IPC
  streams (e.g. a duckdb-wasm result), decoded via `StreamReader` and the *same*
  shared `laterite-ags4-emit` Arrow→Value transpose the native host uses — the read path's
  per-group Arrow IPC, reversed. Avoids the per-cell JSON round-trip for large data.

> [!bug] Until 2026-07-14 every `encoding`-taking export above resolved an
> unrecognised label by silently falling back to UTF-8
> (`resolve_encoding(label).unwrap_or(UTF_8)`) — a corruption vector, not
> leniency: the bytes `C3 A9` decode cleanly as `é` in UTF-8 and as `Ã©` in
> cp1252, so a caller who mistyped a label got the wrong text and a clean bill
> of health (`apply_fixes` would then rewrite the file from that mis-decode).
> Python raised on the same input; the crate's own test suite *codified* the
> asymmetry ("an unknown label falls back to UTF-8, not an error") rather than
> catching it. Now every export raises instead (`validate`/`parse`/`diff`/
> `merge`/`read`/`certify`/`apply_fixes` throw a `JsError`; `compute_fixes`, which
> has no error channel, returns no fixes) — matching Python. The browser's own
> encoding `<select>` only offers UTF-8/Windows-1252 (a closed union), so the UI
> itself could never trigger the old fallback, but a direct wasm caller could.
> See [[data-single-source-audit]] and [[surface-census]]'s `encodings` table.

## Three shapes of the same crate (#330, #352)

Since #330 the crate is **feature-gated**, and the artifact the browser fetches
depends on who built it. Six features — `excel`, `arrow`, `certify`, `diff`,
`merge`, `censor` — sit behind `default = full`
(`repo:rust-packages/laterite-ags4-wasm/Cargo.toml`).

`repo:.github/workflows/release.yml` builds `--no-default-features` instead, so
**npm's `@laterite/ags4-wasm` is the slim artifact**: validate, read, build, fix,
rules, dictionary, versions.

A **third** shape exists since #352: **tier 1**, the engine minus `arrow` and
`excel`, built by `npm run build:wasm` in `repo:web/package.json` — the one place
that feature list is written down, so CI and the deploy run that script rather
than spelling the flags out a second time.

Since #355 the app builds **two** of these three, and that is what
`repo:.github/workflows/deploy-validator.yml` now does: tier 1 into
`web/src/wasm`, and the full engine into `web/src/wasm-full` under `--out-name
ags4_wasm_full`. Tier 1 is precached and answers Validate, Fix, Export and every
tool but Excel; the full build is fetched only when Explore or Tools → Excel is
opened. The distinct out-name is load-bearing rather than tidy — see
[[dec-engine-tiering]] for what two artifacts sharing a fingerprinted stem would
do to the precache glob.

Measured 2026-08-16, same toolchain, wasm-bindgen pinned 0.2.125,
`wasm-opt = ['-O', '-all']`:

| build | raw | gzip -9 | brotli -q 11 | 
|---|---:|---:|---:|
| full (`default`) | 5,314,319 | 1,798,129 | 1,301,063 |
| tier 1 (`--features certify,diff,merge,censor`) | 2,144,163 | 848,292 | 645,503 |
| slim (`--no-default-features`) | 1,913,737 | 767,370 | 587,212 |

**Name the instrument or the number is unreproducible.** The gzip column is the
`gzip -9` binary; `repo:tools/release/check-wasm-slim.mjs` uses node's `zlib` at
the same level and reads ~8 KB HIGHER on the same bytes, which is why its ceiling
is stated in its own units rather than these. Brotli at `-q 11` is the artifact's
floor, not what a CDN serves — Cloudflare's is ~10% under gzip, so treat the
gzip column as the honest delivery figure and this one as a lower bound.

Two things about that split are easy to get wrong:

- **It is not a subset relation.** `read()` survives, but its Arrow door
  (`arrow_ipc`) does not — the slim build reads a group through `rows_json()`
  instead, same `parse_value` cast, JSON framing. A consumer holding `arrow_ipc`
  is not carried across. Tier 1 sits on the same side of that line: it drops
  `arrow` too, so it is the four cheap gates added to slim, not a smaller full.
- **No single byte-count axis judges it.** Each shape has its own gate —
  `repo:tools/release/check-wasm-slim.mjs` for npm's, and
  `repo:tools/release/check-wasm-tier1.mjs` for the browser's — over one shared
  instrument, `repo:tools/release/wasm-artifact-gate.mjs`. Each holds an exact
  check of the exported surface, down to class MEMBERS (`arrow`'s door into the
  build is `ParsedDataset::arrow_ipc`, which a top-level-only check cannot see),
  plus **two** ceilings, gzip and raw, and the three catch different failures.
  A flipped feature shows up in the export list and essentially nowhere else —
  `certify`, `diff`, `censor` and `merge` cost +14 to +36 KiB gzipped each, which
  no ceiling with honest headroom would fire on. Gzip catches what a client pays. Raw catches what compresses
  away: #342's duplicate dictionary was +1375 KiB raw against +98 KiB gzipped,
  and would have cleared a gzip-only ceiling by five kilobytes. (Same axis
  disagreement as #330's rejected `wasm-opt -Oz`: −360 KB raw, +1.2 KB brotli —
  which is why neither axis is allowed to be the only instrument.)

  This corrects an earlier reading of the same gate. Before #342, turning `diff`
  on appeared to cost 1.47 MB raw against 82 KB brotli, and that lopsidedness was
  taken as proof that raw was simply the wrong axis. It was not `diff`: it was the
  duplicate dictionary `diff` dragged in. With the duplicate gone `diff` costs
  55 KB raw against 17 KB brotli — an ordinary code ratio.

`opt-level = "z"` and `"s"` were both measured on #330 and both **rejected** —
13–95% slower in the browser for 164–400 KB. Do not re-propose them.

Phase 3 (Explore + Fix + Tools) is **delivered** (PR series #27–#38) — the
explorer (Arrow IPC → DuckDB-wasm + ECharts + an Analyse view), the Fix tab,
and the Tools suite (incl. a second wasm export `diff()` and a proj4
coordinate tool — now with an opt-in **OSTN15** sub-metre mode, via a
committed BSD-licensed NTv2 grid, plus GeoJSON export). See [[validator-site]]
for the phase table + the OSTN15 note (linked, not duplicated here).

## Why it matters

The casting in `parse()` shares the [[laterite-ags4-types]] leaf crate, so the
browser casts a file **identically** to a native `.ags5db` — parity by
construction, not a second implementation that can drift
([[dec-laterite-ags4-types-leaf]]). Arrow type mapping (off the file's TYPE row):
[[DT]] → `Timestamp(µs)` (tz-naive), [[0DP]] → `Int64`,
`2DP/RL/nSF/nSCI` → `Float64`, [[YN]] → `Boolean`, `ID/X/PA/…` → `Utf8`.

`getrandom` is a **host** proc-macro (`const-random` bakes ahash's seed
at compile time via `arrow`); it never runs in the wasm runtime, so no
`js`-feature workaround is needed.

The module is instantiated and called **inside a dedicated Web Worker**
(`repo:web/src/lib/validator.worker.ts`, since Phase 1.5), not on the main
thread — `validate()` is synchronous and can churn for tens of seconds on
a dirty file, so running it off-thread is what keeps the UI from freezing.
The main thread holds no wasm; it talks to the worker through an
id-correlated client. The "Download full report" path re-runs `validate`
uncapped *in the worker* and gzips the JSON there (streaming
`CompressionStream`), transferring back compressed bytes so the big string
never reaches the main thread. See [[validator-site]] Phase 1.5.

Since #354 there are **two** such workers, and since #355 they run **different
engines**. The one above is always on, runs tier 1, and serves Validate, Fix,
Export and every tool but Excel; a second (`repo:web/src/lib/tier2.worker.ts`) is
created only when Explore or Tools → Excel is opened, runs the full engine, and
owns the `ParsedDataset` handle — the app's only stateful wasm handle, now in the
worker of its sole consumer.

The split is enforced at compile time, not by convention: `EngineApi` in
`repo:web/src/lib/engineDispatch.ts` is the full build's type with `read`,
`ags4_to_xlsx` and `xlsx_to_ags4` marked optional, and `validator.worker.ts`
names the ops tier 1 serves one by one rather than handing over its whole
module. Add one of the other four there and it stops typechecking, naming
`ParsedDataset.arrow_ipc` as what tier 1 does not have.

> [!note] DuckDB-wasm asset loading (resolved)
> DuckDB-wasm + ECharts + proj4 **lazy-load on their views only** (confirmed:
> separate chunks; the entry chunk stays ~150 kB). The EH DuckDB wasm is
> ~36 MB raw / ~8 MB gzipped, fetched + instantiated on first *Explore* use
> (a few seconds on mobile), then HTTP-cached. The mvp/eh workers + wasm are
> shipped via Vite `?url` so they're fingerprinted + base-path-correct under
> `/laterite/` (an `import.meta.url` fetch would 404 under a non-root
> base). **Known-benign 404 (fixed at source):** the prebuilt DuckDB worker
> carries a `//# sourceMappingURL=…worker.js.map` comment but we never emit
> the .map, so the browser logged a harmless 404 for it (noticed first on
> iOS). A build-only Vite plugin (`stripDuckdbWorkerSourcemaps` in
> `repo:web/vite.config.ts`) strips that comment from the emitted worker
> assets. Source maps for a third-party prebuilt worker aren't useful anyway.

> [!note] DuckDB-wasm is the browser's per-host DuckDB — read **and** write
> `duckdb-wasm` is **not** a special case: it plays the same role as pip-`duckdb`
> (Python) and `@duckdb/node-api` (node) in the one cross-surface model
> ([[dec-duckdb-per-host-engine]]). The wasm crate stays DuckDB-free and emits typed
> Arrow IPC (`arrow_ipc(code)`); `duckdb-wasm` ingests it to **query** *and* to
> **persist/materialise** — an OPFS-backed database or a `COPY`/`EXPORT` to a
> downloadable file. So "materialise AGS4 → a queryable `.duckdb`" works in the
> browser too, via `duckdb-wasm`. The genuinely native-only pieces are the
> *Rust-side* persistence accelerators (the `.ags.idx` byte-index sidecar, the parse
> cache), which need Rust filesystem access wasm-Rust doesn't have
> ([[dec-duckdb-perf-architecture]]) — not the materialise itself.

> [!todo] Deferred E2E parity
> End-to-end value parity against a *live* native `.ags5db` was not run.
> The "identical to `.ags5db`" claim rests on both sides calling the one
> shared [[laterite-ags4-types]] crate; host unit tests prove the casting logic.
> Tracked in [[validator-site]].

## Diagram

```mermaid
graph LR
  file["AGS4 file<br/>(browser, never uploaded)"]
  wasm["laterite-ags4-wasm<br/>wasm32 cdylib"]
  rep["ValidationReport"]
  ipc["typed Arrow IPC<br/>per group"]
  duck["DuckDB-wasm + ECharts + Analyse<br/>(Phase 3 ✅, lazy)"]
  file --> wasm
  wasm -->|validate| rep
  wasm -->|parse / arrow_ipc| ipc
  ipc --> duck
```

## Where it shows up

`laterite-ags4-wasm` is the browser arm of the [[crate-map]] (validator → wasm
cdylib + the shared [[laterite-ags4-types]] leaf). The roadmap, phase status, and
verification notes live in [[validator-site]]; the single-typing-source
rationale is [[dec-laterite-ags4-types-leaf]].

## Related

[[crate-map]] · [[laterite-ags4-tokenizer-wasm]] · [[validator-site]] · ci-and-runners · [[dec-laterite-ags4-types-leaf]] · [[dec-rust-drives-python]] · [[dec-duckdb-per-host-engine]] · [[laterite-ags4-types]] · [[DT]] · [[0DP]] · [[YN]] · [[surface-census]] · [[data-single-source-audit]]
