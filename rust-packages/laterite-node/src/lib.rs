//! Node-API (napi-rs) bindings for the laterite AGS4 engine — the Node
//! analog of `laterite-py`. Re-expresses the engine surface through `#[napi]`:
//! parse → typed **Arrow IPC `Buffer`** per group (the Node analog of
//! laterite-py's pyo3-arrow capsule, exactly what `ags4-wasm` frames for the
//! browser), validate, emit. The TS `laterite` package layers the high-level
//! API on top. napi auto-camelCases names (`table_ipc` → `tableIpc`).

use std::io::Cursor;
use std::path::Path;

use ags4_validator::dict::Dictionary;
use ags4_validator::findings::{Findings, Severity};
use ags4_validator::parse::{ParsedFile, parse_file_with_encoding, parse_str};
use ags4_validator::{
    CheckOptions, DictVersion, ValidatorError, check_file_with_dict, resolve_dict_version, rules,
    tran_ags_of,
};
use ags5_types::sql_type;
use arrow::ipc::reader::StreamReader;
use arrow::ipc::writer::StreamWriter;
use napi::bindgen_prelude::*;
use napi_derive::napi;
use serde_json::{Map, Value};

mod ags_types_fns;
mod transport_fns;

// --- error protocol -----------------------------------------------------
//
// The Node analog of laterite-py's `{ok:false, error_kind, exit_code}` failure
// dict + `_errors.py::raise_for`. Hard failures (missing / not-AGS4 / bad
// edition) carry a `kind` + `exit_code` so the TS layer maps them to the right
// `Ags4Error` subclass WITHOUT brittle message-matching — byte-faithful to the
// `ags4-check` exit codes (3 not-found/io, 4 not-utf8/not-ags4/unsupported-
// edition, 5 bad-dict/bad-args). `runCheck` returns the failure as data (an
// object, mirroring Python's dict); `parseArrow` returns a `Reading` *handle*,
// so it can't carry an `{ok}` field and instead THROWS this — a `\u{1f}`
// (unit-separator) delimited `kind␟code␟message` the TS `fromNativeError`
// recovers.

const SEP: char = '\u{1f}';

/// `(exit_code, error_kind)` for a `ValidatorError` — mirrors laterite-py's
/// `map_err`. The message is the error's `Display`.
fn classify(e: &ValidatorError) -> (i32, &'static str) {
    match e {
        ValidatorError::NotFound(_) => (3, "not_found"),
        ValidatorError::Io { .. } => (3, "io"),
        ValidatorError::NotUtf8(_) => (4, "not_utf8"),
        ValidatorError::NotAgs4(_) => (4, "not_ags4"),
        ValidatorError::UnsupportedEdition { .. } => (4, "unsupported_edition"),
        ValidatorError::BadDict { .. } => (5, "bad_dict"),
    }
}

/// A `ValidatorError` as a thrown napi error (for the handle-returning
/// `parseArrow`): `kind␟code␟message`, recovered by the TS `fromNativeError`.
fn thrown(e: ValidatorError) -> Error {
    let (code, kind) = classify(&e);
    Error::from_reason(format!("{kind}{SEP}{code}{SEP}{e}"))
}

/// The crate version.
#[napi]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

// --- parse → typed Arrow ------------------------------------------------

/// Per-group schema — parallel arrays, one entry per heading.
#[napi(object)]
pub struct GroupMeta {
    pub headings: Vec<String>,
    pub units: Vec<String>,
    /// AGS TYPE codes from the file's TYPE row (e.g. "2DP", "DT", "ID").
    pub types: Vec<String>,
    /// The SQL/DuckDB column type each heading lands as ("DOUBLE", "BIGINT",
    /// "TIMESTAMP", "VARCHAR", …).
    pub sql_types: Vec<String>,
    /// 1-indexed source line of each DATA row (parallel to the group's rows).
    pub line_numbers: Vec<u32>,
}

/// A parsed AGS4 file held native-side — the Node analog of laterite-py's
/// `Reading` handle (and `ags4-wasm`'s `ParsedDataset`). Each group's typed
/// `RecordBatch` is built lazily on `tableIpc(code)` and dropped after the
/// bytes are returned, so peak residency is one batch.
#[napi]
pub struct Reading {
    parsed: ParsedFile,
}

#[napi]
impl Reading {
    /// Group codes in file order (the order to load tables in).
    #[napi]
    pub fn group_codes(&self) -> Vec<String> {
        self.parsed.group_order.clone()
    }

    /// The file's `TRAN_AGS` edition string, if present.
    #[napi(getter)]
    pub fn tran_ags(&self) -> Option<String> {
        tran_ags_of(&self.parsed)
    }

    /// `{headings, units, types, sqlTypes}` for one group, or `null` if the
    /// code isn't present. No Arrow built — cheap metadata only.
    #[napi]
    pub fn meta(&self, code: String) -> Option<GroupMeta> {
        let group = self.parsed.groups.get(&code)?;
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
        Some(GroupMeta {
            headings: group.headings.clone(),
            units: (0..n)
                .map(|i| group.units.get(i).cloned().unwrap_or_default())
                .collect(),
            sql_types: types.iter().map(|t| sql_type(t).to_string()).collect(),
            types,
            line_numbers: group.rows.iter().map(|r| r.line).collect(),
        })
    }

    /// One group's rows as an Arrow **IPC stream** (`Buffer`), columns already
    /// correctly typed. The Node analog of the pyo3-arrow capsule: the typed
    /// columns come from the one shared emitter (`ags5_types::arrow_cols`), the
    /// SAME casting Python/wasm use — so a file types byte-identically across
    /// hosts. Returns `null` if the code isn't in the file.
    #[napi]
    pub fn table_ipc(&self, code: String) -> Result<Option<Buffer>> {
        let Some(group) = self.parsed.groups.get(&code) else {
            return Ok(None);
        };
        let batch = ags5_types::arrow_cols::build_record_batch(
            &group.headings,
            &group.types,
            group.rows.len(),
            |col, row| {
                group
                    .rows
                    .get(row)
                    .and_then(|r| r.values.get(col))
                    .map(String::as_str)
            },
        )
        .map_err(|e| Error::from_reason(format!("arrow batch for {code}: {e}")))?;
        let schema = batch.schema();
        let mut buf = Vec::new();
        let mut w = StreamWriter::try_new(&mut buf, &schema)
            .map_err(|e| Error::from_reason(format!("arrow ipc for {code}: {e}")))?;
        w.write(&batch)
            .map_err(|e| Error::from_reason(format!("arrow ipc for {code}: {e}")))?;
        w.finish()
            .map_err(|e| Error::from_reason(format!("arrow ipc for {code}: {e}")))?;
        drop(w);
        Ok(Some(buf.into()))
    }

    /// Re-emit byte-faithful AGS4 text from the retained parse (the raw DATA
    /// values, unchanged). = laterite-py's `Reading::emit`.
    #[napi]
    pub fn emit(&self) -> Result<String> {
        // EmitGroup borrows `&str`, so build an owned mirror first. Pad each
        // DATA row to the heading count (a ragged row fills its tail with "").
        struct Owned {
            code: String,
            headings: Vec<String>,
            units: Vec<String>,
            types: Vec<String>,
            rows: Vec<Vec<String>>,
        }
        let owned: Vec<Owned> = self
            .parsed
            .group_order
            .iter()
            .filter_map(|code| {
                let g = self.parsed.groups.get(code)?;
                let n = g.headings.len();
                Some(Owned {
                    code: code.clone(),
                    headings: g.headings.clone(),
                    units: g.units.clone(),
                    types: g.types.clone(),
                    rows: g
                        .rows
                        .iter()
                        .map(|r| {
                            (0..n)
                                .map(|i| r.values.get(i).cloned().unwrap_or_default())
                                .collect()
                        })
                        .collect(),
                })
            })
            .collect();
        let groups: Vec<ags4_emit::EmitGroup<'_>> = owned
            .iter()
            .map(|o| ags4_emit::EmitGroup {
                code: &o.code,
                headings: o.headings.iter().map(String::as_str).collect(),
                units: o.units.iter().map(String::as_str).collect(),
                types: o.types.iter().map(String::as_str).collect(),
                rows: o.rows.clone(),
            })
            .collect();
        let mut buf = Vec::new();
        ags4_emit::write_ags4(&mut buf, &groups).map_err(|e| Error::from_reason(e.to_string()))?;
        String::from_utf8(buf).map_err(|e| Error::from_reason(format!("emit utf8: {e}")))
    }
}

/// Parse an AGS4 file (`path`) or in-memory `text` into a `Reading` handle.
/// `encoding`: `"utf-8"` (default) / `"windows-1252"` / a label. Throws the
/// classified `kind␟code␟message` (see the error-protocol note) on bad input.
#[napi]
pub fn parse_arrow(
    path: Option<String>,
    text: Option<String>,
    encoding: Option<String>,
) -> Result<Reading> {
    // `text` wins when both are given (matches P1 / laterite-py's source split).
    // Text is already a decoded UTF-8 `String`, so `parse_str`; a path reads
    // with the requested encoding via the engine (which classifies IO errors).
    let parsed = if let Some(t) = text {
        parse_str(&t).map_err(thrown)?
    } else if let Some(p) = path {
        let enc = resolve_encoding(encoding.as_deref());
        parse_file_with_encoding(Path::new(&p), enc).map_err(thrown)?
    } else {
        return Err(Error::from_reason(format!(
            "bad_args{SEP}5{SEP}provide `path` or `text`"
        )));
    };
    Ok(Reading { parsed })
}

// --- validate -----------------------------------------------------------

/// One rule violation (omitting `severity` ⇒ error, matching the engine).
#[napi(object)]
pub struct Finding {
    pub rule: String,
    pub line: Option<u32>,
    pub group: String,
    pub desc: String,
    pub severity: Option<String>,
}

/// The validation report — the Node mirror of laterite-py's `run_check` dict.
/// `ok` is **false only for un-validatable input** (the TS `raiseFor` raises
/// then); rule *violations* come back in `findings` with `ok:true`. `Report`'s
/// `isValid` is the separate `count == 0`. `json`/`ndjson` are byte-identical
/// to `ags4-check --json` / `--ndjson`.
#[napi(object)]
pub struct ValidationReport {
    pub ok: bool,
    /// Set (with `error`) only when `ok` is false — the failure kind the TS
    /// `raiseFor` maps to an exception (`not_ags4`, `unsupported_edition`, …).
    pub error_kind: Option<String>,
    pub error: Option<String>,
    /// Mirrors the `ags4-check` binary: 0 valid / 1 findings on success;
    /// 3 not-found/io, 4 not-utf8/not-ags4/bad-edition, 5 bad-dict on failure.
    pub exit_code: i32,
    pub file: String,
    pub dict_version: String,
    pub resolution: String,
    pub count: u32,
    pub findings: Vec<Finding>,
    pub json: String,
    pub ndjson: String,
}

impl ValidationReport {
    /// The `{ok:false}` failure report (success fields defaulted) — the data
    /// analog of laterite-py's `err_dict`.
    fn failure(kind: &str, exit_code: i32, message: String) -> Self {
        ValidationReport {
            ok: false,
            error_kind: Some(kind.to_string()),
            error: Some(message),
            exit_code,
            file: String::new(),
            dict_version: String::new(),
            resolution: String::new(),
            count: 0,
            findings: Vec::new(),
            json: String::new(),
            ndjson: String::new(),
        }
    }
}

/// `{file, findings:{ "AGS Format Rule N":[{line,group,desc}] }}` pretty-JSON —
/// byte-identical to `ags4-check --json` (ported verbatim from laterite-py's
/// `findings_json`; `preserve_order` keeps the key order stable).
fn findings_json(file: &str, found: &Findings) -> String {
    let mut fmap = Map::new();
    for (rule, items) in found {
        let arr: Vec<Value> = items
            .iter()
            .map(|f| serde_json::to_value(f).unwrap_or(Value::Null))
            .collect();
        fmap.insert(rule.clone(), Value::Array(arr));
    }
    let mut root = Map::new();
    root.insert("file".into(), Value::from(file.to_string()));
    root.insert("findings".into(), Value::Object(fmap));
    serde_json::to_string_pretty(&Value::Object(root)).unwrap_or_default()
}

/// One flat `{rule, …}` JSON object per finding per line — byte-identical to
/// `ags4-check --ndjson` (ported verbatim from laterite-py's `findings_ndjson`).
fn findings_ndjson(found: &Findings) -> String {
    let mut s = String::new();
    for (rule, items) in found {
        for f in items {
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

/// Run the validator from a `path` or in-memory `text` — mirrors laterite-py's
/// `validate` helper (path → `check_file_with_dict` so Rule 20's on-disk half
/// and the encoding are handled identically; text → `parse_str` + resolve +
/// `run_all`). Returns `(file, dict_version, resolution, findings)` or the
/// `(exit_code, error_kind, message)` of a hard failure.
#[allow(clippy::type_complexity)]
fn validate_inner(
    path: Option<&str>,
    text: Option<&str>,
    forced: Option<DictVersion>,
    opts: CheckOptions,
) -> std::result::Result<(String, String, String, Findings), (i32, &'static str, String)> {
    let map = |e: ValidatorError| {
        let (code, kind) = classify(&e);
        (code, kind, e.to_string())
    };
    if let Some(t) = text {
        let parsed = parse_str(t).map_err(map)?;
        let (dv, res) =
            resolve_dict_version(forced, tran_ags_of(&parsed).as_deref()).map_err(map)?;
        let dict = Dictionary::bundled(dv);
        let mut found = Findings::new();
        rules::run_all(&parsed, &dict, &opts, None, &mut found);
        Ok((
            "<text>".to_string(),
            dv.as_str().to_string(),
            res.as_str().to_string(),
            found,
        ))
    } else if let Some(p) = path {
        let (found, dv, res) = check_file_with_dict(Path::new(p), &opts).map_err(map)?;
        Ok((
            p.to_string(),
            dv.as_str().to_string(),
            res.as_str().to_string(),
            found,
        ))
    } else {
        Err((5, "bad_args", "provide `path` or `text`".to_string()))
    }
}

/// Validate an AGS4 file (`path`) or `text` against the AGS4 rules. `dict_version`
/// `None`/`"auto"` auto-detects from `TRAN_AGS`, else forces an edition. Returns
/// the `{ok:false}` failure report (not a throw) for un-validatable input.
#[napi]
pub fn run_check(
    path: Option<String>,
    text: Option<String>,
    dict_version: Option<String>,
    include_warnings: Option<bool>,
    include_fyi: Option<bool>,
    check_files: Option<bool>,
    encoding: Option<String>,
) -> Result<ValidationReport> {
    let forced = match resolve_edition(dict_version.as_deref()) {
        Ok(v) => v,
        Err(msg) => return Ok(ValidationReport::failure("bad_dict", 5, msg)),
    };
    let opts = CheckOptions {
        dict_version: forced,
        include_warnings: include_warnings.unwrap_or(false),
        include_fyi: include_fyi.unwrap_or(false),
        check_files: check_files.unwrap_or(false),
        encoding: resolve_encoding(encoding.as_deref()),
        ..CheckOptions::default()
    };
    let (file, dv, res, found) =
        match validate_inner(path.as_deref(), text.as_deref(), forced, opts) {
            Ok(t) => t,
            Err((code, kind, msg)) => return Ok(ValidationReport::failure(kind, code, msg)),
        };
    let findings: Vec<Finding> = found
        .iter()
        .flat_map(|(rule, items)| {
            items.iter().map(move |f| Finding {
                rule: rule.clone(),
                line: f.line,
                group: f.group.clone(),
                desc: f.desc.clone(),
                severity: match f.severity {
                    Severity::Error => None,
                    s => Some(format!("{s:?}").to_lowercase()),
                },
            })
        })
        .collect();
    let count = findings.len() as u32;
    Ok(ValidationReport {
        ok: true,
        error_kind: None,
        error: None,
        exit_code: if count == 0 { 0 } else { 1 },
        json: findings_json(&file, &found),
        ndjson: findings_ndjson(&found),
        file,
        dict_version: dv,
        resolution: res,
        count,
        findings,
    })
}

// --- emit (data → AGS4) -------------------------------------------------

/// One group of columnar input — its code + an Arrow IPC stream (`Buffer`)
/// whose column names are the AGS headings.
#[napi(object)]
pub struct GroupIpc {
    pub code: String,
    pub ipc: Buffer,
}

/// The emit result. `bytes` is the AGS4 document; `findingsJson` is the
/// validator's `{rule:[…]}` map on the output; `fixesApplied` counts safe fixes.
#[napi(object)]
pub struct EmitResult {
    pub bytes: Buffer,
    pub findings_json: String,
    pub fixes_applied: u32,
}

/// Build valid AGS4 from per-group **Arrow IPC** streams (the columnar
/// producer; the read boundary reversed). = `ags4-wasm`'s `to_ags4_ipc`.
#[napi]
pub fn emit_ags4_from_ipc(
    groups: Vec<GroupIpc>,
    edition: Option<String>,
    mode: Option<String>,
) -> Result<EmitResult> {
    let opts = ags4_emit::EmitOpts {
        mode: resolve_mode(mode.as_deref())?,
        edition: resolve_edition(edition.as_deref())
            .map_err(Error::from_reason)?
            .unwrap_or(DictVersion::V4_1_1),
    };
    let mut inputs = Vec::with_capacity(groups.len());
    for g in groups {
        inputs.push(group_from_ipc(g.code, &g.ipc)?);
    }
    let res =
        ags4_emit::emit_ags4(&inputs, &opts).map_err(|e| Error::from_reason(e.to_string()))?;
    let findings_json = serde_json::to_string(&res.findings).unwrap_or_else(|_| "{}".into());
    Ok(EmitResult {
        bytes: res.bytes.into(),
        findings_json,
        fixes_applied: res.fixes_applied as u32,
    })
}

fn group_from_ipc(code: String, bytes: &[u8]) -> Result<ags4_emit::GroupInput> {
    let reader = StreamReader::try_new(Cursor::new(bytes), None)
        .map_err(|e| Error::from_reason(format!("arrow ipc: {e}")))?;
    let schema = reader.schema();
    let mut batches = Vec::new();
    for b in reader {
        batches.push(b.map_err(|e| Error::from_reason(format!("arrow ipc batch: {e}")))?);
    }
    Ok(ags4_emit::group_from_arrow(code, schema.as_ref(), &batches))
}

// --- helpers ------------------------------------------------------------

fn resolve_encoding(label: Option<&str>) -> &'static encoding_rs::Encoding {
    match label.map(|s| s.trim().to_ascii_lowercase()).as_deref() {
        None | Some("") | Some("utf-8") | Some("utf8") => encoding_rs::UTF_8,
        Some("windows-1252") | Some("cp1252") | Some("latin1") => encoding_rs::WINDOWS_1252,
        Some(other) => {
            encoding_rs::Encoding::for_label(other.as_bytes()).unwrap_or(encoding_rs::UTF_8)
        }
    }
}

/// Plain-`String` error (not napi) so `run_check` can surface a bad edition as
/// a `{ok:false}` failure report while `emit_ags4_from_ipc` throws it.
fn resolve_edition(s: Option<&str>) -> std::result::Result<Option<DictVersion>, String> {
    match s.map(str::trim) {
        None | Some("") | Some("auto") => Ok(None),
        Some("4.0.3") => Ok(Some(DictVersion::V4_0_3)),
        Some("4.0.4") => Ok(Some(DictVersion::V4_0_4)),
        Some("4.1") => Ok(Some(DictVersion::V4_1)),
        Some("4.1.1") => Ok(Some(DictVersion::V4_1_1)),
        Some("4.2") => Ok(Some(DictVersion::V4_2)),
        Some(o) => Err(format!(
            "unknown dict_version {o:?}; expected auto|4.0.3|4.0.4|4.1|4.1.1|4.2"
        )),
    }
}

fn resolve_mode(s: Option<&str>) -> Result<ags4_emit::EmitMode> {
    match s.map(str::trim).map(str::to_ascii_lowercase).as_deref() {
        None | Some("") | Some("autofix") => Ok(ags4_emit::EmitMode::AutoFix),
        Some("report") => Ok(ags4_emit::EmitMode::Report),
        Some("strict") => Ok(ags4_emit::EmitMode::Strict),
        Some(o) => Err(Error::from_reason(format!(
            "unknown mode {o:?}; expected autofix|report|strict"
        ))),
    }
}
