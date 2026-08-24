//! The parity verdict model — **rule-label-set presence only**.
//!
//! Comparison is never line / group / desc / per-rule counts. The Rust
//! and python validators deliberately differ in wording and per-rule
//! attribution (OBSERVATIONS O-3 Rule 5↔4, O-26 19b triple-report, the
//! count-only O-11/O-16/O-22 families), so anything finer than "which
//! rules fired at all" produces false divergences. Known, documented
//! divergences are reconciled to their `O-N` id; only the unexplained
//! ones (+ validity disagreements) are the dogfood action list.
//!
//! Moved verbatim from `laterite-ags4-corpus-qa/src/parity.rs` (behaviour
//! byte-identical — the unit tests below moved with it and assert it).

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

/// What the Rust validator said, reduced to presence semantics.
#[derive(Debug, Clone, PartialEq)]
pub enum RustResult {
    Clean,
    Rules(BTreeSet<String>),
    HardError(String),
    Panic,
}

impl RustResult {
    /// Reduce a finished `laterite-ags4-validator` run to presence semantics.
    /// `include_fyi`/`include_warnings` are the caller's `CheckOptions`
    /// choice — the corpus-qa/forge convention is both ON so Rust is
    /// tier-comparable to python (python reports every tier).
    #[must_use]
    pub fn from_findings(found: &laterite_ags4_validator::Findings) -> Self {
        let s: BTreeSet<String> = laterite_ags4_validator::findings::count_by_rule(found)
            .into_iter()
            .map(|(r, _)| r.to_string())
            .collect();
        if s.is_empty() {
            RustResult::Clean
        } else {
            RustResult::Rules(s)
        }
    }

    /// Map a validator hard error to the stable `HardError(variant)`
    /// label the `classify` O-30/O-34 arms key off.
    #[must_use]
    pub fn from_validator_error(e: &laterite_ags4_validator::ValidatorError) -> Self {
        RustResult::HardError(
            match e {
                laterite_ags4_validator::ValidatorError::NotFound(_) => "NotFound",
                laterite_ags4_validator::ValidatorError::Io { .. } => "Io",
                laterite_ags4_validator::ValidatorError::NotAgs4(_) => "NotAgs4",
                laterite_ags4_validator::ValidatorError::BadDict { .. } => "BadDict",
                laterite_ags4_validator::ValidatorError::UnsupportedEdition { .. } => {
                    "UnsupportedEdition"
                }
                // Unreachable from the parity oracle, which always validates a real
                // file on disk (so the world is always answerable). Labelled anyway:
                // a silent `_ =>` would make the next hard error look like this one.
                laterite_ags4_validator::ValidatorError::WorldCheckRequiresSource => {
                    "WorldCheckRequiresSource"
                }
            }
            .to_string(),
        )
    }
}

/// Per-rule finding counts from a finished Rust run — the numbers
/// [`RustResult::from_findings`] deliberately throws away.
///
/// Same contract as [`crate::PyOracle::check_counts`]: for a reader, never
/// for [`classify`]. The two validators split one defect across rules
/// differently (O-11/O-16/O-22/O-26), so comparing counts manufactures
/// divergences the presence model exists to avoid — but a person reading
/// a report still wants to know that one side said this nine times and
/// the other said it once (#654).
#[must_use]
pub fn rust_rule_counts(found: &laterite_ags4_validator::Findings) -> BTreeMap<String, u64> {
    laterite_ags4_validator::findings::count_by_rule(found)
        .into_iter()
        .map(|(r, n)| (r.to_string(), u64::try_from(n).unwrap_or(u64::MAX)))
        .collect()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "verdict", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Parity {
    Agree,
    RustOnlyRules {
        rules: Vec<String>,
    },
    PythonOnlyRules {
        rules: Vec<String>,
    },
    /// Both sides carry rules the other does not (#652). Kept distinct from
    /// the one-sided verdicts because the two halves mean opposite things:
    /// a rust-only rule is usually a check we add on purpose, a python-only
    /// one is the shape of a false negative in our own engine. Collapsing
    /// this into either of them discards the half that matters most.
    RulesDiffer {
        rust_only: Vec<String>,
        python_only: Vec<String>,
    },
    ValidityDisagree {
        rust: String,
        python: String,
    },
    KnownDivergence {
        observation: String,
        detail: String,
    },
    PythonError {
        reason: String,
    },
}

impl Parity {
    #[must_use]
    pub fn tag(&self) -> &'static str {
        match self {
            Parity::Agree => "AGREE",
            Parity::RustOnlyRules { .. } => "RUST_ONLY_RULES",
            Parity::PythonOnlyRules { .. } => "PYTHON_ONLY_RULES",
            Parity::RulesDiffer { .. } => "RULES_DIFFER",
            Parity::ValidityDisagree { .. } => "VALIDITY_DISAGREE",
            Parity::KnownDivergence { .. } => "KNOWN_DIVERGENCE",
            Parity::PythonError { .. } => "PYTHON_ERROR",
        }
    }
    /// The dogfood action set: a real divergence to file as a fixture
    /// / bug (not AGREE, not a documented `KNOWN_DIVERGENCE`, not a
    /// python-side error).
    #[must_use]
    pub fn is_action(&self) -> bool {
        match self {
            Parity::RustOnlyRules { .. }
            | Parity::PythonOnlyRules { .. }
            | Parity::RulesDiffer { .. }
            | Parity::ValidityDisagree { .. } => true,
            Parity::Agree | Parity::KnownDivergence { .. } | Parity::PythonError { .. } => false,
        }
    }

    /// The same question asked of a **serialized** verdict, for the readers
    /// that have only the tag back from a report file. It lives here so the
    /// answer has one home: #652 was a verdict that stopped naming half of
    /// what it saw, and that fault copied into every reader's own tag list
    /// is a variant that quietly stops counting as an action.
    ///
    /// An allow-list, and it has to be one: the tag space is WIDER than this
    /// enum. `forge` mints `RUST_ONLY` for a candidate the oracle never saw,
    /// which is an absence of comparison, not a divergence — so "anything
    /// unrecognised is an action" would file every oracle-less run as a
    /// finding. The paired test is what keeps this list in step with the
    /// variants, since the compiler cannot.
    #[must_use]
    pub fn is_action_tag(tag: &str) -> bool {
        matches!(
            tag,
            "RUST_ONLY_RULES" | "PYTHON_ONLY_RULES" | "RULES_DIFFER" | "VALIDITY_DISAGREE"
        )
    }
}

fn py_desc(py: &BTreeSet<String>) -> String {
    if py.is_empty() {
        "valid (0 findings)".to_string()
    } else {
        format!("{} rule(s)", py.len())
    }
}

/// Reconcile a symmetric difference against the documented
/// Rust↔python divergences (OBSERVATIONS). Returns the matched `O-N`
/// id(s) iff the *entire* difference is explained. `py_all` is python's
/// full rule set (not just the py-only diff) — the cascade arms condition
/// on a rule the two validators *agree* on (e.g. the Rule 7 that triggers
/// python's duplicate-heading rename before it cascades to Rule 9).
fn reconcile(
    rust_only: &BTreeSet<String>,
    py_only: &BTreeSet<String>,
    py_all: &BTreeSet<String>,
) -> Option<String> {
    let mut ro = rust_only.clone();
    let mut po = py_only.clone();
    let mut ids: Vec<&str> = Vec::new();

    // O-2: python-ags4's rule_6 is a no-op; the Rust validator does
    // implement the embedded-CR check → Rust may uniquely fire Rule 6.
    if ro.remove("AGS Format Rule 6") {
        ids.push("O-2");
    }
    // O-3: an unquoted DATA field — Rust attributes it to Rule 5,
    // python to Rule 4.
    if ro.contains("AGS Format Rule 5") && po.contains("AGS Format Rule 4") {
        ro.remove("AGS Format Rule 5");
        po.remove("AGS Format Rule 4");
        ids.push("O-3");
    }
    // O-6 / O-7: laterite enforces the *de-facto* Rule 19 (a GROUP name is
    // exactly 4 uppercase LETTERS) and Rule 19b (a 4-letter prefix + a 1–4
    // char field) where python-ags4's looser `isupper()`/`len==4` checks
    // pass — so laterite uniquely fires Rule 19 (O-6) on a digit-bearing
    // group and/or Rule 19b (O-7) on a lowercase/over-long prefix. (The
    // *inverse* — python's redundant extra 19b — is O-26, below.)
    if ro.remove("AGS Format Rule 19") {
        ids.push("O-6");
    }
    if ro.remove("AGS Format Rule 19b") {
        ids.push("O-7");
    }
    // O-52: laterite REPORTS the Rule 10c parentage check it declined (a
    // child row whose parent-KEY cells are all empty). python-ags4 has no
    // equivalent and cannot have one — its rule_10c never knows it declined
    // anything — so this label is rust-only by construction on every file
    // with a standalone row.
    //
    // Live for FORGE, which validates with `include_warnings: true` so the
    // Rust side is tier-comparable to python. corpus-qa's crawl→validate→
    // parity run does NOT reach it: its validate stage is errors-only
    // (`show_warnings: false`), so the label never enters `classify` there.
    // The arm is for the pipeline that can see it; without it every forge
    // run over a corpus with standalone rows fills its ACTION list with a
    // divergence we wrote on purpose.
    if ro.remove("Warning (Related to Rule 10c)") {
        ids.push("O-52");
    }
    // O-26: python triple-reports Rule 19b for a malformed heading the
    // Rust validator reports once → python uniquely has extra 19b.
    if po.remove("AGS Format Rule 19b") {
        ids.push("O-26");
    }
    // O-27: Rule 20's on-disk half is opt-in (`check_files`). A harness that
    // runs with it OFF — the cross-surface compliance matrix, since the
    // duckdb surface has no filesystem stat — sees python's always-on
    // check fire Rule 20 where Rust stays silent. (corpus-qa runs
    // check_files ON, so Rust fires Rule 20 too and it never lands in the
    // py-only diff — this arm is inert there. See O-27 in OBSERVATIONS.)
    if po.remove("AGS Format Rule 20") {
        ids.push("O-27");
    }
    // O-35 (BOM cascade): python's parse layer turns a leading byte-order
    // mark into a multi-rule cascade — the BOM bytes make line 1 "not a
    // valid data descriptor" (Rule 3) and break the enclosure check (Rule
    // 5) — where laterite strips the BOM and reports Rule 1 only.
    // Signature-narrow (BOTH 3 and 5 python-only): O-35 sanctions narrow
    // arms, never generic widening.
    if po.contains("AGS Format Rule 3") && po.contains("AGS Format Rule 5") {
        po.remove("AGS Format Rule 3");
        po.remove("AGS Format Rule 5");
        ids.push("O-35");
    }
    // O-35 (duplicate-heading rename cascade): python's default
    // `rename_duplicate_headers` renames a repeated HEADING — the Rule 7
    // both validators agree on — to `<NAME>_N`, which then isn't in the
    // dictionary → a python-only Rule 9. laterite flags the duplicate
    // (Rule 7) without renaming, so no cascade. Gated on the agreed Rule 7.
    if py_all.contains("AGS Format Rule 7")
        && po.remove("AGS Format Rule 9")
        && !ids.contains(&"O-35")
    {
        ids.push("O-35");
    }

    if ro.is_empty() && po.is_empty() && !ids.is_empty() {
        Some(ids.join("+"))
    } else {
        None
    }
}

/// The classifier. Pure over presence sets — unit-tested.
#[must_use]
pub fn classify(rust: &RustResult, py: &Result<BTreeSet<String>, String>) -> Parity {
    // No dedicated O-8 (python `rule_7_2` IndexError) arm: a probe
    // (`probe-o8-dup-heading.ags`) refuted it. python-ags4's *default*
    // `rename_duplicate_headers=True` renames a dup HEADING to
    // `<NAME>_1` before `rule_7_2`, so the subset test fails and the
    // unguarded `temp[i]` is never reached — O-8's crash is
    // effectively unreachable via a HEADING-row duplicate under
    // default 1.2.0 behaviour (what this wrapper uses). The generic
    // PythonError short-circuit below is therefore adequate for the
    // rare genuine crash; a speculative O-8 arm would be over-claiming.
    // See ags-wiki/.bootstrap/probes/RESULTS.md.
    let py = match py {
        Ok(s) => s,
        Err(r) => return Parity::PythonError { reason: r.clone() },
    };
    match rust {
        RustResult::HardError(v) => {
            // O-30: an unsupported edition (AGS3 — Rust deliberately
            // refuses) where python silently validates it as AGS4
            // (typically Rule 3) is an *expected* divergence, not an
            // action item. Keep it out of the triage/ACTION list.
            if v == "UnsupportedEdition" {
                return Parity::KnownDivergence {
                    observation: "O-30".to_string(),
                    detail: format!(
                        "Rust refuses unsupported edition (AGS3); python validated it ({})",
                        py_desc(py)
                    ),
                };
            }
            // O-34: a tab-delimited/empty file (no spec-valid quoted
            // GROUP rows) → Rust `NotAgs4`. python has no refuse path,
            // so it mislabels it as missing every mandatory group.
            // When python *independently* agrees there's no AGS4
            // structure — PROJ *and* TRAN *and* TYPE all absent
            // (Rule 13 & 14 & 17) — that's the expected O-30-shaped
            // divergence, not an action. The triple guard keeps it
            // narrow: a NotAgs4 where python saw *some* structure
            // still falls through to ValidityDisagree.
            if v == "NotAgs4"
                && py.contains("AGS Format Rule 13")
                && py.contains("AGS Format Rule 14")
                && py.contains("AGS Format Rule 17")
            {
                return Parity::KnownDivergence {
                    observation: "O-34".to_string(),
                    detail: format!(
                        "Rust refuses non-AGS4-CSV (NotAgs4); python reports \
                         missing mandatory groups ({})",
                        py_desc(py)
                    ),
                };
            }
            return Parity::ValidityDisagree {
                rust: format!("hard error: {v}"),
                python: py_desc(py),
            };
        }
        RustResult::Panic => {
            return Parity::ValidityDisagree {
                rust: "panic".to_string(),
                python: py_desc(py),
            };
        }
        _ => {}
    }
    let rust_set: BTreeSet<String> = match rust {
        RustResult::Clean => BTreeSet::new(),
        RustResult::Rules(s) => s.clone(),
        _ => unreachable!(),
    };
    if &rust_set == py {
        return Parity::Agree;
    }
    let rust_only: BTreeSet<String> = rust_set.difference(py).cloned().collect();
    let py_only: BTreeSet<String> = py.difference(&rust_set).cloned().collect();
    if let Some(obs) = reconcile(&rust_only, &py_only, py) {
        return Parity::KnownDivergence {
            observation: obs,
            detail: format!("rust_only={rust_only:?} python_only={py_only:?}"),
        };
    }
    // One-sided first, then the both-sided residue. The middle arm is the
    // one #652 was missing: `rust_only` non-empty used to answer for the
    // whole difference, and every python-only rule beside it went unsaid.
    // Neither side empty is the only case left — the sets differ, so at
    // least one rule is unmatched somewhere.
    if rust_only.is_empty() {
        Parity::PythonOnlyRules {
            rules: py_only.into_iter().collect(),
        }
    } else if py_only.is_empty() {
        Parity::RustOnlyRules {
            rules: rust_only.into_iter().collect(),
        }
    } else {
        Parity::RulesDiffer {
            rust_only: rust_only.into_iter().collect(),
            python_only: py_only.into_iter().collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn set(xs: &[&str]) -> BTreeSet<String> {
        xs.iter().map(std::string::ToString::to_string).collect()
    }
    fn rules(xs: &[&str]) -> RustResult {
        RustResult::Rules(set(xs))
    }

    #[test]
    fn agree_when_rule_sets_match() {
        let r = rules(&["AGS Format Rule 8"]);
        let p = Ok(set(&["AGS Format Rule 8"]));
        assert_eq!(classify(&r, &p), Parity::Agree);
        // Both clean.
        assert_eq!(
            classify(&RustResult::Clean, &Ok(BTreeSet::new())),
            Parity::Agree
        );
    }

    #[test]
    fn o3_rule5_vs_rule4_is_known_divergence() {
        // Rust attributes the unquoted DATA field to Rule 5, python to
        // Rule 4 (OBSERVATIONS O-3). Must NOT be RUST_ONLY/PYTHON_ONLY.
        let r = rules(&["AGS Format Rule 5"]);
        let p = Ok(set(&["AGS Format Rule 4"]));
        match classify(&r, &p) {
            Parity::KnownDivergence { observation, .. } => assert_eq!(observation, "O-3"),
            other => panic!("expected KnownDivergence O-3, got {other:?}"),
        }
    }

    #[test]
    fn o26_python_extra_19b_is_known_divergence() {
        // python triple-reports 19b; Rust reports the defect once (it
        // still has Rule 9 in common).
        let r = rules(&["AGS Format Rule 9"]);
        let p = Ok(set(&["AGS Format Rule 9", "AGS Format Rule 19b"]));
        match classify(&r, &p) {
            Parity::KnownDivergence { observation, .. } => assert_eq!(observation, "O-26"),
            other => panic!("expected KnownDivergence O-26, got {other:?}"),
        }
    }

    #[test]
    fn o6_o7_rust_only_19_and_19b_is_known_divergence() {
        // laterite's de-facto Rule 19/19b fire on a digit-bearing group
        // (`TES1`) where python's isupper()/len==4 passes → Rust-only 19+19b
        // over the shared Rule 10c. Reconciles as O-6+O-7, not an action.
        let r = rules(&[
            "AGS Format Rule 10c",
            "AGS Format Rule 19",
            "AGS Format Rule 19b",
        ]);
        let p = Ok(set(&["AGS Format Rule 10c"]));
        match classify(&r, &p) {
            Parity::KnownDivergence { observation, .. } => assert_eq!(observation, "O-6+O-7"),
            other => panic!("expected KnownDivergence O-6+O-7, got {other:?}"),
        }
        assert!(!classify(&r, &p).is_action());
    }

    #[test]
    fn o52_rust_only_declined_parentage_warning_is_known_divergence() {
        // The warning laterite adds (#656) is rust-only on every file with a
        // standalone row, and there are real corpora full of them. It must
        // reconcile, or the dogfood ACTION list fills with a divergence we
        // wrote on purpose.
        let r = rules(&["Warning (Related to Rule 10c)"]);
        let p = Ok(BTreeSet::new());
        match classify(&r, &p) {
            Parity::KnownDivergence { observation, .. } => assert_eq!(observation, "O-52"),
            other => panic!("expected KnownDivergence O-52, got {other:?}"),
        }
        assert!(!classify(&r, &p).is_action());
        // Negative guard: it reconciles ITSELF, never a real Rule 10c
        // difference standing beside it.
        let both = rules(&["Warning (Related to Rule 10c)", "AGS Format Rule 10c"]);
        assert!(matches!(
            classify(&both, &Ok(BTreeSet::new())),
            Parity::RustOnlyRules { .. }
        ));
    }

    #[test]
    fn o27_python_only_rule20_is_known_divergence() {
        // check_files OFF (the compliance harness) → python's on-disk Rule
        // 20 fires where Rust stays silent. Reconciles as O-27.
        let r = RustResult::Clean;
        let p = Ok(set(&["AGS Format Rule 20"]));
        match classify(&r, &p) {
            Parity::KnownDivergence { observation, .. } => assert_eq!(observation, "O-27"),
            other => panic!("expected KnownDivergence O-27, got {other:?}"),
        }
    }

    #[test]
    fn o35_bom_cascade_is_known_divergence() {
        // A leading BOM: both fire Rule 1; python's parse layer cascades to
        // Rule 3 + Rule 5 where laterite strips the BOM. Signature-narrow
        // (both 3 and 5 python-only) → O-35.
        let r = rules(&["AGS Format Rule 1"]);
        let p = Ok(set(&[
            "AGS Format Rule 1",
            "AGS Format Rule 3",
            "AGS Format Rule 5",
        ]));
        match classify(&r, &p) {
            Parity::KnownDivergence { observation, .. } => assert_eq!(observation, "O-35"),
            other => panic!("expected KnownDivergence O-35, got {other:?}"),
        }
    }

    #[test]
    fn o35_rename_cascade_is_known_divergence() {
        // Duplicate headings: both fire Rule 7; python renames the dup, whose
        // `<NAME>_1` isn't in the dict → python-only Rule 9. Gated on the
        // agreed Rule 7 → O-35.
        let r = rules(&["AGS Format Rule 7"]);
        let p = Ok(set(&["AGS Format Rule 7", "AGS Format Rule 9"]));
        match classify(&r, &p) {
            Parity::KnownDivergence { observation, .. } => assert_eq!(observation, "O-35"),
            other => panic!("expected KnownDivergence O-35, got {other:?}"),
        }
    }

    #[test]
    fn python_only_rule9_without_agreed_rule7_stays_an_action() {
        // Negative guard: the rename arm is gated on the agreed Rule 7. A
        // bare python-only Rule 9 (no shared Rule 7) is NOT the rename
        // cascade and must remain a real action.
        let r = RustResult::Clean;
        let p = Ok(set(&["AGS Format Rule 9"]));
        assert!(matches!(classify(&r, &p), Parity::PythonOnlyRules { .. }));
    }

    #[test]
    fn o2_rust_only_rule6_is_known_divergence() {
        // python's rule_6 is a no-op; Rust uniquely fires Rule 6.
        let r = rules(&["AGS Format Rule 6"]);
        let p = Ok(BTreeSet::new());
        match classify(&r, &p) {
            Parity::KnownDivergence { observation, .. } => assert_eq!(observation, "O-2"),
            other => panic!("expected KnownDivergence O-2, got {other:?}"),
        }
    }

    #[test]
    fn hard_error_vs_findings_is_validity_disagree() {
        let r = RustResult::HardError("NotAgs4".into());
        let p = Ok(set(&["AGS Format Rule 8"]));
        assert!(matches!(classify(&r, &p), Parity::ValidityDisagree { .. }));
        // Panic vs python-clean is also a validity disagreement.
        assert!(matches!(
            classify(&RustResult::Panic, &Ok(BTreeSet::new())),
            Parity::ValidityDisagree { .. }
        ));
    }

    #[test]
    fn ags3_unsupported_edition_is_known_divergence_not_action() {
        // O-30: Rust refuses AGS3 (UnsupportedEdition); python
        // mis-validates it as AGS4 → Rule 3. Expected, not an action.
        let r = RustResult::HardError("UnsupportedEdition".into());
        let p = Ok(set(&["AGS Format Rule 3"]));
        match classify(&r, &p) {
            Parity::KnownDivergence { observation, .. } => {
                assert_eq!(observation, "O-30");
            }
            other => panic!("expected KnownDivergence O-30, got {other:?}"),
        }
        assert!(
            !classify(&r, &p).is_action(),
            "AGS3 must leave the ACTION list"
        );
        // A genuine read failure (NotUtf8) is still a real disagreement.
        let nu = RustResult::HardError("NotUtf8".into());
        assert!(matches!(
            classify(&nu, &Ok(set(&["AGS Format Rule 1"]))),
            Parity::ValidityDisagree { .. }
        ));
    }

    #[test]
    fn o34_notags4_vs_missing_groups_is_known_divergence() {
        // O-34: Rust refuses a tab-delimited/empty file (NotAgs4);
        // python, lacking a refuse path, reports every mandatory group
        // missing. python independently agreeing there's no AGS4
        // structure (Rule 13 & 14 & 17) → expected divergence.
        let r = RustResult::HardError("NotAgs4".into());
        let p = Ok(set(&[
            "AGS Format Rule 13",
            "AGS Format Rule 14",
            "AGS Format Rule 15",
            "AGS Format Rule 17",
        ]));
        match classify(&r, &p) {
            Parity::KnownDivergence { observation, .. } => {
                assert_eq!(observation, "O-34");
            }
            other => panic!("expected KnownDivergence O-34, got {other:?}"),
        }
        assert!(
            !classify(&r, &p).is_action(),
            "O-34 must leave the ACTION list"
        );
        // Negative guard: NotAgs4 where python saw *some* structure
        // (not all mandatory groups absent) stays a real disagreement.
        let partial = Ok(set(&["AGS Format Rule 3"]));
        assert!(matches!(
            classify(&r, &partial),
            Parity::ValidityDisagree { .. }
        ));
    }

    #[test]
    fn unexplained_difference_is_an_action() {
        let r = rules(&["AGS Format Rule 99"]);
        let p = Ok(BTreeSet::new());
        assert!(matches!(classify(&r, &p), Parity::RustOnlyRules { .. }));
        let p2 = Ok(set(&["AGS Format Rule 7"]));
        assert!(matches!(
            classify(&RustResult::Clean, &p2),
            Parity::PythonOnlyRules { .. }
        ));
    }

    #[test]
    fn both_directions_are_reported_together() {
        // The landing demo's shape (#652): laterite emits an FYI python has
        // no equivalent for, python emits the Rule 10c of O-39. Neither side
        // may be dropped for the other's sake — a python-only rule is the
        // shape of a false negative in OUR engine, so it is the one that
        // must never go missing.
        let r = rules(&[
            "AGS Format Rule 16",
            "AGS Format Rule 8",
            "FYI (Related to Rule 16)",
        ]);
        let p = Ok(set(&[
            "AGS Format Rule 10c",
            "AGS Format Rule 16",
            "AGS Format Rule 8",
        ]));
        match classify(&r, &p) {
            Parity::RulesDiffer {
                rust_only,
                python_only,
            } => {
                assert_eq!(rust_only, vec!["FYI (Related to Rule 16)".to_string()]);
                assert_eq!(python_only, vec!["AGS Format Rule 10c".to_string()]);
            }
            other => panic!("expected RulesDiffer, got {other:?}"),
        }
        assert!(classify(&r, &p).is_action());
    }

    #[test]
    fn every_verdict_answers_the_same_whether_read_live_or_from_a_tag() {
        // The readers downstream have only the serialized tag, so the two
        // answers must never part. The exhaustive match is the point: a new
        // variant fails to compile here until it is listed, which is what
        // stops it from being born already invisible to the action list.
        // Compiler-checked completeness of the list below: a new variant
        // stops this compiling until someone gives it a value to test.
        fn ordinal(p: &Parity) -> usize {
            match p {
                Parity::Agree => 0,
                Parity::RustOnlyRules { .. } => 1,
                Parity::PythonOnlyRules { .. } => 2,
                Parity::RulesDiffer { .. } => 3,
                Parity::ValidityDisagree { .. } => 4,
                Parity::KnownDivergence { .. } => 5,
                Parity::PythonError { .. } => 6,
            }
        }
        let all = [
            Parity::Agree,
            Parity::RustOnlyRules { rules: vec![] },
            Parity::PythonOnlyRules { rules: vec![] },
            Parity::RulesDiffer {
                rust_only: vec![],
                python_only: vec![],
            },
            Parity::ValidityDisagree {
                rust: String::new(),
                python: String::new(),
            },
            Parity::KnownDivergence {
                observation: String::new(),
                detail: String::new(),
            },
            Parity::PythonError {
                reason: String::new(),
            },
        ];
        let seen: BTreeSet<usize> = all.iter().map(ordinal).collect();
        assert_eq!(seen.len(), all.len(), "one value per variant, no repeats");
        for p in &all {
            assert_eq!(
                Parity::is_action_tag(p.tag()),
                p.is_action(),
                "{} disagrees with its own tag",
                p.tag()
            );
        }
        // `forge` mints this one where no verdict exists at all — the oracle
        // did not run. No comparison happened, so there is nothing to file.
        assert!(!Parity::is_action_tag("RUST_ONLY"));
    }

    #[test]
    fn python_error_short_circuits() {
        let r = rules(&["AGS Format Rule 8"]);
        let e: Result<BTreeSet<String>, String> = Err("timeout".into());
        assert_eq!(
            classify(&r, &e),
            Parity::PythonError {
                reason: "timeout".into()
            }
        );
    }
}
