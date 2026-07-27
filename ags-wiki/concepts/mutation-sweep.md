---
type: concept
title: "mutation sweep: keeping Rust tests falsifiable"
status: drafted
tags: [concept, testing, process, register]
volatile: [status]
volatile_asof: 2026-07-27
ags_editions: []
repo_refs:
  cli_verbs: "repo:rust-packages/laterite-ags4-check/tests/cli_verbs.rs"
related: [coverage-campaign, testing-strategy, perf-campaign, reliquary]
sources: []
---
# mutation sweep: keeping Rust tests falsifiable

## Definition

A **mutation sweep** runs `cargo-mutants` over a module: it mutates the source
(flip `==`→`!=`, delete a `!`, replace a return value with `Default::default()`),
rebuilds, and reruns the tests. A **surviving mutant** is a change the tests did
*not* catch — proof that a line is covered (executed) but not *asserted*. The
test that executes it is a **non-falsifiable test**: it passes, but it could not
fail, so it proves nothing. (This is what the campaign informally called "fluff";
"non-falsifiable" is the term of record.)

Where [[coverage-campaign]] rule 1 says *test behaviour, not lines*, the sweep is
the mechanism that **proves** it: coverage says the line ran; only a *killed*
mutant says a bug in that line would fail the suite. It is the third rung of
[[testing-strategy]]'s enforcement ladder (coverage → smell linters →
**mutation** → review) — the first rung a green number cannot game.

A module is **swept** once every surviving mutant is either killed (a test now
catches it) or classified as a non-defect (see the workflow's three kinds). The
**ledger** below records which modules are swept, so the *un-swept surface* —
where non-falsifiable tests may still hide — stays explicit rather than assumed
clean.

## Why it matters — coverage was green, the assertion could not fail

The `lat`-verb CLI tests (`repo:rust-packages/laterite-ags4-check/tests/cli_verbs.rs`)
gave the `validate` verb green line-coverage. The first sweep still survived
`delete field include_warnings`: the `--no-warnings` wiring executed on every run,
but no test asserted its *effect* — a fixture with only error-tier findings reads
the same warnings-on and warnings-off. Line coverage is structurally blind to
that; a surviving mutant is not. Closed by
`validate_warnings_are_on_by_default_and_no_warnings_drops_them`, which pins a
4-findings-default file dropping to 3 under `--no-warnings`. That is the exact
"it runs" gap [[coverage-campaign]] exists to refuse, made visible.

## The workflow (Rust)

Every batch in the Rust phase of [[coverage-campaign]] carries a sweep:

1. Write the behavioural tests for the module.
2. **Sweep the touched module** — `cargo mutants -d rust-packages -p <crate> -f <module>.rs`.
3. **Kill the real survivors.** Sweeping a file you are adding tests to routinely
   exposes *pre-existing* non-falsifiable tests as well — fix those in the same
   batch, not later.
4. **Record** the module in the ledger (date swept, mutants, residual).

Five kinds of survivor — only the first is a test defect:

- **Real gap** — the mutated behaviour is observable at the boundary but no test
  asserts it → write the assertion.
- **Inert code** — the mutated value genuinely does not affect this path's output
  (e.g. an option a verb accepts but ignores) → not a test defect: **remove the
  dead code** (inventory in [[reliquary]], then delete in the same or a following
  change) — never write a test around it, which would enshrine it as if it
  mattered.
- **`cfg`-false survivor** — `cargo-mutants` mutates *source text* blind to `cfg`,
  so `#[cfg(feature = "tui")]` code the default build never compiles shows phantom
  survivors. Score it honestly by building `--features tui`, or record it as
  cfg-false with the line noted.
- **Equivalent mutant** — the mutation changes the source but *no input reaches the
  difference*, so behaviour is identical and no test can tell them apart. `arg_json`
  guards a `None`-long argument with `is_positional()`; flipping that guard to
  `true` changes nothing, because every no-long-name argument in the CLI today *is*
  positional (there are no short-only flags). Unlike inert code it is not dead — the
  guard earns its place the moment a short-only flag is added — so **do not remove
  it**, and do not contort the program to kill it (adding a phantom flag just to
  make a test fail is the gaming this whole practice refuses). Record it as
  equivalent with the reason.
- **Harness-bound survivor** — the mutated effect is real and user-visible, but only
  through a channel the test *process* cannot observe: a live-terminal handle
  (`indicatif` and `comfy-table` suppress drawing and colour off a TTY, so a zebra
  stripe or a live progress bar renders identically to none), the terminal width, or
  a write straight to the global `stdout`/`stderr` rather than an injected writer
  (`emit` locks stdout; `note` is an `eprintln`). Not a test defect and not dead
  code — a pty-driven or output-capturing integration test could kill it. Kill what
  the harness *can* reach first (assert the returned handle's state, not its
  rendering — e.g. a hidden bar has no `length()` even though `is_hidden()` can't see
  the gate), then record the rest as harness-bound with the channel named. Prefer an
  injectable writer where the design allows.

**A clean sweep is not proof of total coverage.** `cargo-mutants` mutates at
function granularity — whole-return replacement, operator flips — not every
*literal*. A function that is one big `match` returning a distinct string per arm
(`cert::why`'s 14 reason messages) is mutated as a unit: replace the return with
one default and *any* test that calls it once kills the mutant, so the sweep reads
clean while thirteen arms are unasserted. The sweep is a rung, not the ladder;
this is exactly the per-arm gap the **review** rung above it exists to catch —
when a swept module is a message/lookup table, pin the arms with a unit test
regardless of the mutant score.

## The sweep ledger

Swept modules only; everything absent is **unswept** (see below). "residual" is
the count of survivors left standing *with a non-defect reason*.

| module | crate | swept | mutants | killed / residual | notes |
|---|---|---|---|---|---|
| `commands/validate.rs` | laterite-ags4-check | 2026-07-27 | 6 | 4 killed / 2 cfg-false | the `include_warnings` real gap closed; residual are `#[cfg(feature = "tui")]` (lines 103, 113) |
| `commands/fix.rs` | laterite-ags4-check | 2026-07-27 | 10 | 10 killed / 0 | three real gaps closed by two new tests — warnings counted in the residual, the sign of the `--json` exit code, and the `!risky && risky_available > 0` hint gate |
| `commands/rules.rs` | laterite-ags4-check | 2026-07-27 | 0 | n/a | `cargo-mutants` finds nothing mutable — the verb prints a static catalogue with no branch/return logic |
| `commands/diff.rs` | laterite-ags4-check | 2026-07-27 | 2 | 2 killed / 0 | clean — the lone survivor was the inert `include_warnings` field (`diff` never runs the rule engine, so it was never read); **removed**, not test-rounded ([[reliquary]]) |
| `commands/read.rs` | laterite-ags4-check | 2026-07-27 | 9 | 5 killed / 0 (4 unviable) | clean — 7 new `run`-verb tests (group listing, render, `--csv` quoting, `--out`, not-found 3 vs 4) on top of the existing `render_group` unit tests; unviable mutants are on the `-> !` `run` signature |
| `commands/merge.rs` | laterite-ags4-check | 2026-07-27 | 5 | 4 killed / 1 residual | 5 new tests (revision last-wins, `--json` summary, missing 3 / unparseable 4, and the `--tran-issue`/`--tran-date` stamp synthesis). The inert `include_warnings` field was **removed** ([[reliquary]]); the one residual is the `edition` field — live but observable only across editions with **differing** KEY structure, a documented gap needing an edition-divergent fixture (tracked in #127) |
| `commands/transport.rs` | laterite-ags4-check | 2026-07-27 | 3 | 3 killed / 0 | clean — 6 new tests: pack/unpack and lock/unlock round-trips (byte-identical), passphrase via both `--password-file` and `$LAT_TRANSPORT_PASSWORD`, wrong-pw exit 6, missing-input exit 3, non-envelope exit 6. The interactive TTY-prompt path is deliberately **not** e2e-tested (`rpassword` reads `/dev/tty`, so a subprocess test could hang) |
| `commands/certify.rs` + `cert.rs` | laterite-ags4-check | 2026-07-27 | 6 | 5 killed / 0 (1 unviable) | 5 e2e tests: certify clean / errors(1) / missing(3), and the `validate --index` round-trip — a fresh cert **skips the engine**, a stale cert is **refused** and re-validated. **Plus** a `cert::why` unit test: cargo-mutants reported clean while only 2 of `why`'s 14 reason arms were exercised (see the blind-spot note) — the unit test pins all 14 |
| `commands/common.rs` (shared flag folding) | laterite-ags4-check | 2026-07-27 | 6 | 5 killed / 0 (1 unviable) | 2 new e2e tests for the `--dict`/`--encoding` fold: a `--dict` overlay with a **forced base** (`--dict-version 4.2`) makes the bespoke `XTRA` group known (16 findings → 0) **without** tripping the `--dict-replace` conflict guard — killing `&& → \|\|` on `dict_replace && dict_version.is_some()`; and `--encoding` accepts `utf-8` / rejects an unknown label at exit 5 — killing `resolve_encoding → None`. The unviable mutant is on an `exit(5)` diverging path |
| `commands/excel.rs` | laterite-ags4-check | 2026-07-27 | 6 | 6 killed / 0 | clean — the `direction` inference already had unit tests; the one survivor was `delete !` on `!args.no_format_numeric`, the import-side default. Killed by a binary-only round-trip: export a 3DP column holding an under-precise `523145.1`, then import — default padding gives `523145.100`, `--no-format-numeric` keeps `523145.1`. No new deps (export writes strings, so the diff needs a value non-canonical for its TYPE, not a numeric xlsx cell) |
| `commands/census.rs` | laterite-ags4-check | 2026-07-27 | 7 | 6 killed / 1 equivalent | one unit test closed both real survivors — the `encodings` table (asserting `utf-8`/`latin9` resolve and the `cp1252x` typo → `null` policy pin) and the dropped-positionals arm (validate's `<file>` must survive reflection). The residual is an **equivalent mutant**: flipping the `is_positional()` guard to `true` is a no-op because the CLI has no short-only flags today |
| `main.rs` (`with_default_subcommand`) | laterite-ags4-check | 2026-07-27 | 18 | 18 killed / 0 | one unit test pins the argv pre-scan directly (asserting its output as a joined string) rather than through fragile e2e stdout matching: the loop bound (a no-positional line must not walk off the end), the four help/version alternatives (each passes through even with a trailing token — killing the three `\|\| → &&` mutants), and the flag-skip `+=` (a leading flag before an explicit verb — killing `+= → -=`; the `+= → *=` infinite-loop mutant is caught by timeout). **The whole `laterite-ags4-check` binary is now swept.** |
| `lib.rs` | laterite-cliutil | 2026-07-27 | 63 | 42 killed / 16 harness-bound (5 unviable) | new tests pin the pure layer: the coloured-JSON pretty-printer (each scalar's palette entry; a structure-preserving round-trip after stripping the palette back out — which catches the comma logic and empty-`[]`/`{}` rendering; per-depth indentation, incl. the array-of-object item recursion), the colour gate off a TTY, the progress-bar gate (via `length()` — `indicatif` auto-hides off a TTY so `is_hidden()` can't see the gate), and the spinner Live/Static gate. The 16 residuals are **harness-bound**: `comfy-table` drops colour off a TTY (zebra dim, 4), `indicatif` Live-only handles (Spinner set/drop + MultiLine set_line/set_header/suspend/drop/degenerate-gate, 7), `term_cols` width (2), `colour_enabled`'s TTY-masked terms (2), and the readme `process::exit` wrapper (1) |
| `report.rs` | laterite-cliutil | 2026-07-27 | 11 | 6 killed / 4 harness-bound (1 unviable) | a tiny in-test `Report` pins the default `full_value`/`compact_value` projections (the whole document, not a null) and `Plan::render_table` (headline, string-vs-JSON detail lines, footer), plus the `Ctx::colour` gate. Residuals: `emit` writes to a locked `stdout` and `note` to `stderr` (neither capturable in-process — their component writers are all separately tested), plus two TTY-masked `colour` terms |
| `lib.rs` | laterite-ags4-parse | 2026-07-27 | 152 | 136 killed / 6 equivalent (10 unviable) | one new `tests/lib_coverage.rs` pins the untested public surface (the crate leans on `tests/*` integration files, but these paths had no asserting caller): `ParsedGroup::cell`/`col`; `LineTerminator::as_str` arms; `split_ags_line` junk-skip + unquoted splits; `field_span` exact char offsets (quoted + unquoted); the walk's encoding passthrough + `source_true` flag (a Windows-1252 `©` byte), the BOM-for-decode strip (asserting the decoded line-1 *text* — the tag is read BOM-tolerantly via `first_field`, so group detection alone can't see it), and AGS3-marker detection; and a doubled-quote line-scan that must not swallow the newline. 11 of the kills are by timeout on loop mutants. The 6 residuals are **equivalent/defensive**: the `FieldStart` CR/LF arm the `Unquoted` fallback also catches (450), the after-close `+1` whose closing-quote byte is never a delimiter (464), the `LineSpans::next` `pos > len` guard that never fires under the `pos ≤ len` invariant (511×3), and `rest`'s `len > 1` whose `>=` branch still yields the same empty `split_off(1)` (546) |
| `scan.rs` | laterite-ags4-parse | 2026-07-27 | 72 | 68 killed / 2 equivalent (2 unviable) | two unit tests pin the doubled-quote (`""`) two-byte advance in both `scan_line` and `first_field_with` — the differential oracle deliberately **skips escape cases** (a borrowed slice can't unescape), so nothing else exercised that step; a mis-step overshoots the close (caught by the field bounds) or steps backward into a loop (timeout-killed). The 2 residuals are **equivalent**: `finish`'s `end - 1` in the `!closed && unterminated_to_eol` branch is reached only when `had_comma` is true, but an unterminated quoted field is always the trailing token (`had_comma = false`), so that sub-expression is unreachable |
| (whole crate) | laterite-ags4-reference | 2026-07-27 | 223 | 190 killed / 3 residual (30 unviable) | tests across five files pin the untested surface — **union** (`table`/`view` names, `group_depth` incl. its cycle guard, `Registry` len/is_empty/to_groups_json), **dict** (the custom-dict OVERLAY path: a directly-built `OwnedDelta` — its fields are public — drives `Dictionary::layered` through heading/group/group_codes/group_headings in both overlay and full-replacement forms, previously exercised only *downstream* in the validator, which this crate's own sweep never runs; plus `dictionary_dto` units/parents), **catalogue** (`rule_metadata_json`), **keychain** (`key_heading_names`/`shared_keys`), and **overlay** (`DictError` Display messages, `valid_status`, `desc_of`, and `build_delta`'s parent/description override + unit-wins-vs-inherits). The 3 residuals: `dict.rs` group_headings is an **equivalent** mutant (its branch is reached only with `fall_through=false`, where both sides yield the same empty result), and `detect_base`'s two edition-scoring mutants need a custom dict that scores *strictly higher on a non-newest edition* — a heading whose `(type,status)` diverges across editions, which the bundled dicts' cross-edition stability makes edition-archaeology (deferred) |

## The unswept surface

Everything **not** in the ledger is unswept — assume non-falsifiable tests may
hide there until a sweep says otherwise. The whole `laterite-ags4-check` binary
(every `lat` command module + `main.rs`), the shared `laterite-cliutil`
presentation crate, the `laterite-ags4-parse` leaf (`lib.rs` + `scan.rs`), and
the `laterite-ags4-reference` reference-data leaf are now swept; near-term Rust
queue (tracks [[coverage-campaign]]'s): the remaining engine crates —
`laterite-ags4-core`, `laterite-ags4-validator`, `laterite-ags4-emit`.

**Tests written before this workflow (2026-07-27) were never swept.** That backlog
is a standing GitHub issue (**#127**) — retro-sweep opportunistically whenever a
module is touched, rather than as one big pass.

## Other languages

The same idea has a tool per language, for when those coverage phases arrive:
**`mutmut`** or **`cosmic-ray`** (Python), **Stryker** (TypeScript — `web/`,
`laterite-node`). Rust/`cargo-mutants` is the active one; this ledger is Rust-only
for now.

## Related

- [[coverage-campaign]] — the line-% campaign this enforces the *quality* of
- [[testing-strategy]] — the enforcement ladder this is the mutation rung of
- [[perf-campaign]] — the sibling living-ledger this is modelled on
- [[reliquary]] — where inert-code survivors are inventoried for removal
