//! Stage F2b-5: `write_db(proj, path) -> None`.
//!
//! Takes a Python typed-graph PROJ tree (built from compiled
//! `#[pyclass]` types and/or `laterite.dynamic` factories) and writes
//! a `.ags5db` file. Reuses the converter's bucket-writer path
//! (`ags5db::convert::write_buckets`) so AGS4-ingest and typed-graph
//! writes share UUID7 minting, parent-id resolution, DDL, and the
//! `_spec_*` self-describing tables.
//!
//! F2b-5a: standard groups (compiled `#[pyclass]`) handled.
//! F2b-5b: dynamic / passthrough classes (`laterite.dynamic.*`,
//! identifiable by their `_ags_code` + `_ags_heading_specs` class
//! attrs) also handled — discovered during the walk and added to a
//! session-extended registry so their schema reaches the file's DDL
//! and `_spec_*` tables. `BlobAttachment` side-channel deferred to
//! F2c (the surviving Python `ags5_db.write_ags5db` still supports
//! it until then).

use std::collections::{BTreeMap, HashMap};
use std::path::Path;

use ags5db::convert::{compact_db, write_buckets};
use ags5db::registry::{GroupDescriptor, Heading, Registry, registry};
use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use pyo3::types::PyList;

use crate::ags5db_fns::map_cli_err;

#[pyfunction]
#[pyo3(signature = (proj, path, append=false))]
pub fn ags5db_write_db(py: Python<'_>, proj: Py<PyAny>, path: &str, append: bool) -> PyResult<()> {
    let static_reg = registry();

    // Walk once with the compiled registry as the reference for
    // children-discovery. Dynamic instances (passthrough) get caught
    // by class-attribute lookup; their descriptors are collected as
    // we go and folded into a session-extended registry before the
    // actual write.
    let mut buckets: HashMap<String, Vec<HashMap<String, String>>> = HashMap::new();
    let mut passthrough: BTreeMap<String, GroupDescriptor> = BTreeMap::new();
    walk_into_buckets(
        py,
        proj.bind(py),
        static_reg,
        &mut buckets,
        &mut passthrough,
    )?;

    // Session-extended registry includes any passthrough descriptors
    // we discovered. Matches what `ags4_to_db` does on the AGS4 path
    // (`build_passthrough_descriptors` → `extended_with`), so the
    // DDL / `_spec_*` writers don't need to know they're passthrough.
    let session_reg: Registry;
    let reg: &Registry = if passthrough.is_empty() {
        static_reg
    } else {
        session_reg = static_reg.extended_with(passthrough.into_values().collect());
        &session_reg
    };

    // For a fresh write (default), remove any existing file first —
    // `write_buckets` opens the path for writing and mixing our DDL
    // with prior schema would corrupt the result. For `append=True`,
    // the file must already exist and we preserve its dedup state via
    // `preload_codec_state`.
    let p = Path::new(path);
    if !append && p.exists() {
        std::fs::remove_file(p).map_err(|e| {
            PyRuntimeError::new_err(format!("remove existing {}: {e}", p.display()))
        })?;
    }

    write_buckets(p, reg, &buckets, append).map_err(map_cli_err)?;
    // CTAS rewrite to drop the per-batch dead segments — same shape
    // the AGS4 ingest path takes (`ags4_to_db` → `do_convert` →
    // `compact_db` unless `no_compact` is set). Roughly halves output
    // size on small files; bigger gain on real data.
    compact_db(p).map_err(map_cli_err)?;
    Ok(())
}

/// Depth-first walk of the PROJ tree. For each instance:
/// - look up its AGS code via the Python class name
/// - if the code is in the compiled registry, extract every heading
///   value via `getattr(py_name)` and emit a row map keyed by the
///   UPPERCASE heading name (matches the `insert_group_rows`
///   contract on the ags5db side)
/// - if not in the compiled registry, treat it as a dynamic /
///   passthrough class: read its `_ags_code` + `_ags_heading_specs`
///   class attrs (set by `laterite.dynamic.get_or_register`),
///   register a `GroupDescriptor` in the session-passthrough map,
///   and extract values the same way.
/// - recurse into every known child-list field
///   (`{child_code.lower()}s`) — compiled children via the registry,
///   plus any passthrough children attached via setattr (discovered
///   by scanning the parent's `__dict__` for list-valued attrs).
fn walk_into_buckets(
    py: Python<'_>,
    instance: &Bound<'_, PyAny>,
    static_reg: &Registry,
    buckets: &mut HashMap<String, Vec<HashMap<String, String>>>,
    passthrough: &mut BTreeMap<String, GroupDescriptor>,
) -> PyResult<()> {
    let py_class_name = instance.get_type().name()?.to_string();
    let code = py_class_name.to_uppercase();

    // Resolve the headings list — either from the compiled registry
    // or, for a dynamic class, from its `_ags_heading_specs` attr.
    let (headings, parent_for_recurse): (Vec<(String, String)>, Option<String>) =
        if let Some(g) = static_reg.get(&code) {
            (
                g.headings
                    .iter()
                    .map(|h| (h.name.clone(), h.ags_type.clone()))
                    .collect(),
                Some(code.clone()),
            )
        } else if let Ok(specs_attr) = instance.getattr("_ags_heading_specs") {
            // Dynamic class — collect the descriptor for the session
            // registry and extract its headings list for the row build.
            let specs: Vec<(String, String)> = specs_attr.extract()?;
            // Use _ags_code if present (canonical UPPERCASE code) —
            // beats falling back to the Python class name which may be
            // mangled for shape-conflict cases (`MYCUSTOM__a1b2c3`).
            let code_attr: String = instance
                .getattr("_ags_code")
                .ok()
                .and_then(|v| v.extract::<String>().ok())
                .unwrap_or_else(|| code.clone());
            passthrough
                .entry(code_attr.clone())
                .or_insert_with(|| descriptor_from_specs(&code_attr, &specs));
            (specs, Some(code_attr))
        } else {
            // Not a registered class and not a dynamic one — skip
            // silently. This catches stray Python objects that found
            // their way onto a child list.
            return Ok(());
        };

    let mut row: HashMap<String, String> = HashMap::new();
    for (name, ags_type) in &headings {
        let py_name = name.to_lowercase();
        match instance.getattr(py_name.as_str()) {
            Ok(value) => {
                let s = python_value_to_string(&value, ags_type)?;
                row.insert(name.clone(), s);
            }
            Err(_) => {
                row.insert(name.clone(), String::new());
            }
        }
    }
    let bucket_code = parent_for_recurse.as_deref().unwrap_or(&code).to_string();
    buckets.entry(bucket_code.clone()).or_default().push(row);

    // Recurse into children. Compiled children: iterate the registry
    // for declared parent==bucket_code. Passthrough children: any
    // list-valued attribute on the instance's __dict__ whose elements
    // expose `_ags_code` (and aren't a compiled child we've already
    // handled).
    let mut handled_fields: std::collections::HashSet<String> = std::collections::HashSet::new();
    for child in static_reg
        .iter()
        .filter(|c| c.parent.as_deref() == Some(&bucket_code))
    {
        let field = format!("{}s", child.code.to_lowercase());
        handled_fields.insert(field.clone());
        recurse_child_field(py, instance, &field, static_reg, buckets, passthrough)?;
    }

    // Walk __dict__ for any extra list-valued attrs (these are the
    // passthrough children attached by read_db's setattr step).
    if let Ok(dict_attr) = instance.getattr("__dict__")
        && let Ok(items) = dict_attr.call_method0("items")
    {
        for kv in items.try_iter()? {
            let kv = kv?;
            let key: String = kv.get_item(0)?.extract()?;
            if handled_fields.contains(&key) {
                continue;
            }
            let val = kv.get_item(1)?;
            if val.cast::<PyList>().is_ok() {
                recurse_child_field(py, instance, &key, static_reg, buckets, passthrough)?;
            }
        }
    }

    Ok(())
}

/// Pull a child list off `instance` by name and recurse into every
/// element. No-op if the attribute is missing or not a list.
fn recurse_child_field(
    py: Python<'_>,
    instance: &Bound<'_, PyAny>,
    field: &str,
    static_reg: &Registry,
    buckets: &mut HashMap<String, Vec<HashMap<String, String>>>,
    passthrough: &mut BTreeMap<String, GroupDescriptor>,
) -> PyResult<()> {
    let Ok(child_attr) = instance.getattr(field) else {
        return Ok(());
    };
    let list: Bound<'_, PyList> = match child_attr.cast_into::<PyList>() {
        Ok(l) => l,
        Err(_) => return Ok(()),
    };
    for item in list.iter() {
        walk_into_buckets(py, &item, static_reg, buckets, passthrough)?;
    }
    Ok(())
}

/// Build a `GroupDescriptor` from a dynamic class's
/// `_ags_heading_specs`. Mirrors the AGS4 path's
/// `build_passthrough_descriptors`: parent defaults to `LOCA`, every
/// heading is `status=OTHER`, missing/empty types fall through to
/// `"X"`.
fn descriptor_from_specs(code: &str, specs: &[(String, String)]) -> GroupDescriptor {
    let headings: Vec<Heading> = specs
        .iter()
        .map(|(name, ags_type)| Heading {
            name: name.clone(),
            status: "OTHER".into(),
            ags_type: if ags_type.is_empty() {
                "X".into()
            } else {
                ags_type.clone()
            },
            unit: None,
            description: String::new(),
            indexed: None,
        })
        .collect();
    GroupDescriptor {
        code: code.to_string(),
        contents: format!("(passthrough) {}", code),
        parent: Some("LOCA".into()),
        headings,
        is_high_volume: false,
        index_parent: None,
    }
}

/// Convert a Python attribute value into the AGS4-style string that
/// `parse_value` (the codec's typed-value parser) accepts on read.
///
/// `None` → empty string; numerics formatted by the heading's
/// declared precision (`1DP`, `2DP`, …, `nSF`); `DT` calls
/// `value.isoformat()`; `YN` round-trips Python `True`/`False` as
/// `"Y"`/`"N"`. Everything else stringifies via `str(value)`.
fn python_value_to_string(value: &Bound<'_, PyAny>, ags_type: &str) -> PyResult<String> {
    if value.is_none() {
        return Ok(String::new());
    }
    let t = ags_type.trim().to_uppercase();
    match t.as_str() {
        "YN" => {
            let b: bool = value.extract()?;
            Ok(if b { "Y".into() } else { "N".into() })
        }
        "DT" => {
            // datetime → ISO 8601 string (matches what AGS4 carries).
            // Python `datetime.isoformat()` is the canonical form.
            let s: String = value.call_method0("isoformat")?.extract()?;
            Ok(s)
        }
        "0DP" => {
            let i: i64 = value.extract()?;
            Ok(i.to_string())
        }
        _ => {
            if let Some(prec) = ags_decimal_precision(&t) {
                let f: f64 = value.extract()?;
                Ok(format!("{:.*}", prec, f))
            } else {
                // String types (ID, X, PA, PT, etc.) and unknown codes
                // — let Python's str() do the conversion.
                Ok(value.str()?.to_string())
            }
        }
    }
}

/// Parse `NDP` / `NSF` numeric AGS codes into a decimal-place count.
/// `2DP` → `Some(2)`, `4SF` → `Some(4)`, `RL` → `Some(6)` (close
/// enough; see `ags_types::format_value` for the canonical fallback).
/// Anything else → `None` (caller treats as string).
fn ags_decimal_precision(ags_type_upper: &str) -> Option<usize> {
    if ags_type_upper == "RL" {
        return Some(6);
    }
    for suffix in ["DP", "SF", "SCI"] {
        if let Some(prefix) = ags_type_upper.strip_suffix(suffix)
            && !prefix.is_empty()
            && prefix.chars().all(|c| c.is_ascii_digit())
        {
            return prefix.parse::<usize>().ok();
        }
    }
    None
}
