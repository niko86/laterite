---
type: decision
title: "the codec's read projection goes span-backed — AgsGroup.rows dies as a field, accessors + copy-on-write replace it (queue M8's shape)"
status: proposed
tags: [design, decision]
decided: ""
supersedes: []
from_gap: []
related: [perf-campaign, dec-parse-structure-layout, dec-parse-cell-representation, laterite-ags4-core, laterite-crate, crate-map]
sources: []
---

# the codec's read projection goes span-backed — `AgsGroup.rows` dies as a field, accessors + copy-on-write replace it

The successor to [[dec-parse-cell-representation]] (M4) and
[[dec-parse-structure-layout]] (M6) one layer up: those two slimmed what a
retained **parse** holds, and this decision stops the string-read codec
from immediately re-inflating it. The prices live where the campaign
keeps them (the M8 row of [[perf-campaign]]'s memory queue and the
diagnosis record on #893); no tracked page carries a figure nothing
recomputes.

## Context

Core's string-read door (`read_ags4_with` →
`from_shared`, `rust-packages/laterite-ags4-core/src/ags4_codec.rs`)
converts the whole span `ParsedFile` into `ParsedAgs4` **eagerly**: every
group's rows re-materialise as `Vec<HashMap<Arc<str>, String>>` — one
owned `String` per cell plus per-row map overhead — before any consumer
has asked for a single cell. The #893 diagnosis priced that slab by
variant children, alone and paired: it is **the entire prize** of the M8
queue row (−62…−65% of the read door's whole peak at every gate rung,
with a wall-time rider; the third copy — the CLI's projection — measured
as co-peak shadow), and the span-sourcing child was **byte-identical** to
the shipped door at every rung.

Three facts bound the design space:

- **Every read consumer projects positionally.** The #893 inventory
  walked all of them: the three `lat` read doors (the Rust binary's
  `commands/read.rs`, the wheel's and the npm package's `read_groups_raw`
  — byte-faithful to each other **by contract**), excel's `to_excel`
  worksheet walk, the compliance QA bin, and `effective_dict`'s DICT walk
  all iterate `headings` and do `row.get(heading)` per cell — the by-name
  map is built eagerly for every row in the file and then used as a
  positional intermediate. Name→column resolves once per group; nothing
  needs a per-row map.
- **Two consumers are not plain readers.** The crates.io facade
  (`rust-packages/laterite/src/ags4/document.rs`) holds a `ParsedAgs4`
  long-lived and **mutates** it (`set_cell`, `push_row`,
  `remove_group`) before emitting positionally; and excel's `from_excel`
  uses `AgsGroup` as a **builder** — it constructs groups that never came
  from a parse and hands them to the same positional emit leg. Any new
  representation must keep an owned, mutable construction path.
- **The strictness contract is part of the door.** `from_shared` is where
  `DuplicateHeadings` policy resolves and where a ragged-long row raises
  `ExcessFields` **at read time** (#776). Neither check needs
  materialisation — headings resolution is per-group metadata, and row
  arity is `n_values()` against the heading count, readable off the spans
  without allocating — but both must stay eager: deferring a refusal to
  first access would let a certified read hold data it would later refuse.

And the process fact: unlike M6, the spike gate is already passed — the
#893 children measured the mechanism's removal over the invasive floor
three times over, byte-identical, on the paying instrument. The remaining
risk is the **contract migration**, not the prize.

## Options considered

1. **Status quo** — the eager whole-file map slab. This is what #893
   priced; every string-read consumer on every surface pays ~7× of input
   it never asked for.
2. **Positional eager rows** (`Vec<Vec<String>>` instead of maps).
   Rejected: kills the per-row map overhead but keeps a `String` per cell
   — the M4-era lesson says that is most of the weight, and no #893 child
   measured this shape, so its prize is a guess. The campaign has paid
   three times for unmeasured ceilings (M1, M5, #848's correction).
3. **Span-backed `AgsGroup` + accessors + copy-on-write for mutation.**
   Chosen; the shape below.
4. **CLI-only bypass** (the spike's shape shipped as the Rust binary's
   private path; core unchanged). Rejected: the wheel's and npm package's
   `lat` pay the identical slab through the same door, so the
   byte-faithful-by-contract triple would fork into one fast and two heavy
   implementations of the same verb — exactly the hand-synced drift
   laterite-dev#530 retired — and excel and the facade keep paying.

## Decision

**Shape.** `AgsGroup` keeps its resolved metadata exactly as today —
trimmed `code`, policy-resolved `headings`, padded `units`/`types` — and
its **rows become span-backed**: the group holds the parse leaf's
per-group arena (the M6 layout: span arena + slim row index) plus the
shared decoded buffer by refcount, and serves cells through accessors —
row count, positional cell (trimmed on access, `""`-padded past the
row's tail, the `from_shared` contract verbatim), a positional
padded-row iterator, and by-name access that resolves the heading to a
column index once per group. The public `rows` **field** dies; no
whole-file owned materialisation exists on any read path.

**Eager policy, lazy strings.** `read_ags4*` still resolves
`DuplicateHeadings` and still refuses `ExcessFields` at read time — the
arity walk runs over the spans during conversion, allocating nothing.
What was the conversion's `String`-per-cell build becomes the accessors'
trim-on-read against the retained buffer (the #893 `slab` child proved
the projection byte-identical).

**Mutation and construction (the two non-readers).** A group mutates by
**copy-on-write**: the facade's first `set_cell`/`push_row` on a group
materialises that one group into an owned positional representation and
the group serves reads from the overlay thereafter — one group's worth,
only on the paths that asked. Excel's `from_excel` (and any other
builder) constructs that same owned representation directly; the
map-shaped builder dies with the field.

**Migration.** One arc through core's `ags4_codec` with every consumer
moved onto the accessors in the same stack — the three `lat` doors, both
excel directions, the facade (whose **public API does not change**: its
`Group`/`Row` types wrap the accessors, and its `Row.cell(heading)`
resolves through the group's heading index), `effective_dict`, the
compliance bin, and `index::parse_group_slice` (whose one-group slice
just stops pre-materialising). No dual representation ships except the
copy-on-write overlay itself. The break lands on published crates (core
at minimum) under the pre-1.0 minor-for-breaking convention;
`check_semver` prices it against the crates.io baseline as usual.

**Mint conditions (the land gate).** One fix ticket, minted from the
queue's M8 row once this page is accepted. The spike-shaped question is
already answered (#893), so the gate is on the land:

1. `lat read` output pinned **byte-identical across all three programs**
   on the pinned rungs (the shared render writers already hold the
   format; the pin is the door's cells).
2. The land A/B/A on the committed CLI-lane instrument at the gate
   rungs: **GO** requires rule 10's invasive floor cleared on the read
   door's peak at 100 and 265 MB — the #893 children put the ceiling at
   ~3× the floor, so a land that misses it has built the wrong shape.
3. Excel round-trip and facade behaviour contract-green, including
   mutation-after-read (the copy-on-write seam) and the read-time
   `ExcessFields`/`DuplicateHeadings` refusals, held by tests.
4. Refreshed lane cells in the committed results file at the landed
   tree; the node/py `read_groups_raw` doors inherit the fix through the
   shared door and their surfaces' cells ride the next matrix run.
5. Under the bar: an M1-style decline in absolute terms, the queue row
   moves back with the record, and this page flips to `rejected`.

## Why

- **Accessors over a new eager shape** because every consumer is already
  positional at its boundary, and the parse leaf's own #844 seam
  (`value_at` / `padded_row_strings`) is the proven pattern: the layout
  changed twice under it (M4, M6) with the migration a review of
  construction sites, not of every read.
- **Copy-on-write over keeping any eager slab** because exactly one
  consumer mutates, it mutates per group, and it should pay per group —
  the read paths that made M8 the queue's biggest row mutate nothing.
- **Eager policy checks** because #776's refusal semantics are the
  door's contract, and they cost nothing to keep: the arity walk reads
  span counts, not cells.
- **No CLI fork** (option 4) because three programs answer to `lat` by
  contract, not by construction — the door is shared so the fix is
  shared, or the contract rots one surface at a time.

## Consequences

- `AgsGroup.rows` (the field) and the map-of-owned-strings shape leave
  core's public API — a breaking change on `laterite-ags4-core`, priced
  by the semver gate; the facade's published API is explicitly held
  stable across the arc.
- The read codec commits to the parse leaf's buffer lifetime: a held
  `ParsedAgs4` keeps the decoded buffer alive (it already effectively
  did — the strings were copies of it; now the copies are gone). The
  sliced single-group path keeps its bounded hold.
- Rules out: a dual eager/lazy read API, a per-row map anywhere on a
  read path, and any surface-private projection of the shared door.
- The arc updates [[laterite-ags4-core]]'s codec section and the ledger's
  M8 row in the same stack, so the queue knows the design-page
  precondition is met.

## Related

[[perf-campaign]] · [[dec-parse-cell-representation]] ·
[[dec-parse-structure-layout]] · [[laterite-ags4-core]] ·
[[laterite-crate]] · [[crate-map]] ·
repo: rust-packages/laterite-ags4-core/src/ags4_codec.rs ·
repo: rust-packages/laterite-ags4-core/src/index.rs ·
repo: rust-packages/laterite-cli/src/commands/read.rs ·
repo: rust-packages/laterite-ags4-excel/src/lib.rs ·
repo: rust-packages/laterite/src/ags4/document.rs
