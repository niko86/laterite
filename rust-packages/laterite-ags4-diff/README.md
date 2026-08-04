# laterite-ags4-diff

A **KEY-aware, type-aware** comparison of two AGS4 files — the diff a line diff
structurally cannot be, because it understands the data model.

```rust
let delta = laterite_ags4_diff::diff_parsed(&baseline, &revision, &dict, None);
```

<!-- BEGIN GENERATED: availability — DO NOT EDIT BY HAND. Regenerate: uv run --no-project python tools/gen_crate_graph.py -->
## Install it

```bash
cargo add laterite-ags4-diff
```

Currently v0.9.0 — the engine crates move in lockstep on the workspace version.
<!-- END GENERATED: availability -->

## Why a line diff is the wrong tool

AGS4 is a tabular transfer format, and the two things that make text diffs noisy
on it are both properties of the format rather than of any particular file.

**Row order carries no meaning.** Re-sorting a delivery, or renumbering it,
changes every line without changing a single fact. So rows are matched by the
group's *dictionary* KEY headings, not by position: the same borehole pairs with
the same borehole, and a moved row is a moved row rather than a delete plus an
add. Groups with no dictionary KEY present in both files — custom or passthrough
groups — fall back to matching on the whole row tuple.

**Equal values have unequal spellings.** `1.0` and `1.00` are the same number,
and an AGS4 `DT` field admits several spellings of one instant. Matched cells are
therefore compared through the type system rather than as bytes, so a
formatting-only change reports nothing and a genuine typed change always
reports.

What comes back is a `RevisionDelta` — group, row and cell deltas — which
serialises, so the same comparison drives a CLI verb, a JSON payload and a
browser view without any of them re-deciding what "changed" means.

Row identity comes from the same keychain definition
`laterite-ags4-merge` consumes, so what identifies a row is defined once for the
whole toolchain rather than twice, slightly differently.

Part of the [laterite](https://github.com/niko86/laterite) AGS4 toolchain.

## Licence

MIT.
