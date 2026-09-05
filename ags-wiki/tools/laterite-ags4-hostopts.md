---
type: tool
title: laterite-ags4-hostopts
status: drafted
tags: [tool, internal]
tool_kind: crate
language: rust
artifact: laterite-ags4-hostopts
ags_editions: []
repo_refs:
  root: "repo:rust-packages/laterite-ags4-hostopts"
  lib: "repo:rust-packages/laterite-ags4-hostopts/src/lib.rs"
related: [crate-map, laterite-ags4-emit, laterite-ags4-validator, edition-resolution, pyo3-boundary]
sources: []
---
# laterite-ags4-hostopts

<!-- BEGIN GENERATED: crate-card — DO NOT EDIT BY HAND. Regenerate: uv run --no-project python tools/gen_crate_graph.py -->
> [!note] **Cleared for crates.io** — `laterite-ags4-hostopts` declares `publish = true`, so it is a public API under semver, not an internal detail. It is versioned on its own line.
> **Used by** — [[laterite-ags4-corpus-qa]], [[laterite-ags4-wasm]], [[laterite-cli]], [[laterite-node]], [[laterite-py]].
<!-- END GENERATED: crate-card -->

## What it is

The one copy of the caller-facing option normalisation every surface binding
shares (#923): edition labels (`auto` kept deferrable for the check doors,
collapsed to the dictionary's generated fallback for the emit doors), write
modes, the custom-dictionary ladder, and the staged atomic `out=` write. A
binding shrinks to marshal → call → map `OptError` into its own error type;
what stays per-surface is *data*, not logic — the flag spellings that
surface's user actually typed, so a refusal names the knob as the caller
spelled it.

Before #923 each surface normalised these knobs in its own dialect, and the
copies drifted three ways before anything noticed (two stale fallback
editions, `auto` with two semantics, a staged write that lost its exclusive
create). See [[edition-resolution]] for the edition half of that story.

## Why it is a separate crate

Born as `laterite_ags4_emit::hostopts`, and the tenancy was a **publishing
accident**: a module the published facade adopts from must itself live in a
published crate, and emit was the one already on crates.io. Nothing inside
emit ever called it — so every hostopts change rode emit's release train for
no reason (emit 0.17.0 was forced partly by a hostopts addition, and that
bump cascaded into three sibling re-pins). Extracted by #947, on the owner's
call, when #930's facade adoption was about to write the import lines anyway.

Layering: it returns emit's `EmitMode` and builds the validator's
`CustomDict`, so it sits **above** both (L3) and below the five callers —
the PyO3, napi and wasm bindings, the CLI, and (post-#930) the facade.

## Surface

`edition` / `edition_or_fallback` · `write_mode` · `custom_dict` (+
`DictFlags`, the per-surface spelling data) · `staged_write` /
`staged_write_io` (#938 — the `io::Error`-shaped twin for hosts whose error
contract carries the OS shape) · `OptError`.

## Where it lives

`repo:rust-packages/laterite-ags4-hostopts` — a single `src/lib.rs`.

## Related

[[crate-map]] · [[laterite-ags4-emit]] · [[laterite-ags4-validator]] · [[edition-resolution]] · [[pyo3-boundary]]
