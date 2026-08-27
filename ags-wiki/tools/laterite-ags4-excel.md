---
type: tool
title: laterite-ags4-excel
status: drafted
tags: [tool, internal, compat]
tool_kind: crate
language: rust
artifact: laterite-ags4-excel
ags_editions: []
repo_refs:
  root: "repo:rust-packages/laterite-ags4-excel"
  lib: "repo:rust-packages/laterite-ags4-excel/src/lib.rs"
related: [crate-map, laterite-ags4-core, laterite-py, python-ags4, reliquary, core-perf-baseline]
sources: []
---
# laterite-ags4-excel

<!-- BEGIN GENERATED: crate-card — DO NOT EDIT BY HAND. Regenerate: uv run --no-project python tools/gen_crate_graph.py -->
> [!note] **Not published** — `laterite-ags4-excel` is a workspace crate, internal to this repo, at v0.11.0 (inherited from the workspace).
> **Used by** — [[laterite-ags4-wasm]], [[laterite-cli]], [[laterite-node]], [[laterite-py]].
<!-- END GENERATED: crate-card -->

> [!note] [[laterite-py]] exposes it as `laterite.compat.AGS4_to_excel` /
> `excel_to_AGS4`; the card above lists every consumer.

## What it is

AGS4 ↔ XLSX conversion: the Rust-backed Excel I/O behind the
[[python-ags4]] drop-in surface. Writing uses `rust_xlsxwriter`, reading uses
`calamine` — both pure Rust, so no Python dependency crosses the boundary even
though the behaviour mirrors python-ags4's openpyxl implementation.

Stage 2b of the python-ags4 parity arc.

## Why it is a separate crate

Extracted out of [[laterite-ags4-core]] (2026-06-18) so core stops dragging
`calamine` + `rust_xlsxwriter` — roughly **1.5 MB** — into every consumer that
never touches Excel: the DuckDB extension, `ags4-perf`, and the wasm builds.

The dependency edge is also load-bearing in the other direction. Excel depends on
core with `default-features = false`, which drops core's default-on `transport`
module (`age` + `zstd`). Pulling that in dragged `age` → `getrandom` into the
tree, which **blocks the wasm32 build** (#359). Consumers that genuinely want
transport enable it on their own core dependency instead.

> [!note] Excel sits **above** core in the layering: it consumes the parser,
> never the reverse. That direction is what keeps the leaf constraint honest.

## Output layout

Matches python-ags4's, deliberately — the point is a drop-in:

- one sheet per AGS4 group;
- the HEADING column first;
- UNIT / TYPE / DATA pseudo-rows preserved;
- column widths `min(max(13, max_str_len + 1), 75)`.

`apply_type_formatting` pads numeric cells to their AGS4 TYPE's decimal places
(a `3DP` value renders `5.100`), skipping blank specs and unparseable values
rather than guessing.

## Surface

`ags4_to_excel` / `ags4_bytes_to_xlsx` and `excel_to_ags4` /
`xlsx_bytes_to_ags4`, plus `ExcelStats` for the row/sheet counts the Python
surface reports back.

The read direction consumes `AgsGroup` from [[laterite-ags4-core]]'s
`read_ags4_bytes`, so it inherits that projection's string/trim semantics — see
[[core-perf-baseline]] for why those rows are keyed by shared `Arc<str>` heading
names rather than per-row `String` clones.

## Status: flagged for rewrite

> [!warning] This is a **rough extraction**, not a designed library. The logic
> was lifted verbatim out of `laterite-ags4-core::excel` to shed the dependency
> weight, and is unchanged since.
>
> Today it is AGS4-specific (one sheet per group, AGS4 UNIT/TYPE pseudo-rows).
> The intent — and the reason for the general name — is to grow it into a proper
> general-purpose Excel library. Until that happens, treat the module boundary as
> aspirational and the contents as inherited.

Tracked in the relic register: [[reliquary]].

## Where it lives

`repo:rust-packages/laterite-ags4-excel` — a single `src/lib.rs`.

## Related

[[crate-map]] · [[laterite-ags4-core]] · [[laterite-py]] · [[python-ags4]] · [[reliquary]]
