---
type: decision
title: "the parse leaf's cells become spans over one retained decoded buffer (queue M4's shape)"
status: accepted
tags: [design, decision]
decided: 2026-08-31
supersedes: []
from_gap: []
related: [laterite-ags4-parse, perf-campaign, core-perf-baseline, dec-emit-cell-representation, crate-map]
sources: []
---

# the parse leaf's cells become spans over one retained decoded buffer

The deliberate sibling of [[dec-emit-cell-representation]]: the same
"priced, then invasive representation rewrite" decision on the read side of
the pipeline. The prices live where the campaign keeps them — the M4 row of
[[perf-campaign]]'s memory queue and the fix ticket minted from it — by the
same rule as the sibling: no tracked page carries a figure nothing
recomputes.

## Context

`laterite-ags4-parse` owns the text it parses, one `String` at a time:
`DataRow.values` is a `Vec<String>` (one owned, unescaped cell each) and,
under the `validating()` profile, `RawLine.text` re-owns every full line —
even when the UTF-8 decode borrowed, so a line's bytes are resident twice
(`repo:rust-packages/laterite-ags4-parse/src/lib.rs` marks the
`text.into_owned()` site as "the one site that genuinely needs ownership").
Every read-shaped operation on every surface sits on top of this hold:
validate, the typed read, and every write door's input half. The memory
queue's M4 row carries the attribution (dhat: ~one block per cell,
requested-live at the parse peak) and the prize ceiling; the same rewrite
was priced and declined on the *time* axis (the "Priced, declined" table's
`raw_lines` row, was #112), and rule 12 re-priced it on the memory axis —
landing M4 collects the declined time win as a rider, and neither verdict
carries to the other's axis.

Two pieces of history bound the design space:

- **The tombstone.** `AgsSpan` + `tokenize_spans` were retired as a third
  hand-written implementation of the line grammar, and the tombstone
  (`repo:rust-packages/laterite-ags4-parse/src/lib.rs`, the
  `tokenize_spans` note) records *why the last span type died*: display
  consumers needed an owned `text` copy of every field, so the spans grew
  owned copies and stopped being spans. Any new span representation must
  answer that failure structurally, not by convention.
- **The precedent.** `scan.rs::RawField` is the surviving span idiom in the
  same crate — pure offsets plus `quoted`/`has_escape`/`had_comma` flags,
  with a `ValuePolicy` deciding what a consumer reads
  (`repo:rust-packages/laterite-ags4-parse/src/scan.rs`). M4 generalises
  that idiom from a per-line scanner to the retained file.

And one lesson bounds the *process*: M1 and M5 both showed that a ceiling
priced from a mechanism read is not the delta the paying instrument returns
(M1: the OS never returned the released pages; M5: the "sum-like" copies
were being reused, so removing them moved ~nothing). An invasive rewrite
does not get to discover that after the API migration — so the mint is
gated on a measured spike (below).

## Options considered

1. **`ParsedFile<'a>` borrowing the caller's buffer.** The zero-copy ideal,
   ruled out by the holders: three host-language objects hold a
   `ParsedFile` across arbitrary foreign calls — the PyO3 `Reading`
   pyclass, the napi `Reading`, and the wasm `ParsedDataset` — and a
   lifetime-bearing type inside them means self-referential structs across
   an FFI boundary, a trick this workspace deliberately avoids.
2. **Spans over the raw source only, `Cow` on read.** Cheapest parse; but a
   cell containing `""` escapes cannot be a pure span (unescaping changes
   bytes), so every read grows a `Cow` in its signature and every repeated
   reader — the rule families, the Arrow build, emit — re-pays the
   unescape. This is the `AgsSpan` failure mode arriving by another door.
3. **Offset spans into one retained decoded buffer, escapes fixed up once
   at parse.** Chosen; the shape below.
4. **Staged migration behind a dual-representation shim.** Ruled out as a
   migration strategy regardless of shape: a window where both forms
   coexist is reliquary bait and a self-consistent-revert hazard, and the
   validator's re-export of the parse types gives the one-arc migration a
   choke point that makes it reviewable.

## Decision

**Shape.** `ParsedFile` retains the whole file's decoded text once, in one
atomically-refcounted buffer shared into each `ParsedGroup`. (Written as
`Arc<str>` when this page was decided; the spike measured `Arc<str>`'s
materialisation — a whole-file *copy* — sitting exactly at the operation
peak, and the landed form is `Arc<String>`, which adopts the built buffer
zero-copy. Every other property argued here — one buffer, refcount sharing,
plain `&str` reads, no lifetimes — is unchanged; the A/B/A record is on
#838.) `RawLine.text` and
`DataRow.values` become `u32` span pairs into that buffer. A cell the
tokenizer found escapes in is unescaped **once, at parse**, into a fix-up
tail appended to the same buffer, and its span points there — so there is
one span space, every read returns a plain `&str`, and the tombstone's
display-consumer problem is answered structurally. The existing accessors
(`ParsedGroup::cell`, `col`) keep their signatures, borrowing through
`&self` into the shared buffer; consumer churn concentrates at the fields'
types, not at every read site.

**Two coordinate systems, both kept.** The decode becomes whole-buffer
(the per-line `decode_line` doc already records that per-line and
whole-buffer decode agree for the accepted encodings). Span offsets index
the *decoded* buffer. The existing `byte_offset` fields on `RawLine`,
`DataRow` and `GroupRecord` stay **original-byte** exactly as today — they
feed the `.ags.idx` certificate under `byte_offsets_source_true`, and for
non-UTF-8 encodings the two coordinate systems genuinely diverge. The page
that documents the leaf ([[laterite-ags4-parse]]) already has a "two
coordinate systems" section; this decision widens it rather than
replacing it.

**Scope.** The representation covers the per-cell holds — `DataRow.values`
and `RawLine.text`. The descriptor rows (HEADING/UNIT/TYPE) **stay owned
`Vec<String>` deliberately**: one line per group, nowhere near the cell hold
this rewrite exists to kill, and not worth the churn of a third span
consumer. Plus the one collapse the representation makes honest:
`retain_raw_lines` dies (a raw line as a span over a buffer that is
retained anyway costs ~a span), so the `lean()`/`validating()` profile
split reduces to encoding policy and the explicit opt-ins
(`strict_structure`, `locate_only`). Out of scope, recorded here so they
stay deliberate: migrating `split_ags_line` callers onto span reads (a
future surgery that moves no measured cell today), and anything on the
typed-build side — the Arrow-cast direction was spiked and **rejected**
(slower, and it dragged the cast kernels into the wasm binary; the
rejection is pinned beside `build_column` in
`repo:rust-packages/laterite-ags4-types/src/arrow_cols.rs`), and the
attribution pass found the typed build holds ~nothing at peak.

**Migration.** One arc: the fields change once and every consumer adapts in
the same stack, most of them through the validator's re-export
(`repo:rust-packages/laterite-ags4-validator/src/parse.rs`). No shim ever
exists. The break lands on published crates (parse, validator, core at
minimum); the nightly engine cut prices the bumps and the coherence gate
carries the pins — no hand-managed version choreography.

**Mint conditions (the spike gate).** One fix ticket, minted from the
queue's M4 row one-at-a-time as usual, whose work order is
**spike → verdict → conditional land**:

1. The spike is a rough span rewrite on a `spike/` branch — parse crate
   plus the minimal shims that make the ladder run, no API polish, no
   surface migration.
2. It runs the committed Python-lane instrument's fresh-child cells for
   **validate** and **read_typed** at the 25 MB and 100 MB rungs,
   A/B/A-bracketed on a quiet machine, plus the paired criterion
   `parse_bytes` / `check_parsed` benches.
3. **GO** requires the campaign's invasive floor (rule 10 of
   [[perf-campaign]]) cleared on *both* operations' peaks at *both* rungs,
   with the time benches within noise or better — the span indirection may
   not reopen the closed validator band.
4. Under the bar, the ticket records an M1-style decline with the numbers,
   the queue row moves to declined, and this page's status flips to
   `rejected` with the record cited. The full migration is scoped only on
   a GO.

## Why

- **`Arc<str>` over parent-side accessors** because it keeps
  `cell()`/`col()` and the `raw_lines` iteration shapes intact: the one-arc
  migration stays a review of *field types and construction*, not a rewrite
  of every rule family and QA tool. The Arc costs pointer-sized overhead
  per group against a buffer the design retains regardless.
- **Fix-up tail over `Cow`-on-read** because escapes are rare and reads are
  hot and repeated; paying the unescape once at parse keeps the public
  surface `&str` and prices the rarity where it belongs.
- **Profiles collapse inside scope** because the knob exists *because of*
  the old representation; keeping `retain_raw_lines` after spans would
  preserve a lie on an API this decision is already breaking.
- **Spike before migration** because this campaign has now twice paid for
  believing a priced ceiling ahead of the paying instrument, and the third
  time would be on its most invasive candidate. The floors are the
  campaign's own (cited by rule, not restated here), so the bar cannot
  drift apart from the ledger that owns it.

**Outcome (2026-08-31).** The mint condition was met and the arc landed:
the spike's A/B/A cleared rule 10's invasive floor on every cell — validate
−29.0% / read_typed −28.4% of peak at the 25 MB rung, −36.5% / −35.5% at
100 MB, A-legs bracketing within 0.05% — and the paired criterion benches
took `parse_bytes` down 43–47% while `check_parsed` stayed inside the 5%
resolution floor. The verdict record with both findings (the `Arc<str>`
transient above, and why `scan_line` could not serve as the span tokenizer)
lives on #838; the ledger's M4 row carries the claim.

## Consequences

- Commits the leaf to **two documented coordinate systems**: original-byte
  offsets for the certificate and locator, decoded-buffer offsets for
  spans. The arc must keep `byte_offsets_source_true` semantics exactly
  (whole-buffer decode changes *when* replacement is detected, not what it
  means), and `Reject`-mode invalid-UTF-8 errors must still name the
  offending line, which now takes a scan rather than falling out of the
  per-line loop.
- The existing `field_span` helper returns **char** spans (a display
  convenience); the new representation's spans are byte offsets. The arc
  must not blur the units — the page documenting the leaf gets the trap in
  writing.
- Free riders, collected without extra scope: the declined time-axis
  `raw_lines` win (its "revisit when" condition — a change already
  rewriting `RawLine` — is met by this decision), forge's per-line clone
  lookups and the wasm validate line map become borrow reads, and the
  doubled residency of every line dies.
- Rules out: any dual-representation shim; re-proposing the Arrow-cast
  typed build; `retain_raw_lines` surviving in any form.
- The arc updates [[laterite-ags4-parse]]'s affected sections (two
  coordinate systems, trim policy — values stay RAW/untrimmed, parse
  profiles, the line-grammar section) in the same stack, and the ledger's
  M4 row points here so the queue knows the design-page precondition is
  met.

## Related

[[perf-campaign]] · [[core-perf-baseline]] · [[dec-emit-cell-representation]] ·
[[laterite-ags4-parse]] · [[crate-map]] ·
repo: rust-packages/laterite-ags4-parse/src/lib.rs ·
repo: rust-packages/laterite-ags4-parse/src/scan.rs ·
repo: rust-packages/laterite-ags4-validator/src/parse.rs ·
repo: rust-packages/laterite-py/src/lib.rs
