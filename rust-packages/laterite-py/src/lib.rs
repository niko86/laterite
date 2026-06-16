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
//! the `lat-check` binary uses, so `--json`/`--ndjson` are
//! byte-faithful; the python-ags4 `check_file` dict (with
//! `Metadata`/`Summary`) is assembled in `laterite/compat.py`.

use std::path::Path;

use laterite_ags4_validator::findings::{Severity, Target};
use laterite_ags4_validator::{CheckOptions, DictVersion, Dictionary, Findings, ValidatorError};
use laterite_core::error::CliError;
use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
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

/// Map an `laterite-core` `CliError` to a PyRuntimeError, preserving the
/// exit-code label python callers may surface in error messages.
/// Previously lived in `ags5db_fns.rs`; that module moved to the
/// AGS5 cdylib in S3b, so the base-wheel transport + excel functions
/// share this local copy instead.
pub(crate) fn map_cli_err(e: CliError) -> PyErr {
    let code = e.exit_code();
    PyRuntimeError::new_err(format!("ags5db error (exit {code}): {e}"))
}

/// Map a `--dict-version` string to the optional override. `None` /
/// `"auto"` ⇒ no override (TRAN_AGS auto-pick). Unknown ⇒ `Err`.
fn parse_dv(s: Option<&str>) -> Result<Option<DictVersion>, String> {
    match s {
        None | Some("auto") | Some("") => Ok(None),
        Some("4.0.3") => Ok(Some(DictVersion::V4_0_3)),
        Some("4.0.4") => Ok(Some(DictVersion::V4_0_4)),
        Some("4.1") => Ok(Some(DictVersion::V4_1)),
        Some("4.1.1") => Ok(Some(DictVersion::V4_1_1)),
        Some("4.2") => Ok(Some(DictVersion::V4_2)),
        Some(other) => Err(format!(
            "unknown --dict-version {other:?} (expected auto|4.0.3|4.0.4|4.1|4.1.1|4.2)"
        )),
    }
}

/// (exit_code, error_kind, message) for a validator error — exit codes
/// mirror the `lat-check` binary exactly (3 not-found/io, 4
/// not-utf8/not-ags4/unsupported-edition, 5 bad-dict).
fn map_err(e: ValidatorError) -> (i32, String, String) {
    let msg = e.to_string();
    let (code, kind) = match e {
        ValidatorError::NotFound(_) => (3, "not_found"),
        ValidatorError::Io { .. } => (3, "io"),
        ValidatorError::NotUtf8(_) => (4, "not_utf8"),
        ValidatorError::NotAgs4(_) => (4, "not_ags4"),
        ValidatorError::UnsupportedEdition { .. } => (4, "unsupported_edition"),
        ValidatorError::BadDict { .. } => (5, "bad_dict"),
    };
    (code, kind.to_string(), msg)
}

/// Run the validator from either a path or in-memory text. Returns
/// `(file, dict_version, resolution, findings)` or
/// `(exit_code, error_kind, message)`.
#[allow(clippy::type_complexity)]
fn validate(
    path: Option<&str>,
    text: Option<&str>,
    dvr: Option<&str>,
    warnings: bool,
    fyi: bool,
    check_files: bool,
    encoding: Option<&str>,
) -> Result<(String, String, String, Findings), (i32, String, String)> {
    let over = parse_dv(dvr).map_err(|m| (5, "bad_dict".to_string(), m))?;
    let enc = match encoding {
        None | Some("") => encoding_rs::UTF_8,
        Some(label) => encoding_rs::Encoding::for_label(label.as_bytes()).ok_or_else(|| {
            (
                5,
                "bad_args".to_string(),
                format!("unknown encoding {label:?}"),
            )
        })?,
    };
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
        let pf = laterite_ags4_validator::parse::parse_str(t).map_err(map_err)?;
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
    } else {
        Err((
            5,
            "bad_args".to_string(),
            "either path or text is required".to_string(),
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

/// Lowercase serde-name of a [`Severity`] for the structured PyDict.
fn severity_str(s: Severity) -> &'static str {
    match s {
        Severity::Error => "error",
        Severity::Warning => "warning",
        Severity::Fyi => "fyi",
    }
}

/// `{file, findings:{ "AGS Format Rule N":[{line,group,desc}] }}` —
/// the exact `serde_json` value + insertion order the `lat-check`
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
#[pyo3(signature = (path=None, text=None, dict_version=None, include_warnings=false, include_fyi=false, check_files=false, encoding=None))]
#[allow(clippy::too_many_arguments)]
fn run_check<'py>(
    py: Python<'py>,
    path: Option<String>,
    text: Option<String>,
    dict_version: Option<String>,
    include_warnings: bool,
    include_fyi: bool,
    check_files: bool,
    encoding: Option<String>,
) -> PyResult<Bound<'py, PyDict>> {
    match validate(
        path.as_deref(),
        text.as_deref(),
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

/// Parse `path` or `text` into per-group primitives (string cells —
/// AGS4 is a text format). `headings`/`units`/`types`/row `values`
/// exclude the leading row tag (matching `ParsedGroup`); the Python
/// side prepends the literal `HEADING`/`UNIT`/`TYPE`/`DATA` when it
/// needs the python-ags4-shaped frame.
#[pyfunction]
#[pyo3(signature = (path=None, text=None, encoding=None))]
fn parse_primitives<'py>(
    py: Python<'py>,
    path: Option<String>,
    text: Option<String>,
    encoding: Option<String>,
) -> PyResult<Bound<'py, PyDict>> {
    let enc = match encoding.as_deref() {
        None | Some("") => encoding_rs::UTF_8,
        Some(label) => match encoding_rs::Encoding::for_label(label.as_bytes()) {
            Some(e) => e,
            None => {
                return err_dict(py, 5, "bad_args", &format!("unknown encoding {label:?}"));
            }
        },
    };
    let parsed = match (path.as_deref(), text.as_deref()) {
        (Some(p), _) => laterite_ags4_validator::parse::parse_file_with_encoding(Path::new(p), enc),
        (_, Some(t)) => laterite_ags4_validator::parse::parse_str(t),
        _ => {
            return err_dict(py, 5, "bad_args", "either path or text is required");
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
    parsed: laterite_ags4_validator::parse::ParsedFile,
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
        let batch = laterite_types::arrow_cols::build_record_batch(
            &g.headings,
            &g.types,
            g.rows.len(),
            |col, row| {
                g.rows
                    .get(row)
                    .and_then(|r| r.values.get(col))
                    .map(String::as_str)
            },
        )
        .map_err(|e| PyRuntimeError::new_err(format!("arrow batch for {code}: {e}")))?;
        let schema = batch.schema();
        Ok(Some(PyTable::try_new(vec![batch], schema)?))
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
#[pyo3(signature = (path=None, text=None, encoding=None))]
fn parse_arrow<'py>(
    py: Python<'py>,
    path: Option<String>,
    text: Option<String>,
    encoding: Option<String>,
) -> PyResult<Bound<'py, PyDict>> {
    let enc = match encoding.as_deref() {
        None | Some("") => encoding_rs::UTF_8,
        Some(label) => match encoding_rs::Encoding::for_label(label.as_bytes()) {
            Some(e) => e,
            None => return err_dict(py, 5, "bad_args", &format!("unknown encoding {label:?}")),
        },
    };
    let parsed = match (path.as_deref(), text.as_deref()) {
        (Some(p), _) => laterite_ags4_validator::parse::parse_file_with_encoding(Path::new(p), enc),
        (_, Some(t)) => laterite_ags4_validator::parse::parse_str(t),
        _ => return err_dict(py, 5, "bad_args", "either path or text is required"),
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
            pyo3::exceptions::PyValueError::new_err(
                "edition cannot be empty/auto; pass one of 4.0.3/4.0.4/4.1/4.1.1/4.2",
            )
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

#[pymodule]
fn _laterite_native(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(run_check, m)?)?;
    m.add_function(wrap_pyfunction!(parse_primitives, m)?)?;
    m.add_function(wrap_pyfunction!(parse_arrow, m)?)?;
    m.add_function(wrap_pyfunction!(resolve_dict, m)?)?;
    m.add_function(wrap_pyfunction!(dict_group_unit_type, m)?)?;
    m.add_function(wrap_pyfunction!(emit_typed::emit_ags4_from_arrow, m)?)?;
    m.add_function(wrap_pyfunction!(emit_typed::emit_ags4_compat, m)?)?;
    registry_fns::register(m)?;
    ags_types_fns::register(m)?;
    transport_fns::register(m)?;
    typed_graph::register(m)?;
    excel_fns::register(m)?;
    Ok(())
}
