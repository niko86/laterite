//! Findings presentation — the plain table / `--json` / `--ndjson` shaping,
//! lifted **verbatim** from the pre-rework `main.rs` so validate/fix output is
//! byte-identical (the `test_cli_*` byte-parity gate depends on it). One model
//! (`Finding`) serialised everywhere; line-only findings still emit exactly
//! `{line,group,desc}`, migrated ones additively gain the rich keys.

use std::io;
use std::path::Path;

use laterite_ags4_validator::{Findings, findings};
use laterite_cliutil::{colour_enabled, styled_table, write_json_pretty};
use serde_json::Value;

/// Findings flattened to one row per finding, in spec-rule order (the `Findings`
/// map is a `BTreeMap`, so deterministic), rendered through the shared
/// `laterite-cliutil` `UTF8_FULL` grid. `use_color` off → no ANSI (for `--out`
/// plain files / piped); on → the house TTY style.
pub fn findings_table(found: &Findings, use_color: bool) -> String {
    let rows: Vec<Vec<String>> = found
        .iter()
        .flat_map(|(rule, items)| {
            // "AGS Format Rule 8" → "8" keeps the column tight; the full label
            // is redundant once it's a column.
            let short = rule
                .strip_prefix("AGS Format Rule ")
                .unwrap_or(rule)
                .to_string();
            items.iter().map(move |f| {
                let line = f.line.map_or_else(|| "-".into(), |l| l.to_string());
                vec![short.clone(), line, f.group.clone(), f.desc.clone()]
            })
        })
        .collect();
    styled_table(&["Rule", "Line", "Group", "Description"], rows, use_color).to_string()
}

pub fn report_table(path: &Path, found: &Findings, n: usize, dict: &str, resolution: &str) {
    // `dict`/`resolution` ride the head line: which dictionary judged the file
    // is a launcher-contract FACT, not decoration (#542).
    println!(
        "{}: {n} finding(s) — dictionary {dict} ({resolution})",
        path.display()
    );
    println!("{}", findings_table(found, colour_enabled(false)));
}

/// The nested report value `{file, findings:{rule:[{line,group,desc}]}}`. Shared
/// by the stdout `--json` path and the `--out`/`--json-out` file writers so they
/// never disagree.
///
/// A `&Path`-taking adapter over the engine's renderer — the rendering itself
/// lives beside `Finding` in `laterite_ags4_validator::findings` so this
/// binary, laterite-py and laterite-node cannot spell `--json` differently
/// (laterite-dev#530).
pub fn json_value(path: &Path, found: &Findings) -> Value {
    findings::findings_json_value(&path.display().to_string(), found)
}

/// Plain pretty JSON (never coloured) — for files (`--out`/`--json-out`).
pub fn json_string(path: &Path, found: &Findings) -> String {
    findings::findings_json(&path.display().to_string(), found)
}

/// One flat JSON object per finding per line (NDJSON). Stream/grep friendly;
/// identical whether it goes to stdout or a file (no colour). Empty (no lines)
/// when there are zero findings.
pub fn ndjson_string(found: &Findings) -> String {
    findings::findings_ndjson(found)
}

/// The plain report rendered to a `String` (no colour) — for `--out` when
/// neither `--json` nor `--ndjson` is active.
pub fn plain_string(
    path: &Path,
    found: &Findings,
    n: usize,
    dict: &str,
    resolution: &str,
) -> String {
    if n == 0 {
        return format!(
            "{}: clean (0 findings) — dictionary {dict} ({resolution})\n",
            path.display()
        );
    }
    format!(
        "{}: {n} finding(s) — dictionary {dict} ({resolution})\n{}\n",
        path.display(),
        findings_table(found, false)
    )
}

/// `--json` to stdout: the report value rendered rich-style (coloured on a TTY)
/// or plain pretty otherwise, via the shared writer.
pub fn emit_json(path: &Path, found: &Findings) {
    let v = json_value(path, found);
    let mut out = io::stdout().lock();
    let _ = write_json_pretty(&mut out, &v, colour_enabled(false));
}

#[cfg(test)]
mod tests {
    use super::*;
    use laterite_ags4_validator::findings::add;

    fn sample() -> Findings {
        let mut f = Findings::new();
        add(
            &mut f,
            "AGS Format Rule 8",
            Some(5),
            "LOCA",
            "bad \"value\"",
        );
        add(&mut f, "AGS Format Rule 9", None, "SAMP", "x");
        f
    }

    #[test]
    fn ndjson_is_one_object_per_finding() {
        let s = ndjson_string(&sample());
        let lines: Vec<&str> = s.lines().collect();
        assert_eq!(lines.len(), 2);
        let a: Value = serde_json::from_str(lines[0]).unwrap();
        assert_eq!(a["rule"], "AGS Format Rule 8");
        assert_eq!(a["line"], 5);
        assert_eq!(a["group"], "LOCA");
        assert_eq!(a["desc"], "bad \"value\""); // quotes escaped correctly
        let b: Value = serde_json::from_str(lines[1]).unwrap();
        assert!(b["line"].is_null()); // None → null
        assert_eq!(ndjson_string(&Findings::new()), ""); // zero findings → empty
    }

    #[test]
    fn json_string_round_trips() {
        let v: Value = serde_json::from_str(&json_string(Path::new("x.ags"), &sample())).unwrap();
        assert_eq!(v["file"], "x.ags");
        assert!(v["findings"]["AGS Format Rule 8"].is_array());
    }

    #[test]
    fn plain_string_reports_clean_and_findings() {
        let clean = plain_string(Path::new("x.ags"), &Findings::new(), 0, "4.1.1", "exact");
        assert!(clean.contains("clean (0 findings)"));
        assert!(
            clean.contains("— dictionary 4.1.1 (exact)"),
            "the judging dictionary is a launcher-contract fact (#542)"
        );
        let p = plain_string(Path::new("x.ags"), &sample(), 2, "4.0.4", "fallback");
        assert!(p.contains("2 finding(s)") && p.contains("LOCA"));
        assert!(p.contains("— dictionary 4.0.4 (fallback)"));
    }
}
