//! AGS4 codec — parse the plain-text AGS4 transfer format.
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
//! Blank lines separate group sections. Mirrors Python `ags5_ags4.codec`'s
//! `read_ags4` semantics.

use std::collections::HashMap;
use std::path::Path;

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

/// Parse an AGS4 file. Uses the `csv` crate with flexible-record mode
/// (rows vary in length: HEADING typically has N+1 fields, DATA has N+1
/// fields but some emitters trim trailing empty fields).
pub fn read_ags4(path: &Path) -> Result<ParsedAgs4, CliError> {
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .flexible(true)
        .trim(csv::Trim::None)
        .from_path(path)
        .map_err(|e| CliError::Schema(format!("open AGS4 file: {}", e)))?;

    let mut groups: HashMap<String, AgsGroup> = HashMap::new();
    let mut order: Vec<String> = Vec::new();
    let mut current_code: Option<String> = None;

    for result in rdr.records() {
        let record = match result {
            Ok(r) => r,
            Err(e) => return Err(CliError::Schema(format!("AGS4 CSV: {}", e))),
        };
        if record.is_empty() {
            continue;
        }
        let tag = record.get(0).unwrap_or("").trim();
        match tag {
            "GROUP" => {
                let code = record
                    .get(1)
                    .ok_or_else(|| CliError::Schema("GROUP row missing group code".into()))?
                    .trim()
                    .to_string();
                if !groups.contains_key(&code) {
                    order.push(code.clone());
                    groups.insert(
                        code.clone(),
                        AgsGroup {
                            code: code.clone(),
                            headings: Vec::new(),
                            units: Vec::new(),
                            types: Vec::new(),
                            rows: Vec::new(),
                        },
                    );
                }
                current_code = Some(code);
            }
            "HEADING" => {
                let code = current_code
                    .as_ref()
                    .ok_or_else(|| CliError::Schema("HEADING row before any GROUP".into()))?;
                let g = groups.get_mut(code).expect("group exists from GROUP row");
                g.headings = record
                    .iter()
                    .skip(1)
                    .map(|s| s.trim().to_string())
                    .collect();
            }
            "UNIT" => {
                let code = current_code
                    .as_ref()
                    .ok_or_else(|| CliError::Schema("UNIT row before any GROUP".into()))?;
                let g = groups.get_mut(code).expect("group exists");
                let mut units: Vec<String> = record
                    .iter()
                    .skip(1)
                    .map(|s| s.trim().to_string())
                    .collect();
                // Pad / truncate to the heading length so column lookups work.
                units.resize(g.headings.len(), String::new());
                g.units = units;
            }
            "TYPE" => {
                let code = current_code
                    .as_ref()
                    .ok_or_else(|| CliError::Schema("TYPE row before any GROUP".into()))?;
                let g = groups.get_mut(code).expect("group exists");
                let mut types: Vec<String> = record
                    .iter()
                    .skip(1)
                    .map(|s| s.trim().to_string())
                    .collect();
                types.resize(g.headings.len(), "X".to_string());
                g.types = types;
            }
            "DATA" => {
                let code = current_code
                    .as_ref()
                    .ok_or_else(|| CliError::Schema("DATA row before any GROUP".into()))?;
                let g = groups.get_mut(code).expect("group exists");
                let mut row: HashMap<String, String> = HashMap::with_capacity(g.headings.len());
                for (i, heading) in g.headings.iter().enumerate() {
                    let v = record.get(i + 1).unwrap_or("").trim();
                    row.insert(heading.clone(), v.to_string());
                }
                g.rows.push(row);
            }
            // Unknown tags / empty rows: skip silently. AGS4 occasionally
            // carries blank-tag rows between sections; that's intentional
            // (visual separation).
            _ => {}
        }
    }

    Ok(ParsedAgs4 { groups, order })
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
}
