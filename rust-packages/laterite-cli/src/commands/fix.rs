//! `lat fix <file>` — mechanically repair via the shared `fix_document`, ported
//! from the pre-rework `run_fix`. Destination: the source (`--in-place`), an
//! explicit `--fix-out`, or a sibling `<file>.fixed.ags`. The old imperative
//! `--in-place`↔`--fix-out` check is now the clap `fixdest` `ArgGroup`.

use std::process::exit;

use laterite_ags4_validator::{
    CheckOptions, CodeRewrite, FixError, SkippedRewrite, findings, fix_document_recoding,
};
use laterite_cliutil::{Spinner, write_atomic};
use serde_json::json;

use crate::cli::FixArgs;
use crate::commands::common::{apply_dict_args, read_recode, sibling_fixed_path};

/// One applied fix as the `--json` report carries it: the whole `Fix` serialised
/// (`{kind, label, rule, line, risk}`, serde `snake_case` — the shape the Node
/// `AppliedFix` and wasm fix payloads already use) minus the internal `edits` span
/// list, which a report consumer never needs. Built from `serde_json` so the CLI
/// crate needs no serde-derive dependency of its own.
fn applied_fix_json(f: &laterite_ags4_validator::fixes::Fix) -> serde_json::Value {
    let mut v = serde_json::to_value(f).unwrap_or(serde_json::Value::Null);
    if let Some(obj) = v.as_object_mut() {
        obj.remove("edits");
    }
    v
}

/// Safe fixes always; `--risky` also applies the intent-guessing ones. Exit 0 if
/// the repaired file is clean, 1 if findings remain that aren't mechanically
/// fixable (3/4/5 on read/parse/dict errors).
pub fn run(args: &FixArgs, json: bool, quiet: bool) -> ! {
    let opts = apply_dict_args(
        CheckOptions {
            include_warnings: true,
            ..CheckOptions::default()
        },
        &args.dict,
    );
    let rewrite = CodeRewrite {
        recode: read_recode(args.recode.as_deref()),
        on_code_case: args.on_code_case,
    };
    let path = args.file.as_path();
    let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("file");
    let spinner = Spinner::start(&format!("fixing {name}..."), quiet);

    let raw = match std::fs::read(path) {
        Ok(b) => b,
        Err(e) => {
            drop(spinner);
            eprintln!("error: {}: {e}", path.display());
            exit(3);
        }
    };
    let outcome = match fix_document_recoding(&raw, &opts, args.risky, None, &[], &rewrite) {
        Ok(o) => o,
        Err(FixError::Validator(e)) => {
            drop(spinner);
            eprintln!("error: {e}");
            exit(e.exit_code());
        }
        // A recode naming what the file does not have is the caller's
        // argument: the usage-class code `lat merge` gives it.
        Err(e @ FixError::Recode(_)) => {
            drop(spinner);
            eprintln!("error: {e}");
            exit(5);
        }
    };
    drop(spinner);

    let dest = if args.in_place {
        path.to_path_buf()
    } else if let Some(o) = args.fix_out.as_deref() {
        o.to_path_buf()
    } else {
        sibling_fixed_path(path)
    };
    if let Err(e) = write_atomic(&dest, &outcome.fixed) {
        eprintln!("error: writing {}: {e}", dest.display());
        exit(3);
    }

    let n_residual = findings::count(&outcome.residual);

    // --json: the machine-readable report replaces the human summary on stdout (the
    // file is written either way). Exit code is unchanged — 0 clean, 1 residual.
    if json {
        let applied: Vec<serde_json::Value> =
            outcome.applied.iter().map(applied_fix_json).collect();
        // `risky_available` is a human-only hint here: the Node `FixReport` has no
        // risky-count field to mirror, so keeping it out of the machine report is what
        // keeps `fix --json` byte-identical across the three launchers (laterite-dev#545).
        let mut report = json!({
            "file": path.display().to_string(),
            "dest": dest.display().to_string(),
            "applied": applied,
            "residual": n_residual,
        });
        // Only when a requested rewrite was left undone, so a run that asked
        // for none reports exactly what it always did.
        if !outcome.skipped.is_empty() {
            report["skipped"] = serde_json::to_value(&outcome.skipped).unwrap_or_default();
        }
        println!(
            "{}",
            serde_json::to_string_pretty(&report).unwrap_or_default()
        );
        exit(i32::from(n_residual != 0));
    }

    // Distinct fix kinds (serde snake_case names) for a one-line summary.
    let mut kinds: Vec<String> = outcome
        .applied
        .iter()
        .filter_map(|f| {
            serde_json::to_value(f.kind)
                .ok()
                .and_then(|v| v.as_str().map(String::from))
        })
        .collect();
    kinds.sort();
    kinds.dedup();
    let n_applied = outcome.applied.len();
    if n_applied == 0 {
        println!("no fixes applicable → {}", dest.display());
    } else {
        println!(
            "applied {n_applied} fix(es) [{}] → {}",
            kinds.join(", "),
            dest.display()
        );
    }
    for s in &outcome.skipped {
        println!("{}", skipped_line(s));
    }
    if !args.risky && outcome.risky_available > 0 {
        println!(
            "{} more fixable with --fix-risky (intent-guessing fixes withheld)",
            outcome.risky_available
        );
    }
    if n_residual == 0 {
        println!("{}: clean (0 findings)", dest.display());
        exit(0);
    }
    println!(
        "{}: {n_residual} finding(s) remain (not mechanically fixable)",
        dest.display()
    );
    exit(1);
}

/// One requested rewrite `fix` left undone, as the three launchers print it.
fn skipped_line(s: &SkippedRewrite) -> String {
    let codes: Vec<String> = s.codes.iter().map(|c| format!("{c:?}")).collect();
    format!(
        "left {} under {} as written: rewriting to {:?} would give two {} rows the KEY {:?}",
        codes.join(", "),
        s.heading,
        s.target,
        s.group,
        s.key.join("|")
    )
}
