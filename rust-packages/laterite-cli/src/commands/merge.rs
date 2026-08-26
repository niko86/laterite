//! `lat merge <files...> --out <merged.ags>` — reconcile N deliveries of one
//! project into a single file (KEY-aware, argument-order recency, type-clash lattice).

use std::path::Path;
use std::process::exit;

use laterite_ags4_merge::{MergeError, MergeOpts, TranStamp, merge_parsed};
use laterite_ags4_parse::parse_bytes;
use laterite_ags4_validator::dict::FALLBACK;
use laterite_ags4_validator::{CheckOptions, ValidatorError, resolve_dict_version, tran_ags_of};
use laterite_cliutil::Spinner;

use crate::cli::MergeArgs;
use crate::commands::common::apply_dict_args;

/// Merge `args.files` in order (last wins a KEY conflict) → `args.out`. Edition is
/// picked from the newest file's `TRAN_AGS` (forced by `--dict-version`). `--json`
/// emits a `{warnings, revisions}` summary; otherwise a human summary. The merged
/// bytes always go to `--out`, so stdout stays clean.
pub fn run(args: &MergeArgs, json: bool, quiet: bool) -> ! {
    let opts = apply_dict_args(CheckOptions::default(), &args.dict);

    let spinner = Spinner::start("merging...", quiet);
    let read = |p: &Path| match std::fs::read(p) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("error: {}: {e}", p.display());
            exit(3);
        }
    };
    let parsed: Vec<_> = args
        .files
        .iter()
        .map(|p| match parse_bytes(&read(p), opts.encoding) {
            Ok(pf) => pf,
            Err(e) => {
                eprintln!("error: {}: {}", p.display(), ValidatorError::from(e));
                exit(4);
            }
        })
        .collect();
    drop(spinner);

    // Edition from the LAST (newest) file's TRAN_AGS, forced by --dict-version.
    let dv = resolve_dict_version(
        opts.dict_version,
        parsed.last().and_then(tran_ags_of).as_deref(),
    )
    .map_or(FALLBACK, |(dv, _)| dv);

    // All five `--tran-*` flags or none: the shared rule, in the shared place.
    // This was a hand-rolled issue+date match that let producer/recipient/status
    // default to empty — three REQUIRED headings, silently blank. Four-of-five is
    // now a usage error naming the missing flags, not a quietly incomplete file.
    // `ags` is left to merge, which knows the edition it resolved.
    let tran = match TranStamp::from_parts(
        args.tran_issue.clone(),
        args.tran_date.clone(),
        args.tran_producer.clone(),
        args.tran_recipient.clone(),
        args.tran_status.clone(),
        args.tran_description.clone(),
        args.tran_remarks.clone(),
    ) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("error: {e}");
            eprintln!(
                "       flags: --tran-issue --tran-date --tran-producer --tran-recipient --tran-status"
            );
            exit(2); // usage, not data — the files were never the problem
        }
    };

    let merge_opts = MergeOpts {
        on_type_clash: args.on_type_clash,
        edition: dv,
        tran,
        ..Default::default()
    };

    match merge_parsed(&parsed, &merge_opts) {
        Ok(res) => {
            if let Err(e) = std::fs::write(&args.out, &res.bytes) {
                eprintln!("error: writing {}: {e}", args.out.display());
                exit(3);
            }
            if json {
                // The two-key `{out, bytes}` wrapper is CLI-local (the engine is
                // never told the output path); the arrays serialise straight off
                // the engine structs, whose derive owns the wire shape (#542).
                let summary = serde_json::json!({
                    "out": args.out.display().to_string(),
                    "bytes": res.bytes.len(),
                    "warnings": res.warnings,
                    "revisions": res.revisions,
                });
                println!(
                    "{}",
                    serde_json::to_string_pretty(&summary).unwrap_or_default()
                );
            } else {
                println!(
                    "merged {} files → {} ({} bytes)",
                    parsed.len(),
                    args.out.display(),
                    res.bytes.len()
                );
                for w in &res.warnings {
                    println!("  warning [{}]: {}", w.kind, w.message);
                }
                if !res.revisions.is_empty() {
                    println!("  {} row revision(s):", res.revisions.len());
                    for r in &res.revisions {
                        println!(
                            "    {} {:?}: changed {:?} (from file[{}])",
                            r.group, r.key, r.changed, r.winner_file
                        );
                    }
                }
            }
            exit(0);
        }
        Err(MergeError::TypeConflict {
            group,
            heading,
            types,
        }) => {
            eprintln!("error: TYPE conflict in {group}.{heading}: files declared {types:?}");
            // Both escape hatches, in lattice order — promote first, because it is
            // the one that KEEPS the type. Offering only the lossy one would push
            // every clash toward X, which is exactly what laterite-dev#500 set out to stop.
            eprintln!(
                "hint: --on-type-clash promote  keeps the greatest precision when every code is \
                 nDP (e.g. 2DP + 5DP -> 5DP, coarser values zero-padded; no digit is changed)"
            );
            eprintln!(
                "hint: --on-type-clash widen    falls back to X (free text) — raw values kept, \
                 but the column's TYPE is thrown away"
            );
            exit(6);
        }
        Err(MergeError::UnitConflict {
            group,
            heading,
            units,
        }) => {
            eprintln!("error: UNIT conflict in {group}.{heading}: files declared {units:?}");
            // Deliberately NO mode hint here: no mode absorbs a unit clash.
            // Offering one here would send the user in a circle (see laterite-dev#501).
            eprintln!(
                "hint: merge will not convert units, and no mode can absorb this — picking one \
                 would silently mislabel the other file's values. Reconcile the UNIT row in the \
                 source files, then merge."
            );
            exit(6);
        }
        Err(MergeError::Emit(e)) => {
            eprintln!("error: emitting merged file: {e}");
            exit(6);
        }
    }
}
