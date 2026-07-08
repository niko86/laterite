//! `lat certify <file>` — mint the `.ags.idx` validity certificate for an
//! error-clean file (was `lat-check --emit-index`). Validates first; a file with
//! errors can't be certified. The mint logic lives in `cert` (shared with the
//! `validate --index` consume path).

use std::process::exit;

use laterite_ags4_validator::{CheckOptions, check_file};
use laterite_cliutil::Spinner;

use crate::cli::CertifyArgs;
use crate::commands::cert::mint_index;
use crate::commands::common::apply_dict_args;
use crate::render::count_severities;

pub fn run(args: &CertifyArgs, quiet: bool) -> ! {
    // Record accurate advisory counts (warnings + fyi) in the cert, so check with
    // both tiers on; `check_files` rides into the cert PROFILE.
    let mut opts = apply_dict_args(
        CheckOptions {
            include_warnings: true,
            include_fyi: true,
            ..CheckOptions::default()
        },
        &args.dict,
    );
    if args.check_files {
        opts.check_files = true;
    }

    let path = args.file.as_path();
    let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("file");
    let spinner = Spinner::start(&format!("validating {name}..."), quiet);
    let result = check_file(path, &opts);
    drop(spinner);

    let found = match result {
        Ok(f) => f,
        Err(e) => {
            eprintln!("error: {e}");
            exit(e.exit_code());
        }
    };

    // A certificate attests an ERROR-clean file; warnings/fyi ride on the stamp
    // as counts but don't block it.
    let (errors, warnings, fyi) = count_severities(&found);
    if errors > 0 {
        eprintln!(
            "cannot certify: {errors} error(s) — a certificate attests a clean \
             validation (run `lat validate {name}` to see them)"
        );
        exit(1);
    }

    match mint_index(path, &opts, warnings, fyi, args.out.as_deref()) {
        Ok(dest) => {
            println!("certificate written to {}", dest.display());
            exit(0);
        }
        Err(e) => {
            eprintln!("error: {e}");
            exit(3);
        }
    }
}
