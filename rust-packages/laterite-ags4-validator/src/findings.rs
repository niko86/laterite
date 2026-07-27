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

// --- the findings renderers -------------------------------------------
//
// ONE renderer per format, living next to the type it renders. The `lat`
// binary, laterite-py and laterite-node each used to carry their own copy,
// kept in sync only by "byte-identical to the binary's ndjson_string" /
// "ported verbatim from laterite-py's findings_ndjson" comments — three
// chances for `--json` to mean three different things (#530). They now all
// call these.
//
// Key ORDER is part of the output contract and depends on serde_json's
// `preserve_order` (declared in Cargo.toml — see the note there).

/// The nested report value `{file, findings:{rule:[{line,group,desc}]}}`.
/// Returned as a `Value` (not a string) because the CLI's `--json` stdout path
/// renders it rich/coloured; `findings_json` is the plain-string form.
#[must_use]
pub fn findings_json_value(file: &str, found: &Findings) -> serde_json::Value {
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
    Value::Object(root)
}

/// Pretty JSON of [`findings_json_value`] — the `--json` file/report form.
#[must_use]
pub fn findings_json(file: &str, found: &Findings) -> String {
    serde_json::to_string_pretty(&findings_json_value(file, found)).unwrap_or_default()
}

/// One flat JSON object per finding per line (NDJSON). Stream/grep friendly,
/// never coloured; empty (no lines) when there are zero findings.
#[must_use]
pub fn findings_ndjson(found: &Findings) -> String {
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
    s
}

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

    /// The three renderers must actually emit the finding content — a canned
    /// empty/default return passes no assertion here.
    #[test]
    fn renderers_emit_the_finding_content() {
        let mut f = Findings::new();
        add(&mut f, "AGS Format Rule 8", Some(5), "LOCA", "boom");

        let v = findings_json_value("d.ags", &f);
        assert_eq!(v["file"], serde_json::json!("d.ags"));
        assert_eq!(
            v["findings"]["AGS Format Rule 8"][0]["desc"],
            serde_json::json!("boom")
        );

        let json = findings_json("d.ags", &f);
        assert!(json.contains("\"file\": \"d.ags\""), "{json}");
        assert!(
            json.contains("boom") && json.contains("AGS Format Rule 8"),
            "{json}"
        );

        let nd = findings_ndjson(&f);
        assert_eq!(
            nd.trim(),
            r#"{"rule":"AGS Format Rule 8","line":5,"group":"LOCA","desc":"boom"}"#
        );
    }
}
