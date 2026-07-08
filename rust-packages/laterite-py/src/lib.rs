//! PyO3 module `laterite._laterite_native`.
//!
//! Thin boundary: every bit of AGS4 logic lives in `laterite-ags4-validator`
//! (clean-room, parity-tested) or the local `emit` module. The Python
//! side (`laterite/__init__.py`, `compat.py`, `_cli.py`) builds the
//! python-ags4-shaped dict from the primitives `parse_primitives`
//! returns; the read path's frames come born-typed from the Arrow
//! tables `parse_arrow` hands over (pyo3-arrow capsule, zero-copy).
//!
//! Two error-JSON shapes are deliberately preserved (see the package
//! README): the Rust-CLI shape `{file, findings:{rule:[...]}}` is
//! built *here* with the same `serde_json` (`preserve_order`) calls
//! the `lat` binary uses, so `--json`/`--ndjson` are
//! byte-faithful; the python-ags4 `check_file` dict (with
//! `Metadata`/`Summary`) is assembled in `laterite/compat.py`.

use std::path::Path;

use laterite_ags4_core::error::CliError;
use laterite_ags4_core::index::{ENGINE_IDENTITY, Sidecar as CoreSidecar, ValidationStamp};
use laterite_ags4_validator::findings::{Severity, Target};
use laterite_ags4_validator::fixes::FixRisk;
use laterite_ags4_validator::{
    CheckOptions, DictVersion, Dictionary, Findings, Fix, ValidatorError, fix_document_selective,
};
use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyDict, PyList};
use pyo3_arrow::PyTable;
use serde_json::{Map, Value};

// S3b (release/v0.1.0-prep): `ags5db_fns` moved to the separate
// `laterite-py-ags5` cdylib. Base wheel ships without DuckDB; the
// AGS5 surface is gated behind the `laterite[ags5]` extra.
mod ags_types_fns;
mod emit;
mod emit_typed;
mod excel_fns;
mod registry_fns;
mod transport_fns;
mod typed_graph;

/// Map an `laterite-ags4-core` `CliError` to a PyRuntimeError, preserving the
/// exit-code label python callers may surface in error messages.
/// Previously lived in `ags5db_fns.rs`; that module moved to the
/// AGS5 cdylib in S3b, so the base-wheel transport + excel functions
/// share this local copy instead.
pub(crate) fn map_cli_err(e: CliError) -> PyErr {
    let code = e.exit_code();
    PyRuntimeError::new_err(format!("laterite error (exit {code}): {e}"))
}

/// Map a `--dict-version` string to the optional override. `None` /
/// `"auto"` ⇒ no override (TRAN_AGS auto-pick). Unknown ⇒ `Err`.
pub(crate) fn parse_dv(s: Option<&str>) -> Result<Option<DictVersion>, String> {
    match s {
        None | Some("auto") | Some("") => Ok(None),
        Some(other) => DictVersion::from_edition(other).map(Some).ok_or_else(|| {
            format!(
                "unknown --dict-version {other:?} (expected auto|{})",
                laterite_ags4_validator::editions_joined("|")
            )
        }),
    }
}

/// (exit_code, error_kind, message) for a validator error — exit codes
/// mirror the `lat` binary exactly (3 not-found/io, 4
/// not-utf8/not-ags4/unsupported-edition, 5 bad-dict).
fn map_err(e: ValidatorError) -> (i32, String, String) {
    // Delegate to the single producers so codes/kinds can't drift.
    (e.exit_code(), e.kind().to_string(), e.to_string())
}

/// Run the validator from a path, in-memory text, or raw bytes. Returns
/// `(file, dict_version, resolution, findings)` or
/// `(exit_code, error_kind, message)`.
#[allow(clippy::type_complexity, clippy::too_many_arguments)]
fn validate(
    path: Option<&str>,
    text: Option<&str>,
    data: Option<&[u8]>,
    dvr: Option<&str>,
    warnings: bool,
    fyi: bool,
    check_files: bool,
    encoding: Option<&str>,
) -> Result<(String, String, String, Findings), (i32, String, String)> {
    let over = parse_dv(dvr).map_err(|m| (5, "bad_dict".to_string(), m))?;
    let enc = laterite_ags4_parse::resolve_encoding(encoding).ok_or_else(|| {
        (
            5,
            "bad_args".to_string(),
            format!("unknown encoding {:?}", encoding.unwrap_or("")),
        )
    })?;
    let opts = CheckOptions {
        dict_version: over,
        custom_dict: None,
        include_warnings: warnings,
        include_fyi: fyi,
        check_files,
        encoding: enc,
    };

    if let Some(p) = path {
        match laterite_ags4_validator::check_file_with_dict(Path::new(p), &opts) {
            Ok((found, dv, res)) => Ok((
                p.to_string(),
                dv.as_str().to_string(),
                res.as_str().to_string(),
                found,
            )),
            Err(e) => Err(map_err(e)),
        }
    } else if let Some(t) = text {
        let pf = laterite_ags4_parse::parse_str(t)
            .map_err(ValidatorError::from)
            .map_err(map_err)?;
        let tran = laterite_ags4_validator::tran_ags_of(&pf);
        let (dv, res) = laterite_ags4_validator::resolve_dict_version(over, tran.as_deref())
            .map_err(map_err)?;
        let dict = Dictionary::bundled(dv);
        let mut found = Findings::new();
        laterite_ags4_validator::rules::run_all(&pf, &dict, &opts, None, &mut found);
        Ok((
            "<text>".to_string(),
            dv.as_str().to_string(),
            res.as_str().to_string(),
            found,
        ))
    } else if let Some(d) = data {
        // bytes path: decode with `enc` (BOM-sniffed) then validate — the text
        // branch's twin for callers that hold raw bytes (a web backend, an
        // embedded host) with no file on disk. Same engine the wasm surface uses.
        let pf = laterite_ags4_parse::parse_bytes(d, enc)
            .map_err(ValidatorError::from)
            .map_err(map_err)?;
        let tran = laterite_ags4_validator::tran_ags_of(&pf);
        let (dv, res) = laterite_ags4_validator::resolve_dict_version(over, tran.as_deref())
            .map_err(map_err)?;
        let dict = Dictionary::bundled(dv);
        let mut found = Findings::new();
        laterite_ags4_validator::rules::run_all(&pf, &dict, &opts, None, &mut found);
        Ok((
            "<bytes>".to_string(),
            dv.as_str().to_string(),
            res.as_str().to_string(),
            found,
        ))
    } else {
        Err((
            5,
            "bad_args".to_string(),
            "one of path, text, or data is required".to_string(),
        ))
    }
}

/// Lowercase serde-name of a [`Target`] for the structured PyDict
/// (matches the JSON `target` value).
fn target_str(t: Target) -> &'static str {
    match t {
        Target::Line => "line",
        Target::Heading => "heading",
        Target::Cell => "cell",
        Target::Group => "group",
    }
}

/// Lowercase serde-name of a [`Severity`] for the structured PyDict — delegates
/// to the single producer so it can't drift from the serde token.
fn severity_str(s: Severity) -> &'static str {
    s.as_str()
}

/// `{file, findings:{ "AGS Format Rule N":[{line,group,desc}] }}` —
/// the exact `serde_json` value + insertion order the `lat`
/// binary's `json_value`/`json_string` produce. `preserve_order` (set
/// in Cargo.toml, matching the validator) keeps the key order stable.
fn findings_json(file: &str, found: &Findings) -> String {
    let mut fmap = Map::new();
    for (rule, items) in found {
        // Serialize the engine `Finding` directly — byte-faithful to the
        // CLI's `json_value`. Unset location/severity fields skip, so
        // line-only findings stay `{line,group,desc}`; migrated findings
        // additively gain the rich keys.
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

/// One flat JSON object per finding per line (NDJSON) — byte-identical
/// to the binary's `ndjson_string`.
fn findings_ndjson(found: &Findings) -> String {
    let mut s = String::new();
    for (rule, items) in found {
        for f in items {
            // `rule`-first then the serialized `Finding` body — byte-faithful
            // to the CLI's `ndjson_string`.
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

fn err_dict<'py>(
    py: Python<'py>,
    code: i32,
    kind: &str,
    msg: &str,
) -> PyResult<Bound<'py, PyDict>> {
    let d = PyDict::new(py);
    d.set_item("ok", false)?;
    d.set_item("error_kind", kind)?;
    d.set_item("error", msg)?;
    d.set_item("exit_code", code)?;
    Ok(d)
}

/// Validate `path` or `text`. Returns a dict the Python layer turns
/// into a `Report` / a python-ags4-shaped `check_file` dict / CLI
/// output. On un-validatable input returns `{ok:false, error_kind,
/// error, exit_code}` (the Python layer raises the mapped exception;
/// the CLI uses `exit_code` directly).
#[pyfunction]
#[pyo3(signature = (path=None, text=None, data=None, dict_version=None, include_warnings=false, include_fyi=false, check_files=false, encoding=None))]
#[allow(clippy::too_many_arguments)]
fn run_check<'py>(
    py: Python<'py>,
    path: Option<String>,
    text: Option<String>,
    data: Option<Vec<u8>>,
    dict_version: Option<String>,
    include_warnings: bool,
    include_fyi: bool,
    check_files: bool,
    encoding: Option<String>,
) -> PyResult<Bound<'py, PyDict>> {
    match validate(
        path.as_deref(),
        text.as_deref(),
        data.as_deref(),
        dict_version.as_deref(),
        include_warnings,
        include_fyi,
        check_files,
        encoding.as_deref(),
    ) {
        Err((code, kind, msg)) => err_dict(py, code, &kind, &msg),
        Ok((file, dv, res, found)) => {
            let d = PyDict::new(py);
            d.set_item("ok", true)?;
            d.set_item("file", &file)?;
            d.set_item("dict_version", dv)?;
            d.set_item("resolution", res)?;

            let mut count = 0usize;
            let items = PyList::empty(py);
            for (rule, fs) in &found {
                for f in fs {
                    count += 1;
                    let o = PyDict::new(py);
                    o.set_item("rule", rule)?;
                    o.set_item("line", f.line)?;
                    o.set_item("group", &f.group)?;
                    o.set_item("desc", &f.desc)?;
                    // Additively surface the rich location/severity fields
                    // when set away from default — existing readers keyed on
                    // line/group/desc are unaffected. Target/Severity render
                    // as their lowercase serde names; char_span as a 2-tuple.
                    let loc = &f.location;
                    if loc.target != Target::Line {
                        o.set_item("target", target_str(loc.target))?;
                    }
                    if let Some(fi) = loc.field_index {
                        o.set_item("field_index", fi)?;
                    }
                    if let Some(h) = &loc.heading {
                        o.set_item("heading", h)?;
                    }
                    if let Some(dr) = loc.data_row {
                        o.set_item("data_row", dr)?;
                    }
                    if let Some((a, b)) = loc.char_span {
                        o.set_item("char_span", (a, b))?;
                    }
                    if f.severity != Severity::Error {
                        o.set_item("severity", severity_str(f.severity))?;
                    }
                    items.append(o)?;
                }
            }
            d.set_item("count", count)?;
            d.set_item("exit_code", if count == 0 { 0 } else { 1 })?;
            d.set_item("findings", items)?;
            d.set_item("json", findings_json(&file, &found))?;
            d.set_item("ndjson", findings_ndjson(&found))?;
            Ok(d)
        }
    }
}

/// Serialise a slice of [`Fix`] into the Python `applied`-ledger shape — one
/// `{kind, label, rule, line, risk}` dict per fix — shared by `fix()`'s
/// `FixResult.applied` and `build_ags4`'s `BuildResult.applied` so both surfaces
/// present an identical record (#294 F#7).
pub(crate) fn fixes_to_pylist<'py>(py: Python<'py>, fixes: &[Fix]) -> PyResult<Bound<'py, PyList>> {
    let list = PyList::empty(py);
    for f in fixes {
        let o = PyDict::new(py);
        let kind = serde_json::to_value(f.kind)
            .ok()
            .and_then(|v| v.as_str().map(str::to_string))
            .unwrap_or_default();
        o.set_item("kind", kind)?;
        o.set_item("label", &f.label)?;
        o.set_item("rule", &f.rule)?;
        o.set_item("line", f.line)?;
        o.set_item(
            "risk",
            match f.risk {
                FixRisk::Safe => "safe",
                FixRisk::Risky => "risky",
            },
        )?;
        list.append(o)?;
    }
    Ok(list)
}

/// Headless one-shot mechanical repair of a delivered AGS4 file (the same engine
/// the browser fix UI uses, applied without a UI). Reads from path/text/data,
/// computes fixes against the file's own findings, applies the *safe* set (plus
/// the intent-guessing *risky* set when `include_risky`), and re-validates the
/// result. Returns `(fixed_bytes, residual_findings, applied_fixes, dict_version,
/// resolution)` or the `(exit_code, kind, msg)` error triple. Mirrors the
/// AutoFix path in `laterite-ags4-emit::emit`, but on a file's own bytes rather
/// than freshly-built data.
#[allow(clippy::type_complexity, clippy::too_many_arguments)]
fn fix_core(
    path: Option<&str>,
    text: Option<&str>,
    data: Option<&[u8]>,
    dvr: Option<&str>,
    encoding: Option<&str>,
    include_risky: bool,
    only: Option<&[String]>,
    exclude: &[String],
) -> Result<(Vec<u8>, Findings, Vec<Fix>, String, String, usize), (i32, String, String)> {
    let over = parse_dv(dvr).map_err(|m| (5, "bad_dict".to_string(), m))?;
    let enc = laterite_ags4_parse::resolve_encoding(encoding).ok_or_else(|| {
        (
            5,
            "bad_args".to_string(),
            format!("unknown encoding {:?}", encoding.unwrap_or("")),
        )
    })?;
    // The source bytes — fix needs the raw document to apply byte/char edits to,
    // so unlike `validate` we always materialise bytes (a path is read here).
    let raw: Vec<u8> = if let Some(p) = path {
        std::fs::read(p).map_err(|e| {
            let kind = if e.kind() == std::io::ErrorKind::NotFound {
                "not_found"
            } else {
                "io"
            };
            (3, kind.to_string(), format!("{p}: {e}"))
        })?
    } else if let Some(d) = data {
        d.to_vec()
    } else if let Some(t) = text {
        t.as_bytes().to_vec()
    } else {
        return Err((
            5,
            "bad_args".to_string(),
            "one of path, text, or data is required".to_string(),
        ));
    };

    // The orchestration (parse → run → compute → apply → re-validate) lives once,
    // in the validator's `fix_document`; the CLI shares it.
    // The residual re-validation tier matches `validate()`'s default (errors +
    // warnings) so a fix reports what it left behind at the same tier the user
    // sees on validate — was errors + FYI, which both under- and over-reported
    // vs the other surfaces (#294 Batch C).
    let opts = CheckOptions {
        dict_version: over,
        custom_dict: None,
        include_warnings: true,
        include_fyi: false,
        check_files: false,
        encoding: enc,
    };
    let out = fix_document_selective(&raw, &opts, include_risky, only, exclude).map_err(map_err)?;
    Ok((
        out.fixed,
        out.residual,
        out.applied,
        out.dict_version.as_str().to_string(),
        out.resolution.as_str().to_string(),
        out.risky_available,
    ))
}

/// Headless mechanical fix of `path`/`text`/`data`. Returns a dict the Python
/// `fix()` turns into a `FixResult` (`fixed` bytes, `findings_json` residual
/// `{rule:[…]}`, the `applied` fix list, `fixes_applied` count). On
/// un-fixable input returns the same `{ok:false, …}` shape as `run_check`, so
/// the Python layer raises the mapped exception.
#[pyfunction]
#[pyo3(signature = (path=None, text=None, data=None, dict_version=None, encoding=None, include_risky=false, only=None, exclude=None))]
#[allow(clippy::too_many_arguments)]
fn fix_file<'py>(
    py: Python<'py>,
    path: Option<String>,
    text: Option<String>,
    data: Option<Vec<u8>>,
    dict_version: Option<String>,
    encoding: Option<String>,
    include_risky: bool,
    only: Option<Vec<String>>,
    exclude: Option<Vec<String>>,
) -> PyResult<Bound<'py, PyDict>> {
    let exclude = exclude.unwrap_or_default();
    match fix_core(
        path.as_deref(),
        text.as_deref(),
        data.as_deref(),
        dict_version.as_deref(),
        encoding.as_deref(),
        include_risky,
        only.as_deref(),
        &exclude,
    ) {
        Err((code, kind, msg)) => err_dict(py, code, &kind, &msg),
        Ok((bytes, residual, applied, dv, res, risky_available)) => {
            let d = PyDict::new(py);
            d.set_item("ok", true)?;
            d.set_item("fixed", PyBytes::new(py, &bytes))?;
            d.set_item("dict_version", dv)?;
            d.set_item("resolution", res)?;
            d.set_item("risky_available", risky_available)?;
            d.set_item(
                "findings_json",
                serde_json::to_string(&residual).unwrap_or_else(|_| "{}".into()),
            )?;
            d.set_item("fixes_applied", applied.len())?;
            d.set_item("applied", fixes_to_pylist(py, &applied)?)?;
            Ok(d)
        }
    }
}

/// The rule catalogue (title / severity / fixable / cited observations per rule)
/// as the engine's gated `rules_meta.json` — the single source
/// `laterite.list_rules()` parses. Read-only; no file argument; the JSON is
/// compile-time-embedded so this never touches disk.
#[pyfunction]
fn list_rules() -> &'static str {
    laterite_ags4_validator::rule_metadata_json()
}

/// The KEY-aware/type-aware revision diff of two AGS4 documents (`a` baseline,
/// `b` revision), serialised as a `RevisionDelta` JSON string — the same diff
/// the browser's Tools tab produces, via the shared `laterite-ags4-diff` leaf.
/// Rows are matched by the group's dictionary KEY headings; cells are compared
/// through the typed value, so a formatting-only change (`"1.0"` → `"1.00"`) is
/// suppressed. The dictionary edition is the revision's `TRAN_AGS` (forced by
/// `dict_version`), falling back to the bundled default — like the wasm `diff()`.
fn diff_core(
    a: &[u8],
    b: &[u8],
    dvr: Option<&str>,
    encoding: Option<&str>,
) -> Result<String, (i32, String, String)> {
    let over = parse_dv(dvr).map_err(|m| (5, "bad_dict".to_string(), m))?;
    let enc = laterite_ags4_parse::resolve_encoding(encoding).ok_or_else(|| {
        (
            5,
            "bad_args".to_string(),
            format!("unknown encoding {:?}", encoding.unwrap_or("")),
        )
    })?;
    let pa = laterite_ags4_parse::parse_bytes(a, enc)
        .map_err(ValidatorError::from)
        .map_err(map_err)?;
    let pb = laterite_ags4_parse::parse_bytes(b, enc)
        .map_err(ValidatorError::from)
        .map_err(map_err)?;
    // KEY headings come from the dictionary; pick the edition from the revision
    // (b)'s TRAN_AGS (forced by dict_version), falling back to the standard.
    let tran = laterite_ags4_validator::tran_ags_of(&pb);
    let dv = laterite_ags4_validator::resolve_dict_version(over, tran.as_deref())
        .map(|(dv, _)| dv)
        .unwrap_or(laterite_ags4_validator::dict::FALLBACK);
    let dict = Dictionary::bundled(dv);
    let delta = laterite_ags4_diff::diff_parsed(&pa, &pb, &dict, None);
    Ok(serde_json::to_string(&delta).unwrap_or_else(|_| "{}".to_string()))
}

/// Compare two AGS4 documents (raw `a`/`b` bytes). Returns `{ok:true,
/// delta_json}` (the serialised `RevisionDelta`, which the Python layer parses)
/// or the `{ok:false, error_kind, exit_code, error}` failure dict.
#[pyfunction]
#[pyo3(signature = (a, b, dict_version=None, encoding=None))]
fn diff_files<'py>(
    py: Python<'py>,
    a: Vec<u8>,
    b: Vec<u8>,
    dict_version: Option<String>,
    encoding: Option<String>,
) -> PyResult<Bound<'py, PyDict>> {
    match diff_core(&a, &b, dict_version.as_deref(), encoding.as_deref()) {
        Err((code, kind, msg)) => err_dict(py, code, &kind, &msg),
        Ok(delta_json) => {
            let d = PyDict::new(py);
            d.set_item("ok", true)?;
            d.set_item("delta_json", delta_json)?;
            Ok(d)
        }
    }
}

/// Parse `path` or `text` into per-group primitives (string cells —
/// AGS4 is a text format). `headings`/`units`/`types`/row `values`
/// exclude the leading row tag (matching `ParsedGroup`); the Python
/// side prepends the literal `HEADING`/`UNIT`/`TYPE`/`DATA` when it
/// needs the python-ags4-shaped frame.
#[pyfunction]
#[pyo3(signature = (path=None, text=None, data=None, encoding=None))]
fn parse_primitives<'py>(
    py: Python<'py>,
    path: Option<String>,
    text: Option<String>,
    data: Option<Vec<u8>>,
    encoding: Option<String>,
) -> PyResult<Bound<'py, PyDict>> {
    let enc = match laterite_ags4_parse::resolve_encoding(encoding.as_deref()) {
        Some(e) => e,
        None => {
            let label = encoding.as_deref().unwrap_or("");
            return err_dict(py, 5, "bad_args", &format!("unknown encoding {label:?}"));
        }
    };
    let parsed = match (path.as_deref(), text.as_deref(), data.as_deref()) {
        (Some(p), _, _) => {
            laterite_ags4_validator::parse::parse_file_with_encoding(Path::new(p), enc)
        }
        (_, Some(t), _) => laterite_ags4_parse::parse_str(t).map_err(ValidatorError::from),
        (_, _, Some(d)) => laterite_ags4_parse::parse_bytes(d, enc).map_err(ValidatorError::from),
        _ => {
            return err_dict(py, 5, "bad_args", "one of path, text, or data is required");
        }
    };
    let pf = match parsed {
        Ok(pf) => pf,
        Err(e) => {
            let (c, k, m) = map_err(e);
            return err_dict(py, c, &k, &m);
        }
    };

    let d = PyDict::new(py);
    d.set_item("ok", true)?;
    d.set_item("group_order", pf.group_order.clone())?;
    d.set_item("tran_ags", laterite_ags4_validator::tran_ags_of(&pf))?;

    let groups = PyDict::new(py);
    for code in &pf.group_order {
        let Some(g) = pf.groups.get(code) else {
            continue;
        };
        let gd = PyDict::new(py);
        gd.set_item("group_line", g.group_line)?;
        gd.set_item("heading_line", g.heading_line)?;
        gd.set_item("unit_line", g.unit_line)?;
        gd.set_item("type_line", g.type_line)?;
        gd.set_item("headings", g.headings.clone())?;
        gd.set_item("units", g.units.clone())?;
        gd.set_item("types", g.types.clone())?;
        let rows = PyList::empty(py);
        for r in &g.rows {
            let ro = PyDict::new(py);
            ro.set_item("line", r.line)?;
            ro.set_item("values", r.values.clone())?;
            rows.append(ro)?;
        }
        gd.set_item("rows", rows)?;
        groups.set_item(code, gd)?;
    }
    d.set_item("groups", groups)?;
    Ok(d)
}

/// A parsed AGS4 file held Rust-side. `parse_arrow` crosses only cheap
/// per-group metadata; the raw `ParsedFile` stays HERE and serves two roles
/// on demand: `table_for` builds a group's typed Arrow table on first touch
/// (per-group lazy reads), and `emit` re-emits byte-faithful AGS4 for
/// `write()`. No O(cells) PyObject dict crosses the boundary, and there is no
/// second AGS formatter — emit reproduces the source DATA values exactly. The
/// Python `Ags4File` holds one of these as its `_handle`.
#[pyclass]
struct Reading {
    parsed: laterite_ags4_parse::ParsedFile,
}

#[pymethods]
impl Reading {
    /// Reconstruct spec-correct AGS4 text from the retained parse (CRLF,
    /// every field quoted, blank line between groups) — byte-faithful to
    /// the source DATA values it re-emits.
    fn emit(&self) -> String {
        let blocks: Vec<emit::GroupBlock> = self
            .parsed
            .group_order
            .iter()
            .filter_map(|code| {
                let g = self.parsed.groups.get(code)?;
                let n = g.headings.len();
                // tag + each of the n columns, padded/truncated like the old
                // Python `_matrix` (a ragged DATA row fills its tail with "").
                let pad = |tag: &str, src: &[String]| {
                    let mut row = Vec::with_capacity(n + 1);
                    row.push(tag.to_string());
                    for i in 0..n {
                        row.push(src.get(i).cloned().unwrap_or_default());
                    }
                    row
                };
                let mut matrix: Vec<Vec<String>> = Vec::with_capacity(3 + g.rows.len());
                let mut heading = Vec::with_capacity(n + 1);
                heading.push("HEADING".to_string());
                heading.extend(g.headings.iter().cloned());
                matrix.push(heading);
                matrix.push(pad("UNIT", &g.units));
                matrix.push(pad("TYPE", &g.types));
                for r in &g.rows {
                    matrix.push(pad("DATA", &r.values));
                }
                Some(emit::GroupBlock {
                    code: code.clone(),
                    matrix,
                })
            })
            .collect();
        emit::emit(&blocks)
    }

    /// Build ONE group's typed Arrow table on demand. The read path is
    /// per-group lazy: `read()` / `scan()` only pay for the groups actually
    /// touched, so a 69-group file you query two groups of builds two
    /// RecordBatches, not 69. Returns `None` if `code` isn't in the file.
    /// Same shared emitter (`laterite_types::arrow_cols`) as the old eager build —
    /// byte-identical columns, and still the SAME cast the browser's IPC path
    /// uses. The Python `Ags4File` memoises the result per code.
    fn table_for(&self, code: &str) -> PyResult<Option<PyTable>> {
        let Some(g) = self.parsed.groups.get(code) else {
            return Ok(None);
        };
        // Relational layer is **always-keyed**: a KNOWN group's batch carries the
        // two content-addressed key columns (`_id` col 0, `_parent_id` col 1) so a
        // cross-group join in `.sql()` works with no opt-in — the Python frame
        // accessor strips them by default. The ids come from the one shared
        // keychain (`group_row_ids` → `keychain::row_ids`), so they are byte-
        // identical to the `.ags5db` extension's. A custom/passthrough group is
        // absent from the registry → it has no spec keys → unkeyed batch (#303).
        let reg = laterite_ags4_core::registry::registry();
        let batch = if reg.get(code).is_some() {
            let ids = laterite_ags4_core::keychain::group_row_ids(
                reg,
                code,
                &g.headings,
                g.rows.len(),
                |col, row| g.cell(col, row),
            );
            laterite_types::arrow_cols::build_record_batch_with_ids(
                &ids,
                &g.headings,
                &g.types,
                g.rows.len(),
                |col, row| g.cell(col, row),
            )
        } else {
            laterite_types::arrow_cols::build_record_batch(
                &g.headings,
                &g.types,
                g.rows.len(),
                |col, row| g.cell(col, row),
            )
        }
        .map_err(|e| PyRuntimeError::new_err(format!("arrow batch for {code}: {e}")))?;
        let schema = batch.schema();
        Ok(Some(PyTable::try_new(vec![batch], schema)?))
    }
}

/// A read-only handle to an `.ags.idx` validity **certificate** + byte-offset
/// index (the core `laterite_ags4_core::index::Sidecar`). `Ags4File.certify`
/// mints one over an already-clean file; `read(index=…)` loads + freshness-checks
/// one to skip re-validation. Core owns the format and can *read* a cert with no
/// validator, but cannot *mint* (it doesn't depend on the validator); minting fills
/// the validator identity here and trusts that the CALLER (the Python `certify`)
/// confirmed a clean validation first.
#[pyclass(name = "Sidecar")]
struct PySidecar {
    inner: CoreSidecar,
}

#[pymethods]
impl PySidecar {
    /// Assemble a certificate for an ALREADY-clean file: index `data`'s group
    /// sections and stamp the validation. The caller MUST have validated `data`
    /// clean (0 error findings) — core trusts that, it cannot re-check. `edition`
    /// is the resolved AGS edition (e.g. "4.1.1"); `checked_at` an RFC-3339 UTC
    /// timestamp; `warnings`/`fyi` the advisory counts present at validation
    /// (errors are 0 by construction). The validator name + version are filled
    /// here. Raises `ValueError` if `data` isn't indexable AGS4 (e.g. non-UTF-8,
    /// which the byte index rejects).
    #[staticmethod]
    #[pyo3(signature = (data, edition, checked_at, warnings=0, fyi=0, compat=None, check_files=false, edition_forced=false))]
    #[allow(clippy::too_many_arguments)] // a builder-style mint API; all keyword-only from Python
    fn assemble(
        data: &[u8],
        edition: String,
        checked_at: String,
        warnings: u32,
        fyi: u32,
        compat: Option<String>,
        check_files: bool,
        edition_forced: bool,
    ) -> PyResult<Self> {
        let stamp = ValidationStamp {
            validator: ENGINE_IDENTITY.to_string(),
            // The ENGINE version (not this wheel's), so the cert is comparable
            // across surfaces (e.g. a cert minted by the DuckDB extension).
            validator_version: laterite_ags4_validator::VERSION.to_string(),
            compat,
            check_files,
            edition_forced,
            checked_at,
            warnings,
            fyi,
        };
        let inner = CoreSidecar::assemble(data, edition, stamp)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
        Ok(Self { inner })
    }

    /// Parse a certificate from its on-disk JSON bytes, rejecting an unknown
    /// format version. Raises `ValueError` on malformed / unsupported JSON.
    #[staticmethod]
    fn from_json(data: &[u8]) -> PyResult<Self> {
        let inner = CoreSidecar::from_json(data)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
        Ok(Self { inner })
    }

    /// Is this certificate still current for `data`? Strong check: format version
    /// + byte length + SHA-256. A mismatch means the source changed under the cert
    /// (its byte offsets and clean verdict are now lies), so it must be rebuilt.
    fn is_fresh_for(&self, data: &[u8]) -> bool {
        self.inner.is_fresh_for(data)
    }

    /// Serialise to the on-disk `.ags.idx` JSON (pretty). Raises on a serialize
    /// failure (not expected for a well-formed cert).
    fn to_json<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, pyo3::types::PyBytes>> {
        let bytes = self
            .inner
            .to_json()
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
        Ok(pyo3::types::PyBytes::new(py, &bytes))
    }

    /// The byte-offset index — `{group_code: (start, end)}` in file order (an
    /// insertion-ordered dict). Locates each group's bytes for a sliced read.
    fn index<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let d = PyDict::new(py);
        for code in &self.inner.order {
            if let Some((s, e)) = self.inner.groups.get(code) {
                d.set_item(code, (*s, *e))?;
            }
        }
        Ok(d)
    }

    #[getter]
    fn version(&self) -> u32 {
        self.inner.version
    }
    #[getter]
    fn size(&self) -> u64 {
        self.inner.file.size
    }
    #[getter]
    fn sha256(&self) -> &str {
        &self.inner.file.sha256
    }
    #[getter]
    fn edition(&self) -> &str {
        &self.inner.file.edition
    }
    #[getter]
    fn validator(&self) -> &str {
        &self.inner.validation.validator
    }
    #[getter]
    fn validator_version(&self) -> &str {
        &self.inner.validation.validator_version
    }
    #[getter]
    fn compat(&self) -> Option<&str> {
        self.inner.validation.compat.as_deref()
    }
    #[getter]
    fn checked_at(&self) -> &str {
        &self.inner.validation.checked_at
    }

    #[getter]
    fn check_files(&self) -> bool {
        self.inner.validation.check_files
    }
    #[getter]
    fn edition_forced(&self) -> bool {
        self.inner.validation.edition_forced
    }
    #[getter]
    fn etag(&self) -> Option<&str> {
        self.inner.file.etag.as_deref()
    }
    #[getter]
    fn last_modified(&self) -> Option<&str> {
        self.inner.file.last_modified.as_deref()
    }

    /// Was this cert minted by the CURRENT native validator engine (same engine
    /// version, not the `laterite.compat` profile)? `.validate()` skips the rule
    /// engine only when a carried cert is both fresh (bytes unchanged) AND this
    /// holds — a cert from an older engine is re-validated, not trusted.
    fn matches_native_validator(&self) -> bool {
        self.inner
            .checker_matches(ENGINE_IDENTITY, laterite_ags4_validator::VERSION, None)
    }

    /// Does this cert's check profile cover a request's? (`check_files`: the cert
    /// must have run at least what's asked; `forced_edition`: a forced request is
    /// covered only by the same forced edition, an auto request only by an auto
    /// cert.) The `.validate()` skip requires this alongside a fresh cert + engine
    /// match.
    #[pyo3(signature = (check_files, forced_edition=None))]
    fn profile_covers(&self, check_files: bool, forced_edition: Option<&str>) -> bool {
        self.inner.profile_covers(check_files, forced_edition)
    }
    #[getter]
    fn warnings(&self) -> u32 {
        self.inner.validation.warnings
    }
    #[getter]
    fn fyi(&self) -> u32 {
        self.inner.validation.fyi
    }
    #[getter]
    fn order(&self) -> Vec<String> {
        self.inner.order.clone()
    }

    fn __repr__(&self) -> String {
        format!(
            "<Sidecar v{} {} groups {} bytes edition={:?} by {} {}>",
            self.inner.version,
            self.inner.order.len(),
            self.inner.file.size,
            self.inner.file.edition,
            self.inner.validation.validator,
            self.inner.validation.validator_version,
        )
    }
}

/// Parse `path` or `text` for `read()` / `scan()`: per-group metadata only
/// (headings/units/types/line_numbers). The typed Arrow table for a group is
/// NOT built here — it is built lazily, per group, on first touch via the
/// `_handle`'s `Reading::table_for` (cast by the one shared emitter
/// `laterite_types::arrow_cols`, byte-identical to the browser's IPC path). The
/// raw parse stays Rust-side in that `Reading` handle, also feeding
/// byte-faithful `write()`; no per-cell PyObject rows cross the boundary.
#[pyfunction]
#[pyo3(signature = (path=None, text=None, data=None, encoding=None))]
fn parse_arrow<'py>(
    py: Python<'py>,
    path: Option<String>,
    text: Option<String>,
    data: Option<Vec<u8>>,
    encoding: Option<String>,
) -> PyResult<Bound<'py, PyDict>> {
    let enc = match laterite_ags4_parse::resolve_encoding(encoding.as_deref()) {
        Some(e) => e,
        None => {
            let label = encoding.as_deref().unwrap_or("");
            return err_dict(py, 5, "bad_args", &format!("unknown encoding {label:?}"));
        }
    };
    let parsed = match (path.as_deref(), text.as_deref(), data.as_deref()) {
        (Some(p), _, _) => {
            laterite_ags4_validator::parse::parse_file_with_encoding(Path::new(p), enc)
        }
        (_, Some(t), _) => laterite_ags4_parse::parse_str(t).map_err(ValidatorError::from),
        (_, _, Some(d)) => laterite_ags4_parse::parse_bytes(d, enc).map_err(ValidatorError::from),
        _ => return err_dict(py, 5, "bad_args", "one of path, text, or data is required"),
    };
    let pf = match parsed {
        Ok(pf) => pf,
        Err(e) => {
            let (c, k, m) = map_err(e);
            return err_dict(py, c, &k, &m);
        }
    };

    let d = PyDict::new(py);
    d.set_item("ok", true)?;
    d.set_item("group_order", pf.group_order.clone())?;
    d.set_item("tran_ags", laterite_ags4_validator::tran_ags_of(&pf))?;

    let groups = PyDict::new(py);
    for code in &pf.group_order {
        let Some(g) = pf.groups.get(code) else {
            continue;
        };
        let gd = PyDict::new(py);
        gd.set_item("group_line", g.group_line)?;
        gd.set_item("heading_line", g.heading_line)?;
        gd.set_item("unit_line", g.unit_line)?;
        gd.set_item("type_line", g.type_line)?;
        gd.set_item("headings", g.headings.clone())?;
        gd.set_item("units", g.units.clone())?;
        gd.set_item("types", g.types.clone())?;
        gd.set_item(
            "line_numbers",
            g.rows.iter().map(|r| r.line).collect::<Vec<_>>(),
        )?;
        // No eager Arrow table here: the read path is per-group lazy — the
        // typed table is built on first touch via `Reading::table_for` (the
        // `_handle` below), so this loop only crosses cheap metadata.
        groups.set_item(code, gd)?;
    }
    d.set_item("groups", groups)?;
    // Keep the parse Rust-side for byte-faithful write (no per-cell PyObject
    // rows cross; Reading::emit reproduces the source DATA values exactly).
    d.set_item("_handle", Reading { parsed: pf })?;
    Ok(d)
}

/// Resolve the bundled edition exactly as the engine does. Returns
/// `(version, resolution)` e.g. `("4.1.1", "fallback")`. Raises
/// `ValueError` on a bad override or an unsupported (AGS3) edition.
#[pyfunction]
#[pyo3(signature = (tran_ags=None, override_version=None))]
fn resolve_dict(
    tran_ags: Option<String>,
    override_version: Option<String>,
) -> PyResult<(String, String)> {
    let over =
        parse_dv(override_version.as_deref()).map_err(pyo3::exceptions::PyValueError::new_err)?;
    match laterite_ags4_validator::resolve_dict_version(over, tran_ags.as_deref()) {
        Ok((dv, res)) => Ok((dv.as_str().to_string(), res.as_str().to_string())),
        Err(e) => Err(pyo3::exceptions::PyValueError::new_err(e.to_string())),
    }
}

/// Return the bundled standard dictionary's UNIT/TYPE for each
/// heading in one group, for a given edition. Used by
/// `laterite.compat.convert_to_text(df, dictionary='<version>')` to
/// recover UNIT/TYPE rows when the caller's frame has them dropped
/// (e.g. after `convert_to_numeric` which strips them).
///
/// Returns `{heading: (unit, ags_type)}` — only headings defined in
/// the standard dictionary for that group are included; non-standard
/// headings (the file's own DICT group's territory) return as missing
/// keys.
///
/// Raises `ValueError` on an unknown edition string.
#[pyfunction]
fn dict_group_unit_type<'py>(
    py: Python<'py>,
    edition: &str,
    group: &str,
) -> PyResult<Bound<'py, pyo3::types::PyDict>> {
    let dv = parse_dv(Some(edition))
        .map_err(pyo3::exceptions::PyValueError::new_err)?
        .ok_or_else(|| {
            pyo3::exceptions::PyValueError::new_err(format!(
                "edition cannot be empty/auto; pass one of {}",
                laterite_ags4_validator::editions_joined("/")
            ))
        })?;
    let dict = laterite_ags4_validator::dict::Dictionary::bundled(dv);
    let out = pyo3::types::PyDict::new(py);
    for h in dict.group_headings(group) {
        if let Some(entry) = dict.heading(group, h) {
            out.set_item(*h, (entry.unit, entry.ags_type))?;
        }
    }
    Ok(out)
}

/// Raw group cells for the CLI `lat read` — the group `order` + per-group
/// `{headings, rows}` (rows as string lists in heading order), straight from
/// core's read codec (no typing). So the Rust binary and the Python `lat read`
/// agree byte-for-byte on `read --json` / `--csv` (#430 PR 2).
#[pyfunction]
fn read_groups_raw<'py>(py: Python<'py>, path: String) -> PyResult<Bound<'py, PyDict>> {
    let parsed = laterite_ags4_core::ags4_codec::read_ags4(std::path::Path::new(&path))
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
    let d = PyDict::new(py);
    d.set_item("order", PyList::new(py, &parsed.order)?)?;
    let groups = PyDict::new(py);
    for code in &parsed.order {
        if let Some(g) = parsed.get(code) {
            let gd = PyDict::new(py);
            gd.set_item("headings", PyList::new(py, &g.headings)?)?;
            let rows = PyList::empty(py);
            for row in &g.rows {
                let cells: Vec<&str> = g
                    .headings
                    .iter()
                    .map(|h| row.get(h).map(String::as_str).unwrap_or(""))
                    .collect();
                rows.append(PyList::new(py, &cells)?)?;
            }
            gd.set_item("rows", rows)?;
            groups.set_item(code, gd)?;
        }
    }
    d.set_item("groups", groups)?;
    Ok(d)
}

#[pymodule]
fn _laterite_native(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(run_check, m)?)?;
    m.add_function(wrap_pyfunction!(fix_file, m)?)?;
    m.add_function(wrap_pyfunction!(list_rules, m)?)?;
    m.add_function(wrap_pyfunction!(diff_files, m)?)?;
    m.add_function(wrap_pyfunction!(parse_primitives, m)?)?;
    m.add_function(wrap_pyfunction!(parse_arrow, m)?)?;
    m.add_function(wrap_pyfunction!(read_groups_raw, m)?)?;
    m.add_function(wrap_pyfunction!(resolve_dict, m)?)?;
    m.add_function(wrap_pyfunction!(dict_group_unit_type, m)?)?;
    m.add_function(wrap_pyfunction!(emit_typed::emit_ags4_from_arrow, m)?)?;
    m.add_function(wrap_pyfunction!(emit_typed::emit_ags4_compat, m)?)?;
    m.add_class::<PySidecar>()?;
    registry_fns::register(m)?;
    ags_types_fns::register(m)?;
    transport_fns::register(m)?;
    typed_graph::register(m)?;
    excel_fns::register(m)?;
    Ok(())
}
