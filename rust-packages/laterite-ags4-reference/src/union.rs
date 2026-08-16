//! AGS4 group registry.
//!
//! Reconstructs the UNION of all editions (latest-edition heading definitions)
//! from the compiled `phf` tables in [`crate::dict`], and caches it in a
//! singleton at first access. Provides `GroupDescriptor` + `Heading` structs and
//! the parent-chain walks the DDL builder and migrate command need.
//!
//! It used to reconstruct that union by parsing an `include_str!`d copy of
//! `ags_dictionary.json` — a second embedded dictionary sitting beside the
//! tables projected from it. See the note below for what that cost.

use std::collections::HashMap;
use std::sync::OnceLock;

use serde::{Deserialize, Serialize};

// The consolidated multi-edition dictionary (heading-local layout), generated
// from the official AGS standard dictionaries by `tools/gen_dictionary.py`, is
// the SSOT — but it is consumed at BUILD time, not shipped.
//
// It used to be `include_str!`d here and parsed on first `registry()` call. That
// put 1.4 MB of JSON — descriptions, keys, indentation — into every artifact
// that touched the registry: the wasm bundle, the wheel's `.so`, `lat`, the Node
// addon. And the same facts were ALREADY in the binary, as the `phf` tables
// `build.rs` projects out of that very file. The registry now reconstructs the
// union from those tables (`dict::union_view`), so the dictionary is embedded
// once instead of twice.
//
// It hid well: JSON that repetitive compresses ~18:1, so the duplicate cost
// 1.4 MB raw and only ~82 KB of what a browser actually downloads. Raw size is
// what found it.
//
// #475 PR2: the SSOT JSON physically lives in this leaf's own `data/` — every
// other reader (this crate's build.rs, laterite-py's build.rs, the node
// typed-graph codegen, the web sync scripts, the Python generators) reads it
// cross-crate/cross-package from there, at build time.

// `PartialEq`/`Eq` are test-only on purpose. The parity oracle below compares
// whole reconstructed groups against the JSON document, which wants equality —
// but this crate is published, so crates.io would freeze the impls as surface
// this change never set out to add. If a consumer ever needs to compare
// headings, that is a deliberate API decision, not a side effect of a test.
#[cfg_attr(test, derive(PartialEq, Eq))]
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
}

impl Heading {
    #[must_use]
    pub fn is_key(&self) -> bool {
        // The official dict uses combined statuses (e.g. "KEY+REQUIRED"); a
        // heading is a KEY iff "KEY" is one of the `+`-separated parts.
        self.status
            .split('+')
            .any(|p| p.eq_ignore_ascii_case("KEY"))
    }
}

#[cfg_attr(test, derive(PartialEq, Eq))] // test-only; see `Heading` above
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GroupDescriptor {
    pub code: String,
    pub contents: String,
    pub parent: Option<String>,
    pub headings: Vec<Heading>,
}

impl GroupDescriptor {
    pub fn key_headings(&self) -> impl Iterator<Item = &Heading> {
        self.headings.iter().filter(|h| h.is_key())
    }
}

/// The heading-local on-disk schema of `ags_dictionary.json`. The flat heading
/// fields ARE each heading's latest-edition definition; the `by_ed`/`eds`
/// per-edition variation is intentionally not captured (serde drops the unknown
/// fields) — the registry consumes the UNION at the latest-edition definition.
// `pub(crate)` (not private): the custom-dictionary overlay (#568, `overlay.rs`
// + `dict_read.rs`) deserialises a `--dict` JSON straight into these structs, so
// the custom-JSON schema and the bundled `ags_dictionary.json` schema can never
// drift — there is one serde definition, not two. `Clone` so a parsed
// `DictionaryFile` can live on an owned `CustomDict`.
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct DictHeading {
    pub(crate) name: String,
    pub(crate) status: String,
    #[serde(rename = "type")]
    pub(crate) ags_type: String,
    #[serde(default)]
    pub(crate) unit: Option<String>,
    #[serde(default)]
    pub(crate) description: String,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct DictGroup {
    pub(crate) parent: Option<String>,
    #[serde(default)]
    pub(crate) description: Option<String>,
    pub(crate) headings: Vec<DictHeading>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct DictionaryFile {
    // `#[serde(default)]`: the bundled dictionary always carries these, but a
    // hand-authored `--dict` JSON — the format we pitch *for* editability —
    // needn't declare `format_version`/`editions` just to add one group.
    #[allow(dead_code)]
    #[serde(default)]
    pub(crate) format_version: String,
    #[allow(dead_code)]
    #[serde(default)]
    pub(crate) editions: Vec<String>,
    pub(crate) groups: HashMap<String, DictGroup>,
}

impl DictGroup {
    /// An empty group shell the runtime `.ags` reader fills incrementally as it
    /// walks GROUP / HEADING rows (#568 `dict_read`).
    pub(crate) fn empty() -> Self {
        DictGroup {
            parent: None,
            description: None,
            headings: Vec::new(),
        }
    }
}

impl DictionaryFile {
    /// Wrap a runtime-built group map as a `DictionaryFile` (the `.ags` reader
    /// has no `format_version`/`editions` to declare — a custom dict needn't).
    pub(crate) fn from_groups(groups: HashMap<String, DictGroup>) -> Self {
        DictionaryFile {
            format_version: String::new(),
            editions: Vec::new(),
            groups,
        }
    }
}

/// Parent-depth of `code` (PROJ-rooted; root = 0). Orders groups
/// parents-before-children — the invariant the DDL emission relies on.
///
/// `parent_of` returns `None` for a root group AND for a code the dictionary
/// does not contain, which is what stops the walk in both cases.
fn group_depth(parent_of: &HashMap<&str, Option<&str>>, code: &str) -> usize {
    let (mut depth, mut cur, mut guard) = (0, code, 0);
    while let Some(Some(p)) = parent_of.get(cur) {
        if !parent_of.contains_key(p) {
            break;
        }
        depth += 1;
        cur = p;
        guard += 1;
        if guard > 64 {
            break; // cycle guard — should never fire
        }
    }
    depth
}

/// The UNION group set (every heading across editions, each at its
/// latest-edition definition), ordered parents-before-children and
/// deterministically by `(depth, code)`. Shared by `registry()` and the
/// `build.rs` typed-class codegen so the typed graph, the DDL, and the registry
/// single-source one reconstruction of the heading-local dictionary.
#[must_use]
pub fn union_groups() -> Vec<GroupDescriptor> {
    // The union comes from the compiled `phf` tables, not from a second embedded
    // copy of the dictionary — see the note at the top of this file. `dict`
    // owns the edition masks and answers "latest-edition definition"; the
    // ordering and the owned `GroupDescriptor` shape are this module's job.
    let mut groups = crate::dict::union_view::groups();

    let parent_of: HashMap<&str, Option<&str>> = groups
        .iter()
        .map(|g| (g.code, (!g.parent.is_empty()).then_some(g.parent)))
        .collect();
    groups.sort_by(|a, b| {
        group_depth(&parent_of, a.code)
            .cmp(&group_depth(&parent_of, b.code))
            .then_with(|| a.code.cmp(b.code))
    });

    groups
        .into_iter()
        .map(|g| GroupDescriptor {
            code: g.code.to_string(),
            contents: g.desc.to_string(),
            // `GroupMeta` stores a root's parent as `""`; the registry's shape
            // is `Option`, and the JSON it replaced wrote `null`.
            parent: (!g.parent.is_empty()).then(|| g.parent.to_string()),
            headings: g
                .headings
                .into_iter()
                .map(|(name, e)| Heading {
                    name: name.to_string(),
                    status: e.status.to_string(),
                    ags_type: e.ags_type.to_string(),
                    // Same asymmetry as `parent`: the table stores "no unit" as
                    // `""`, the registry as `None`.
                    unit: (!e.unit.is_empty()).then(|| e.unit.to_string()),
                    description: e.desc.to_string(),
                })
                .collect(),
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
    #[must_use]
    pub fn get(&self, code: &str) -> Option<&GroupDescriptor> {
        self.by_code.get(code).map(|i| &self.entries[*i].1)
    }

    pub fn iter(&self) -> impl Iterator<Item = &GroupDescriptor> {
        self.entries.iter().map(|(_, g)| g)
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Serialise every group as a JSON array, in declaration order. The
    /// per-group field order matches the on-disk dictionary's
    /// declaration order (`code, contents, parent, headings`)
    /// thanks to `serde_json`'s
    /// `preserve_order` feature + the struct field ordering. Used by
    /// `laterite.registry` to ferry the registry across the PyO3
    /// boundary without rebuilding a parallel schema (Stage D2 of
    #[must_use]
    pub fn to_groups_json(&self) -> String {
        let groups: Vec<&GroupDescriptor> = self.iter().collect();
        serde_json::to_string(&groups).expect("registry to_json must succeed")
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
#[must_use]
pub fn ancestor_chain<'a>(reg: &'a Registry, code: &str) -> Vec<&'a GroupDescriptor> {
    let mut chain: Vec<&'a GroupDescriptor> = Vec::new();
    let Some(mut cursor) = reg.get(code) else {
        return chain;
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
#[must_use]
pub fn inherited_key_names(
    reg: &Registry,
    g: &GroupDescriptor,
) -> std::collections::HashSet<String> {
    let mut out = std::collections::HashSet::new();
    let Some(parent_code) = g.parent.as_deref() else {
        return out;
    };
    let Some(parent) = reg.get(parent_code) else {
        return out;
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
            "SAMP should inherit LOCA_ID from LOCA, got {inherited:?}",
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

    #[test]
    fn group_depth_counts_in_map_ancestors_and_guards_cycles() {
        use std::collections::HashMap;
        // `parent_of` now carries borrowed codes rather than the parsed
        // `DictGroup`s — the map is built from the compiled tables, not from a
        // deserialised document. Same three cases either way.
        let groups: HashMap<&str, Option<&str>> = HashMap::from([
            ("PROJ", None),
            ("LOCA", Some("PROJ")),
            ("SAMP", Some("LOCA")),
            ("ORPH", Some("GHOST")), // parent not in the map
        ]);
        assert_eq!(group_depth(&groups, "PROJ"), 0);
        assert_eq!(group_depth(&groups, "LOCA"), 1);
        assert_eq!(group_depth(&groups, "SAMP"), 2);
        // an ancestor that isn't in the map neither counts nor recurses
        assert_eq!(group_depth(&groups, "ORPH"), 0);

        // a cycle must terminate at the 64-step guard, not spin forever
        let cyc: HashMap<&str, Option<&str>> = HashMap::from([("A", Some("B")), ("B", Some("A"))]);
        assert_eq!(group_depth(&cyc, "A"), 65);
    }

    #[test]
    fn registry_len_is_empty_and_json_reflect_the_group_set() {
        use std::collections::HashMap;
        let reg = registry();
        assert!(
            reg.len() > 100,
            "the union is ~174 groups, got {}",
            reg.len()
        );
        assert!(!reg.is_empty());
        assert_eq!(reg.len(), reg.iter().count());
        // an actually-empty registry reports empty / zero (the real one never is)
        let empty = Registry {
            entries: Vec::new(),
            by_code: HashMap::new(),
        };
        assert!(empty.is_empty());
        assert_eq!(empty.len(), 0);
        // the JSON is a non-empty array that names the groups
        let json = reg.to_groups_json();
        assert!(json.starts_with('['), "not a JSON array: {}", &json[..20]);
        assert!(json.contains("PROJ"));
    }
}

#[cfg(test)]
mod union_parity_tests {
    //! The reconstruction, held against the document it replaced.
    //!
    //! `union_groups()` used to parse `ags_dictionary.json` at runtime; it now
    //! rebuilds the same answer from the `phf` tables `build.rs` projects out of
    //! that file. Those are two different readings of one source, and the only
    //! thing that makes swapping them safe is comparing them — the JSON is
    //! ~3,500 headings across 174 groups, and "spot-check PROJ and LOCA" would
    //! not notice a heading whose edition set made its latest definition differ.
    //!
    //! So the JSON stays, as an ORACLE rather than a source: `include_str!`
    //! inside `#[cfg(test)]` reaches the test binary and no shipped artifact —
    //! which is the whole point of the change, and is itself asserted below.
    use super::*;

    const ORACLE_JSON: &str = include_str!("../data/ags_dictionary.json");

    /// `union_groups()` exactly as it read before the swap, kept verbatim so the
    /// comparison is against the old BEHAVIOUR and not a fresh interpretation of
    /// what it was supposed to do.
    fn oracle() -> Vec<GroupDescriptor> {
        fn depth(groups: &HashMap<String, DictGroup>, code: &str) -> usize {
            let (mut depth, mut cur, mut guard) = (0, code, 0);
            while let Some(g) = groups.get(cur) {
                match g.parent.as_deref() {
                    Some(p) if groups.contains_key(p) => {
                        depth += 1;
                        cur = p;
                        guard += 1;
                        if guard > 64 {
                            break;
                        }
                    }
                    _ => break,
                }
            }
            depth
        }

        let file: DictionaryFile =
            serde_json::from_str(ORACLE_JSON).expect("bundled ags_dictionary.json must parse");
        let mut codes: Vec<&String> = file.groups.keys().collect();
        codes.sort_by(|a, b| {
            depth(&file.groups, a)
                .cmp(&depth(&file.groups, b))
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
                        })
                        .collect(),
                }
            })
            .collect()
    }

    #[test]
    fn the_tables_rebuild_the_document_exactly() {
        let from_tables = union_groups();
        let from_json = oracle();

        // Compared per group before the whole-vector assert: a mismatch deep in
        // 174 groups renders as an unreadable wall of Debug otherwise, and the
        // first differing group is what a reader needs.
        assert_eq!(
            from_tables.len(),
            from_json.len(),
            "group COUNT differs: tables {} vs json {}",
            from_tables.len(),
            from_json.len()
        );
        for (t, j) in from_tables.iter().zip(from_json.iter()) {
            assert_eq!(t.code, j.code, "group ORDER differs");
            assert_eq!(t.parent, j.parent, "{}: parent differs", t.code);
            assert_eq!(t.contents, j.contents, "{}: contents differ", t.code);
            let names = |g: &GroupDescriptor| -> Vec<String> {
                g.headings.iter().map(|h| h.name.clone()).collect()
            };
            assert_eq!(
                names(t),
                names(j),
                "{}: heading names or their ORDER differ",
                t.code
            );
            for (th, jh) in t.headings.iter().zip(j.headings.iter()) {
                assert_eq!(th, jh, "{}.{}: definition differs", t.code, th.name);
            }
        }
        assert_eq!(from_tables, from_json);
    }

    #[test]
    fn the_shipped_crate_no_longer_embeds_the_dictionary_json() {
        // The regression this change exists to prevent: someone re-adds an
        // `include_str!` of the dictionary to get at a field the tables do not
        // carry, and 1.4 MB silently returns to every binary. It compresses ~18:1,
        // so no size gate downstream would notice.
        //
        // Asserted against the SOURCE rather than a built artifact because this
        // crate's tests cannot see the wasm/wheel/CLI links, and because the
        // source is where the mistake is made.
        const SRC: &str = include_str!("union.rs");
        let this_module = SRC
            .find("mod union_parity_tests")
            .expect("this module is in the source it reads");
        assert_eq!(
            SRC.match_indices("include_str!(\"../data/ags_dictionary.json\")")
                .count(),
            SRC[this_module..]
                .match_indices("include_str!(\"../data/ags_dictionary.json\")")
                .count(),
            "the dictionary JSON is embedded outside this test module — it is \
             ~1.4 MB, it duplicates the phf tables, and it compresses ~18:1 so \
             nothing downstream will flag it. Read the union from `dict::union_view`."
        );
    }
}
