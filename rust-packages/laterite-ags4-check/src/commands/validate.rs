//! `lat validate <file>` — run the numbered rules and report (the default verb).
//! Ported from the pre-rework validate path. `--index <cert>` consumes a
//! certificate to skip the engine (the cert logic lives in `cert`); minting
//! moved to `lat certify`.

use std::process::exit;

use laterite_ags4_validator::{CheckOptions, Findings, check_file, findings};
use laterite_cliutil::{Spinner, write_atomic};

use crate::cli::ValidateArgs;
use crate::commands::cert::{CertOutcome, report_certified_skip, try_certified_skip};
use crate::commands::common::apply_dict_args;
use crate::render;

/// Build `CheckOptions` from the verb's flags (WARNINGs on by default, like the
/// pre-rework binding; `--no-warnings` drops to errors-only, `--show-fyi` adds
/// the low-signal tier).
fn options(args: &ValidateArgs) -> CheckOptions {
    let mut opts = apply_dict_args(
        CheckOptions {
            include_warnings: true,
            ..CheckOptions::default()
        },
        &args.dict,
    );
    if args.no_warnings {
        opts.include_warnings = false;
    }
    if args.show_fyi {
        opts.include_fyi = true;
    }
    if args.check_files {
        opts.check_files = true;
    }
    opts
}

pub fn run(args: &ValidateArgs, json: bool, ndjson: bool, quiet: bool) -> ! {
    let opts = options(args);
    let path = args.file.as_path();
    let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("file");

    // `--index <cert>`: try the certificate short-circuit before touching the
    // rule engine. A skip yields an empty `Findings`; a non-skip falls through to
    // a normal engine run with a note saying why the cert wasn't trusted.
    let result = match args.index.as_deref() {
        Some(cert_path) => match try_certified_skip(path, &opts, cert_path) {
            CertOutcome::Skip(stamp) => {
                report_certified_skip(&stamp, opts.include_warnings, opts.include_fyi);
                Ok(Findings::new())
            }
            CertOutcome::Revalidate(reason) => {
                eprintln!("note: --index not used ({reason}); running the full check");
                let spinner = Spinner::start(&format!("validating {name}..."), quiet);
                let r = check_file(path, &opts);
                drop(spinner);
                r
            }
        },
        None => {
            let spinner = Spinner::start(&format!("validating {name}..."), quiet);
            let r = check_file(path, &opts);
            drop(spinner);
            r
        }
    };

    match result {
        Ok(found) => {
            let n = findings::count(&found);

            #[cfg(feature = "tui")]
            if args.tui {
                use std::io::IsTerminal;
                let interactive = std::io::stdin().is_terminal() && std::io::stdout().is_terminal();
                if interactive {
                    if json {
                        eprintln!(
                            "note: --tui is active; --json is ignored on an interactive terminal"
                        );
                    }
                    if let Err(e) = crate::tui::run(&found, name, opts.dict_version) {
                        eprintln!("error: tui: {e}");
                    }
                    exit(if n == 0 { 0 } else { 1 });
                }
                eprintln!(
                    "note: --tui requires an interactive terminal; using {} output",
                    if json { "JSON" } else { "plain" }
                );
            }

            let code = if n == 0 { 0 } else { 1 };

            // `--json-out`: always tee a JSON artifact, independent of stdout.
            if let Some(p) = args.json_out.as_deref() {
                if let Err(e) = write_atomic(p, render::json_string(path, &found).as_bytes()) {
                    eprintln!("error: --json-out {}: {e}", p.display());
                    exit(3);
                }
                eprintln!("note: JSON written to {}", p.display());
            }

            // `--out`: redirect the active format to a file; one confirmation line.
            if let Some(p) = args.out.as_deref() {
                let body = if json {
                    render::json_string(path, &found)
                } else if ndjson {
                    render::ndjson_string(&found)
                } else {
                    render::plain_string(path, &found, n)
                };
                if let Err(e) = write_atomic(p, body.as_bytes()) {
                    eprintln!("error: --out {}: {e}", p.display());
                    exit(3);
                }
                println!("wrote {n} finding(s) to {}", p.display());
                exit(code);
            }

            if json {
                render::emit_json(path, &found);
            } else if ndjson {
                print!("{}", render::ndjson_string(&found));
            } else if n == 0 {
                println!("{}: clean (0 findings)", path.display());
            } else {
                render::report_table(path, &found, n);
            }
            exit(code);
        }
        Err(e) => {
            eprintln!("error: {e}");
            exit(e.exit_code());
        }
    }
}
