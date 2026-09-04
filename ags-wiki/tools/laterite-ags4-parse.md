---
type: tool
title: laterite-ags4-parse
status: drafted
tags: [tool, internal, architecture]
tool_kind: crate
language: rust
artifact: laterite-ags4-parse
ags_editions: []
repo_refs:
  root: "repo:rust-packages/laterite-ags4-parse"
  lib: "repo:rust-packages/laterite-ags4-parse/src/lib.rs"
  scan: "repo:rust-packages/laterite-ags4-parse/src/scan.rs"
  builder: "repo:rust-packages/laterite-ags4-parse/src/builder.rs"
  benches: "repo:rust-packages/laterite-ags4-parse/benches/parse.rs"
related: [crate-map, laterite-ags4-core, laterite-ags4-validator, laterite-ags4-types, laterite-ags4-emit, core-perf-baseline, testing-strategy, dec-parse-cell-representation, dec-emit-streamed-verdict]
sources: []
---
# laterite-ags4-parse

<!-- BEGIN GENERATED: crate-card — DO NOT EDIT BY HAND. Regenerate: uv run --no-project python tools/gen_crate_graph.py -->
> [!note] **Cleared for crates.io** — `laterite-ags4-parse` declares `publish = true`, so it is a public API under semver, not an internal detail. It is versioned on its own line.
> **Used by** — [[laterite]], [[laterite-ags4-censor]], [[laterite-ags4-core]], [[laterite-ags4-diff]], [[laterite-ags4-emit]], [[laterite-ags4-forge]], [[laterite-ags4-merge]], [[laterite-ags4-perf]], [[laterite-ags4-reference]], [[laterite-ags4-tokenizer-wasm]], [[laterite-ags4-trust]], [[laterite-ags4-validator]], [[laterite-ags4-wasm]], [[laterite-ags4-xcheck]], [[laterite-cli]], [[laterite-node]], [[laterite-py]].
<!-- END GENERATED: crate-card -->

> [!note] Nothing in the [[laterite]] wheel re-exports it; it is reached
> through [[laterite-ags4-core]] and [[laterite-ags4-validator]].

## What it is

The **shared AGS4 parse leaf** (#168): one tolerant tokenizer plus one
source-true byte/line/char walk, sitting *below* both
[[laterite-ags4-core]] and [[laterite-ags4-validator]] so the two historical
parsers (core's `ags4_codec`, the validator's `parse`) converge here instead of
drifting apart.

It is the single busiest crate in the tree. **Every read goes through it** — the
validator, the read codec, the certificate index builder, and all four surfaces
(wheel, node, wasm, CLI). A percent here is a percent everywhere, which is why
its benches exist and why it has absorbed most of the perf work in
[[core-perf-baseline]].

## Why it is a leaf

Dependencies are `encoding_rs` (the decode front door) and `memchr` (newline
scan) — **nothing else**. That is a constraint, not an accident: the crate must
stay filesystem-free and wasm-clean, so it returns raw strings and leaves typing
to [[laterite-ags4-types]]. Anything that would drag in DuckDB, `age`, or an
allocator-heavy dependency belongs above it.

## The coordinate systems

The leaf carries them in a single pass, none back-derived from another:

- each record's absolute **byte** offset in the original buffer (what the
  `.ags.idx` certificate indexes — see `O-40`). Group-level offsets
  (`group_records`, `group_byte`, `total_bytes`) exist under every profile;
  the per-row/per-line ones are profile-gated — see *Parse profiles*;
- its 1-indexed **line** number (what findings and fixes join on);
- since [[dec-parse-cell-representation]], every line and cell is also a
  `u32` `Span` into the **retained decoded buffer** (`ParsedFile::text`) — a
  third space, in *decoded-buffer bytes*.

`byte_offsets_source_true` is the guard on the first: if decoding ever
substituted bytes and shifted a record start, the flag goes false and the cert
path refuses to mint an index. A certificate that points at the wrong bytes
while reporting success is worse than no certificate. For non-UTF-8 encodings
and lossy replacement the original-byte and decoded-buffer spaces genuinely
diverge — both are kept, and neither substitutes for the other.

> [!warning] **The units trap.** Three spellings of "span" live here and they do
> not mix: record `byte_offset`s are ORIGINAL bytes, `Span`s are DECODED-buffer
> bytes, and `field_span` returns **char** offsets (a display convenience).
> Reading one with another's offsets is silent corruption, not an error.

## The representation: spans over one retained buffer

`ParsedFile` retains the whole file's decoded text ONCE — line bodies with
terminators dropped (`RawLine::had_crlf` keeps the evidence), each line's
`""`-escaped cells appended once-unescaped as a **fix-up run** directly after
that line's body (fix-ups are *interleaved*, so consecutive line spans are not
contiguous — slice per span, never across spans) — and shares it into each
`ParsedGroup` by refcount, so a group
handed out alone (the three long-lived FFI holders) still resolves its rows.
`RawLine::text` is a `u32` span into that buffer. A DATA row's cells are
spans in their group's **span arena** — one contiguous `Vec<Span>` per
group, with `DataRow` reduced to a slim index into it (`{line, first, n}`,
12 bytes) — so the per-row heap block the pre-M6 layout paid on every DATA
row is gone ([[dec-parse-structure-layout]]; landed by owner decision on
#850 at the measured prize, under the campaign's invasive floor). Every
read still comes back a plain `&str` through the same seam
(`ParsedGroup::cell`, its row-relative sibling `value_at`, `row_spans` for
the raw slice, `ParsedFile::line_text`); `padded_row_strings` is the one
owning accessor, for the matrix-building emitters. A `DataRow`'s arena
indices are group-local: resolve a row only against the group that
produced it.
The buffer is an `Arc<String>` rather than the design page's literal
`Arc<str>`: the `Arc<str>` materialisation *copies* the buffer, and the M4
spike measured that whole-file transient sitting exactly at the operation
peak — adopting the built `String` zero-copy is what cleared the campaign's
invasive floor at the 25 MB rung (the record is on #838).

## The builder: a ParsedFile from a writer's own records

`builder::ParsedFileBuilder` is the leaf's second way to obtain a
`ParsedFile` — not by parsing, but by **assembly from a byte-authoring
writer's records** ([[dec-emit-streamed-verdict]]). An emitter that authors
every byte already knows everything the walk would re-derive; the builder
lets it say so: the writer records each row it writes (tag, body span, cell
spans into its own output), and `finish` assembles the same retained
structure the walk builds, over the written bytes adopted as the retained
buffer, returning the emitted length beside the file so a caller can take
the bytes back zero-copy (truncate the refcount-1 buffer there).

It lives in THIS crate because the pieces it must get right are the walk's
own — first-seen-wins group identity, descriptor overwrite on a redeclared
code, the `""` unescape into a fix-up region, the `u32` span-space guard —
and because `ParsedGroup`'s arena and `DataRow`'s index are private by
design. The result matches `parse_bytes` of the same bytes under the
validating profile on every field the rule engine reads; the buffers differ
only in LAYOUT (the builder keeps terminators and appends fix-ups after the
end, the walk drops terminators and interleaves), which spans absorb. The
emit crate's differential test holds that equality permanently — over
crafted shapes here, and its own recorded writer end-to-end.

## Trim policy

Values, units, types and headings come back **RAW (untrimmed)** — the
validator's semantics, because on an unquoted field the surrounding whitespace
*is* the Rule 5 violation and trimming it would hide the finding. The span
representation changes nothing here: a span bounds the raw value, and the
fix-up region unescapes without trimming. Core's lean projection re-applies
its own trims in `from_shared`; see [[laterite-ags4-core]].

## Parse profiles

The old `retain_raw_lines` knob is **dead** — a raw line is a span over a
buffer the parse keeps anyway, so the overlay collapsed into the base model
([[dec-parse-cell-representation]]). What remains is decode policy, the
explicit opt-ins (`strict_structure`, `locate_only`), and — since
[[dec-parse-structure-layout]] — **profile-dependent field presence**: the
per-row/per-line SOURCE byte offsets (`ParsedGroup::row_byte_offsets`,
`ParsedFile::line_byte_offsets`, read via `row_byte_offset(i)` /
`line_byte_offset(i)`) exist only under a profile that sets
`retain_source_offsets`. The rule engine reads none of them, so
`validating()` drops them; the group-level coordinates (`group_byte`,
`group_records`, `total_bytes`, `byte_offsets_source_true`) are NOT gated —
every profile records them, which is why the cert index can mint from any
parse. Every profile walks the file identically — same line spans, same
decode, same UTF-8 handling, same AGS3 sniff:

| profile | is | used by |
|---|---|---|
| `validating()` | lossy decode; per-row/per-line source offsets dropped | the rule engine |
| `lean()` | reject invalid bytes; source offsets retained | the read codec (+ `strict_structure`) |
| `lean()` + `locate_only` | retains no text: GROUP records only | the certificate index |

A consumer that needs per-row/per-line source-byte coordinates must parse
under a profile that retains them — the leaf enumerates its consumers
rather than abstracting for them ([[dec-parse-structure-layout]]'s
decision; at land the per-row/per-line offsets had no readers outside the
leaf's own tests).

`locate_only` exists because the index reads `group_records`, `group_order` and
`total_bytes` and nothing else, but used to pay for the whole row model and
discard it — a full parse to keep ~123 records. The equivalence is pinned in
`tests/locate_only.rs`: records, order, byte offsets, line numbers, BOM state
and the source-true flag must be **identical** to the full walk, and the same
inputs must be rejected for the same reasons.

> [!warning] `locate_only` returns `groups` with empty headings/units/types/rows.
> It is a **locator**, not a read profile. Reading data out of it silently yields
> nothing rather than failing.

## The line grammar: one scanner, two policies

The AGS4 line grammar (quote-wrapped fields, `""` escapes, comma separators) was
implemented **three times** here — `split_ags_line`, `field_span` and the
now-retired `tokenize_spans` — which is how they came to disagree on five
behaviours (empty-line field count, unquoted trimming, field indexing, whether
`""` stays escaped, and what an unterminated quote yields).

`scan::scan_line` is the shared core they sit on. Two decisions make that work:

- **Bytes, not code points.** `"` and `,` are ASCII and UTF-8 never places an
  ASCII byte inside a multi-byte sequence, so a byte scan is exactly equivalent
  — and the validator's per-line walk stops paying for code-point offsets only
  the browser needs. The browser's conversion happens in the wasm adapter that
  actually has the requirement (see [[laterite-ags4-wasm]]).
- **The value policy is a parameter, not a fork.** What was duplicated is the
  *state machine*; what legitimately differs is how a token's inner value
  resolves. `RAW` judges the bytes (the validator's need), `DISPLAY` trims for
  highlighting (the browser's). Collapsing them would let a UI concern define
  what the validator calls a value. Measured, the policy is **free** — 147.8 vs
  147.3 ns/line.

`RawField` is a strict superset of the retired `AgsSpan`: the same four bounds
plus `quoted`, `has_escape` and `had_comma`. Callers that used to infer structure
from string suffixes now read it directly.

> [!note] One divergence is irreducible: a borrowed slice **cannot unescape**,
> because `""`→`"` yields a value shorter than its source. `has_escape` flags it,
> so a caller can borrow the common case and allocate only for the rare one.

**Two state machines remain** (`split_ags_line`'s, `field_span`'s),
deliberately. Folding `field_span` would cost its short-circuit — it stops at
the field it wants where a full scan pays for the line plus a `Vec`.

`split_ags_line` itself no longer allocates its own machine: the M4 rewrite
extracted it to **`split_ags_line_spans`** (byte bounds + a `has_escape` flag
per field), and the owned splitter is now its adapter — one machine, agreement
by construction. The parser's DATA cells are built from the same span core.
This mattered because `scan_line` could NOT serve that role: the tolerant
reader and the scanner genuinely disagree on a mid-field quote in an unquoted
field (`x"y",z` — the scanner opens a quoted section, `split_ags_line` takes
the bytes verbatim), so building cells on `scan_line` would have changed
validated values. The tolerant grammar and the display/judging grammar are
different grammars, and each keeps its own machine.

## Line terminators

`\r\n` is the only AGS4-conforming terminator (Rule 2a), but a lone `\n` or lone
`\r` is a genuine split point here, reported as improper rather than swallowed
(#422). An embedded CR/LF *inside a quoted field* is **not** a terminator — it
stays in the body for Rule 6 to flag (`O-2`).

## Non-UTF-8

`InvalidUtf8::Reject` hard-fails; `LossyReplace` substitutes `U+FFFD` and carries
on. The split is deliberate and ratified as `O-46`: the lean read path rejects
non-UTF-8 (a data reader must not silently invent characters), while the
validator decodes lossily so its rule engine can still report *every* problem
rather than stopping at the first (`O-32`).

## Where it lives

`repo:rust-packages/laterite-ags4-parse` — `src/lib.rs` (the walk, the legacy
tokenizers, the profiles), `src/scan.rs` (the shared scanner + value policies).

Tests: `tests/display_spans.rs` (the display tokenizer contract, proptest),
`tests/locate_only.rs` (locator equivalence), `tests/byte_walk.rs`,
`tests/line_split.rs`, `tests/dep_graph.rs` (the leaf's dependency floor —
guards the wasm-clean constraint).

Benches: `benches/parse.rs`, split into whole-file (`parse_bytes`, with
throughput so the number is comparable across sizes) and per-line tiers. The
per-line tier matters because a tokenizer regression is invisible at file level
until it is already large.

## Related

[[crate-map]] · [[laterite-ags4-core]] · [[laterite-ags4-validator]] · [[laterite-ags4-types]] · [[core-perf-baseline]] · [[testing-strategy]]
