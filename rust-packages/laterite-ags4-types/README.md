# laterite-ags4-types

The **AGS4** type system: the small set of canonical types every AGS type code
maps to, plus permissive parsing and spec-faithful formatting of values.

```rust
use laterite_ags4_types::{canonical_type, parse_value, CanonicalType};

assert_eq!(canonical_type("2DP"), Some(CanonicalType::Decimal));
let v = parse_value(Some("12.30"), "2DP");   // -> 12.3
```

AGS4 encodes a column's type in a short code — `X` (text), `ID`, `2DP`, `3SF`,
`0SCI`, `DT`, `YN`, `PA`, and so on. This crate is the one place that knows what
each means, in both directions:

- **Reading:** `parse_value` turns a raw field into a typed JSON value. It is
  deliberately permissive — an unparseable value becomes `Null` rather than an
  error, and an unrecognised code falls through to string storage. Refusing to
  read a file because one cell is malformed is not useful behaviour for a format
  whose whole point is exchanging imperfect field data.
- **Writing:** `format_ndp`, `format_nsf`, `format_nsci` and `pad_decimals`
  reproduce the significant-figure and decimal-place conventions the spec
  requires, so a value read and written back is byte-identical rather than
  merely numerically equal.

`quote_field` / `write_quoted_field` implement AGS4's quoting rules for the
writer side.

<!-- BEGIN GENERATED: availability — DO NOT EDIT BY HAND. Regenerate: uv run --no-project python tools/gen_crate_graph.py -->
> **Engine crate, not a door.** `laterite-ags4-types` is machinery inside the laterite
> toolchain, reshaped whenever the toolchain needs it. The Rust door is
> [`laterite`](https://crates.io/crates/laterite); depend on this one directly
> only if that suits you, and expect it to move.

## Install it

```bash
cargo add laterite-ags4-types
```

Currently v0.9.0 — the engine crates move in lockstep on the workspace version.
<!-- END GENERATED: availability -->

## Scope

A wasm-safe leaf with no I/O and no dictionary. The optional `arrow` feature
adds typed Apache Arrow column builders and IPC framing, used by the Python and
browser bindings; without it the crate is dependency-light.

Part of the [laterite](https://github.com/niko86/laterite) AGS4 toolchain.

## Licence

MIT.
