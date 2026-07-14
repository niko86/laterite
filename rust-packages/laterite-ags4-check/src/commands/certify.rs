//! `lat certify <file>` — mint the `.ags.idx` validity certificate for an
//! error-clean file.
//!
//! The command used to validate here, count the severities here, decide here whether
//! the file was certifiable, and then hand the counts to a mint that wrote down
//! whatever it was told. Now it reads the file and calls `trust::mint`, which validates
//! and counts and decides for itself — because the caller getting those wrong is the
//! whole history of this feature. (`laterite-py`'s mint took `warnings=0, fyi=0` as
//! DEFAULT ARGUMENTS, and nothing ever passed them.)

use std::process::exit;

use laterite_ags4_trust::mint;
use laterite_ags4_validator::CheckOptions;
use laterite_cliutil::Spinner;

use crate::cli::CertifyArgs;
use crate::commands::cert;
use crate::commands::common::apply_dict_args;

pub fn run(args: &CertifyArgs, quiet: bool) -> ! {
    // Only the CONTENT knobs. `mint` forces both tiers on regardless — a certificate
    // that measured less than it could is a certificate that answers fewer questions —
    // and it will not take a `check_files` at all: Rule 20's on-disk half reads a
    // directory, and no statement about the certified bytes can speak for a directory.
    let opts = apply_dict_args(CheckOptions::default(), &args.dict);

    let path = args.file.as_path();
    let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("file");

    let bytes = match std::fs::read(path) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("error: read {}: {e}", path.display());
            exit(3);
        }
    };

    let spinner = Spinner::start(&format!("validating {name}..."), quiet);
    let minted = mint(
        &bytes,
        &opts,
        chrono::Utc::now().to_rfc3339(),
        None, // the native engine, not the compat shim
    );
    drop(spinner);

    let sidecar = match minted {
        Ok(s) => s,
        Err(e) => {
            // A file with errors is not certifiable, and `mint` is the one that knows —
            // it ran the rules. It says so in its own words, and carries its own exit
            // code (1 for findings, like every other verb that reports them).
            match e {
                laterite_ags4_trust::MintError::NotCertifiable { .. } => {
                    eprintln!("error: {e} (run `lat validate {name}` to see them)");
                }
                _ => eprintln!("error: {e}"),
            }
            exit(e.exit_code());
        }
    };

    match cert::write(&sidecar, path, args.out.as_deref()) {
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
