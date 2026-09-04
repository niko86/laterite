---
type: decision
title: "the parse leaf's retained structure flattens — a span arena + profile-gated source-byte fields (queue M6's shape)"
status: accepted
tags: [design, decision]
decided: 2026-09-01
supersedes: []
from_gap: []
related: [laterite-ags4-parse, perf-campaign, dec-parse-cell-representation, dec-emit-streamed-verdict, crate-map]
sources: []
---

# the parse leaf's retained structure flattens — a span arena + profile-gated source-byte fields

> [!note] **Gate outcome (2026-09-01): the spike MISSED this page's own
> floor, and the owner landed the shape anyway at the measured prize.** The
> flattening was built exactly as decided below and measured on the lane
> instrument: **−6.9/−6.2% (25 MB) and −13.1/−10.7% (100 MB)** of the
> validate/read_typed peaks — under rule 10's 20% floor on all four cells
> (the page briefly stood `rejected` on that verdict). The owner's decision,
> recorded on #850: the measured ~10–13% plus the time rider
> (`parse_bytes` −9…−15%, `check_parsed` unchanged) is worth the remaining
> effort with the spike already built and contract-green — a dated waiver
> of the floor for this row, **not** a precedent that ~10% clears an
> invasive gate. The full A/B/A record — protocol, bracket table, and the
> requested-vs-touched-pages correction to this page's pricing (the
> requested-bytes instrument OVERcounted here, the inverse of the M4-era
> undercount) — is on **#850**; the ledger's M6 row ([[perf-campaign]])
> carries the landed numbers. With the layout landed,
> [[dec-emit-streamed-verdict]]'s "re-price on the post-layout lane"
> precondition is live again.

The direct successor of [[dec-parse-cell-representation]]: M4 replaced the
per-cell `String`s with spans over one retained decoded buffer, and this
decision flattens what M4 left standing — the per-row and per-line
**structure** those spans live in. The prices live where the campaign keeps
them (the M6 row of [[perf-campaign]]'s memory queue and the record on
#848); no tracked page carries a figure nothing recomputes.

## Context

Post-M4, a retained `ParsedFile` is one shared decoded buffer plus
structure: a heap-allocated `Vec<Span>` **per DATA row**, a `DataRow`
record per row carrying a source-byte offset, and a `RawLine` record per
physical line carrying another. The #848 attribution probe priced this
structure as most of the retained hold beyond the buffer itself — and
found that the write door pays it **twice** (the caller's retained parse
plus the emit door's validating parse-back; the full record, both
instruments, is on #848). Every `ParsedFile` holder pays once: the read
handle, validate, every write door's input half, and — once
[[dec-emit-streamed-verdict]] lands — the write door's constructed
verdict, whose residual cost *is* this structure.

Two facts bound the design space:

- **The validator reads none of the source-byte fields.** The #848
  inventory (on the ticket) walked every field access: `check_parsed` and
  `compute_fixes` read spans, line numbers, `had_crlf`, `has_bom` and the
  descriptor rows — never `DataRow.byte_offset`, `RawLine.byte_offset`,
  `group_byte`, `group_records`, `total_bytes` or
  `byte_offsets_source_true`. Those fields have exactly three consumers:
  the `.ags.idx` certificate index
  (`repo:rust-packages/laterite-ags4-core/src/index.rs`), forge's editor
  (`repo:rust-packages/laterite-ags4-forge/src/edit.rs`), and one xcheck
  helper. For them the source-byte coordinate system is load-bearing
  ([[dec-parse-cell-representation]]'s "two coordinate systems" doctrine
  is untouched here); for the validator it is pure weight.
- **The accessor seam already exists.** #844/#846 moved consumers onto
  `value_at` / `padded_row_strings` / `cell`, so the parse leaf owns the
  span/buffer pairing and the retained layout can change under its
  accessors — the migration is a review of field types and construction,
  not of every read site. This is the same property that made M4's one-arc
  migration reviewable.

And the process lesson is #848's own: a slice's at-peak *share* is not its
contribution — the write peak stood on two co-peak shoulders that backed
each other up, and only removing both moved the instrument. This row's
prize is denominated the only honest way: a measured spike on the paying
instrument, per-row heap blocks being exactly the shape whose RSS cost the
requested-bytes diagnosis instrument undercounts (allocator rounding and
per-block overhead on millions of small blocks).

## Options considered

1. **Status quo** — per-row `Vec<Span>` heap blocks, source-byte fields
   unconditional. This is what #848 priced; the write-door verdict cannot
   clear the campaign's invasive floor on top of it, and every retained
   parse carries validator-unread weight.
2. **Span arena + slim row index; source-byte fields profile-gated.**
   Chosen; the shape below.
3. **Option 2 plus packed spans** (`u32` offset + `u16` length). Rejected:
   it buys a marginal slice at the cost of a length-cap invariant on every
   cell forever, nothing in the gate arithmetic needs it, and a declined
   spike should implicate the idea being tested, not a rider.
4. **A trait/view input for `check_parsed`** so the validator stops
   depending on the concrete layout. Rejected as scope creep: the
   validator's re-export of the parse types is the migration choke point
   that made M4 reviewable; abstracting the input contract is a rules
   -engine change this row's evidence does not motivate.

## Decision

**Shape.** Per group, the per-row `Vec<Span>` heap blocks collapse into
one **span arena** (a single `Vec<Span>` per group) with a slim per-row
index into it. The `DataRow` and `RawLine` **source-byte fields become
profile-gated**: the `validating()` profile drops them (the validator
never reads them), while the certify/read path keeps them source-true
exactly as today — byte-offset retention *is* decode policy, so the knob
rides the **existing** profile mechanism from
[[dec-parse-cell-representation]]'s collapse; no new axis is added.
`raw_lines` slims accordingly. Spans stay `u32` pairs; the buffer, the
fix-up-tail escape handling, the accessors' signatures and the two
coordinate systems are all M4's, unchanged.

**Consumers.** The three source-byte consumers (cert index, forge edit,
xcheck) parse under a profile that retains what they read; the design
enumerates them rather than abstracting for them. `check_parsed`,
`compute_fixes` and every accessor consumer are layout-blind through the
seam.

**Migration.** One arc through the validator's re-export choke point, no
shim — M4's migration doctrine verbatim. The break lands on published
crates (parse at minimum) under the pre-1.0 minor-for-breaking
convention; the coherence gate carries the pins.

**Mint conditions (the spike gate).** One fix ticket, minted from the
queue's M6 row, work order **spike → verdict → conditional land**, gated
exactly as [[dec-parse-cell-representation]]'s was:

1. A rough flattening on a `spike/` branch — parse crate plus minimal
   shims, no API polish, no surface migration.
2. The committed Python-lane instrument's fresh-child cells for
   **validate** and **read_typed** at the 25 MB and 100 MB rungs,
   A/B/A-bracketed on a quiet machine, plus the paired criterion
   `parse_bytes` / `check_parsed` benches.
3. **GO** requires the campaign's invasive floor (rule 10 of
   [[perf-campaign]]) cleared on *both* operations' peaks at *both*
   rungs, with the time benches within noise or better.
4. Under the bar: an M1-style decline in absolute terms, the queue row
   moves to declined, and this page flips to `rejected` with the record
   cited. [[dec-emit-streamed-verdict]] is conditional on this gate and
   its row is re-priced only on a GO.

## Why

- **Arena over per-row vecs** because the per-row heap block is the cost:
  one small allocation per DATA row, millions per big file, each carrying
  allocator rounding and block metadata the lane's instrument pays and
  the diagnosis instrument does not show. The arena keeps the same span
  content contiguous per group.
- **Profile-gating over unconditional slimming** because the certificate's
  source-byte offsets are load-bearing (`byte_offsets_source_true` is a
  cert contract), and over a new knob because three consumers do not
  justify multiplying the profile matrix.
- **No packing** (option 3) for the reason given there: floors, not
  maximal shrinkage, decide what lands.
- **Spike before migration**, still: the campaign has paid three times now
  for priced ceilings ahead of the paying instrument (M1, M5, and #848's
  own co-peak correction), and this row's estimates sit close enough to
  the gate that the honest answer is the measurement.

## Consequences

- The write-door row ([[dec-emit-streamed-verdict]], queue M2) is
  **sequenced behind this one**: its verdict structure inherits whatever
  layout this row lands, and its ticket is minted only after a re-price
  on the post-layout lane. This is the queue's one-candidate-at-a-time
  rule applied to a split mechanism, recorded here so the sequencing is a
  decision, not drift.
- Commits the leaf to profile-dependent field *presence* (not just decode
  policy): a consumer that needs source-byte offsets must parse under a
  profile that retains them, and the leaf's page documents which profile
  carries what.
- Rules out: packed spans; a new retention knob outside the profiles; any
  dual-layout shim.
- The arc updates [[laterite-ags4-parse]]'s affected sections (profiles,
  the retained-structure description) in the same stack, and the ledger's
  M6 row points here so the queue knows the design-page precondition is
  met.

## Related

[[perf-campaign]] · [[dec-parse-cell-representation]] ·
[[dec-emit-streamed-verdict]] · [[laterite-ags4-parse]] · [[crate-map]] ·
repo: rust-packages/laterite-ags4-parse/src/lib.rs ·
repo: rust-packages/laterite-ags4-core/src/index.rs ·
repo: rust-packages/laterite-ags4-forge/src/edit.rs ·
repo: rust-packages/laterite-ags4-validator/src/parse.rs
