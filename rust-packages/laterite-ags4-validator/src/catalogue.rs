//! The rule catalogue — the single source for `lat rules` and
//! (via the follow-up that repoints it) the web `RuleExplainer`.
//!
//! Two facts combine here, reconciled by the gate tests so they cannot drift:
//!   1. the engine's **inventory** — [`RULE_LABELS`], the exact set of rule
//!      labels the engine can emit; and
//!   2. the **editorial metadata** — `rules_meta.json` (title / checks /
//!      severity / fixable / cited observations), embedded at compile time.
//!
//! The metadata is embedded with `include_str!`, so the `--list-rules --json`
//! passthrough pays **zero runtime parse**; only the human table parses it once
//! on that one-shot informational command. The faithfulness gate
//! ([`tests`]) asserts the metadata covers EXACTLY [`RULE_LABELS`] (so no
//! phantom rules like a no-op "12" or a non-existent "16a" can creep in) and
//! that `fixable` matches the fix engine ([`crate::fixes::FIXABLE_RULE_LABELS`]).
//!
//! [`RULE_LABELS`] and [`rule_metadata_json`] themselves moved to the
//! `laterite-ags4-reference` leaf (laterite-dev#475 PR2) — re-exported below — so a
//! consumer that only wants the inventory/metadata needn't depend on the
//! whole rule engine. This gate stays here: it needs
//! [`crate::fixes::FIXABLE_RULE_LABELS`], which the leaf can't see.
pub use laterite_ags4_reference::catalogue::{RULE_LABELS, rule_metadata_json};

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
        let want: BTreeSet<String> = RULE_LABELS
            .iter()
            .map(std::string::ToString::to_string)
            .collect();
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
        let engine: BTreeSet<String> = FIXABLE_RULE_LABELS
            .iter()
            .map(std::string::ToString::to_string)
            .collect();
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
            .map(std::string::ToString::to_string)
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
