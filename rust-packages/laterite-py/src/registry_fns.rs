//! PyO3 wrappers over the `laterite_ags4_core::registry` module.
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

use laterite_ags4_core::registry::{ancestor_chain, inherited_key_names, registry};

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

/// The bundled standard dictionary for `edition`, serialised as JSON — the
/// `{ags_edition, groups:[{code, contents, parent, headings:[…]}]}` shape the
/// browser and Node's `registry.dictionary()` also render, from the ONE shared
/// `dict::dictionary_dto` builder (#294 F#6). `edition` `None`/`"auto"` → the
/// fallback edition; else `4.0.3|4.0.4|4.1|4.1.1|4.2`. Raises `ValueError` on an
/// unknown edition. (The union `GROUPS` stays the default registry; this is the
/// per-edition, standard-dictionary view.)
#[pyfunction]
#[pyo3(signature = (edition=None))]
fn registry_dictionary_json(edition: Option<String>) -> PyResult<String> {
    let version = crate::parse_dv(edition.as_deref())
        .map_err(PyValueError::new_err)?
        .unwrap_or(laterite_ags4_validator::dict::FALLBACK);
    let dto = laterite_ags4_validator::dict::dictionary_dto(version);
    Ok(serde_json::to_string(&dto).expect("dictionary serialises"))
}

pub fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(registry_groups_json, m)?)?;
    m.add_function(wrap_pyfunction!(registry_ancestor_chain, m)?)?;
    m.add_function(wrap_pyfunction!(registry_inherited_key_names, m)?)?;
    m.add_function(wrap_pyfunction!(registry_dictionary_json, m)?)?;
    Ok(())
}
