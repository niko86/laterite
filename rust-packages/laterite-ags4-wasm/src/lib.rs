//! Browser wasm wrapper around the clean-room AGS4 validator.
//!
//! `validate()` replicates the body of `laterite_ags4_validator::check_file_with_dict`
//! (`lib.rs`) but from in-memory bytes with `source = None`, so it runs
//! the entire rule engine **client-side** with no filesystem and nothing
//! uploaded. Rule *violations* come back as data in the report; only
//! un-validatable inputs (not AGS4, unsupported edition, bad arguments)
//! populate `report.error` — nothing throws across the wasm boundary.
//!
//! Phase 2 adds `read()` → typed Arrow IPC for the DuckDB-wasm data
//! explorer; this file is Phase 1 (validator) only.

// #168 Phase 3: parse types + tokenizer come straight from the leaf
// (encoding_rs + memchr only — wasm-safe); the validator dep stays for rules.
use laterite_ags4_parse::{ParsedFile, parse_bytes};
use laterite_ags4_validator::{
    CheckOptions, DictVersion, ValidatorError, dict::Dictionary, dict::FALLBACK, findings,
    resolve_dict_version, rules, tran_ags_of,
};
use laterite_types::sql_type;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

/// One rule violation — mirrors the CLI's `{line, group, desc}` JSON,
/// plus the additive rule-aware location/severity fields (omitted when
/// unset so the base shape is unchanged). `target`/`severity` use
/// snake_case to match the engine's serde rename + the TS interface.
#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
struct FindingDto {
    line: Option<u32>,
    group: String,
    desc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    target: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    field_index: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    heading: Option<String>,
    /// 1-based row ordinal within the group (distinct from `line`); set
    /// for data-row findings so the UI can address the exact cell.
    #[serde(skip_serializing_if = "Option::is_none")]
    data_row: Option<u32>,
    /// Half-open `[start, end)` char-offset span within the raw line —
    /// either carried by the finding (Rules 1/6) or computed here from
    /// `field_index` so every cell/heading finding gets a precise span.
    /// Serialized as a 2-element array to match the TS `[number, number]`.
    #[serde(skip_serializing_if = "Option::is_none")]
    char_span: Option<[u32; 2]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    severity: Option<String>,
}

/// Findings for a single rule. The report flattens the engine's
/// `BTreeMap<rule, Vec<Finding>>` into an ordered array of these so the
/// UI can render one collapsible section per rule without re-sorting
/// (the engine already orders by rule label).
#[derive(Serialize)]
struct RuleGroup {
    rule: String,
    /// True number of findings for this rule, **before** any serialization
    /// cap. `items.len()` may be smaller when `max_per_rule` clips the tail;
    /// the UI shows "N of `total`" so a cap never hides the real count.
    total: usize,
    items: Vec<FindingDto>,
}

/// An un-validatable input (not a rule violation). `kind` is a stable
/// machine token the UI switches on; `message` is the human string.
#[derive(Serialize)]
struct ValErr {
    kind: String,
    message: String,
}

/// The whole result of a validation run. `ok` is the verdict
/// (`error.is_none() && finding_count == 0`); when `error` is set the
/// other fields are empty/zero and no rules ran.
#[derive(Serialize)]
struct ValidationReport {
    ok: bool,
    /// The bundled edition the file was judged against (`"4.1.1"`, …),
    /// empty on error.
    dict_version: String,
    /// How that edition was chosen: `forced` / `exact` / `guessed` /
    /// `fallback` (see `DictResolution`), empty on error.
    resolution: String,
    /// True total across every rule — always the full count the engine
    /// found, independent of any serialization cap.
    finding_count: usize,
    /// How many `FindingDto` were actually serialized into `findings`
    /// (the sum of each group's `items.len()`). Equals `finding_count`
    /// when uncapped; smaller when `max_per_rule` clipped some groups, so
    /// the UI can say "showing `shown_count` of `finding_count`".
    shown_count: usize,
    findings: Vec<RuleGroup>,
    error: Option<ValErr>,
}

impl ValidationReport {
    fn failure(kind: &str, message: String) -> Self {
        ValidationReport {
            ok: false,
            dict_version: String::new(),
            resolution: String::new(),
            finding_count: 0,
            shown_count: 0,
            findings: Vec::new(),
            error: Some(ValErr {
                kind: kind.to_string(),
                message,
            }),
        }
    }
}

/// Map a `ValidatorError` to a `(kind, message)`. In the wasm path only
/// `NotAgs4` / `UnsupportedEdition` are actually reachable — there is no
/// filesystem (so no `NotFound`/`Io`), decode is lossy (non-UTF-8 surfaces
/// as a Rule 1 finding), and we never set `custom_dict` (so no `BadDict`) —
/// but we map every arm so the `match` is total and future-proof.
fn classify(e: &ValidatorError) -> (&'static str, String) {
    let kind = match e {
        ValidatorError::NotAgs4(_) => "not_ags4",
        ValidatorError::UnsupportedEdition { .. } => "unsupported_edition",
        ValidatorError::BadDict { .. } => "bad_dict",
        ValidatorError::NotFound(_) | ValidatorError::Io { .. } => "io",
    };
    (kind, e.to_string())
}

/// Resolve a UI encoding label to an `encoding_rs` encoding. The select
/// offers UTF-8 + Windows-1252 (the legacy producer); any other WHATWG
/// label flows through `for_label`. Default + unknown → UTF-8, matching
/// the validator's historical `from_utf8_lossy` behaviour.
fn resolve_encoding(label: Option<&str>) -> &'static encoding_rs::Encoding {
    let Some(label) = label else {
        return encoding_rs::UTF_8;
    };
    match label.trim().to_ascii_lowercase().as_str() {
        "" | "utf-8" | "utf8" => encoding_rs::UTF_8,
        "cp1252" | "windows-1252" | "latin1" | "latin-1" | "iso-8859-1" => {
            encoding_rs::WINDOWS_1252
        }
        other => encoding_rs::Encoding::for_label(other.as_bytes()).unwrap_or(encoding_rs::UTF_8),
    }
}

/// Map a UI dict-version string to a forced edition. `None` / `"auto"`
/// ⇒ `Ok(None)` (auto-detect from `TRAN_AGS`). An unrecognised string
/// returns `Err(message)` (the caller turns it into a `bad_args`
/// report); we return the short message rather than the whole report so
/// the `Err` variant stays small (clippy `result_large_err`).
/// `"4.0.3|4.0.4|4.1|4.1.1|4.2"` for error messages — from the generated set.
fn editions_pipe() -> String {
    DictVersion::ALL
        .iter()
        .map(|v| v.as_str())
        .collect::<Vec<_>>()
        .join("|")
}

fn resolve_dict_override(s: Option<&str>) -> Result<Option<DictVersion>, String> {
    match s.map(str::trim) {
        None | Some("") | Some("auto") => Ok(None),
        Some(other) => DictVersion::from_edition(other).map(Some).ok_or_else(|| {
            format!(
                "unknown dict_version {other:?}; expected auto|{}",
                editions_pipe()
            )
        }),
    }
}

// --- AGS4 production: `build_ags4` (the read path reversed) -----------------

/// One group of input data, deserialised from the browser's JSON. The
/// column `headings` are the AGS headings; `units`/`types` are optional
/// per-heading overrides (the dictionary fills the rest); `rows` cells are
/// JSON values (numbers/strings/bools/null).
#[derive(Deserialize)]
struct GroupInputJson {
    code: String,
    headings: Vec<String>,
    #[serde(default)]
    units: Option<Vec<String>>,
    #[serde(default)]
    types: Option<Vec<String>>,
    rows: Vec<Vec<serde_json::Value>>,
}

/// One emit finding, flattened with its rule label for the JS side.
#[derive(Serialize)]
struct EmitFinding {
    rule: String,
    line: Option<u32>,
    group: String,
    desc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    severity: Option<String>,
}

/// The `build_ags4` result. `text` is the AGS4 document (UTF-8, CRLF line
/// endings) — the browser wraps it in a `Blob` to download.
#[derive(Serialize)]
struct BuildAgs4Report {
    text: String,
    findings: Vec<EmitFinding>,
    fixes_applied: usize,
}

fn emit_edition(s: Option<&str>) -> Result<DictVersion, String> {
    match s.map(str::trim) {
        None | Some("") | Some("auto") => Ok(FALLBACK),
        Some(other) => DictVersion::from_edition(other)
            .ok_or_else(|| format!("unknown edition {other:?}; expected {}", editions_pipe())),
    }
}

fn emit_mode(s: Option<&str>) -> Result<laterite_ags4_emit::EmitMode, String> {
    match s.map(str::trim).map(str::to_ascii_lowercase).as_deref() {
        None | Some("") | Some("autofix") => Ok(laterite_ags4_emit::EmitMode::AutoFix),
        Some("report") => Ok(laterite_ags4_emit::EmitMode::Report),
        Some("strict") => Ok(laterite_ags4_emit::EmitMode::Strict),
        Some(other) => Err(format!(
            "unknown mode {other:?}; expected autofix|report|strict"
        )),
    }
}

/// Core of [`build_ags4`], host-testable (no `JsValue`): parse the JSON, run
/// the shared `laterite-ags4-emit` orchestrator, flatten the findings.
fn build_ags4_from_json(
    groups_json: &str,
    edition: Option<&str>,
    mode: Option<&str>,
) -> Result<BuildAgs4Report, String> {
    let parsed: Vec<GroupInputJson> =
        serde_json::from_str(groups_json).map_err(|e| format!("invalid groups JSON: {e}"))?;
    let groups: Vec<laterite_ags4_emit::GroupInput> = parsed
        .into_iter()
        .map(|g| laterite_ags4_emit::GroupInput {
            code: g.code,
            headings: g.headings,
            units: g.units,
            types: g.types,
            rows: g.rows,
        })
        .collect();
    emit_report(groups, edition, mode)
}

/// Run the shared orchestrator over already-built `GroupInput`s and shape the
/// JS report. The common tail of both input paths (JSON and Arrow IPC).
fn emit_report(
    groups: Vec<laterite_ags4_emit::GroupInput>,
    edition: Option<&str>,
    mode: Option<&str>,
) -> Result<BuildAgs4Report, String> {
    let opts = laterite_ags4_emit::EmitOpts {
        mode: emit_mode(mode)?,
        edition: emit_edition(edition)?,
    };
    let res = laterite_ags4_emit::emit_ags4(&groups, &opts).map_err(|e| e.to_string())?;
    let findings = res
        .findings
        .iter()
        .flat_map(|(rule, items)| {
            items.iter().map(move |f| EmitFinding {
                rule: rule.clone(),
                line: f.line,
                group: f.group.clone(),
                desc: f.desc.clone(),
                severity: match f.severity {
                    findings::Severity::Error => None,
                    s => Some(format!("{s:?}").to_lowercase()),
                },
            })
        })
        .collect();
    Ok(BuildAgs4Report {
        text: String::from_utf8_lossy(&res.bytes).into_owned(),
        findings,
        fixes_applied: res.fixes_applied,
    })
}

/// Decode one group's Arrow IPC stream → a [`laterite_ags4_emit::GroupInput`] (the
/// column names are the AGS headings). Uses the shared Arrow→Value transpose.
fn group_from_ipc(code: String, bytes: &[u8]) -> Result<laterite_ags4_emit::GroupInput, String> {
    let reader = arrow::ipc::reader::StreamReader::try_new(std::io::Cursor::new(bytes), None)
        .map_err(|e| format!("arrow ipc: {e}"))?;
    let schema = reader.schema();
    let mut batches = Vec::new();
    for b in reader {
        batches.push(b.map_err(|e| format!("arrow ipc batch: {e}"))?);
    }
    Ok(laterite_ags4_emit::group_from_arrow(
        code,
        schema.as_ref(),
        &batches,
    ))
}

/// Build valid AGS4 from typed/string data in the browser — the data→AGS4
/// producer (the read path reversed), with no server round-trip.
///
/// * `groups_json` — a JSON array of `{ code, headings, units?, types?, rows }`
///   (each row an array of cell values). The headings are the AGS headings;
///   UNIT/TYPE fill from the chosen edition's dictionary where omitted.
/// * `dict_version` — `None`/`"auto"` → `4.1.1`, or `4.0.3|4.0.4|4.1|4.1.1|4.2`.
/// * `mode` — `None`/`"autofix"` (default) | `"report"` | `"strict"`. Under
///   `"autofix"` the missing UNIT/TYPE/TRAN/ABBR metadata groups are synthesized
///   (from the data) so a data-only build is valid; `"report"`/`"strict"` don't.
///
/// Returns `{ text, findings, fixes_applied }`; `text` is the AGS4 document
/// (UTF-8, CRLF) for the browser to wrap in a `Blob`.
#[wasm_bindgen]
pub fn build_ags4(
    groups_json: &str,
    dict_version: Option<String>,
    mode: Option<String>,
) -> Result<JsValue, JsError> {
    console_error_panic_hook::set_once();
    let report = build_ags4_from_json(groups_json, dict_version.as_deref(), mode.as_deref())
        .map_err(|e| JsError::new(&e))?;
    serde_wasm_bindgen::to_value(&report).map_err(|e| JsError::new(&e.to_string()))
}

/// Build valid AGS4 from **columnar Arrow IPC** input — the same as
/// [`build_ags4`] but for large, already-columnar browser data (e.g. a
/// duckdb-wasm query result) without a per-cell JSON round-trip.
///
/// * `groups` — a JS array of `{ code: string, ipc: Uint8Array }`, each `ipc`
///   an Arrow **IPC stream** for one group (its schema's field names are the
///   AGS headings). Order is preserved (put `PROJ` first).
/// * `dict_version` / `mode` — as [`build_ags4`].
///
/// Returns the same `{ text, findings, fixes_applied }`. The Arrow→AGS
/// transpose is the read path's IPC reversed.
#[wasm_bindgen]
pub fn build_ags4_ipc(
    groups: JsValue,
    dict_version: Option<String>,
    mode: Option<String>,
) -> Result<JsValue, JsError> {
    use wasm_bindgen::JsCast;
    console_error_panic_hook::set_once();
    let arr = js_sys::Array::from(&groups);
    let mut inputs: Vec<laterite_ags4_emit::GroupInput> = Vec::with_capacity(arr.length() as usize);
    for item in arr.iter() {
        let code = js_sys::Reflect::get(&item, &JsValue::from_str("code"))
            .ok()
            .and_then(|v| v.as_string())
            .ok_or_else(|| JsError::new("each group needs a string `code`"))?;
        let ipc_val = js_sys::Reflect::get(&item, &JsValue::from_str("ipc"))
            .map_err(|_| JsError::new("each group needs an `ipc` Uint8Array"))?;
        let ipc = ipc_val
            .dyn_into::<js_sys::Uint8Array>()
            .map_err(|_| JsError::new("group `ipc` must be a Uint8Array"))?
            .to_vec();
        inputs.push(group_from_ipc(code, &ipc).map_err(|e| JsError::new(&e))?);
    }
    let report = emit_report(inputs, dict_version.as_deref(), mode.as_deref())
        .map_err(|e| JsError::new(&e))?;
    serde_wasm_bindgen::to_value(&report).map_err(|e| JsError::new(&e.to_string()))
}

#[cfg(test)]
mod build_ags4_tests {
    use super::*;

    #[test]
    fn emits_valid_and_canonicalises_typed() {
        let json = r#"[
          {"code":"PROJ","headings":["PROJ_ID","PROJ_NAME"],"rows":[["P1","Demo"]]},
          {"code":"LOCA","headings":["LOCA_ID","LOCA_GL"],"rows":[["BH01",12.3]]}
        ]"#;
        let r = build_ags4_from_json(json, Some("4.1.1"), Some("autofix")).unwrap();
        assert!(
            r.text.contains("\"12.30\""),
            "expected canonical 2DP:\n{}",
            r.text
        );
        assert!(
            r.text.contains("\"UNIT\",\"\",\"m\""),
            "dict UNIT fill:\n{}",
            r.text
        );
        // The emitted text re-parses as well-formed AGS4.
        let parsed = parse_bytes(r.text.as_bytes(), encoding_rs::UTF_8).unwrap();
        assert!(parsed.groups.contains_key("LOCA"));
    }

    #[test]
    fn autofix_pads_a_string_numeric() {
        let json = r#"[
          {"code":"PROJ","headings":["PROJ_ID"],"rows":[["P1"]]},
          {"code":"LOCA","headings":["LOCA_ID","LOCA_GL"],"rows":[["BH01","12.3"]]}
        ]"#;
        let r = build_ags4_from_json(json, None, Some("autofix")).unwrap();
        assert!(r.fixes_applied >= 1, "AutoFix should apply a safe fix");
        assert!(r.text.contains("\"12.30\""), "{}", r.text);
    }

    #[test]
    fn report_keeps_strings_verbatim() {
        let json = r#"[{"code":"LOCA","headings":["LOCA_ID","LOCA_GL"],"rows":[["BH01","12.3"]]}]"#;
        let r = build_ags4_from_json(json, None, Some("report")).unwrap();
        assert!(r.text.contains("\"12.3\""));
        assert_eq!(r.fixes_applied, 0);
    }

    #[test]
    fn rejects_unknown_mode_and_edition() {
        let json = r#"[{"code":"LOCA","headings":["LOCA_ID"],"rows":[["BH01"]]}]"#;
        assert!(build_ags4_from_json(json, None, Some("banana")).is_err());
        assert!(build_ags4_from_json(json, Some("9.9"), None).is_err());
    }

    #[test]
    fn ipc_columnar_input_emits_valid() {
        use arrow::array::{Float64Array, StringArray};
        use arrow::datatypes::{DataType, Field, Schema};
        use arrow::record_batch::RecordBatch;
        use std::sync::Arc;

        // Serialize a batch to an Arrow IPC stream (the read path's writer).
        fn ipc_bytes(schema: &Arc<Schema>, batch: &RecordBatch) -> Vec<u8> {
            let mut buf = Vec::new();
            let mut w = arrow::ipc::writer::StreamWriter::try_new(&mut buf, schema).unwrap();
            w.write(batch).unwrap();
            w.finish().unwrap();
            drop(w);
            buf
        }

        let proj_schema = Arc::new(Schema::new(vec![Field::new(
            "PROJ_ID",
            DataType::Utf8,
            false,
        )]));
        let proj_batch = RecordBatch::try_new(
            proj_schema.clone(),
            vec![Arc::new(StringArray::from(vec!["P1"]))],
        )
        .unwrap();
        let loca_schema = Arc::new(Schema::new(vec![
            Field::new("LOCA_ID", DataType::Utf8, false),
            Field::new("LOCA_GL", DataType::Float64, true), // a 2DP heading
        ]));
        let loca_batch = RecordBatch::try_new(
            loca_schema.clone(),
            vec![
                Arc::new(StringArray::from(vec!["BH01", "BH02"])),
                Arc::new(Float64Array::from(vec![12.3, 13.0])),
            ],
        )
        .unwrap();

        // Decode each IPC stream via the shared transpose, then emit.
        let proj = group_from_ipc("PROJ".into(), &ipc_bytes(&proj_schema, &proj_batch)).unwrap();
        let loca = group_from_ipc("LOCA".into(), &ipc_bytes(&loca_schema, &loca_batch)).unwrap();
        let r = emit_report(vec![proj, loca], Some("4.1.1"), Some("autofix")).unwrap();

        assert!(r.text.contains("\"12.30\""), "float64 → 2DP:\n{}", r.text);
        assert!(r.text.contains("\"13.00\""), "{}", r.text);
        assert!(
            r.text.contains("\"UNIT\",\"\",\"m\""),
            "dict fill:\n{}",
            r.text
        );
        let parsed = parse_bytes(r.text.as_bytes(), encoding_rs::UTF_8).unwrap();
        assert!(parsed.groups.contains_key("LOCA"));
    }
}

/// Validate AGS4 bytes in the browser.
///
/// * `data` — the file bytes (from a `FileReader`/textarea, never uploaded).
/// * `dict_version` — `None`/`"auto"` to detect from `TRAN_AGS`, or a
///   forced edition string.
/// * `include_warnings` — surface WARNING-severity findings (e.g. Rule 18).
/// * `include_fyi` — surface FYI-severity findings (e.g. Rule 1 FYI).
/// * `encoding_label` — `None`/`"utf-8"` or `"windows-1252"` for legacy files.
/// * `max_per_rule` — cap on how many findings per rule are **serialized**
///   into the report. `None` (the download path) serializes everything;
///   `Some(n)` (the interactive UI) clips each group to its first `n` so a
///   pathologically dirty file moves tens of thousands of rows across the
///   wasm→JS boundary, not the full millions. The cap is purely on output:
///   every rule still runs over every line, and `finding_count` /
///   `RuleGroup.total` always report the true, uncapped counts.
///
/// Returns a [`ValidationReport`] as a plain JS object (json-compatible:
/// `None` → `null`, matching the CLI's `--json`).
#[wasm_bindgen]
pub fn validate(
    data: &[u8],
    dict_version: Option<String>,
    include_warnings: bool,
    include_fyi: bool,
    encoding_label: Option<String>,
    max_per_rule: Option<u32>,
) -> JsValue {
    console_error_panic_hook::set_once();

    let report = run(
        data,
        dict_version.as_deref(),
        include_warnings,
        include_fyi,
        encoding_label.as_deref(),
        max_per_rule.map(|c| c as usize),
    );
    // json_compatible so the JS side sees plain objects + null (not Map
    // / undefined) — same shape the CLI emits.
    let serializer = serde_wasm_bindgen::Serializer::json_compatible();
    report
        .serialize(&serializer)
        .expect("ValidationReport is plain data and always serialises")
}

/// The AGS4 rule catalogue as the gated `rules_meta.json` JSON string — the
/// browser parses it into typed rule entries. Mirrors `laterite.list_rules()` /
/// `lat-check --list-rules`. No input.
#[wasm_bindgen]
pub fn list_rules() -> String {
    laterite_ags4_validator::rule_metadata_json().to_string()
}

fn run(
    data: &[u8],
    dict_version: Option<&str>,
    include_warnings: bool,
    include_fyi: bool,
    encoding_label: Option<&str>,
    max_per_rule: Option<usize>,
) -> ValidationReport {
    let dict_over = match resolve_dict_override(dict_version) {
        Ok(v) => v,
        Err(message) => return ValidationReport::failure("bad_args", message),
    };
    let encoding = resolve_encoding(encoding_label);

    let parsed = match parse_bytes(data, encoding) {
        Ok(p) => p,
        Err(e) => {
            // #168 Phase 3: convert the leaf's ParseError via the validator's
            // `From` bridge so `classify` (and the surfaced text) is unchanged.
            let (kind, message) = classify(&ValidatorError::from(e));
            return ValidationReport::failure(kind, message);
        }
    };

    let (dv, kind) = match resolve_dict_version(dict_over, tran_ags_of(&parsed).as_deref()) {
        Ok(r) => r,
        Err(e) => {
            let (k, message) = classify(&e);
            return ValidationReport::failure(k, message);
        }
    };

    let dict = Dictionary::bundled(dv);
    let opts = CheckOptions {
        dict_version: dict_over,
        include_warnings,
        include_fyi,
        encoding,
        ..CheckOptions::default()
    };

    let mut found = findings::Findings::new();
    // `source = None`: the data-level rules are path-independent and the
    // opt-in on-disk Rule 20 is double-gated behind `check_files` (false
    // here) AND `Some(source)`, so this is provably filesystem-free —
    // the wasm sandbox has no filesystem.
    rules::run_all(&parsed, &dict, &opts, None, &mut found);

    let finding_count = findings::count(&found);
    // Raw line text by 1-based line number — `raw_lines` is sequential
    // from 1, but index by `number` defensively rather than assuming the
    // offset. Used to compute a precise char span for cell/heading
    // findings that carry a `field_index` but no explicit `char_span`.
    // Built once (O(lines)) rather than re-scanned per finding, so the
    // span computation below stays O(1) per finding.
    let line_index: std::collections::HashMap<u32, &str> = parsed
        .raw_lines
        .iter()
        .map(|rl| (rl.number, rl.text.as_str()))
        .collect();
    let raw_line = |line: Option<u32>| -> Option<&str> { line_index.get(&line?).copied() };
    // Cap per rule on *serialization* only — every rule already ran over
    // every line above; we just clip the tail each group contributes to
    // the JS payload. `total` preserves the true per-rule count and
    // `shown_count` sums what actually crossed the boundary.
    //
    // On top of the per-rule cap, a GLOBAL budget bounds the TOTAL rows that
    // cross the boundary in the interactive (capped) mode — so a file with
    // hundreds of dirty rules can't structured-clone 100k–300k objects (a
    // multi-second freeze / OOM on a weak machine), no matter how many rules
    // fire. It only applies when `max_per_rule` is set; the download path
    // (`max_per_rule = None`) is intentionally uncapped so the full report is
    // available. `finding_count` stays the true total, so the UI's "showing N
    // of M" reflects the cap.
    const MAX_SHOWN_TOTAL: usize = 30_000;
    let global_budget = max_per_rule.map(|_| MAX_SHOWN_TOTAL);
    let mut shown_count = 0usize;
    let findings_out = found
        .into_iter()
        .map(|(rule, items)| {
            let total = items.len();
            let per_rule = max_per_rule.map_or(total, |c| total.min(c));
            let take = match global_budget {
                Some(budget) => per_rule.min(budget.saturating_sub(shown_count)),
                None => per_rule,
            };
            shown_count += take;
            RuleGroup {
                rule,
                total,
                items: items
                    .into_iter()
                    .take(take)
                    .map(|f| {
                        // target omitted when whole-line (the default); the
                        // finer targets serialize as their snake_case token,
                        // matching the engine + TS string union.
                        let target = match f.location.target {
                            findings::Target::Line => None,
                            findings::Target::Heading => Some("heading".to_string()),
                            findings::Target::Cell => Some("cell".to_string()),
                            findings::Target::Group => Some("group".to_string()),
                        };
                        // severity emitted as the lowercase token — taken from
                        // `Severity`'s own serde rename (the ONE source) rather
                        // than re-spelling error/warning/fyi here. Harmless that
                        // it's optional (TS treats it so) and lets the UI pick
                        // the row-band colour without inferring a default.
                        let severity = serde_json::to_value(f.severity)
                            .ok()
                            .and_then(|v| v.as_str().map(str::to_string));
                        // Span precedence: a finding-carried span (Rules 1/6)
                        // wins; otherwise, for a field-targeted finding,
                        // compute the inner-value span from the raw line so
                        // cell/heading findings get a precise highlight too.
                        let char_span = f.location.char_span.map(|(s, e)| [s, e]).or_else(|| {
                            let fi = f.location.field_index?;
                            let line = raw_line(f.line)?;
                            laterite_ags4_parse::field_span(line, fi).map(|(s, e)| [s, e])
                        });
                        FindingDto {
                            line: f.line,
                            group: f.group,
                            desc: f.desc,
                            target,
                            field_index: f.location.field_index,
                            heading: f.location.heading,
                            data_row: f.location.data_row,
                            char_span,
                            severity,
                        }
                    })
                    .collect(),
            }
        })
        .collect();

    ValidationReport {
        ok: finding_count == 0,
        dict_version: dv.as_str().to_string(),
        resolution: kind.as_str().to_string(),
        finding_count,
        shown_count,
        findings: findings_out,
        error: None,
    }
}

// ---------------------------------------------------------------------
// Apply-Fixes: compute_fixes() / apply_fixes() — a separate surface from
// validate() so the byte-faithful finding JSON is never perturbed. Both
// reuse the validate() skeleton (resolve_encoding + parse_bytes + the
// dict resolution + run_all) so a fix is computed against exactly the
// same findings the report shows.
// ---------------------------------------------------------------------

/// Compute the safe fixes for AGS4 bytes in the browser.
///
/// Parses + runs the full rule engine (FYI on, so e.g. the Rule 1 BOM
/// path is seen), then `laterite_ags4_validator::fixes::compute_fixes`. Returns a
/// JSON-compatible `Fix[]` (empty array on a parse error — there's
/// nothing to fix in an un-parseable file, and `validate` already
/// surfaces the error to the UI).
#[wasm_bindgen]
pub fn compute_fixes(
    data: &[u8],
    dict_version: Option<String>,
    encoding_label: Option<String>,
) -> JsValue {
    console_error_panic_hook::set_once();
    let serializer = serde_wasm_bindgen::Serializer::json_compatible();
    let empty: Vec<laterite_ags4_validator::fixes::Fix> = Vec::new();

    let dict_over = match resolve_dict_override(dict_version.as_deref()) {
        Ok(v) => v,
        Err(_) => return empty.serialize(&serializer).unwrap(),
    };
    let encoding = resolve_encoding(encoding_label.as_deref());
    let parsed = match parse_bytes(data, encoding) {
        Ok(p) => p,
        Err(_) => return empty.serialize(&serializer).unwrap(),
    };
    let (dv, _kind) = match resolve_dict_version(dict_over, tran_ags_of(&parsed).as_deref()) {
        Ok(r) => r,
        Err(_) => return empty.serialize(&serializer).unwrap(),
    };
    let dict = Dictionary::bundled(dv);
    let opts = CheckOptions {
        dict_version: dict_over,
        include_fyi: true,
        encoding,
        ..CheckOptions::default()
    };
    let mut found = findings::Findings::new();
    rules::run_all(&parsed, &dict, &opts, None, &mut found);

    let fixes = laterite_ags4_validator::fixes::compute_fixes(&parsed, &found);
    fixes
        .serialize(&serializer)
        .expect("Fixes is plain data and always serialises")
}

/// Apply a user-selected subset of fixes to AGS4 bytes, returning the new
/// file as **UTF-8 bytes** (a JS `Uint8Array`). The input is decoded with
/// `encoding_label` (capturing whether it carried a BOM), the fixes are
/// applied by the shared engine, and the result is always re-encoded as
/// UTF-8 — so applying to a cp1252 file also normalises its encoding
/// (Rule-1-friendly, and the caller resets its encoding select to utf-8).
#[wasm_bindgen]
pub fn apply_fixes(data: &[u8], encoding_label: Option<String>, fixes_json: JsValue) -> Vec<u8> {
    console_error_panic_hook::set_once();
    let encoding = resolve_encoding(encoding_label.as_deref());
    // Decode to text + capture the BOM the same way the engine does, so
    // apply_fixes can honour "keep the BOM" when StripBom isn't selected.
    let has_bom = data.starts_with(&[0xEF, 0xBB, 0xBF]);
    let (text, _enc, _had) = encoding.decode(data);

    let fixes: Vec<laterite_ags4_validator::fixes::Fix> =
        serde_wasm_bindgen::from_value(fixes_json).unwrap_or_default();

    let out = laterite_ags4_validator::fixes::apply_fixes(&text, has_bom, &fixes);
    out.into_bytes()
}

// ---------------------------------------------------------------------
// Phase 2: read() -> typed Arrow IPC for the DuckDB-wasm data explorer.
//
// AGS4 isn't a format DuckDB reads natively. We parse it in Rust, build
// ONE correctly-typed Arrow RecordBatch per group, and hand JS the IPC
// bytes; DuckDB-wasm's `insertArrowFromIPCStream` ingests it as the final
// typed table — no per-cell JS objects, no staging table, no TRY_CAST.
//
// Typing uses the SAME laterite_types::{canonical_type, parse_value,
// parse_datetime} the native .ags5db conversion uses, off the file's own
// TYPE row (convert.rs does the same), so the explorer casts a file
// IDENTICALLY to a .ags5db — parity by construction.
// ---------------------------------------------------------------------

/// A parsed AGS4 file held in wasm memory as the lightweight string
/// `ParsedFile`. Each group's typed `RecordBatch` is built lazily on the
/// `arrow_ipc(code)` call and dropped after the bytes are returned, so
/// peak residency is one batch — not every group at once.
#[wasm_bindgen]
pub struct ParsedDataset {
    parsed: ParsedFile,
}

/// Per-group schema for the UI: parallel arrays (one entry per heading).
#[derive(Serialize)]
struct GroupMeta {
    headings: Vec<String>,
    units: Vec<String>,
    /// AGS TYPE codes from the file's TYPE row (e.g. "2DP", "DT", "ID").
    types: Vec<String>,
    /// The DuckDB column type each heading lands as ("DOUBLE", "BIGINT",
    /// "TIMESTAMP", "VARCHAR", …) — what the table will report.
    sql_types: Vec<String>,
}

#[wasm_bindgen]
impl ParsedDataset {
    /// Group codes in file order (the order to load tables in).
    pub fn group_codes(&self) -> Vec<String> {
        self.parsed.group_order.clone()
    }

    /// `{headings, units, types, sql_types}` for one group, or `null` if
    /// the code isn't present.
    pub fn meta(&self, code: &str) -> JsValue {
        let Some(group) = self.parsed.groups.get(code) else {
            return JsValue::NULL;
        };
        let n = group.headings.len();
        let types: Vec<String> = (0..n)
            .map(|i| {
                group
                    .types
                    .get(i)
                    .cloned()
                    .unwrap_or_else(|| "X".to_string())
            })
            .collect();
        let meta = GroupMeta {
            headings: group.headings.clone(),
            units: (0..n)
                .map(|i| group.units.get(i).cloned().unwrap_or_default())
                .collect(),
            sql_types: types.iter().map(|t| sql_type(t).to_string()).collect(),
            types,
        };
        let serializer = serde_wasm_bindgen::Serializer::json_compatible();
        meta.serialize(&serializer).unwrap_or(JsValue::NULL)
    }

    /// One group's rows as an Arrow IPC **stream** (Uint8Array), columns
    /// already correctly typed. Built lazily here and dropped on return.
    pub fn arrow_ipc(&self, code: &str) -> Result<Vec<u8>, JsError> {
        let group = self
            .parsed
            .groups
            .get(code)
            .ok_or_else(|| JsError::new(&format!("group {code:?} not in dataset")))?;

        // Typed columns + IPC framing both come from laterite-types now
        // (`ipc::build_group_ipc` = the shared `arrow_cols` cast + StreamWriter)
        // — the SAME composition the napi host frames, so the browser, Node and
        // Python type a file byte-identically by construction. Framed here only
        // for duckdb-wasm.
        let buf = laterite_types::ipc::build_group_ipc(
            &group.headings,
            &group.types,
            group.rows.len(),
            |col, row| group.cell(col, row),
        )
        .map_err(|e| JsError::new(&format!("arrow ipc for {code}: {e}")))?;
        Ok(buf)
    }
}

/// Parse AGS4 bytes into a typed dataset for the explorer. Validation is
/// a separate concern (`validate`); this is permissive — it builds typed
/// columns for whatever parsed, so the explorer works even on a file with
/// findings. Only an unparseable-as-AGS4 input returns `Err`.
#[wasm_bindgen]
pub fn read(data: &[u8], encoding_label: Option<String>) -> Result<ParsedDataset, JsError> {
    console_error_panic_hook::set_once();
    let encoding = resolve_encoding(encoding_label.as_deref());
    let parsed = parse_bytes(data, encoding)
        .map_err(|e| JsError::new(&ValidatorError::from(e).to_string()))?;
    Ok(ParsedDataset { parsed })
}

/// Compare two AGS4 files. `max_rows_per_group` caps how many per-row deltas
/// each group serialises (the `added`/`removed`/`changed` counts are always
/// the true totals); `None` serialises everything.
#[wasm_bindgen]
pub fn diff(
    a: &[u8],
    b: &[u8],
    encoding_label: Option<String>,
    max_rows_per_group: Option<u32>,
) -> Result<JsValue, JsError> {
    console_error_panic_hook::set_once();
    let encoding = resolve_encoding(encoding_label.as_deref());
    let pa =
        parse_bytes(a, encoding).map_err(|e| JsError::new(&ValidatorError::from(e).to_string()))?;
    let pb =
        parse_bytes(b, encoding).map_err(|e| JsError::new(&ValidatorError::from(e).to_string()))?;

    // KEY headings come from the dictionary; pick the edition from the
    // revision's TRAN_AGS (the "new" file), falling back to the standard.
    let dv = resolve_dict_version(None, tran_ags_of(&pb).as_deref())
        .map(|(dv, _)| dv)
        .unwrap_or(laterite_ags4_validator::dict::FALLBACK);
    let dict = Dictionary::bundled(dv);
    let cap = max_rows_per_group.map(|c| c as usize);

    // The KEY-aware/type-aware comparison itself lives in the shared
    // laterite-ags4-diff leaf (so PyO3 + the CLI reuse it); this wrapper only
    // parses, resolves the dictionary, and serialises the result to JS.
    let delta = laterite_ags4_diff::diff_parsed(&pa, &pb, &dict, cap);
    let serializer = serde_wasm_bindgen::Serializer::json_compatible();
    delta
        .serialize(&serializer)
        .map_err(|e| JsError::new(&e.to_string()))
}

// ---------------------------------------------------------------------
// dictionary() -> the bundled STANDARD dictionary for an edition.
//
// The Tools reference (Dictionary browser / Template generator) used to fetch
// a static `ags5_dictionary.json` — the scaffolded AGS5 *merged* dict, where
// ~91% of headings had EMPTY descriptions and it was a single fixed edition.
// This exposes the validator's real per-edition standard dictionary instead:
// canonical names + descriptions + units + types + status, selectable across
// 4.0.3 … 4.2 (the same data the engine validates against).
// ---------------------------------------------------------------------

#[derive(Serialize)]
struct DictHeadingDto {
    name: String,
    status: String,
    #[serde(rename = "type")]
    ags_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    unit: Option<String>,
    description: String,
}

#[derive(Serialize)]
struct DictGroupDto {
    code: String,
    /// The group's standard description (its "contents"/name).
    contents: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    parent: Option<String>,
    headings: Vec<DictHeadingDto>,
}

#[derive(Serialize)]
struct DictDto {
    ags_edition: String,
    groups: Vec<DictGroupDto>,
}

/// Serialise the bundled standard dictionary for `dict_version`
/// (`None`/`"auto"` → the [`FALLBACK`] edition; else `4.0.3|4.0.4|4.1|4.1.1|
/// 4.2`). Groups are sorted by code; each group's headings keep the canonical
/// dictionary order. Returns the web reference UI's `{ags_edition, groups:[…]}`
/// shape.
#[wasm_bindgen]
pub fn dictionary(dict_version: Option<String>) -> Result<JsValue, JsError> {
    console_error_panic_hook::set_once();
    let version = resolve_dict_override(dict_version.as_deref())
        .map_err(|e| JsError::new(&e))?
        .unwrap_or(FALLBACK);
    let d = Dictionary::bundled(version);
    let mut codes: Vec<&'static str> = d.group_codes().collect();
    codes.sort_unstable();
    let groups: Vec<DictGroupDto> = codes
        .into_iter()
        .map(|code| {
            let gm = d.group(code);
            let headings = d
                .group_headings(code)
                .iter()
                .map(|&h| {
                    let e = d.heading(code, h);
                    DictHeadingDto {
                        name: h.to_string(),
                        status: e.map(|x| x.status).unwrap_or("").to_string(),
                        ags_type: e.map(|x| x.ags_type).unwrap_or("").to_string(),
                        unit: e
                            .map(|x| x.unit)
                            .filter(|u| !u.is_empty())
                            .map(str::to_string),
                        description: e.map(|x| x.desc).unwrap_or("").to_string(),
                    }
                })
                .collect();
            DictGroupDto {
                code: code.to_string(),
                contents: gm.map(|m| m.desc).unwrap_or("").to_string(),
                parent: gm
                    .map(|m| m.parent)
                    .filter(|p| !p.is_empty())
                    .map(str::to_string),
                headings,
            }
        })
        .collect();
    let dto = DictDto {
        ags_edition: version.as_str().to_string(),
        groups,
    };
    let serializer = serde_wasm_bindgen::Serializer::json_compatible();
    dto.serialize(&serializer)
        .map_err(|e| JsError::new(&e.to_string()))
}

#[cfg(test)]
mod tests {
    //! Parity-by-construction guard for `read()`'s typed-Arrow path.
    //!
    //! `build_column` is the whole casting surface (the wasm-bindgen
    //! wrappers above only marshal it), and it casts through the SAME
    //! `laterite_types` fns — off the file's TYPE row — that the native
    //! `.ags5db` convert uses (`laterite-ags5-db/src/convert.rs`). So asserting the
    //! Arrow `DataType` + cell values here proves the explorer casts a
    //! file identically to a `.ags5db`, with no DuckDB/Node/wasm runtime.
    //! The datetime oracle is computed independently via `chrono`.
    use super::*;
    // `Array` provides `is_null`/`len`; ArrayRef/DataType/TimeUnit assert the
    // shape of what the shared laterite-types builder hands back.
    use arrow::array::{
        Array, ArrayRef, BooleanArray, Float64Array, Int64Array, StringArray,
        TimestampMicrosecondArray,
    };
    use arrow::datatypes::{DataType, TimeUnit};
    use chrono::NaiveDate;

    // Exercises every canonical category: ID/X -> Utf8, 2DP -> Float64,
    // DT -> Timestamp (full datetime, date-only -> midnight, empty ->
    // null), 0DP -> Int64, YN -> Bool. BH03's blank coords/dates check
    // the null path; SAMP is a child group with a YN column.
    const FIXTURE: &[u8] = b"\"GROUP\",\"LOCA\"\r\n\
\"HEADING\",\"LOCA_ID\",\"LOCA_NATE\",\"LOCA_STAR\",\"LOCA_ENDD\",\"LOCA_REM\"\r\n\
\"UNIT\",\"\",\"m\",\"yyyy-mm-dd\",\"yyyy-mm-dd\",\"\"\r\n\
\"TYPE\",\"ID\",\"2DP\",\"DT\",\"DT\",\"X\"\r\n\
\"DATA\",\"BH01\",\"523145.67\",\"2020-08-18 09:30:00\",\"2020-08-19\",\"first\"\r\n\
\"DATA\",\"BH02\",\"523200.00\",\"2020-08-20\",\"\",\"second\"\r\n\
\"DATA\",\"BH03\",\"\",\"\",\"\",\"third\"\r\n\
\r\n\
\"GROUP\",\"GEOL\"\r\n\
\"HEADING\",\"LOCA_ID\",\"GEOL_STAT\"\r\n\
\"UNIT\",\"\",\"\"\r\n\
\"TYPE\",\"ID\",\"0DP\"\r\n\
\"DATA\",\"BH01\",\"1\"\r\n\
\"DATA\",\"BH01\",\"2\"\r\n\
\r\n\
\"GROUP\",\"SAMP\"\r\n\
\"HEADING\",\"LOCA_ID\",\"SAMP_DEPTH_OK\"\r\n\
\"UNIT\",\"\",\"\"\r\n\
\"TYPE\",\"ID\",\"YN\"\r\n\
\"DATA\",\"BH01\",\"Y\"\r\n\
\"DATA\",\"BH01\",\"N\"\r\n";

    fn parsed() -> ParsedFile {
        parse_bytes(FIXTURE, encoding_rs::UTF_8).expect("fixture parses")
    }

    /// Build the typed column for `group`'s heading `name`, returning the
    /// array + its `DataType`. Routes through the shared laterite-types builder
    /// (the production path), feeding it this column's cells.
    fn column(file: &ParsedFile, group: &str, name: &str) -> (ArrayRef, DataType) {
        let g = &file.groups[group];
        let col = g.headings.iter().position(|h| h == name).expect("heading");
        let ags_type = &g.types[col];
        laterite_types::arrow_cols::build_column(g.rows.len(), ags_type, |row| {
            g.rows
                .get(row)
                .and_then(|r| r.values.get(col))
                .map(String::as_str)
        })
    }

    fn micros(y: i32, m: u32, d: u32, hh: u32, mm: u32, ss: u32) -> i64 {
        NaiveDate::from_ymd_opt(y, m, d)
            .unwrap()
            .and_hms_opt(hh, mm, ss)
            .unwrap()
            .and_utc()
            .timestamp_micros()
    }

    #[test]
    fn id_and_x_are_utf8() {
        let file = parsed();
        for name in ["LOCA_ID", "LOCA_REM"] {
            let (arr, dt) = column(&file, "LOCA", name);
            assert_eq!(dt, DataType::Utf8, "{name}");
            assert!(arr.as_any().is::<StringArray>());
        }
        let (rem, _) = column(&file, "LOCA", "LOCA_REM");
        let rem = rem.as_any().downcast_ref::<StringArray>().unwrap();
        assert_eq!(rem.value(0), "first");
        assert_eq!(rem.value(2), "third");
    }

    #[test]
    fn two_dp_is_float64_with_nulls() {
        let file = parsed();
        let (arr, dt) = column(&file, "LOCA", "LOCA_NATE");
        assert_eq!(dt, DataType::Float64);
        let a = arr.as_any().downcast_ref::<Float64Array>().unwrap();
        assert_eq!(a.value(0), 523145.67);
        assert_eq!(a.value(1), 523200.00);
        assert!(a.is_null(2), "blank 2DP cell -> null");
    }

    #[test]
    fn dt_is_timestamp_full_dateonly_and_null() {
        let file = parsed();
        let (star, dt) = column(&file, "LOCA", "LOCA_STAR");
        assert_eq!(dt, DataType::Timestamp(TimeUnit::Microsecond, None));
        let star = star
            .as_any()
            .downcast_ref::<TimestampMicrosecondArray>()
            .unwrap();
        // full datetime kept; date-only promoted to midnight; blank null.
        assert_eq!(star.value(0), micros(2020, 8, 18, 9, 30, 0));
        assert_eq!(star.value(1), micros(2020, 8, 20, 0, 0, 0));
        assert!(star.is_null(2));

        let (end, _) = column(&file, "LOCA", "LOCA_ENDD");
        let end = end
            .as_any()
            .downcast_ref::<TimestampMicrosecondArray>()
            .unwrap();
        assert_eq!(end.value(0), micros(2020, 8, 19, 0, 0, 0));
        assert!(end.is_null(1), "blank DT cell -> null");
    }

    #[test]
    fn zero_dp_is_int64() {
        let file = parsed();
        let (arr, dt) = column(&file, "GEOL", "GEOL_STAT");
        assert_eq!(dt, DataType::Int64);
        let a = arr.as_any().downcast_ref::<Int64Array>().unwrap();
        assert_eq!(a.value(0), 1);
        assert_eq!(a.value(1), 2);
    }

    #[test]
    fn yn_is_bool() {
        let file = parsed();
        let (arr, dt) = column(&file, "SAMP", "SAMP_DEPTH_OK");
        assert_eq!(dt, DataType::Boolean);
        let a = arr.as_any().downcast_ref::<BooleanArray>().unwrap();
        assert!(a.value(0));
        assert!(!a.value(1));
    }

    #[test]
    fn ragged_short_row_yields_nulls_not_panic() {
        // A data row shorter than the heading count must null the missing
        // tail columns, never panic or misalign — the explorer has to
        // survive malformed real-world files.
        let bytes = b"\"GROUP\",\"LOCA\"\r\n\
\"HEADING\",\"LOCA_ID\",\"LOCA_NATE\"\r\n\
\"UNIT\",\"\",\"m\"\r\n\
\"TYPE\",\"ID\",\"2DP\"\r\n\
\"DATA\",\"BH01\"\r\n";
        let file = parse_bytes(bytes, encoding_rs::UTF_8).unwrap();
        let (arr, _) = column(&file, "LOCA", "LOCA_NATE");
        let a = arr.as_any().downcast_ref::<Float64Array>().unwrap();
        assert_eq!(a.len(), 1);
        assert!(a.is_null(0));
    }

    // --- encoding resolution + transcode (the cp1252/UTF-8 path) ---
    // This is the bug class behind the web app's mojibake guard + the
    // "applying a fix also normalises encoding" promise; it had zero coverage.

    #[test]
    fn resolve_encoding_maps_the_offered_labels() {
        assert_eq!(resolve_encoding(None).name(), "UTF-8");
        assert_eq!(resolve_encoding(Some("")).name(), "UTF-8");
        assert_eq!(resolve_encoding(Some("utf-8")).name(), "UTF-8");
        for label in ["windows-1252", "CP1252", "latin1", "ISO-8859-1"] {
            assert_eq!(
                resolve_encoding(Some(label)).name(),
                "windows-1252",
                "{label}"
            );
        }
        // An unknown label falls back to UTF-8 (lossy), not an error.
        assert_eq!(resolve_encoding(Some("not-a-charset")).name(), "UTF-8");
    }

    #[test]
    fn byte_0xe9_is_replacement_under_utf8_but_e_acute_under_cp1252() {
        let data = [b'a', 0xE9, b'b']; // 0xE9 = 'é' in cp1252, invalid UTF-8
        let (utf8, _, had_errors) = resolve_encoding(Some("utf-8")).decode(&data);
        assert!(had_errors, "0xE9 is not valid UTF-8");
        assert!(
            utf8.contains('\u{FFFD}'),
            "lossy decode inserts U+FFFD: {utf8:?}"
        );
        let (cp, _, had) = resolve_encoding(Some("windows-1252")).decode(&data);
        assert!(!had);
        assert_eq!(cp, "aéb");
    }

    #[test]
    fn apply_fixes_encoding_path_transcodes_cp1252_to_utf8() {
        // Mirror apply_fixes' encoding pipeline (decode → apply → into_bytes)
        // without the wasm-bindgen JsValue: a cp1252 0xE9 byte must come back as
        // the UTF-8 encoding of 'é' (0xC3 0xA9), even with no fixes selected.
        let data = [b'a', 0xE9, b'b'];
        let encoding = resolve_encoding(Some("windows-1252"));
        let (text, _, _) = encoding.decode(&data);
        let out = laterite_ags4_validator::fixes::apply_fixes(&text, false, &[]).into_bytes();
        assert_eq!(out, vec![b'a', 0xC3, 0xA9, b'b']);
    }

    // --- dictionary() data source ---
    // The JsValue wrapper can't be built off-wasm, so assert the per-edition
    // standard dictionary the export serialises actually carries real names +
    // descriptions (the scaffolded merged JSON the UI used to fetch did not).

    #[test]
    fn bundled_dictionary_has_real_names_and_descriptions() {
        let d = Dictionary::bundled(DictVersion::V4_1_1);
        let codes: Vec<&str> = d.group_codes().collect();
        assert!(codes.contains(&"LOCA"), "LOCA must be a standard group");
        assert!(
            !d.group("LOCA").unwrap().desc.trim().is_empty(),
            "LOCA must have a real group description",
        );
        let e = d.heading("LOCA", "LOCA_ID").expect("LOCA_ID heading");
        assert_eq!(e.status, "KEY");
        assert_eq!(e.ags_type, "ID");
        assert!(
            !e.desc.trim().is_empty(),
            "LOCA_ID must have a real description, got {:?}",
            e.desc,
        );
    }

    #[test]
    fn bundled_dictionary_differs_across_editions() {
        // 4.2 added groups over 4.0.3 — the per-edition dicts are not identical,
        // which is the whole point of making the browser edition-selectable.
        let n_403 = Dictionary::bundled(DictVersion::V4_0_3)
            .group_codes()
            .count();
        let n_42 = Dictionary::bundled(DictVersion::V4_2).group_codes().count();
        assert!(n_403 > 0 && n_42 > 0);
        assert!(
            n_42 >= n_403,
            "4.2 should have at least as many groups as 4.0.3"
        );
    }
}
