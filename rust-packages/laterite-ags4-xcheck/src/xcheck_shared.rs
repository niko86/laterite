//! Shared contract for the cross-surface OUTPUT-VALUE gate (plan
//! `output/output-value-gate-plan.md`). Both the AUTHORITY leg (`emit-cases`)
//! and the COMPARATOR (`xcheck`) `#[path]`-include this one file, so the case
//! manifest and the observation envelope have exactly ONE definition — the
//! whole design turns on the legs and the comparator not drifting from each
//! other, so they cannot each carry a private copy of the schema.
//!
//! This is a deliberate SUPERSET schema `#[path]`-included by two bins that each
//! read a different subset (the authority leg dispatches on `op`; the comparator
//! reads `invariants`/`equivalent_to`) — so per-bin dead-code analysis flags the
//! fields the other bin uses. The `dead_code` allow is scoped to this file only.
#![allow(dead_code)]

use std::collections::BTreeMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

/// The name the in-process Rust authority leg writes/reads. Its column is the
/// REFERENCE every surface leg is held to — not a peer (plan §1, §2).
pub const AUTHORITY: &str = "rust-leaf";

// --- the case manifest (cases/*.json) --------------------------------

#[derive(Deserialize, Debug, Clone)]
pub struct Manifest {
    pub schema: u32,
    pub cases: Vec<Case>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Case {
    pub id: String,
    /// The door the legs drive for this case (e.g. `reemit_canonical`). Each
    /// leg maps this to ONE public expression in its own language — never any
    /// adapter logic (plan §9.5).
    pub op: String,
    pub input: Input,
    /// The legs this case is compared across. A leg absent here is not expected
    /// to observe the case at all.
    pub legs: Vec<String>,
    #[serde(default)]
    pub invariants: Vec<String>,
    /// A sibling case whose authority output these bytes must equal (cross-path
    /// equivalence, e.g. `build_ags4` vs `from_excel`). Unused until the numeric
    /// cut; parsed now so the schema is stable.
    #[serde(default)]
    pub equivalent_to: Option<String>,
    #[serde(default)]
    pub spec_ref: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Input {
    /// A repo-root-relative path to an AGS4 fixture (resolved by every leg
    /// against the same `--repo-root`, default cwd).
    #[serde(default)]
    pub fixture: Option<String>,
    /// Inline typed rows — for the BUILD direction, where the input is a cell
    /// matrix (there is no legal AGS4 text spelling of a cell that contains a
    /// newline, drift #1b). Each group's `rows[0]` is the HEADING line
    /// (`["HEADING", …headings]`); the rest are the tagged UNIT/TYPE/DATA rows,
    /// verbatim — the exact shape the compat emitter and `write_ags4_matrix`
    /// consume.
    #[serde(default)]
    pub groups: Option<Vec<InlineGroup>>,
    /// Inline TYPED rows — for the `build_ags4` door (the data→AGS4 direction).
    /// This is the exact `GroupInput` / wasm `groups_json` shape: `headings`
    /// plus `rows` of JSON values (numbers/strings/bools/null), the dictionary
    /// filling UNIT/TYPE. Each surface constructs its own idiom from it (a
    /// polars frame, node row-objects, the wasm JSON string).
    #[serde(default)]
    pub build: Option<Vec<BuildGroup>>,
    /// Options for the `build_typed` op. Absent means every surface's defaults
    /// (`AutoFix`, fallback edition, synthesis OFF, no `TRAN`) — which is what all
    /// the original build cases exercise.
    ///
    /// A case that sets these is checking that the surfaces agree on the
    /// SYNTHESIS path too, not just the passthrough one. That path mints whole
    /// groups the caller never wrote, so a surface diverging there produces a
    /// file with different GROUPS, not merely different bytes.
    #[serde(default)]
    pub build_opts: Option<BuildOpts>,
}

/// The `build_typed` knobs, in the shared vocabulary every surface uses.
#[derive(Deserialize, Debug, Clone, Default)]
pub struct BuildOpts {
    #[serde(default)]
    pub synthesise_metadata: bool,
    /// All five or none — the shared `TranStamp` rule. A partial stamp is an
    /// error on every surface, so a case cannot express one here either.
    #[serde(default)]
    pub tran: Option<TranParts>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct TranParts {
    pub issue: String,
    pub date: String,
    pub producer: String,
    pub recipient: String,
    pub status: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct InlineGroup {
    pub code: String,
    pub rows: Vec<Vec<String>>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct BuildGroup {
    pub code: String,
    pub headings: Vec<String>,
    #[serde(default)]
    pub units: Option<Vec<String>>,
    #[serde(default)]
    pub types: Option<Vec<String>>,
    pub rows: Vec<Vec<laterite_ags4_emit::Cell>>,
}

/// Load and concatenate every `*.json` manifest in `cases_dir` (excluding the
/// `inputs/` subdir), sorted by filename for a deterministic case order.
pub fn load_manifests(cases_dir: &Path) -> Result<Vec<Case>, String> {
    let mut files: Vec<std::path::PathBuf> = std::fs::read_dir(cases_dir)
        .map_err(|e| format!("read {}: {e}", cases_dir.display()))?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("json"))
        .collect();
    files.sort();
    let mut cases = Vec::new();
    for f in files {
        let text = std::fs::read_to_string(&f).map_err(|e| format!("read {}: {e}", f.display()))?;
        let m: Manifest =
            serde_json::from_str(&text).map_err(|e| format!("parse {}: {e}", f.display()))?;
        cases.extend(m.cases);
    }
    Ok(cases)
}

// --- the observation envelope (one per leg) --------------------------

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LegObservations {
    pub schema: u32,
    pub leg: String,
    /// The identity of the engine THIS leg is actually running — the validator's
    /// build-time digest over every rule source, the dictionary and the rules
    /// catalogue (`laterite_ags4_validator::ENGINE_FINGERPRINT`).
    ///
    /// Without it this gate can only report that the legs agreed, not that they
    /// were comparable. That distinction is not academic: half these legs run
    /// against BUILT ARTEFACTS — `wasm-engine` against a `wasm-pack` output,
    /// `cli-npx` against a tsup dist, `python` against an installed wheel — any
    /// of which can be stale while every case still matches, because a stale
    /// engine and a current one usually agree. The run then reports N-way
    /// identity across a surface that was compiled two weeks ago.
    ///
    /// That is exactly the failure this repo has already paid for once, at a
    /// different level: `ags4-compliance` hard-coded a version literal and went on
    /// printing it for two minors, so its report claimed an identity it had not
    /// checked. A version number could not have caught it; this can, because it
    /// moves when a rule is edited even if nobody bumps anything.
    ///
    /// `Option` because the three `cli-*` launcher legs have no door to ask
    /// through yet — they drive a subprocess, and none of the three launchers
    /// prints its engine. The comparator NAMES the legs that did not report
    /// rather than passing over them, so this reads as a known gap instead of a
    /// clean bill of health.
    #[serde(default)]
    pub engine: Option<String>,
    pub cases: BTreeMap<String, Observation>,
}

/// One leg's observation of one case. Externally tagged, so it serialises as
/// `{"ok": <any json>}` / `{"err": "<sentinel>"}` / `{"absent": "<reason>"}` —
/// the exact three-variant envelope every leg (Rust, Python, JS) emits.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Observation {
    Ok(serde_json::Value),
    Err(String),
    Absent(String),
}

// --- the reparse invariant (`emit_reparses`) -------------------------

/// A group reduced to what the invariant compares: its code, headings, and
/// every descriptor/data row PADDED to heading width. Padding is deliberate —
/// the canonical emitter fills a ragged DATA row's tail with `""`, so a faithful
/// re-emit re-parses to the padded shape; comparing padded-to-padded means a
/// widened row is NOT a false split, while a TORN row (an embedded newline that
/// splits one record across two physical lines) changes the row COUNT and IS
/// caught.
#[derive(Debug, Clone, PartialEq)]
pub struct CanonGroup {
    pub code: String,
    pub headings: Vec<String>,
    pub units: Vec<String>,
    pub types: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

/// Parse AGS4 text into the padded canonical structure the `emit_reparses`
/// invariant compares. Uses the SAME shared parse leaf every surface wraps.
pub fn canonical(text: &str) -> Result<Vec<CanonGroup>, String> {
    let pf = laterite_ags4_parse::parse_str(text).map_err(|e| format!("{e:?}"))?;
    let mut out = Vec::with_capacity(pf.group_order.len());
    for code in &pf.group_order {
        let Some(g) = pf.groups.get(code) else {
            continue;
        };
        let n = g.headings.len();
        let pad = |src: &[String]| -> Vec<String> {
            (0..n)
                .map(|i| src.get(i).cloned().unwrap_or_default())
                .collect()
        };
        out.push(CanonGroup {
            code: code.clone(),
            headings: g.headings.clone(),
            units: pad(&g.units),
            types: pad(&g.types),
            rows: g
                .rows
                .iter()
                .map(|r| {
                    (0..n)
                        .map(|i| {
                            r.values
                                .get(i)
                                .map_or_else(String::new, |s| s.slice(g.text()).to_string())
                        })
                        .collect()
                })
                .collect(),
        });
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn observation_round_trips_the_three_variant_envelope() {
        let ok = Observation::Ok(serde_json::json!("\"GROUP\",\"PROJ\"\r\n"));
        assert_eq!(
            serde_json::to_value(&ok).unwrap(),
            serde_json::json!({"ok": "\"GROUP\",\"PROJ\"\r\n"})
        );
        let err = Observation::Err("EmbeddedNewline".into());
        assert_eq!(
            serde_json::to_value(&err).unwrap(),
            serde_json::json!({"err": "EmbeddedNewline"})
        );
        let absent = Observation::Absent("no filesystem in the wasm engine".into());
        assert_eq!(
            serde_json::to_value(&absent).unwrap(),
            serde_json::json!({"absent": "no filesystem in the wasm engine"})
        );
    }

    /// An envelope written before the `engine` field existed must still parse.
    ///
    /// The three `cli-*` legs emit exactly this shape today, so this is not a
    /// compatibility nicety — it is the live case. If it stopped parsing, every
    /// launcher leg would drop out of the run and the gate would report a
    /// comfortable pass over three fewer surfaces.
    #[test]
    fn an_envelope_without_an_engine_still_parses_as_unknown() {
        let old = serde_json::json!({
            "schema": 1,
            "leg": "cli-native",
            "cases": {"x": {"ok": "\"GROUP\",\"PROJ\"\r\n"}}
        });
        let parsed: LegObservations = serde_json::from_value(old).expect("parses");
        assert_eq!(
            parsed.engine, None,
            "a missing engine is unknown, not empty"
        );
        assert_eq!(parsed.cases.len(), 1, "the cases must survive intact");
    }

    /// An absent engine and an empty one are different facts, and only one of
    /// them can be compared. `Some("")` would match another `Some("")` and read
    /// as identity — so a leg that reports a blank digest must not be silently
    /// treated as having reported nothing, nor as agreeing.
    #[test]
    fn an_empty_engine_is_not_the_same_as_no_engine() {
        let blank: LegObservations = serde_json::from_value(
            serde_json::json!({"schema": 1, "leg": "l", "engine": "", "cases": {}}),
        )
        .expect("parses");
        assert_eq!(blank.engine.as_deref(), Some(""));
        assert_ne!(blank.engine, None);
    }

    #[test]
    fn canonical_pads_ragged_rows_so_widening_is_not_a_split() {
        let full = "\"GROUP\",\"PROJ\"\r\n\"HEADING\",\"A\",\"B\"\r\n\"UNIT\",\"\",\"\"\r\n\"TYPE\",\"X\",\"X\"\r\n\"DATA\",\"x\",\"\"\r\n";
        let ragged = "\"GROUP\",\"PROJ\"\r\n\"HEADING\",\"A\",\"B\"\r\n\"UNIT\",\"\",\"\"\r\n\"TYPE\",\"X\",\"X\"\r\n\"DATA\",\"x\"\r\n";
        assert_eq!(canonical(full).unwrap(), canonical(ragged).unwrap());
    }

    #[test]
    fn canonical_catches_a_torn_row_as_an_extra_record() {
        let one = "\"GROUP\",\"PROJ\"\r\n\"HEADING\",\"A\"\r\n\"UNIT\",\"\"\r\n\"TYPE\",\"X\"\r\n\"DATA\",\"a b\"\r\n";
        // The same file with the DATA value torn across two physical lines.
        let torn = "\"GROUP\",\"PROJ\"\r\n\"HEADING\",\"A\"\r\n\"UNIT\",\"\"\r\n\"TYPE\",\"X\"\r\n\"DATA\",\"a\r\nb\"\r\n";
        assert_ne!(canonical(one).unwrap(), canonical(torn).unwrap());
    }
}
