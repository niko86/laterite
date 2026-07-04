# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.6.1] — 2026-07-04

### Fixed

- **The risky Rule-1 fix now transliterates *all* non-ASCII characters, not just
  typography.** `fix(risky=True)` / `lat-check --fix-risky` folds `µ→u`,
  `°→deg`, `ß→ss`, `Ø→O`, accents→base letter (via `deunicode`), and maps the
  un-representable — including the `U+FFFD` replacement character that marks
  mojibake / already-lost data — to `?`. So a file whose only defect is
  non-ASCII content can be folded Rule-1-clean and then **certified**, where
  before only a curated typographic subset was fixed and the rest blocked the
  `read → fix → certify` flow. Applies on every surface (Python, Node, CLI,
  browser). Still opt-in (risky), since transliteration is a lossy guess.

## [0.6.0] — 2026-07-04

0.6.0 completes the cross-surface story. The browser and Node now reach every
capability except the `python-ags4` compat shim: Node and browser Excel I/O,
in-browser certificate minting, and browser transport (zstd + age) all land
here, alongside the Anonymiser's pseudonymise/preset upgrade and finer
fix-control across Python, Node and the CLI. One breaking change — the
typed-graph classes moved to `laterite.groups` — is what bumps the minor
pre-1.0. The npm package unifies onto this single version (the separate `node-v*`
number is retired), and a new cross-surface drift-gate
(`tests/test_version_faithful.py`) keeps every surface's version in lockstep.

### Added

- **Node Excel I/O — `toExcel()` / `fromExcel()`.** The Node addon binds the
  shared `laterite-excel` engine, so AGS4 ↔ `.xlsx` round-trips from Node the
  same way Python's `to_excel` / `from_excel` and the browser do — one sheet per
  group, born-typed columns, an `ExcelStats` summary. Closes the last
  non-compat blank in the Node capability column. (#361)
- **Browser Excel — AGS4 ↔ `.xlsx`, client-side.** The web app's Tools pane
  gains a two-way Excel converter running entirely in the browser (wasm) — no
  upload. Backed by a new FS-free bytes API in `laterite-excel`
  (`ags4_bytes_to_xlsx` / `xlsx_bytes_to_ags4`), so one engine drives Python,
  Node and the browser. (#367, #368)
- **Browser certificate minting — "Download certificate" in Validate.** A clean
  validation in the web app now offers the `.ags.idx` certificate as a
  download, minted client-side by a new wasm `certify()` over the shared core
  `Sidecar` — byte-compatible with the certificates Python, Node, the DuckDB
  extension and `lat-check --emit-index` produce. (#363, #364)
- **Browser transport — lock / unlock in Tools.** The web app seals and opens
  files client-side: zstd inside a passphrase-based `age` envelope
  (`.zst.age`), byte-compatible with `laterite.transport.lock`, the Node
  `transport` namespace and `pyrage` — so a file sealed in the browser opens
  with stock `age` + `zstd` given the passphrase. The scrypt work factor is
  pinned (log₂N = 18) for age-ecosystem interop. (#369, #370)
- **Anonymiser upgrade — pseudonymise, don't just blank.** The web Anonymiser
  gains per-category actions: location and sample IDs can be *pseudonymised*
  into stable, reference-preserving tokens (a cross-group join still works after
  scrubbing) instead of blanked, `PROJ_ID` maps to a content hash, coordinates
  blank, and a preset dropdown applies a common policy in one click. (#371)
- **Fix control — per-rule selection + a discoverable fixable-rule catalogue.**
  `fix()` takes `rules=` to apply only chosen repairs, and `list_rules()` /
  `fixable_rules()` expose which rules the engine can mechanically fix (a
  generated `FixableRule` `Literal`, drift-gated against the engine). On Python,
  Node and the CLI. (#297)
- **"N more fixable with `risky=True`" signal.** A `FixResult` (and the CLI's
  `--fix` summary) now reports how many further findings the risky,
  intent-guessing tier could repair beyond the safe pass — so you know a
  stronger fix exists without running it blind. (#298)
- **`Literal` types for enumerated string parameters.** Enumerated string knobs
  (`mode`, `xn`, `backend`, encoding labels, …) are now typed as `Literal`s /
  TypeScript string-unions, so an IDE completes the valid values and a typo is a
  type error rather than a runtime one. (#299)
- **`read()` remembers its encoding for the chained verbs.** A handle retains
  the encoding it was read with, so chained `.validate()` / `.fix()` / `.diff()`
  re-run against the true bytes with matching line numbers — no need to re-pass
  `encoding=`. On Python and Node. (#300)
- **Node certificate lifecycle — `Ags4File.certify()` + `read(f, { index })`.** Node
  gains the `.ags.idx` validity certificate the other surfaces have. After a clean
  `.validate()`, `ags.certify()` mints the certificate (a byte-offset index + a
  validation stamp) beside the file — refusing to overwrite the source or any
  non-certificate file — and `read(f, { index })` consumes it, freshness-checking
  size + SHA-256 (a mismatch throws `StaleCertError`). A fresh, engine-matching cert
  lets a later errors-only `.validate()` **skip the rule engine** (the report's
  `resolution` is the sentinel `"certified"`). The cert wraps the ONE core `Sidecar`,
  so a Node-minted `.ags.idx` is **byte-compatible + checker-compatible** with
  Python / `lat-check --emit-index` / the DuckDB extension (verified: a Node-minted
  cert reads fresh + engine-matching from Python). Exposes the `Sidecar` class and
  `StaleCertError`. (#294 Batch E / #14)
- **Node fluent chained layer — `Ags4File.validate()` / `.fix()` / `.diff()`.** The
  Node `Ags4File` gains the chained verbs, so `read(p).fix().validate()` reads as one
  chain (Python-parity). `.validate(opts?)` returns the handle (chainable) with the
  `Report` on `.report`; `.fix(opts?)` returns a NEW repaired `Ags4File` with its
  `FixResult` on `.fixReport` (non-destructive); `.diff(other, opts?)` returns the
  `RevisionDelta`. A handle now **retains its read source** (path / text / bytes +
  encoding), so the chained verbs re-run against the true bytes — matching original
  line numbers — and default `encoding` to the one it was read with. (#294 Batch E)
- **Node revision diff — `diff(a, b)`.** The Node port of `laterite.diff()`, over the
  SAME shared `laterite-ags4-diff` engine Python / the browser / `lat-check --diff`
  use, so the `RevisionDelta` is byte-identical across surfaces. `a` (baseline) and
  `b` (revision) are each a path, raw bytes, or an already-read `Ags4File`. Rows are
  matched by the group's dictionary KEY headings (not line order) and cells compared
  through the typed value (a formatting-only edit like `"1.0"` → `"1.00"` is not a
  diff); the KEY-heading edition comes from the revision's `TRAN_AGS` unless pinned
  with `dictVersion`. Returns per-group row/heading deltas plus `groups_added` /
  `groups_removed` and the `total_*` counts. Node had no revision-diff at all before.
  (#294 Batch E / #4)
- **`read_typed` reads text and bytes, not just a path.** `laterite.ags4.read_typed`
  now takes the same inputs as `read()` — a positional `source` auto-detected as a
  path / file-like / bytes / AGS4 text, plus keyword-only `path=` / `text=` / `data=`
  doors and an `encoding=` label (WHATWG, default UTF-8) for bytes / path input. It
  was path-and-UTF-8-only before, so an in-memory or non-UTF-8 transfer couldn't reach
  the base typed read. (#294 Batch B / #13)
- **`build_ags4` per-heading UNIT/TYPE overrides.** Pass `units` / `types` as a
  `{code: {heading: value}}` map to override what a heading emits, instead of
  always filling from the standard dictionary — e.g.
  `build_ags4({"LOCA": df}, units={"LOCA": {"LOCA_XTRA": "kPa"}}, types={"LOCA": {"LOCA_XTRA": "3DP"}})`.
  Handy for giving a *custom* heading (one the dictionary doesn't know) a real
  unit/type. Name only the headings you want to set; the rest still fill from the
  dictionary. On Python (`units=`/`types=`) and Node (`EmitOptions.units`/`types`);
  an unknown group or heading raises. The browser build already carried this. (#294 Batch F)
- **`registry.dictionary(edition)` — the per-edition standard dictionary, headless.**
  Python `laterite.registry.dictionary(edition)` and Node `registry.dictionary(edition)`
  return one AGS edition's bundled standard dictionary — canonical group + heading
  names, descriptions, UNIT/TYPE, status — as `{ags_edition, groups: [{code,
  contents, parent, headings: […]}]}`, the same shape (and now the same shared Rust
  builder) the browser reference already used. The module-level union `GROUPS` stays
  the default; this is the single-edition view (`"4.0.3"…"4.2"`, or `None`/`"auto"`
  → fallback). (#294 Batch F)
- **`lat-check --emit-index` — mint an `.ags.idx` certificate from the CLI.** After
  a clean check, `lat-check <file> --emit-index` writes the file's validity
  certificate (a byte-offset index + validation provenance) beside it, or to
  `--index-out <path>`. It's skipped (with a note) if the file still has errors —
  a certificate attests a clean validation — while warnings/FYI ride on the stamp
  as counts without blocking it. This joins the existing minting layers (Python's
  `.certify()`, the DuckDB extension's `ags_index`), which the `index` module's
  docs already named. (#294 Batch F)
- **`BuildResult.applied` — the build's safe-fix ledger.** `build_ags4`'s autofix
  already computed the full list of safe fixes it applied, then discarded it to a
  bare count. It now surfaces `applied` — one `{kind, label, rule, line, risk}`
  record per fix, the *same* shape `fix()`'s `FixResult.applied` carries — on both
  Python (`BuildResult.applied`) and Node (`BuildResult.applied`), with
  `fixes_applied` / `fixesApplied` now just its length. So a data→AGS4 build can
  show or audit exactly what it normalised, not only how many. (#294 Batch F)
- **Content-addressed `_id` / `_parent_id` keys are now core on every surface —
  Python, Node, and the browser (wasm), not just the DuckDB extension.** Every
  group read carries two synthetic UUIDv8 columns in the relational layer — `_id`
  and, for a child group, `_parent_id` equal to its parent row's `_id` — so
  stateless cross-group joins, merge and dedup work in `.sql()` with no opt-in:
  `ags.sql("SELECT * FROM SAMP s JOIN LOCA l ON s._parent_id = l._id")` (this used
  to raise *Binder Error: no column "_parent_id"*). The ids are deterministic
  (re-reading a file yields identical values) and come from the one shared Rust
  keychain — a golden-UUID test pins them **byte-identical across Python, Node,
  wasm and the `.ags5db` extension**. Frames strip the two columns by **default**
  — `ags["LOCA"]` is AGS columns only — and re-include them on request:
  `read(..., keys=True)` / `ags.table("LOCA", keys=True)` (Python),
  `ags.table("LOCA", { keys: true })` (Node), `arrow_ipc(code, true)` (wasm). Emit
  never writes them. (#303)
- **Type-faithfulness gate (`ty`).** A CI step runs Astral's type checker
  (`uv run ty check`) over the shipped `laterite` package against its real
  abi3-py312 floor — the first static type gate in the project (previously only
  `ruff`). Paired with a new `tests/test_type_faithfulness.py` that asserts the
  hints hold at *runtime* (declared return types, every `Literal` value accepted,
  unknown kwargs rejected, the chained `Ags4File` `-> Self` contract). Config in
  `[tool.ty]`. (#303)

### Changed

- **The 174 typed-graph classes moved to the `laterite.groups` submodule.**
  `from laterite import PROJ, LOCA, …` is now `from laterite.groups import PROJ, LOCA, …`.
  This keeps the four-letter AGS codes out of the top-level `laterite` namespace, which
  drops from ~200 to 26 public names — so `laterite.<TAB>` and `from laterite import *`
  now surface only the read / validate / build API, not 174 group codes. The classes
  themselves are unchanged (still compiled into `_laterite_native`); `read_typed` and
  `build_ags4(<typed graph>)` are unaffected, and `laterite.groups` is reachable after a
  bare `import laterite`. A clean break with no deprecation shim — breaking, but pre-1.0
  with no external users (same rationale as the 0.5.1 removals). (#302)

### Fixed

- **`certify()` never overwrites the source `.ags`.** A `certify(path=…)` that
  pointed at the source file (or any non-certificate) could clobber it; it now
  refuses, writing only the `.ags.idx` sidecar — a data-loss guard. (#296)
- **`fix()`'s residual findings now report at the same errors+warnings tier on
  every surface.** The re-validation that produces a fix's *residual* (what could
  not be mechanically repaired) had drifted: Python reported errors+FYI, Node
  errors-only, the CLI errors+warnings. All three now match each surface's own
  `validate()` default (errors+warnings), so a warning a fix leaves behind — e.g.
  the O-44 unrecognised-`TRAN_AGS` warning — is reported consistently instead of
  silently dropped by Node/Python. The set of fixes *applied* is unchanged (every
  fixable rule is error-tier). (#294 Batch C)
- **`laterite.compat` raised `SyntaxError` on Python 3.12 / 3.13.** Three `except`
  clauses used the unparenthesized multi-exception form (`except A, B:`) that PEP 758
  only made valid in 3.14, so importing `laterite.compat` crashed on the 3.12 / 3.13
  the wheel ships to (the dev env is 3.14, which masked it). Now parenthesized —
  behaviour unchanged on every version. Caught by the new `ty` gate's first run. (#303)

## [0.5.1] — 2026-06-24

A Python patch release: completes one cross-surface parity gap, fixes IDE/type-checker
resolution of the typed-graph classes, and removes the deprecated back-compat shims that
0.5.0 still carried. The removals are technically breaking, but the shimmed surface was
already deprecated and the library is pre-1.0 with no external users — hence a patch. (The
npm package stays at 0.5.0; this is the `v*` Python track only.)

### Added

- **`build_ags4` accepts a typed PROJ graph.** Previously it took only a `{code: frame}`
  mapping or `(code, frame)` list; a typed `PROJ` root raised
  `TypeError: 'PROJ' object is not iterable`. It now also walks a typed-graph root
  depth-first — the same registry-driven traversal as Node's `buildAgs4`, closing a
  cross-surface gap. A custom group attached by `read_typed` survives a
  `read_typed` → `build_ags4` round trip. (#214)

### Fixed

- **`from laterite import PROJ, LOCA, …` resolves in IDEs / type-checkers.** The 174
  typed-graph classes are bound to the package root at runtime by a dynamic loop, which
  static analysers couldn't see — so Pylance/pyright flagged the imports as unknown symbols
  (no autocomplete, spurious type errors) even though they work at runtime. The `.pyi` stub
  now carries an `__all__` and the package re-exports it at type-check time. (#261)

### Removed

- **Deprecated back-compat shims** (all already deprecated in 0.5.0):
  - `build_ags4(edition=)` — use `dict_version=`. (#258)
  - the legacy `db=` / `zst=` / `file=` keyword arguments on
    `transport.{pack,unpack,lock,unlock}` — pass the source positionally. (#258)
  - `read_typed(attachments_dir=)` — was accepted and ignored (no binary side-channel in the
    typed tree). (#260)
  - `lat-check --show-warnings` — warnings are shown by default since 0.5.0; use
    `--no-warnings` to opt out. (#258)
  - the `[pandas]` install extra — use `[compat]`, which includes pandas. (#258)
  - **Node** (`laterite` npm 0.5.1, on its own `node-v*` track): `edition` removed
    from `EmitOptions` — use `dictVersion`. (#258)

## [0.5.0] — 2026-06-23

The next line settles the AGS4 surface and decouples the experimental AGS5 strand. A
**breaking** change (the `[ags5]` extra and the `.ags5db` companion are gone from the
shipped package); the 0.x version signals the surface is still settling.

### Added

- **`lat-check <a> --diff <b>`.** The revision diff on the CLI — the Rust `lat-check`
  binary and the Python `_cli` (byte-identical output): a per-group
  `+added −removed ~changed` summary, or the full `RevisionDelta` with `--json`. Completes
  `revision-diff` across Python + CLI (#204).
- **`laterite.diff(a, b)` + `Ags4File.diff(other)`.** The KEY-aware, type-aware revision
  diff — previously browser-only — now on the Python surface. Compares two AGS4 documents
  (each a path / text / bytes / `Ags4File`), matching rows by the group's dictionary KEY
  headings and comparing cells through the **typed** value (so a formatting-only
  `"1.0"`→`"1.00"` isn't reported); returns a `RevisionDelta` dict (per-group deltas + total
  added/removed/changed). Backed by the shared `laterite-ags4-diff` leaf — the same diff the
  browser's Tools tab uses (the CLI `lat-check --diff` follows). (#204)
- **wasm `list_rules()` + the `warnings` knob.** The browser/wasm surface gains the rule
  catalogue (`list_rules()`, the gated `rules_meta.json`) and an `include_warnings` parameter
  on `validate` — so Rule 18 WARNINGs now reach the web validator (its severity filter already
  had a warning lane; the engine just never produced any). Completes the cross-surface
  warnings/fyi knob (#204). The wasm `build_ags4`/`build_ags4_ipc`/`dictionary` params also
  moved `edition`→`dict_version` (positional, caller-transparent), finishing the #241
  vocabulary alignment on wasm.
- **Node `fix()` + `listRules()`.** The Node surface gains the two verbs it was missing
  versus Python/CLI: headless mechanical repair — `fix(source, { risky })` → a `FixResult`
  (`.bytes` / `.text` / `.save(path)`; `findings` is what couldn't be mechanically fixed) —
  and the rule catalogue `listRules()` → typed `RuleMeta[]` (the gated `rules_meta.json`,
  no phantom `12`/`16a`). Both share the engine's `fix_document` / `rule_metadata_json`, so
  `applied[]` and the catalogue are byte-identical across Python / CLI / Node. (#204
  gap-closures.)
- **Rule 18 — opt-in WARNING for a malformed DICT group (the first WARNING-tier
  producer).** A file that declares custom groups/headings through a structurally
  broken DICT — a missing `DICT_TYPE`/`DICT_GRP`/`DICT_HDNG` column, a blank
  `DICT_GRP`, or a `HEADING` row with no heading name — now surfaces a `Warning
  (Related to Rule 18)` under `validate(warnings=True)` / `lat-check --show-warnings`.
  The engine only *consumed* DICT before, so a malformed one silently degraded every
  downstream check. It's **WARNING, not Error** — the file is broken but the spec is
  silent and python-ags4 doesn't check it, so an error would break parity; opt-in + a
  separate label keep the default verdict and the python-ags4 path unchanged (O-44).
  This makes the previously-inert `--show-warnings` / `warnings=True` flag live.
- **`lat-check --list-rules` + `laterite.list_rules()`.** A read-only AGS4 rule
  catalogue — one entry per rule with title, severity (`error` / `fyi` / `mixed`),
  whether `fix` can repair it, and the cited `O-N` divergence notes. The CLI prints a
  table (or `--json`, no input file needed); the Python verb returns `list[dict]`.
  Backed by a new **gated single source** (`rules_meta.json`, compile-time-embedded so
  there's no runtime parse): a faithfulness test asserts it covers *exactly* the rules
  the engine emits and that `fixable` matches the fix engine — so it can't list a rule
  that doesn't exist (the old hand-curated web catalogue had drifted, carrying a no-op
  `12` and a non-existent `16a`). Repointing the web RuleExplainer onto this source — it
  still serves its old, now-divergent copy until then — and a companion `--edition-delta`,
  follow.
- **Headless mechanical fix/repair.** `laterite.fix(source)` / `Ags4File.fix()` and
  `lat-check --fix` apply the same fix engine the browser uses — without a UI — to an
  existing delivery: the **safe** fixes (CRLF / BOM / embedded-CR / short-row pad /
  numeric reformat / TRAN delimiter+concatenator rows) always, the intent-guessing
  ones (duplicate-heading rename, `dd/mm` date, typography) under `risky=` /
  `--fix-risky`. The result is re-validated, so `FixResult.findings` is what could not
  be mechanically fixed. Non-destructive: the Python verb returns the fixed bytes
  (write with `in_place=`/`out=`/`.save()`); the CLI writes a sibling
  `<file>.fixed.ags` by default (`--in-place` / `--fix-out` to redirect). The single
  fix orchestration lives in the validator (`fix_document`), shared by the PyO3 verb
  and the CLI.
- **Rule 16 — non-standard abbreviation FYI.** A `PA` value self-declared in the file's
  own ABBR group but not in the bundled standard picklist for its heading (a typo like
  `"Borng"`, or an invented code) now surfaces as an opt-in `FYI (Related to Rule 16)`
  (`validate(fyi=True)` / `lat-check --show-fyi`). The file stays spec-legal — Rule 16 is
  satisfied by the ABBR declaration — so it is informational, never an error or warning;
  bounded to headings that have a standard picklist (custom/DICT-defined `PA` headings are
  skipped). A laterite-originated check python-ags4 lacks (OBSERVATIONS O-43).
- **First-class Excel verbs on the modern surface** — `laterite.to_excel(source, out)`
  / `laterite.from_excel(xlsx)` (plus `Ags4File.to_excel(out)`). AGS4↔XLSX round-trip
  was already shipped Rust-side but reachable only through the legacy `compat` shim;
  it is now a first-class verb (one sheet per group, pyarrow/openpyxl-free). `to_excel`
  accepts anything `read` does (path / text / bytes / `Ags4File`); `from_excel` returns
  a parsed `Ags4File` by default, or writes an AGS4 file when given an output path.
- **`Report.findings` / `Report.by_rule()` now carry severity + location.** The frame
  gains `severity` (`"error"` / `"warning"` / `"fyi"`), `target`, `heading`,
  `field_index` and `data_row` columns; `by_rule()` items carry the same plus
  `char_span`. The data already crossed the PyO3 boundary (and `to_json` / `to_ndjson`
  exposed it) — the dataframe path simply dropped it, so `validate(warnings=True,
  fyi=True)` couldn't tell the tiers apart or reach the offending heading/field.
  Additive — the existing `rule` / `line` / `group` / `desc` columns and `by_rule`
  keys are unchanged.
- **`read(..., xn="numeric")`** — an opt-in read-side numeric view of AGS `XN`-typed
  columns ("numeric, may carry a non-numeric qualifier" — `NP` / `<5` / `>100`). Every
  XN heading materialises as `Float64` across the handle (`ags[code]` / `sql` / `at`),
  non-numeric tokens becoming null. Default stays byte-faithful text; write-back is
  unaffected. A fuller bidirectional/born-typed XN treatment is future work.
- **`laterite.source`** — the fluent-chain entry name, an alias of `laterite.read`
  (`laterite.source(x).validate()…`); same callable, exported in `__all__`.
- **A published source distribution.** The release builds + publishes an sdist (once, on
  the Linux leg) alongside the wheels, so a platform without a published wheel (e.g. ARM
  Linux) installs from source. The sdist vendors all the path-dep Rust crates and is
  verified buildable + installable in isolation.

### Changed

- **Validation reports show WARNINGs by default** — severity tiers now track importance
  like a compiler: errors **and warnings** by default, FYI still opt-in. The default flips
  to `warnings=True` across `laterite.validate()` / `Ags4File.validate()`, the Node
  `runCheck`, and `lat-check` (opt out with the new `--no-warnings`; the old `--show-warnings`
  is an accepted no-op). Pass `warnings=False` for errors-only. The **verdict is unchanged** —
  warnings never gate `is_valid` / the error count — and the `compat` python-ags4 shim keeps
  its own errors-only output, so the 122/9 parity is untouched. (The DuckDB `validate_ags` SQL
  function stays opt-in by convention.) One consequence: a *default* `validate()` no longer
  takes the `.ags.idx` certificate skip (a cert records the error verdict, not the warning
  list) — pass `warnings=False` to engage it on a known-clean file. (#203)
- **An unrecognised `TRAN_AGS` edition is now a WARNING (was an FYI).** A `TRAN_AGS` that
  isn't a recognised AGS4 edition makes laterite fall back to a default dictionary and possibly
  validate against the wrong schema; a default-visible `Warning (Related to Rule 14)` now flags
  that risk rather than the buried FYI. `compat` keeps python-ags4's FYI for drop-in fidelity.
  Combined with the WARNING-default above, the schema-fallback risk is now seen without a flag
  (OBSERVATIONS O-45).
- **`Ags4File.fix()` now returns a repaired `Ags4File`** (was a `FixResult`) — the fluent
  capstone, so `read(path).fix().validate().save(out)` reads as one chain. The repaired
  handle inherits the source's `backend` / `xn`; the `FixResult` (what was applied + the
  residual findings) rides on its new `.fix_report` attribute. **Breaking** for callers of
  the *method* — the free `laterite.fix(source)` is unchanged and still returns a
  `FixResult` (with its `in_place=` / `out=` write options), so reach for that when you want
  the report. Completes the fluent surface (#204).
- **New internal `laterite-ags4-diff` leaf.** The KEY-aware/type-aware revision-diff core
  (`diff_parsed` + its row/cell deltas) is extracted out of the `laterite-ags4-wasm` crate
  into a wasm-safe leaf — the browser `diff()` output is unchanged. It's the shared engine
  that brings `revision-diff` to Python + the CLI next (#204).
- **`build_ags4`: `edition` → `dict_version`.** The data→AGS4 builder's edition argument is
  renamed to `dict_version` (Python `build_ags4(dict_version=...)`, Node
  `buildAgs4(groups, {dictVersion})`) so the whole surface speaks one word for the AGS
  edition (matching `validate` / `read`). `edition` stays a **deprecated alias** — a
  `DeprecationWarning` in Python, a `@deprecated` JSDoc in Node — so existing callers keep
  working. First step of the cross-surface vocabulary alignment (#204); wasm + the DuckDB
  extension follow.
- **Pre-GROUP orphan rows are now reported (Rule 2).** A `HEADING`/`UNIT`/`TYPE`/`DATA`
  row appearing before any `GROUP` was previously dropped silently (the lenient parser);
  it is now reported as an `AGS Format Rule 2` finding, so the validator gives a complete
  report where python-ags4 hard-fails on the same input (O-41).
- **`TRAN_AGS="4.0"` resolves to the newer 4.0.4 dictionary** (it is a strict superset of
  4.0.3), and an exact `"4.0.3"` is upgraded to 4.0.4 when the file uses a 4.0.4-only
  heading — with a transparency FYI. This avoids the false **Rule 10c** orphans a stale
  `"4.0"`→4.0.3 mapping produces on a file that is actually 4.0.4 (O-30 / O-42).
- **One faithful, multi-edition dictionary.** The registry is now generated from a single
  `ags_dictionary.json` projecting the official AGS dictionaries across editions
  4.0.3 → 4.2, consumed as the latest-edition **union**. The AGS4 dictionary drops the
  AGS-L draft groups (CONL / TREL / TRIL) and the `is_high_volume` flag. (Also fixed a
  KEY-detection bug: combined statuses like `KEY+REQUIRED` are now recognised as KEY.)

### Removed

- **The experimental `.ags5db` / `.agsx` (AGS5) surface is decoupled from the shipped
  package** — the `[ags5]` extra, the `laterite-ags5` companion wheel, and the `lat-db`
  CLI are no longer built or published (moved to a dormant holding folder for a future,
  separately-published AGS5 strand). `from laterite.ags5db import …` no longer resolves;
  `pip install laterite[ags5]` no longer exists. This reverses the 0.4.0 "`laterite-ags5`
  now publishes to PyPI". The shipped `laterite` is AGS4-only and links no bundled DuckDB.

### Fixed

- **Stale package metadata** — the PyPI `description`, README and keywords claimed a
  "narwhals-native API" and advertised the removed `[ags5]` extra. Both have been gone
  since 0.4.0 (the API returns polars/pandas directly, pyarrow-free; the AGS5 surface was
  decoupled). Corrected so the package page reflects the shipped surface.
- **Cross-surface CLI / consistency fixes (0.5.0 audit).** The Python `lat-check` gained
  `--encoding` (the Rust binary already had it) and now rejects `--json` + `--ndjson`
  together with exit 5 (it previously preferred JSON silently); the Python CLI README was
  resynced to the Rust one (it had drifted — missing the `--fix` / `--list-rules` docs).
  The DuckDB perf tool referenced a non-existent `ags_validate` (the registered function
  is `validate_ags`), so its validate timing never ran. Transport/Excel errors no longer
  carry a stale `"ags5db error"` brand (now `"laterite error"`). Comment/doc cleanups: the
  typed-graph codegen said "92 classes" (it emits 174); the DuckDB-extension design page
  used the old `ags_validate` name.

## [0.4.0] — 2026-06-16

The **0.4.0** line — the DuckDB-engine redesign of the read / data-science surface, the
`laterite-*` crate rebrand, and version unification. This is a **breaking** change (frame
return types move from narwhals to polars / pandas; the validator CLI is renamed to
`lat-check`); the 0.x version signals the surface is still settling.

### Added

- **`laterite-ags5` now publishes to PyPI.** `pip install laterite[ags5]` resolves the
  `.ags5db` companion wheel from PyPI (previously the companion shipped via GitHub release
  only). The base `laterite` and `laterite-ags5` are separate PyPI projects with separate
  trusted publishers; the `[ags5]` extra pulls the companion.
- **In-memory DuckDB engine.** `laterite.read(path)` loads each AGS4 group into a
  born-typed table in a Python-owned in-memory DuckDB (the pip `duckdb` package, *not*
  bundled in the wheel), populated lazily per group on first touch:
  - `ags["LOCA"]` → an eager **polars** `DataFrame` by default, born typed (a 2DP heading
    is `Float64`, an ID `str`, a non-conforming numeric cell `null`).
  - `read(path, backend="pandas")` → the same accessor returns **pandas**; both paths are
    **pyarrow-free** (polars via the Arrow C-stream capsule, pandas via DuckDB's NumPy
    `.df()`).
  - `ags.sql("SELECT … WHERE …")` → a **DuckDB relation**: real cross-group joins and
    **filter pushdown** (the `WHERE` runs in the engine, so a huge file materialises only
    the rows you keep); finish with `.df()` / `pl.from_arrow(rel)` or chain more SQL.
  - `ags.at("LOCA", ["BH01", "BH02"])` → a **filtered subset view**: pull a parent entity's
    related records across groups (everything keyed by `LOCA_ID`), materialising only the
    matching rows. `sub.groups` lists the related groups; chain `sub["SAMP"]`.
  - `ags.register(name, frame)` joins your own frames against the groups; `ags.connection`
    exposes the raw DuckDB connection (parquet export, Arrow via `.arrow()`, …). Read
    handles are context managers — `with laterite.read(p) as ags: …` — with `close()`.
- **Born-typed reads on a zero-copy Apache Arrow boundary.** The Rust parser builds typed
  Arrow per group (`pyo3-arrow`, pyarrow-free) to seed the engine; `to_numeric` is now
  redundant for typed columns (kept for the compat shape). Writes stay **byte-faithful** —
  re-emitted from the retained Rust parse, independent of which groups were read.
- `laterite.transport` (`pack` / `unpack` / `lock` / `unlock`) documented as
  **content-agnostic** — any file, not only `.ags5db`; a non-path arg now raises an
  actionable `TypeError`.
- **Data → valid AGS4.** `laterite.emit_ags4(groups, *, edition, mode)` builds a
  spec-correct AGS4 document from your own per-group polars / pandas frames (columns =
  AGS headings) — the inverse of `read`, with `autofix` / `report` / `strict` modes and
  per-edition UNIT/TYPE fill. A **base** feature, pyarrow-free for polars. The browser
  gets the same producer via the wasm `to_ags4` / `to_ags4_ipc` exports and a web Export
  tab.

### Changed

- **BREAKING — frame return types are now polars / pandas, not narwhals.** The public
  surface (`ags[code]`, `Report.findings`, `compat`) returns native polars (default) or
  pandas frames directly; **narwhals is dropped**. A narwhals user wraps the result in one
  line — `nw.from_native(ags["LOCA"])`. No deprecation shim (nothing depends on 0.2's
  narwhals returns yet).
- **Dependencies.** Base is now **`polars + duckdb`** (was `polars + narwhals`); **narwhals
  and pyarrow are no longer base dependencies**. The `[compat]` extra is **`pandas +
  pyarrow`** — compat's python-ags4 pandas output goes through `polars.to_pandas()`, which
  needs pyarrow; compat-polars and the whole core are pyarrow-free. The single
  `cp312-abi3` wheel is preserved.
- `Ags4File(...)` given a bad argument (e.g. a file path) now raises a clear `TypeError`
  pointing at `laterite.read()`, instead of failing several calls later with a cryptic
  `'PosixPath' object is not subscriptable`.
- **BREAKING — the validator CLI command is renamed `ags4-check` → `lat-check`.** The wheel
  console-script and the standalone Rust binary attached to the GitHub release are now
  `lat-check` (the flags, JSON/NDJSON shape, and exit codes are unchanged). Re-invoke as
  `lat-check delivery.ags --json`. Part of the crate-suite rebrand (the Rust crate is now
  `laterite-ags4-check`); there is no `ags4-check` alias.

### Deprecated

- The first parameter of `transport.pack` / `lock` (was `db=`), `unpack` (was `zst=`)
  and `unlock` (was `file=`) is unified to `src=`. The old keyword names still work but
  emit a `DeprecationWarning` and will be removed in a future release.

### Fixed

- **Base install no longer reaches into the optional extras** (#111). On a plain
  `pip install laterite` (no `[ags5]` / `[compat]`):
  - `laterite.ags4.read_typed(path)` builds the typed PROJ graph on the base path — it
    no longer routes through a temporary `.ags5db`, so it no longer pulls the `[ags5]`
    (DuckDB) companion.
  - `laterite.emit_ags4(...)` with a polars frame is pyarrow-free — it no longer round-
    trips through DuckDB (whose polars ingest called `.to_arrow()`, pulling `[compat]`'s
    pyarrow).
  A regression guard exercises the whole base surface under a simulated base-only install.

### Changed

- **Python support widened to ≥ 3.12** (previously ≥ 3.14). Both `laterite` and
  `laterite-ags5` now build as a single `cp312-abi3` wheel per platform —
  one artefact, installable on Python 3.12, 3.13, and 3.14.

### Added

- Validator findings now carry character-level spans and rule-aware severity,
  enabling precise in-place error highlighting.
- **AGS4 fix engine** — compute and apply mechanical corrections (pad short rows
  to a group's field count, typographic→ASCII normalisation), with an opt-in
  *risky* DATETIME-canonicalisation fixer kept separate from the safe set.
- Edition-selectable dictionary / templates, sourced from the per-edition AGS
  standard dictionary.

### Fixed

- Correct the TRIL group classification and flag CONL/TREL/TRIL as AGS-L.

## [0.1.0] — 2026-05-27

Initial public release.

### Added

- **AGS4 reader / writer / validator**, Rust-backed via PyO3.
  - `laterite.validate(text=...)` / `laterite.read(...)` / `laterite.write(...)` — narwhals-native return types.
  - `ags4-check` CLI binary (bundled in the wheel as a Python entry point; standalone Rust binary attached to the GitHub release).
  - 122/131 of the python-ags4 parity suite passes; see [`docs/parity-coverage-map.md`](docs/parity-coverage-map.md).
- **`laterite.compat`** — drop-in replacement for `python-ags4`'s public surface (pandas frames by default; `set_backend("polars")` opts out of the pandas dependency).
- **92 typed-graph classes** auto-generated from the AGS dictionary: `from laterite import PROJ, LOCA, SAMP, …`. Field names, units, and types match the AGS4 standard dictionary.
- **`laterite.transport`** — zstd + age envelope helpers (`pack` / `unpack` / `lock` / `unlock`). Compression and password-protected secure transport for AGS data files.
- **`laterite[ags5]` extra** *(experimental, opt-in)* — installs the
  `laterite-ags5` companion wheel and unlocks `laterite.ags5db`:
  DuckDB-backed `.ags5db` read / write / convert / export / query /
  diff / peek, plus binary blob attachments. Adds ~50 MB to the install
  footprint (bundles DuckDB), kept out of the base wheel so AGS4-only
  users stay light.
- **`lat-db` CLI binary** *(experimental)* — attached to the GitHub release for users who want the `.ags5db` toolchain without installing the Python wheel.

### Known limitations

- Python 3.14 only (the wheel is pinned to a specific CPython build
  via pyo3-polars).
- `.ags5db` is experimental — format and API may change in v0.2. AGS4
  surface is stable.
- 9 python-ags4 parity tests fail by design; see
  [`docs/parity-coverage-map.md`](docs/parity-coverage-map.md).

[Unreleased]: https://github.com/niko86/laterite/compare/v0.6.1...HEAD
[0.6.1]: https://github.com/niko86/laterite/releases/tag/v0.6.1
[0.6.0]: https://github.com/niko86/laterite/releases/tag/v0.6.0
[0.5.1]: https://github.com/niko86/laterite/releases/tag/v0.5.1
[0.5.0]: https://github.com/niko86/laterite/releases/tag/v0.5.0
[0.4.0]: https://github.com/niko86/laterite/releases/tag/v0.4.0
[0.2.0]: https://github.com/niko86/laterite/releases/tag/v0.2.0
[0.1.0]: https://github.com/niko86/laterite/releases/tag/v0.1.0
