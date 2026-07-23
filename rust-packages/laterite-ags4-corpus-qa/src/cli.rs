//! Command-line surface (clap derive).
//!
//! Conventions follow the gogcli → discrawl → cli-printing-press
//! lineage that `lat` already uses: a small set of
//! **global** flags (`global = true`, valid before or after the
//! subcommand) controlling output mode + side-effects, results to
//! **stdout** in the resolved mode, progress to **stderr**, and
//! `ndjson` automatically when stdout is piped.

use std::path::PathBuf;

use clap::{ArgGroup, Args, Parser, Subcommand};
use laterite_cliutil::OutputMode;

#[derive(Parser)]
#[command(
    name = "laterite-ags4-corpus-qa",
    about = "Crawl a share, dogfood the AGS4 validator, parity-check vs python-ags4",
    version,
    after_help = HELP_EPILOG
)]
pub struct Cli {
    /// Output mode for the result document on stdout (default: table
    /// in a TTY, ndjson when piped — agent-friendly with no flag).
    #[arg(long, short, value_enum, global = true)]
    pub output: Option<OutputMode>,

    /// Shortcut for `--output json` (pretty / coloured on a TTY).
    #[arg(long, global = true, conflicts_with = "output")]
    pub json: bool,

    /// Disable ANSI colour (also honours the `NO_COLOR` env var).
    #[arg(long, env = "NO_COLOR", global = true)]
    pub no_color: bool,

    /// Suppress the progress spinner/bar on stderr.
    #[arg(long, short, global = true)]
    pub quiet: bool,

    /// Print what would happen and exit without copying, writing a
    /// manifest, or invoking the validator/python — mutate nothing.
    #[arg(long, global = true)]
    pub dry_run: bool,

    /// Never prompt / launch the `--pick` TUI; fail cleanly instead
    /// (for autonomous / agent / CI use).
    #[arg(long, global = true)]
    pub no_input: bool,

    /// Token-lean machine output: summary/counts only, drop the
    /// per-file arrays (json/ndjson) or the per-file triage/action
    /// enumeration (table).
    #[arg(long, global = true)]
    pub compact: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Recursively find .ags files under a (network) root, select a
    /// subset, and copy them into the corpus dir; write manifest.json.
    Crawl(CrawlArgs),
    /// Batch-run the clean-room Rust validator over a manifest;
    /// bucket Clean/Findings/HardError/Panic; write report.json.
    Validate(ValidateArgs),
    /// Cross-check the odd/failed files against python-ags4.
    Parity(ParityArgs),
    /// crawl → validate → parity in one invocation.
    Run(RunArgs),
    /// Freeze (`--out`) or drift-check (`--check`) a deterministic,
    /// privacy-scrubbed baseline of the validator's findings over a
    /// manifest — the parser-convergence finding-drift gate. Keyed by
    /// content sha256; stores only structural `(rule, line, group,
    /// field_index, severity)` tuples (no paths / filenames / finding
    /// text) so it's safe to commit and mirror publicly.
    Baseline(BaselineArgs),
    /// Anonymise harvested .ags files (gather → clean → check): scrub the
    /// sensitive cell values `sensitive_headings.json` classifies — IDs
    /// pseudonymised, coordinates blanked, names/labs/accreditation
    /// tokenised — and write hash-named files + a source-stripped
    /// manifest to --out-dir, so the cleaned corpus is shareable.
    Censor(CensorArgs),
}

#[derive(Args)]
pub struct CrawlArgs {
    /// Network/share/local root to walk (Windows UNC `\\srv\share` OK).
    #[arg(long)]
    pub root: PathBuf,
    /// Corpus working dir (default: $`AGS4_CORPUS_DIR` or ./corpus,
    /// relative to the current directory, created on demand).
    #[arg(long)]
    pub corpus_dir: Option<PathBuf>,
    /// Copy every .ags found.
    #[arg(long)]
    pub all: bool,
    /// Reservoir-sample N files while walking.
    #[arg(long, value_name = "N")]
    pub sample: Option<usize>,
    /// Interactive multi-select (needs the `tui` build feature).
    #[arg(long)]
    pub pick: bool,
    /// Copy parallelism — parallel file *copy* fan-out, not the walk
    /// (that's --walk-jobs). Default: available CPU cores.
    #[arg(long)]
    pub jobs: Option<usize>,
    /// Parallel directory walk: fan a `WalkDir` out over `root`'s
    /// top-level subdirs across N threads (default 1 = single-threaded,
    /// today's exact behaviour). The dominant cost on a slow network
    /// share. Sampling stays deterministic under any value (`--seed`).
    #[arg(long, default_value_t = 1)]
    pub walk_jobs: usize,
    /// Skip files larger than this many bytes.
    #[arg(long)]
    pub max_bytes: Option<u64>,
    /// Follow symlinks while walking (default: off).
    #[arg(long)]
    pub follow_links: bool,
    /// Manifest output path. Default: <corpus>/runs/<run-id>/manifest.json
    /// (an explicit path here wins and is left out of runs/).
    #[arg(long)]
    pub manifest: Option<PathBuf>,
    /// Run id for the artifacts dir (default: a fresh UTC timestamp).
    /// Crawl writes <corpus>/runs/<run-id>/ and points runs/latest at
    /// it so a later validate/parity finds it without a flag.
    #[arg(long)]
    pub run_id: Option<String>,
    /// Deterministic reservoir seed (testing / reproducibility).
    #[arg(long)]
    pub seed: Option<u64>,
}

#[derive(Args)]
pub struct ValidateArgs {
    #[arg(long)]
    pub manifest: Option<PathBuf>,
    #[arg(long)]
    pub corpus_dir: Option<PathBuf>,
    /// Dictionary edition: `auto` (default — per-file from `TRAN_AGS`) or
    /// a forced `4.0.3|4.0.4|4.1|4.1.1|4.2`.
    #[arg(long, default_value = "auto")]
    pub dict_version: String,
    #[arg(long)]
    pub show_warnings: bool,
    #[arg(long)]
    pub show_fyi: bool,
    /// Rule 20's on-disk check (sidecar `FILE/<fset>/<name>` tree) is
    /// **ON by default here** so the Rust verdict matches python-ags4's
    /// always-on filesystem stat — the parity harness then AGREEs on
    /// Rule 20 instead of needing an O-27 reconcile shim. Pass
    /// `--no-check-files` to opt out (e.g. validating emitted files
    /// that intentionally ship without a sidecar tree).
    #[arg(long = "no-check-files")]
    pub no_check_files: bool,
    /// Validation parallelism — files validated concurrently.
    /// Default: available CPU cores.
    #[arg(long)]
    pub jobs: Option<usize>,
    /// Report output path. Default:
    /// <corpus>/runs/<run-id>/report.json (explicit path wins). A path
    /// inside runs/ still repoints runs/latest; one outside does not.
    #[arg(long)]
    pub report: Option<PathBuf>,
    /// Which run to validate (default: the newest — `runs/latest`).
    /// Writing into this run also repoints runs/latest — last activity
    /// wins (re-validating an older --run-id makes it current again).
    #[arg(long)]
    pub run_id: Option<String>,
}

#[derive(Args)]
pub struct ParityArgs {
    #[arg(long)]
    pub report: Option<PathBuf>,
    #[arg(long)]
    pub manifest: Option<PathBuf>,
    #[arg(long)]
    pub corpus_dir: Option<PathBuf>,
    /// Extra random non-triage files to also parity-check.
    #[arg(long, default_value_t = 0)]
    pub parity_sample: usize,
    /// python-ags4 subprocess concurrency.
    #[arg(long, default_value_t = 2)]
    pub parity_jobs: usize,
    /// Per-file python timeout (seconds).
    #[arg(long, default_value_t = 120)]
    pub timeout: u64,
    /// `uv` executable.
    #[arg(long, default_value = "uv")]
    pub uv: String,
    /// Python wrapper script (default: <repo>/`tools/py_ags4_check_json.py`).
    #[arg(long)]
    pub wrapper: Option<PathBuf>,
    /// parity.json output. Default:
    /// <corpus>/runs/<run-id>/parity.json (explicit path wins). A path
    /// inside runs/ still repoints runs/latest; one outside does not.
    #[arg(long)]
    pub out: Option<PathBuf>,
    /// Which run to parity-check (default: the newest — `runs/latest`).
    /// Writing into this run also repoints runs/latest — last activity
    /// wins (re-checking an older --run-id makes it current again).
    #[arg(long)]
    pub run_id: Option<String>,
    #[arg(long)]
    pub seed: Option<u64>,
}

#[derive(Args)]
pub struct RunArgs {
    #[arg(long)]
    pub root: PathBuf,
    #[arg(long)]
    pub corpus_dir: Option<PathBuf>,
    #[arg(long)]
    pub all: bool,
    #[arg(long, value_name = "N")]
    pub sample: Option<usize>,
    #[arg(long)]
    pub pick: bool,
    /// `auto` (default — per-file from `TRAN_AGS`) or a forced edition.
    #[arg(long, default_value = "auto")]
    pub dict_version: String,
    #[arg(long, default_value_t = 0)]
    pub parity_sample: usize,
    #[arg(long)]
    pub jobs: Option<usize>,
    /// Parallel directory walk (default 1 = single-threaded).
    #[arg(long, default_value_t = 1)]
    pub walk_jobs: usize,
    /// Run id for this crawl→validate→parity run's artifacts dir
    /// (default: a fresh UTC timestamp, shared by all three stages).
    #[arg(long)]
    pub run_id: Option<String>,
    #[arg(long)]
    pub seed: Option<u64>,
}

/// `baseline` — freeze or drift-check the validator's findings over a
/// manifest. Exactly one of `--out` (freeze) / `--check` (gate) is
/// required (the `mode` arg-group).
#[derive(Args)]
#[command(group(ArgGroup::new("mode").required(true).args(["out", "check"])))]
pub struct BaselineArgs {
    /// Manifest to validate (default: the run named by --run-id, else
    /// runs/latest — same resolution as `validate`).
    #[arg(long)]
    pub manifest: Option<PathBuf>,
    #[arg(long)]
    pub corpus_dir: Option<PathBuf>,
    /// `auto` (default — per-file from `TRAN_AGS`) or a forced edition.
    #[arg(long, default_value = "auto")]
    pub dict_version: String,
    /// Include Warning-severity findings in the baseline (default:
    /// errors only, matching `validate`).
    #[arg(long)]
    pub show_warnings: bool,
    /// Include FYI-severity findings in the baseline.
    #[arg(long)]
    pub show_fyi: bool,
    /// Opt out of Rule 20's on-disk sidecar check (ON by default, as in
    /// `validate`).
    #[arg(long = "no-check-files")]
    pub no_check_files: bool,
    /// Validation parallelism (default: CPU cores).
    #[arg(long)]
    pub jobs: Option<usize>,
    /// Which run's manifest to baseline (default: runs/latest).
    #[arg(long)]
    pub run_id: Option<String>,
    /// FREEZE: write the captured baseline here. Mutually exclusive with
    /// --check.
    #[arg(long)]
    pub out: Option<PathBuf>,
    /// DRIFT-CHECK: compare a fresh capture against this committed
    /// baseline; print what moved and exit 1 on any drift. Mutually
    /// exclusive with --out.
    #[arg(long)]
    pub check: Option<PathBuf>,
}

/// `censor` — anonymise harvested files for sharing.
#[derive(Args)]
pub struct CensorArgs {
    /// Manifest of files to clean (default: runs/latest, like `validate`).
    #[arg(long)]
    pub manifest: Option<PathBuf>,
    #[arg(long)]
    pub corpus_dir: Option<PathBuf>,
    #[arg(long)]
    pub run_id: Option<String>,
    /// Output dir for the cleaned files + the scrubbed manifest.json.
    #[arg(long)]
    pub out_dir: PathBuf,
    /// Anonymise only N files, size-stratified small→large (for the
    /// empirical review pass). Default: every file in the manifest.
    #[arg(long, value_name = "N")]
    pub sample: Option<usize>,
    /// Replacement text for `token`-action cells (names, labs, etc).
    #[arg(long, default_value = "REDACTED")]
    pub token: String,
    /// Fully tokenise free-text DESCRIPTION columns too (default: keep the
    /// text but strip bracketed geological units like "[LONDON CLAY]").
    #[arg(long)]
    pub include_freetext: bool,
    /// Extra keyword(s) to redact wherever they appear in any data cell
    /// (ASCII case-insensitive substring → the token). Repeatable —
    /// e.g. --redact "Acme Geotech" --redact "Smith".
    #[arg(long, value_name = "SUBSTRING")]
    pub redact: Vec<String>,
    /// Keep non-standard (vendor-custom) columns and groups. By default
    /// they're DELETED — they're not in any AGS4 edition and can be very
    /// client-specific.
    #[arg(long)]
    pub keep_custom: bool,
    /// Override the embedded sensitive-headings list (point at a candidate
    /// `sensitive_headings.json` while iterating the overlay).
    #[arg(long)]
    pub sensitive: Option<PathBuf>,
}

const HELP_EPILOG: &str = "--readme  print the full CLI guide (what each
          command/parity does, examples) and exit.

exit codes:
  0  success / no triage items
  1  triage items present (validate) or parity actions (parity/run)
  3  I/O — share unreachable / manifest or report missing
  5  bad args (e.g. no selection mode, --pick without `tui`, or
     --pick with --no-input / no terminal)

output modes (--output, or auto):
  table   styled summary + triage/action list (default in a TTY)
  json    indented report document (pretty; coloured on a TTY)
  ndjson  the report document on one line (default when piped)

--dry-run mutates nothing: crawl walks + selects and prints the plan
(no copy, no manifest); validate/parity print what they would process;
run does the crawl preview then stops. --compact drops the per-file
arrays for token-lean machine output.

concurrency (which flag parallelizes which stage):
  crawl  --walk-jobs N   parallel directory walk — fans WalkDir out
                         over root's top-level subdirs (default 1 =
                         sequential; the dominant cost on slow shares).
  crawl  --jobs N        parallel file copy (default: CPU cores).
  validate --jobs N      parallel validation (default: CPU cores).
  parity --parity-jobs N python-ags4 subprocess fan-out (default 2).
  parity --parity-sample N  also parity-check N random non-triage
                            files on top of triage (default 0).
Reservoir sampling stays deterministic under any --walk-jobs: a
--seed sample sorts the path list before sampling, so the same seed
gives the same sample on any machine and any thread count.

artifacts: each run writes <corpus>/runs/<run-id>/{manifest,report,
parity}.json (re-runs no longer overwrite). <corpus>/runs/latest =
the most recent run written under runs/ by ANY stage
(crawl/validate/parity), so validate/parity need no flag — last
activity wins (re-running an older --run-id makes it current again).
An explicit --manifest/--report/--out OUTSIDE runs/ still wins and
does NOT move runs/latest; --run-id pins one. harvested/ is the
shared content cache (untouched, accumulates).";
