//! `lat validate <file>` — run the numbered rules and report (the default verb).
//! Ported from the pre-rework validate path. `--index <cert>` consumes a
//! certificate to skip the engine (the cert logic lives in `cert`); minting
//! moved to `lat certify`.

use std::process::exit;

use laterite_ags4_trust::{Request, check};
use laterite_ags4_validator::{CheckOptions, WorldScope, findings, verdict::Verdict};
use laterite_cliutil::{Spinner, write_atomic};

use crate::cli::ValidateArgs;
use crate::commands::cert;
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

    let bytes = match std::fs::read(path) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("error: read {}: {e}", path.display());
            exit(3);
        }
    };

    // `--index <cert>` is OPT-IN and never auto-discovered: an `.ags.idx` lying beside a
    // file is not consent to trust it. A cert that cannot be read is not fatal — it just
    // doesn't help, and you pay for the validation you would have paid for anyway.
    let sidecar = match args.index.as_deref() {
        Some(cert_path) => match cert::load(cert_path) {
            Ok(s) => Some(s),
            Err(e) => {
                eprintln!("note: --index not used ({e}); running the full check");
                None
            }
        },
        None => None,
    };

    let spinner = Spinner::start(&format!("validating {name}..."), quiet);
    // The world is OnDisk because we have a real path — but `check` only looks at it if
    // `--check-files` actually asked. Handing over a path is not the same as asking.
    let outcome = check(Request {
        bytes: &bytes,
        opts: &opts,
        cert: sidecar.as_ref(),
        world: WorldScope::OnDisk(path.to_path_buf()),
        compat: None,
    });
    drop(spinner);

    let result = outcome.map(|o| {
        if o.certified {
            // The rule ENGINE was skipped. If --check-files was on, its on-disk half
            // still ran — say so, so nobody reads "certified" as "unexamined".
            cert::report_certified_skip(
                sidecar.as_ref().expect("certified implies a cert"),
                opts.check_files,
            );
        } else if let Some(reason) = o.revalidate_reason {
            eprintln!(
                "note: --index not used ({}); running the full check",
                cert::why(reason)
            );
        }
        (o.findings, o.dict_version, o.resolution)
    });

    match result {
        Ok((found, dv, resolution)) => {
            // `n` is what the report SHOWS (every tier the caller asked for);
            // `verdict` is what it CONCLUDES. They stopped being the same
            // question in #321 — a warning prints and does not fail.
            let n = findings::count(&found);
            let verdict = Verdict::of(&found, args.warnings_as_errors);

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
                    exit(verdict.exit_code());
                }
                eprintln!(
                    "note: --tui requires an interactive terminal; using {} output",
                    if json { "JSON" } else { "plain" }
                );
            }

            let code = verdict.exit_code();

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
                    render::plain_string(path, &found, n, dv.as_str(), resolution.as_str())
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
                // The dictionary that judged the file, and how it was chosen,
                // is a FACT of the verdict — the launcher contract binds facts
                // across launchers (npx already stated it; #542). One clean-line
                // shape, owned by render::plain_string, not a second copy here.
                print!(
                    "{}",
                    render::plain_string(path, &found, 0, dv.as_str(), resolution.as_str())
                );
            } else {
                render::report_table(path, &found, n, dv.as_str(), resolution.as_str());
            }
            exit(code);
        }
        Err(e) => {
            eprintln!("error: {e}");
            exit(e.exit_code());
        }
    }
}
