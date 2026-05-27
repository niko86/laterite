//! AGS4 ↔ XLSX conversion — Rust-backed Excel I/O for
//! `laterite.compat.AGS4_to_excel` and `laterite.compat.excel_to_AGS4`.
//!
//! Mirrors python-ags4's openpyxl-based implementation but uses two
//! pure-Rust crates: `rust_xlsxwriter` for writing and `calamine` for
//! reading. No Python deps cross the boundary; outputs match
//! python-ags4's layout (one sheet per group, HEADING column first,
//! UNIT / TYPE / DATA pseudo-rows preserved, column widths
//! `min(max(13, max_str_len + 1), 75)`).
//!
//! Stage 2b of the python-ags4 parity arc.

use std::collections::HashMap;
use std::path::Path;

use calamine::{Data, Reader, open_workbook_auto};
use rust_xlsxwriter::Workbook;

use crate::ags4_codec::{AgsGroup, read_ags4};
use crate::ags4_writer::{EmitGroup, write_ags4};
use crate::error::CliError;

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
    let parsed = read_ags4(input)?;
    let order: Vec<String> = ordered_keys.unwrap_or_else(|| parsed.order.clone());

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
        let Some(group) = parsed.groups.get(code) else {
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
        for (i, heading) in group.headings.iter().enumerate() {
            sheet
                .write_string(0, (i + 1) as u16, heading)
                .map_err(|e| CliError::Schema(format!("write heading: {e}")))?;
        }

        // UNIT, TYPE, then each DATA row.
        write_group_row(sheet, 1, "UNIT", &group.units, group.headings.len())?;
        write_group_row(sheet, 2, "TYPE", &group.types, group.headings.len())?;
        for (ri, row) in group.rows.iter().enumerate() {
            let r = (3 + ri) as u32;
            sheet
                .write_string(r, 0, "DATA")
                .map_err(|e| CliError::Schema(format!("write DATA tag: {e}")))?;
            for (ci, heading) in group.headings.iter().enumerate() {
                let value = row.get(heading).map(String::as_str).unwrap_or("");
                if !value.is_empty() {
                    sheet
                        .write_string(r, (ci + 1) as u16, value)
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
        for (i, heading) in group.headings.iter().enumerate() {
            sheet
                .set_column_width((i + 1) as u16, column_width(heading, group))
                .map_err(|e| CliError::Schema(format!("col width: {e}")))?;
        }

        sheets_written += 1;
    }

    workbook
        .save(output)
        .map_err(|e| CliError::Schema(format!("save xlsx {}: {e}", output.display())))?;
    Ok(ExcelStats {
        sheets_written,
        rows_written,
        warnings,
    })
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
        let value = cells.get(i).map(String::as_str).unwrap_or("");
        if !value.is_empty() {
            sheet
                .write_string(r, (i + 1) as u16, value)
                .map_err(|e| CliError::Schema(format!("write {tag} cell: {e}")))?;
        }
    }
    Ok(())
}

/// Column width matching python-ags4's `min(max(13, max_len+1), 75)`.
/// Considers the heading name itself plus the UNIT / TYPE / every
/// DATA value for that column.
fn column_width(heading: &str, group: &AgsGroup) -> f64 {
    let mut max_len = heading.len();
    // Compare against UNIT/TYPE entries at the same column index.
    if let Some(idx) = group.headings.iter().position(|h| h == heading) {
        if let Some(u) = group.units.get(idx) {
            max_len = max_len.max(u.len());
        }
        if let Some(t) = group.types.get(idx) {
            max_len = max_len.max(t.len());
        }
    }
    // Then every DATA row.
    for row in &group.rows {
        if let Some(v) = row.get(heading) {
            max_len = max_len.max(v.len());
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
    let mut workbook = open_workbook_auto(input)
        .map_err(|e| CliError::Schema(format!("open xlsx {}: {e}", input.display())))?;

    let sheet_names: Vec<String> = workbook.sheet_names().to_vec();
    let mut emit_groups: Vec<AgsGroup> = Vec::new();
    let mut warnings: Vec<String> = Vec::new();
    let mut rows_written = 0usize;

    for sheet_name in &sheet_names {
        let range = workbook
            .worksheet_range(sheet_name)
            .map_err(|e| CliError::Schema(format!("read sheet {sheet_name}: {e}")))?;

        let mut rows_iter = range.rows();
        let header_row = match rows_iter.next() {
            Some(r) => r,
            None => {
                warnings.push(format!("{sheet_name}: empty sheet, skipped"));
                continue;
            }
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
        let mut data_rows: Vec<HashMap<String, String>> = Vec::new();

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
                    let values = payload();
                    let row_map: HashMap<String, String> = headings
                        .iter()
                        .zip(values)
                        .map(|(h, v)| (h.clone(), v))
                        .collect();
                    data_rows.push(row_map);
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

        emit_groups.push(AgsGroup {
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

    // Convert AgsGroup → EmitGroup (borrowed view) and call write_ags4.
    let emit_views: Vec<EmitGroup<'_>> = emit_groups
        .iter()
        .map(|g| EmitGroup {
            code: &g.code,
            headings: g.headings.iter().map(String::as_str).collect(),
            units: g.units.iter().map(String::as_str).collect(),
            types: g.types.iter().map(String::as_str).collect(),
            rows: g
                .rows
                .iter()
                .map(|row| {
                    g.headings
                        .iter()
                        .map(|h| row.get(h).cloned().unwrap_or_default())
                        .collect()
                })
                .collect(),
        })
        .collect();

    let mut out_file = std::fs::File::create(output)
        .map_err(|e| CliError::Schema(format!("create {}: {e}", output.display())))?;
    write_ags4(&mut out_file, &emit_views)?;

    Ok(ExcelStats {
        sheets_written: emit_groups.len(),
        rows_written,
        warnings,
    })
}

/// Re-format every DATA cell whose column's TYPE row declares a
/// numeric AGS spec (`<N>DP` / `<N>SF` / `<N>SCI`). Mirrors
/// python-ags4's `convert_to_text` / `format_numeric_column` chain:
/// pad trailing zeros to the precision the spec demands so AGS4
/// emit matches the column's declared TYPE.
///
/// Non-numeric specs pass through untouched. Cells that can't be
/// parsed as floats (e.g. blank strings, sentinels like `?`) also
/// pass through — matches python-ags4's silent ValueError fallback.
fn apply_type_formatting(
    headings: &[String],
    types: &[String],
    rows: &mut [HashMap<String, String>],
) {
    for (i, heading) in headings.iter().enumerate() {
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
            if let Some(value) = row.get(heading) {
                if value.is_empty() {
                    continue;
                }
                let Ok(parsed) = value.parse::<f64>() else {
                    continue;
                };
                row.insert(heading.clone(), formatter.format(parsed));
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
        match *self {
            Self::Dp(n) => format!("{v:.*}", n),
            Self::Sci(n) => {
                // python-ags4 uses `f"{x:.{N}E}"` — uppercase E with
                // a single digit exponent (e.g. 1.23E+02). Rust's
                // default scientific format matches.
                format!("{v:.*E}", n)
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

/// Significant-figure formatter — mirrors python-ags4's `_format_SF`.
fn format_sf(value: f64, n: usize) -> String {
    if value == 0.0 {
        return format!("{value}");
    }
    let i: i32 = (n as i32) - 1 - value.abs().log10().floor() as i32;
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
        Data::String(s) => s.clone(),
        Data::Float(f) => {
            // Floats from calamine may be 1.0 for ints. Strip the
            // trailing .0 to match the string representation
            // python-ags4 / openpyxl produces for integer-valued cells.
            if f.fract() == 0.0 && f.is_finite() {
                format!("{}", *f as i64)
            } else {
                format!("{f}")
            }
        }
        Data::Int(i) => i.to_string(),
        Data::Bool(b) => b.to_string(),
        Data::DateTime(d) => d.to_string(),
        Data::DateTimeIso(s) => s.clone(),
        Data::DurationIso(s) => s.clone(),
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
