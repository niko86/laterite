//! PyO3 wrappers over the `ags5_core::registry` module.
//!
//! Read-only at this stage (D2 of
//! the static Rust registry to Python as JSON-shaped data so a thin
//! Python facade (`laterite/registry.py`) can produce equivalent
//! msgspec.Struct objects without re-parsing the on-disk JSON. D4a
//! later routes `ags5_models.GROUPS` through this same path,
//! collapsing the duplicate-parse.
//!
//! Mutation (`register` / passthrough auto-registration) stays Python-
//! side as an overlay merged with this base — see D4a in the plan.

use ags5_core::ddl::build_ddl;
use ags5_core::registry::{GroupDescriptor, ancestor_chain, inherited_key_names, registry};

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

/// All groups serialised as a JSON array, in declaration order. The
/// per-group field order matches the dictionary's declaration order
/// thanks to `serde_json::preserve_order` (set workspace-wide in the
/// `ags5db` Cargo.toml) + the `GroupDescriptor` struct field order.
///
/// Returning a JSON string rather than a list-of-dicts avoids walking
/// every nested heading through PyO3's per-call type conversion — one
/// large UTF-8 buffer crosses the boundary, then Python's msgspec
/// decodes into typed structs in C.
#[pyfunction]
fn registry_groups_json() -> String {
    registry().to_groups_json()
}

/// Single-group JSON lookup. Returns `None` when the code isn't in the
/// registry (Python equivalent: `GROUPS.get(code)`).
#[pyfunction]
fn registry_get_group(code: &str) -> Option<String> {
    let reg = registry();
    reg.get(code)
        .map(|g| serde_json::to_string(g).expect("group serialises"))
}

/// Parent chain from `code` up to the registry root (`[code, parent,
/// ..., root]`). Matches Python's `_ddl._ancestor_chain` direction.
/// Raises `ValueError` if `code` isn't in the registry — distinguishes
/// "no parent" (root group, returns `[code]`) from "unknown code".
#[pyfunction]
fn registry_ancestor_chain(code: &str) -> PyResult<Vec<String>> {
    let reg = registry();
    if reg.get(code).is_none() {
        return Err(PyValueError::new_err(format!(
            "unknown group code: {:?}",
            code
        )));
    }
    Ok(ancestor_chain(reg, code)
        .into_iter()
        .map(|g| g.code.clone())
        .collect())
}

/// KEY heading names a group inherits from its parent (Python's
/// `_ddl._inherited_key_names`). Returns a sorted list for determinism;
/// callers that want set semantics can wrap in `set()`.
#[pyfunction]
fn registry_inherited_key_names(code: &str) -> PyResult<Vec<String>> {
    let reg = registry();
    let g = reg
        .get(code)
        .ok_or_else(|| PyValueError::new_err(format!("unknown group code: {:?}", code)))?;
    let mut names: Vec<String> = inherited_key_names(reg, g).into_iter().collect();
    names.sort();
    Ok(names)
}

/// Emit the full `.ags5db` DDL (tables + indexes + views + blob table)
/// against the Rust singleton extended with `overlay_groups_json` — a
/// JSON array of additional `GroupDescriptor`s contributed by Python-
/// side `register()` calls (passthrough auto-registration; custom
/// dictionaries registered in tests). Mirrors `Registry::extended_with
/// (extra)` semantics: a code that already exists in the base registry
/// gets replaced.
///
/// This is the seam that closes gate B.1 — `ags5_db._ddl.build_ddl()`
/// delegates here (Stage D4b of `dec-rust-engine-staged-adoption.md`).
/// Output is byte-identical to the pre-D4b Python implementation when
/// the overlay is empty (which the previous parity test, now retired,
/// validated for 4400 lines).
#[pyfunction]
fn registry_build_ddl_with_overlay(overlay_groups_json: &str) -> PyResult<String> {
    let extra: Vec<GroupDescriptor> =
        if overlay_groups_json.is_empty() || overlay_groups_json == "[]" {
            Vec::new()
        } else {
            serde_json::from_str(overlay_groups_json)
                .map_err(|e| PyValueError::new_err(format!("overlay decode: {}", e)))?
        };
    let reg = registry().extended_with(extra);
    Ok(build_ddl(&reg))
}

pub fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(registry_groups_json, m)?)?;
    m.add_function(wrap_pyfunction!(registry_get_group, m)?)?;
    m.add_function(wrap_pyfunction!(registry_ancestor_chain, m)?)?;
    m.add_function(wrap_pyfunction!(registry_inherited_key_names, m)?)?;
    m.add_function(wrap_pyfunction!(registry_build_ddl_with_overlay, m)?)?;
    Ok(())
}
