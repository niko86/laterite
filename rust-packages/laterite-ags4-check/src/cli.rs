//! Command-line surface (clap derive) for `lat` — the reworked AGS4 tool.
//!
//! Follows the workspace CLI lineage (`ags4-forge` / `ags4-corpus-qa`): a few
//! **global** flags valid before/after the subcommand, results to **stdout**,
//! progress to **stderr**, typed exit codes in `after_help`. Each verb owns its
//! flags, so the six imperative mutual-exclusion checks the old flat parser
//! carried mostly vanish structurally (a flag can't reach a verb it doesn't
//! belong to); the two that remain are declared here (`--json`↔`--ndjson`,
//! `fix --in-place`↔`--fix-out`).
//!
//! `--json`/`--ndjson` are deliberately kept as **global bools** (not the
//! `OutputMode` enum the siblings use) so the validate report is byte-identical
//! to the pre-rework CLI — the byte-parity gate (`test_cli_*`) depends on it.

use std::path::PathBuf;

use clap::{ArgGroup, Args, Parser, Subcommand};

/// The known subcommand names — the `main` default-subcommand pre-scan uses this
/// to decide whether a bare `lat <file>` should have `validate` spliced in.
pub const SUBCOMMANDS: &[&str] = &[
    "validate", "read", "fix", "diff", "certify", "rules", "pack", "unpack", "lock", "unlock",
    "excel",
];

#[derive(Parser)]
#[command(name = "lat", about = "AGS4 validate / read / fix / diff / certify", version, after_help = HELP_EPILOG)]
pub struct Cli {
    /// Machine-readable findings (pretty JSON).
    #[arg(long, global = true)]
    pub json: bool,
    /// One flat JSON object per finding per line.
    #[arg(long, global = true, conflicts_with = "json")]
    pub ndjson: bool,
    /// Suppress the progress spinner.
    #[arg(long, global = true)]
    pub quiet: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Validate an AGS4 file against the numbered rules (the default verb: a
    /// bare `lat <file>` runs this).
    Validate(ValidateArgs),
    /// Read a group's rows as a table / CSV / JSON — or list the file's group
    /// codes when no group is named.
    Read(ReadArgs),
    /// Mechanically repair an AGS4 file — safe fixes by default, `--risky` adds
    /// the intent-guessing ones. Non-destructive (writes a sibling).
    Fix(FixArgs),
    /// Compare two revisions — the KEY-aware / type-aware delta.
    Diff(DiffArgs),
    /// Mint the `.ags.idx` validity certificate for an error-clean file.
    Certify(CertifyArgs),
    /// Print the AGS4 rule catalogue (no input file needed).
    Rules,
    /// Package a file for transport — zstd-compress (any file type).
    #[cfg(feature = "transport")]
    Pack(PackArgs),
    /// Restore a `pack`ed file — zstd-decompress.
    #[cfg(feature = "transport")]
    Unpack(UnpackArgs),
    /// Encrypt + compress a file with an age passphrase (zstd + age).
    #[cfg(feature = "transport")]
    Lock(LockArgs),
    /// Decrypt + decompress a `lock`ed file.
    #[cfg(feature = "transport")]
    Unlock(UnlockArgs),
    /// Convert AGS4 ↔ Excel — direction inferred from the output extension
    /// (`.xlsx` ⇒ export, `.ags` ⇒ import; override with `--export` / `--import`).
    #[cfg(feature = "excel")]
    Excel(ExcelArgs),
}

/// The passphrase source shared by `lock` / `unlock`. We never take a `--password`
/// flag — argv leaks into `ps` and shell history; precedence is `--password-file`
/// → `$LAT_TRANSPORT_PASSWORD` → an interactive TTY prompt.
#[cfg(feature = "transport")]
#[derive(Args)]
pub struct PasswordArgs {
    /// Read the passphrase from <path> (a trailing newline is stripped).
    #[arg(long, value_name = "PATH")]
    pub password_file: Option<PathBuf>,
}

#[cfg(feature = "transport")]
#[derive(Args)]
pub struct PackArgs {
    /// The file to package (any type — `.ags`, `.ags5db`, anything).
    pub input: PathBuf,
    /// The `.zst` output path.
    pub output: PathBuf,
    /// zstd level: 1 (fastest) – 22 (highest ratio). 9 is the AGS sweet spot.
    #[arg(long, default_value_t = 9)]
    pub level: i32,
}

#[cfg(feature = "transport")]
#[derive(Args)]
pub struct UnpackArgs {
    /// The `pack`ed (`.zst`) file.
    pub input: PathBuf,
    /// The restored output path.
    pub output: PathBuf,
}

#[cfg(feature = "transport")]
#[derive(Args)]
pub struct LockArgs {
    /// The file to encrypt (any type).
    pub input: PathBuf,
    /// The `.age` output path.
    pub output: PathBuf,
    /// zstd level applied before encryption: 1 – 22 (default 9).
    #[arg(long, default_value_t = 9)]
    pub level: i32,
    /// scrypt work factor `log2(N)` for the passphrase KDF (default 18 — the
    /// interop-pinned value the browser + library use).
    #[arg(long, value_name = "N")]
    pub log_n: Option<u8>,
    #[command(flatten)]
    pub password: PasswordArgs,
}

#[cfg(feature = "transport")]
#[derive(Args)]
pub struct UnlockArgs {
    /// The `lock`ed (`.age`) file.
    pub input: PathBuf,
    /// The decrypted output path.
    pub output: PathBuf,
    #[command(flatten)]
    pub password: PasswordArgs,
}

#[cfg(feature = "excel")]
#[derive(Args)]
#[command(group(ArgGroup::new("exceldir").args(["export", "import"])))]
pub struct ExcelArgs {
    /// The input file (`.ags` to export, `.xlsx` to import).
    pub input: PathBuf,
    /// The output file: `.xlsx` ⇒ export (AGS4 → Excel), `.ags` ⇒ import.
    pub output: PathBuf,
    /// Force AGS4 → Excel (else the direction is inferred from `output`).
    #[arg(long)]
    pub export: bool,
    /// Force Excel → AGS4 (else the direction is inferred from `output`).
    #[arg(long)]
    pub import: bool,
    /// (import only) leave numeric-looking columns as text, don't reformat them.
    #[arg(long)]
    pub no_format_numeric: bool,
}

/// Dictionary + encoding flags shared by every file-consuming verb.
#[derive(Args)]
pub struct DictArgs {
    /// Force a bundled edition: `auto` (default — from the file's TRAN_AGS) |
    /// 4.0.3 | 4.0.4 | 4.1 | 4.1.1 | 4.2.
    #[arg(long, value_name = "V")]
    pub dict_version: Option<String>,
    /// External dictionary override (not supported).
    #[arg(long, value_name = "PATH")]
    pub dict: Option<PathBuf>,
    /// Source file encoding (default utf-8): utf-8 | cp1252 | latin1 |
    /// iso-8859-1 | iso-8859-15.
    #[arg(long, value_name = "NAME")]
    pub encoding: Option<String>,
}

#[derive(Args)]
pub struct ValidateArgs {
    /// The .ags file to validate.
    pub file: PathBuf,
    #[command(flatten)]
    pub dict: DictArgs,
    /// Errors only — suppress the WARNING tier (shown by default).
    #[arg(long)]
    pub no_warnings: bool,
    /// Include FYI-severity findings (e.g. Rule 1).
    #[arg(long)]
    pub show_fyi: bool,
    /// Also run Rule 20's on-disk check (the sibling FILE/ tree must exist).
    #[arg(long)]
    pub check_files: bool,
    /// Write the active format to <path> instead of stdout.
    #[arg(long, value_name = "PATH")]
    pub out: Option<PathBuf>,
    /// Also tee the JSON report to <path> while the normal report still prints.
    #[arg(long, value_name = "PATH")]
    pub json_out: Option<PathBuf>,
    /// Consume an `.ags.idx` certificate: if fresh + same-engine + profile-
    /// covering, skip the rule engine and report the certified verdict.
    #[arg(long, value_name = "PATH")]
    pub index: Option<PathBuf>,
    /// Interactive findings browser (needs the `tui` build feature + a terminal).
    #[cfg(feature = "tui")]
    #[arg(long)]
    pub tui: bool,
}

#[derive(Args)]
pub struct ReadArgs {
    /// The .ags file to read.
    pub file: PathBuf,
    /// The group code to dump (e.g. `LOCA`). Omit to list the file's group codes.
    pub group: Option<String>,
    /// Output CSV (quote-doubling) instead of the aligned table.
    #[arg(long, conflicts_with = "json")]
    pub csv: bool,
    /// Write the output to <path> instead of stdout.
    #[arg(long, value_name = "PATH")]
    pub out: Option<PathBuf>,
}

#[derive(Args)]
#[command(group(ArgGroup::new("fixdest").args(["in_place", "fix_out"])))]
pub struct FixArgs {
    /// The .ags file to repair.
    pub file: PathBuf,
    #[command(flatten)]
    pub dict: DictArgs,
    /// Also apply the intent-guessing fixes (duplicate-heading rename, ambiguous
    /// dd/mm date canonicalisation, smart-quote→ASCII).
    #[arg(long)]
    pub risky: bool,
    /// Overwrite the source file in place.
    #[arg(long)]
    pub in_place: bool,
    /// Write the repaired file to <path>.
    #[arg(long, value_name = "PATH")]
    pub fix_out: Option<PathBuf>,
}

#[derive(Args)]
pub struct DiffArgs {
    /// The baseline .ags file.
    pub file: PathBuf,
    /// The revision .ags file to compare against.
    pub other: PathBuf,
    #[command(flatten)]
    pub dict: DictArgs,
}

#[derive(Args)]
pub struct CertifyArgs {
    /// The .ags file to certify (must validate error-clean).
    pub file: PathBuf,
    #[command(flatten)]
    pub dict: DictArgs,
    /// Also run Rule 20's on-disk check — recorded in the cert profile.
    #[arg(long)]
    pub check_files: bool,
    /// Write the certificate to <path> instead of <file>.ags.idx.
    #[arg(long, value_name = "PATH")]
    pub out: Option<PathBuf>,
}

const HELP_EPILOG: &str = "\
exit codes:
  0  clean            1  findings          3  not found / unreadable
  4  not AGS4         5  bad arguments     6  schema violation

A bare `lat <file.ags>` is shorthand for `lat validate <file.ags>`.";
