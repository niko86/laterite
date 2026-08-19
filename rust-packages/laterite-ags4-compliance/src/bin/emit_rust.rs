//! `emit-rust` (#169 Workstream C 5a) — the RUST leg of the cross-surface
//! findings harness.
//!
//! Runs the in-workspace validator engine
//! ([`laterite_ags4_validator::check_file`]) over a fixtures dir and writes
//! `rust.json` in the exact shape the `laterite-ags4-compliance` comparator reads — the
//! same schema the dev satellite's `tools/compliance/_worker.py` emits for the
//! Python surfaces.
//! This is the *reference* engine every laterite surface (python-laterite /
//! node / wasm / duckdb) wraps, so its rule FLOOR is the identity the
//! 4-laterite check holds the bindings to (duckdb left findings in laterite-dev#458).
//!
//! Knob choices are the harness invariants (plan §3.3/§3.4): FYI + warnings
//! **ON** (so the FYI-capable surfaces are comparable), `check_files` **OFF**
//! (the fixtures don't ship their referenced binaries, and duckdb never runs
//! Rule 20's on-disk half). The floor is `"AGS Format Rule N"` only; FYI labels
//! (`"FYI …"`, stored under their own keys by the engine) ride the `fyi` field,
//! split by prefix exactly like the Python worker.

use std::path::PathBuf;

use laterite_ags4_parity::RustResult;
use laterite_ags4_validator::findings::count_by_rule;
use laterite_ags4_validator::{CheckOptions, check_file};
use serde::Serialize;

/// One floor finding as the comparator's 4-laterite check compares it (#555
/// part 1) — rule label + the WHERE (`line`/`group`/`field_index`) and WHAT
/// (`desc`). Field names + order match `laterite-ags4-compliance`'s `FindingTuple`.
#[derive(Serialize)]
struct FindingTuple {
    rule: String,
    line: Option<u32>,
    group: String,
    desc: String,
    field_index: Option<u32>,
}

/// One fixture's finding-set — the per-row shape the comparator deserialises.
#[derive(Serialize)]
struct Fixture {
    #[serde(rename = "fixture")]
    name: String,
    /// The `"AGS Format Rule N"` floor as bare LABELS (the python-ags4 leg's
    /// input; kept for that comparison and back-compat).
    rules: Vec<String>,
    /// The `"AGS Format Rule N"` floor as full TUPLES — what the 4-laterite
    /// identity check actually compares (#555 part 1).
    findings: Vec<FindingTuple>,
    /// The `"FYI …"` labels (compared only among the FYI-capable surfaces).
    fyi: Vec<String>,
    /// Canonical hard-error sentinel (`NotAgs4` / `UnsupportedEdition` / …) for
    /// an un-validatable file; `None` when the engine produced findings.
    hard_error: Option<String>,
    /// The reference (python-ags4) error channel — never set for a laterite
    /// surface; present so the schema is one shape across all six.
    error: Option<String>,
}

#[derive(Serialize)]
struct Surface {
    schema: u32,
    #[serde(rename = "surface")]
    kind: &'static str,
    version: &'static str,
    results: Vec<Fixture>,
}

/// Partition finding-keys into the rule FLOOR and the FYI labels — the same
/// prefix split `_worker.py` applies to `Report.by_rule().keys()`.
fn split(labels: impl IntoIterator<Item = String>) -> (Vec<String>, Vec<String>) {
    let (mut floor, mut fyi) = (Vec::new(), Vec::new());
    for l in labels {
        if l.starts_with("AGS Format Rule ") {
            floor.push(l);
        } else if l.starts_with("FYI") {
            fyi.push(l);
        }
    }
    floor.sort();
    fyi.sort();
    (floor, fyi)
}

fn main() {
    let mut args = std::env::args().skip(1);
    let Some(fixtures_dir) = args.next().map(PathBuf::from) else {
        eprintln!("usage: emit-rust <fixtures-dir> [out-dir]");
        std::process::exit(2);
    };
    let out_dir = args
        .next()
        .map_or_else(|| PathBuf::from("output/compliance-results"), PathBuf::from);

    // The harness invariants — FYI/warnings ON, on-disk Rule 20 OFF.
    let opts = CheckOptions {
        include_warnings: true,
        include_fyi: true,
        check_files: false,
        ..Default::default()
    };

    let mut paths: Vec<PathBuf> = std::fs::read_dir(&fixtures_dir)
        .unwrap_or_else(|e| {
            eprintln!("read {}: {e}", fixtures_dir.display());
            std::process::exit(2);
        })
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("ags"))
        .collect();
    paths.sort();

    let results: Vec<Fixture> = paths
        .iter()
        .map(|p| {
            let name = p
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned();
            match check_file(p, &opts) {
                Ok(found) => {
                    let (rules, fyi) = split(
                        count_by_rule(&found)
                            .into_iter()
                            .map(|(r, _)| r.to_string()),
                    );
                    // Full floor tuples (the 4-laterite check's input). Floor
                    // rules only — FYI excluded, exactly like `rules`. The
                    // comparator sorts before comparing, so emit in engine order.
                    let findings = found
                        .iter()
                        .filter(|(rule, _)| rule.starts_with("AGS Format Rule "))
                        .flat_map(|(rule, items)| {
                            items.iter().map(move |f| FindingTuple {
                                rule: rule.clone(),
                                line: f.line,
                                group: f.group.clone(),
                                desc: f.desc.clone(),
                                field_index: f.location.field_index,
                            })
                        })
                        .collect();
                    Fixture {
                        name,
                        rules,
                        findings,
                        fyi,
                        hard_error: None,
                        error: None,
                    }
                }
                Err(e) => {
                    // Reuse laterite-ags4-parity's mapping so the sentinel vocabulary is
                    // single-sourced with the pairwise oracle.
                    let hard_error = match RustResult::from_validator_error(&e) {
                        RustResult::HardError(s) => Some(s),
                        _ => None,
                    };
                    Fixture {
                        name,
                        rules: vec![],
                        findings: vec![],
                        fyi: vec![],
                        hard_error,
                        error: None,
                    }
                }
            }
        })
        .collect();

    std::fs::create_dir_all(&out_dir).expect("create out dir");
    let surface = Surface {
        schema: 1,
        kind: "rust",
        version: env!("CARGO_PKG_VERSION"),
        results,
    };
    let path = out_dir.join("rust.json");
    std::fs::write(&path, serde_json::to_string(&surface).expect("serialize"))
        .expect("write rust.json");
    eprintln!(
        "rust v{}: {} fixtures -> {}",
        surface.version,
        surface.results.len(),
        path.display()
    );
}

#[cfg(test)]
mod tests {
    use super::split;

    #[test]
    fn split_partitions_floor_fyi_and_drops_neither() {
        let (floor, fyi) = split([
            "AGS Format Rule 8".to_string(),
            "FYI (Related to Rule 1)".to_string(),
            "AGS Format Rule 1".to_string(),
            "FYI".to_string(),
            "something else".to_string(), // neither prefix → dropped, like _worker.py
        ]);
        assert_eq!(floor, ["AGS Format Rule 1", "AGS Format Rule 8"]); // sorted
        assert_eq!(fyi, ["FYI", "FYI (Related to Rule 1)"]);
    }
}
