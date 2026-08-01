# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

<!-- GENERATED FROM changelog.json BY tools/gen_changelog.py — DO NOT EDIT BY HAND -->

## [Unreleased]

### Added

- **The browser package publishes its remaining result types, and `any` is gone from its `.d.ts` entirely.** `diff`, `dictionary` and `compute_fixes` were the three still returning `any`; they now return `RevisionDelta`, `StandardDict` and `Fix[]`, joining the shapes published previously. Consumers can delete their hand-written `RevisionDelta` / `StandardDict` / `Fix` / `SpanEdit` / `CellDelta` / `RowDelta` / `GroupDelta` / `DictGroup` / `DictHeading` mirrors — the validator site deleted ~120 lines of exactly those. Each new interface is bound to the Rust struct that serialises it by the same drift test as the others, and `Fix`'s `kind`/`risk` unions are checked against the validator's own enums, so a new fix kind cannot ship a `.d.ts` that lies about what you can receive. ([#188](https://github.com/niko86/laterite/pull/188))
- **The fingerprint build refuses to emit a digest that covers too little.** It previously narrowed in silence — the dependency walk simply found nothing and carried on. It now fails the build, naming what it found. The existing coverage test cannot do this job: tests do not ship in a published crate, so the one build that had the defect was also the one build with nothing checking it. ([#190](https://github.com/niko86/laterite/pull/190))
- **Three gates that have to work before the first crates.io publish, not after.** A published version's public API and its declared `rust-version` are both frozen, and the only remedy for either being wrong is a version bump every consumer has to follow. The new `publish-gates` CI job (1) builds the ten engine crates on the `rust-version` their own manifests declare, (2) diffs each crate's rendered public API against a checked-in snapshot under `tools/release/public-api/`, and (3) runs `cargo semver-checks` against `main`. The snapshot is the one that changes review: it puts the whole public surface into the PR's own diff, including the `Send`/`Sync` impls that are public API nobody writes down — an `Rc` in a private field silently removes `Send` from a consumer's type, and that is now a build failure rather than something a reviewer has to notice. Opaque `-> impl Trait` returns are the one thing a snapshot cannot see, so each is asserted at compile time instead, and the gate refuses to let a new one be added without one. ([#193](https://github.com/niko86/laterite/pull/193))
- **The engine crates carry the metadata a crates.io page is made of.** `repository`, `keywords` and `categories` were absent, and seven of the eight round-one crates had no README. That is not a cosmetic gap: crates.io bakes metadata into the PUBLISHED VERSION, so adding it afterwards fixes future versions only — the pages for a version already on the registry would keep no link back to the source, permanently, and seven of them would have been a one-line description and nothing else. Each README is written to stand alone on the registry rather than assume the repository around it. The packaging gate caught all seven new files, which is what it is for. ([#195](https://github.com/niko86/laterite/pull/195))
- **The eight round-one engine crates are armed for crates.io, and there is a tool that publishes them safely.** Every crate carried `publish = false` — a blanket safety catch, so `cargo publish` refused outright. It is now `true` on the eight; `laterite-ags4-diff` and `laterite-ags4-merge` stay `false`, held for 0.2, with the reason recorded at the flag rather than somewhere a reader has to go looking. `tools/publish_crates.py` does the release: it derives the dependency waves from the manifests each run rather than restating them, waits for each wave to become RESOLVABLE from the registry (not merely uploaded — crates.io indexes on a delay, and a plain loop dies on wave 2 with wave 1 already permanently published), and is idempotent, so a re-run after a failure resumes instead of restarting. It refuses a dirty tree, any branch but `main`, a HEAD that is not `origin/main`, and any crate still marked `publish = false`. Nothing happens without `--execute`. ([#196](https://github.com/niko86/laterite/pull/196))
- **The engine crates are on crates.io.** Eight of them at 0.9.0 — `laterite-ags4-parse`, `-types`, `-reference`, `-core`, `-validator`, `-emit`, `-trust` and `laterite-transport` — so a Rust consumer no longer needs a git dependency or a vendored path. `laterite-ags4-diff` and `laterite-ags4-merge` are deliberately NOT published; they ship in 0.2, and the `publish = false` guard is what keeps that a decision rather than something to remember. `tools/publish_crates.py` gained `--through-wave N`, so a release can publish one wave, let the live pages be inspected, and only then commit the rest — worth the pause while metadata is still correctable on the crates that have not gone out. ([#197](https://github.com/niko86/laterite/pull/197))
- **`laterite` — a Rust API, not just a facade.** The user-facing crate now exists and is functional at 0.1.0: read an AGS4 file from a path or bytes, walk its groups/headings/rows, edit cells and append rows, validate against the full numbered rule set, and write valid AGS4 back out. It carries its own version, deliberately — the engine crates are lockstep 0.9.x and reshape as the format work demands; this one is a different promise to a different audience. The design's central rule is now **mechanically enforced** rather than reviewed for: no third-party type may appear anywhere in its public API, checked against the rendered surface, so a `serde_json::Value` return or an `encoding_rs::Encoding` argument fails the build instead of silently binding our semver to a dependency's. `encoding_rs` is not even a dependency — encodings are WHATWG label strings. Held for 0.2, additively: diff, merge, typed cell access, indexed scan. ([#198](https://github.com/niko86/laterite/pull/198))

### Changed

- **Breaking: `diff` and `censor` take an options object too.** They were the last two `@laterite/ags4-wasm` exports with a positional tail — `diff(a, b, encoding, maxRowsPerGroup)` and `censor(data, sensitiveJson, selectedCodes, token, dropCustom, includeFreetext)`, the latter six-deep with three consecutive booleans. Both now take one named object after their inputs (`diff(a, b, { encoding, maxRowsPerGroup })`, `censor(data, sensitiveJson, { selectedCodes, token, dropCustom, includeFreetext })`), every field optional and defaulted, with the same unknown-key refusal the other exports got. The test that recorded these two as "not yet migrated" now has nothing left to exempt. ([#188](https://github.com/niko86/laterite/pull/188))
- **Every engine crate now declares what it may publish.** `cargo package` ships whatever sits in a crate directory and is not excluded, and no crate here specified `include` — `laterite-ags4-validator` alone would have put 41 test and bench files plus 2.1 MB of reference data no code reads into its tarball. All ten engine-tier crates now carry an explicit allowlist, and a CI gate diffs `cargo package --list` against a checked-in manifest so a new file entering a tarball turns the build red. This had to land before the first publish rather than after: crates.io is append-only, and `yank` stops new resolution without removing the tarball. ([#189](https://github.com/niko86/laterite/pull/189))
- **The engine crates declare the versions a publish needs.** `cargo package` rejects a dependency with no version requirement — publishing strips the `path` and the version is all that survives to identify it — so 7 of the 10 engine-tier crates could not be packaged at all. The 21 in-workspace dependency sites now inherit from `[workspace.dependencies]`, which carries both a path and a version. Nothing changes locally: the path still wins, and `Cargo.lock` is untouched. The versions sit in `rust-packages/Cargo.toml` specifically so the release bump rewrites them alongside the workspace version rather than leaving them behind, and a test asserts that lockstep instead of assuming it. ([#191](https://github.com/niko86/laterite/pull/191))
- **The CLI crate is `laterite-cli`.** `laterite-ags4-check` named a checker; the crate reads, fixes, diffs, merges, certifies, packs, converts and mints certificates, and `-ags4-` marks the engine tier, which a product binary is not. The shipped binary is still `lat` — nothing a user types changes. Renamed now because crates.io has no rename: a name is free until its first publish and irreversible after, and the engine tier went out yesterday. It is deliberately NOT added to the publish set, which looks like an oversight and is not: a binary's dependencies are still dependencies, so publishing it requires `laterite-ags4-diff` and `-merge` (held for 0.2) plus `laterite-cliutil` and `laterite-excel` (never considered for the registry) — and both publish gates render a lib target that a bin-only crate does not have. The CLI ships with 0.2, alongside the diff/merge publish it already depends on. ([#199](https://github.com/niko86/laterite/pull/199))

### Removed

- **`GroupDescriptor::table()` / `view()` are gone** (again). The two DuckDB `g_<code>`/`v_<code>` name builders sat on the format-neutral reference-data leaf with no callers but their own unit test, and publishing that crate would have frozen an unrelated product's schema naming into its public API. They were deleted in #175 — but #178 branched before it and squash-merged after, so its diff restored them, and the closed issue made the reverted state read as the finished one. ([#189](https://github.com/niko86/laterite/pull/189))

### Fixed

- **The engine fingerprint no longer forgets its dependencies when built from a published crate.** A certificate records which engine judged a file, and the engine's identity is a hash over everything that can change a verdict — the rules, plus the crates they run through. `cargo publish` rewrites the packaged manifest and strips the `path` key from every dependency, and the hash found those crates BY that key, so a build from a crates.io tarball covered 14 files in 1 crate instead of 29 across 4, without a word. A consumer who picked up a new `laterite-ags4-parse` — the tokenizer that decides where fields end — would have kept trusting every certificate minted before it. Our own crates are now identified by `name@version` when their sources are not reachable, which is sound precisely there: a published version's content is immutable. The in-workspace fingerprint is unchanged, so no existing certificate is affected. ([#190](https://github.com/niko86/laterite/pull/190))
- **The engine crates build on the Rust version they promise.** Every crate declares `rust-version = "1.85"`, cargo only ever enforces that field on our dependencies and never on us, and so the promise had quietly been false: `laterite-ags4-emit` used a `let` chain, stable in 1.88. A consumer pinned to an older toolchain would have resolved to us precisely because the floor said they could, then failed to compile against our source. The one line is rewritten and the MSRV gate now builds the publish set on whatever floor the manifests declare — read from the manifests rather than restated, so raising the floor cannot leave the gate testing the old one. ([#193](https://github.com/niko86/laterite/pull/193))

## [0.9.0] — 2026-07-30

The browser API breaks: every `@laterite/ags4-wasm` export now takes an options object instead of a positional tail. And a synthesised `TRAN` is the caller's to supply — the placeholder the emitter used to invent satisfied Rule 14, so files could assert a transmission that never happened and pass validation doing it.

### Added

- **A transmission can carry a description and remarks.** `TranStamp` takes optional `description` and `remarks` (`TRAN_DESC` / `TRAN_REM`), emitted only when set. `TRAN_AGS`, `TRAN_DLIM` and `TRAN_RCON` remain derived and are deliberately not accepted from callers — they describe the syntax of the bytes the emitter is writing, so a supplied value could only contradict them. ([#178](https://github.com/niko86/laterite/pull/178))
- **The browser's `merge` accepts `dictVersion`**, so an edition can be forced from wasm as it always could from Python, Node and the CLI. This gap was found by the new cross-surface knob gate on its first run — the browser had simply never been held to the same option set as the other surfaces. ([#182](https://github.com/niko86/laterite/pull/182))
- **The browser package ships its own result types.** `validate`, `build_ags4`, `build_ags4_ipc`, `censor` and `ParsedDataset.meta` were typed `any` in the generated `.d.ts`, so every consumer had to re-describe the report shape by hand — a downstream browser consumer reported `build_ags4` returning `any` as a defect. The crate now publishes `ValidationReport`, `RuleGroup`, `FindingDto`, `ValErr`, `BuildReport`, `EmitFinding`, `AppliedFix`, `CensorResult`, `CensorTally` and `GroupMeta`, so **you can delete your copy**. Two tests bind the published interfaces to the Rust serde structs and the `AppliedFix` unions to the validator's enums, so they cannot drift into being prettier `any`. `diff`, `compute_fixes` and `dictionary` still return `any`: their shapes belong to other crates, and hand-mirroring them here would rebuild the very problem this removes. ([#185](https://github.com/niko86/laterite/pull/185))

### Changed

- **Breaking: every `@laterite/ags4-wasm` export takes an options object.** `validate`, `certify`, `build_ags4`, `build_ags4_ipc` and `merge` had grown positional tails of up to nine arguments, where `undefined, undefined, true` was an ordinary thing to write and slot order was the only thing keeping it correct. Each now takes one named object — `{ dictVersion, warnings, fyi, … }` — every field optional and defaulted to match the other surfaces. Named options introduce a mistake positional ones could not have (you cannot misspell slot 3), so **an unrecognised key is refused with a suggestion** rather than silently ignored: `unknown build option 'synthesizeMetadata' — did you mean 'synthesiseMetadata'?`. Call sites need rewriting; there is no positional fallback. ([#179](https://github.com/niko86/laterite/pull/179), [#180](https://github.com/niko86/laterite/pull/180), [#181](https://github.com/niko86/laterite/pull/181))
- **Breaking: a synthesised `TRAN` is stamped by the caller, never invented.** With metadata synthesis on, the emitter used to write `"TBC"` into `TRAN_PROD`/`TRAN_RECV`/`TRAN_STAT` and a fixed `1900-01-01` date — a combination that **satisfies Rule 14**. The file asserted a transmission that never happened, passed validation, and gave a recipient no way to tell it from a real transmission record. The five REQUIRED values are now supplied together or not at all: **Python and Node take a `TranStamp` object in place of the five `tran_*` arguments**, and the CLI's `--tran-*` flags are all-or-none (a partial set exits 2, naming what is missing). With no stamp, no `TRAN` is minted and Rule 14 reports the gap honestly. Note that **a clean file now genuinely needs all five values** — the placeholder had also been silencing Rule 10b on three REQUIRED headings, which is why dropping it surfaces findings that were always true. ([#178](https://github.com/niko86/laterite/pull/178))

### Fixed

- **Metadata synthesis could mint an empty `UNIT` group** — itself a Rule 2 error. The unconditional `TRAN`'s `"yyyy-mm-dd"` on `TRAN_DATE` had quietly been the only guarantee that any unit existed at all, so synthesis was trading a Rule 15 finding for a Rule 2 one and reporting success. `UNIT` is now skipped when nothing uses one. ([#178](https://github.com/niko86/laterite/pull/178))
- **The browser's `merge` no longer leaks its result.** The wasm `MergeResult` is freed once its fields have been read, rather than being left to the caller to remember. ([#181](https://github.com/niko86/laterite/pull/181))
- **The browser mis-classified every error as a warning.** `severity` is *absent* on an error — the engine writes the field only for `warning`/`fyi` — but the app's hand-written type declared `"error"` as a value it might receive, and five call sites resolved the absent case with `?? "warning"`. So the summary banner's split counted errors as warnings, the severity filter hid them from the **error** selection while showing them under **warning**, and a finding's highlight band rendered amber instead of red. Found by replacing that hand-written type with the engine's own, which stopped the app compiling in exactly the six places the assumption had spread to. All resolution now goes through one `severityOf` helper. ([#185](https://github.com/niko86/laterite/pull/185))

## [0.8.2] — 2026-07-30

No functional change on any surface. The browser package is republished from CI so it carries a build provenance attestation, which 0.8.1 could not.

### Changed

- **`@laterite/ags4-wasm` is published by CI with a build provenance attestation.** 0.8.1 was published by hand because npm cannot configure a trusted publisher for a package that does not exist yet, so it is the one version carrying no attestation. **The wasm bytes are unchanged from 0.8.1** — if you verify provenance in your supply chain, take 0.8.2; otherwise there is nothing here to upgrade for. ([#172](https://github.com/niko86/laterite/pull/172))

## [0.8.1] — 2026-07-29

The browser build ships as an npm package for the first time, carrying two things the other surfaces already had: reachable metadata synthesis, and a visible AutoFix ledger.

### Added

- **`@laterite/ags4-wasm` — the browser build, published.** The wasm surface ships to npm for the first time, from its own `wasm-v*` tag so a browser-only fix no longer has to move the Node package. The `.wasm` embeds the AGS4 dictionary, so the third-party notice rides in the package and a release gate verifies it inside the tarball npm would upload. ([#165](https://github.com/niko86/laterite/pull/165))
- **`applied` on the wasm build result.** `build_ags4` / `build_ags4_ipc` now return the ledger of what AutoFix rewrote — `{kind, label, rule, line, risk}` per fix — matching Python's `BuildResult.applied` and Node's `EmitResult.applied`. AutoFix returns only *residual* findings, so the previous count alone could say how many fixes ran but never which. ([#165](https://github.com/niko86/laterite/pull/165))

### Fixed

- **`synthesise_metadata` is reachable from the browser.** `build_ags4` and `build_ags4_ipc` took no such argument, so a wasm caller could not opt in to the mandatory `UNIT`/`TYPE`/`TRAN`/`ABBR` catalogs. The flag reached Python and Node when synthesis became opt-in, but missed the wasm door. ([#149](https://github.com/niko86/laterite/pull/149))
- **The produce/build docs described the pre-opt-in synthesis behaviour.** Three pages claimed `mode="autofix"` synthesises the metadata catalogs, and that `mode="strict"` is how you skip synthesis. Synthesis is opt-in and independent of mode; `strict` is a hard gate that rejects the build rather than emitting it. ([#165](https://github.com/niko86/laterite/pull/165))

## [0.8.0] — 2026-07-28

A performance pass on the read path (a typed read is now ~4× faster) and a new `to_duckdb()` persistence door, plus one breaking change: metadata synthesis is now opt-in.

### Added

- **`to_duckdb()` — persist a read as a keyed DuckDB database.** A read handle can be written straight to a `.duckdb` file with the content-addressed `_id` / `_parent_id` keys materialised, so cross-group joins survive a round-trip to disk. ([#107](https://github.com/niko86/laterite/pull/107))

### Changed

- **Metadata synthesis is now opt-in.** `build_ags4` and the emit path no longer synthesise the mandatory `TRAN` / `UNIT` / `TYPE` catalog groups a data-only build is missing — a data-only build now *reports* Rule 14/15/17 rather than silently filling them. **Breaking:** pass `synthesise_metadata=True` (Python) / `synthesiseMetadata: true` (Node) to restore the previous behaviour. Minting whole groups the caller never wrote is the kind of magic a caller should ask for. ([#85](https://github.com/niko86/laterite/pull/85))
- **The read path is substantially faster.** The typed read is built by a single bulk Arrow cast rather than per-row conversion (~75.9 ms → ~19.9 ms on the 25 MB baseline); the content-addressed keychain streams each key straight into a reused hasher and reads KEY cells positionally (dropping a per-row all-columns map); the Python read/validate compute releases the GIL; and a keys-less default read on Node skips the keychain entirely. ([#93](https://github.com/niko86/laterite/pull/93), [#101](https://github.com/niko86/laterite/pull/101), [#106](https://github.com/niko86/laterite/pull/106), [#108](https://github.com/niko86/laterite/pull/108), [#98](https://github.com/niko86/laterite/pull/98))

### Fixed

- **A duplicated heading is refused rather than silently returning the wrong column.** A group whose HEADING row repeats a name no longer resolves reads of that name to an arbitrary one of the duplicates. ([#88](https://github.com/niko86/laterite/pull/88))
- **`build_ags4` from a pandas frame no longer risks a heap corruption under the bundled allocator.** A use-after-free in the pyarrow C-stream path (upstream `arrow-rs#10439`) is worked around with a temporary guard until the fix lands. ([#123](https://github.com/niko86/laterite/pull/123))
- **The parser recovery flag is wired on all three launchers** (the native binary, `uvx` and the console-script) and covered by tests.

## [0.7.0] — 2026-07-08

The **`lat` CLI** — one AGS4 tool with three launchers (the native binary, `uvx --from laterite lat`, and `npx laterite`), the full verb set, and byte-identical scriptable output — plus a round of parser / emit / fix correctness fixes.

### Added

- **`lat read <file> [group]`** — a data-out verb: dump one group's rows as an aligned table (default), `--csv`, or `--json`, or list the file's group codes when no group is named. Raw file cells, faithful to the bytes; `--csv` / `--json` are byte-identical across the native binary, `uvx`, and the Python console-script. `--out <path>` writes to a file. ([#430](https://github.com/niko86/laterite/pull/430))
- **`lat pack` / `unpack` / `lock` / `unlock`** — transport verbs: zstd-compress any file, or add an age passphrase envelope (standard zstd-inside-age). The passphrase is never a flag — precedence is `--password-file` → `$LAT_TRANSPORT_PASSWORD` → an interactive prompt. `--level` tunes the zstd ratio and `--log-n` the scrypt tier (default 18). `--no-default-features` builds a lean, age/zstd-free tool. ([#430](https://github.com/niko86/laterite/pull/430))
- **`lat excel <in> <out>`** — convert AGS4 ↔ Excel. The direction is inferred from the output extension (`.xlsx` ⇒ export, one sheet per group; `.ags` ⇒ import) or forced with `--export` / `--import`; `--no-format-numeric` keeps imported columns as text. With this the `lat` verb set is complete. ([#430](https://github.com/niko86/laterite/pull/430))
- **`lat` on npm — `npx laterite`** — the Node CLI, the third launcher of the one AGS4 tool. All the same verbs (validate / read / fix / diff / certify / rules / transport / excel) over the public API; scriptable outputs are byte-identical to the binary. Ships as a `bin` on the `laterite` npm package. ([#430](https://github.com/niko86/laterite/pull/430))
- **Unified `.ags.idx` certificate identity across surfaces.** A certificate minted by any surface (the `lat` CLI, the wheel, Node, the DuckDB extension) now carries one shared engine identity, so a cert minted by one is trusted by all. ([#430](https://github.com/niko86/laterite/pull/430))
- **`log_n` scrypt work factor on `lock` / `unlock`** (library + the CLI `--log-n`; default 18) — an optional tuning knob, pinned on decrypt so files open on any machine. ([#432](https://github.com/niko86/laterite/pull/432))

### Changed

- **The CLI binary and Python console-script are renamed `lat-check` → `lat`.** The tool grew well beyond "check" (read / transport / excel / diff / certify). **Breaking:** scripts invoking `lat-check` must switch to `lat`. ([#430](https://github.com/niko86/laterite/pull/430))
- **`fix()` canonicalises unambiguous dates by default.** Date normalisation is assessed per value — an unambiguous date is a safe fix, where before it needed `risky=True`.

### Fixed

- **Parser: old-Mac (lone `CR`) line endings split correctly.** The line splitter is now quote-aware and universal-newline, so a bare `\r` is a break outside quotes and a literal inside them. ([#422](https://github.com/niko86/laterite/pull/422))
- **Emit: a cell carrying a raw `CR` / `LF` is rejected**, not written into an illegal AGS4 file. ([#423](https://github.com/niko86/laterite/pull/423))
- **Emit: the fix no-op path always produces valid UTF-8**, even when the source carried a UTF-8 label.
- **Fix: quotes produced by Rule 1 transliteration are AGS-escaped**, so a transliterated value stays a valid quoted field.
- **Encoding: one shared encoding-label resolver across all surfaces** — the same WHATWG label maps to the same encoding everywhere.
- **`registry.GROUPS` is sealed read-only** — mutating the shared registry mapping now raises instead of corrupting it for later calls.

## [0.6.2] — 2026-07-05

A round of cross-surface I/O-form additions from the modality audit — every capability offered in the same *forms* (path / bytes / text) on every surface it makes sense on. All additive; nothing existing changes.

### Added

- **In-memory (bytes) transport — `pack_bytes` / `unpack_bytes` / `lock_bytes` / `unlock_bytes`.** The filesystem-free counterparts, so you can package a value you already hold in memory straight to an upload — and `lock_bytes` seals sensitive data without ever writing the plaintext to disk. Same envelopes as the file forms (standard zstd / age). ([#389](https://github.com/niko86/laterite/pull/389))
- **Node in-memory transport — `packBytes` / `unpackBytes` / `lockBytes` / `unlockBytes`.** The Node mirror of the bytes API, over the same shared transport leaf; a blob sealed in memory interops with the file `unpack` / `unlock` and with `pyrage` / the browser. ([#389](https://github.com/niko86/laterite/pull/389))
- **In-memory certificate output — `certify_bytes()` (Python) / `certifyBytes()` (Node).** Mint an `.ags.idx` and get its bytes back instead of a file, so a web backend can hand the cert straight to an upload with no temp-file round-trip. The bytes interop with `read(index=…)`, the CLI `--index`, and the browser. ([#390](https://github.com/niko86/laterite/pull/390))
- **Excel bytes ↔ bytes — Python `to_excel(output=None)` / `from_excel(bytes)`, Node `toExcel` / `fromExcel` bytes forms.** AGS4 ↔ `.xlsx` now round-trips entirely in memory on the library surfaces, so an uploaded `.xlsx` needn't hit disk. ([#391](https://github.com/niko86/laterite/pull/391))
- **`lat-check --index <file>.ags.idx` — consume a certificate.** The CLI could *mint* an `.ags.idx`; now it can *consume* one — a fresh, same-engine, profile-covering certificate skips the rule engine and reports the certified verdict (a stale / foreign / insufficient cert is re-validated). ([#393](https://github.com/niko86/laterite/pull/393))
- **Node `fix()` write-back + per-rule selection.** `fix()` gains `inPlace` / `out` and `only` / `exclude` (restrict which fixable rules apply, typed by a new `FixableRule` union), matching Python's free `fix`. ([#394](https://github.com/niko86/laterite/pull/394))
- **DuckDB `validate_ags_text` / `certify_ags_text` + `encoding` on `certify_ags`.** The content (VARCHAR) twins of `validate_ags` / `certify_ags` — validate and certify AGS4 that isn't a file you can hand the VFS; `certify_ags_text` returns the certificate JSON in a `cert` column. ([#392](https://github.com/niko86/laterite/pull/392))

## [0.6.1] — 2026-07-04

### Fixed

- **The risky Rule-1 fix now transliterates *all* non-ASCII characters, not just typography.** `fix(risky=True)` folds `µ→u`, `°→deg`, `ß→ss`, accents→base letter (via `deunicode`), and maps the un-representable — including the `U+FFFD` replacement character that marks mojibake — to `?`. So a file whose only defect is non-ASCII content can be folded Rule-1-clean and then certified. Applies on every surface. Still opt-in (risky), since transliteration is a lossy guess.

## [0.6.0] — 2026-07-04

0.6.0 completes the cross-surface story. The browser and Node now reach every capability except the `python-ags4` compat shim: Node and browser Excel I/O, in-browser certificate minting, and browser transport (zstd + age) all land here, alongside finer fix-control across Python, Node and the CLI. One breaking change — the typed-graph classes moved to `laterite.groups` — is what bumps the minor pre-1.0. The npm package unifies onto this single version (the separate `node-v*` number is retired), and a new cross-surface drift-gate (`tests/test_version_faithful.py`) keeps every surface's version in lockstep.

### Added

- **Node Excel I/O — `toExcel()` / `fromExcel()`.** The Node addon binds the shared `laterite-excel` engine, so AGS4 ↔ `.xlsx` round-trips from Node the same way Python and the browser do — one sheet per group, born-typed columns, an `ExcelStats` summary. ([#361](https://github.com/niko86/laterite/pull/361))
- **Browser Excel — AGS4 ↔ `.xlsx`, client-side.** A two-way Excel converter running entirely in the browser (wasm) — no upload. Backed by a new FS-free bytes API in `laterite-excel`, so one engine drives Python, Node and the browser. ([#367](https://github.com/niko86/laterite/pull/367), [#368](https://github.com/niko86/laterite/pull/368))
- **Browser certificate minting — "Download certificate" in Validate.** A clean validation in the web app offers the `.ags.idx` certificate as a download, minted client-side by a new wasm `certify()` — byte-compatible with the certificates the other surfaces produce. ([#363](https://github.com/niko86/laterite/pull/363), [#364](https://github.com/niko86/laterite/pull/364))
- **Browser transport — lock / unlock in Tools.** The web app seals and opens files client-side: zstd inside a passphrase-based `age` envelope, byte-compatible with `laterite.transport.lock`, the Node `transport` namespace and `pyrage`. ([#369](https://github.com/niko86/laterite/pull/369), [#370](https://github.com/niko86/laterite/pull/370))
- **Anonymiser upgrade — pseudonymise, don't just blank.** Per-category actions: location and sample IDs can be pseudonymised into stable, reference-preserving tokens (a cross-group join still works after scrubbing), `PROJ_ID` maps to a content hash, coordinates blank, and a preset applies a common policy in one click. ([#371](https://github.com/niko86/laterite/pull/371))
- **Fix control — per-rule selection + a discoverable fixable-rule catalogue.** `fix()` takes `rules=` to apply only chosen repairs, and `list_rules()` / `fixable_rules()` expose which rules the engine can mechanically fix (a generated `FixableRule` `Literal`, drift-gated against the engine). On Python, Node and the CLI. ([#297](https://github.com/niko86/laterite/pull/297))
- **"N more fixable with `risky=True`" signal.** A `FixResult` (and the CLI `--fix` summary) reports how many further findings the risky tier could repair beyond the safe pass — so you know a stronger fix exists without running it blind. ([#298](https://github.com/niko86/laterite/pull/298))
- **`Literal` types for enumerated string parameters** (`mode`, `xn`, `backend`, encoding labels, …) — so an IDE completes the valid values and a typo is a type error rather than a runtime one. ([#299](https://github.com/niko86/laterite/pull/299))
- **`read()` remembers its encoding for the chained verbs.** A handle retains the encoding it was read with, so chained `.validate()` / `.fix()` / `.diff()` re-run against the true bytes with matching line numbers — no need to re-pass `encoding=`. ([#300](https://github.com/niko86/laterite/pull/300))
- **Node certificate lifecycle — `Ags4File.certify()` + `read(f, { index })`.** Node gains the `.ags.idx` validity certificate: a clean `.validate()` then `certify()` mints it beside the file (refusing to overwrite the source), and `read(f, { index })` consumes it, freshness-checking size + SHA-256. A Node-minted cert is byte- and checker-compatible with Python / the CLI / the DuckDB extension. ([#294](https://github.com/niko86/laterite/pull/294))
- **Node fluent chained layer — `Ags4File.validate()` / `.fix()` / `.diff()`.** So `read(p).fix().validate()` reads as one chain (Python-parity); `.fix()` returns a NEW repaired `Ags4File` (non-destructive), and a handle retains its read source so the chained verbs re-run against the true bytes. ([#294](https://github.com/niko86/laterite/pull/294))
- **Node revision diff — `diff(a, b)`.** The Node port of `laterite.diff()`, over the same shared engine, so the `RevisionDelta` is byte-identical across surfaces. Rows are matched by the group's dictionary KEY headings (not line order) and cells compared through the typed value. ([#294](https://github.com/niko86/laterite/pull/294))
- **`read_typed` reads text and bytes, not just a path** — the same inputs as `read()`, plus an `encoding=` label for bytes / path input. It was path-and-UTF-8-only before. ([#294](https://github.com/niko86/laterite/pull/294))
- **`build_ags4` per-heading UNIT/TYPE overrides.** Pass `units` / `types` as a `{code: {heading: value}}` map to give a custom heading a real unit/type instead of always filling from the dictionary. On Python and Node. ([#294](https://github.com/niko86/laterite/pull/294))
- **`registry.dictionary(edition)` — the per-edition standard dictionary, headless.** Returns one AGS edition's bundled standard dictionary (group + heading names, descriptions, UNIT/TYPE, status), the same shape the browser reference already used. ([#294](https://github.com/niko86/laterite/pull/294))
- **`lat-check --emit-index` — mint an `.ags.idx` certificate from the CLI.** After a clean check, writes the file's validity certificate beside it (or to `--index-out`). Skipped with a note if the file still has errors. ([#294](https://github.com/niko86/laterite/pull/294))
- **`BuildResult.applied` — the build's safe-fix ledger.** `build_ags4` now surfaces one `{kind, label, rule, line, risk}` record per safe fix it applied (the same shape `fix()` carries), so a data→AGS4 build can audit exactly what it normalised. ([#294](https://github.com/niko86/laterite/pull/294))
- **Content-addressed `_id` / `_parent_id` keys are core on every surface** — Python, Node and the browser (wasm), not just the DuckDB extension. Every group read carries two synthetic UUIDv8 columns so stateless cross-group joins, merge and dedup work in `.sql()` with no opt-in. The ids are deterministic and come from one shared Rust keychain — a golden-UUID test pins them byte-identical across Python, Node, wasm and the DuckDB extension. Frames strip the two columns by default and re-include them with `keys=True`. ([#303](https://github.com/niko86/laterite/pull/303))
- **Type-faithfulness gate (`ty`).** A CI step runs Astral's type checker over the shipped package against its abi3-py312 floor — the first static type gate in the project — paired with a runtime test asserting the hints hold (declared return types, every `Literal` value accepted, unknown kwargs rejected, the chained `-> Self` contract). ([#303](https://github.com/niko86/laterite/pull/303))

### Changed

- **The 174 typed-graph classes moved to the `laterite.groups` submodule.** `from laterite import PROJ, LOCA, …` is now `from laterite.groups import PROJ, LOCA, …`. This keeps the four-letter AGS codes out of the top-level namespace (which drops from ~200 to 26 public names). A clean break with no deprecation shim — **breaking**, but pre-1.0 with no external users. ([#302](https://github.com/niko86/laterite/pull/302))

### Fixed

- **`certify()` never overwrites the source `.ags`.** A `certify(path=…)` pointed at the source file (or any non-certificate) could clobber it; it now refuses, writing only the `.ags.idx` sidecar — a data-loss guard. ([#296](https://github.com/niko86/laterite/pull/296))
- **`fix()`'s residual findings report at the same errors+warnings tier on every surface.** The re-validation that produces a fix's residual had drifted (Python errors+FYI, Node errors-only, CLI errors+warnings); all three now match each surface's `validate()` default, so a warning a fix leaves behind is reported consistently. ([#294](https://github.com/niko86/laterite/pull/294))
- **`laterite.compat` raised `SyntaxError` on Python 3.12 / 3.13.** Three `except` clauses used the unparenthesized multi-exception form that only became valid in 3.14; now parenthesized — behaviour unchanged on every version. ([#303](https://github.com/niko86/laterite/pull/303))

[Unreleased]: https://github.com/niko86/laterite/compare/v0.9.0...HEAD
[0.9.0]: https://github.com/niko86/laterite/releases/tag/v0.9.0
[0.8.2]: https://github.com/niko86/laterite/releases/tag/v0.8.2
[0.8.1]: https://github.com/niko86/laterite/releases/tag/v0.8.1
[0.8.0]: https://github.com/niko86/laterite/releases/tag/v0.8.0
[0.7.0]: https://github.com/niko86/laterite/releases/tag/v0.7.0
[0.6.2]: https://github.com/niko86/laterite/releases/tag/v0.6.2
[0.6.1]: https://github.com/niko86/laterite/releases/tag/v0.6.1
[0.6.0]: https://github.com/niko86/laterite/releases/tag/v0.6.0
