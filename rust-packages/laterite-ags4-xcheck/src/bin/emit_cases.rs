// The clap `#[arg]` doc comments below are BOTH rustdoc and the CLI's help
// text, and they name their placeholders the way a CLI must — `<path>`,
// `<dir>`, `<run-id>`. rustdoc reads those as unclosed HTML tags. Every
// markdown-level fix (backticks, `\<path\>`) leaks straight into `--help`,
// and this crate's help text is mirrored byte-identically into README-cli.md
// and gated, so a rustdoc warning would be paid for in user-facing output.
//
// Allowed rather than fixed because the trade is one-sided HERE and nowhere
// else: `publish = false`, so no docs.rs reader exists to be misled. The
// published crates stay strict — #748's `cargo doc` gate is workspace-wide and
// this is the only class exempted from it.
#![allow(rustdoc::invalid_html_tags)]
//! `emit-cases` — the AUTHORITY leg of the cross-surface OUTPUT-VALUE gate
//! (plan `output/output-value-gate-plan.md` §2).
//!
//! Writes one observation file, `rust-leaf.json`, by driving the SHARED emit
//! leaf (`laterite_ags4_emit`) DIRECTLY — no binding, no launcher. Its column
//! is the REFERENCE the surface legs (`python`, `node`, `wasm`, the launchers)
//! are held to by `xcheck`, not a peer in an N-way vote: a divergence a single
//! surface carries alone (a private emitter, drift #1b) still fails, because it
//! disagrees with this column even when no other surface reproduces it.
//!
//!   emit-cases --out <dir> [--cases <dir>] [--repo-root <dir>]

use std::collections::BTreeMap;
use std::path::PathBuf;

#[path = "../xcheck_shared.rs"]
mod shared;
use shared::{
    AUTHORITY, BuildGroup, BuildOpts, Case, InlineGroup, LegObservations, Observation,
    load_manifests,
};

/// Canonical re-emit: parse a fixture and write it back through the shared
/// writer in the canonical shape (`trailing_blank_line = false`) — the exact
/// path `laterite-py`'s `Ags4File.text` and `laterite-node`'s `.text` take
/// post-#518. This is the reference those surfaces must reproduce byte-for-byte.
fn reemit_canonical(input_path: &PathBuf) -> Observation {
    let text = match std::fs::read_to_string(input_path) {
        Ok(t) => t,
        Err(e) => return Observation::Err(format!("Io: {e}")),
    };
    let parsed = match laterite_ags4_parse::parse_str(&text) {
        Ok(p) => p,
        Err(e) => return Observation::Err(format!("{e:?}")),
    };
    // Build the (code, matrix) blocks exactly as `Reading::emit` does: the tag
    // plus each of the group's n columns, a ragged row's tail padded with "".
    let blocks: Vec<(String, Vec<Vec<String>>)> = parsed
        .group_order
        .iter()
        .filter_map(|code| {
            let g = parsed.groups.get(code)?;
            let n = g.headings.len();
            let pad = |tag: &str, src: &[String]| {
                let mut row = Vec::with_capacity(n + 1);
                row.push(tag.to_string());
                for i in 0..n {
                    row.push(src.get(i).cloned().unwrap_or_default());
                }
                row
            };
            let mut matrix: Vec<Vec<String>> = Vec::with_capacity(3 + g.rows.len());
            let mut heading = Vec::with_capacity(n + 1);
            heading.push("HEADING".to_string());
            heading.extend(g.headings.iter().cloned());
            matrix.push(heading);
            matrix.push(pad("UNIT", &g.units));
            matrix.push(pad("TYPE", &g.types));
            for r in &g.rows {
                matrix.push(pad("DATA", &r.values));
            }
            Some((code.clone(), matrix))
        })
        .collect();

    let mut out = Vec::new();
    match laterite_ags4_emit::write_ags4_matrix(&mut out, &blocks, false) {
        Ok(()) => match String::from_utf8(out) {
            Ok(text) => Observation::Ok(serde_json::Value::String(text)),
            Err(e) => Observation::Err(format!("NotUtf8: {e}")),
        },
        // The shared writer's Rule-6 guard: a cell carrying an embedded CR/LF is
        // REFUSED, not torn into an illegal file. The canonical sentinel is the
        // error variant's name, single-sourced with the other legs by hand.
        Err(laterite_ags4_emit::EmitError::EmbeddedNewline { .. }) => {
            Observation::Err("EmbeddedNewline".into())
        }
        Err(e) => Observation::Err(format!("{e:?}")),
    }
}

/// Verbatim serialise an inline cell matrix through the shared writer in the
/// compat shape (`trailing_blank_line = true`) — the exact path
/// `compat.dataframe_to_AGS4` takes (`emit_ags4_compat` → `write_ags4_matrix`).
/// A cell carrying an embedded CR/LF is REFUSED (the Rule-6 guard, #423); this
/// is the reference `python-compat` must reproduce — refuse when it refuses,
/// same bytes when it doesn't.
fn emit_typed_verbatim(groups: &[InlineGroup]) -> Observation {
    let blocks: Vec<(String, Vec<Vec<String>>)> = groups
        .iter()
        .map(|g| (g.code.clone(), g.rows.clone()))
        .collect();
    let mut out = Vec::new();
    match laterite_ags4_emit::write_ags4_matrix(&mut out, &blocks, true) {
        Ok(()) => match String::from_utf8(out) {
            Ok(text) => Observation::Ok(serde_json::Value::String(text)),
            Err(e) => Observation::Err(format!("NotUtf8: {e}")),
        },
        Err(laterite_ags4_emit::EmitError::EmbeddedNewline { .. }) => {
            Observation::Err("EmbeddedNewline".into())
        }
        Err(e) => Observation::Err(format!("{e:?}")),
    }
}

/// The `build_ags4` door: run typed rows through the SHARED emit orchestrator
/// (`emit_ags4`) with the surfaces' shared defaults (`AutoFix`, fallback edition).
/// Every surface's build door — `laterite.build_ags4`, node `buildAgs4`, wasm
/// `build_ags4` — routes through this same orchestrator, so this column is the
/// reference they must reproduce; a surface that re-implemented build would
/// diverge from it.
fn build_typed(groups: &[BuildGroup], build_opts: Option<&BuildOpts>) -> Observation {
    let inputs: Vec<laterite_ags4_emit::GroupInput> = groups
        .iter()
        .map(|g| laterite_ags4_emit::GroupInput {
            code: g.code.clone(),
            headings: g.headings.clone(),
            units: g.units.clone(),
            types: g.types.clone(),
            rows: g.rows.clone(),
        })
        .collect();
    // AutoFix + 4.1.1 — the resolved project defaults every surface's build door
    // uses when the caller names neither. A case may turn on synthesis and state
    // a transmission; those are the only two knobs the build legs share.
    let opts = laterite_ags4_emit::EmitOpts {
        synthesise_metadata: build_opts.is_some_and(|o| o.synthesise_metadata),
        tran: build_opts.and_then(|o| o.tran.as_ref()).map(|t| {
            laterite_ags4_emit::TranStamp::new(
                &t.issue,
                &t.date,
                &t.producer,
                &t.recipient,
                &t.status,
            )
        }),
        ..laterite_ags4_emit::EmitOpts::default()
    };
    match laterite_ags4_emit::emit_ags4(&inputs, &opts) {
        Ok(res) => match String::from_utf8(res.bytes) {
            Ok(text) => Observation::Ok(serde_json::Value::String(text)),
            Err(e) => Observation::Err(format!("NotUtf8: {e}")),
        },
        Err(e) => Observation::Err(format!("{e:?}")),
    }
}

fn observe(case: &Case, repo_root: &std::path::Path) -> Option<Observation> {
    match case.op.as_str() {
        "reemit_canonical" => {
            let fixture = case.input.fixture.as_ref()?;
            Some(reemit_canonical(&repo_root.join(fixture)))
        }
        "emit_typed_verbatim" => {
            let groups = case.input.groups.as_ref()?;
            Some(emit_typed_verbatim(groups))
        }
        "build_typed" => {
            let groups = case.input.build.as_ref()?;
            Some(build_typed(groups, case.input.build_opts.as_ref()))
        }
        // Unknown op: the leg records nothing; `xcheck --require-legs all` turns
        // a case the authority silently skipped into a hard failure, so a new op
        // cannot ship half-wired.
        _ => None,
    }
}

fn main() {
    let mut out_dir = PathBuf::from("output/xcheck");
    let mut cases_dir = PathBuf::from("rust-packages/laterite-ags4-xcheck/cases");
    let mut repo_root = PathBuf::from(".");
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--out" => out_dir = args.next().map(PathBuf::from).expect("--out <dir>"),
            "--cases" => cases_dir = args.next().map(PathBuf::from).expect("--cases <dir>"),
            "--repo-root" => repo_root = args.next().map(PathBuf::from).expect("--repo-root <dir>"),
            other => {
                eprintln!("unknown arg: {other}");
                std::process::exit(2);
            }
        }
    }

    let cases = load_manifests(&cases_dir).unwrap_or_else(|e| {
        eprintln!("{e}");
        std::process::exit(2);
    });

    let mut observations: BTreeMap<String, Observation> = BTreeMap::new();
    for case in &cases {
        if !case.legs.iter().any(|l| l == AUTHORITY) {
            continue;
        }
        if let Some(obs) = observe(case, &repo_root) {
            observations.insert(case.id.clone(), obs);
        }
    }

    std::fs::create_dir_all(&out_dir).expect("create out dir");
    let payload = LegObservations {
        schema: 1,
        leg: AUTHORITY.into(),
        // The authority's engine is the reference every other leg is held to —
        // read from the linked crate, not passed in, so this leg cannot report an
        // engine it is not running.
        engine: Some(laterite_ags4_validator::ENGINE_FINGERPRINT.to_string()),
        cases: observations,
    };
    let path = out_dir.join(format!("{AUTHORITY}.json"));
    std::fs::write(
        &path,
        serde_json::to_string_pretty(&payload).expect("serialize"),
    )
    .expect("write authority observations");
    eprintln!(
        "{}: {} cases -> {}",
        AUTHORITY,
        payload.cases.len(),
        path.display()
    );
}
