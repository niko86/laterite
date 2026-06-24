//! `lat-check` — CLI wrapper around the validator library.
//!
//! Presentation is deliberately matched to `ags5db` (the workspace's
//! read-side CLI): an `indicatif` progress spinner, a `comfy-table`
//! `UTF8_FULL` finding table with bold-cyan headers + alternating dim
//! rows, the same `NO_COLOR`/TTY colour gate, and rich-style coloured
//! JSON. Those shared primitives now live in the `laterite-cliutil` crate
//! findings-specific JSON/NDJSON/table *shaping* stays here. (The lib
//! still links none of it — `laterite-cliutil` is bin-only.)
//!
//! ```text
//! lat-check <file.ags> [--dict-version 4.1|4.2] [--dict <path>]
//!                       [--json] [--show-fyi] [--quiet]
//!                       [--fix [--in-place | --fix-out <path>]]
//! ```
//!
//! Exit codes (mirror ags5db's convention):
//!   0 clean · 1 findings · 3 not found / unreadable ·
//!   4 not UTF-8 / not an AGS4 file · 5 bad arguments / bad dictionary

use std::io;
// Only the `--tui` interactivity gate needs the `IsTerminal` trait;
// gated so the default build doesn't carry an unused import (-D warnings).
#[cfg(feature = "tui")]
use std::io::IsTerminal;
use std::path::{Path, PathBuf};
use std::process::exit;

use laterite_ags4_parse::parse_bytes;
use laterite_ags4_validator::dict::{Dictionary, FALLBACK};
use laterite_ags4_validator::{
    CheckOptions, DictVersion, Findings, ValidatorError, check_file, findings, fix_document,
    resolve_dict_version, tran_ags_of,
};
use laterite_cliutil::{Spinner, colour_enabled, styled_table, write_atomic, write_json_pretty};
use serde_json::{Map, Value};

/// Map an `--encoding <name>` label to an `encoding_rs` encoding. The
/// label set is intentionally narrow: the AGS4 transfer-format spec
/// prefers UTF-8 / ASCII; cp1252 and latin1 are the legacy producers
/// (Excel "Save as text" on Windows, older delivery systems). Other
/// WHATWG encodings flow through `Encoding::for_label_no_replacement`
/// case-insensitively for completeness.
fn resolve_encoding(label: &str) -> Option<&'static encoding_rs::Encoding> {
    // First the canonical short labels the user types.
    let trimmed = label.trim().to_ascii_lowercase();
    let canonical = match trimmed.as_str() {
        "utf-8" | "utf8" => Some(encoding_rs::UTF_8),
        "cp1252" | "windows-1252" => Some(encoding_rs::WINDOWS_1252),
        "latin1" | "latin-1" | "iso-8859-1" => Some(encoding_rs::WINDOWS_1252),
        // Latin-1 ≈ Windows-1252 except for the 0x80-0x9F range
        // (Latin-1 has C1 control chars there; CP1252 defines `€`,
        // `™`, etc.). For AGS4 input we treat them as the same — the
        // 128-159 range is where the divergence sits and CP1252 is
        // the strict superset python-ags4 uses by default.
        "iso-8859-15" | "latin9" | "latin-9" => Some(encoding_rs::ISO_8859_15),
        _ => None,
    };
    canonical.or_else(|| encoding_rs::Encoding::for_label(label.as_bytes()))
}

// The interactive findings browser. Compiled in ONLY with
// `--features tui`; the default (LLM/automation-facing) build links no
// `ratatui` and the `--tui` flag below isn't recognised. See the plan's
// guardrails G1/G6.
#[cfg(feature = "tui")]
#[path = "ags4_check_tui.rs"]
mod tui;

const USAGE: &str = "\
usage: lat-check <file.ags> [options]

  --dict-version <V>        force a bundled edition: auto (default —
                            picked from the file's TRAN_AGS) | 4.0.3 |
                            4.0.4 | 4.1 | 4.1.1 | 4.2
  --dict <path>             external dictionary override (not supported)
  --json                    machine-readable findings (pretty JSON)
  --ndjson                  one flat JSON object per finding per line
  --out <path>              write the active format to <path> instead
                            of stdout (prints a one-line confirmation)
  --json-out <path>         also write the JSON report to <path> while
                            the normal report still prints (tee)
  --fix                     mechanically repair the file: apply the SAFE
                            fixes (CRLF / BOM / embedded-CR / short-row
                            pad / numeric reformat / TRAN rows) and write
                            the result. Non-destructive — writes a sibling
                            <file>.fixed.ags by default (see --in-place /
                            --fix-out). Exit 0 if the result is clean, 1 if
                            findings remain that can't be auto-fixed.
  --fix-risky               like --fix but also apply the intent-guessing
                            fixes (duplicate-heading rename, dd/mm date
                            canonicalisation, smart-quote→ASCII)
  --in-place                with --fix: overwrite the source file in place
  --fix-out <path>          with --fix: write the repaired file to <path>
  --diff <other.ags>        compare the input file against <other> and print the
                            KEY-aware/type-aware revision delta (per-group
                            +added -removed ~changed; --json for the full delta)
  --no-warnings             errors only — suppress the WARNING tier, which is
                            shown by default (malformed DICT, nonstandard
                            abbreviations, unrecognised TRAN_AGS edition)
  --show-fyi                include FYI-severity findings (e.g. Rule 1)
  --check-files             also run Rule 20's on-disk check: the
                            sidecar FILE/<fset>/<name> tree must exist
                            next to the .ags. Default OFF — data-level
                            Rule 20 is path-independent (see O-27);
                            enable for a packaging/QA pass on a real
                            delivery on disk.
  --encoding <name>         source file encoding (default utf-8).
                            Accepts utf-8 / cp1252 / latin1 /
                            iso-8859-1 / iso-8859-15. Use for legacy
                            files with extended-ASCII descriptions.
  --list-rules              print the AGS4 rule catalogue (title /
                            severity / fixable / cited observations) and
                            exit; add --json for the full machine form.
                            No input file needed.
  --quiet                   suppress the progress spinner
  --tui                     interactive findings browser (needs the
                            `tui` build feature + an interactive terminal)
  --readme                  print the full CLI guide and exit
  -h, --help                this message

exit: 0 clean · 1 findings · 3 unreadable · 4 not AGS4 · 5 bad args";

fn main() {
    // `--readme` → embedded CLI guide to stdout, exit 0 (before argv
    // parsing, like --help). Version-locked via include_str!.
    laterite_cliutil::print_readme_if_requested(include_str!("../README-cli.md"));

    let mut path: Option<PathBuf> = None;
    // Default report verbosity (#203): like a compiler, lat-check shows errors AND
    // WARNINGs by default (the engine default is errors-only; this binding opts in).
    // `--no-warnings` drops to errors-only; `--show-fyi` adds the low-signal tier.
    let mut opts = CheckOptions {
        include_warnings: true,
        ..CheckOptions::default()
    };
    let mut json = false;
    let mut ndjson = false;
    let mut out_path: Option<PathBuf> = None;
    let mut json_out_path: Option<PathBuf> = None;
    let mut quiet = false;
    // Headless fix (#198): --fix applies safe fixes, --fix-risky also the
    // intent-guessing ones; the result lands at a sibling / in place / --fix-out.
    let mut fix = false;
    let mut fix_risky = false;
    let mut in_place = false;
    let mut fix_out: Option<PathBuf> = None;
    // `--diff <other.ags>` (#204): compare the input file against <other> and
    // print the KEY-aware/type-aware revision delta (--json for the full delta).
    let mut diff_path: Option<PathBuf> = None;
    // `--list-rules` (#197): informational, no input file — print the rule
    // catalogue and exit (the `--json` flag selects machine-readable output).
    let mut list_rules = false;
    // Only declared/used with the `tui` feature; without it, `--tui`
    // is an unknown option (exit 5) — guardrail G6.
    #[cfg(feature = "tui")]
    let mut tui_requested = false;

    let mut argv = std::env::args().skip(1);
    while let Some(a) = argv.next() {
        match a.as_str() {
            "-h" | "--help" => {
                println!("{USAGE}");
                exit(0);
            }
            "--json" => json = true,
            "--ndjson" => ndjson = true,
            "--out" => match argv.next() {
                Some(p) => out_path = Some(PathBuf::from(p)),
                None => {
                    eprintln!("error: --out expects a path");
                    exit(5);
                }
            },
            "--json-out" => match argv.next() {
                Some(p) => json_out_path = Some(PathBuf::from(p)),
                None => {
                    eprintln!("error: --json-out expects a path");
                    exit(5);
                }
            },
            "--quiet" => quiet = true,
            "--list-rules" => list_rules = true,
            "--fix" => fix = true,
            "--fix-risky" => {
                fix = true;
                fix_risky = true;
            }
            "--in-place" => in_place = true,
            "--fix-out" => match argv.next() {
                Some(p) => fix_out = Some(PathBuf::from(p)),
                None => {
                    eprintln!("error: --fix-out expects a path");
                    exit(5);
                }
            },
            "--diff" => match argv.next() {
                Some(p) => diff_path = Some(PathBuf::from(p)),
                None => {
                    eprintln!("error: --diff expects a path to the other file");
                    exit(5);
                }
            },
            #[cfg(feature = "tui")]
            "--tui" => tui_requested = true,
            // WARNINGs are on by default now; `--no-warnings` drops to errors-only.
            "--no-warnings" => opts.include_warnings = false,
            "--show-fyi" => opts.include_fyi = true,
            "--check-files" => opts.check_files = true,
            "--encoding" => match argv.next().as_deref() {
                Some(label) => match resolve_encoding(label) {
                    Some(enc) => opts.encoding = enc,
                    None => {
                        eprintln!(
                            "error: --encoding {label:?} not recognised \
                             (try utf-8 / cp1252 / latin1 / iso-8859-1)"
                        );
                        exit(5);
                    }
                },
                None => {
                    eprintln!("error: --encoding expects a name (e.g. cp1252)");
                    exit(5);
                }
            },
            "--dict-version" => match argv.next().as_deref() {
                Some("auto") => opts.dict_version = None,
                Some("4.0.3") => opts.dict_version = Some(DictVersion::V4_0_3),
                Some("4.0.4") => opts.dict_version = Some(DictVersion::V4_0_4),
                Some("4.1") => opts.dict_version = Some(DictVersion::V4_1),
                Some("4.1.1") => opts.dict_version = Some(DictVersion::V4_1_1),
                Some("4.2") => opts.dict_version = Some(DictVersion::V4_2),
                other => {
                    eprintln!(
                        "error: --dict-version expects auto|4.0.3|4.0.4|4.1|4.1.1|4.2, \
                         got {other:?}"
                    );
                    exit(5);
                }
            },
            "--dict" => match argv.next() {
                Some(p) => opts.custom_dict = Some(PathBuf::from(p)),
                None => {
                    eprintln!("error: --dict expects a path");
                    exit(5);
                }
            },
            other if other.starts_with('-') => {
                eprintln!("error: unknown option {other:?}\n\n{USAGE}");
                exit(5);
            }
            _ => {
                if path.is_some() {
                    eprintln!("error: more than one input file given\n\n{USAGE}");
                    exit(5);
                }
                path = Some(PathBuf::from(a));
            }
        }
    }

    // `--list-rules`: informational, input-independent — print the catalogue
    // and exit (before the required-input-file check), like `--readme`.
    if list_rules {
        run_list_rules(json);
    }

    let Some(path) = path else {
        eprintln!("error: no input file\n\n{USAGE}");
        exit(5);
    };

    if json && ndjson {
        eprintln!("error: --json and --ndjson are mutually exclusive");
        exit(5);
    }

    // `--diff <b>`: compare the input file against another and exit (never falls
    // through to the validate-report below).
    if let Some(other) = diff_path.as_deref() {
        run_diff(&path, other, &opts, json, quiet);
    }

    // `--fix`: repair-and-write path. Runs the shared `fix_document` and exits;
    // it never falls through to the validate-report below.
    if fix {
        run_fix(&path, &opts, fix_risky, in_place, fix_out.as_deref(), quiet);
    } else if in_place || fix_out.is_some() {
        eprintln!("error: --in-place / --fix-out only apply with --fix");
        exit(5);
    }

    let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("file");
    let spinner = Spinner::start(&format!("validating {name}..."), quiet);

    let result = check_file(&path, &opts);
    drop(spinner); // clear the animation before any output

    match result {
        Ok(found) => {
            let n = findings::count(&found);

            // Guardrails G2/G3/G4: the TUI engages only when explicitly
            // requested AND both stdin/stdout are real terminals. When
            // not a TTY (piped/CI/agent) the existing structured/plain
            // path runs unchanged with one stderr notice — a piped
            // `--tui` run is observationally identical on stdout to a
            // non-`--tui` run. `check_file` errors are handled below in
            // the `Err` arm, *before* this, so the TUI is never entered
            // on a hard error.
            #[cfg(feature = "tui")]
            if tui_requested {
                let interactive = io::stdin().is_terminal() && io::stdout().is_terminal();
                if interactive {
                    if json {
                        eprintln!(
                            "note: --tui is active; --json is ignored on an \
                             interactive terminal"
                        );
                    }
                    if let Err(e) = tui::run(&found, name, opts.dict_version) {
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

            // `--json-out`: always tee a JSON artifact, independent of
            // the stdout format. The normal report below still prints.
            if let Some(p) = &json_out_path {
                if let Err(e) = write_atomic(p, json_string(&path, &found).as_bytes()) {
                    eprintln!("error: --json-out {}: {e}", p.display());
                    exit(3);
                }
                eprintln!("note: JSON written to {}", p.display());
            }

            // `--out`: redirect the active format to a file instead of
            // stdout; print one deterministic confirmation line.
            if let Some(p) = &out_path {
                let body = if json {
                    json_string(&path, &found)
                } else if ndjson {
                    ndjson_string(&found)
                } else {
                    plain_string(&path, &found, n)
                };
                if let Err(e) = write_atomic(p, body.as_bytes()) {
                    eprintln!("error: --out {}: {e}", p.display());
                    exit(3);
                }
                println!("wrote {n} finding(s) to {}", p.display());
                exit(code);
            }

            // No `--out`: the normal stdout path. Plain + `--json` are
            // byte-identical to before (coloured on a TTY); `--ndjson`
            // is the new flat per-finding stream.
            if json {
                emit_json(&path, &found);
            } else if ndjson {
                print!("{}", ndjson_string(&found));
            } else if n == 0 {
                println!("{}: clean (0 findings)", path.display());
            } else {
                report_table(&path, &found, n);
            }
            exit(code);
        }
        Err(e) => {
            eprintln!("error: {e}");
            exit(match e {
                ValidatorError::NotFound(_) | ValidatorError::Io { .. } => 3,
                ValidatorError::NotAgs4(_) | ValidatorError::UnsupportedEdition { .. } => 4,
                ValidatorError::BadDict { .. } => 5,
            });
        }
    }
}

/// `--fix`: mechanically repair the file via the validator's shared
/// `fix_document`, write the result, print a terse summary, and exit. Safe
/// fixes always; `risky` also applies the intent-guessing ones. Destination:
/// the source itself (`in_place`), an explicit `fix_out`, or a sibling
/// `<file>.fixed.ags`. Exit 0 if the repaired file is clean, 1 if findings
/// remain that aren't mechanically fixable (3/4/5 on read/parse/dict errors).
/// `--diff <b>`: the KEY-aware/type-aware revision delta between the input file
/// (`a`, baseline) and `b` (the revision), via the shared `laterite-ags4-diff`
/// leaf. `--json` emits the full `RevisionDelta`; otherwise a per-group summary.
fn run_diff(a: &Path, b: &Path, opts: &CheckOptions, json: bool, quiet: bool) -> ! {
    let spinner = Spinner::start("diffing...", quiet);
    let read = |p: &Path| match std::fs::read(p) {
        Ok(bytes) => bytes,
        Err(e) => {
            eprintln!("error: {}: {e}", p.display());
            exit(3);
        }
    };
    let raw_a = read(a);
    let raw_b = read(b);
    let parse = |raw: &[u8], p: &Path| match parse_bytes(raw, opts.encoding) {
        Ok(pf) => pf,
        Err(e) => {
            eprintln!("error: {}: {}", p.display(), ValidatorError::from(e));
            exit(4);
        }
    };
    let pa = parse(&raw_a, a);
    let pb = parse(&raw_b, b);
    drop(spinner);

    // KEY headings come from the dictionary; pick the edition from the revision
    // (b)'s TRAN_AGS (forced by --dict-version), falling back to the standard.
    let dv = resolve_dict_version(opts.dict_version, tran_ags_of(&pb).as_deref())
        .map(|(dv, _)| dv)
        .unwrap_or(FALLBACK);
    let dict = Dictionary::bundled(dv);
    let delta = laterite_ags4_diff::diff_parsed(&pa, &pb, &dict, None);

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&delta).unwrap_or_default()
        );
    } else {
        println!("{} → {}", a.display(), b.display());
        for g in &delta.groups {
            println!("  {:<6} +{} -{} ~{}", g.code, g.added, g.removed, g.changed);
        }
        if !delta.groups_added.is_empty() {
            println!("  groups added:   {}", delta.groups_added.join(", "));
        }
        if !delta.groups_removed.is_empty() {
            println!("  groups removed: {}", delta.groups_removed.join(", "));
        }
        println!(
            "  total: +{} added · −{} removed · ~{} changed",
            delta.total_added, delta.total_removed, delta.total_changed
        );
    }
    exit(0);
}

fn run_fix(
    path: &Path,
    opts: &CheckOptions,
    risky: bool,
    in_place: bool,
    fix_out: Option<&Path>,
    quiet: bool,
) -> ! {
    if in_place && fix_out.is_some() {
        eprintln!("error: --in-place and --fix-out are mutually exclusive");
        exit(5);
    }
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
    let outcome = match fix_document(&raw, opts, risky) {
        Ok(o) => o,
        Err(e) => {
            drop(spinner);
            eprintln!("error: {e}");
            exit(match e {
                ValidatorError::NotFound(_) | ValidatorError::Io { .. } => 3,
                ValidatorError::NotAgs4(_) | ValidatorError::UnsupportedEdition { .. } => 4,
                ValidatorError::BadDict { .. } => 5,
            });
        }
    };
    drop(spinner);

    let dest = if in_place {
        path.to_path_buf()
    } else if let Some(o) = fix_out {
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

/// `--list-rules`: print the AGS4 rule catalogue and exit. `--json` emits the
/// raw gated `rules_meta.json` (compile-time-embedded — no disk, no validation
/// run); otherwise a compact table. Input-independent (exits 0).
fn run_list_rules(json: bool) -> ! {
    let raw = laterite_ags4_validator::rule_metadata_json();
    if json {
        println!("{raw}");
        exit(0);
    }
    let doc: Value = serde_json::from_str(raw).expect("rules_meta.json is gated to parse");
    let rows: Vec<Vec<String>> = doc["rules"]
        .as_array()
        .map(|rs| {
            rs.iter()
                .map(|r| {
                    let fixable = r["fixable"].as_bool().unwrap_or(false);
                    vec![
                        r["rule"].as_str().unwrap_or("").to_string(),
                        r["title"].as_str().unwrap_or("").to_string(),
                        r["severity"].as_str().unwrap_or("").to_string(),
                        if fixable { "yes" } else { "" }.to_string(),
                    ]
                })
                .collect()
        })
        .unwrap_or_default();
    println!(
        "{}",
        styled_table(
            &["Rule", "Title", "Severity", "Fix?"],
            rows,
            colour_enabled(false)
        )
    );
    exit(0);
}

/// `delivery.ags` → `delivery.fixed.ags` (insert `.fixed` before the
/// extension); an extension-less `foo` → `foo.fixed`.
fn sibling_fixed_path(path: &Path) -> PathBuf {
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("out");
    let fname = match path.extension().and_then(|s| s.to_str()) {
        Some(ext) => format!("{stem}.fixed.{ext}"),
        None => format!("{stem}.fixed"),
    };
    path.with_file_name(fname)
}

/// Findings flattened to one row per finding, in spec-rule order (the
/// `Findings` map is a `BTreeMap`, so deterministic), rendered through
/// the shared `laterite-cliutil` `UTF8_FULL` grid (bold-cyan header,
/// alternating dim rows). `use_color` off → no ANSI (for `--out`
/// plain files / piped); on → the house TTY style. Returns the table
/// as a `String` so the binary owns no comfy-table types.
fn findings_table(found: &Findings, use_color: bool) -> String {
    let rows: Vec<Vec<String>> = found
        .iter()
        .flat_map(|(rule, items)| {
            // "AGS Format Rule 8" → "8" keeps the column tight; the
            // full label is redundant once it's a column.
            let short = rule
                .strip_prefix("AGS Format Rule ")
                .unwrap_or(rule)
                .to_string();
            items.iter().map(move |f| {
                let line = f.line.map(|l| l.to_string()).unwrap_or_else(|| "-".into());
                vec![short.clone(), line, f.group.clone(), f.desc.clone()]
            })
        })
        .collect();
    styled_table(&["Rule", "Line", "Group", "Description"], rows, use_color).to_string()
}

fn report_table(path: &Path, found: &Findings, n: usize) {
    println!("{}: {n} finding(s)", path.display());
    println!("{}", findings_table(found, colour_enabled(false)));
}

/// `--json`: `{file, findings:{rule:[{line,group,desc}]}}` rendered
/// rich-style (keys bold-cyan, strings green, numbers yellow, literals
/// magenta, structure dim) when colour is on — byte-identical token
/// palette to `ags5db`'s coloured JSON; plain pretty JSON otherwise.
/// The nested report value `{file, findings:{rule:[{line,group,desc}]}}`.
/// Shared by the stdout `--json` path and the `--out`/`--json-out`
/// file writers so they never disagree.
fn json_value(path: &Path, found: &Findings) -> Value {
    let mut fmap = Map::new();
    for (rule, items) in found {
        // Serialize the engine `Finding` directly — one model, surfaced
        // everywhere. Line-only error findings still emit exactly
        // `{line,group,desc}` (location/severity skip at default); migrated
        // findings additively gain target/field_index/heading/char_span/severity.
        let arr: Vec<Value> = items
            .iter()
            .map(|f| serde_json::to_value(f).unwrap_or(Value::Null))
            .collect();
        fmap.insert(rule.clone(), Value::Array(arr));
    }
    let mut root = Map::new();
    root.insert("file".into(), Value::from(path.display().to_string()));
    root.insert("findings".into(), Value::Object(fmap));
    Value::Object(root)
}

/// Plain pretty JSON (never coloured) — for files (`--out`/`--json-out`).
fn json_string(path: &Path, found: &Findings) -> String {
    serde_json::to_string_pretty(&json_value(path, found)).unwrap_or_default()
}

/// One flat JSON object per finding per line (NDJSON). Stream/grep
/// friendly; identical whether it goes to stdout or a file (no
/// colour). Empty (no lines) when there are zero findings.
fn ndjson_string(found: &Findings) -> String {
    let mut s = String::new();
    for (rule, items) in found {
        for f in items {
            // Build `rule`-first (the historical NDJSON key position), then
            // splice in the serialized `Finding` body so line-only findings
            // stay `{rule,line,group,desc}` byte-for-byte and migrated ones
            // additively gain the rich location/severity keys.
            let mut o = Map::new();
            o.insert("rule".into(), Value::from(rule.clone()));
            if let Value::Object(body) = serde_json::to_value(f).unwrap_or(Value::Null) {
                o.extend(body);
            }
            s.push_str(&serde_json::to_string(&Value::Object(o)).unwrap_or_default());
            s.push('\n');
        }
    }
    s
}

/// The plain report rendered to a `String` (no colour) — for `--out`
/// when neither `--json` nor `--ndjson` is active. Mirrors what
/// `report_table` prints to a TTY, minus ANSI.
fn plain_string(path: &Path, found: &Findings, n: usize) -> String {
    if n == 0 {
        return format!("{}: clean (0 findings)\n", path.display());
    }
    format!(
        "{}: {n} finding(s)\n{}\n",
        path.display(),
        findings_table(found, false)
    )
}

/// `--json` to stdout: the report value rendered rich-style (coloured
/// on a TTY) or plain pretty otherwise, via the shared writer so every
/// CLI's JSON looks identical. `lat-check` has no `--no-color` flag,
/// only the `NO_COLOR` env (folded into `colour_enabled`).
fn emit_json(path: &Path, found: &Findings) {
    let v = json_value(path, found);
    let mut out = io::stdout().lock();
    let _ = write_json_pretty(&mut out, &v, colour_enabled(false));
}

#[cfg(test)]
mod tests {
    use super::*;
    use laterite_ags4_validator::findings::add;

    fn sample() -> Findings {
        let mut f = Findings::new();
        add(
            &mut f,
            "AGS Format Rule 8",
            Some(5),
            "LOCA",
            "bad \"value\"",
        );
        add(&mut f, "AGS Format Rule 9", None, "SAMP", "x");
        f
    }

    #[test]
    fn ndjson_is_one_object_per_finding() {
        let s = ndjson_string(&sample());
        let lines: Vec<&str> = s.lines().collect();
        assert_eq!(lines.len(), 2);
        let a: Value = serde_json::from_str(lines[0]).unwrap();
        assert_eq!(a["rule"], "AGS Format Rule 8");
        assert_eq!(a["line"], 5);
        assert_eq!(a["group"], "LOCA");
        assert_eq!(a["desc"], "bad \"value\""); // quotes escaped correctly
        let b: Value = serde_json::from_str(lines[1]).unwrap();
        assert!(b["line"].is_null()); // None → null
        // Zero findings → empty (no lines), valid NDJSON.
        assert_eq!(ndjson_string(&Findings::new()), "");
    }

    #[test]
    fn json_string_round_trips() {
        let v: Value = serde_json::from_str(&json_string(Path::new("x.ags"), &sample())).unwrap();
        assert_eq!(v["file"], "x.ags");
        assert!(v["findings"]["AGS Format Rule 8"].is_array());
    }

    #[test]
    fn plain_string_reports_clean_and_findings() {
        assert!(
            plain_string(Path::new("x.ags"), &Findings::new(), 0).contains("clean (0 findings)")
        );
        let p = plain_string(Path::new("x.ags"), &sample(), 2);
        assert!(p.contains("2 finding(s)") && p.contains("LOCA"));
    }
}
