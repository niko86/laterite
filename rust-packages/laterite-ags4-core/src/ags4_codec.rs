//! AGS4 codec — read the plain-text AGS4 transfer format into core's
//! name-keyed [`ParsedAgs4`].
//!
//! AGS4 files are CSV-like: every field is double-quoted, embedded
//! quotes become `""`, and each row's first field is a TAG that
//! disambiguates its purpose:
//!
//!   "GROUP","LOCA"
//!   "HEADING","LOCA_ID","LOCA_TYPE",...
//!   "UNIT","","","",...
//!   "TYPE","ID","PA","2DP",...
//!   "DATA","BH01","CP","100.50",...
//!   <blank line>
//!   "GROUP","SAMP"
//!   ...
//!
//! Blank lines separate group sections. Since #168 Phase 5 the actual parsing
//! is the shared leaf ([`laterite_ags4_parse`]) — one tokenizer + one
//! source-true walk for the whole toolchain; [`from_shared`] projects the
//! leaf's positional, *raw* `ParsedFile` into core's name-keyed, *trimmed*
//! shape (re-applying core's trims + UNIT/TYPE padding). The read path opts
//! into the leaf's strict-structure mode (a data reader fails fast on a
//! HEADING/DATA-before-GROUP or a code-less GROUP; the validator keeps the
//! lenient default and reports those as findings instead).

use std::collections::HashMap;
use std::path::Path;

use laterite_ags4_parse::{ParseError, ParseOptions, ParsedFile, parse_bytes_opts};

use crate::error::CliError;

/// One AGS4 group section after parsing.
#[derive(Debug, Clone)]
pub struct AgsGroup {
    pub code: String,
    /// Heading names in declaration order (e.g. ["LOCA_ID","LOCA_TYPE",...]).
    pub headings: Vec<String>,
    /// Units row aligned with `headings`. Padded with empty strings if the
    /// UNIT row was shorter than the heading list (some AGS4 emitters).
    pub units: Vec<String>,
    /// TYPE row aligned with `headings` — AGS4 type codes (X / ID / 2DP / …).
    pub types: Vec<String>,
    /// Each DATA row as a `{heading_name: raw_string_value}` map.
    pub rows: Vec<HashMap<String, String>>,
}

/// Whole-file parse result. `order` preserves the group-section order from
/// the input file — useful for round-trip emission.
#[derive(Debug, Clone)]
pub struct ParsedAgs4 {
    pub groups: HashMap<String, AgsGroup>,
    pub order: Vec<String>,
}

impl ParsedAgs4 {
    pub fn get(&self, code: &str) -> Option<&AgsGroup> {
        self.groups.get(code)
    }
}

/// Read an AGS4 file from a path — slurps the bytes and delegates to
/// [`read_ags4_bytes`] (the shared leaf walks the whole buffer at once).
pub fn read_ags4(path: &Path) -> Result<ParsedAgs4, CliError> {
    let bytes =
        std::fs::read(path).map_err(|e| CliError::Schema(format!("open AGS4 file: {e}")))?;
    read_ags4_bytes(&bytes)
}

/// Read AGS4 from an in-memory byte buffer through the shared parse leaf, then
/// project to core's name-keyed [`ParsedAgs4`] (#168 Phase 5). Uses the leaf's
/// lean profile with `strict_structure` ON, so the locked structure terminals
/// (`error_mapping.rs`) are raised exactly as the retired csv reader did. The
/// leaf also strips a leading BOM, so a sliced read of a BOM file is clean.
///
/// A file with **no GROUP rows** is NOT an error for the reader — the retired
/// csv reader returned an empty parse for it, and downstream consumers
/// (e.g. the excel exporter's "No valid AGS4 data found" check) rely on that.
/// So the leaf's `NotAgs4` is mapped back to an empty `ParsedAgs4`; structural
/// violations (pre-GROUP rows, a code-less GROUP) still propagate as errors.
pub fn read_ags4_bytes(bytes: &[u8]) -> Result<ParsedAgs4, CliError> {
    let opts = ParseOptions {
        strict_structure: true,
        ..ParseOptions::lean()
    };
    match parse_bytes_opts(bytes, opts) {
        Ok(parsed) => Ok(from_shared(parsed)),
        Err(ParseError::NotAgs4(_)) => Ok(ParsedAgs4 {
            groups: HashMap::new(),
            order: Vec::new(),
        }),
        Err(e) => Err(map_parse_err(e)),
    }
}

/// Project the shared leaf's positional, RAW [`ParsedFile`] into core's
/// name-keyed, TRIMMED [`ParsedAgs4`] (#168 Phase 5). Re-applies the trims core
/// has always done — the leaf leaves values raw (validator semantics) — on
/// every heading/unit/type/value, pads UNIT to the heading count (empty) and
/// TYPE (with `"X"`), and keys each DATA row by heading name. First-seen wins on
/// a duplicate (trimmed) code, matching the csv reader this replaced.
fn from_shared(pf: ParsedFile) -> ParsedAgs4 {
    let mut groups: HashMap<String, AgsGroup> = HashMap::with_capacity(pf.group_order.len());
    let mut order: Vec<String> = Vec::with_capacity(pf.group_order.len());
    for raw_code in &pf.group_order {
        let code = raw_code.trim().to_string();
        if groups.contains_key(&code) {
            continue; // first-seen wins on the trimmed code (csv-reader parity)
        }
        let pg = &pf.groups[raw_code];
        let headings: Vec<String> = pg.headings.iter().map(|h| h.trim().to_string()).collect();
        // Pad/truncate to the heading count — but ONLY when the row was actually
        // present (the csv reader resized inside its UNIT/TYPE arm; a group with
        // no UNIT row kept an empty vec, never padded). `unit_line`/`type_line`
        // are `Some` iff the leaf saw that descriptor row.
        let mut units: Vec<String> = pg.units.iter().map(|u| u.trim().to_string()).collect();
        if pg.unit_line.is_some() {
            units.resize(headings.len(), String::new());
        }
        let mut types: Vec<String> = pg.types.iter().map(|t| t.trim().to_string()).collect();
        if pg.type_line.is_some() {
            types.resize(headings.len(), "X".to_string());
        }
        let rows: Vec<HashMap<String, String>> = pg
            .rows
            .iter()
            .map(|r| {
                let mut row = HashMap::with_capacity(headings.len());
                for (i, h) in headings.iter().enumerate() {
                    let v = r.values.get(i).map(|s| s.trim()).unwrap_or("");
                    row.insert(h.clone(), v.to_string());
                }
                row
            })
            .collect();
        order.push(code.clone());
        groups.insert(
            code.clone(),
            AgsGroup {
                code,
                headings,
                units,
                types,
                rows,
            },
        );
    }
    ParsedAgs4 { groups, order }
}

/// Map the shared leaf's `ParseError` into core's `CliError`. The structure
/// terminals carry their exact messages verbatim (`error_mapping.rs` pins them);
/// the non-UTF-8 wording is the leaf's — the csv reader's old wording changes
/// (asserted loosely in `error_mapping.rs`, ratified at Phase 7).
pub(crate) fn map_parse_err(e: ParseError) -> CliError {
    let msg = match e {
        ParseError::NotAgs4(m) | ParseError::Structure(m) => m,
        ParseError::UnsupportedEdition { found } => format!("unsupported AGS edition {found:?}"),
        ParseError::NotUtf8 => "input is not valid UTF-8".to_string(),
    };
    CliError::Schema(msg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn write_fixture(path: &Path, content: &str) {
        let mut f = std::fs::File::create(path).unwrap();
        f.write_all(content.as_bytes()).unwrap();
    }

    const FIXTURE: &str = r#""GROUP","PROJ"
"HEADING","PROJ_ID","PROJ_NAME"
"UNIT","",""
"TYPE","X","X"
"DATA","P1","Test project"

"GROUP","LOCA"
"HEADING","LOCA_ID","LOCA_TYPE","LOCA_NATE"
"UNIT","","","m"
"TYPE","ID","PA","2DP"
"DATA","BH01","CP","100.50"
"DATA","BH02","TP","200.75"
"#;

    #[test]
    fn parses_two_groups_in_order() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.ags");
        write_fixture(&path, FIXTURE);

        let parsed = read_ags4(&path).unwrap();
        assert_eq!(parsed.order, vec!["PROJ", "LOCA"]);

        let proj = parsed.get("PROJ").unwrap();
        assert_eq!(proj.headings, vec!["PROJ_ID", "PROJ_NAME"]);
        assert_eq!(proj.types, vec!["X", "X"]);
        assert_eq!(proj.rows.len(), 1);
        assert_eq!(proj.rows[0]["PROJ_ID"], "P1");

        let loca = parsed.get("LOCA").unwrap();
        assert_eq!(loca.headings, vec!["LOCA_ID", "LOCA_TYPE", "LOCA_NATE"]);
        assert_eq!(loca.units, vec!["", "", "m"]);
        assert_eq!(loca.rows.len(), 2);
        assert_eq!(loca.rows[1]["LOCA_NATE"], "200.75");
    }

    #[test]
    fn read_ags4_bytes_matches_path() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.ags");
        write_fixture(&path, FIXTURE);

        let from_path = read_ags4(&path).unwrap();
        let from_bytes = read_ags4_bytes(FIXTURE.as_bytes()).unwrap();

        assert_eq!(from_path.order, from_bytes.order);
        for code in &from_path.order {
            let a = from_path.get(code).unwrap();
            let b = from_bytes.get(code).unwrap();
            assert_eq!(a.headings, b.headings);
            assert_eq!(a.types, b.types);
            assert_eq!(a.units, b.units);
            assert_eq!(a.rows, b.rows);
        }
    }

    #[test]
    fn missing_unit_row_leaves_units_empty_not_padded() {
        // A group with NO UNIT row keeps an empty units vec — the retired csv
        // reader only padded inside its UNIT arm, so an absent row was never
        // padded. Regression guard for the from_shared padding fix (#168 Phase 5).
        let f = "\"GROUP\",\"PROJ\"\n\"HEADING\",\"PROJ_ID\",\"PROJ_NAME\"\n\"TYPE\",\"ID\",\"X\"\n\"DATA\",\"P1\",\"n\"\n";
        let parsed = read_ags4_bytes(f.as_bytes()).unwrap();
        let proj = parsed.get("PROJ").unwrap();
        assert!(
            proj.units.is_empty(),
            "no UNIT row → empty units, not padded"
        );
        // a present TYPE row IS padded/aligned to the heading count
        assert_eq!(proj.types, vec!["ID", "X"]);
        assert_eq!(proj.rows[0]["PROJ_ID"], "P1");
    }

    #[test]
    fn read_path_opts_into_strict_structure() {
        // core's reader fails fast on a HEADING before any GROUP (the leaf's
        // strict mode), with the exact locked message error_mapping.rs pins.
        let err = read_ags4_bytes(b"\"HEADING\",\"X\"\n").unwrap_err();
        assert!(
            matches!(err, CliError::Schema(ref m) if m == "HEADING row before any GROUP"),
            "got {err:?}"
        );
    }

    #[test]
    fn no_group_file_reads_as_empty_not_an_error() {
        // A file with no GROUP rows is an EMPTY parse for the reader, not an
        // error — matching the retired csv reader. Downstream consumers (the
        // excel exporter's "No valid AGS4 data found" check) rely on this.
        // Regression guard: Phase 5's leaf returns NotAgs4 here, mapped back to
        // an empty parse (caught the empty-file→xlsx python-ags4 parity test).
        for empty in [b"".as_slice(), b"\r\n\r\n".as_slice()] {
            let parsed = read_ags4_bytes(empty).unwrap();
            assert!(parsed.order.is_empty() && parsed.groups.is_empty());
        }
    }
}
