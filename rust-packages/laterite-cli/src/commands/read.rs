//! `lat read <FILE> [GROUP]` — dump a group's rows (table / CSV / JSON), or list
//! the file's group codes when no group is named. Reuses core's read codec; no
//! rule engine runs. Errors carry the shared `CliError` exit codes.

use std::process::exit;

use laterite_ags4_core::ags4_codec::{AgsGroup, ReadOptions, read_ags4_with};
use laterite_ags4_core::read_render;
use laterite_cliutil::{colour_enabled, styled_table, write_atomic};

use crate::cli::ReadArgs;

pub fn run(args: &ReadArgs, json: bool) -> ! {
    let path = args.file.as_path();
    // `read_ags4` maps a missing file to a Schema error (exit 4); a genuine I/O
    // miss should be exit 3, so pre-check existence.
    if !path.exists() {
        eprintln!("error: {}: not found", path.display());
        exit(3);
    }
    let read_opts =
        ReadOptions::from_flags(args.recover_duplicate_headings, args.truncate_excess_fields);
    let parsed = match read_ags4_with(path, read_opts) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("error: {e}");
            exit(e.exit_code());
        }
    };

    let body = match args.group.as_deref() {
        None => list_groups(parsed.order(), json),
        Some(code) => {
            if let Some(group) = parsed.get(code) {
                render_group(group, json, args.csv)
            } else {
                let present = if parsed.order().is_empty() {
                    "none".to_string()
                } else {
                    parsed.order().join(", ")
                };
                eprintln!(
                    "error: group {code:?} not found in {} (present: {present})",
                    path.display()
                );
                exit(4);
            }
        }
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

/// One group's rows, projected positionally off the span-backed accessors
/// (#900) — ONE group's owned copy, which the #893 diagnosis measured as
/// co-peak shadow (the whole-file slab was the prize, and it is gone).
fn render_group(group: &AgsGroup, json: bool, csv: bool) -> String {
    let rows: Vec<Vec<String>> = (0..group.n_rows()).map(|i| group.padded_row(i)).collect();
    if json {
        // JSON + CSV come from core's `read_render` — the same writers Node and
        // Python now call, so `read --json`/`--csv` is one format, not three
        // hand-synced ones (laterite-dev#530). The table stays local: it renders through
        // `laterite-cliutil`'s styled grid, a CLI-only concern.
        read_render::render_rows_json(group.headings(), &rows)
    } else if csv {
        read_render::render_rows_csv(group.headings(), &rows)
    } else {
        let headers: Vec<&str> = group.headings().iter().map(String::as_str).collect();
        styled_table(&headers, rows, colour_enabled(false)).to_string() + "\n"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    fn group() -> AgsGroup {
        AgsGroup::from_owned_rows(
            "LOCA".to_string(),
            vec!["LOCA_ID".to_string(), "LOCA_REM".to_string()],
            vec![],
            vec![],
            vec![vec!["BH01".to_string(), "has, a \"comma\"".to_string()]],
        )
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
        // A short row renders "" for the missing tail (never a dropped column) —
        // `from_owned_rows` pads to the heading count by construction.
        let g = AgsGroup::from_owned_rows(
            "X".into(),
            vec!["A".into(), "B".into()],
            vec![],
            vec![],
            vec![vec!["1".to_string()]],
        );
        assert_eq!(render_group(&g, false, true), "A,B\n1,\n");
    }
}
