//! `lat fix <file>` — mechanically repair via the shared `fix_document`, ported
//! from the pre-rework `run_fix`. Destination: the source (`--in-place`), an
//! explicit `--fix-out`, or a sibling `<file>.fixed.ags`. The old imperative
//! `--in-place`↔`--fix-out` check is now the clap `fixdest` ArgGroup.

use std::process::exit;

use laterite_ags4_validator::{CheckOptions, findings, fix_document};
use laterite_cliutil::{Spinner, write_atomic};

use crate::cli::FixArgs;
use crate::commands::common::{apply_dict_args, sibling_fixed_path};

/// Safe fixes always; `--risky` also applies the intent-guessing ones. Exit 0 if
/// the repaired file is clean, 1 if findings remain that aren't mechanically
/// fixable (3/4/5 on read/parse/dict errors).
pub fn run(args: &FixArgs, quiet: bool) -> ! {
    let opts = apply_dict_args(
        CheckOptions {
            include_warnings: true,
            ..CheckOptions::default()
        },
        &args.dict,
    );
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
    let outcome = match fix_document(&raw, &opts, args.risky) {
        Ok(o) => o,
        Err(e) => {
            drop(spinner);
            eprintln!("error: {e}");
            exit(e.exit_code());
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
    if !args.risky && outcome.risky_available > 0 {
        println!(
            "{} more fixable with --fix-risky (intent-guessing fixes withheld)",
            outcome.risky_available
        );
    }
    let n_residual = findings::count(&outcome.residual);
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
