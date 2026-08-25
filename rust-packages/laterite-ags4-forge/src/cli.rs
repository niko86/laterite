//! Command-line surface (clap derive) — the gogcli → discrawl →
//! cli-printing-press lineage that laterite-ags4-corpus-qa/lat
//! follow: a small set of **global** flags (valid before/after the
//! subcommand) controlling output mode + side-effects, results to
//! **stdout** in the resolved mode, progress to **stderr**, `ndjson`
//! automatically when piped, typed exit codes in the `after_help`.

use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};
use laterite_cliutil::OutputMode;

#[derive(Parser)]
#[command(
    name = "laterite-ags4-forge",
    about = "Evolutionary AGS4 dual-validation dogfood generator",
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
    /// Print what would happen and write nothing — mutate nothing.
    #[arg(long, global = true)]
    pub dry_run: bool,
    /// Never prompt; fail cleanly (for autonomous / agent / CI use).
    #[arg(long, global = true)]
    pub no_input: bool,
    /// Token-lean machine output: counts only, drop the per-candidate
    /// array (json/ndjson) / per-row list (table).
    #[arg(long, global = true)]
    pub compact: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Dual-validate one existing AGS4 file (Rust + python-ags4) and
    /// classify it with the shared parity model.
    Check(CheckArgs),
    /// Generate candidate AGS4 files (synthesize a realistic spec-valid
    /// base; inject single rule violations with `--inject` and/or
    /// multi-fault combinations with `--combine`) and dual-validate each.
    Gen(GenArgs),
    /// Run the evolutionary loop: synthesize → inject → dual-validate
    /// (confidence-gated) → fitness/staleness → auto-permute → frontier.
    Run(RunArgs),
    /// Corpus-gap divergence miner: synthesize every rule-combination
    /// across a placement-seed sweep, subtract what the python-ags4
    /// fixture corpus already covers, and spend the oracle on the novel
    /// divergence-prone signatures (`--always-validate` for all gaps).
    Mine(MineArgs),
    /// List the rule-fault injectors (token → target AGS rule, scaffold,
    /// mutation) and the canonical rules that aren't single-injectable —
    /// the injector→rule map the compliance/parity matrix builds on.
    Catalog,
    /// Generate constraint-valid BS 5930 soil descriptions (the realistic
    /// `GEOL_DESC` engine) — a preview of the synthetic strata descriptions.
    Describe(DescribeArgs),
    /// Synthesize a valid AGS4 file at a target byte size (the scale
    /// ladder): calibrate the borehole count to `--size` and write it out.
    Scale(ScaleArgs),
    /// ddmin-shrink a divergence-producing .ags to a minimal,
    /// signature-preserving reproducer (e.g. a corpus-qa ACTION file).
    Minimize(MinimizeArgs),
    /// Apply structured edits to a real .ags file — set/blank a cell, add
    /// or delete a row, drop a column or a whole group — leaving every
    /// line no operation names byte-for-byte alone.
    Edit(EditArgs),
    /// Author / schema-check / explain a declarative strategy file
    /// (the author↔CLI contract). `validate` runs nothing (read-only).
    Strategy(StrategyArgs),
    /// Inspect the persistent parity-confidence ledger.
    Confidence(ConfidenceArgs),
    /// Vendor python-ags4's upstream `tests/` breaking corpus (opt-in
    /// seed source) — pinned, immutable, with provenance.
    SeedVendor(VendorArgs),
}

#[derive(Args)]
pub struct CheckArgs {
    /// The .ags file(s) to dual-validate. A directory is walked
    /// recursively for `.ags` files, and several paths may be given at
    /// once. One file in gives the single-file document; a directory or
    /// more than one path gives the sweep document (a verdict tally plus
    /// one entry per file).
    #[arg(required = true)]
    pub paths: Vec<PathBuf>,
    /// python-ags4 per-file timeout (seconds).
    #[arg(long, default_value_t = 120)]
    pub timeout: u64,
    /// Skip python-ags4 entirely (Rust-only).
    #[arg(long)]
    pub no_oracle: bool,
}

#[derive(Args)]
pub struct GenArgs {
    /// Relational scaffold for the synthetic base: `minimal` (PROJ/TRAN),
    /// `loca-samp` (adds LOCA→SAMP→GEOL+ABBR — unlocks the Rule 10a/10c/16
    /// blind spots), or `wide` (loca-samp + every safe LOCA-child group +
    /// the lab-test-result depth below SAMP and the LBSG/LBST schedule — a
    /// ~100-group file for the perf/compliance surface).
    #[arg(long, default_value = "loca-samp")]
    pub scaffold: String,
    /// Deterministic seed — the synthetic file is realistic + varied;
    /// the seed only controls reproducibility (same seed → same bytes).
    #[arg(long, default_value_t = 0)]
    pub seed: u64,
    /// Single-rule violation to inject into the clean base. One
    /// single-fault candidate each.
    #[arg(long = "inject", long_help = crate::ops::Injection::inject_help())]
    pub inject: Vec<String>,
    /// Combined multi-fault candidate: a comma-separated list of inject
    /// tokens applied to ONE base (e.g. `rule10a,rule8,rule5`). Repeatable
    /// — each value is one combined candidate. The faults interact, so the
    /// report's rust/python rule-sets are the file's *actual* validation
    /// result, not the assumed union of the tokens' target rules.
    #[arg(long = "combine")]
    pub combine: Vec<String>,
    /// Also dual-validate each generated candidate (needs python-ags4
    /// for the parity verdict; Rust always runs).
    #[arg(long)]
    pub validate: bool,
    /// python-ags4 per-file timeout (seconds).
    #[arg(long, default_value_t = 120)]
    pub timeout: u64,
    /// Skip python-ags4 (Rust-only verdicts).
    #[arg(long)]
    pub no_oracle: bool,
    /// `wide` scaffold only: the per-sample probability each lab-test result is
    /// present (0.0–1.0). Default 1.0 = dense (every sample has every test);
    /// e.g. `--lab-test-rate 0.4` gives a realistic sparse test matrix (seeded →
    /// deterministic). No effect on `minimal`/`loca-samp` (no lab depth there).
    #[arg(long = "lab-test-rate", default_value_t = 1.0)]
    pub lab_test_rate: f64,
    /// Artifact dir (default: $`AGS4_FORGE_DIR` or ./forge-runs).
    #[arg(long)]
    pub out_dir: Option<PathBuf>,
}

#[derive(Args)]
pub struct RunArgs {
    /// Relational scaffold: `minimal` | `loca-samp` (default).
    #[arg(long, default_value = "loca-samp")]
    pub scaffold: String,
    /// Injector pool (repeatable). Empty → the built-in blind-spot
    /// backlog.
    #[arg(long = "inject", long_help = crate::ops::Injection::inject_help())]
    pub inject: Vec<String>,
    #[arg(long, default_value_t = 200)]
    pub max_generations: u64,
    #[arg(long, default_value_t = 5000)]
    pub max_candidates: u64,
    #[arg(long, default_value_t = 900)]
    pub max_wall_secs: u64,
    /// Hard cap on python-ags4 subprocess calls (the dominant cost).
    #[arg(long, default_value_t = 400)]
    pub python_budget: u64,
    #[arg(long, default_value_t = 20)]
    pub stale_soft: u64,
    #[arg(long, default_value_t = 60)]
    pub stale_hard: u64,
    /// Residual per-class oracle sample rate — never 0.
    #[arg(long, default_value_t = 0.01)]
    pub floor: f64,
    /// Forced oracle calls right after a class's trust collapses.
    #[arg(long, default_value_t = 25)]
    pub force_burst: u32,
    /// Deterministic RNG seed.
    #[arg(long, default_value_t = 42)]
    pub seed: u64,
    /// python-ags4 per-file timeout (seconds).
    #[arg(long, default_value_t = 120)]
    pub timeout: u64,
    /// Skip python-ags4 (Rust-only — deterministic, no parity verdict).
    #[arg(long)]
    pub no_oracle: bool,
    /// Artifact dir (default: $`AGS4_FORGE_DIR` or ./forge-runs). The
    /// confidence ledger persists at <out>/confidence.json across runs.
    #[arg(long)]
    pub out_dir: Option<PathBuf>,
    /// Declarative strategy file (.toml/.json) — overrides the flag
    /// defaults. The executable twin of a strategies/strat-forge-* page.
    #[arg(long)]
    pub strategy: Option<PathBuf>,
}

#[derive(Args)]
pub struct MineArgs {
    /// Relational scaffold: `minimal` | `loca-samp` (default — every
    /// injector is applicable).
    #[arg(long, default_value = "loca-samp")]
    pub scaffold: String,
    /// Directory of .ags files whose Rust signatures define the "covered"
    /// rule-break shapes (default: the vendored python-ags4 corpus at
    /// vendor/pyags4-tests — run `forge seed vendor` first). Absent/empty
    /// → empty covered-set (every synthesized combination reads as a gap).
    #[arg(long)]
    pub corpus: Option<PathBuf>,
    /// Smallest combination size to synthesize.
    #[arg(long, default_value_t = 2)]
    pub min_k: usize,
    /// Largest combination size to synthesize.
    #[arg(long, default_value_t = 3)]
    pub max_k: usize,
    /// Placement seeds tried per combination — placement variety teases
    /// distinct signatures out of the same combination (the search axis).
    #[arg(long, default_value_t = 4)]
    pub seeds: u64,
    /// Base RNG seed for the placement sweep.
    #[arg(long, default_value_t = 0)]
    pub seed: u64,
    /// Dual-validate EVERY synthesized gap with python-ags4, not just the
    /// divergence-prone signatures (the default, budget-saving policy).
    #[arg(long)]
    pub always_validate: bool,
    /// Hard cap on python-ags4 calls (the dominant cost).
    #[arg(long, default_value_t = 50)]
    pub max_oracle: usize,
    /// python-ags4 per-file timeout (seconds).
    #[arg(long, default_value_t = 120)]
    pub timeout: u64,
    /// Skip python entirely (Rust-only: profile + gaps + divergence-prone
    /// flags, no parity verdict).
    #[arg(long)]
    pub no_oracle: bool,
    /// Artifact dir (default: $`AGS4_FORGE_DIR` or ./forge-runs).
    #[arg(long)]
    pub out_dir: Option<PathBuf>,
}

#[derive(Args)]
pub struct DescribeArgs {
    /// How many descriptions to generate.
    #[arg(long, default_value_t = 10)]
    pub count: u64,
    /// Base seed — description `i` uses `seed + i` (same seed → same text).
    #[arg(long, default_value_t = 0)]
    pub seed: u64,
    /// Draw organic soils and PEAT as well as the natural inorganic lanes.
    ///
    /// Opt-in, and deliberately so: the lane needs a third branch where the
    /// engine has a coarse/fine coin flip, which shifts every draw after it.
    /// Without this flag the engine draws exactly as it did before the lane
    /// existed, so a consumer's committed output does not move under them.
    #[arg(long)]
    pub organic: bool,
    /// Keep only descriptions with this principal (`CLAY`, `SILT`, `SAND`,
    /// `GRAVEL`, `PEAT`). Repeatable. `PEAT` implies `--organic`.
    ///
    /// The seed→text mapping is unchanged: this filters the drawn pool rather
    /// than steering the draw, so `--seed N` still means what it meant.
    #[arg(long = "principal", value_name = "PRINCIPAL")]
    pub principal: Vec<String>,
    /// Keep only descriptions from this lane (`coarse` | `fine` | `peat`).
    /// Repeatable. `peat` implies `--organic`.
    #[arg(long = "lane", value_name = "LANE")]
    pub lane: Vec<String>,
}

#[derive(Args)]
pub struct ScaleArgs {
    /// Target file size: `500KB`, `50MB`, `1GB`, or a raw byte count
    /// (decimal K/M/G; trailing `B` optional). The borehole count is
    /// calibrated to land near it.
    #[arg(long)]
    pub size: String,
    /// Scaffold to scale (`loca-samp` | `wide`, the default — a ~50-group
    /// file). `minimal` doesn't scale with boreholes and is rejected.
    #[arg(long, default_value = "wide")]
    pub scaffold: String,
    /// Deterministic seed (same size + seed → byte-identical file).
    #[arg(long, default_value_t = 0)]
    pub seed: u64,
    /// Spread a rule-fault across the scaled file at a controllable *density*
    /// — the fault-density mode. Takes an `--inject` token, but only the
    /// per-row/per-cell (density-capable) ones: `rule10b|rule10c|rule8|rule5|
    /// rule16` (aliases too), or `none`. ABSENT → a clean scale, byte-identical
    /// to the pre-density behaviour (the existing fixtures do not move).
    #[arg(long = "inject")]
    pub inject: Option<String>,
    /// Fraction (0.0, 1.0] of the injector's applicable sites to corrupt.
    /// Requires `--inject`. Default when `--inject` is given: 1.0 (all sites).
    #[arg(long)]
    pub density: Option<f64>,
    /// Where to write the .ags (default: <out-dir>/scale/<scaffold>_<size>.ags).
    #[arg(long)]
    pub out: Option<PathBuf>,
    /// Artifact dir for the default output path (default: $`AGS4_FORGE_DIR`
    /// or ./forge-runs).
    #[arg(long)]
    pub out_dir: Option<PathBuf>,
}

#[derive(Args)]
pub struct EditArgs {
    /// The .ags file to edit. Required by every mode except
    /// `--patch-template`, which prints the patch shape and reads no file.
    #[arg(required_unless_present = "patch_template")]
    pub file: Option<PathBuf>,
    /// `GROUP:ROW:HEADING=VALUE`. ROW counts the file's ORIGINAL data rows,
    /// 1-indexed, so a patch reads the way it was written however much it
    /// changes. Repeatable.
    #[arg(long, value_name = "GROUP:ROW:HEADING=VALUE")]
    pub set: Vec<String>,
    /// `GROUP:ROW:HEADING` — empty the cell, keeping the field. Repeatable.
    #[arg(long, value_name = "GROUP:ROW:HEADING")]
    pub blank: Vec<String>,
    /// `GROUP:ROW`. Repeatable.
    #[arg(long, value_name = "GROUP:ROW")]
    pub delete_row: Vec<String>,
    /// `GROUP:HEADING` — drop the heading and its cell from every row,
    /// descriptor rows included. Repeatable.
    #[arg(long, value_name = "GROUP:HEADING")]
    pub delete_column: Vec<String>,
    /// `GROUP` — the GROUP/HEADING/UNIT/TYPE rows, the data rows, and the
    /// blank separator that followed. Repeatable.
    #[arg(long, value_name = "GROUP")]
    pub delete_group: Vec<String>,
    /// `GROUP` — append one empty data row, padded to the group's heading
    /// count. Repeatable. Use `--patch` to append a row WITH values.
    #[arg(long, value_name = "GROUP")]
    pub add_row: Vec<String>,
    /// A patch file (`.toml` or `.json`) of operations — the form that
    /// carries values, survives review, and can be re-run. Combines with
    /// the flags above; `--patch-template` prints a worked example.
    #[arg(long, value_name = "FILE")]
    pub patch: Option<PathBuf>,
    /// Print a commented patch-file template and exit.
    #[arg(long)]
    pub patch_template: bool,
    /// Write the edited file here. Without it (and without `--in-place`)
    /// nothing is written: the report says what WOULD change.
    #[arg(long)]
    pub out: Option<PathBuf>,
    /// Overwrite the input file.
    #[arg(long, conflicts_with = "out")]
    pub in_place: bool,
}

#[derive(Args)]
pub struct MinimizeArgs {
    /// The .ags file to shrink (its current dual-validate signature is
    /// the invariant ddmin preserves).
    pub file: PathBuf,
    /// Skip python-ags4 (preserve the Rust-only signature only).
    #[arg(long)]
    pub no_oracle: bool,
    #[arg(long, default_value_t = 120)]
    pub timeout: u64,
    /// Write the minimal reproducer here (default: stdout note only).
    #[arg(long)]
    pub out: Option<PathBuf>,
}

#[derive(Args)]
pub struct StrategyArgs {
    /// `new` (print a commented template) · `validate <f>` (schema
    /// check, runs nothing) · `explain <f>` (print the resolved
    /// effective config + the blind-spot rotation).
    pub action: String,
    /// The strategy file (for validate / explain).
    pub file: Option<PathBuf>,
}

#[derive(Args)]
pub struct ConfidenceArgs {
    /// `show` (per-class + global P(Rust≡python) bounds) · `reset`
    /// (cold-start the ledger) · `export` (raw JSON to stdout).
    pub action: String,
    /// Artifact dir holding `confidence.json` (default: ./forge-runs).
    #[arg(long)]
    pub out_dir: Option<PathBuf>,
}

#[derive(Args)]
pub struct VendorArgs {
    /// python-ags4 git ref to pin — REQUIRED, no default (the pinned corpus
    /// version is a deliberate call, not a bury-it-in-a-default). Upstream
    /// tags are bare versions, e.g. `1.2.0`, not `v1.2.0`.
    #[arg(long)]
    pub r#ref: String,
    /// Where to drop the vendored corpus — REQUIRED, no default (e.g.
    /// `rust-packages/laterite-ags4-forge/vendor/pyags4-tests`).
    #[arg(long)]
    pub dest: PathBuf,
}

const HELP_EPILOG: &str = "--readme  print the full CLI guide and exit.

exit codes:
  0  success / no parity action (clean or documented divergence)
  1  parity action present — a real Rust↔python divergence to triage
  2  run stalled — stale_hard reached, frontier.json emitted; the
     next (permuted) strategy must be authored by hand
  3  I/O — file not found / out-dir unwritable
  5  bad args (unknown --scaffold / --inject / --combine token)

output modes (--output, or auto):
  table   styled summary (default in a TTY)
  json    indented document (pretty; coloured on a TTY)
  ndjson  the document on one line (default when piped)

--dry-run mutates nothing (gen prints the plan, writes no files).
--compact drops the per-candidate array for token-lean agent output.

artifacts: `gen` writes <out>/runs/<run-id>/ (the .ags candidates +
report.json); <out> defaults to ./forge-runs. Confirmed reproducers
belong in ags-wiki/.bootstrap/probes/, NEVER tests/fixtures/.";
