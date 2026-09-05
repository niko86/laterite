//! `laterite-ags4-excel` — AGS4 ↔ XLSX conversion (Rust-backed Excel I/O for
//! `laterite.compat.AGS4_to_excel` / `excel_to_AGS4`).
//!
//! ⚠️ ROUGH EXTRACTION (2026-06-18): lifted verbatim out of
//! `laterite-ags4-core::excel` so `laterite-ags4-core` sheds `calamine` +
//! `rust_xlsxwriter` — ~1.5 MB that every core consumer which never touches
//! Excel was carrying (the DuckDB extension, `ags4-perf`).
//! The logic is unchanged. **FLAGGED FOR REWRITE**: today this is
//! AGS4-specific (one sheet per group, AGS4 UNIT/TYPE pseudo-rows).
//!
//! It was called `laterite-excel` until 2026-08-05 — a name chosen for a
//! general-purpose Excel library it never became. `-ags4-` marks the engine
//! tier, which is what this is, and a crates.io name is free until its first
//! publish and irreversible after, so the correction had to land before the
//! crate went out. If the general-purpose rewrite ever happens it wants its own
//! name rather than this one back.
//!
//! Mirrors python-ags4's openpyxl-based implementation but uses two
//! pure-Rust crates: `rust_xlsxwriter` for writing and `calamine` for
//! reading. No Python deps cross the boundary; outputs match
//! python-ags4's layout (one sheet per group, HEADING column first,
//! UNIT / TYPE / DATA pseudo-rows preserved, column widths
//! `min(max(13, max_str_len + 1), 75)`).
//!
//! Stage 2b of the python-ags4 parity arc.

// The README's example is a doctest, not a second copy of one. `cfg(doctest)`
// means this module exists only while rustdoc collects doctests: it is absent
// from a normal build and from the rendered docs.rs page, so the crate's own
// `//!` docs are untouched and nothing is duplicated. The README is the single
// source, and `cargo test --workspace` already compiles it.
#[cfg(doctest)]
#[doc = include_str!("../README.md")]
mod readme_doctests {}

use std::io::Cursor;
use std::path::Path;

use calamine::{Data, Reader, open_workbook_auto_from_rs};
use rust_xlsxwriter::Workbook;

use laterite_ags4_core::ags4_codec::{AgsGroup, ReadOptions, read_ags4_bytes_with};
use laterite_ags4_core::error::CliError;
use laterite_ags4_emit::{EmitError, EmitGroup, write_ags4};

/// Stats returned by both conversion helpers. The PyO3 layer surfaces
/// these as a dict on the Python side.
#[derive(Debug, Clone)]
pub struct ExcelStats {
    pub sheets_written: usize,
    pub rows_written: usize,
    pub warnings: Vec<String>,
}

// --- AGS4 → XLSX ---------------------------------------------------

/// Read an AGS4 file and write each group as a worksheet in `output`.
///
/// `ordered_keys`: if `Some`, write groups in this exact order
/// (caller is responsible for ensuring every key exists in the
/// parsed file). If `None`, preserve the AGS4 source order.
pub fn ags4_to_excel(
    input: &Path,
    output: &Path,
    ordered_keys: Option<Vec<String>>,
) -> Result<ExcelStats, CliError> {
    ags4_to_excel_with(input, output, ordered_keys, ReadOptions::default())
}

/// [`ags4_to_excel`] with explicit [`ReadOptions`] — the duplicate-heading
/// recovery mode matters most here, since an XLSX export is the usual way
/// someone tries to get data out of a file that will not validate.
pub fn ags4_to_excel_with(
    input: &Path,
    output: &Path,
    ordered_keys: Option<Vec<String>>,
    read_opts: ReadOptions,
) -> Result<ExcelStats, CliError> {
    let bytes = std::fs::read(input)
        .map_err(|e| CliError::Schema(format!("read {}: {e}", input.display())))?;
    let (xlsx, stats) = ags4_bytes_to_xlsx_with(&bytes, ordered_keys, read_opts)?;
    std::fs::write(output, xlsx)
        .map_err(|e| CliError::Schema(format!("save xlsx {}: {e}", output.display())))?;
    Ok(stats)
}

/// AGS4 bytes → XLSX bytes — the **FS-free core** the wasm surface (laterite-dev#359) and
/// the path wrapper above both call. Each group becomes a worksheet in
/// python-ags4's layout.
///
/// `ordered_keys`: if `Some`, write groups in this exact order (the caller
/// ensures every key exists in the parsed file); if `None`, preserve the AGS4
/// source order.
pub fn ags4_bytes_to_xlsx(
    input: &[u8],
    ordered_keys: Option<Vec<String>>,
) -> Result<(Vec<u8>, ExcelStats), CliError> {
    ags4_bytes_to_xlsx_with(input, ordered_keys, ReadOptions::default())
}

/// [`ags4_bytes_to_xlsx`] with explicit [`ReadOptions`].
pub fn ags4_bytes_to_xlsx_with(
    input: &[u8],
    ordered_keys: Option<Vec<String>>,
    read_opts: ReadOptions,
) -> Result<(Vec<u8>, ExcelStats), CliError> {
    let parsed = read_ags4_bytes_with(input, read_opts)?;
    let order: Vec<String> = ordered_keys.unwrap_or_else(|| parsed.order().to_vec());

    if order.is_empty() {
        return Err(CliError::Schema(
            "No valid AGS4 data found in input file.".into(),
        ));
    }

    let mut workbook = Workbook::new();
    let mut sheets_written = 0usize;
    let mut rows_written = 0usize;
    let mut warnings: Vec<String> = Vec::new();

    for code in &order {
        let Some(group) = parsed.get(code) else {
            warnings.push(format!("skip {code}: not in parsed AGS4 input"));
            continue;
        };

        let sheet = workbook
            .add_worksheet()
            .set_name(code)
            .map_err(|e| CliError::Schema(format!("sheet name {code}: {e}")))?;

        // Header row: HEADING + every group heading name. python-ags4
        // writes HEADING as the first column then the AGS heading
        // names (LOCA_ID, LOCA_TYPE, ...) in declaration order.
        sheet
            .write_string(0, 0, "HEADING")
            .map_err(|e| CliError::Schema(format!("write HEADING: {e}")))?;
        for (i, heading) in group.headings().iter().enumerate() {
            sheet
                .write_string(0, excel_col(i + 1)?, heading)
                .map_err(|e| CliError::Schema(format!("write heading: {e}")))?;
        }

        // UNIT, TYPE, then each DATA row.
        write_group_row(sheet, 1, "UNIT", group.units(), group.headings().len())?;
        write_group_row(sheet, 2, "TYPE", group.types(), group.headings().len())?;
        for ri in 0..group.n_rows() {
            // Row index, not column: bounded by rows actually held in memory
            // (Excel's own cap is ~1M, u32::MAX is billions — unreachable
            // without exhausting RAM first), so no fallible conversion here.
            #[allow(clippy::cast_possible_truncation)]
            let r = (3 + ri) as u32;
            sheet
                .write_string(r, 0, "DATA")
                .map_err(|e| CliError::Schema(format!("write DATA tag: {e}")))?;
            for (ci, value) in group.row_cells(ri).enumerate() {
                if !value.is_empty() {
                    sheet
                        .write_string(r, excel_col(ci + 1)?, value)
                        .map_err(|e| CliError::Schema(format!("write cell: {e}")))?;
                }
            }
            rows_written += 1;
        }

        // Column widths: `min(max(13, max_str_len + 1), 75)` — same
        // rule python-ags4 applies, computed across HEADING + UNIT +
        // TYPE + every DATA row.
        // Column 0 (HEADING marker): widest value is "HEADING" (7
        // chars) bounded to 13 minimum.
        sheet
            .set_column_width(0, column_width("HEADING", group))
            .map_err(|e| CliError::Schema(format!("col width: {e}")))?;
        for (i, heading) in group.headings().iter().enumerate() {
            sheet
                .set_column_width(excel_col(i + 1)?, column_width(heading, group))
                .map_err(|e| CliError::Schema(format!("col width: {e}")))?;
        }

        sheets_written += 1;
    }

    let buf = workbook
        .save_to_buffer()
        .map_err(|e| CliError::Schema(format!("build xlsx: {e}")))?;
    Ok((
        buf,
        ExcelStats {
            sheets_written,
            rows_written,
            warnings,
        },
    ))
}

/// Pad a row's cells to the heading count and write at row index `r`.
fn write_group_row(
    sheet: &mut rust_xlsxwriter::Worksheet,
    r: u32,
    tag: &str,
    cells: &[String],
    heading_count: usize,
) -> Result<(), CliError> {
    sheet
        .write_string(r, 0, tag)
        .map_err(|e| CliError::Schema(format!("write {tag}: {e}")))?;
    for i in 0..heading_count {
        let value = cells.get(i).map_or("", String::as_str);
        if !value.is_empty() {
            sheet
                .write_string(r, excel_col(i + 1)?, value)
                .map_err(|e| CliError::Schema(format!("write {tag} cell: {e}")))?;
        }
    }
    Ok(())
}

/// One worksheet's harvested group, positional throughout — the local shape
/// `from_excel` feeds the writer (it stopped constructing codec groups when
/// #900 made those span-backed; the writer only ever wanted positions).
struct SheetGroup {
    code: String,
    headings: Vec<String>,
    units: Vec<String>,
    types: Vec<String>,
    rows: Vec<Vec<String>>,
}

/// `usize` column index → the `u16` `rust_xlsxwriter` wants. A group with more
/// than `u16::MAX` headings can't happen from a normal dictionary group (58 max
/// — laterite-dev#475's `ags_dictionary.json`), but an oversized or malformed HEADING row
/// (a passthrough/dynamic group, or a corrupt file) could plausibly carry tens
/// of thousands of fields without needing a huge file — each field can be a
/// few bytes. `as u16` would silently WRAP that into an existing column,
/// overwriting real data instead of failing loudly, so this is checked.
fn excel_col(index: usize) -> Result<u16, CliError> {
    u16::try_from(index).map_err(|_| {
        CliError::Schema(format!(
            "too many columns ({index}) for an Excel worksheet (max {})",
            u16::MAX
        ))
    })
}

/// Column width matching python-ags4's `min(max(13, max_len+1), 75)`.
/// Considers the heading name itself plus the UNIT / TYPE / every
/// DATA value for that column.
fn column_width(heading: &str, group: &AgsGroup) -> f64 {
    let mut max_len = heading.len();
    // Compare against UNIT/TYPE entries at the same column index.
    if let Some(idx) = group.col(heading) {
        if let Some(u) = group.units().get(idx) {
            max_len = max_len.max(u.len());
        }
        if let Some(t) = group.types().get(idx) {
            max_len = max_len.max(t.len());
        }
        // Then every DATA row.
        for ri in 0..group.n_rows() {
            if let Some(v) = group.cell(ri, idx) {
                max_len = max_len.max(v.len());
            }
        }
    }
    let width = max_len + 1;
    width.clamp(13, 75) as f64
}

// --- XLSX → AGS4 ---------------------------------------------------

/// Read an XLSX file and write its contents as an AGS4 file. Each
/// worksheet with a `HEADING` column becomes one AGS4 group; columns
/// not matching Rule 19's `[A-Z0-9]{4}_[A-Z0-9]{1,4}` regex are
/// dropped (with a warning); rows whose HEADING isn't `UNIT`, `TYPE`,
/// or `DATA` are dropped.
///
/// `format_numeric_columns` (default `true`): when set, re-format
/// each DATA cell to match its column's TYPE specifier
/// (`<N>DP` / `<N>SF` / `<N>SCI`). This mirrors python-ags4's
/// `convert_to_text` step — without it, XLSX-derived floats may
/// lose trailing zeros (`5000000.1` instead of the AGS4-canonical
/// `5000000.100` for a 3DP column).
pub fn excel_to_ags4(
    input: &Path,
    output: &Path,
    format_numeric_columns: bool,
) -> Result<ExcelStats, CliError> {
    let bytes = std::fs::read(input)
        .map_err(|e| CliError::Schema(format!("read {}: {e}", input.display())))?;
    let (ags4, stats) = xlsx_bytes_to_ags4(&bytes, format_numeric_columns)?;
    std::fs::write(output, ags4)
        .map_err(|e| CliError::Schema(format!("create {}: {e}", output.display())))?;
    Ok(stats)
}

/// XLSX bytes → AGS4 bytes — the **FS-free core** the wasm surface (laterite-dev#359) and
/// the path wrapper above both call. Each worksheet with a `HEADING` column
/// becomes one AGS4 group; columns not matching Rule 19's
/// `[A-Z0-9]{4}_[A-Z0-9]{1,4}` are dropped (with a warning); rows whose HEADING
/// isn't `UNIT`/`TYPE`/`DATA` are dropped. See [`excel_to_ags4`] for the
/// `format_numeric_columns` semantics.
pub fn xlsx_bytes_to_ags4(
    input: &[u8],
    format_numeric_columns: bool,
) -> Result<(Vec<u8>, ExcelStats), CliError> {
    let mut workbook = open_workbook_auto_from_rs(Cursor::new(input))
        .map_err(|e| CliError::Schema(format!("open xlsx: {e}")))?;

    let sheet_names: Vec<String> = workbook.sheet_names().clone();
    let mut emit_groups: Vec<SheetGroup> = Vec::new();
    let mut warnings: Vec<String> = Vec::new();
    let mut rows_written = 0usize;

    for sheet_name in &sheet_names {
        let range = workbook
            .worksheet_range(sheet_name)
            .map_err(|e| CliError::Schema(format!("read sheet {sheet_name}: {e}")))?;

        let mut rows_iter = range.rows();
        let Some(header_row) = rows_iter.next() else {
            warnings.push(format!("{sheet_name}: empty sheet, skipped"));
            continue;
        };

        // First column must be HEADING for this to be a valid AGS4
        // group sheet. Scan the header row for that.
        let mut heading_col_idx: Option<usize> = None;
        for (i, cell) in header_row.iter().enumerate() {
            if cell_str(cell).eq_ignore_ascii_case("HEADING") {
                heading_col_idx = Some(i);
                break;
            }
        }
        let Some(hcol) = heading_col_idx else {
            warnings.push(format!(
                "Worksheet {sheet_name} dropped as it does not have a HEADING column."
            ));
            continue;
        };

        // Build the heading list from the remaining header cells,
        // applying Rule 19 (4-letter group + underscore + 1-4 char
        // heading suffix). Columns not matching are recorded in
        // warnings and dropped.
        let mut headings: Vec<String> = Vec::new();
        let mut keep_cols: Vec<usize> = Vec::new();
        for (i, cell) in header_row.iter().enumerate() {
            if i == hcol {
                continue;
            }
            let name = cell_str(cell);
            if name.is_empty() {
                continue;
            }
            if matches_rule_19_heading(&name) {
                headings.push(name);
                keep_cols.push(i);
            } else {
                warnings.push(format!(
                    "Column {name} dropped as name does not conform to AGS4 Rule 19."
                ));
            }
        }

        let mut units: Vec<String> = vec![String::new(); headings.len()];
        let mut types: Vec<String> = vec![String::new(); headings.len()];
        // Positional rows, aligned with `headings` by the `keep_cols`
        // projection — the writer wants positional cells, so the by-name map
        // this used to build was a detour (#900 retired it with the codec's).
        let mut data_rows: Vec<Vec<String>> = Vec::new();

        for row in rows_iter {
            let tag = cell_str(row.get(hcol).unwrap_or(&Data::Empty));
            let payload = || -> Vec<String> {
                keep_cols
                    .iter()
                    .map(|&c| cell_str(row.get(c).unwrap_or(&Data::Empty)))
                    .collect()
            };
            match tag.as_str() {
                "UNIT" => units = payload(),
                "TYPE" => types = payload(),
                "DATA" => {
                    data_rows.push(payload());
                    rows_written += 1;
                }
                "" => {} // empty separator rows in the sheet
                _ => {
                    warnings.push(format!(
                        "{sheet_name}: dropped row with HEADING={tag} (not UNIT/TYPE/DATA)"
                    ));
                }
            }
        }

        if format_numeric_columns {
            apply_type_formatting(&headings, &types, &mut data_rows);
        }

        emit_groups.push(SheetGroup {
            code: sheet_name.clone(),
            headings,
            units,
            types,
            rows: data_rows,
        });
    }

    if emit_groups.is_empty() {
        return Err(CliError::Schema(
            "No valid AGS4 data found in input file. Each sheet needs a HEADING column.".into(),
        ));
    }

    // The sheets' rows are positional already, so `EmitGroup` borrows them
    // directly — no per-group copy.
    let emit_views: Vec<EmitGroup<'_>> = emit_groups
        .iter()
        .map(|g| EmitGroup {
            code: &g.code,
            headings: g.headings.iter().map(String::as_str).collect(),
            units: g.units.iter().map(String::as_str).collect(),
            types: g.types.iter().map(String::as_str).collect(),
            rows: &g.rows,
        })
        .collect();

    let mut out: Vec<u8> = Vec::new();
    write_ags4(&mut out, &emit_views).map_err(emit_err)?;

    Ok((
        out,
        ExcelStats {
            sheets_written: emit_groups.len(),
            rows_written,
            warnings,
        },
    ))
}

/// Map the AGS4 writer's error onto excel's `CliError` surface. This was
/// `impl From<EmitError> for CliError` in `laterite-ags4-core`; inlined here
/// (excel is its sole caller) so `core` no longer depends on the `emit` leaf —
/// the `core → emit → validator` layering cut (#441). `write_ags4` only yields
/// `Write` / `EmbeddedNewline`; the `Reparse` / `Invalid` arms are unreachable
/// here but kept for totality, preserving the original messages verbatim.
fn emit_err(e: EmitError) -> CliError {
    match e {
        // Preserve the historical "ags4 write: …" Schema message.
        EmitError::Write(m) => CliError::Schema(format!("ags4 write: {m}")),
        EmitError::Reparse(m) => CliError::Schema(format!("ags4 emit: {m}")),
        EmitError::Invalid(found) => {
            let n: usize = found.values().map(Vec::len).sum();
            CliError::Schema(format!(
                "ags4 emit: strict mode rejected output ({n} finding(s))"
            ))
        }
        e @ EmitError::EmbeddedNewline { .. } => CliError::Schema(format!("ags4 write: {e}")),
    }
}

/// Re-format every DATA cell whose column's TYPE row declares a
/// numeric AGS spec (`<N>DP` / `<N>SF` / `<N>SCI`). Mirrors
/// python-ags4's `convert_to_text` / `format_numeric_column` chain:
/// pad trailing zeros to the precision the spec demands so AGS4
/// emit matches the column's declared TYPE.
///
/// Non-numeric specs pass through untouched. Cells that can't be
/// parsed as floats (e.g. blank strings, sentinels like `?`) also
/// pass through — matches python-ags4's silent `ValueError` fallback.
fn apply_type_formatting(headings: &[String], types: &[String], rows: &mut [Vec<String>]) {
    for i in 0..headings.len() {
        let Some(type_spec) = types.get(i) else {
            continue;
        };
        let spec = type_spec.trim();
        if spec.is_empty() {
            continue;
        }
        let Some(formatter) = NumericFormat::from_spec(spec) else {
            continue;
        };
        for row in rows.iter_mut() {
            if let Some(value) = row.get_mut(i) {
                if value.is_empty() {
                    continue;
                }
                let Ok(parsed) = value.parse::<f64>() else {
                    continue;
                };
                *value = formatter.format(parsed);
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum NumericFormat {
    Dp(usize),
    Sci(usize),
    Sf(usize),
}

impl NumericFormat {
    fn from_spec(spec: &str) -> Option<Self> {
        let upper = spec.to_ascii_uppercase();
        // Order matters: "SCI" must be checked before "SF" before "DP"
        // because the suffix comes after a digit prefix. We use strip_suffix
        // + verify the prefix is all digits.
        if let Some(n) = numeric_prefix(&upper, "SCI") {
            return Some(Self::Sci(n));
        }
        if let Some(n) = numeric_prefix(&upper, "SF") {
            return Some(Self::Sf(n));
        }
        if let Some(n) = numeric_prefix(&upper, "DP") {
            return Some(Self::Dp(n));
        }
        None
    }

    fn format(&self, v: f64) -> String {
        // Clamp the count (see MAX_NUMERIC_COUNT) — it comes uncapped from the
        // TYPE spec, and Dp/Sci feed it straight into the format width.
        match *self {
            Self::Dp(n) => {
                let n = n.min(MAX_NUMERIC_COUNT);
                format!("{v:.n$}")
            }
            Self::Sci(n) => {
                // python-ags4 uses `f"{x:.{N}E}"` — uppercase E with
                // a single digit exponent (e.g. 1.23E+02). Rust's
                // default scientific format matches.
                let n = n.min(MAX_NUMERIC_COUNT);
                format!("{v:.n$E}")
            }
            Self::Sf(n) => format_sf(v, n),
        }
    }
}

fn numeric_prefix(upper: &str, suffix: &str) -> Option<usize> {
    let prefix = upper.strip_suffix(suffix)?;
    if prefix.is_empty() || !prefix.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    prefix.parse::<usize>().ok()
}

/// A numeric TYPE count (the `n` in "3DP" / "3SF" / "3SCI", see `numeric_prefix`)
/// comes straight from the file's TYPE spec with no upper bound, and every
/// `NumericFormat` arm feeds `n` into a format width. f64 carries only ~17
/// significant digits, so clamp to this generous ceiling first — a crafted
/// "9999999999DP" then can't ask for a ~10-billion-char string (an OOM/DoS) or
/// wrap the i32 cast in `format_sf`. Real AGS4 numeric counts are single-digit,
/// so no legitimate value is affected. Hardens laterite-dev#610 Class B (O-49).
const MAX_NUMERIC_COUNT: usize = 30;

/// Significant-figure formatter — mirrors python-ags4's `_format_SF`.
fn format_sf(value: f64, n: usize) -> String {
    if value == 0.0 {
        return format!("{value}");
    }
    // log10 of any finite f64 is bounded to roughly ±308 (f64's exponent
    // range), always fits i32 regardless of `value`'s magnitude.
    #[allow(clippy::cast_possible_truncation)]
    let exp = value.abs().log10().floor() as i32;
    // Clamp first (see MAX_NUMERIC_COUNT): the bounded count fits i32.
    let n = i32::try_from(n.min(MAX_NUMERIC_COUNT)).unwrap_or(i32::MAX);
    let i: i32 = n - 1 - exp;
    if i < 0 {
        let rounded = (value / 10f64.powi(-i)).round() * 10f64.powi(-i);
        format!("{rounded:.0}")
    } else {
        format!("{value:.*}", i as usize)
    }
}

fn cell_str(cell: &Data) -> String {
    match cell {
        Data::Empty => String::new(),
        Data::Float(f) => {
            // Floats from calamine may be 1.0 for ints. Strip the
            // trailing .0 to match the string representation
            // python-ags4 / openpyxl produces for integer-valued cells.
            // An Excel cell can hold any float a user pasted in, so also
            // check the value actually fits i64 — outside that range `as
            // i64` would silently saturate to a wrong value instead of
            // erroring; falling through to `{f}`'s full decimal expansion
            // is the correct (if unusual) AGS4 field text for such a cell.
            let in_i64_range = (i64::MIN as f64..=i64::MAX as f64).contains(f);
            if f.fract() == 0.0 && f.is_finite() && in_i64_range {
                // Guarded by `in_i64_range` above (clippy can't see the
                // preceding `if` proves this in range).
                #[allow(clippy::cast_possible_truncation)]
                let v = *f as i64;
                format!("{v}")
            } else {
                format!("{f}")
            }
        }
        Data::Int(i) => i.to_string(),
        Data::Bool(b) => b.to_string(),
        Data::DateTime(d) => d.to_string(),
        Data::String(s) | Data::DateTimeIso(s) | Data::DurationIso(s) => s.clone(),
        Data::Error(e) => format!("{e:?}"),
    }
}

/// AGS4 Rule 19 column-name check — `^[A-Z0-9]{4}_[A-Z0-9]{1,4}$`.
/// Done by hand rather than pulling in `regex` (lean-dep guard).
fn matches_rule_19_heading(name: &str) -> bool {
    let mut chars = name.chars();
    for _ in 0..4 {
        match chars.next() {
            Some(c) if c.is_ascii_uppercase() || c.is_ascii_digit() => {}
            _ => return false,
        }
    }
    if chars.next() != Some('_') {
        return false;
    }
    let mut suffix_len = 0;
    for c in chars {
        if c.is_ascii_uppercase() || c.is_ascii_digit() {
            suffix_len += 1;
            if suffix_len > 4 {
                return false;
            }
        } else {
            return false;
        }
    }
    suffix_len >= 1
}

#[cfg(test)]
mod tests {
    use super::*;
    // The path round-trip tests re-parse the emitted file to assert its data;
    // the non-test code path now reads bytes with explicit options, so the
    // default-policy entry points are test-only here.
    use laterite_ags4_core::ags4_codec::{read_ags4, read_ags4_bytes};
    use tempfile::tempdir;

    // --- Rule 19 heading guard ------------------------------------

    #[test]
    fn rule_19_accepts_canonical_headings() {
        assert!(matches_rule_19_heading("LOCA_ID"));
        assert!(matches_rule_19_heading("PROJ_ID"));
        assert!(matches_rule_19_heading("ABCD_E")); // 1-char suffix
        assert!(matches_rule_19_heading("ABCD_WXYZ")); // 4-char suffix
        assert!(matches_rule_19_heading("AB12_3D")); // digits allowed
    }

    #[test]
    fn rule_19_rejects_malformed_headings() {
        assert!(!matches_rule_19_heading("HEADING")); // no underscore
        assert!(!matches_rule_19_heading("ABC_ID")); // 3-char group
        assert!(!matches_rule_19_heading("ABCDE_ID")); // 5-char group, 'E' breaks
        assert!(!matches_rule_19_heading("ABCD_")); // empty suffix
        assert!(!matches_rule_19_heading("ABCD_ABCDE")); // 5-char suffix
        assert!(!matches_rule_19_heading("abcd_id")); // lowercase
        assert!(!matches_rule_19_heading("AB-D_ID")); // bad char in group
        assert!(!matches_rule_19_heading("ABCD-ID")); // wrong separator
        assert!(!matches_rule_19_heading("ABC")); // too short overall
    }

    // --- NumericFormat / SF formatter ------------------------------

    #[test]
    fn numeric_format_from_spec_resolves_each_family() {
        assert!(matches!(
            NumericFormat::from_spec("2DP"),
            Some(NumericFormat::Dp(2))
        ));
        assert!(matches!(
            NumericFormat::from_spec("3SF"),
            Some(NumericFormat::Sf(3))
        ));
        assert!(matches!(
            NumericFormat::from_spec("1SCI"),
            Some(NumericFormat::Sci(1))
        ));
        // case-insensitive
        assert!(matches!(
            NumericFormat::from_spec("2dp"),
            Some(NumericFormat::Dp(2))
        ));
        // non-numeric specs / bare suffixes are not numeric formats
        assert!(NumericFormat::from_spec("X").is_none());
        assert!(NumericFormat::from_spec("DP").is_none());
        assert!(NumericFormat::from_spec("XDP").is_none());
    }

    #[test]
    fn numeric_format_format_dp_sci() {
        assert_eq!(NumericFormat::Dp(2).format(100.5), "100.50");
        assert_eq!(NumericFormat::Dp(0).format(7.9), "8");
        assert_eq!(NumericFormat::Sci(2).format(12345.0), "1.23E4");
    }

    #[test]
    fn format_sf_small_and_large_and_zero() {
        // small magnitudes keep precision via fractional digits
        assert_eq!(format_sf(0.002, 3), "0.00200");
        // large magnitudes round to nearest 10^k (i < 0 branch)
        assert_eq!(format_sf(1234.0, 3), "1230");
        // zero short-circuits
        assert_eq!(format_sf(0.0, 3), "0");
        // value with magnitude needing exactly the i==0 path
        assert_eq!(format_sf(5.0, 1), "5");
    }

    #[test]
    fn format_sf_count_is_clamped_so_a_crafted_type_cannot_dos() {
        // The SF count comes straight from the file's TYPE spec ("3SF") with no
        // upper bound (laterite-dev#610 Class B, O-49). python-ags4's `_format_SF` reads it
        // the same way at arbitrary precision, so a crafted "9999999999SF" makes
        // it request a ~10-billion-place width and OOM. We clamp to
        // MAX_NUMERIC_COUNT first, so an absurd count collapses to a bounded string.
        let ceiling = format_sf(1.5, MAX_NUMERIC_COUNT);
        assert_eq!(format_sf(1.5, 9_999_999_999), ceiling);
        assert_eq!(format_sf(1.5, usize::MAX), ceiling); // saturating clamp path
        assert!(ceiling.len() < 40, "clamped output stays bounded");
        // Legit SF counts (≤ the ceiling) render exactly as before.
        assert_eq!(format_sf(0.002, 3), "0.00200");
        assert_eq!(format_sf(1234.0, 3), "1230");
        // The Dp / Sci enum arms feed `n` straight into the width — clamp too.
        assert!(NumericFormat::Dp(usize::MAX).format(1.5).len() < 40);
        assert!(NumericFormat::Sci(usize::MAX).format(1.5).len() < 40);
        assert_eq!(NumericFormat::Dp(3).format(1.5), "1.500"); // legit untouched
        assert_eq!(NumericFormat::Sci(2).format(1500.0), "1.50E3");
    }

    // --- cell_str over every calamine Data variant -----------------

    #[test]
    fn cell_str_handles_all_data_variants() {
        assert_eq!(cell_str(&Data::Empty), "");
        assert_eq!(cell_str(&Data::String("hi".into())), "hi");
        // integer-valued float strips the trailing .0
        assert_eq!(cell_str(&Data::Float(5.0)), "5");
        // fractional float keeps decimals
        assert_eq!(cell_str(&Data::Float(5.5)), "5.5");
        assert_eq!(cell_str(&Data::Int(42)), "42");
        assert_eq!(cell_str(&Data::Bool(true)), "true");
        assert_eq!(
            cell_str(&Data::DateTimeIso("2020-01-01".into())),
            "2020-01-01"
        );
        assert_eq!(cell_str(&Data::DurationIso("PT1H".into())), "PT1H");
    }

    // --- apply_type_formatting (the numeric re-format pass) --------

    #[test]
    fn apply_type_formatting_pads_and_skips() {
        let headings = vec!["TEST_VAL".to_string(), "TEST_TXT".to_string()];
        let types = vec!["3DP".to_string(), "X".to_string()];
        let mut rows = vec![
            vec!["5.1".to_string(), "keep".to_string()],
            vec![String::new()],        // empty -> skipped
            vec!["notnum".to_string()], // unparseable -> untouched
        ];
        apply_type_formatting(&headings, &types, &mut rows);
        assert_eq!(rows[0][0], "5.100"); // padded to 3DP
        assert_eq!(rows[0][1], "keep"); // non-numeric type untouched
        assert_eq!(rows[1][0], ""); // empty stays empty
        assert_eq!(rows[2][0], "notnum"); // parse failure -> verbatim
    }

    #[test]
    fn apply_type_formatting_skips_blank_and_missing_specs() {
        let headings = vec!["TEST_A".to_string(), "TEST_B".to_string()];
        // TEST_A has a blank type; TEST_B has no type entry at all.
        let types = vec![String::new()];
        let mut rows = vec![vec!["1.5".to_string(), "2.5".to_string()]];
        apply_type_formatting(&headings, &types, &mut rows);
        assert_eq!(rows[0][0], "1.5"); // blank spec -> untouched
        assert_eq!(rows[0][1], "2.5"); // missing spec -> untouched
    }

    // --- round trip: AGS4 -> XLSX -> AGS4 --------------------------

    const SAMPLE_AGS4: &str = concat!(
        "\"GROUP\",\"PROJ\"\r\n",
        "\"HEADING\",\"PROJ_ID\",\"PROJ_NAME\"\r\n",
        "\"UNIT\",\"\",\"\"\r\n",
        "\"TYPE\",\"ID\",\"X\"\r\n",
        "\"DATA\",\"P1\",\"Demo Project\"\r\n",
        "\r\n",
        "\"GROUP\",\"LOCA\"\r\n",
        "\"HEADING\",\"LOCA_ID\",\"LOCA_NATE\"\r\n",
        "\"UNIT\",\"\",\"m\"\r\n",
        "\"TYPE\",\"ID\",\"2DP\"\r\n",
        "\"DATA\",\"BH01\",\"523145.10\"\r\n",
        "\"DATA\",\"BH02\",\"523200.00\"\r\n",
    );

    /// The UNIT/TYPE rows must actually SURVIVE the xlsx round trip.
    ///
    /// The existing round-trip asserted `LOCA_NATE == "523145.10"` on a fixture
    /// whose input was **already** `"523145.10"` — so it passed whether or not
    /// the TYPE row made it across, and `write_group_row` (which writes only the
    /// UNIT and TYPE rows) could be stubbed to `Ok(())` with every assertion
    /// still green. A non-falsifiable assertion, found by mutation sweep
    /// (laterite#127).
    ///
    /// The fix is a fixture where survival CHANGES the value: `523145.1` is only
    /// re-formatted to `523145.10` if the `2DP` TYPE row round-tripped. Losing
    /// the TYPE row, shifting it a column (so its tag is overwritten and the row
    /// is dropped as unrecognised), or dropping the reader's `"UNIT"`/`"TYPE"`
    /// match arms all now fail here.
    #[test]
    fn unit_and_type_rows_survive_the_round_trip_and_drive_formatting() {
        let src = concat!(
            "\"GROUP\",\"LOCA\"\r\n",
            "\"HEADING\",\"LOCA_ID\",\"LOCA_NATE\"\r\n",
            "\"UNIT\",\"\",\"m\"\r\n",
            "\"TYPE\",\"ID\",\"2DP\"\r\n",
            // deliberately NOT already 2DP-formatted
            "\"DATA\",\"BH01\",\"523145.1\"\r\n",
        );
        let (xlsx, _) = ags4_bytes_to_xlsx(src.as_bytes(), None).unwrap();
        let (ags4, _) = xlsx_bytes_to_ags4(&xlsx, true).unwrap();
        let back = read_ags4_bytes(&ags4).unwrap();
        let loca = back.get("LOCA").unwrap();

        assert_eq!(
            loca.units(),
            [String::new(), "m".to_string()],
            "the UNIT row must survive the trip through the sheet"
        );
        assert_eq!(
            loca.types(),
            ["ID".to_string(), "2DP".to_string()],
            "the TYPE row must survive — it is what drives re-formatting"
        );
        assert_eq!(
            loca.cell_named(0, "LOCA_NATE").unwrap(),
            "523145.10",
            "2DP re-formatting only happens if the TYPE row came back"
        );
    }

    /// `column_width` mirrors python-ags4's `min(max(13, max_len + 1), 75)`, so
    /// it is parity behaviour rather than decoration — and nothing asserted it,
    /// leaving the whole function replaceable by a constant.
    #[test]
    // Exact comparison is right: the function clamps a usize into f64, so
    // every value it can return is exactly representable.
    #[allow(clippy::float_cmp)]
    fn column_width_matches_the_python_ags4_formula() {
        let mk = |headings: Vec<&str>, units: Vec<&str>, types: Vec<&str>, values: Vec<&str>| {
            AgsGroup::from_owned_rows(
                "LOCA".into(),
                headings.iter().map(|s| (*s).to_string()).collect(),
                units.iter().map(|s| (*s).to_string()).collect(),
                types.iter().map(|s| (*s).to_string()).collect(),
                vec![values.iter().map(|v| (*v).to_string()).collect()],
            )
        };

        // Short content clamps UP to the 13 floor.
        let g = mk(vec!["LOCA_ID"], vec![""], vec!["ID"], vec!["BH01"]);
        assert_eq!(column_width("LOCA_ID", &g), 13.0);

        // Mid content is exactly max_len + 1 — the unclamped band, which is the
        // only place `+ 1` is observable (a `-` or `*` here changes the number).
        let v = "x".repeat(29);
        let g = mk(vec!["LOCA_ID"], vec![""], vec!["ID"], vec![&v]);
        assert_eq!(column_width("LOCA_ID", &g), 30.0, "29 + 1, not 29 or 28");

        // Long content clamps DOWN to the 75 ceiling.
        let v = "x".repeat(200);
        let g = mk(vec!["LOCA_ID"], vec![""], vec!["ID"], vec![&v]);
        assert_eq!(column_width("LOCA_ID", &g), 75.0);
    }

    /// The UNIT/TYPE lookup must use the heading's OWN column index. Matching on
    /// `!=` would take the first *other* column and size against its metadata.
    #[test]
    // Exact comparison is right: the function clamps a usize into f64, so
    // every value it can return is exactly representable.
    #[allow(clippy::float_cmp)]
    fn column_width_reads_unit_and_type_at_the_headings_own_index() {
        // Column 0 carries a long UNIT; column 1's own UNIT/TYPE are short. If
        // the index match inverts, LOCA_ID's width is computed from column 1 and
        // LOCA_NATE's from column 0 — so LOCA_NATE would inherit the long unit.
        let long_unit = "u".repeat(40);
        let g = AgsGroup::from_owned_rows(
            "LOCA".into(),
            vec!["LOCA_NATE".into(), "LOCA_ID".into()],
            vec![long_unit, String::new()],
            vec!["2DP".into(), "ID".into()],
            vec![vec!["1.0".to_string(), "BH01".to_string()]],
        );

        assert_eq!(
            column_width("LOCA_NATE", &g),
            41.0,
            "LOCA_NATE is column 0 and owns the 40-char unit"
        );
        assert_eq!(
            column_width("LOCA_ID", &g),
            13.0,
            "LOCA_ID is column 1: its own unit is blank, so it clamps to the floor"
        );
    }

    /// The all-digits guard in `numeric_prefix` is load-bearing, not redundant
    /// with the `parse` that follows it.
    ///
    /// It looks redundant — anything non-numeric would fail `parse::<usize>()`
    /// anyway — but Rust's integer `FromStr` **accepts a leading `+`**, so
    /// `"+5".parse::<usize>()` is `Ok(5)`. Drop the guard and a malformed TYPE of
    /// `"+5DP"` silently becomes a valid 5-decimal-place format instead of being
    /// rejected. Mutation sweep flagged this as a survivor (laterite#127) and it
    /// was nearly dismissed as an equivalent mutant.
    #[test]
    fn numeric_prefix_rejects_signs_that_rust_would_otherwise_parse() {
        assert_eq!(numeric_prefix("5DP", "DP"), Some(5), "the ordinary case");
        assert_eq!(
            numeric_prefix("+5DP", "DP"),
            None,
            "`+5` parses as 5 in Rust — the digit guard is what rejects it"
        );
        assert_eq!(numeric_prefix("DP", "DP"), None, "empty prefix");
        assert_eq!(numeric_prefix("A5DP", "DP"), None, "non-digit prefix");
        assert_eq!(numeric_prefix("5SF", "DP"), None, "suffix does not match");
    }

    /// A blank row in a sheet is *skipped*, not warned about.
    ///
    /// Only reachable from a hand-edited or third-party workbook — our own
    /// writer never emits one — so no round-trip test can cover it, and the arm
    /// survived the sweep. Built here with `rust_xlsxwriter` directly so the
    /// empty-row path is actually exercised: delete the `""` arm and this row
    /// falls through to the catch-all and produces a spurious warning.
    #[test]
    fn a_blank_row_is_skipped_without_a_warning() {
        let mut wb = Workbook::new();
        let sheet = wb.add_worksheet().set_name("LOCA").unwrap();
        sheet.write_string(0, 0, "HEADING").unwrap();
        sheet.write_string(0, 1, "LOCA_ID").unwrap();
        sheet.write_string(1, 0, "UNIT").unwrap();
        sheet.write_string(2, 0, "TYPE").unwrap();
        sheet.write_string(2, 1, "ID").unwrap();
        // row 3 left entirely empty — the separator case
        sheet.write_string(4, 0, "DATA").unwrap();
        sheet.write_string(4, 1, "BH01").unwrap();
        let xlsx = wb.save_to_buffer().unwrap();

        let (ags4, stats) = xlsx_bytes_to_ags4(&xlsx, true).unwrap();
        assert_eq!(
            stats.warnings,
            Vec::<String>::new(),
            "a blank row is an expected separator, not something to warn about"
        );
        assert_eq!(stats.rows_written, 1, "only the DATA row counts");
        let back = read_ags4_bytes(&ags4).unwrap();
        assert_eq!(
            back.get("LOCA").unwrap().cell_named(0, "LOCA_ID"),
            Some("BH01")
        );
    }

    /// The `i < 0` boundary in `format_sf`. At `i == 0` the two arms are NOT
    /// interchangeable: `{:.0}` rounds half-to-even (matching python-ags4's
    /// `f"{v:.0f}"`), while the `i < 0` arm's `.round()` rounds half away from
    /// zero. `12.5` at 2SF is exactly that boundary — "12" if the comparison is
    /// right, "13" if it is `<=`.
    #[test]
    fn format_sf_boundary_at_i_zero_keeps_python_rounding() {
        assert_eq!(
            format_sf(12.5, 2),
            "12",
            "2SF of 12.5 → i == 0 → half-to-even, as python-ags4 formats it"
        );
        assert_eq!(
            format_sf(13.5, 2),
            "14",
            "half-to-even rounds 13.5 up to 14"
        );
        // i > 0 and i < 0 either side, so the boundary is pinned from both.
        assert_eq!(format_sf(12.5, 3), "12.5");
        assert_eq!(format_sf(1250.0, 2), "1300");
    }

    #[test]
    fn ags4_to_excel_then_back_round_trips() {
        let dir = tempdir().unwrap();
        let ags_in = dir.path().join("in.ags");
        let xlsx = dir.path().join("mid.xlsx");
        let ags_out = dir.path().join("out.ags");
        std::fs::write(&ags_in, SAMPLE_AGS4).unwrap();

        let w = ags4_to_excel(&ags_in, &xlsx, None).unwrap();
        assert_eq!(w.sheets_written, 2);
        assert_eq!(w.rows_written, 3); // 1 PROJ + 2 LOCA
        assert!(xlsx.exists());

        let r = excel_to_ags4(&xlsx, &ags_out, true).unwrap();
        assert_eq!(r.sheets_written, 2);
        assert_eq!(r.rows_written, 3);

        // The re-emitted AGS4 carries both groups and the data values.
        let reparsed = read_ags4(&ags_out).unwrap();
        assert!(reparsed.get("PROJ").is_some());
        let loca = reparsed.get("LOCA").unwrap();
        assert_eq!(loca.n_rows(), 2);
        assert_eq!(loca.cell_named(0, "LOCA_ID"), Some("BH01"));
        // 2DP numeric formatting preserved through the round trip.
        assert_eq!(loca.cell_named(0, "LOCA_NATE"), Some("523145.10"));
    }

    #[test]
    fn bytes_round_trips_with_no_filesystem() {
        // The FS-free path the wasm surface (laterite-dev#359) drives: bytes in, bytes out,
        // no temp files. Must match the path round-trip exactly.
        let (xlsx, w) = ags4_bytes_to_xlsx(SAMPLE_AGS4.as_bytes(), None).unwrap();
        assert_eq!(w.sheets_written, 2);
        assert_eq!(w.rows_written, 3); // 1 PROJ + 2 LOCA
        // A real .xlsx is a zip container — the magic bytes are "PK".
        assert_eq!(&xlsx[0..2], b"PK");

        let (ags4, r) = xlsx_bytes_to_ags4(&xlsx, true).unwrap();
        assert_eq!(r.sheets_written, 2);
        assert_eq!(r.rows_written, 3);

        let reparsed = read_ags4_bytes(&ags4).unwrap();
        assert!(reparsed.get("PROJ").is_some());
        let loca = reparsed.get("LOCA").unwrap();
        assert_eq!(loca.n_rows(), 2);
        assert_eq!(loca.cell_named(0, "LOCA_ID"), Some("BH01"));
        // 2DP numeric formatting survives the FS-free round trip too.
        assert_eq!(loca.cell_named(0, "LOCA_NATE"), Some("523145.10"));
    }

    #[test]
    fn ags4_to_excel_honours_ordered_keys_and_warns_on_missing() {
        let dir = tempdir().unwrap();
        let ags_in = dir.path().join("in.ags");
        let xlsx = dir.path().join("mid.xlsx");
        std::fs::write(&ags_in, SAMPLE_AGS4).unwrap();

        // Request LOCA first, then a non-existent group.
        let order = vec!["LOCA".to_string(), "NOPE".to_string()];
        let stats = ags4_to_excel(&ags_in, &xlsx, Some(order)).unwrap();
        assert_eq!(stats.sheets_written, 1); // only LOCA written
        assert!(stats.warnings.iter().any(|w| w.contains("NOPE")));
    }

    #[test]
    fn ags4_to_excel_empty_order_errors() {
        let dir = tempdir().unwrap();
        let ags_in = dir.path().join("in.ags");
        let xlsx = dir.path().join("mid.xlsx");
        std::fs::write(&ags_in, SAMPLE_AGS4).unwrap();
        // An explicit empty order produces no groups -> Schema error.
        let err = ags4_to_excel(&ags_in, &xlsx, Some(vec![])).unwrap_err();
        assert!(matches!(err, CliError::Schema(_)));
    }

    #[test]
    fn excel_to_ags4_drops_bad_columns_and_rows() {
        // Build an XLSX by hand with a HEADING column, one Rule-19-valid
        // heading, one invalid heading, and a stray non-UNIT/TYPE/DATA row.
        let dir = tempdir().unwrap();
        let xlsx = dir.path().join("hand.xlsx");
        {
            let mut wb = Workbook::new();
            let sheet = wb.add_worksheet().set_name("TEST").unwrap();
            sheet.write_string(0, 0, "HEADING").unwrap();
            sheet.write_string(0, 1, "TEST_ID").unwrap();
            sheet.write_string(0, 2, "badcol").unwrap(); // dropped by Rule 19
            sheet.write_string(1, 0, "UNIT").unwrap();
            sheet.write_string(2, 0, "TYPE").unwrap();
            sheet.write_string(2, 1, "ID").unwrap();
            sheet.write_string(3, 0, "DATA").unwrap();
            sheet.write_string(3, 1, "A1").unwrap();
            sheet.write_string(4, 0, "NOTE").unwrap(); // dropped (not UNIT/TYPE/DATA)
            wb.save(&xlsx).unwrap();
        }
        let ags_out = dir.path().join("out.ags");
        let stats = excel_to_ags4(&xlsx, &ags_out, false).unwrap();
        assert_eq!(stats.sheets_written, 1);
        assert_eq!(stats.rows_written, 1);
        assert!(stats.warnings.iter().any(|w| w.contains("badcol")));
        assert!(stats.warnings.iter().any(|w| w.contains("NOTE")));

        let parsed = read_ags4(&ags_out).unwrap();
        let g = parsed.get("TEST").unwrap();
        assert_eq!(g.headings(), ["TEST_ID".to_string()]);
        assert_eq!(g.cell_named(0, "TEST_ID"), Some("A1"));
    }

    #[test]
    fn excel_to_ags4_skips_sheet_without_heading_column() {
        let dir = tempdir().unwrap();
        let xlsx = dir.path().join("noheading.xlsx");
        {
            let mut wb = Workbook::new();
            // Sheet without a HEADING column -> dropped with warning.
            let bad = wb.add_worksheet().set_name("BAD").unwrap();
            bad.write_string(0, 0, "FOO").unwrap();
            bad.write_string(0, 1, "BAR").unwrap();
            // A valid sheet so the whole call still succeeds.
            let good = wb.add_worksheet().set_name("TEST").unwrap();
            good.write_string(0, 0, "HEADING").unwrap();
            good.write_string(0, 1, "TEST_ID").unwrap();
            good.write_string(1, 0, "DATA").unwrap();
            good.write_string(1, 1, "X1").unwrap();
            wb.save(&xlsx).unwrap();
        }
        let ags_out = dir.path().join("out.ags");
        let stats = excel_to_ags4(&xlsx, &ags_out, false).unwrap();
        assert_eq!(stats.sheets_written, 1);
        assert!(
            stats
                .warnings
                .iter()
                .any(|w| w.contains("BAD") && w.contains("HEADING column"))
        );
    }

    #[test]
    fn excel_to_ags4_all_sheets_invalid_errors() {
        let dir = tempdir().unwrap();
        let xlsx = dir.path().join("allbad.xlsx");
        {
            let mut wb = Workbook::new();
            let bad = wb.add_worksheet().set_name("BAD").unwrap();
            bad.write_string(0, 0, "FOO").unwrap();
            wb.save(&xlsx).unwrap();
        }
        let ags_out = dir.path().join("out.ags");
        let err = excel_to_ags4(&xlsx, &ags_out, false).unwrap_err();
        assert!(matches!(err, CliError::Schema(_)));
    }

    #[test]
    fn excel_to_ags4_missing_input_errors() {
        let dir = tempdir().unwrap();
        let missing = dir.path().join("nope.xlsx");
        let ags_out = dir.path().join("out.ags");
        let err = excel_to_ags4(&missing, &ags_out, false).unwrap_err();
        assert!(matches!(err, CliError::Schema(_)));
    }
}

/// **The numeric-formatter authority gate.**
///
/// AGS4's numeric spelling (`"0.00"`, `"1.23e4"`, `"1230"` …) is defined by ONE function,
/// [`laterite_ags4_types::ags4_str`]. This crate's [`NumericFormat`] is a *hand-copy* of that
/// logic — it deps `laterite-ags4-types` nowhere at runtime — and a hand-copy is a thing that
/// drifts. It has: excel spells `nSCI` with an **uppercase `E`** where the authority uses
/// lowercase `e`, and excel spells `nSF` of zero as a bare `"0"` where the authority pads
/// to the figure count (`"0.00"`).
///
/// Those two divergences are real today, and this test does not hide them — it **explains**
/// them, each by a *rule*. A divergence that matches its rule is a known, registered fact;
/// a divergence that does **not** — excel drifting further from the authority on some third
/// value — is a hard failure. That is the whole point: an opaque allowlist of
/// `(value, spec)` pairs would go green the moment excel broke a *fourth* way on a listed
/// pair, because the pair was already "expected to differ". A rule cannot be fooled that
/// way.
///
/// Why this bug class needs its own gate: no cross-surface value test can see it. Every
/// surface (Python, Node, wasm, CLI) calls each of these formatters as ONE shared Rust
/// function, so they agree with each other forever — while the two *paths* (a direct
/// `build_ags4` through `ags4_str`, and an Excel round-trip through `NumericFormat`)
/// disagree. N-way equality is green by construction; only comparing each formatter to the
/// single authority finds it.
///
/// **Owner-decidable, deliberately surfaced, NOT silently ratified here:** whether excel
/// *should* match the canonical emitter (so an Excel round-trip preserves numeric spelling)
/// or *should* keep matching python-ags4's openpyxl output (uppercase `E`, bare `0`) is a
/// product call. This gate makes the divergence visible and bounded; it does not decide it.
#[cfg(test)]
mod formatter_authority {
    use super::{NumericFormat, format_sf};
    use laterite_ags4_types::{Cell, ags4_str};

    /// Format `value` under `spec` the way the canonical AGS4 emitter does.
    fn authority(value: f64, spec: &str) -> String {
        ags4_str(&Cell::from(value), spec)
    }

    /// Format `value` under `spec` the way an Excel round-trip does.
    fn candidate(spec: &str, value: f64) -> String {
        NumericFormat::from_spec(spec)
            .unwrap_or_else(|| panic!("excel has no formatter for spec {spec:?}"))
            .format(value)
    }

    /// Every `(value, spec)` the gate compares. Chosen to exercise each branch and each
    /// known divergence: DP (plain), SF (zero, sub-1 magnitude, integer rounding, exact),
    /// SCI (zero, large, small, negative).
    const MATRIX: &[(f64, &str)] = &[
        // DP — expected to AGREE (both `{:.n}`).
        (0.0, "0DP"),
        (1.5, "1DP"),
        (-2.345, "2DP"),
        (100.0, "3DP"),
        // SF — zero DIVERGES; the rest AGREE.
        (0.0, "2SF"),
        (0.0, "3SF"),
        (0.002, "3SF"),
        (1234.0, "3SF"),
        (100.0, "3SF"),
        (-5.5, "2SF"),
        // SCI — the exponent-marker case DIVERGES on every non-... value.
        (0.0, "1SCI"),
        (12345.0, "2SCI"),
        (0.00012, "3SCI"),
        (-678.9, "2SCI"),
    ];

    /// A registered divergence: excel's output, related to the authority's by a *rule*.
    /// If the rule holds, the divergence is the known one; if it does not, the formatters
    /// have drifted in a NEW way and the gate fails.
    enum Verdict {
        /// Byte-identical to the authority.
        Agrees,
        /// `nSCI`: excel uppercases the exponent marker and nothing else.
        /// (`1.23e4` → `1.23E4`.) Owner-decidable; see the module doc.
        ExcelUppercasesSciExponent,
        /// `nSF` of exactly zero: excel emits a bare `"0"` where the authority pads to the
        /// figure count. Owner-decidable; see the module doc.
        ExcelDropsSfZeroPadding,
    }

    /// Classify a `(candidate, authority)` pair — or panic if the difference is not one of
    /// the two registered rules. This is the ratchet: a third kind of drift has no arm.
    fn classify(value: f64, spec: &str, cand: &str, auth: &str) -> Verdict {
        if cand == auth {
            return Verdict::Agrees;
        }
        let upper = spec.to_ascii_uppercase();
        if upper.ends_with("SCI") && cand.replace('E', "e") == auth {
            return Verdict::ExcelUppercasesSciExponent;
        }
        if upper.ends_with("SF") && value == 0.0 && cand == "0" {
            return Verdict::ExcelDropsSfZeroPadding;
        }
        panic!(
            "UNREGISTERED formatter drift: {value} under {spec:?} — authority {auth:?}, \
             excel {cand:?}. If this is a deliberate change, add a rule to `classify` (and \
             update the module doc); do not just widen the matrix."
        );
    }

    #[test]
    fn every_matrix_point_is_explained() {
        for &(value, spec) in MATRIX {
            let auth = authority(value, spec);
            let cand = candidate(spec, value);
            // The classify() call is the assertion: it panics on an unregistered drift.
            let _ = classify(value, spec, &cand, &auth);
        }
    }

    /// The two known divergences must ACTUALLY be exercised — otherwise a future edit that
    /// silently makes excel match the authority would leave two dead `Verdict` arms and a
    /// weaker gate than the module doc claims. So we assert the divergence set is non-empty
    /// on both its axes: at least one SCI point and the SF-zero point diverge today.
    #[test]
    fn the_known_divergences_are_present_and_bounded() {
        let mut sci = 0usize;
        let mut sf_zero = 0usize;
        let mut agree = 0usize;
        for &(value, spec) in MATRIX {
            match classify(
                value,
                spec,
                &candidate(spec, value),
                &authority(value, spec),
            ) {
                Verdict::Agrees => agree += 1,
                Verdict::ExcelUppercasesSciExponent => sci += 1,
                Verdict::ExcelDropsSfZeroPadding => sf_zero += 1,
            }
        }
        assert!(
            sci >= 1,
            "the SCI uppercase-E divergence is no longer exercised"
        );
        assert_eq!(
            sf_zero, 2,
            "both SF-zero matrix points must diverge (2SF, 3SF)"
        );
        assert!(
            agree >= 5,
            "the AGREE cases vanished — the authority itself may have moved"
        );
    }

    /// Pin the AUTHORITY to concrete strings. Without this, a change that broke `ags4_str`
    /// to match excel would make the gate go green for the wrong reason — the two would
    /// "agree" at a value neither should produce. These are the canonical AGS4 spellings.
    #[test]
    fn the_authority_produces_the_canonical_spellings() {
        assert_eq!(authority(0.0, "3SF"), "0.00");
        assert_eq!(authority(0.002, "3SF"), "0.00200");
        assert_eq!(authority(1234.0, "3SF"), "1230");
        assert_eq!(authority(12345.0, "2SCI"), "1.23e4");
        assert_eq!(authority(-2.345, "2DP"), "-2.35");
        // And excel's actual output at the two divergence points, so the rules above are
        // anchored to real values, not just to a transform.
        assert_eq!(format_sf(0.0, 3), "0");
        assert_eq!(candidate("2SCI", 12345.0), "1.23E4");
    }
}
