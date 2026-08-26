//! Command-line surface (clap derive) for `lat` — the reworked AGS4 tool.
//!
//! Follows the workspace CLI lineage (`ags4-forge` / `ags4-corpus-qa`): `--quiet`
//! is the one **global** flag, results to **stdout**, progress to **stderr**, typed
//! exit codes in `after_help`. Each verb owns its flags, so a flag can't reach a
//! verb it doesn't belong to — which is the whole point of laterite-dev#545: `--json`/`--ndjson`
//! are declared **per-verb, only on the verbs that produce a report**, so a verb
//! that can't render JSON (`certify`, the transport verbs, …) rejects the flag
//! structurally instead of accepting it and silently rendering a table. They used to
//! be global bools honoured by 1 of 13 verbs; clap and these declarations are now the
//! gate that keeps that from recurring.
//!
//! `validate` keeps both `--json` and `--ndjson` with identical semantics, so the
//! byte-parity gate (`test_cli_*`) is unaffected. `--json` is `bool` per verb (not
//! the `OutputMode` enum the siblings use) so the report bytes match the pre-rework
//! CLI. `ndjson`↔`json` and `fix --in-place`↔`--fix-out` are the remaining
//! declared mutual exclusions.

use std::path::PathBuf;

use clap::builder::TypedValueParser; // for `.map()` on PossibleValuesParser
use clap::{ArgGroup, Args, Parser, Subcommand};
use laterite_ags4_merge::{MissingTranMode, TypeClashMode};

/// The known subcommand names — the `main` default-subcommand pre-scan uses this
/// to decide whether a bare `lat <file>` should have `validate` spliced in.
pub const SUBCOMMANDS: &[&str] = &[
    "validate", "read", "fix", "diff", "merge", "certify", "rules", "pack", "unpack", "lock",
    "unlock", "excel",
];

#[derive(Parser)]
#[command(name = "lat", about = "AGS4 validate / read / fix / diff / certify", version, after_help = HELP_EPILOG)]
pub struct Cli {
    /// Suppress the progress spinner. Genuinely cross-cutting (every verb has a
    /// spinner to quiet), so unlike `--json`/`--ndjson` it stays global.
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
    /// Merge N deliveries of one project into a single file — KEY-aware,
    /// argument-order recency, union-not-intersection.
    Merge(MergeArgs),
    /// Mint the `.ags.idx` validity certificate for an error-clean file.
    Certify(CertifyArgs),
    /// Print the AGS4 rule catalogue (no input file needed).
    Rules(RulesArgs),
    /// Dump this binary's own parser as JSON — the AUTHORITY the surface census
    /// diffs the uvx / npx launchers against (`tools/gen_census.py`). Hidden: a
    /// machine door, not a user command.
    #[command(hide = true)]
    Census,
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
    /// The file to package (any type — `.ags`, anything).
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
    /// Force a bundled edition: `auto` (default — from the file's `TRAN_AGS`) |
    /// 4.0.3 | 4.0.4 | 4.1 | 4.1.1 | 4.2. With `--dict`, selects the overlay BASE.
    #[arg(long, value_name = "V")]
    pub dict_version: Option<String>,
    /// Custom dictionary override (laterite-dev#568): an `.ags` or JSON dictionary layered over
    /// a base edition detected from the dictionary itself. Overrides of standard
    /// definitions are honoured with a warning.
    #[arg(long, value_name = "PATH")]
    pub dict: Option<PathBuf>,
    /// (with `--dict`) treat the custom dictionary as a FULL REPLACEMENT — no base
    /// edition contributes. Cannot be combined with `--dict-version`.
    #[arg(long)]
    pub dict_replace: bool,
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
    /// Machine-readable findings (pretty JSON).
    #[arg(long)]
    pub json: bool,
    /// One flat JSON object per finding per line.
    #[arg(long, conflicts_with = "json")]
    pub ndjson: bool,
    /// Errors only — suppress the WARNING tier (shown by default).
    #[arg(long)]
    pub no_warnings: bool,
    /// Fail on warnings too (like a compiler's `-Werror`). Warnings are shown
    /// by default but do not affect the exit code; this opts into that.
    /// Contradicts `--no-warnings`, which suppresses the tier entirely.
    #[arg(long, conflicts_with = "no_warnings")]
    pub warnings_as_errors: bool,
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
    /// Machine-readable output (JSON).
    #[arg(long)]
    pub json: bool,
    /// Output CSV (quote-doubling) instead of the aligned table.
    #[arg(long, conflicts_with = "json")]
    pub csv: bool,
    /// Write the output to <path> instead of stdout.
    #[arg(long, value_name = "PATH")]
    pub out: Option<PathBuf>,
    /// Read a file with duplicate headings (AGS4 Rule 7) instead of refusing it,
    /// suffixing the repeats `__2`, `__3`, … so no column is lost. The output is
    /// deliberately NOT valid AGS4 — this recovers data, it does not repair the
    /// file. To repair it, use `lat fix --risky`.
    #[arg(long)]
    pub recover_duplicate_headings: bool,
}

#[derive(Args)]
#[command(group(ArgGroup::new("fixdest").args(["in_place", "fix_out"])))]
pub struct FixArgs {
    /// The .ags file to repair.
    pub file: PathBuf,
    #[command(flatten)]
    pub dict: DictArgs,
    /// Machine-readable report of what was repaired (JSON).
    #[arg(long)]
    pub json: bool,
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
    /// Machine-readable delta (JSON).
    #[arg(long)]
    pub json: bool,
}

#[derive(Args)]
pub struct MergeArgs {
    /// The .ags deliveries to merge, earliest first — the LAST file wins a KEY
    /// conflict (argument order is authority). At least two.
    #[arg(required = true, num_args = 2..)]
    pub files: Vec<PathBuf>,
    /// Where to write the merged .ags. A deliberate choice, so required — never a
    /// silent default over one of the inputs.
    #[arg(long, value_name = "PATH")]
    pub out: PathBuf,
    /// How to settle a heading two deliveries typed differently:
    ///
    ///   error   — refuse (default; reconciling two producers' types is high-stakes)
    ///   widen   — fall back to X (free text); raw values untouched, TYPE thrown away
    ///   promote — keep the greatest precision when every code is nDP (e.g. 2DP + 5DP
    ///             -> 5DP) and zero-pad the coarser values; falls back to widen for
    ///             nSF/nSCI and cross-family clashes. The only mode that rewrites a cell.
    ///
    /// The allowed values are projected from `TypeClashMode::ALL`, so the CLI cannot
    /// drift from the library's vocabulary.
    #[arg(
        long,
        value_name = "MODE",
        default_value = "error",
        value_parser = clap::builder::PossibleValuesParser::new(TypeClashMode::ALL.map(|m| m.as_str()))
            .map(|s| s.parse::<TypeClashMode>().expect("clap restricted the value")),
    )]
    pub on_type_clash: TypeClashMode,
    /// What to do when none of the --tran-* flags is given and the deliveries
    /// carry TRAN rows of their own:
    ///
    ///   reconcile — merge TRAN like any other group and warn (default). Each
    ///               delivery's TRAN row survives, because `TRAN_ISNO` is a KEY
    ///               heading and issue numbers differ — and Rule 14 allows one.
    ///   error     — refuse, before anything is written to --out
    ///
    /// The allowed values are projected from `MissingTranMode::ALL`, so the CLI
    /// cannot drift from the library's vocabulary.
    #[arg(
        long,
        value_name = "MODE",
        default_value = "reconcile",
        value_parser = clap::builder::PossibleValuesParser::new(MissingTranMode::ALL.map(|m| m.as_str()))
            .map(|s| s.parse::<MissingTranMode>().expect("clap restricted the value")),
    )]
    pub on_missing_tran: MissingTranMode,
    /// Issue reference (`TRAN_ISNO`) for the merged file's own synthesised TRAN.
    /// With the other four --tran-* flags, a fresh merge-transmission TRAN is
    /// written (recording the inputs' ISNOs/dates in `TRAN_REM`); with none of
    /// them, TRAN is reconciled and a warning notes no stamp was supplied.
    #[arg(long, value_name = "ISNO")]
    pub tran_issue: Option<String>,
    /// Production date (`TRAN_DATE`, yyyy-mm-dd) for the merged file's TRAN.
    #[arg(long, value_name = "DATE")]
    pub tran_date: Option<String>,
    /// Producer / recipient / status for the merged TRAN. REQUIRED with the
    /// other two: all five are REQUIRED headings, so it is all five or none.
    /// (This line used to say "optional", which was true of an older
    /// issue-plus-date rule and has not been since.)
    #[arg(long, value_name = "NAME")]
    pub tran_producer: Option<String>,
    #[arg(long, value_name = "NAME")]
    pub tran_recipient: Option<String>,
    #[arg(long, value_name = "STATUS")]
    pub tran_status: Option<String>,
    /// What was transferred (`TRAN_DESC`). Genuinely optional — an OTHER
    /// heading, so it stands outside the all-five rule.
    #[arg(long, value_name = "TEXT")]
    pub tran_description: Option<String>,
    /// Free remarks (`TRAN_REM`). Optional, and APPENDED to merge's own
    /// provenance note rather than replacing it — both are true of the result.
    #[arg(long, value_name = "TEXT")]
    pub tran_remarks: Option<String>,
    #[command(flatten)]
    pub dict: DictArgs,
    /// Machine-readable merge report (JSON).
    #[arg(long)]
    pub json: bool,
}

#[derive(Args)]
pub struct RulesArgs {
    /// Machine-readable catalogue (JSON).
    #[arg(long)]
    pub json: bool,
}

#[derive(Args)]
pub struct CertifyArgs {
    /// The .ags file to certify (must validate error-clean).
    pub file: PathBuf,
    #[command(flatten)]
    pub dict: DictArgs,
    // `--check-files` was here. It recorded, in the certificate, that Rule 20's on-disk
    // half had run — and a later `lat validate --check-files --index` read that record
    // and skipped the check. Delete the FILE/ tree in between and the file still
    // reported clean: the .ags bytes had not moved, so the certificate was still
    // "valid". A certificate is a statement about bytes; the directory beside them is
    // not one, and there is now nowhere in the format to pretend otherwise. Use
    // `lat validate --check-files`, which runs it live, every time.
    /// Write the certificate to <path> instead of <file>.ags.idx.
    #[arg(long, value_name = "PATH")]
    pub out: Option<PathBuf>,
}

const HELP_EPILOG: &str = "\
exit codes:
  0  clean            1  findings          3  not found / unreadable
  4  not AGS4         5  bad arguments     6  schema violation

A bare `lat <file.ags>` is shorthand for `lat validate <file.ags>`.";
