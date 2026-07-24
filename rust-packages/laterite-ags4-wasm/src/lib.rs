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
    CheckOptions, DictVersion, ValidatorError, WorldScope, check_parsed_with_dict,
    dict::Dictionary, dict::FALLBACK, findings, overlay, resolve_dict_version, tran_ags_of,
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
    /// Why a proffered `.ags.idx` certificate did NOT stand in for the rule engine,
    /// as the stable snake_case token (`"dictionary_changed"`, `"content_changed"`,
    /// …), else `null`. Present for cross-surface shape parity with `Report`
    /// (laterite-py) and Node's `revalidateReason` (#568 Phase 6). **Structurally
    /// always `null` here:** this surface has no cert-consume door — `validate`
    /// re-runs the engine unconditionally (`certify` only *mints*), so no
    /// certificate is ever offered to accept or reject. The field exists so a JS
    /// consumer reads the same report shape on every surface, and is ready if a
    /// wasm cert-consume path is ever added.
    revalidate_reason: Option<String>,
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
            revalidate_reason: None,
        }
    }
}

/// Map a `ValidatorError` to a `(kind, message)`. In the wasm path only
/// `NotAgs4` / `UnsupportedEdition` are actually reachable — there is no
/// filesystem (so no `NotFound`/`Io`), decode is lossy (non-UTF-8 surfaces
/// as a Rule 1 finding), and we never set `custom_dict` (so no `BadDict`) —
/// but we map every arm so the `match` is total and future-proof.
fn classify(e: &ValidatorError) -> (&'static str, String) {
    // Delegate to the single producer `ValidatorError::kind()`, except the
    // deliberate, allowlisted divergence: with no filesystem, `NotFound`/`Io` are
    // unreachable, so they collapse to `"io"` here (vs the producer's
    // `"not_found"`) purely to keep the match total. Gated in the tests below.
    let kind = match e {
        ValidatorError::NotFound(_) | ValidatorError::Io { .. } => "io",
        other => other.kind(),
    };
    (kind, e.to_string())
}

/// Resolve a UI encoding label to an `encoding_rs` encoding, via the shared label
/// table in the parse leaf so a label means the same thing on every surface.
///
/// An unknown label is an ERROR, not a fallback. It used to return UTF-8 — which
/// reads like leniency and behaves like corruption: `C3 A9` decodes cleanly as `é`
/// in UTF-8 and `Ã©` in cp1252, so a caller who asked for the wrong label got the
/// wrong text and a clean bill of health. Python raised on the same input. The
/// browser's own select only offers UTF-8 / Windows-1252, so the UI cannot trip
/// this; the wasm API is public, and a caller who names a charset we do not know
/// deserves to be told so.
fn resolve_encoding(
    label: Option<&str>,
) -> std::result::Result<&'static encoding_rs::Encoding, String> {
    laterite_ags4_parse::resolve_encoding(label)
        .ok_or_else(|| format!("unknown encoding {:?}", label.unwrap_or("")))
}

/// Map a UI dict-version string to a forced edition. `None` / `"auto"`
/// ⇒ `Ok(None)` (auto-detect from `TRAN_AGS`). An unrecognised string
/// returns `Err(message)` (the caller turns it into a `bad_args`
/// report); we return the short message rather than the whole report so
/// the `Err` variant stays small (clippy `result_large_err`).
fn resolve_dict_override(s: Option<&str>) -> Result<Option<DictVersion>, String> {
    match s.map(str::trim) {
        None | Some("") | Some("auto") => Ok(None),
        Some(other) => DictVersion::from_edition(other).map(Some).ok_or_else(|| {
            format!(
                "unknown dict_version {other:?}; expected auto|{}",
                laterite_ags4_validator::editions_joined("|")
            )
        }),
    }
}

/// Build the runtime custom-dictionary overlay (#568) from browser-supplied bytes.
///
/// The wasm sandbox has no filesystem, so — unlike the CLI/Python/Node twins — this
/// has no path arm: a custom dict always arrives as raw bytes (a `Uint8Array` the UI
/// read from a file). `over` forces a base edition (from `dict_version`),
/// `dict_replace` drops the base entirely, and the two cannot both hold (a forced
/// base contradicts a full replacement). `enc` is the caller's already-resolved
/// source encoding — the same one it hands `CheckOptions`.
///
/// Returns `Ok(None)` when no dict was supplied. The error is a short message the
/// caller surfaces on the same channel a bad `dict_version` uses.
fn build_custom_dict(
    dict_bytes: Option<&[u8]>,
    dict_replace: bool,
    over: Option<DictVersion>,
    enc: &'static encoding_rs::Encoding,
) -> std::result::Result<Option<overlay::CustomDict>, String> {
    let Some(bytes) = dict_bytes else {
        return Ok(None);
    };
    if dict_replace && over.is_some() {
        return Err("dict_replace cannot be combined with dict_version \
             (a forced base contradicts a full replacement)"
            .to_string());
    }
    let base = if dict_replace {
        overlay::BaseSpec::Replace
    } else if let Some(v) = over {
        overlay::BaseSpec::Force(v)
    } else {
        overlay::BaseSpec::Auto
    };
    // The advisory name the cert records is a neutral label — never a filesystem path
    // (the browser has none anyway), matching the in-memory-bytes arm on every surface.
    overlay::parse_dict(bytes, overlay::DictFormat::Auto, enc, base, "custom-dict")
        .map(Some)
        .map_err(|e| format!("bad dict: {e}"))
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
        Some(other) => DictVersion::from_edition(other).ok_or_else(|| {
            format!(
                "unknown edition {other:?}; expected {}",
                laterite_ags4_validator::editions_joined("|")
            )
        }),
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
        // Metadata synthesis inherits the default, which is now OFF
        // (2026-07-24): no surface mints GROUPs the caller never wrote without
        // being asked. See EmitOpts::synthesise_metadata.
        ..laterite_ags4_emit::EmitOpts::default()
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
                    s => Some(s.as_str().to_string()),
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
    // Never emit a synthetic content-addressed key column: drop any `_`-prefixed
    // column a `read(keys=true)` frame might carry back into build. AGS headings
    // never start with "_", so this is safe (a no-op when none are present). (#303)
    let keep: Vec<usize> = (0..schema.fields().len())
        .filter(|&i| !schema.field(i).name().starts_with('_'))
        .collect();
    if keep.len() != schema.fields().len() {
        let pschema = std::sync::Arc::new(
            schema
                .project(&keep)
                .map_err(|e| format!("arrow project schema: {e}"))?,
        );
        let pbatches: Vec<arrow::record_batch::RecordBatch> = batches
            .iter()
            .map(|b| b.project(&keep))
            .collect::<Result<_, _>>()
            .map_err(|e| format!("arrow project batch: {e}"))?;
        return Ok(laterite_ags4_emit::group_from_arrow(
            code,
            pschema.as_ref(),
            &pbatches,
        ));
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
/// * `mode` — `None`/`"autofix"` (default) | `"report"` | `"strict"`. `"autofix"`
///   repairs what the input contains. It does NOT mint the mandatory
///   UNIT/TYPE/TRAN/ABBR catalogs — that became opt-in on 2026-07-24, so a
///   data-only build reports Rule 14/15/17 rather than silently filling them.
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
/// * `dict_bytes` — an optional custom AGS4 dictionary (`.ags` or JSON),
///   supplied as raw bytes (#568). The browser has no filesystem, so — unlike
///   `lat --dict <path>` — the dict always arrives in memory. `None` uses the
///   bundled edition. A bespoke group declared here becomes first-class instead
///   of being flagged as unknown.
/// * `dict_replace` — with `dict_bytes`, drop the bundled base entirely (the dict
///   fully replaces the standard) rather than overlaying on top of it. Contradicts
///   a forced `dict_version`; supplying both is a `bad_dict` error.
///
/// Returns a [`ValidationReport`] as a plain JS object (json-compatible:
/// `None` → `null`, matching the CLI's `--json`).
#[allow(clippy::too_many_arguments)] // the wasm surface mirrors lat's positional flags
#[wasm_bindgen]
pub fn validate(
    data: &[u8],
    dict_version: Option<String>,
    include_warnings: bool,
    include_fyi: bool,
    encoding_label: Option<String>,
    max_per_rule: Option<u32>,
    dict_bytes: Option<Vec<u8>>,
    dict_replace: bool,
) -> JsValue {
    console_error_panic_hook::set_once();

    let report = run(
        data,
        dict_version.as_deref(),
        include_warnings,
        include_fyi,
        encoding_label.as_deref(),
        max_per_rule.map(|c| c as usize),
        dict_bytes.as_deref(),
        dict_replace,
    );
    // json_compatible so the JS side sees plain objects + null (not Map
    // / undefined) — same shape the CLI emits.
    let serializer = serde_wasm_bindgen::Serializer::json_compatible();
    report
        .serialize(&serializer)
        .expect("ValidationReport is plain data and always serialises")
}

/// Mint a `.ags.idx` validity certificate, entirely client-side (#360). Returns the
/// certificate JSON for the browser to download.
///
/// * `checked_at` — an RFC-3339 timestamp from the browser (`new Date()
///   .toISOString()`): wasm has no clock, so the caller supplies it.
///
/// Errors if the file can't be parsed, or has **errors** — warnings and FYI findings are
/// *measured and recorded*, not fatal.
///
/// The mint is the shared one (`laterite_ags4_trust::mint`), so a browser-minted
/// certificate is byte-for-byte the same statement as one from `lat certify`. It used to
/// be assembled here: this surface ran an ERRORS-ONLY validation and then wrote
/// `warnings: 0, fyi: 0` into the stamp — a claim to have measured two tiers it had never
/// looked at, which a later `validate --warnings --index` on any surface would have
/// believed. The mint now measures every tier itself, and there is no parameter left
/// through which this function could assert one.
///
/// It also cannot record an on-disk `FILE/` check: the wasm sandbox has no filesystem, and
/// the stamp no longer has a field in which to say otherwise.
///
/// `dict_bytes` / `dict_replace` mint the certificate against a custom dictionary
/// (#568), the same overlay `validate` accepts: the stamp records the dict's
/// `{name, hash}` so a later `validate --index` on any surface re-validates (never
/// silently vouches) when the effective dictionary differs (O-48, record-not-contract).
#[wasm_bindgen]
pub fn certify(
    data: &[u8],
    dict_version: Option<String>,
    encoding_label: Option<String>,
    checked_at: String,
    dict_bytes: Option<Vec<u8>>,
    dict_replace: bool,
) -> Result<String, JsError> {
    console_error_panic_hook::set_once();

    let dict_over = resolve_dict_override(dict_version.as_deref()).map_err(|m| JsError::new(&m))?;
    let encoding = resolve_encoding(encoding_label.as_deref()).map_err(|m| JsError::new(&m))?;
    let custom_dict = build_custom_dict(dict_bytes.as_deref(), dict_replace, dict_over, encoding)
        .map_err(|m| JsError::new(&m))?;

    let opts = CheckOptions {
        dict_version: dict_over,
        encoding,
        custom_dict,
        ..CheckOptions::default()
    };
    let sidecar = laterite_ags4_trust::mint(data, &opts, checked_at, None)
        .map_err(|e| JsError::new(&e.to_string()))?;
    let json = sidecar
        .to_json()
        .map_err(|e| JsError::new(&e.to_string()))?;
    String::from_utf8(json).map_err(|e| JsError::new(&e.to_string()))
}

/// The AGS4 rule catalogue as the gated `rules_meta.json` JSON string — the
/// browser parses it into typed rule entries. Mirrors `laterite.list_rules()` /
/// `lat rules`. No input.
#[wasm_bindgen]
pub fn list_rules() -> String {
    laterite_ags4_validator::rule_metadata_json().to_string()
}

/// The browser highlight's char-span, by precedence: a finding-CARRIED span
/// (Rules 1/6 attach one directly) wins; otherwise, for a field-targeted finding
/// that carries a `field_index`, derive the inner-value span from the raw source
/// line via the parse leaf's [`laterite_ags4_parse::field_span`].
///
/// This derivation exists on NO other surface — `char_span` is serialized only
/// by wasm, and it drives the browser's cell/heading highlight — yet before #555
/// part 1 it had zero test coverage (and `field_span`, the leaf it calls, had
/// none either). Extracted from `run`'s finding-mapping closure so the precedence
/// AND the offset it produces are pinned by the `derive_char_span_*` tests.
fn derive_char_span(
    carried: Option<(u32, u32)>,
    field_index: Option<u32>,
    raw_line: Option<&str>,
) -> Option<[u32; 2]> {
    carried
        .map(|(s, e)| [s, e])
        .or_else(|| laterite_ags4_parse::field_span(raw_line?, field_index?).map(|(s, e)| [s, e]))
}

#[cfg(test)]
mod char_span_tests {
    use super::derive_char_span;

    // A DATA row whose tag-stripped column 1 is the value "10.5".
    const LINE: &str = r#""DATA","BH1","10.5""#;

    #[test]
    fn carried_span_wins_over_derivation() {
        // Rules 1/6 attach an explicit span; it is used verbatim even when a
        // field_index + line are also present — the derivation must not override.
        assert_eq!(
            derive_char_span(Some((3, 7)), Some(1), Some(LINE)),
            Some([3, 7])
        );
    }

    #[test]
    fn derives_inner_value_span_from_field_index() {
        // No carried span → derive from the raw line. field_index is the
        // TAG-STRIPPED column, so on `"DATA","BH1","10.5"` index 1 is the value
        // "10.5" at chars 14..18 and index 0 is "BH1" at 8..11 — the exact
        // offsets the browser highlights, and the first coverage of both this
        // derivation and the leaf `field_span` it calls.
        assert_eq!(derive_char_span(None, Some(1), Some(LINE)), Some([14, 18]));
        assert_eq!(derive_char_span(None, Some(0), Some(LINE)), Some([8, 11]));
    }

    #[test]
    fn none_without_field_index() {
        // A whole-line / whole-group finding (no field_index) gets no span.
        assert_eq!(derive_char_span(None, None, Some(LINE)), None);
    }

    #[test]
    fn none_without_raw_line() {
        // field_index present but the source line unavailable → no span, no panic.
        assert_eq!(derive_char_span(None, Some(1), None), None);
    }
}

#[allow(clippy::too_many_arguments)] // mirrors the public `validate` arg list one-to-one
fn run(
    data: &[u8],
    dict_version: Option<&str>,
    include_warnings: bool,
    include_fyi: bool,
    encoding_label: Option<&str>,
    max_per_rule: Option<usize>,
    dict_bytes: Option<&[u8]>,
    dict_replace: bool,
) -> ValidationReport {
    let dict_over = match resolve_dict_override(dict_version) {
        Ok(v) => v,
        Err(message) => return ValidationReport::failure("bad_args", message),
    };
    let encoding = match resolve_encoding(encoding_label) {
        Ok(e) => e,
        // Same channel a bad dict_version uses: the caller SEES the bad label,
        // instead of getting findings that are artefacts of a UTF-8 fallback.
        Err(message) => return ValidationReport::failure("bad_args", message),
    };
    // The custom-dict overlay is resolved (base detected, delta built, hash minted)
    // once here, before parsing the delivery — a bad dictionary is the DICTIONARY's
    // problem and is reported as such, on the same channel a bad dict_version uses.
    let custom_dict = match build_custom_dict(dict_bytes, dict_replace, dict_over, encoding) {
        Ok(c) => c,
        Err(message) => return ValidationReport::failure("bad_dict", message),
    };

    let parsed = match parse_bytes(data, encoding) {
        Ok(p) => p,
        Err(e) => {
            // #168 Phase 3: convert the leaf's ParseError via the validator's
            // `From` bridge so `classify` (and the surfaced text) is unchanged.
            let (kind, message) = classify(&ValidatorError::from(e));
            return ValidationReport::failure(kind, message);
        }
    };

    let opts = CheckOptions {
        dict_version: dict_over,
        include_warnings,
        include_fyi,
        encoding,
        custom_dict,
        ..CheckOptions::default()
    };

    // The one door: it resolves the edition, applies the O-42 4.0.3→4.0.4 content
    // guard, runs the rules, and emits the transparency FYI. The browser used to
    // assemble those steps itself and left the guard out, so a file mislabelled
    // 4.0.3 was judged here against a different dictionary than `lat` used on the
    // same bytes. `WorldScope::None` is the only scope the browser can name —
    // `OnDisk` needs a path, and the wasm sandbox has no filesystem — so this run is
    // filesystem-free by construction, not by a coincidentally-false flag.
    let (found, dv, kind) = match check_parsed_with_dict(&parsed, &opts, &WorldScope::None) {
        Ok(r) => r,
        Err(e) => return ValidationReport::failure(e.kind(), e.to_string()),
    };

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
                        // severity emitted as the lowercase token via the single
                        // producer `Severity::as_str` (gated == the serde rename),
                        // rather than re-spelling error/warning/fyi here. Optional
                        // (TS treats it so) and lets the UI pick the row-band
                        // colour without inferring a default.
                        let severity = Some(f.severity.as_str().to_string());
                        // Span precedence (see `derive_char_span`): a
                        // finding-carried span (Rules 1/6) wins; else derive the
                        // inner-value span from the raw line for field-targeted
                        // findings so cell/heading findings highlight precisely.
                        let char_span = derive_char_span(
                            f.location.char_span,
                            f.location.field_index,
                            raw_line(f.line),
                        );
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
        // No cert-consume path on this surface (see the field's doc): the engine
        // always ran, so there is no proffered certificate to have rejected.
        revalidate_reason: None,
    }
}

#[cfg(test)]
mod dict_overlay_tests {
    use super::run;

    // The #568 Phase-3 end-to-end fixtures, shared with the validator's `custom_dict.rs`
    // so the browser is proven against the same bytes: a bespoke `XTRA` group hung off
    // the standard `SAMP`, and a delivery that uses it.
    const DELIVERY: &[u8] = include_bytes!(
        "../../laterite-ags4-validator/tests/fixtures/custom_dict/delivery_with_xtra.ags"
    );
    const DICT_JSON: &[u8] =
        include_bytes!("../../laterite-ags4-validator/tests/fixtures/custom_dict/xtra.dict.json");

    fn xtra_findings(report: &super::ValidationReport) -> usize {
        report
            .findings
            .iter()
            .flat_map(|g| &g.items)
            .filter(|f| f.group == "XTRA")
            .count()
    }

    #[test]
    fn dict_bytes_make_a_bespoke_group_valid_in_the_browser() {
        // Acceptance #3 (browser leg) + #5: the same bytes the CLI/py/node smokes use.
        // Warnings + FYI on so the unknown-group findings are actually surfaced.
        let err_msg = |r: &super::ValidationReport| {
            r.error
                .as_ref()
                .map(|e| e.message.clone())
                .unwrap_or_default()
        };
        let without = run(DELIVERY, None, true, true, None, None, None, false);
        assert!(
            without.error.is_none(),
            "delivery parses: {}",
            err_msg(&without)
        );
        assert!(
            xtra_findings(&without) > 0,
            "the bundled dictionary must flag the unknown XTRA group"
        );

        // With the custom dictionary supplied as bytes (the browser's only form),
        // XTRA is a first-class group and draws no findings.
        let with = run(
            DELIVERY,
            None,
            true,
            true,
            None,
            None,
            Some(DICT_JSON),
            false,
        );
        assert!(
            with.error.is_none(),
            "delivery parses with dict: {}",
            err_msg(&with)
        );
        assert_eq!(
            xtra_findings(&with),
            0,
            "the overlay makes XTRA recognised ({} residual XTRA findings)",
            xtra_findings(&with)
        );

        // Acceptance #5: `revalidate_reason` is a present field on the report — always
        // `None` here because this surface has no cert-consume door (see the field's doc).
        assert!(with.revalidate_reason.is_none());
    }

    #[test]
    fn dict_replace_with_a_forced_base_is_a_bad_dict_error() {
        // The one contradiction: a full replacement cannot also force a base edition.
        let report = run(
            DELIVERY,
            Some("4.2"),
            true,
            true,
            None,
            None,
            Some(DICT_JSON),
            true,
        );
        let err = report
            .error
            .expect("contradiction is surfaced, not silently ignored");
        assert_eq!(err.kind, "bad_dict");
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
    // An unknown label yields no fixes rather than fixes computed against the
    // wrong decoding — this fn has no error channel, and silently "fixing" text
    // we mis-decoded is the worst option on the table.
    let Ok(encoding) = resolve_encoding(encoding_label.as_deref()) else {
        return empty.serialize(&serializer).unwrap();
    };
    let parsed = match parse_bytes(data, encoding) {
        Ok(p) => p,
        Err(_) => return empty.serialize(&serializer).unwrap(),
    };
    let opts = CheckOptions {
        dict_version: dict_over,
        include_fyi: true,
        encoding,
        ..CheckOptions::default()
    };
    // Through the door, so the fixes offered in the browser are computed against the
    // same dictionary `lat fix` would use on the same bytes (the O-42 guard included).
    // No error channel here: a failure yields no fixes rather than fixes derived from
    // the wrong dictionary.
    let Ok((found, _dv, _kind)) = check_parsed_with_dict(&parsed, &opts, &WorldScope::None) else {
        return empty.serialize(&serializer).unwrap_or(JsValue::NULL);
    };

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
///
/// Throws on an unknown `encoding_label`. This used to be infallible and fell back
/// to UTF-8 — meaning it would REWRITE a file it had just mis-decoded, which is the
/// one place a silent fallback does permanent damage. The worker already turns a
/// throw into an `ok: false` reply, and the UI's encoding select is a closed union,
/// so the browser cannot reach this path; a direct wasm caller can, and should be
/// told rather than handed a corrupted file.
#[wasm_bindgen]
pub fn apply_fixes(
    data: &[u8],
    encoding_label: Option<String>,
    fixes_json: JsValue,
) -> Result<Vec<u8>, JsError> {
    console_error_panic_hook::set_once();
    let encoding = resolve_encoding(encoding_label.as_deref()).map_err(|m| JsError::new(&m))?;
    // Decode to text + capture the BOM the same way the engine does, so
    // apply_fixes can honour "keep the BOM" when StripBom isn't selected.
    let has_bom = data.starts_with(&[0xEF, 0xBB, 0xBF]);
    let (text, _enc, _had) = encoding.decode(data);

    let fixes: Vec<laterite_ags4_validator::fixes::Fix> =
        serde_wasm_bindgen::from_value(fixes_json).unwrap_or_default();

    let out = laterite_ags4_validator::fixes::apply_fixes(&text, has_bom, &fixes);
    Ok(out.into_bytes())
}

// ---------------------------------------------------------------------
// AGS4 ↔ XLSX (#359). The FS-free laterite-excel cores (`ags4_bytes_to_xlsx` /
// `xlsx_bytes_to_ags4`) drive the browser Excel surface: the Tools pane hands
// us bytes and gets bytes + warnings back, no filesystem. calamine reads and
// rust_xlsxwriter writes — both pure-Rust and wasm-clean.
// ---------------------------------------------------------------------

/// The result of an Excel conversion: the output `bytes` (a JS `Uint8Array` —
/// the `.xlsx` or `.ags` file), plus the `warnings` and counts the UI surfaces
/// (dropped non-Rule-19 columns, skipped sheets, …).
#[wasm_bindgen]
pub struct ExcelResult {
    bytes: Vec<u8>,
    warnings: Vec<String>,
    sheets: usize,
    rows: usize,
}

#[wasm_bindgen]
impl ExcelResult {
    #[wasm_bindgen(getter)]
    pub fn bytes(&self) -> Vec<u8> {
        self.bytes.clone()
    }
    #[wasm_bindgen(getter)]
    pub fn warnings(&self) -> Vec<String> {
        self.warnings.clone()
    }
    #[wasm_bindgen(getter)]
    pub fn sheets(&self) -> usize {
        self.sheets
    }
    #[wasm_bindgen(getter)]
    pub fn rows(&self) -> usize {
        self.rows
    }
}

/// AGS4 bytes → an `.xlsx` workbook (one sheet per group, python-ags4's
/// layout). `JsError` if the input carries no valid AGS4 groups.
#[wasm_bindgen]
pub fn ags4_to_xlsx(
    data: &[u8],
    recover_duplicate_headings: Option<bool>,
) -> Result<ExcelResult, JsError> {
    console_error_panic_hook::set_once();
    use laterite_ags4_core::ags4_codec::{DuplicateHeadings, ReadOptions};
    // Duplicate headings are fatal by default here as on every read surface; the
    // browser caller opts into the suffixed recovery read.
    let opts = ReadOptions {
        duplicate_headings: if recover_duplicate_headings.unwrap_or(false) {
            DuplicateHeadings::Recover
        } else {
            DuplicateHeadings::Error
        },
    };
    let (bytes, stats) = laterite_excel::ags4_bytes_to_xlsx_with(data, None, opts)
        .map_err(|e| JsError::new(&e.to_string()))?;
    Ok(ExcelResult {
        bytes,
        warnings: stats.warnings,
        sheets: stats.sheets_written,
        rows: stats.rows_written,
    })
}

/// An `.xlsx` workbook's bytes → AGS4 bytes. Each sheet with a `HEADING` column
/// becomes a group; non-Rule-19 columns and non-`UNIT`/`TYPE`/`DATA` rows are
/// dropped (surfaced in `warnings`). `format_numeric` re-pads DATA cells to
/// their column's TYPE (mirrors python-ags4's `convert_to_text`). `JsError` if
/// no sheet yields a valid group.
#[wasm_bindgen]
pub fn xlsx_to_ags4(data: &[u8], format_numeric: bool) -> Result<ExcelResult, JsError> {
    console_error_panic_hook::set_once();
    let (bytes, stats) = laterite_excel::xlsx_bytes_to_ags4(data, format_numeric)
        .map_err(|e| JsError::new(&e.to_string()))?;
    Ok(ExcelResult {
        bytes,
        warnings: stats.warnings,
        sheets: stats.sheets_written,
        rows: stats.rows_written,
    })
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
// parse_datetime} the native DuckDB conversion uses, off the file's own
// TYPE row (convert.rs does the same), so the explorer casts a file
// IDENTICALLY to the native DuckDB conversion — parity by construction.
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
    ///
    /// `keys` (default `false`) prepends the two content-addressed key columns
    /// `_id`/`_parent_id` — the SAME UUIDv8s the wheel / Node / DuckDB
    /// extension produce (via the one shared keychain). Pass `true` when feeding
    /// duckdb-wasm so cross-group joins (`s._parent_id = l._id`) resolve; leave
    /// it off (the default) for a plain typed frame. A custom/passthrough group
    /// carries no keys, so `keys` is a no-op for it. (#303)
    ///
    /// `content_hash` (default `false`) appends a trailing `_content_hash`
    /// value fingerprint (SHA-256 over the typed, blank-normalised heading
    /// values) — the SAME hash Node/Python produce via the one shared
    /// keychain. Unlike `keys` this needs no registry entry, so a
    /// custom/passthrough group still gets a usable `_content_hash` even
    /// without an `_id`. (#448)
    pub fn arrow_ipc(
        &self,
        code: &str,
        keys: Option<bool>,
        content_hash: Option<bool>,
    ) -> Result<Vec<u8>, JsError> {
        let group = self
            .parsed
            .groups
            .get(code)
            .ok_or_else(|| JsError::new(&format!("group {code:?} not in dataset")))?;

        // Typed columns + IPC framing both come from laterite-types now
        // (`ipc::build_group_ipc_synth` = the shared `arrow_cols` cast + StreamWriter,
        // `_id`/`_parent_id` col 0/1, `_content_hash` trailing) — the SAME
        // composition the napi host frames, so the browser, Node and Python type
        // a file byte-identically by construction. Framed here only for
        // duckdb-wasm.
        let reg = laterite_ags4_core::registry::registry();
        let ids = (keys.unwrap_or(false) && reg.get(code).is_some()).then(|| {
            laterite_ags4_core::keychain::group_row_ids(
                reg,
                code,
                &group.headings,
                group.rows.len(),
                |col, row| group.cell(col, row),
            )
        });
        let hashes = if content_hash.unwrap_or(false) {
            Some(laterite_ags4_core::keychain::group_content_hashes(
                code,
                &group.headings,
                &group.units,
                &group.types,
                group.rows.len(),
                |col, row| group.cell(col, row),
            ))
        } else {
            None
        };
        let buf = laterite_types::ipc::build_group_ipc_synth(
            &laterite_types::arrow_cols::SynthColumns {
                ids: ids.as_deref(),
                hashes: hashes.as_deref(),
            },
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
    let encoding = resolve_encoding(encoding_label.as_deref()).map_err(|m| JsError::new(&m))?;
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
    let encoding = resolve_encoding(encoding_label.as_deref()).map_err(|m| JsError::new(&m))?;
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

/// The result of a merge: the reconciled `bytes` (a JS `Uint8Array` — the merged
/// `.ags` file), plus `warnings_json` and `revisions_json` (the audit arrays the
/// Tools UI parses — the same shape PyO3 / Node return).
#[wasm_bindgen]
pub struct MergeResult {
    bytes: Vec<u8>,
    warnings_json: String,
    revisions_json: String,
}

#[wasm_bindgen]
impl MergeResult {
    #[wasm_bindgen(getter)]
    pub fn bytes(&self) -> Vec<u8> {
        self.bytes.clone()
    }
    #[wasm_bindgen(getter)]
    pub fn warnings_json(&self) -> String {
        self.warnings_json.clone()
    }
    #[wasm_bindgen(getter)]
    pub fn revisions_json(&self) -> String {
        self.revisions_json.clone()
    }
}

/// Merge two AGS4 deliveries of one project into one file (`a` then `b` — `b`
/// wins a KEY conflict). Rows are matched by their dictionary KEY headings. A
/// heading the two files typed differently is a `JsError` unless `on_type_clash`
/// settles it — `"widen"` falls back to `X` (raw values kept), `"promote"` keeps the
/// greatest nDP precision (zero-padding the coarser values). `tran_issue` +
/// `tran_date` (both) stamp a synthesised
/// merge-TRAN. The edition is `b`'s `TRAN_AGS`, falling back to the standard.
#[allow(clippy::too_many_arguments)]
#[wasm_bindgen]
pub fn merge(
    a: &[u8],
    b: &[u8],
    encoding_label: Option<String>,
    on_type_clash: Option<String>,
    tran_issue: Option<String>,
    tran_date: Option<String>,
    tran_producer: Option<String>,
    tran_recipient: Option<String>,
    tran_status: Option<String>,
) -> Result<MergeResult, JsError> {
    use laterite_ags4_merge::{MergeOpts, TranStamp, TypeClashMode, merge_parsed};

    console_error_panic_hook::set_once();
    let encoding = resolve_encoding(encoding_label.as_deref()).map_err(|m| JsError::new(&m))?;
    let pa =
        parse_bytes(a, encoding).map_err(|e| JsError::new(&ValidatorError::from(e).to_string()))?;
    let pb =
        parse_bytes(b, encoding).map_err(|e| JsError::new(&ValidatorError::from(e).to_string()))?;

    // Edition from the newest file (b)'s TRAN_AGS, falling back to the standard.
    let dv = resolve_dict_version(None, tran_ags_of(&pb).as_deref())
        .map(|(dv, _)| dv)
        .unwrap_or(laterite_ags4_validator::dict::FALLBACK);

    // A merge-TRAN is synthesised only when both an issue and a date are given.
    let tran = match (tran_issue, tran_date) {
        (Some(isno), Some(date)) => Some(TranStamp {
            isno,
            date,
            prod: tran_producer.unwrap_or_default(),
            recv: tran_recipient.unwrap_or_default(),
            stat: tran_status.unwrap_or_default(),
            ags: dv.as_str().to_string(),
        }),
        _ => None,
    };

    // One vocabulary for every surface: accepted tokens + rejection message come
    // from the merge crate's FromStr, so the browser cannot drift from the CLI.
    let clash: TypeClashMode = on_type_clash
        .as_deref()
        .unwrap_or("error")
        .parse()
        .map_err(|m: String| JsError::new(&m))?;

    let opts = MergeOpts {
        on_type_clash: clash,
        edition: dv,
        tran,
        ..Default::default()
    };

    let res = merge_parsed(&[pa, pb], &opts).map_err(|e| JsError::new(&e.to_string()))?;
    let warnings: Vec<_> = res
        .warnings
        .iter()
        .map(|w| {
            serde_json::json!({
                "kind": w.kind, "group": w.group,
                "heading": w.heading, "message": w.message,
            })
        })
        .collect();
    let revisions: Vec<_> = res
        .revisions
        .iter()
        .map(|r| {
            serde_json::json!({
                "group": r.group, "key": r.key,
                "changed": r.changed, "winnerFile": r.winner_file,
            })
        })
        .collect();
    Ok(MergeResult {
        bytes: res.bytes,
        warnings_json: serde_json::to_string(&warnings).unwrap_or_else(|_| "[]".into()),
        revisions_json: serde_json::to_string(&revisions).unwrap_or_else(|_| "[]".into()),
    })
}

// ---------------------------------------------------------------------
// dictionary() -> the bundled STANDARD dictionary for an edition.
//
// The Tools reference (Dictionary browser / Template generator) used to fetch
// a static scaffolded dictionary — a single fixed edition where ~91% of
// headings had EMPTY descriptions.
// This exposes the validator's real per-edition standard dictionary instead:
// canonical names + descriptions + units + types + status, selectable across
// 4.0.3 … 4.2 (the same data the engine validates against).
// ---------------------------------------------------------------------

/// Serialise the bundled standard dictionary for `dict_version`
/// (`None`/`"auto"` → the [`FALLBACK`] edition; else `4.0.3|4.0.4|4.1|4.1.1|
/// 4.2`). Groups are sorted by code; each group's headings keep the canonical
/// The crate version — the same answer Node's `version()` gives, from the same
/// `CARGO_PKG_VERSION`.
///
/// It exists because `ags4-compliance`'s wasm runner HARD-CODED `version: "0.5.1"`
/// (tools/compliance/emit_js.mjs) — a literal true when it was written, that the
/// workspace moved past to 0.7.0 while nothing compared it back. The harness then
/// printed "wasm v0.5.1" next to three 0.7.0 surfaces and called the comparison
/// 4-laterite identity. The build was current; only the report lied. Node had this
/// all along and asked the module; wasm had nothing to ask, which is why someone
/// wrote a constant instead. (#556)
#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// dictionary order. Returns the web reference UI's `{ags_edition, groups:[…]}`
/// shape — built by the shared `dict::dictionary_dto` (#294 F#6), the same
/// source `laterite.registry.dictionary()` and Node's render.
#[wasm_bindgen]
pub fn dictionary(dict_version: Option<String>) -> Result<JsValue, JsError> {
    console_error_panic_hook::set_once();
    let version = resolve_dict_override(dict_version.as_deref())
        .map_err(|e| JsError::new(&e))?
        .unwrap_or(FALLBACK);
    let dto = laterite_ags4_validator::dict::dictionary_dto(version);
    let serializer = serde_wasm_bindgen::Serializer::json_compatible();
    dto.serialize(&serializer)
        .map_err(|e| JsError::new(&e.to_string()))
}

// ---------------------------------------------------------------------
// censor() -> anonymise a file with the shared scrub engine (#581).
//
// The browser Anonymiser drives the SAME `laterite-ags4-censor` engine the
// corpus `censor` tool uses (Phase 2 of the #527 convergence), instead of a
// hand-written TS reimplementation. It's a batch action (Download click), off
// the render path, so it rides the engine wasm asynchronously in the validator
// worker rather than a boot-critical main-thread instance.
// ---------------------------------------------------------------------

/// `{ text, tally }` — the anonymised file plus the per-action cell/structure
/// counts the Anonymiser surfaces. `tally`'s fields match the leaf's snake_case.
#[derive(Serialize)]
struct CensorDto {
    text: String,
    tally: laterite_ags4_censor::Tally,
}

/// Anonymise `data` with the shared engine. `sensitive_json` is the
/// classification SSOT (`sensitive_headings.json`); `selected_codes` (a JS
/// array of heading codes, or `null` for every classified heading) restricts
/// the policy to the user's ticked columns; `token` replaces token/brackets
/// hits; `drop_custom` removes non-dictionary groups/columns + their orphaned
/// DICT/ABBR rows; `include_freetext` tokenises descriptions instead of
/// stripping their `[units]`. Returns `{ text, tally }`.
///
/// `PROJ_ID`'s filehash is the full 64-hex SHA-256 of `data` (a KEY field —
/// full width so a collision is cryptographically nil); the leaf takes that id
/// precomputed, so this wrapper hashes the bytes.
#[wasm_bindgen]
pub fn censor(
    data: &[u8],
    sensitive_json: &str,
    selected_codes: JsValue,
    token: &str,
    drop_custom: bool,
    include_freetext: bool,
) -> Result<JsValue, JsError> {
    console_error_panic_hook::set_once();

    // Lossy decode (matches the Anonymiser's `TextDecoder({fatal:false})`): a
    // browser anonymises what it can rather than skipping non-UTF-8 outright.
    let text = String::from_utf8_lossy(data);
    let file_id = hex::encode(<sha2::Sha256 as sha2::Digest>::digest(data));

    let mut policy =
        laterite_ags4_censor::Policy::from_sensitive_json(sensitive_json, include_freetext)
            .map_err(|e| JsError::new(&e.to_string()))?;
    // null → keep the full policy (every classified heading); an array
    // restricts it to the browser's column selection.
    let selected: Option<Vec<String>> =
        serde_wasm_bindgen::from_value(selected_codes).map_err(|e| JsError::new(&e.to_string()))?;
    if let Some(codes) = selected {
        policy.retain_codes(&codes.into_iter().collect());
    }

    let opts = laterite_ags4_censor::CensorOptions {
        token: token.to_string(),
        keywords: Vec::new(),
        drop_custom,
    };
    let (out_text, tally) = laterite_ags4_censor::censor(&text, &file_id, &policy, &opts);

    let dto = CensorDto {
        text: out_text,
        tally,
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
    //! DuckDB conversion uses (`laterite-ags5-db/src/convert.rs`). So asserting the
    //! Arrow `DataType` + cell values here proves the explorer casts a
    //! file identically to that native conversion, with no DuckDB/Node/wasm runtime.
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

    const CLEAN_FIXTURE: &[u8] =
        include_bytes!("../../laterite-ags4-validator/tests/fixtures/clean_minimal.ags");

    /// A wasm-minted certificate now carries the shared engine identity — it used
    /// to stamp "laterite-ags4-wasm", which siloed browser certs from every other
    /// surface. Now a cert downloaded from the web app is one every surface can read.
    /// (#430 PR 1a)
    #[test]
    fn certify_stamps_the_unified_engine_identity() {
        let json = match certify(
            CLEAN_FIXTURE,
            None,
            None,
            "2020-01-01T00:00:00Z".to_string(),
            None,
            false,
        ) {
            Ok(s) => s,
            Err(_) => panic!("a clean minimal AGS4 file must certify"),
        };
        assert!(
            json.contains("laterite_ags4") && !json.contains("laterite-ags4-wasm"),
            "wasm cert must stamp the unified engine identity, got: {json}"
        );
    }

    /// The browser used to run an ERRORS-ONLY validation and then write `warnings: 0,
    /// fyi: 0` into the stamp — two tiers it had never measured, on a claim any other
    /// surface would have believed. Every tier a wasm cert names, it looked at.
    #[test]
    fn a_browser_minted_certificate_measured_every_tier_it_names() {
        let json = certify(
            CLEAN_FIXTURE,
            None,
            None,
            "2020-01-01T00:00:00Z".to_string(),
            None,
            false,
        )
        .unwrap_or_else(|_| panic!("a clean minimal AGS4 file must certify"));
        let v: serde_json::Value = serde_json::from_str(&json).expect("the cert is JSON");
        for tier in ["errors", "warnings", "fyi"] {
            assert_eq!(
                v["validation"][tier]["state"], "measured",
                "the {tier} tier must be MEASURED, not asserted: {json}"
            );
            assert_eq!(v["validation"][tier]["count"], 0, "{tier}");
        }
    }

    /// The sandbox has no filesystem, and the stamp has no field in which to pretend
    /// otherwise: nothing a browser mints can claim Rule 20's on-disk half ran.
    #[test]
    fn a_browser_minted_certificate_cannot_claim_a_world_check() {
        let json = certify(
            CLEAN_FIXTURE,
            None,
            None,
            "2020-01-01T00:00:00Z".to_string(),
            None,
            false,
        )
        .unwrap_or_else(|_| panic!("a clean minimal AGS4 file must certify"));
        assert!(
            !json.contains("check_files"),
            "the stamp must carry no world claim at all: {json}"
        );
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
        assert_eq!(resolve_encoding(None).unwrap().name(), "UTF-8");
        assert_eq!(resolve_encoding(Some("")).unwrap().name(), "UTF-8");
        assert_eq!(resolve_encoding(Some("utf-8")).unwrap().name(), "UTF-8");
        for label in ["windows-1252", "CP1252", "latin1", "ISO-8859-1"] {
            assert_eq!(
                resolve_encoding(Some(label)).unwrap().name(),
                "windows-1252",
                "{label}"
            );
        }
    }

    /// An unknown label is an ERROR here, as it always was on Python.
    ///
    /// This assertion used to say the opposite — "an unknown label falls back to
    /// UTF-8 (lossy), not an error" — which is how the bug survived: it was not an
    /// oversight, it was *codified*. But the fallback is not lossy, it is silent:
    /// `C3 A9` decodes cleanly as `é` in UTF-8 and `Ã©` in cp1252, so a caller who
    /// typo'd their label got the wrong text and no error at all. `apply_fixes` would
    /// then rewrite the file from that mis-decode.
    #[test]
    fn an_unknown_encoding_label_is_an_error_not_a_fallback() {
        assert!(resolve_encoding(Some("not-a-charset")).is_err());
        assert!(resolve_encoding(Some("cp1252x")).is_err());
        // ...and the label is named, so the caller can see their typo.
        assert!(
            resolve_encoding(Some("cp1252x"))
                .unwrap_err()
                .contains("cp1252x")
        );
    }

    #[test]
    fn classify_collapses_notfound_and_io_to_io() {
        // The deliberate, allowlisted divergence from the producer: with no
        // filesystem `NotFound`/`Io` are unreachable, so both collapse to "io"
        // (the producer's `kind()` returns "not_found" for `NotFound`). Everything
        // else delegates verbatim. Pins the divergence in-crate.
        assert_eq!(classify(&ValidatorError::NotFound("x".into())).0, "io");
        assert_eq!(
            classify(&ValidatorError::Io {
                path: "x".into(),
                source: std::io::Error::other("x"),
            })
            .0,
            "io"
        );
        assert_eq!(classify(&ValidatorError::NotAgs4("x".into())).0, "not_ags4");
    }

    #[test]
    fn resolve_dict_override_accepts_every_bundled_edition() {
        use laterite_ags4_validator::DictVersion;
        for ed in DictVersion::ALL {
            assert!(
                resolve_dict_override(Some(ed.as_str())).is_ok(),
                "bundled edition {} must resolve",
                ed.as_str()
            );
        }
        assert!(resolve_dict_override(Some("auto")).unwrap().is_none());
        assert!(resolve_dict_override(None).unwrap().is_none());
        // A bogus label errors, and the message lists EVERY bundled edition —
        // proving it derives from `DictVersion::ALL`, not a stale hand-list.
        let err = resolve_dict_override(Some("9.9")).unwrap_err();
        for ed in DictVersion::ALL {
            assert!(
                err.contains(ed.as_str()),
                "message must list {}",
                ed.as_str()
            );
        }
    }

    #[test]
    fn byte_0xe9_is_replacement_under_utf8_but_e_acute_under_cp1252() {
        let data = [b'a', 0xE9, b'b']; // 0xE9 = 'é' in cp1252, invalid UTF-8
        let (utf8, _, had_errors) = resolve_encoding(Some("utf-8")).unwrap().decode(&data);
        assert!(had_errors, "0xE9 is not valid UTF-8");
        assert!(
            utf8.contains('\u{FFFD}'),
            "lossy decode inserts U+FFFD: {utf8:?}"
        );
        let (cp, _, had) = resolve_encoding(Some("windows-1252"))
            .unwrap()
            .decode(&data);
        assert!(!had);
        assert_eq!(cp, "aéb");
    }

    #[test]
    fn apply_fixes_encoding_path_transcodes_cp1252_to_utf8() {
        // Mirror apply_fixes' encoding pipeline (decode → apply → into_bytes)
        // without the wasm-bindgen JsValue: a cp1252 0xE9 byte must come back as
        // the UTF-8 encoding of 'é' (0xC3 0xA9), even with no fixes selected.
        let data = [b'a', 0xE9, b'b'];
        let encoding = resolve_encoding(Some("windows-1252")).unwrap();
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

    #[test]
    fn arrow_ipc_keys_match_the_shared_golden_and_default_strips() {
        // SAME fixture + golden UUIDv8s as the Python (test_content_keys.py) and
        // Node (p3-content-keys.test.ts) tests — the ids come from the ONE shared
        // keychain, so matching here proves the wasm produces byte-identical keys
        // (a cross-surface parity check, ahead of Phase 6's full proof). (#303)
        const SRC: &[u8] = b"\"GROUP\",\"PROJ\"\r\n\"HEADING\",\"PROJ_ID\"\r\n\"UNIT\",\"\"\r\n\"TYPE\",\"ID\"\r\n\"DATA\",\"P1\"\r\n\
\"GROUP\",\"LOCA\"\r\n\"HEADING\",\"LOCA_ID\",\"PROJ_ID\"\r\n\"UNIT\",\"\",\"\"\r\n\"TYPE\",\"ID\",\"ID\"\r\n\"DATA\",\"BH1\",\"P1\"\r\n";
        let ds = ParsedDataset {
            parsed: parse_bytes(SRC, encoding_rs::UTF_8).expect("parses"),
        };

        // First-row string cell of `col` in an IPC stream, or None (missing col / null).
        let first = |ipc: &[u8], col: &str| -> Option<String> {
            let mut r =
                arrow::ipc::reader::StreamReader::try_new(std::io::Cursor::new(ipc.to_vec()), None)
                    .unwrap();
            let batch = r.next().unwrap().unwrap();
            let i = batch.schema().index_of(col).ok()?;
            let a = batch
                .column(i)
                .as_any()
                .downcast_ref::<StringArray>()
                .unwrap();
            if a.is_null(0) {
                None
            } else {
                Some(a.value(0).to_string())
            }
        };

        // keys=true → the golden UUIDv8s; child._parent_id links to parent._id;
        // a root group's _parent_id is NULL.
        let proj = ds
            .arrow_ipc("PROJ", Some(true), None)
            .ok()
            .expect("PROJ keyed");
        let loca = ds
            .arrow_ipc("LOCA", Some(true), None)
            .ok()
            .expect("LOCA keyed");
        assert_eq!(
            first(&proj, "_id").as_deref(),
            Some("ac30a95d-e0ca-85f9-83c8-37a64af2762b"),
        );
        assert_eq!(
            first(&loca, "_id").as_deref(),
            Some("a7025a6f-d9b8-83b6-8fad-81c0c744edbc"),
        );
        assert_eq!(
            first(&loca, "_parent_id").as_deref(),
            Some("ac30a95d-e0ca-85f9-83c8-37a64af2762b"),
        );
        assert_eq!(first(&proj, "_parent_id"), None);

        // The default (no keys) strips: a plain frame carries no `_id` column.
        let plain = ds.arrow_ipc("PROJ", None, None).ok().expect("PROJ plain");
        assert!(
            first(&plain, "_id").is_none(),
            "default arrow_ipc must not carry _id",
        );
    }
}
