//! The rule-catalogue **data accessors** — moved out of the validator (#475
//! PR2) so a consumer that only wants the inventory / editorial metadata
//! needn't depend on the whole rule engine. The faithfulness gate that
//! cross-checks these against the engine's actual emissions (fixable rules,
//! FYI/WARNING-bearing rules) stays in `laterite-ags4-validator::catalogue`
//! — it needs `crate::fixes::FIXABLE_RULE_LABELS`, which lives there.

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
/// `rules_meta.json` (see the validator's
/// `severity_only_mixed_for_fyi_or_warning_bearing_rules` gate).
pub const RULE_LABELS: &[&str] = &[
    "1", "2", "2a", "2b", "3", "4", "5", "6", "7", "8", "9", "10a", "10b", "10c", "11a", "11b",
    "11c", "13", "14", "15", "16", "17", "18", "19", "19a", "19b", "20",
];

/// The editorial rule metadata as the raw `rules_meta.json`, embedded at compile
/// time (no runtime parse for the `--json` passthrough). Gated by the
/// validator's `catalogue::tests`.
#[must_use]
pub fn rule_metadata_json() -> &'static str {
    include_str!("../data/rules_meta.json")
}
