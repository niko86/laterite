//! The rule catalogue — the single source for `lat-check --list-rules` and
//! (via the follow-up that repoints it) the web RuleExplainer.
//!
//! Two facts combine here, reconciled by the gate tests so they cannot drift:
//!   1. the engine's **inventory** — [`RULE_LABELS`], the exact set of rule
//!      labels the engine can emit; and
//!   2. the **editorial metadata** — `data/rules_meta.json` (title / checks /
//!      severity / fixable / cited observations), embedded at compile time.
//!
//! The metadata is embedded with `include_str!`, so the `--list-rules --json`
//! passthrough pays **zero runtime parse**; only the human table parses it once
//! on that one-shot informational command. The faithfulness gate
//! ([`tests`]) asserts the metadata covers EXACTLY [`RULE_LABELS`] (so no
//! phantom rules like a no-op "12" or a non-existent "16a" can creep in) and
//! that `fixable` matches the fix engine ([`crate::fixes::FIXABLE_RULE_LABELS`]).

/// The exact set of **numbered** AGS4 rule labels the engine can emit — the 27
/// rules laterite implements (Rule 12 is a deliberate no-op, subsumed by Rule
/// 10b, and `16a` is folded into Rule 16, so neither appears). This is the
/// catalogue inventory authority: `rules_meta.json` is gated to cover exactly
/// this set, and `regression.rs` asserts the engine emits no numbered label
/// outside it.
///
/// NOT included here (by design): the **FYI / WARNING buckets** the engine can
/// also emit — the top-level `"FYI"` (an unrecognised `TRAN_AGS` / a 4.0.3→4.0.4
/// edition upgrade), the per-rule `"FYI (Related to Rule 1)"` / `"FYI (Related to
/// Rule 16)"`, and the per-rule `"Warning (Related to Rule 18)"` (malformed
/// DICT). Those are a separate, non-numbered label space surfaced only with
/// `include_fyi` / `include_warnings`; they have no catalogue row, and a rule
/// whose *number* can yield such an FYI/Warning is marked `severity: "mixed"` in
/// `rules_meta.json` (see the `severity_only_mixed_for_fyi_or_warning_bearing_rules`
/// gate).
pub const RULE_LABELS: &[&str] = &[
    "1", "2", "2a", "2b", "3", "4", "5", "6", "7", "8", "9", "10a", "10b", "10c", "11a", "11b",
    "11c", "13", "14", "15", "16", "17", "18", "19", "19a", "19b", "20",
];

/// The numbered rules that can also emit a related FYI (`"FYI (Related to Rule
/// N)"`). Tied to the FYI emitters in `rules/line_format.rs` (Rule 1 — BOM /
/// extended-ASCII) and `rules/groups.rs` (Rule 16 — description drift /
/// non-standard abbreviation).
#[cfg(test)]
const FYI_BEARING_RULES: &[&str] = &["1", "16"];

/// The numbered rules that can also emit a related WARNING (`"Warning (Related
/// to Rule N)"`). Tied to `rules/groups.rs::rule_18_structure` (Rule 18 —
/// malformed DICT, the first WARNING-tier producer).
#[cfg(test)]
const WARN_BEARING_RULES: &[&str] = &["18"];

/// The editorial rule metadata as the raw `rules_meta.json`, embedded at compile
/// time (no runtime parse for the `--json` passthrough). Gated by [`tests`].
pub fn rule_metadata_json() -> &'static str {
    include_str!("../data/rules_meta.json")
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use serde_json::Value;

    use super::*;
    use crate::fixes::FIXABLE_RULE_LABELS;

    fn meta_rules() -> Vec<Value> {
        let doc: Value =
            serde_json::from_str(rule_metadata_json()).expect("rules_meta.json must parse");
        doc["rules"].as_array().expect("rules array").clone()
    }

    #[test]
    fn rule_labels_is_the_27_and_distinct() {
        assert_eq!(RULE_LABELS.len(), 27);
        let set: BTreeSet<_> = RULE_LABELS.iter().collect();
        assert_eq!(set.len(), 27, "RULE_LABELS has a duplicate");
        // The two phantoms must never be here.
        assert!(!RULE_LABELS.contains(&"12"));
        assert!(!RULE_LABELS.contains(&"16a"));
    }

    #[test]
    fn metadata_covers_exactly_the_inventory() {
        let meta: BTreeSet<String> = meta_rules()
            .iter()
            .map(|r| r["rule"].as_str().expect("rule string").to_string())
            .collect();
        let want: BTreeSet<String> = RULE_LABELS.iter().map(|s| s.to_string()).collect();
        assert_eq!(
            meta, want,
            "rules_meta.json must cover EXACTLY the engine inventory (no phantom / missing rules)"
        );
    }

    #[test]
    fn fixable_flags_match_the_fix_engine() {
        let meta_fixable: BTreeSet<String> = meta_rules()
            .iter()
            .filter(|r| r["fixable"].as_bool().unwrap_or(false))
            .map(|r| r["rule"].as_str().unwrap().to_string())
            .collect();
        let engine: BTreeSet<String> = FIXABLE_RULE_LABELS.iter().map(|s| s.to_string()).collect();
        assert_eq!(
            meta_fixable, engine,
            "rules_meta.json `fixable` must match the rules the fix engine actually repairs"
        );
    }

    #[test]
    fn severities_are_from_the_known_vocabulary() {
        for r in meta_rules() {
            let sev = r["severity"].as_str().expect("severity string");
            assert!(
                matches!(sev, "error" | "fyi" | "mixed"),
                "rule {:?} has unknown severity {sev:?}",
                r["rule"]
            );
        }
    }

    #[test]
    fn severity_only_mixed_for_fyi_or_warning_bearing_rules() {
        // `severity` is editorial (the engine emits each NUMBERED label as Error;
        // the related FYIs / WARNINGs ride separate `FYI (Related to Rule N)` /
        // `Warning (Related to Rule N)` labels). So ground the one drift-prone
        // field: a rule is `"mixed"` iff its number can ALSO yield a related FYI
        // ([`FYI_BEARING_RULES`]) or WARNING ([`WARN_BEARING_RULES`]); everything
        // else is `"error"`. (No rule is `"fyi"`-only today — every numbered label
        // is also an error; this asserts that too.)
        let non_error: BTreeSet<String> = meta_rules()
            .iter()
            .filter(|r| r["severity"].as_str() != Some("error"))
            .map(|r| r["rule"].as_str().unwrap().to_string())
            .collect();
        let want: BTreeSet<String> = FYI_BEARING_RULES
            .iter()
            .chain(WARN_BEARING_RULES.iter())
            .map(|s| s.to_string())
            .collect();
        assert_eq!(
            non_error, want,
            "only FYI/Warning-bearing rules may be non-`error`; severity drifted from the engine"
        );
        for r in meta_rules() {
            if r["severity"].as_str() != Some("error") {
                assert_eq!(r["severity"].as_str(), Some("mixed"), "{:?}", r["rule"]);
            }
        }
    }
}
