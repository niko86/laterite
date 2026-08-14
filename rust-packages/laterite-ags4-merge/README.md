# laterite-ags4-merge

Reconcile **N deliveries** of one project into a single AGS4 file.

```rust
use laterite_ags4_merge::{merge_parsed, MergeOpts};
use laterite_ags4_parse::parse_str;

const AGS4: &str = concat!(
    "\"GROUP\",\"PROJ\"\r\n",
    "\"HEADING\",\"PROJ_ID\"\r\n",
    "\"UNIT\",\"\"\r\n",
    "\"TYPE\",\"ID\"\r\n",
    "\"DATA\",\"P1\"\r\n",
);

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let a = parse_str(AGS4).expect("valid AGS4");
    let b = parse_str(AGS4).expect("valid AGS4");

    let merged = merge_parsed(&[a, b], &MergeOpts::default())?;
    println!("{} revision(s)", merged.revisions.len());
    Ok(())
}
```

<!-- BEGIN GENERATED: availability — DO NOT EDIT BY HAND. Regenerate: uv run --no-project python tools/gen_crate_graph.py -->
> **Engine crate, not a door.** `laterite-ags4-merge` is machinery inside the laterite
> toolchain, reshaped whenever the toolchain needs it. The Rust door is
> [`laterite`](https://crates.io/crates/laterite); depend on this one directly
> only if that suits you, and expect it to move.

## Install it

```bash
cargo add laterite-ags4-merge
```

Currently v0.9.0 — the engine crates move in lockstep on the workspace version.
<!-- END GENERATED: availability -->

## Why this is a crate

Real geotechnical delivery is incremental: each file carries only what was
captured that round, and someone eventually has to fold a season of them into one
document. Doing that correctly turns on three decisions that are easy to get
quietly wrong, which is why they are made once here rather than in every caller.

**Union, never intersection.** A row or group missing from a later file is
*silence*, not a deletion — the producer expressed no opinion this round. Merge
only ever adds. There is deliberately no delete-or-supersede primitive, so a
corrected KEY value arrives as a new row; that is a documented limit of KEY-based
identity rather than a case handled silently and wrongly.

**Argument order is authority.** When two files carry the same KEY with different
content, the later argument wins. `TRAN_DATE` only cross-checks: if it
contradicts that ordering the result carries a warning, never an error, because
it is file-level and blind to a per-row regression — orderable enough to notice a
contradiction, not enough to settle one.

**Type disagreement resolves up, never down.** A heading two files typed
differently is settled by `TypeClashMode`. `Error` refuses, and is the default.
`Widen` falls back to `X`, the top of the AGS type lattice, where raw text holds
any value faithfully. `Promote` keeps the column numeric at the greatest
precision present. `Promote` is the only place merge rewrites a cell, and it is
confined to appending zeros to a decimal — string-only, never through `f64`, and
never rounding.

Row identity comes from the same keychain definition `laterite-ags4-diff`
consumes, so what identifies a row is defined once for the whole toolchain.

Part of the [laterite](https://github.com/niko86/laterite) AGS4 toolchain.

## Licence

MIT.
