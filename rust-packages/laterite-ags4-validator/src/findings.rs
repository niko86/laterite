//! Finding model — the validator's output shape.
//!
//! Conceptually parallels python-ags4's `{rule: [{line, group, desc}]}`
//! dict so cross-checking finding *counts* against it is meaningful, but
//! the types + wording here are our own (clean-room).

use std::collections::BTreeMap;

use serde::Serialize;

/// What a finding points *at* within its source line. The default,
/// [`Target::Line`], is the historical whole-line attribution; the
/// finer targets let the UI highlight a single heading / cell / group.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Target {
    /// The whole source line (historical default).
    #[default]
    Line,
    /// A single HEADING-row field (a column name).
    Heading,
    /// A single DATA-row cell.
    Cell,
    /// The GROUP as a whole (e.g. a GROUP-name field).
    Group,
}

fn is_default_target(t: &Target) -> bool {
    *t == Target::Line
}

/// Where within a line a finding lives, beyond the line number itself.
///
/// **Numbering convention.** `field_index` is the *tag-stripped* column
/// index — identical to the `ci` every rule already holds
/// (`headings[ci]` / `values[ci]`). The raw source line carries the
/// `DATA`/`HEADING` tag in field 0, so the raw on-line field is
/// `field_index + 1`. `heading` is the resolved column name for that
/// index; `data_row` is the **1-based row ordinal within the group**
/// (distinct from `line`, the physical source line). `char_span` is a
/// `(start, end)` char-offset pair within the raw line (a later phase;
/// unset here). Every field skips serialization when at its default so
/// line-only findings stay byte-identical to the historical shape.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize)]
pub struct Location {
    #[serde(skip_serializing_if = "is_default_target")]
    pub target: Target,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub field_index: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub heading: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_row: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub char_span: Option<(u32, u32)>,
}

/// How serious a finding is. `Error` is the historical implicit default
/// (every existing finding) and is skipped in serialization so line-only
/// error findings keep the original `{line, group, desc}` JSON.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    // `Error` is the historical implicit default for every finding.
    #[default]
    Error,
    Warning,
    Fyi,
}

impl Severity {
    /// The lowercase token, identical to the serde `rename_all = "snake_case"`
    /// wire form — the single PRODUCER of the severity value domain, so Node/wasm
    /// stop deriving it from `format!("{:?}").to_lowercase()` (a different code
    /// path that would silently diverge on any future multi-word variant). Gated
    /// against the serde rename in the tests below.
    pub fn as_str(self) -> &'static str {
        match self {
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Fyi => "fyi",
        }
    }
}

fn is_error_severity(s: &Severity) -> bool {
    *s == Severity::Error
}

/// One rule violation. `line` is `None` for whole-group / whole-file
/// findings that don't attach to a single source line.
///
/// Field order is load-bearing: serde emits in declaration order, and
/// `line, group, desc` first — with `location`/`severity` skipping when
/// at their defaults — keeps line-only error findings byte-identical to
/// the historical `{"line":…,"group":…,"desc":…}` JSON.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Finding {
    pub line: Option<u32>,
    pub group: String,
    pub desc: String,
    #[serde(flatten)]
    pub location: Location,
    #[serde(skip_serializing_if = "is_error_severity")]
    pub severity: Severity,
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
        location: Location::default(),
        severity: Severity::Error,
    });
}

/// Append a finding with an explicit [`Location`] and [`Severity`] —
/// the rich path that lets a rule point at a single heading / cell and
/// flag warning/fyi severity. The line-only `add` stays the fast path.
#[allow(clippy::too_many_arguments)]
pub fn add_at(
    findings: &mut Findings,
    rule: &str,
    line: Option<u32>,
    group: &str,
    desc: impl Into<String>,
    location: Location,
    severity: Severity,
) {
    findings.entry(rule.to_string()).or_default().push(Finding {
        line,
        group: group.to_string(),
        desc: desc.into(),
        location,
        severity,
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

    #[test]
    fn severity_as_str_matches_the_serde_token() {
        // `as_str` is the single producer; pin it to the serde wire form so the
        // two can never drift (a rename here without an as_str change fails).
        for s in [Severity::Error, Severity::Warning, Severity::Fyi] {
            let json = serde_json::to_value(s).expect("severity serializes");
            assert_eq!(json.as_str().expect("a JSON string"), s.as_str());
        }
    }

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

    /// Byte-identity guard: a line-only finding (via `add`) must still
    /// serialize as exactly `{"line":…,"group":…,"desc":…}` — no
    /// `target`/`severity`/etc. keys — so the historical JSON shape is
    /// preserved for every rule not yet migrated to `add_at`. serde_json
    /// is a dev-only dep here; the engine itself owns no serde_json.
    #[test]
    fn line_only_finding_serializes_minimally() {
        let mut f = Findings::new();
        add(&mut f, "AGS Format Rule 8", Some(5), "LOCA", "boom");
        let finding = &f["AGS Format Rule 8"][0];
        assert_eq!(
            serde_json::to_string(finding).unwrap(),
            r#"{"line":5,"group":"LOCA","desc":"boom"}"#
        );
    }

    /// The rich path emits the extra keys (and a non-`line` target /
    /// non-`error` severity) only when set away from default.
    #[test]
    fn add_at_serializes_location_and_severity() {
        let mut f = Findings::new();
        add_at(
            &mut f,
            "AGS Format Rule 9",
            Some(2),
            "SAMP",
            "nope",
            Location {
                target: Target::Heading,
                field_index: Some(3),
                heading: Some("SAMP_FOO".to_string()),
                ..Default::default()
            },
            Severity::Warning,
        );
        let finding = &f["AGS Format Rule 9"][0];
        assert_eq!(
            serde_json::to_string(finding).unwrap(),
            r#"{"line":2,"group":"SAMP","desc":"nope","target":"heading","field_index":3,"heading":"SAMP_FOO","severity":"warning"}"#
        );
    }
}
