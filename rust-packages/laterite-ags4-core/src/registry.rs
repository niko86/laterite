//! AGS4 group registry.
//!
//! Loads the bundled multi-edition `ags_dictionary.json` (embedded via
//! `include_str!`), reconstructs the UNION of all editions (latest-edition
//! heading definitions), and caches it in a singleton at first access.
//! Provides `GroupDescriptor` + `Heading` structs and the parent-chain walks
//! the DDL builder and migrate command need.

use std::collections::HashMap;
use std::sync::OnceLock;

use serde::{Deserialize, Serialize};

// The consolidated multi-edition dictionary (heading-local layout), generated
// from the official AGS standard dictionaries by `tools/gen_dictionary.py`. The
// registry / typed graph / DDL consume the UNION of every heading across
// editions, each at its latest-edition definition — which is exactly the flat
// heading fields of the heading-local schema (the `by_ed`/`eds` per-edition
// variation is ignored here; edition-aware consumers reconstruct a specific
// edition separately). A faithfulness gate (tests/test_dictionary_faithful.py)
// keeps this file == the official-projection generator output.
const DICTIONARY_JSON: &str = include_str!("../data/ags_dictionary.json");

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
        // The official dict uses combined statuses (e.g. "KEY+REQUIRED"); a
        // heading is a KEY iff "KEY" is one of the `+`-separated parts.
        self.status
            .split('+')
            .any(|p| p.eq_ignore_ascii_case("KEY"))
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GroupDescriptor {
    pub code: String,
    pub contents: String,
    pub parent: Option<String>,
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

/// The heading-local on-disk schema of `ags_dictionary.json`. The flat heading
/// fields ARE each heading's latest-edition definition; the `by_ed`/`eds`
/// per-edition variation is intentionally not captured (serde drops the unknown
/// fields) — the registry consumes the UNION at the latest-edition definition.
#[derive(Debug, Deserialize)]
struct DictHeading {
    name: String,
    status: String,
    #[serde(rename = "type")]
    ags_type: String,
    #[serde(default)]
    unit: Option<String>,
    #[serde(default)]
    description: String,
}

#[derive(Debug, Deserialize)]
struct DictGroup {
    parent: Option<String>,
    #[serde(default)]
    description: Option<String>,
    headings: Vec<DictHeading>,
}

#[derive(Debug, Deserialize)]
struct DictionaryFile {
    #[allow(dead_code)]
    format_version: String,
    #[allow(dead_code)]
    editions: Vec<String>,
    groups: HashMap<String, DictGroup>,
}

/// Parent-depth of `code` (PROJ-rooted; root = 0). Orders groups
/// parents-before-children — the invariant the DDL emission relies on.
fn group_depth(groups: &HashMap<String, DictGroup>, code: &str) -> usize {
    let (mut depth, mut cur, mut guard) = (0, code, 0);
    while let Some(g) = groups.get(cur) {
        match g.parent.as_deref() {
            Some(p) if groups.contains_key(p) => {
                depth += 1;
                cur = p;
                guard += 1;
                if guard > 64 {
                    break; // cycle guard — should never fire
                }
            }
            _ => break,
        }
    }
    depth
}

/// The UNION group set (every heading across editions, each at its
/// latest-edition definition), ordered parents-before-children and
/// deterministically by `(depth, code)`. Shared by `registry()` and the
/// `build.rs` typed-class codegen so the typed graph, the DDL, and the registry
/// single-source one reconstruction of the heading-local dictionary.
pub fn union_groups() -> Vec<GroupDescriptor> {
    let file: DictionaryFile =
        serde_json::from_str(DICTIONARY_JSON).expect("bundled ags_dictionary.json must parse");
    let mut codes: Vec<&String> = file.groups.keys().collect();
    codes.sort_by(|a, b| {
        group_depth(&file.groups, a)
            .cmp(&group_depth(&file.groups, b))
            .then_with(|| a.cmp(b))
    });
    codes
        .into_iter()
        .map(|code| {
            let g = &file.groups[code];
            GroupDescriptor {
                code: code.clone(),
                contents: g.description.clone().unwrap_or_default(),
                parent: g.parent.clone(),
                headings: g
                    .headings
                    .iter()
                    .map(|h| Heading {
                        name: h.name.clone(),
                        status: h.status.clone(),
                        ags_type: h.ags_type.clone(),
                        unit: h.unit.clone(),
                        description: h.description.clone(),
                        indexed: None,
                    })
                    .collect(),
                index_parent: None,
            }
        })
        .collect()
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
    /// declaration order (`code, contents, parent, headings,
    /// index_parent?`) thanks to `serde_json`'s
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
        let groups = union_groups();
        let mut entries: Vec<(String, GroupDescriptor)> = Vec::with_capacity(groups.len());
        let mut by_code = HashMap::with_capacity(groups.len());
        for g in groups {
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
        let chain = ancestor_chain(reg, "LLPL");
        // LLPL parent chain: LLPL -> SAMP -> LOCA -> PROJ
        let codes: Vec<&str> = chain.iter().map(|g| g.code.as_str()).collect();
        assert_eq!(codes.first(), Some(&"LLPL"));
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
    fn contents_come_from_the_official_description() {
        // The consolidated dict uses the official AGS group description (no
        // laterite edition tags / "(scaffolded)" placeholders). Spot-check a
        // couple and guard against a bare "(AGS 4.x)" tag creeping back in.
        let reg = registry();
        assert_eq!(
            reg.get("MOND").map(|g| g.contents.as_str()),
            Some("Monitoring Readings")
        );
        for g in reg.iter() {
            assert!(
                !g.contents.contains("(AGS 4.") && !g.contents.contains("(scaffolded)"),
                "{} carries a non-official contents string: {:?}",
                g.code,
                g.contents,
            );
        }
    }

    #[test]
    fn dropped_agsl_drafts_are_absent() {
        // CONL/TREL/TRIL are AGS-L 2026 drafts, not in official 4.0.3-4.2;
        // the faithful dict drops them.
        let reg = registry();
        for code in ["CONL", "TREL", "TRIL"] {
            assert!(
                reg.get(code).is_none(),
                "{code} (AGS-L draft) must be absent"
            );
        }
    }

    #[test]
    fn union_gained_official_groups() {
        // The faithful union has the official groups the old curated subset
        // lacked (~92 -> ~174). Spot-check a few that were missing before.
        let reg = registry();
        for code in ["CPTG", "CBRP", "CTRC"] {
            assert!(
                reg.get(code).is_some(),
                "official group {code} must be present"
            );
        }
    }
}
