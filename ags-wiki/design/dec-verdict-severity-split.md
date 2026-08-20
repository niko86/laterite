---
type: decision
title: "The verdict splits from the report: a warning is shown, not fatal"
status: accepted
tags: [design, decision, validator, severity]
decided: "2026-08-20"
supersedes: []
from_gap: []
related: [laterite-ags4-validator, laterite-cli, laterite-ags4-xcheck, parity-model, dec-custom-dict-overlay, O-44, O-45, O-51]
sources: []
---

# The verdict splits from the report: a warning is shown, not fatal

## Context

The severity tiers decided two different things with one dial. `include_warnings`
and `include_fyi` chose what a report **showed**, and the exit code was computed
from the resulting total — `i32::from(count != 0)` in `laterite-cli`, and the
same expression again in `laterite-py`. So a tier that existed to draw attention
also decided pass/fail, and the two questions could not be answered separately.

The consequence a user meets: a file whose only blemish is an unrecognised
`TRAN_AGS` edition ([[O-45]]) breaks no rule, is reported as a WARNING — and
exited `1`, failing a CI job. Since #203 made the warning tier visible by
default, that outcome arrived without anyone opting in.

The tiers themselves were not the defect. Applying the rule below to all seven
warning sites keeps six of them; the exit code was what was wrong.

## The rule a warning has to satisfy

**A warning predicts a downstream *surprise*: a consumer may silently receive
something other than the author meant.** Everything else is FYI.

This rule is only stateable once a warning no longer fails a build. While it
did, the stakes of the tier were "does this stop the pipeline", and the question
could not be about attention. Once the verdict moves out, warning and FYI differ
only in default visibility, and the test becomes a question about the reader.

Applied to the seven sites it demotes exactly one — the custom-dictionary
type/status override ([[O-51]]), which is announced by the caller and honoured
exactly as declared, with nothing silently differing. One demotion out of seven
is the evidence for the headline: the tiers were mostly right.

## The compiler analogy, finished

`severity-tiers.md` already reached for the compiler: *"like a compiler — the
default report shows errors and warnings"*. That is true of display and was
false of the verdict. A compiler shows warnings and **exits 0**; `-Werror` is
how you opt into failure. The analogy the docs leaned on broke at exactly the
point users were complaining about.

So: two dials, named for the two questions.

| | shows | fails the run |
|---|---|---|
| default | errors + warnings | errors only |
| `--no-warnings` | errors only | errors only |
| `--warnings-as-errors` | errors + warnings | errors **and** warnings |
| `--show-fyi` | adds fyi | unchanged — fyi never fails |

`--no-warnings` and `--warnings-as-errors` contradict each other, so the
combination is **rejected** rather than silently resolved — clap's
`conflicts_with` on the binary, an explicit check in the other two launchers,
which is how they already spell `--json`/`--ndjson`.

## One producer

`laterite_ags4_validator::verdict::Verdict` is the only thing in the tree that
answers "did it pass". `exit_code()` is *derived from* `is_valid()` rather than
recomputed, so `is_valid == (exit_code == 0)` holds **by construction** rather
than by test.

This is the same single-producer shape [[cert-trust-v2]] arrived at for the
certificate trust decision, and for the same reason: the formula was previously
written out twice, and agreed only because it was trivial. A *split* formula
copied twice is a divergence with a date on it. [[laterite-ags4-xcheck]] carries
a pair of cases — one warning-pure file, with and without the flag — so a
launcher that silently ignores the opt-in splits against the other two rather
than quietly agreeing with the default.

`Report.is_valid` therefore stops being `count == 0`. A passing file can carry
findings, so `errors` / `warnings` / `fyi` are exposed alongside `count`: one
field was being asked to say both how much was found and whether it mattered.

## Consequences taken deliberately

- **`is_valid` changes meaning on every surface.** Pre-1.0 is the cheapest this
  break will ever be; holding it to 1.0 means shipping the change untested
  against real consumers, in the release where it most wants to be proven.
- **FYI stays opt-in.** It was hidden for volume, and this changes nothing about
  volume.
- **The free `laterite_ags4_validator::is_valid` was renamed `is_clean`.** It
  means "did the run find anything", which used to be the same answer as the
  verdict and is no longer. Two public `is_valid` in one crate, disagreeing on a
  warning-carrying file, is a trap for the next caller. The old name survives as
  a `#[deprecated]` alias naming both replacements — see [[reliquary]].
- **The engine fingerprint had to grow.** `build.rs` hashes a hand-written subset
  of the crate, and `verdict.rs` — which decides more of the answer than any
  other file — would not have been in it.
- **The browser app read the wasm `ok` flag as "zero findings".** With `ok`
  following the verdict, a warnings-only file would have been headlined
  *"Clean — 0 findings"* over a table listing the warning.
- **The Rust facade already disagreed with everything else.** `laterite::ags4::Report::is_valid`
  has always been errors-only, so the published Rust API and the `lat` binary
  gave different answers on the same file. They agree now.

## What this does not change

Error parity with python-ags4. The parity gate compares only `AGS Format Rule N`
keys ([[parity-model]]), no error changed tier, and `compat` keeps its own
python-faithful defaults. See [[O-44]], [[O-45]] and [[O-51]] for the per-site
records.
