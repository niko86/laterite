//! `lat read <FILE> [GROUP]` — dump a group's rows (table / CSV / JSON), or list
//! the file's group codes when no group is named. Reuses core's read codec; no
//! rule engine runs. Errors carry the shared `CliError` exit codes.

use std::process::exit;

use laterite_ags4_core::ags4_codec::{AgsGroup, read_ags4};
use laterite_cliutil::{colour_enabled, styled_table, write_atomic};
use serde_json::{Map, Value};

use crate::cli::ReadArgs;

pub fn run(args: &ReadArgs, json: bool) -> ! {
    let path = args.file.as_path();
    // `read_ags4` maps a missing file to a Schema error (exit 4); a genuine I/O
    // miss should be exit 3, so pre-check existence.
    if !path.exists() {
        eprintln!("error: {}: not found", path.display());
        exit(3);
    }
    let parsed = match read_ags4(path) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("error: {e}");
            exit(e.exit_code());
        }
    };

    let body = match args.group.as_deref() {
        None => list_groups(&parsed.order, json),
        Some(code) => match parsed.get(code) {
            Some(group) => render_group(group, json, args.csv),
            None => {
                let present = if parsed.order.is_empty() {
                    "none".to_string()
                } else {
                    parsed.order.join(", ")
                };
                eprintln!(
                    "error: group {code:?} not found in {} (present: {present})",
                    path.display()
                );
                exit(4);
            }
        },
    };

    if let Some(p) = args.out.as_deref() {
        if let Err(e) = write_atomic(p, body.as_bytes()) {
            eprintln!("error: --out {}: {e}", p.display());
            exit(3);
        }
        eprintln!("note: written to {}", p.display());
    } else {
        print!("{body}");
    }
    exit(0);
}

/// No group named → the file's group codes in source order (one per line, or a
/// JSON array with `--json`). A file with no groups prints nothing + a note.
fn list_groups(order: &[String], json: bool) -> String {
    if order.is_empty() {
        eprintln!("note: no groups in the file");
        return String::new();
    }
    if json {
        serde_json::to_string_pretty(order).unwrap_or_default() + "\n"
    } else {
        let mut s = String::new();
        for code in order {
            s.push_str(code);
            s.push('\n');
        }
        s
    }
}

/// One group's rows. Rows are `HashMap`s, so project through `group.headings`
/// for deterministic column order — never iterate the map directly.
fn render_group(group: &AgsGroup, json: bool, csv: bool) -> String {
    if json {
        let arr: Vec<Value> = group
            .rows
            .iter()
            .map(|row| {
                let mut o = Map::new();
                for h in &group.headings {
                    o.insert(
                        h.clone(),
                        Value::from(row.get(h).cloned().unwrap_or_default()),
                    );
                }
                Value::Object(o)
            })
            .collect();
        serde_json::to_string_pretty(&Value::Array(arr)).unwrap_or_default() + "\n"
    } else if csv {
        let mut s = csv_row(group.headings.iter().map(String::as_str));
        for row in &group.rows {
            s.push_str(&csv_row(
                group
                    .headings
                    .iter()
                    .map(|h| row.get(h).map(String::as_str).unwrap_or("")),
            ));
        }
        s
    } else {
        let headers: Vec<&str> = group.headings.iter().map(String::as_str).collect();
        let rows: Vec<Vec<String>> = group
            .rows
            .iter()
            .map(|row| {
                group
                    .headings
                    .iter()
                    .map(|h| row.get(h).cloned().unwrap_or_default())
                    .collect()
            })
            .collect();
        styled_table(&headers, rows, colour_enabled(false)).to_string() + "\n"
    }
}

/// One RFC-4180-ish CSV line: quote a field iff it contains `,` / `"` / CR / LF,
/// doubling internal quotes. Trailing `\n`.
fn csv_row<'a>(cells: impl Iterator<Item = &'a str>) -> String {
    let fields: Vec<String> = cells
        .map(|c| {
            if c.contains([',', '"', '\r', '\n']) {
                format!("\"{}\"", c.replace('"', "\"\""))
            } else {
                c.to_string()
            }
        })
        .collect();
    fields.join(",") + "\n"
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn group() -> AgsGroup {
        let mut r1 = HashMap::new();
        r1.insert("LOCA_ID".to_string(), "BH01".to_string());
        r1.insert("LOCA_REM".to_string(), "has, a \"comma\"".to_string());
        AgsGroup {
            code: "LOCA".to_string(),
            headings: vec!["LOCA_ID".to_string(), "LOCA_REM".to_string()],
            units: vec![],
            types: vec![],
            rows: vec![r1],
        }
    }

    #[test]
    fn csv_quotes_and_doubles_and_keeps_heading_order() {
        // A cell with a comma + quotes → RFC-4180 quoting; columns in heading order.
        let csv = render_group(&group(), false, true);
        assert_eq!(csv, "LOCA_ID,LOCA_REM\nBH01,\"has, a \"\"comma\"\"\"\n");
    }

    #[test]
    fn json_is_an_array_of_objects_with_raw_string_cells() {
        let v: Value = serde_json::from_str(&render_group(&group(), true, false)).unwrap();
        assert_eq!(v[0]["LOCA_ID"], "BH01");
        // Raw cells: a value stays a string even when it looks numeric elsewhere.
        assert!(v[0]["LOCA_REM"].is_string());
    }

    #[test]
    fn missing_cell_becomes_empty_not_absent() {
        // A row lacking a heading's value renders "" (never a dropped column).
        let g = AgsGroup {
            code: "X".into(),
            headings: vec!["A".into(), "B".into()],
            units: vec![],
            types: vec![],
            rows: vec![HashMap::from([("A".to_string(), "1".to_string())])],
        };
        assert_eq!(render_group(&g, false, true), "A,B\n1,\n");
    }
}
