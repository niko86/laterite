//! `ags5db db-to-ags4 <db> <ags4>` — export .ags5db back to AGS4 plaintext.
//!
//! Algorithm (mirrors Python `ags5_ags4.codec.db_to_ags4`):
//!
//! 1. For each group in registry-declared order:
//!    (a) `SELECT * FROM v_<code>`;
//!    (b) emit `GROUP`, `HEADING`, `UNIT`, `TYPE`, then DATA rows.
//! 2. Sections separated by a blank line.
//!
//! Bail conditions per the approved G4 plan:
//!
//! * **Hard bail** (exit 7) — any heading with `ags_type='RL'` (AGS4.1
//!   Rule 11 Record Link). Record-link handling is unscoped; we don't
//!   want to silently emit half-converted output that looks valid but
//!   drops the RL relationships.
//!
//!   * **Warn-but-continue** on stderr (exit 0) — missing TRAN (Rule 14),
//!     missing or duplicated PROJ (Rule 13), missing UNIT/ABBR when
//!     referenced (Rules 15/16). The output is still written; user runs
//!     `python_ags4`'s validator if they need spec-conformance assurance.
//!
//! The "is this AGS4 file valid?" question belongs to python-ags4 (see
//! the `ags4-validator` skill). This binary trusts its input and only
//! refuses on the one feature it can't faithfully round-trip.

use std::fs;
use std::path::PathBuf;
use std::time::Instant;

use clap::Args;
use serde_json::{Map, Value};

use crate::Ctx;
use crate::commands::pack::build_dry_run;
use crate::output::render_record;
use ags5db::db::display_native;
use ags5db::error::CliError;

#[derive(Args, Debug)]
pub struct Cmd {
    /// Source .ags5db path
    pub db: PathBuf,

    /// Output .ags file (AGS4 plaintext). Stored attachment blobs are
    /// also written beside it as the AGS4 Rule 20 sidecar tree
    /// `FILE/<FILE_FSET>/<FILE_NAME>` (spec-check with `ags4-check
    /// --check-files`).
    pub ags4: PathBuf,

    /// Report what would be written, write nothing
    #[arg(long)]
    pub dry_run: bool,

    /// After writing, run the bundled Rust AGS4 validator on the output
    /// and exit non-zero (code 10) if it produces any findings. Emit +
    /// verify in one pass, no Python.
    #[arg(long)]
    pub validate: bool,
}

pub fn run(args: Cmd, ctx: Ctx) -> anyhow::Result<()> {
    if !args.db.exists() {
        return Err(CliError::FileNotFound(args.db.display().to_string()).into());
    }
    if args.dry_run {
        let payload = build_dry_run("db-to-ags4", &args.db, &args.ags4, None);
        render_record(&payload, ctx.mode)?;
        return Ok(());
    }

    let spinner = crate::output::Spinner::start(
        &format!(
            "exporting {} -> {} (AGS4)...",
            args.db.file_name().and_then(|s| s.to_str()).unwrap_or(""),
            args.ags4.file_name().and_then(|s| s.to_str()).unwrap_or(""),
        ),
        ctx.quiet,
    );

    let t0 = Instant::now();
    let stats = ags5db::convert::db_to_ags4(&args.db, &args.ags4)?;

    // Unspool FILE_FSET attachments alongside the output `.ags`. Writing
    // them into the same directory matches the AGS4.1 convention that
    // referenced files live "next to" the transfer file. Users wanting
    // a separate folder can post-process.
    let out_dir = args
        .ags4
        .parent()
        .map(std::path::Path::to_path_buf)
        .unwrap_or_else(|| std::path::PathBuf::from("."));
    spinner.set_message("unspooling FILE_FSET attachments...");
    let attach_stats = ags5db::attachments::unspool_attachments(&args.db, &out_dir)?;

    let elapsed = t0.elapsed().as_secs_f64();
    drop(spinner);

    // Warn-but-continue conformance hints. These are advisory only — the
    // user's expected to run `ags4_cli check <out>` if they need a real
    // validity guarantee.
    for w in &stats.warnings {
        eprintln!("warning: {}", w);
    }
    for w in &attach_stats.warnings {
        eprintln!("warning: {}", w);
    }

    // `--validate`: run the bundled Rust validator on what we just
    // wrote. Emit + verify in one pass — no Python, no shell-out. The
    // file is already on disk + flushed (do_export) and attachments
    // unspooled, so we validate the final artefact.
    if args.validate {
        use ags4_validator::{CheckOptions, check_file, findings};
        // Default = auto-pick the dictionary edition from the emitted
        // file's own TRAN_AGS (errors only). Validating the output
        // against *its own* declared edition is exactly what we want.
        let opts = CheckOptions::default();
        match check_file(&args.ags4, &opts) {
            Ok(found) => {
                let n = findings::count(&found);
                if n > 0 {
                    return Err(CliError::Validation {
                        findings: n,
                        file: args.ags4.display().to_string(),
                    }
                    .into());
                }
            }
            Err(e) => {
                // The validator couldn't even parse what we wrote — that
                // is a hard problem with the emitted file, surface it.
                return Err(CliError::Validation {
                    findings: 0,
                    file: format!("{} ({e})", args.ags4.display()),
                }
                .into());
            }
        }
    }

    let bytes = fs::metadata(&args.ags4)
        .map_err(|e| CliError::Schema(format!("stat dst: {}", e)))?
        .len();

    let mut payload = Map::new();
    payload.insert("command".into(), Value::from("db-to-ags4"));
    payload.insert("src".into(), Value::from(display_native(&args.db)));
    payload.insert("out".into(), Value::from(display_native(&args.ags4)));
    payload.insert("bytes".into(), Value::from(bytes));
    payload.insert("groups".into(), Value::from(stats.groups_emitted));
    payload.insert("rows".into(), Value::from(stats.rows_emitted));
    payload.insert(
        "attachments".into(),
        Value::from(attach_stats.files_processed as u64),
    );
    payload.insert(
        "attachment_bytes".into(),
        Value::from(attach_stats.bytes_total),
    );
    payload.insert(
        "warnings".into(),
        Value::Array(stats.warnings.into_iter().map(Value::from).collect()),
    );
    payload.insert(
        "elapsed_s".into(),
        Value::from((elapsed * 100.0).round() / 100.0),
    );
    render_record(&payload, ctx.mode)?;
    Ok(())
}
