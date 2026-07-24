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
  benches: "repo:rust-packages/laterite-ags4-parse/benches/parse.rs"
related: [crate-map, laterite-ags4-core, laterite-ags4-validator, laterite-types, core-perf-baseline, testing-strategy]
sources: []
---
# laterite-ags4-parse

> [!note] **Internal implementation detail** — a workspace leaf crate, not a
> public API. Nothing in the [[laterite]] wheel re-exports it; it is reached
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
to [[laterite-types]]. Anything that would drag in DuckDB, `age`, or an
allocator-heavy dependency belongs above it.

## The two coordinate systems

The leaf carries **both** in a single pass, neither back-derived from the other:

- each record's absolute **byte** offset in the original buffer (what the
  `.ags.idx` certificate indexes — see `O-40`);
- its 1-indexed **line** number (what findings and fixes join on).

`byte_offsets_source_true` is the guard: if decoding ever substituted bytes and
shifted a record start, the flag goes false and the cert path refuses to mint an
index. A certificate that points at the wrong bytes while reporting success is
worse than no certificate.

## Trim policy

Values, units, types and headings come back **RAW (untrimmed)** — the
validator's semantics, because on an unquoted field the surrounding whitespace
*is* the Rule 5 violation and trimming it would hide the finding. Core's lean
projection re-applies its own trims in `from_shared`; see
[[laterite-ags4-core]].

## Parse profiles

`ParseOptions` selects how much of the model to build. All three walk the file
identically — same line spans, same decode, same UTF-8 handling, same AGS3
sniff — and differ only in what they retain:

| profile | retains | used by |
|---|---|---|
| `validating()` | everything, including `raw_lines` | the rule engine |
| `lean()` | no raw lines | the read codec |
| `lean()` + `locate_only` | GROUP records only | the certificate index |

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

**Two implementations remain** (`split_ags_line`, `field_span`), deliberately.
Folding `field_span` would cost its short-circuit — it stops at the field it
wants (48.7 ns) where a full scan is 147.3 ns plus a `Vec`.

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

[[crate-map]] · [[laterite-ags4-core]] · [[laterite-ags4-validator]] · [[laterite-types]] · [[core-perf-baseline]] · [[testing-strategy]]
