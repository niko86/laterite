//! `lat merge <files...> --out <merged.ags>` — reconcile N deliveries of one
//! project into a single file (KEY-aware, argument-order recency, widen-to-X).

use std::path::Path;
use std::process::exit;

use laterite_ags4_merge::{MergeError, MergeOpts, TranStamp, TypeMismatchMode, merge_parsed};
use laterite_ags4_parse::parse_bytes;
use laterite_ags4_validator::dict::FALLBACK;
use laterite_ags4_validator::{CheckOptions, ValidatorError, resolve_dict_version, tran_ags_of};
use laterite_cliutil::Spinner;

use crate::cli::MergeArgs;
use crate::commands::common::apply_dict_args;

/// Merge `args.files` in order (last wins a KEY conflict) → `args.out`. Edition is
/// picked from the newest file's TRAN_AGS (forced by `--dict-version`). `--json`
/// emits a `{warnings, revisions}` summary; otherwise a human summary. The merged
/// bytes always go to `--out`, so stdout stays clean.
pub fn run(args: &MergeArgs, json: bool, quiet: bool) -> ! {
    let opts = apply_dict_args(
        CheckOptions {
            include_warnings: true,
            ..CheckOptions::default()
        },
        &args.dict,
    );

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
    .map(|(dv, _)| dv)
    .unwrap_or(FALLBACK);

    // A merge-TRAN is synthesised only when both an issue and a date are given.
    let tran = match (&args.tran_issue, &args.tran_date) {
        (Some(isno), Some(date)) => Some(TranStamp {
            isno: isno.clone(),
            date: date.clone(),
            prod: args.tran_producer.clone().unwrap_or_default(),
            recv: args.tran_recipient.clone().unwrap_or_default(),
            stat: args.tran_status.clone().unwrap_or_default(),
            ags: dv.as_str().to_string(),
        }),
        _ => None,
    };

    let merge_opts = MergeOpts {
        type_mismatch: if args.lenient {
            TypeMismatchMode::Lenient
        } else {
            TypeMismatchMode::Strict
        },
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
                let summary = serde_json::json!({
                    "out": args.out.display().to_string(),
                    "bytes": res.bytes.len(),
                    "warnings": res.warnings.iter().map(|w| serde_json::json!({
                        "kind": w.kind,
                        "group": w.group,
                        "heading": w.heading,
                        "message": w.message,
                    })).collect::<Vec<_>>(),
                    "revisions": res.revisions.iter().map(|r| serde_json::json!({
                        "group": r.group,
                        "key": r.key,
                        "changed": r.changed,
                        "winner_file": r.winner_file,
                    })).collect::<Vec<_>>(),
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
            eprintln!("hint: pass --lenient to widen the column to X (text), keeping raw values");
            exit(6);
        }
        Err(MergeError::Emit(e)) => {
            eprintln!("error: emitting merged file: {e}");
            exit(6);
        }
    }
}
