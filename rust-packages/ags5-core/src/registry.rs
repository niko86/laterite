//! AGS5 group registry — port of `ags5_models.registry`.
//!
//! Loads the bundled `ags5_dictionary.json` (embedded via `include_str!`)
//! into a singleton at first access. Provides `GroupDescriptor` + `Heading`
//! structs and the parent-chain walks the DDL builder and migrate command
//! need.

use std::collections::HashMap;
use std::sync::OnceLock;

use serde::{Deserialize, Serialize};

// Dictionary lives in the Rust crate's own data/ dir as of Stage D1; the
// Python ags5-models wheel pulls the same bytes via hatchling force-include
// (see packages/ags5-models/pyproject.toml). tests/test_dictionary_single_source.py
// asserts byte-equality across the two distribution paths.
const DICTIONARY_JSON: &str = include_str!("../data/ags5_dictionary.json");

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Heading {
    pub name: String,
    pub status: String, // "KEY" / "REQUIRED" / "OTHER"
    #[serde(rename = "type")]
    pub ags_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub description: String,
    /// Phase 6.5.2 declarative index override. `None` = use built-in
    /// rule (index iff status=='KEY' AND ags_type=='ID').
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub indexed: Option<bool>,
}

impl Heading {
    /// Lower-cased heading name — matches the view's column form.
    pub fn py_name(&self) -> String {
        self.name.to_lowercase()
    }

    pub fn is_key(&self) -> bool {
        self.status.eq_ignore_ascii_case("KEY")
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GroupDescriptor {
    pub code: String,
    pub contents: String,
    pub parent: Option<String>,
    #[serde(default)]
    pub is_high_volume: bool,
    pub headings: Vec<Heading>,
    /// Phase 6.5.2 declarative index override. `None` = use built-in rule
    /// (auto-index parent_id iff parent is not None).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub index_parent: Option<bool>,
}

impl GroupDescriptor {
    /// `g_<code>` — the DuckDB table name.
    pub fn table(&self) -> String {
        format!("g_{}", self.code.to_lowercase())
    }

    /// `v_<code>` — the DuckDB view name.
    pub fn view(&self) -> String {
        format!("v_{}", self.code.to_lowercase())
    }

    pub fn key_headings(&self) -> impl Iterator<Item = &Heading> {
        self.headings.iter().filter(|h| h.is_key())
    }

    pub fn non_key_headings(&self) -> impl Iterator<Item = &Heading> {
        self.headings.iter().filter(|h| !h.is_key())
    }
}

#[derive(Debug, Deserialize)]
struct DictionaryFile {
    #[allow(dead_code)]
    format_version: String,
    #[allow(dead_code)]
    ags_edition: String,
    groups: Vec<GroupDescriptor>,
}

/// Insertion-order-preserving map of group code → descriptor. Group
/// iteration order matches the dictionary's declaration order (PROJ
/// first, then top-level groups, then children) — this is load-bearing
/// for the migrate command's DDL emission order (parents before children).
pub struct Registry {
    /// `Vec<(code, descriptor)>` preserves declaration order.
    pub entries: Vec<(String, GroupDescriptor)>,
    by_code: HashMap<String, usize>,
}

impl Registry {
    pub fn get(&self, code: &str) -> Option<&GroupDescriptor> {
        self.by_code.get(code).map(|i| &self.entries[*i].1)
    }

    pub fn iter(&self) -> impl Iterator<Item = &GroupDescriptor> {
        self.entries.iter().map(|(_, g)| g)
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Serialise every group as a JSON array, in declaration order. The
    /// per-group field order matches the on-disk dictionary's
    /// declaration order (`code, contents, parent, is_high_volume,
    /// headings, index_parent?`) thanks to `serde_json`'s
    /// `preserve_order` feature + the struct field ordering. Used by
    /// `laterite.registry` to ferry the registry across the PyO3
    /// boundary without rebuilding a parallel schema (Stage D2 of
    pub fn to_groups_json(&self) -> String {
        let groups: Vec<&GroupDescriptor> = self.iter().collect();
        serde_json::to_string(&groups).expect("registry to_json must succeed")
    }

    /// Clone this registry and append the given group descriptors at
    /// the end, preserving the original entries' order. Used by the
    /// AGS4 codec when an input file carries groups not in the static
    /// dictionary — passthrough auto-registration. Replaces any prior
    /// registration of the same code.
    pub fn extended_with(&self, extra: Vec<GroupDescriptor>) -> Self {
        let mut entries: Vec<(String, GroupDescriptor)> = self.entries.clone();
        let mut by_code: HashMap<String, usize> = self.by_code.clone();
        for g in extra {
            if let Some(&i) = by_code.get(&g.code) {
                entries[i] = (g.code.clone(), g);
            } else {
                by_code.insert(g.code.clone(), entries.len());
                entries.push((g.code.clone(), g));
            }
        }
        Registry { entries, by_code }
    }
}

static REGISTRY: OnceLock<Registry> = OnceLock::new();

pub fn registry() -> &'static Registry {
    REGISTRY.get_or_init(|| {
        let file: DictionaryFile =
            serde_json::from_str(DICTIONARY_JSON).expect("bundled ags5_dictionary.json must parse");
        let mut entries: Vec<(String, GroupDescriptor)> = Vec::with_capacity(file.groups.len());
        let mut by_code = HashMap::with_capacity(file.groups.len());
        for g in file.groups {
            by_code.insert(g.code.clone(), entries.len());
            entries.push((g.code.clone(), g));
        }
        Registry { entries, by_code }
    })
}

/// Return the parent → child chain ending at `code` (i.e. `[root, ...,
/// parent, code]`). Mirrors `_ddl._ancestor_chain` reversed; the DDL
/// uses the reversed form (g first, then parents) but `migrate` uses
/// the root-down form when joining parent columns. Both are useful.
pub fn ancestor_chain<'a>(reg: &'a Registry, code: &str) -> Vec<&'a GroupDescriptor> {
    let mut chain: Vec<&'a GroupDescriptor> = Vec::new();
    let mut cursor = match reg.get(code) {
        Some(g) => g,
        None => return chain,
    };
    chain.push(cursor);
    while let Some(p) = cursor.parent.as_deref() {
        match reg.get(p) {
            Some(parent) => {
                chain.push(parent);
                cursor = parent;
            }
            None => break,
        }
    }
    chain
}

/// The KEY heading names a group inherits from its parent (Python's
/// `_inherited_key_names`). These are dropped from the child's typed
/// columns — the view JOIN-s them through from the parent.
pub fn inherited_key_names(
    reg: &Registry,
    g: &GroupDescriptor,
) -> std::collections::HashSet<String> {
    let mut out = std::collections::HashSet::new();
    let parent_code = match g.parent.as_deref() {
        Some(p) => p,
        None => return out,
    };
    let parent = match reg.get(parent_code) {
        Some(p) => p,
        None => return out,
    };
    let parent_keys: std::collections::HashSet<&str> =
        parent.key_headings().map(|h| h.name.as_str()).collect();
    for h in g.key_headings() {
        if parent_keys.contains(h.name.as_str()) {
            out.insert(h.name.clone());
        }
    }
    out
}

/// For a group's KEY heading, find the depth in the ancestor chain where
/// the heading is *physically stored* (Python's `_heading_storage_index`).
/// 0 = stored on `g` itself; N = stored on the Nth ancestor up.
///
/// A heading is owned by the topmost ancestor whose KEY headings include
/// it before the chain breaks. Walking up: for each ancestor, if it has
/// the heading as a KEY, it's a candidate; the first ancestor that does
/// NOT have it tells us the previous one is the owner.
pub fn heading_storage_index(reg: &Registry, g: &GroupDescriptor, heading_name: &str) -> usize {
    let chain = ancestor_chain(reg, &g.code);
    let mut storage_idx = 0;
    for (i, ancestor) in chain.iter().enumerate() {
        if ancestor.key_headings().any(|h| h.name == heading_name) {
            storage_idx = i;
        } else {
            break;
        }
    }
    storage_idx
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_loads() {
        let reg = registry();
        assert!(!reg.is_empty(), "registry is empty");
        let proj = reg.get("PROJ").expect("PROJ must exist");
        assert_eq!(proj.code, "PROJ");
        assert!(proj.parent.is_none(), "PROJ is the root");
    }

    #[test]
    fn loca_parent_is_proj() {
        let reg = registry();
        let loca = reg.get("LOCA").expect("LOCA must exist");
        assert_eq!(loca.parent.as_deref(), Some("PROJ"));
    }

    #[test]
    fn ancestor_chain_includes_root() {
        let reg = registry();
        let chain = ancestor_chain(reg, "TREL");
        // TREL parent chain: TREL -> TREG -> SAMP -> LOCA -> PROJ
        let codes: Vec<&str> = chain.iter().map(|g| g.code.as_str()).collect();
        assert_eq!(codes.first(), Some(&"TREL"));
        assert_eq!(codes.last(), Some(&"PROJ"));
    }

    #[test]
    fn inherited_keys_for_samp_include_loca_id() {
        let reg = registry();
        let samp = reg.get("SAMP").expect("SAMP must exist");
        let inherited = inherited_key_names(reg, samp);
        assert!(
            inherited.contains("LOCA_ID"),
            "SAMP should inherit LOCA_ID from LOCA, got {:?}",
            inherited,
        );
    }

    #[test]
    fn no_group_contents_carries_a_bare_edition_tag() {
        // TRIL once read "Triaxial Test Logged Data (AGS 4.2)" — a mislabel:
        // it (with CONL/TREL) is an AGS-L draft group, NOT part of AGS4 4.2.
        // Guard the whole table against re-introducing a bare "(AGS 4.x)" tag;
        // provenance belongs in an explicit "AGS-L draft" marker instead.
        for g in registry().iter() {
            assert!(
                !g.contents.contains("(AGS 4."),
                "{} carries a bare edition tag in its contents: {:?}",
                g.code,
                g.contents,
            );
        }
    }

    #[test]
    fn agsl_draft_groups_are_flagged_as_such() {
        let reg = registry();
        for code in ["CONL", "TREL", "TRIL"] {
            let g = reg.get(code).unwrap_or_else(|| panic!("{code} must exist"));
            assert!(
                g.contents.contains("AGS-L"),
                "{code} is an AGS-L draft group; its contents must say so, got {:?}",
                g.contents,
            );
        }
    }
}
