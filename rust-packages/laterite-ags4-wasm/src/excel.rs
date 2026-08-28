//! AGS4 ↔ XLSX (#359).
//!
//! The FS-free `laterite-ags4-excel` cores (`ags4_bytes_to_xlsx` /
//! `xlsx_bytes_to_ags4`) drive the browser Excel surface: the Tools pane hands
//! us bytes and gets bytes + warnings back, no filesystem. calamine reads and
//! rust_xlsxwriter writes — both pure-Rust and wasm-clean.
//!
//! Behind the `excel` feature (#330), and the heaviest gate after `arrow` — a
//! page that never opens a workbook was paying for all of it. The weights are
//! in the crate's own `[features]` comments.
use wasm_bindgen::prelude::*;

/// The result of an Excel conversion: the output `bytes` (a JS `Uint8Array` —
/// the `.xlsx` or `.ags` file), plus the `warnings` and counts the UI surfaces
/// (dropped non-Rule-19 columns, skipped sheets, …).
#[cfg(feature = "excel")]
#[wasm_bindgen]
pub struct ExcelResult {
    bytes: Vec<u8>,
    warnings: Vec<String>,
    sheets: usize,
    rows: usize,
}

#[cfg(feature = "excel")]
#[wasm_bindgen]
impl ExcelResult {
    #[wasm_bindgen(getter)]
    pub fn bytes(&self) -> Vec<u8> {
        self.bytes.clone()
    }
    #[wasm_bindgen(getter)]
    pub fn warnings(&self) -> Vec<String> {
        self.warnings.clone()
    }
    #[wasm_bindgen(getter)]
    pub fn sheets(&self) -> usize {
        self.sheets
    }
    #[wasm_bindgen(getter)]
    pub fn rows(&self) -> usize {
        self.rows
    }
}

/// AGS4 bytes → an `.xlsx` workbook (one sheet per group, python-ags4's
/// layout). `JsError` if the input carries no valid AGS4 groups.
#[cfg(feature = "excel")]
#[wasm_bindgen]
pub fn ags4_to_xlsx(
    data: &[u8],
    recover_duplicate_headings: Option<bool>,
    truncate_excess_fields: Option<bool>,
) -> Result<ExcelResult, JsError> {
    console_error_panic_hook::set_once();
    ags4_to_xlsx_core(
        data,
        recover_duplicate_headings.unwrap_or(false),
        truncate_excess_fields.unwrap_or(false),
    )
    .map_err(|m| JsError::new(&m))
}

/// The host-testable core of [`ags4_to_xlsx`].
#[cfg(feature = "excel")]
fn ags4_to_xlsx_core(
    data: &[u8],
    recover_duplicate_headings: bool,
    truncate_excess_fields: bool,
) -> Result<ExcelResult, String> {
    use laterite_ags4_core::ags4_codec::{DuplicateHeadings, ExcessFields, ReadOptions};
    // Both leniencies are off by default here as on every read surface; the
    // browser caller opts into the suffixed recovery read, or into discarding
    // fields that bind to no heading.
    let opts = ReadOptions {
        duplicate_headings: if recover_duplicate_headings {
            DuplicateHeadings::Recover
        } else {
            DuplicateHeadings::Error
        },
        excess_fields: if truncate_excess_fields {
            ExcessFields::Truncate
        } else {
            ExcessFields::Error
        },
    };
    let (bytes, stats) = laterite_ags4_excel::ags4_bytes_to_xlsx_with(data, None, opts)
        .map_err(|e| e.to_string())?;
    Ok(ExcelResult {
        bytes,
        warnings: stats.warnings,
        sheets: stats.sheets_written,
        rows: stats.rows_written,
    })
}

/// An `.xlsx` workbook's bytes → AGS4 bytes. Each sheet with a `HEADING` column
/// becomes a group; non-Rule-19 columns and non-`UNIT`/`TYPE`/`DATA` rows are
/// dropped (surfaced in `warnings`). `format_numeric` re-pads DATA cells to
/// their column's TYPE (mirrors python-ags4's `convert_to_text`). `JsError` if
/// no sheet yields a valid group.
#[cfg(feature = "excel")]
#[wasm_bindgen]
pub fn xlsx_to_ags4(data: &[u8], format_numeric: bool) -> Result<ExcelResult, JsError> {
    console_error_panic_hook::set_once();
    xlsx_to_ags4_core(data, format_numeric).map_err(|m| JsError::new(&m))
}

/// The host-testable core of [`xlsx_to_ags4`].
#[cfg(feature = "excel")]
fn xlsx_to_ags4_core(data: &[u8], format_numeric: bool) -> Result<ExcelResult, String> {
    let (bytes, stats) =
        laterite_ags4_excel::xlsx_bytes_to_ags4(data, format_numeric).map_err(|e| e.to_string())?;
    Ok(ExcelResult {
        bytes,
        warnings: stats.warnings,
        sheets: stats.sheets_written,
        rows: stats.rows_written,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testdata::CLEAN;

    // ---------------------------------------------------------------
    // the Excel pair
    // ---------------------------------------------------------------

    #[cfg(feature = "excel")]
    #[test]
    fn an_ags_file_becomes_a_workbook_with_a_sheet_per_group() {
        let res = ags4_to_xlsx_core(CLEAN, false, false).expect("converts");
        assert!(res.sheets() > 0, "no sheets were written");
        assert!(res.rows() > 0, "no rows were written");
        // The xlsx magic — a zip container. Anything else is not a workbook,
        // however plausible the byte count.
        assert_eq!(&res.bytes()[..2], b"PK", "output must be a zip/xlsx");
        assert!(res.warnings().len() < 100, "warnings should be bounded");
    }

    #[cfg(feature = "excel")]
    #[test]
    fn duplicate_headings_are_fatal_unless_recovery_is_asked_for() {
        // Fatal by default on every read surface; the browser opts into the
        // suffixed recovery read. Both halves matter — a default that recovered
        // would silently invent column names.
        let dup: &[u8] = b"\"GROUP\",\"LOCA\"\r\n\
\"HEADING\",\"LOCA_ID\",\"LOCA_ID\"\r\n\
\"UNIT\",\"\",\"\"\r\n\
\"TYPE\",\"ID\",\"ID\"\r\n\
\"DATA\",\"BH01\",\"BH01\"\r\n";
        assert!(
            ags4_to_xlsx_core(dup, false, false).is_err(),
            "duplicate headings must be fatal by default"
        );
        assert!(
            ags4_to_xlsx_core(dup, true, false).is_ok(),
            "recovery must be reachable from the browser"
        );
    }

    #[cfg(feature = "excel")]
    #[test]
    fn non_workbook_bytes_are_refused() {
        assert!(xlsx_to_ags4_core(b"not a workbook", false).is_err());
    }

    #[cfg(feature = "excel")]
    #[test]
    fn a_workbook_round_trips_back_to_the_same_groups() {
        // The pair is only useful if it is a pair: converting out and back must
        // preserve the groups, not merely produce *some* AGS4.
        let book = ags4_to_xlsx_core(CLEAN, false, false).expect("to xlsx");
        let back = xlsx_to_ags4_core(&book.bytes(), false).expect("from xlsx");
        let text = String::from_utf8(back.bytes()).expect("utf-8");
        for group in ["PROJ", "TRAN", "UNIT", "TYPE"] {
            assert!(
                text.contains(&format!("\"GROUP\",\"{group}\"")),
                "{group} did not survive the round trip:\n{text}"
            );
        }
    }
}
