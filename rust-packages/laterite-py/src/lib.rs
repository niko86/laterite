//! PyO3 module `laterite._laterite_native`.
//!
//! Thin boundary: every bit of AGS4 logic lives in `ags4-validator`
//! (clean-room, parity-tested) or the local `emit` module. The Python
//! side (`laterite/__init__.py`, `compat.py`, `_cli.py`) builds
//! narwhals/polars frames and the python-ags4-shaped dict from the
//! primitives returned here.
//!
//! Two error-JSON shapes are deliberately preserved (see the package
//! README): the Rust-CLI shape `{file, findings:{rule:[...]}}` is
//! built *here* with the same `serde_json` (`preserve_order`) calls
//! the `ags4-check` binary uses, so `--json`/`--ndjson` are
//! byte-faithful; the python-ags4 `check_file` dict (with
//! `Metadata`/`Summary`) is assembled in `laterite/compat.py`.

use std::path::Path;

use ags4_validator::{CheckOptions, DictVersion, Dictionary, Findings, ValidatorError};
use ags5_core::error::CliError;
use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use serde_json::{Map, Value};

// S3b (release/v0.1.0-prep): `ags5db_fns` moved to the separate
// `laterite-py-ags5` cdylib. Base wheel ships without DuckDB; the
// AGS5 surface is gated behind the `laterite[ags5]` extra.
mod ags_types_fns;
mod emit;
mod excel_fns;
mod registry_fns;
mod transport_fns;
mod typed_graph;

/// Map an `ags5-core` `CliError` to a PyRuntimeError, preserving the
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
/// mirror the `ags4-check` binary exactly (3 not-found/io, 4
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
        match ags4_validator::check_file_with_dict(Path::new(p), &opts) {
            Ok((found, dv, res)) => Ok((
                p.to_string(),
                dv.as_str().to_string(),
                res.as_str().to_string(),
                found,
            )),
            Err(e) => Err(map_err(e)),
        }
    } else if let Some(t) = text {
        let pf = ags4_validator::parse::parse_str(t).map_err(map_err)?;
        let tran = ags4_validator::tran_ags_of(&pf);
        let (dv, res) =
            ags4_validator::resolve_dict_version(over, tran.as_deref()).map_err(map_err)?;
        let dict = Dictionary::bundled(dv);
        let mut found = Findings::new();
        ags4_validator::rules::run_all(&pf, &dict, &opts, None, &mut found);
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

/// `{file, findings:{ "AGS Format Rule N":[{line,group,desc}] }}` —
/// the exact `serde_json` value + insertion order the `ags4-check`
/// binary's `json_value`/`json_string` produce. `preserve_order` (set
/// in Cargo.toml, matching the validator) keeps the key order stable.
fn findings_json(file: &str, found: &Findings) -> String {
    let mut fmap = Map::new();
    for (rule, items) in found {
        let arr: Vec<Value> = items
            .iter()
            .map(|f| {
                let mut o = Map::new();
                o.insert(
                    "line".into(),
                    f.line.map(Value::from).unwrap_or(Value::Null),
                );
                o.insert("group".into(), Value::from(f.group.clone()));
                o.insert("desc".into(), Value::from(f.desc.clone()));
                Value::Object(o)
            })
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
            let mut o = Map::new();
            o.insert("rule".into(), Value::from(rule.clone()));
            o.insert(
                "line".into(),
                f.line.map(Value::from).unwrap_or(Value::Null),
            );
            o.insert("group".into(), Value::from(f.group.clone()));
            o.insert("desc".into(), Value::from(f.desc.clone()));
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
        (Some(p), _) => ags4_validator::parse::parse_file_with_encoding(Path::new(p), enc),
        (_, Some(t)) => ags4_validator::parse::parse_str(t),
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
    d.set_item("tran_ags", ags4_validator::tran_ags_of(&pf))?;

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
    match ags4_validator::resolve_dict_version(over, tran_ags.as_deref()) {
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
    let dict = ags4_validator::dict::Dictionary::bundled(dv);
    let out = pyo3::types::PyDict::new(py);
    for h in dict.group_headings(group) {
        if let Some(entry) = dict.heading(group, h) {
            out.set_item(*h, (entry.unit, entry.ags_type))?;
        }
    }
    Ok(out)
}

/// Emit AGS4 text. `groups` is an ordered list of `(code, matrix)`
/// where `matrix[0]` is the HEADING line (incl. the literal
/// `"HEADING"` cell) and each later row begins with its
/// `UNIT`/`TYPE`/`DATA` tag. CRLF, every field quoted, blank line
/// between groups (Rule 5 / Rule 2a).
#[pyfunction]
fn emit_ags4(groups: Vec<(String, Vec<Vec<String>>)>) -> PyResult<String> {
    let blocks: Vec<emit::GroupBlock> = groups
        .into_iter()
        .map(|(code, matrix)| emit::GroupBlock { code, matrix })
        .collect();
    Ok(emit::emit(&blocks))
}

#[pymodule]
fn _laterite_native(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(run_check, m)?)?;
    m.add_function(wrap_pyfunction!(parse_primitives, m)?)?;
    m.add_function(wrap_pyfunction!(resolve_dict, m)?)?;
    m.add_function(wrap_pyfunction!(dict_group_unit_type, m)?)?;
    m.add_function(wrap_pyfunction!(emit_ags4, m)?)?;
    registry_fns::register(m)?;
    ags_types_fns::register(m)?;
    transport_fns::register(m)?;
    typed_graph::register(m)?;
    excel_fns::register(m)?;
    Ok(())
}
