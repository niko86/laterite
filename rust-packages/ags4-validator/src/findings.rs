//! Finding model — the validator's output shape.
//!
//! Conceptually parallels python-ags4's `{rule: [{line, group, desc}]}`
//! dict so cross-checking finding *counts* against it is meaningful, but
//! the types + wording here are our own (clean-room).

use std::collections::BTreeMap;

/// One rule violation. `line` is `None` for whole-group / whole-file
/// findings that don't attach to a single source line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    pub line: Option<u32>,
    pub group: String,
    pub desc: String,
}

/// Findings keyed by rule label, e.g. `"AGS Format Rule 8"`. `BTreeMap`
/// keeps reports deterministic / diffable across runs.
pub type Findings = BTreeMap<String, Vec<Finding>>;

/// Append a finding under `rule`, creating the bucket on first use.
/// Rule modules call this rather than poking the map directly so the
/// key convention stays in one place.
pub fn add(
    findings: &mut Findings,
    rule: &str,
    line: Option<u32>,
    group: &str,
    desc: impl Into<String>,
) {
    findings.entry(rule.to_string()).or_default().push(Finding {
        line,
        group: group.to_string(),
        desc: desc.into(),
    });
}

/// Total finding count across every rule — what `is_valid` and the CLI
/// exit code key off.
pub fn count(findings: &Findings) -> usize {
    findings.values().map(Vec::len).sum()
}

/// Per-rule counts in spec-rule (`BTreeMap`) order. The element keys are
/// the full rule labels as stored (`"AGS Format Rule 8"`); the values
/// sum to [`count`]. Additive convenience for summary/stats views (the
/// `--tui` browser); does not change any existing behaviour.
pub fn count_by_rule(findings: &Findings) -> Vec<(&str, usize)> {
    findings
        .iter()
        .map(|(k, v)| (k.as_str(), v.len()))
        .collect()
}

/// Per-group counts aggregated across every rule, group-name sorted.
/// Additive convenience for stats views; behaviour-neutral.
pub fn count_by_group(findings: &Findings) -> Vec<(String, usize)> {
    let mut m: BTreeMap<&str, usize> = BTreeMap::new();
    for v in findings.values() {
        for f in v {
            *m.entry(f.group.as_str()).or_default() += 1;
        }
    }
    m.into_iter().map(|(g, n)| (g.to_string(), n)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Findings {
        let mut f = Findings::new();
        add(&mut f, "AGS Format Rule 8", Some(5), "LOCA", "a");
        add(&mut f, "AGS Format Rule 8", Some(6), "LOCA", "b");
        add(&mut f, "AGS Format Rule 9", Some(2), "SAMP", "c");
        f
    }

    #[test]
    fn count_by_rule_sums_to_count() {
        let f = sample();
        let by_rule = count_by_rule(&f);
        assert_eq!(
            by_rule,
            vec![("AGS Format Rule 8", 2), ("AGS Format Rule 9", 1)]
        );
        assert_eq!(by_rule.iter().map(|(_, n)| n).sum::<usize>(), count(&f));
    }

    #[test]
    fn count_by_group_aggregates_across_rules() {
        let by_group = count_by_group(&sample());
        // LOCA: 2 (both Rule 8), SAMP: 1 (Rule 9); name-sorted.
        assert_eq!(
            by_group,
            vec![("LOCA".to_string(), 2), ("SAMP".to_string(), 1)]
        );
    }
}
