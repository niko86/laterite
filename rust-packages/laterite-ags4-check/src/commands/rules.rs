//! `lat rules` — print the AGS4 rule catalogue and exit (was `--list-rules`).

use std::process::exit;

use laterite_cliutil::{colour_enabled, styled_table};
use serde_json::Value;

/// `json` emits the raw gated `rules_meta.json` (compile-time-embedded — no disk,
/// no validation run); otherwise a compact table. Input-independent (exits 0).
pub fn run(json: bool) -> ! {
    let raw = laterite_ags4_validator::rule_metadata_json();
    if json {
        println!("{raw}");
        exit(0);
    }
    let doc: Value = serde_json::from_str(raw).expect("rules_meta.json is gated to parse");
    let rows: Vec<Vec<String>> = doc["rules"]
        .as_array()
        .map(|rs| {
            rs.iter()
                .map(|r| {
                    let fixable = r["fixable"].as_bool().unwrap_or(false);
                    vec![
                        r["rule"].as_str().unwrap_or("").to_string(),
                        r["title"].as_str().unwrap_or("").to_string(),
                        r["severity"].as_str().unwrap_or("").to_string(),
                        if fixable { "yes" } else { "" }.to_string(),
                    ]
                })
                .collect()
        })
        .unwrap_or_default();
    println!(
        "{}",
        styled_table(
            &["Rule", "Title", "Severity", "Fix?"],
            rows,
            colour_enabled(false)
        )
    );
    exit(0);
}
