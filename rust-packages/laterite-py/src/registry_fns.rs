//! PyO3 wrappers over the `laterite_ags4_core::registry` module.
//!
//! Read-only at this stage (D2 of
//! the static Rust registry to Python as JSON-shaped data so a thin
//! Python facade (`laterite/registry.py`) can produce equivalent
//! msgspec.Struct objects without re-parsing the on-disk JSON. D4a
//! later routed the Python-side registry facade through this same
//! path, collapsing the duplicate-parse.
//!
//! Mutation (`register` / passthrough auto-registration) stays Python-
//! side as an overlay merged with this base — see D4a in the plan.

use laterite_ags4_core::registry::{ancestor_chain, inherited_key_names, registry};
use laterite_ags4_merge::TypeClashMode;
use laterite_ags4_validator::dict::DictVersion;

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

/// All groups serialised as a JSON array, in declaration order. The
/// per-group field order matches the dictionary's declaration order
/// thanks to `serde_json::preserve_order` (the same feature flag every
/// producing crate sets) + the `GroupDescriptor` struct field order.
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
            "unknown group code: {code:?}"
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
        .ok_or_else(|| PyValueError::new_err(format!("unknown group code: {code:?}")))?;
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
// PyO3 boundary: owns the deserialized input
#[allow(clippy::needless_pass_by_value)]
fn registry_dictionary_json(edition: Option<String>) -> PyResult<String> {
    let version = crate::parse_dv(edition.as_deref())
        .map_err(PyValueError::new_err)?
        .unwrap_or(laterite_ags4_validator::dict::FALLBACK);
    let dto = laterite_ags4_validator::dict::dictionary_dto(version);
    Ok(serde_json::to_string(&dto).expect("dictionary serialises"))
}

/// What THIS SURFACE resolves an encoding label to — the canonical `encoding_rs`
/// name (`"UTF-8"`, `"windows-1252"`, `"ISO-8859-15"`), or `None` if it refuses.
///
/// The wheel's read/validate/fix paths call `laterite_ags4_parse::resolve_encoding`
/// and raise on `None`; this reports the same answer without needing a file. The
/// surface census diffs it against the other launchers, which is how a surface that
/// quietly re-adds a UTF-8 fallback for unknown labels (Node had exactly that) shows
/// up as `"cp1252x" -> "UTF-8"` instead of `None`.
#[pyfunction]
fn resolve_encoding_label(label: Option<&str>) -> Option<String> {
    laterite_ags4_parse::resolve_encoding(label).map(|e| e.name().to_string())
}

/// The bundled AGS4 editions, oldest first — `["4.0.3", … "4.2"]`.
///
/// GENERATED, all the way down: `DictVersion::ALL` is emitted by the reference
/// leaf's `build.rs` from `ags_dictionary.json`. Exposed so Python does not keep a
/// hand-written copy of a set the dictionary already defines — it had three
/// (`_cli.py`'s `--dict-version` choices, the `Edition` type, `registry.py`), and a
/// new edition would have reached none of them.
#[pyfunction]
fn registry_editions() -> Vec<String> {
    DictVersion::ALL.iter().map(|v| v.as_str().into()).collect()
}

/// The edition `auto` resolves to when a file's `TRAN_AGS` is missing or
/// unrecognised (the union's `fallback_edition`, generated). Exposed for the same
/// reason as [`registry_editions`]: so no surface hard-codes it.
#[pyfunction]
fn registry_fallback_edition() -> String {
    laterite_ags4_validator::dict::FALLBACK.as_str().into()
}

/// The `--on-type-clash` modes merge accepts, in declaration order —
/// `["error", "widen", "promote"]`.
///
/// Same reason as [`registry_editions`]: the set is defined once, by
/// [`TypeClashMode::ALL`] in `laterite-ags4-merge`, and Python kept a hand-typed
/// copy (`_cli.py`'s `_CLASH_CHOICES`) sitting one line below `_DICT_CHOICES`,
/// which already asks the registry. A fourth mode added to the Rust enum would
/// have reached the copy through no path — the exact drift shape #549 is about.
/// Ordered because `ALL` is (Error/default first), and `--help` prints it.
#[pyfunction]
fn registry_type_clash_modes() -> Vec<String> {
    TypeClashMode::ALL
        .iter()
        .map(|m| m.as_str().into())
        .collect()
}

pub fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(registry_groups_json, m)?)?;
    m.add_function(wrap_pyfunction!(registry_ancestor_chain, m)?)?;
    m.add_function(wrap_pyfunction!(registry_inherited_key_names, m)?)?;
    m.add_function(wrap_pyfunction!(registry_dictionary_json, m)?)?;
    m.add_function(wrap_pyfunction!(registry_editions, m)?)?;
    m.add_function(wrap_pyfunction!(resolve_encoding_label, m)?)?;
    m.add_function(wrap_pyfunction!(registry_fallback_edition, m)?)?;
    m.add_function(wrap_pyfunction!(registry_type_clash_modes, m)?)?;
    Ok(())
}
