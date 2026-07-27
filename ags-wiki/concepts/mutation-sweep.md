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

Three kinds of survivor — only the first is a test defect:

- **Real gap** — the mutated behaviour is observable at the boundary but no test
  asserts it → write the assertion.
- **Inert code** — the mutated value genuinely does not affect this path's output
  (e.g. an option a verb accepts but ignores) → not a test defect; note it, and
  weigh whether the code should carry that field at all ([[reliquary]]).
- **`cfg`-false survivor** — `cargo-mutants` mutates *source text* blind to `cfg`,
  so `#[cfg(feature = "tui")]` code the default build never compiles shows phantom
  survivors. Score it honestly by building `--features tui`, or record it as
  cfg-false with the line noted.

## The sweep ledger

Swept modules only; everything absent is **unswept** (see below). "residual" is
the count of survivors left standing *with a non-defect reason*.

| module | crate | swept | mutants | killed / residual | notes |
|---|---|---|---|---|---|
| `commands/validate.rs` | laterite-ags4-check | 2026-07-27 | 6 | 4 killed / 2 cfg-false | the `include_warnings` real gap closed; residual are `#[cfg(feature = "tui")]` (lines 103, 113) |
| `commands/fix.rs` | laterite-ags4-check | 2026-07-27 | 10 | 10 killed / 0 | three real gaps closed by two new tests — warnings counted in the residual, the sign of the `--json` exit code, and the `!risky && risky_available > 0` hint gate |
| `commands/rules.rs` | laterite-ags4-check | 2026-07-27 | 0 | n/a | `cargo-mutants` finds nothing mutable — the verb prints a static catalogue with no branch/return logic |
| `commands/diff.rs` | laterite-ags4-check | 2026-07-27 | 3 | 2 killed / 1 inert | the surviving `include_warnings` mutant is an **inert relic**, not a test gap — `diff` never runs the rule engine, so the field is never read ([[reliquary]], row spotted). Not test-rounded |

## The unswept surface

Everything **not** in the ledger is unswept — assume non-falsifiable tests may
hide there until a sweep says otherwise. Near-term Rust queue (tracks
[[coverage-campaign]]'s): the remaining `lat` command modules (`cert`, `certify`,
`read`, `transport`, `common`, `merge`, `excel`), then `laterite-cliutil`, then
the engine crates.

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
