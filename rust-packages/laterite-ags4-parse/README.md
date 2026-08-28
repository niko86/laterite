# laterite-ags4-parse

The shared parse leaf for the **AGS4** geotechnical data transfer format: one
tolerant tokenizer, and one source-true walk that carries every coordinate
system a caller might need.

```rust
const AGS4: &str = concat!(
    "\"GROUP\",\"PROJ\"\r\n",
    "\"HEADING\",\"PROJ_ID\"\r\n",
    "\"UNIT\",\"\"\r\n",
    "\"TYPE\",\"ID\"\r\n",
    "\"DATA\",\"P1\"\r\n",
);

fn main() -> Result<(), laterite_ags4_parse::ParseError> {
    let parsed = laterite_ags4_parse::parse_str(AGS4)?;
    for (code, group) in &parsed.groups {
        println!("{} — {} rows", code, group.rows.len());
    }
    Ok(())
}
```

AGS4 is a CSV-shaped plaintext format: every field double-quoted, records
prefixed by a descriptor (`"GROUP"`, `"HEADING"`, `"UNIT"`, `"TYPE"`, `"DATA"`).
The tolerant part matters — real deliveries arrive with ragged rows, stray
whitespace, mixed line endings and non-UTF-8 encodings, and a parser that
rejects them is a parser nobody can use to find out *why* they are wrong.

<!-- BEGIN GENERATED: availability — DO NOT EDIT BY HAND. Regenerate: uv run --no-project python tools/gen_crate_graph.py -->
> **Engine crate, not a door.** `laterite-ags4-parse` is machinery inside the laterite
> toolchain, reshaped whenever the toolchain needs it. The Rust door is
> [`laterite`](https://crates.io/crates/laterite); depend on this one directly
> only if that suits you, and expect it to move.

## Install it

```bash
cargo add laterite-ags4-parse
```

The engine crates move in lockstep on the workspace version.
<!-- END GENERATED: availability -->

## What is source-true about it

A single pass records, for every record: its absolute **byte** offset in the
original buffer, its 1-indexed **line** number, and (via `field_span`) **char**
spans within a line. None of the three is back-derived from another, so a
finding can be reported against the byte a tool wants to seek to, the line a
human reads, and the column an editor highlights — without the rounding errors
that reconstruction introduces once a file contains a multi-byte character.

`parse_bytes` additionally handles encoding detection, so a Latin-1 file is not
silently mangled into replacement characters before anything has looked at it.

## Scope

Deliberately small. Two dependencies (`encoding_rs`, `memchr`), no dictionary,
no rules, no opinion about whether a file is *valid* — that is
[`laterite-ags4-validator`](https://crates.io/crates/laterite-ags4-validator),
which is built on this. Parsing and judging are separate jobs and this crate
only does the first.

Written clean-room from the AGS4.1 specification (§4.1.1, Rules 1–7).

## Licence

MIT.
