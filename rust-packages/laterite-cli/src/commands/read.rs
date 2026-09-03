//! `lat read <FILE> [GROUP]` — dump a group's rows (table / CSV / JSON), or list
//! the file's group codes when no group is named. Reuses core's read codec; no
//! rule engine runs. Errors carry the shared `CliError` exit codes.

use std::process::exit;

use laterite_ags4_core::ags4_codec::{
    AgsGroup, DuplicateHeadings, ExcessFields, ReadOptions, read_ags4_with,
};
use laterite_ags4_core::read_render;
use laterite_cliutil::{colour_enabled, styled_table, write_atomic};

use crate::cli::ReadArgs;

/// #893 M8 SPIKE toggle (throwaway branch; never lands). `slab` / `both`
/// source the dumped group's cells straight off the span `ParsedFile`,
/// skipping `from_shared`'s whole-file `HashMap<Arc<str>, String>`
/// re-materialisation; `proj` / `both` skip the `Vec<Vec<String>>`
/// projection copy, rendering the CSV row-by-row. Read per call — crude on
/// purpose, CSV + named-group path only (the lane's bench shape).
fn spike(flag: &str) -> bool {
    std::env::var("LATERITE_M8_SPIKE").is_ok_and(|v| v == flag || v == "both")
}

pub fn run(args: &ReadArgs, json: bool) -> ! {
    let path = args.file.as_path();
    // `read_ags4` maps a missing file to a Schema error (exit 4); a genuine I/O
    // miss should be exit 3, so pre-check existence.
    if !path.exists() {
        eprintln!("error: {}: not found", path.display());
        exit(3);
    }
    // --- SPIKE #893: the M8 variant children take over the bench shape
    // (`lat read <file> <G> --csv`) whole, so the holds are what each child
    // says they are. Everything else falls through to the shipped path.
    if (spike("slab") || spike("proj")) && args.csv && !json {
        if let Some(code) = args.group.as_deref() {
            run_spike_csv(args, code);
        }
    }
    let read_opts = ReadOptions {
        duplicate_headings: if args.recover_duplicate_headings {
            DuplicateHeadings::Recover
        } else {
            DuplicateHeadings::Error
        },
        excess_fields: if args.truncate_excess_fields {
            ExcessFields::Truncate
        } else {
            ExcessFields::Error
        },
    };
    let parsed = match read_ags4_with(path, read_opts) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("error: {e}");
            exit(e.exit_code());
        }
    };

    let body = match args.group.as_deref() {
        None => list_groups(&parsed.order, json),
        Some(code) => {
            if let Some(group) = parsed.get(code) {
                render_group(group, json, args.csv)
            } else {
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

/// #893 M8 SPIKE (throwaway): the three variant children of the CSV dump,
/// each removing one (or both) of the door's re-materialisations while
/// keeping the output byte-identical to the shipped path:
///
/// * `slab`  — parse via the leaf directly and project the dumped group's
///   `Vec<Vec<String>>` straight off the spans (trimmed, ""-padded exactly
///   as `from_shared` does), never building any group's rows-map.
/// * `proj`  — keep the shipped `read_ags4_with` (the map slab builds), but
///   render row-by-row through `csv_row`, never the projection copy.
/// * `both`  — spans → CSV text directly: no maps, no projection copy.
///
/// Exits like `run`: sha-identical `--out` bytes are the equivalence check
/// the probe harness enforces per rung.
fn run_spike_csv(args: &ReadArgs, code: &str) -> ! {
    let path = args.file.as_path();
    let slab = spike("slab");
    let proj = spike("proj");
    let body: String = if slab {
        // The leaf parse, exactly as `read_ags4_bytes_with` configures it.
        let bytes = match std::fs::read(path) {
            Ok(b) => b,
            Err(e) => {
                eprintln!("error: open AGS4 file: {e}");
                exit(4);
            }
        };
        let opts = laterite_ags4_parse::ParseOptions {
            strict_structure: true,
            ..laterite_ags4_parse::ParseOptions::lean()
        };
        let pf = match laterite_ags4_parse::parse_bytes_opts(&bytes, opts) {
            Ok(p) => p,
            Err(e) => {
                // Spike-only wording: the shipped path maps this through
                // core's `map_parse_err`; the probe fixtures never error.
                eprintln!("error: {e:?}");
                exit(4);
            }
        };
        drop(bytes);
        // First-seen on the trimmed code, as `from_shared` resolves it.
        let Some(pg) = pf
            .group_order
            .iter()
            .find(|raw| raw.trim() == code)
            .and_then(|raw| pf.groups.get(raw))
        else {
            eprintln!("error: group {code:?} not found in {}", path.display());
            exit(4);
        };
        let headings: Vec<String> = pg.headings.iter().map(|h| h.trim().to_string()).collect();
        let buf = pg.shared_text();
        if proj {
            // `both`: spans → CSV text directly.
            let mut s = read_render::csv_row(headings.iter().map(String::as_str));
            for r in &pg.rows {
                let spans = pg.row_spans(r);
                s.push_str(&read_render::csv_row((0..headings.len()).map(|i| {
                    spans.get(i).map_or("", |sp| sp.slice(buf).trim())
                })));
            }
            s
        } else {
            // `slab` alone: the projection copy still builds, off the spans.
            let rows: Vec<Vec<String>> = pg
                .rows
                .iter()
                .map(|r| {
                    let spans = pg.row_spans(r);
                    (0..headings.len())
                        .map(|i| {
                            spans
                                .get(i)
                                .map_or_else(String::new, |sp| sp.slice(buf).trim().to_string())
                        })
                        .collect()
                })
                .collect();
            read_render::render_rows_csv(&headings, &rows)
        }
    } else {
        // `proj` alone: the shipped read (map slab builds), streamed render.
        let read_opts = ReadOptions {
            duplicate_headings: if args.recover_duplicate_headings {
                DuplicateHeadings::Recover
            } else {
                DuplicateHeadings::Error
            },
            excess_fields: if args.truncate_excess_fields {
                ExcessFields::Truncate
            } else {
                ExcessFields::Error
            },
        };
        let parsed = match read_ags4_with(path, read_opts) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("error: {e}");
                exit(e.exit_code());
            }
        };
        let Some(group) = parsed.get(code) else {
            eprintln!("error: group {code:?} not found in {}", path.display());
            exit(4);
        };
        let mut s = read_render::csv_row(group.headings.iter().map(String::as_str));
        for row in &group.rows {
            s.push_str(&read_render::csv_row(
                group
                    .headings
                    .iter()
                    .map(|h| row.get(h.as_str()).map_or("", String::as_str)),
            ));
        }
        s
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
    // Project the HashMap rows through `headings` once — the shared renderers
    // take positional rows (the shape the bindings already hand every surface).
    let rows: Vec<Vec<String>> = group
        .rows
        .iter()
        .map(|row| {
            group
                .headings
                .iter()
                .map(|h| row.get(h.as_str()).cloned().unwrap_or_default())
                .collect()
        })
        .collect();
    if json {
        // JSON + CSV come from core's `read_render` — the same writers Node and
        // Python now call, so `read --json`/`--csv` is one format, not three
        // hand-synced ones (laterite-dev#530). The table stays local: it renders through
        // `laterite-cliutil`'s styled grid, a CLI-only concern.
        read_render::render_rows_json(&group.headings, &rows)
    } else if csv {
        read_render::render_rows_csv(&group.headings, &rows)
    } else {
        let headers: Vec<&str> = group.headings.iter().map(String::as_str).collect();
        styled_table(&headers, rows, colour_enabled(false)).to_string() + "\n"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;
    use std::collections::HashMap;
    use std::sync::Arc;

    fn group() -> AgsGroup {
        let mut r1 = HashMap::new();
        r1.insert(Arc::from("LOCA_ID"), "BH01".to_string());
        r1.insert(Arc::from("LOCA_REM"), "has, a \"comma\"".to_string());
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
            rows: vec![HashMap::from([(Arc::from("A"), "1".to_string())])],
        };
        assert_eq!(render_group(&g, false, true), "A,B\n1,\n");
    }
}
