//! `lat diff <a> <b>` — the KEY-aware / type-aware revision delta (was `--diff`),
//! ported verbatim from the pre-rework `run_diff`.

use std::path::Path;
use std::process::exit;

use laterite_ags4_parse::parse_bytes;
use laterite_ags4_validator::dict::{Dictionary, FALLBACK};
use laterite_ags4_validator::{CheckOptions, ValidatorError, resolve_dict_version, tran_ags_of};
use laterite_cliutil::Spinner;

use crate::cli::DiffArgs;
use crate::commands::common::apply_dict_args;

/// `a` (baseline) vs `b` (revision). `json` emits the full `RevisionDelta`;
/// otherwise a per-group summary. KEY headings come from the dictionary, edition
/// picked from the revision's `TRAN_AGS` (forced by `--dict-version`).
pub fn run(args: &DiffArgs, json: bool, quiet: bool) -> ! {
    let opts = apply_dict_args(
        CheckOptions {
            include_warnings: true,
            ..CheckOptions::default()
        },
        &args.dict,
    );
    let (a, b) = (args.file.as_path(), args.other.as_path());
    let spinner = Spinner::start("diffing...", quiet);
    let read = |p: &Path| match std::fs::read(p) {
        Ok(bytes) => bytes,
        Err(e) => {
            eprintln!("error: {}: {e}", p.display());
            exit(3);
        }
    };
    let raw_a = read(a);
    let raw_b = read(b);
    let parse = |raw: &[u8], p: &Path| match parse_bytes(raw, opts.encoding) {
        Ok(pf) => pf,
        Err(e) => {
            eprintln!("error: {}: {}", p.display(), ValidatorError::from(e));
            exit(4);
        }
    };
    let pa = parse(&raw_a, a);
    let pb = parse(&raw_b, b);
    drop(spinner);

    let dv = resolve_dict_version(opts.dict_version, tran_ags_of(&pb).as_deref())
        .map_or(FALLBACK, |(dv, _)| dv);
    let dict = Dictionary::bundled(dv);
    let delta = laterite_ags4_diff::diff_parsed(&pa, &pb, &dict, None);

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&delta).unwrap_or_default()
        );
    } else {
        println!("{} → {}", a.display(), b.display());
        for g in &delta.groups {
            println!("  {:<6} +{} -{} ~{}", g.code, g.added, g.removed, g.changed);
        }
        if !delta.groups_added.is_empty() {
            println!("  groups added:   {}", delta.groups_added.join(", "));
        }
        if !delta.groups_removed.is_empty() {
            println!("  groups removed: {}", delta.groups_removed.join(", "));
        }
        println!(
            "  total: +{} added · −{} removed · ~{} changed",
            delta.total_added, delta.total_removed, delta.total_changed
        );
    }
    exit(0);
}
