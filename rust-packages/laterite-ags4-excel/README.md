# laterite-ags4-excel

Convert **AGS4** geotechnical data to and from **XLSX** workbooks.

```rust
use laterite_ags4_excel::{ags4_bytes_to_xlsx, xlsx_bytes_to_ags4};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ags4 = concat!(
        "\"GROUP\",\"LOCA\"\r\n",
        "\"HEADING\",\"LOCA_ID\",\"LOCA_NATE\"\r\n",
        "\"UNIT\",\"\",\"m\"\r\n",
        "\"TYPE\",\"ID\",\"2DP\"\r\n",
        "\"DATA\",\"BH01\",\"523145.1\"\r\n",
    );

    // One worksheet per AGS4 group, no filesystem involved.
    let (xlsx, stats) = ags4_bytes_to_xlsx(ags4.as_bytes(), None)?;
    assert_eq!(stats.sheets_written, 1);

    // …and back. The TYPE pseudo-row survives the trip, so the 2DP
    // column comes home in AGS4's canonical formatting.
    let (back, _) = xlsx_bytes_to_ags4(&xlsx, true)?;
    assert!(String::from_utf8(back)?.contains("\"523145.10\""));
    Ok(())
}
```

`ags4_to_excel` / `excel_to_ags4` are the same two conversions with paths in
place of byte slices, and the `_with` variants take explicit read options.

<!-- BEGIN GENERATED: availability — DO NOT EDIT BY HAND. Regenerate: uv run --no-project python tools/gen_crate_graph.py -->
> **Engine crate, not a door.** `laterite-ags4-excel` is machinery inside the laterite
> toolchain, reshaped whenever the toolchain needs it. The Rust door is
> [`laterite`](https://crates.io/crates/laterite); depend on this one directly
> only if that suits you, and expect it to move.

## Install it

```bash
cargo add laterite-ags4-excel
```

This crate versions independently of the engine.
<!-- END GENERATED: availability -->

## What it does

- **Writes python-ags4's workbook layout**, deliberately: one sheet per AGS4
  group, the `HEADING` column first, the `UNIT` / `TYPE` / `DATA` pseudo-rows
  preserved as rows. Pure Rust on both directions — `rust_xlsxwriter` writes,
  `calamine` reads — so no Python dependency crosses the boundary.
- **Re-formats numeric columns on the way back.** A spreadsheet edit turns
  `523145.10` into the float `523145.1`; with `format_numeric_columns` on, each
  `DATA` cell is rendered to its column's declared TYPE again, so a `2DP`
  column leaves with its trailing zeros.
- **Drops what it cannot place, out loud.** Worksheets without a `HEADING`
  column, columns whose name is not a valid AGS4 heading, and rows that are
  none of `UNIT`/`TYPE`/`DATA` are skipped with a warning in the returned
  `ExcelStats`, never silently.

## Flagged for rewrite

This crate is a rough extraction from `laterite-ags4-core`, made so that core
consumers which never touch Excel stop carrying the XLSX machinery. The
surface is AGS4-specific and will be redesigned; expect breaking versions. The
[`laterite`](https://crates.io/crates/laterite) facade wraps it opaquely, so
facade users never see those breaks.

Part of the [laterite](https://github.com/niko86/laterite) AGS4 toolchain.

## Licence

MIT.
