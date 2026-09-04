//! The equivalence contract's CORPUS leg (dec-emit-streamed-verdict): for
//! every AGS4 file given, rebuild its groups as emit input (verbatim
//! strings), emit under Report mode — whose findings come from the
//! writer-built verdict — then independently `parse_bytes` the emitted
//! bytes and run the same rules over the re-parse. The two findings sets
//! must be identical: any structural divergence between the constructed
//! verdict and a real parse that the rule engine could see fires here.
//!
//! (The in-crate unit differential additionally holds FIELD equality on the
//! read-inventory over crafted shapes; this leg trades that localisation
//! for real-world data — the forge rungs and the parity corpus.)
//!
//! Usage: `verdict_differential <file-or-dir>…` — a directory sweeps every
//! `*.ags` under it. Prints one line per file; a file whose cells embed
//! CR/LF is a recorded refusal (the writer refuses those by contract), not
//! a silent skip. Exits non-zero on any mismatch.

use laterite_ags4_emit::{Cell, EmitMode, EmitOpts, GroupInput, emit_ags4};
use laterite_ags4_validator::parse::parse_bytes;
use laterite_ags4_validator::{CheckOptions, WorldScope, check_parsed};
use std::path::{Path, PathBuf};

fn collect(path: &Path, out: &mut Vec<PathBuf>) {
    if path.is_dir() {
        let mut entries: Vec<_> = std::fs::read_dir(path)
            .expect("readable dir")
            .map(|e| e.expect("dir entry").path())
            .collect();
        entries.sort();
        for e in entries {
            collect(&e, out);
        }
    } else if path
        .extension()
        .is_some_and(|x| x.eq_ignore_ascii_case("ags"))
    {
        out.push(path.to_path_buf());
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        eprintln!("usage: verdict_differential <file-or-dir>…");
        std::process::exit(2);
    }
    let mut files: Vec<PathBuf> = Vec::new();
    for a in &args {
        collect(Path::new(a), &mut files);
    }
    let opts = EmitOpts {
        mode: EmitMode::Report,
        ..EmitOpts::default()
    };
    let (mut ok, mut refused, mut failed) = (0usize, 0usize, 0usize);
    for f in &files {
        let bytes = std::fs::read(f).expect("readable file");
        let parsed = match parse_bytes(&bytes, encoding_rs::UTF_8) {
            Ok(p) => p,
            Err(e) => {
                println!("SKIP  {} (source does not parse: {e})", f.display());
                refused += 1;
                continue;
            }
        };
        // Rebuild the file's groups as emit input, values verbatim.
        let groups: Vec<GroupInput> = parsed
            .group_order
            .iter()
            .map(|code| {
                let g = &parsed.groups[code];
                GroupInput {
                    code: code.clone(),
                    headings: g.headings.clone(),
                    units: Some(g.units.clone()),
                    types: Some(g.types.clone()),
                    rows: g
                        .rows
                        .iter()
                        .map(|r| {
                            (0..r.n_values())
                                .map(|i| Cell::Text(g.value_at(r, i).unwrap_or("").to_string()))
                                .collect()
                        })
                        .collect(),
                }
            })
            .collect();
        drop(parsed);
        let res = match emit_ags4(&groups, &opts) {
            Ok(r) => r,
            Err(e) => {
                // The writer's refusal contract (embedded CR/LF, Rule 6) —
                // recorded, never silently dropped.
                println!("REFUSED  {} ({e})", f.display());
                refused += 1;
                continue;
            }
        };
        // The independent side: a REAL parse of the emitted bytes, same rules.
        let reparsed = parse_bytes(&res.bytes, encoding_rs::UTF_8).expect("emitted bytes parse");
        let copts = CheckOptions {
            dict_version: Some(opts.edition),
            ..CheckOptions::default()
        };
        let dict = laterite_ags4_validator::dict::Dictionary::bundled(opts.edition);
        let independent =
            check_parsed(&reparsed, &dict, &copts, &WorldScope::None).expect("check re-parse");
        if res.findings == independent {
            println!(
                "OK    {} ({} finding group(s))",
                f.display(),
                independent.len()
            );
            ok += 1;
        } else {
            println!("MISMATCH  {}", f.display());
            for k in independent.keys().chain(res.findings.keys()) {
                let a = res.findings.get(k).map_or(0, Vec::len);
                let b = independent.get(k).map_or(0, Vec::len);
                if a != b {
                    println!("    {k}: constructed={a} reparse={b}");
                }
            }
            failed += 1;
        }
    }
    println!("\n{ok} ok, {refused} refused/skipped (reported above), {failed} mismatched");
    if failed > 0 {
        std::process::exit(1);
    }
}
