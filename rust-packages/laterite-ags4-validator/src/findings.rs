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

// serde's `skip_serializing_if` calls this as `fn(&Target) -> bool` — the
// macro-generated call site fixes the by-ref signature, so a by-value fix
// would not compile.
#[allow(clippy::trivially_copy_pass_by_ref)]
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
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Fyi => "fyi",
        }
    }
}

// Same serde-mandated by-ref signature as `is_default_target` above.
#[allow(clippy::trivially_copy_pass_by_ref)]
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

/// What a per-rule cap ([`crate::CheckOptions::max_findings_per_rule`]) cut from
/// one rule: the rule kept its first `shown` findings and found `total`.
///
/// The tier split of the findings it did not keep rides along, private, because
/// the verdict counts every finding a rule made, kept or not. A cap bounds what a
/// report holds, never what it concludes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Truncation {
    /// Findings kept for this rule, which is the cap.
    pub shown: usize,
    /// Findings this rule made, kept or not.
    pub total: usize,
    #[serde(skip)]
    dropped: [usize; 3],
}

impl Truncation {
    /// How many of the findings this rule did not keep were at `severity`.
    #[must_use]
    pub fn dropped(&self, severity: Severity) -> usize {
        self.dropped[tier_index(severity)]
    }
}

/// The rules a cap cut, keyed like [`Findings`]. Empty when nothing was cut,
/// which is always the case for an uncapped run.
pub type Truncated = BTreeMap<String, Truncation>;

fn tier_index(severity: Severity) -> usize {
    match severity {
        Severity::Error => 0,
        Severity::Warning => 1,
        Severity::Fyi => 2,
    }
}

/// Where a rule puts what it finds.
///
/// Every rule pushes through [`add`] / [`add_at`], and those push through this.
/// That makes it the one seam a cap can sit on: a [`Collector`] stops storing a
/// rule's findings at the cap and only counts the rest, so a file with one finding
/// per line holds the cap's worth, not the file's worth. Rules never see the cap.
///
/// A plain [`Findings`] map is a `Sink` too, keeping everything, so a caller that
/// drives a rule module directly passes one exactly as before.
pub trait Sink {
    /// Record `finding` under `rule`.
    fn push(&mut self, rule: &str, finding: Finding);
    /// Has `rule` found anything so far, kept or not? A rule that reads another
    /// rule's result (Rule 18 reads Rule 9) asks this, not the stored map: under a
    /// cap of `0` nothing is stored, and the answer must not change with the cap.
    fn fired(&self, rule: &str) -> bool;
}

impl Sink for Findings {
    fn push(&mut self, rule: &str, finding: Finding) {
        self.entry(rule.to_string()).or_default().push(finding);
    }

    fn fired(&self, rule: &str) -> bool {
        self.contains_key(rule)
    }
}

/// The engine's own [`Sink`]: a [`Findings`] map that keeps at most `cap`
/// findings per rule and counts the rest in a [`Truncated`].
#[derive(Debug, Default)]
pub struct Collector {
    findings: Findings,
    cap: Option<usize>,
    truncated: Truncated,
}

impl Collector {
    /// A collector keeping at most `cap` findings per rule; `None` keeps all.
    #[must_use]
    pub fn new(cap: Option<usize>) -> Self {
        Self {
            findings: Findings::new(),
            cap,
            truncated: Truncated::new(),
        }
    }

    /// What was kept, and what the cap cut.
    #[must_use]
    pub fn finish(self) -> Collected {
        Collected {
            findings: self.findings,
            truncated: self.truncated,
        }
    }

    /// Count a finding the cap did not keep. Out of line and cold because only a
    /// capped run past its cap reaches it, and `push` is inlined into every rule: kept
    /// inline, this map work was copied into each one and the browser engine's size
    /// gate (`tools/release/check-wasm-slim.mjs`) noticed.
    #[cold]
    #[inline(never)]
    fn cut(&mut self, rule: &str, kept: usize, severity: Severity) {
        let cut = self
            .truncated
            .entry(rule.to_string())
            .or_insert(Truncation {
                shown: kept,
                total: kept,
                dropped: [0; 3],
            });
        cut.total += 1;
        cut.dropped[tier_index(severity)] += 1;
    }
}

impl Sink for Collector {
    fn push(&mut self, rule: &str, finding: Finding) {
        if let Some(cap) = self.cap {
            let kept = self.findings.get(rule).map_or(0, Vec::len);
            if kept >= cap {
                return self.cut(rule, kept, finding.severity);
            }
        }
        self.findings.push(rule, finding);
    }

    fn fired(&self, rule: &str) -> bool {
        self.findings.contains_key(rule) || self.truncated.contains_key(rule)
    }
}

/// What a rule run produced: the findings kept, and what a per-rule cap cut.
///
/// `truncated` is empty for an uncapped run, so `findings` is then every finding.
/// Count with [`Collected::count`], never by summing `findings` — under a cap the
/// two differ, and only the first is the total.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Collected {
    /// The findings kept, at most the cap per rule.
    pub findings: Findings,
    /// The rules the cap cut, with what they kept and what they found.
    pub truncated: Truncated,
}

impl Collected {
    /// Every finding the run made, kept or not.
    #[must_use]
    pub fn count(&self) -> usize {
        count(&self.findings)
            + self
                .truncated
                .values()
                .map(|t| t.total - t.shown)
                .sum::<usize>()
    }
}

// --- the findings renderers -------------------------------------------
//
// ONE renderer per format, living next to the type it renders. The `lat`
// binary, laterite-py and laterite-node each used to carry their own copy,
// kept in sync only by "byte-identical to the binary's ndjson_string" /
// "ported verbatim from laterite-py's findings_ndjson" comments — three
// chances for `--json` to mean three different things (laterite-dev#530). They now all
// call these.
//
// Key ORDER is part of the output contract and depends on serde_json's
// `preserve_order` (declared in Cargo.toml — see the note there).

/// The nested report value `{file, findings:{rule:[{line,group,desc}]}}`.
/// Returned as a `Value` (not a string) because the CLI's `--json` stdout path
/// renders it rich/coloured; `findings_json` is the plain-string form.
///
/// A capped run that cut a rule adds `truncated: {rule: {shown, total}}` after
/// `findings` (#1089). The key is absent when nothing was cut, so an uncapped
/// report is byte-identical to one rendered before the cap existed.
#[must_use]
pub fn findings_json_value(
    file: &str,
    found: &Findings,
    truncated: &Truncated,
) -> serde_json::Value {
    use serde_json::{Map, Value};
    let mut fmap = Map::new();
    for (rule, items) in found {
        // Serialize the engine `Finding` directly. Unset location/severity
        // fields skip, so line-only findings stay `{line,group,desc}` and
        // migrated ones additively gain the rich keys.
        let arr: Vec<Value> = items
            .iter()
            .map(|f| serde_json::to_value(f).unwrap_or(Value::Null))
            .collect();
        fmap.insert(rule.clone(), Value::Array(arr));
    }
    let mut root = Map::new();
    root.insert("file".into(), Value::from(file.to_string()));
    root.insert("findings".into(), Value::Object(fmap));
    if !truncated.is_empty() {
        root.insert("truncated".into(), truncated_value(truncated));
    }
    Value::Object(root)
}

/// `{rule: {shown, total}}` — the one spelling of a cut, shared by both renderers.
fn truncated_value(truncated: &Truncated) -> serde_json::Value {
    serde_json::to_value(truncated).unwrap_or(serde_json::Value::Null)
}

/// Pretty JSON of [`findings_json_value`] — the `--json` file/report form.
#[must_use]
pub fn findings_json(file: &str, found: &Findings, truncated: &Truncated) -> String {
    serde_json::to_string_pretty(&findings_json_value(file, found, truncated)).unwrap_or_default()
}

/// One flat JSON object per finding per line (NDJSON). Stream/grep friendly,
/// never coloured; empty (no lines) when there are zero findings.
///
/// A capped run that cut a rule ends with one summary record,
/// `{"truncated":{rule:{shown,total}}}`, after the last finding (#1089). It has no
/// `rule` key, so a reader keyed on `rule` can tell it from a finding. There is no
/// such record when nothing was cut.
#[must_use]
pub fn findings_ndjson(found: &Findings, truncated: &Truncated) -> String {
    use serde_json::{Map, Value};
    let mut s = String::new();
    for (rule, items) in found {
        for f in items {
            // `rule`-first (the historical NDJSON key position), then splice in
            // the serialized `Finding` body so line-only findings stay
            // `{rule,line,group,desc}` byte-for-byte.
            let mut o = Map::new();
            o.insert("rule".into(), Value::from(rule.clone()));
            if let Value::Object(body) = serde_json::to_value(f).unwrap_or(Value::Null) {
                o.extend(body);
            }
            s.push_str(&serde_json::to_string(&Value::Object(o)).unwrap_or_default());
            s.push('\n');
        }
    }
    if !truncated.is_empty() {
        let mut o = Map::new();
        o.insert("truncated".into(), truncated_value(truncated));
        s.push_str(&serde_json::to_string(&Value::Object(o)).unwrap_or_default());
        s.push('\n');
    }
    s
}

/// Append a finding under `rule`, creating the bucket on first use.
/// Rule modules call this rather than poking the map directly so the
/// key convention stays in one place.
pub fn add(
    findings: &mut impl Sink,
    rule: &str,
    line: Option<u32>,
    group: &str,
    desc: impl Into<String>,
) {
    findings.push(
        rule,
        Finding {
            line,
            group: group.to_string(),
            desc: desc.into(),
            location: Location::default(),
            severity: Severity::Error,
        },
    );
}

/// Append a finding with an explicit [`Location`] and [`Severity`] —
/// the rich path that lets a rule point at a single heading / cell and
/// flag warning/fyi severity. The line-only `add` stays the fast path.
#[allow(clippy::too_many_arguments)]
pub fn add_at(
    findings: &mut impl Sink,
    rule: &str,
    line: Option<u32>,
    group: &str,
    desc: impl Into<String>,
    location: Location,
    severity: Severity,
) {
    findings.push(
        rule,
        Finding {
            line,
            group: group.to_string(),
            desc: desc.into(),
            location,
            severity,
        },
    );
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
#[must_use]
pub fn count_by_rule(findings: &Findings) -> Vec<(&str, usize)> {
    findings
        .iter()
        .map(|(k, v)| (k.as_str(), v.len()))
        .collect()
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

    /// Byte-identity guard: a line-only finding (via `add`) must still
    /// serialize as exactly `{"line":…,"group":…,"desc":…}` — no
    /// `target`/`severity`/etc. keys — so the historical JSON shape is
    /// preserved for every rule not yet migrated to `add_at`. `serde_json`
    /// is a dev-only dep here; the engine itself owns no `serde_json`.
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

    /// A cut rule is named in both renderers, after the findings, and nothing
    /// is added when nothing was cut: that absence is what keeps an uncapped
    /// report byte-identical to one rendered before the cap existed (#1089).
    #[test]
    fn renderers_name_a_cut_rule_and_add_nothing_when_none_was_cut() {
        let mut c = Collector::new(Some(1));
        add(&mut c, "AGS Format Rule 8", Some(5), "LOCA", "a");
        add(&mut c, "AGS Format Rule 8", Some(6), "LOCA", "b");
        add(&mut c, "AGS Format Rule 9", Some(2), "SAMP", "c");
        let Collected {
            findings,
            truncated,
        } = c.finish();

        let json = findings_json("d.ags", &findings, &truncated);
        let tail = json.split("\"truncated\"").nth(1).expect("a truncated key");
        assert!(
            tail.contains("\"AGS Format Rule 8\"") && !tail.contains("Rule 9"),
            "{json}"
        );
        assert!(
            json.find("\"findings\"") < json.find("\"truncated\""),
            "{json}"
        );
        let v = findings_json_value("d.ags", &findings, &truncated);
        assert_eq!(
            v["truncated"],
            serde_json::json!({"AGS Format Rule 8": {"shown": 1, "total": 2}})
        );

        let nd = findings_ndjson(&findings, &truncated);
        assert_eq!(
            nd.lines().last(),
            Some(r#"{"truncated":{"AGS Format Rule 8":{"shown":1,"total":2}}}"#)
        );
        assert_eq!(
            nd.lines().count(),
            3,
            "two kept findings, one summary: {nd}"
        );

        let none = Truncated::new();
        assert!(!findings_json("d.ags", &findings, &none).contains("truncated"));
        assert!(!findings_ndjson(&findings, &none).contains("truncated"));
    }

    /// The three renderers must actually emit the finding content — a canned
    /// empty/default return passes no assertion here.
    #[test]
    fn renderers_emit_the_finding_content() {
        let mut f = Findings::new();
        add(&mut f, "AGS Format Rule 8", Some(5), "LOCA", "boom");

        let v = findings_json_value("d.ags", &f, &Truncated::new());
        assert_eq!(v["file"], serde_json::json!("d.ags"));
        assert_eq!(
            v["findings"]["AGS Format Rule 8"][0]["desc"],
            serde_json::json!("boom")
        );

        let json = findings_json("d.ags", &f, &Truncated::new());
        assert!(json.contains("\"file\": \"d.ags\""), "{json}");
        assert!(
            json.contains("boom") && json.contains("AGS Format Rule 8"),
            "{json}"
        );

        let nd = findings_ndjson(&f, &Truncated::new());
        assert_eq!(
            nd.trim(),
            r#"{"rule":"AGS Format Rule 8","line":5,"group":"LOCA","desc":"boom"}"#
        );
    }
}
