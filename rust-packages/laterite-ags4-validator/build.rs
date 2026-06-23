//! Compile-time codegen: the consolidated union dictionary
//! (`laterite-ags4-core/data/ags_dictionary.json`) → `phf` perfect-hash static
//! tables in `OUT_DIR/dict_data.rs`.
//!
//! Runs once per build. Output is `include!`d by `src/dict.rs`, so the validator
//! pays zero startup cost and never parses the dictionary at runtime.
//!
//! SINGLE SOURCE OF TRUTH: the union JSON is the one machine-readable form of the
//! AGS standard dictionary, generated from the official `.ags` files by
//! `tools/gen_dictionary.py` (the sole `.ags` reader). This build projects each
//! edition out of that union — the same per-edition reconstruction
//! `gen_dictionary.reconstruct` does — so the validator and every other consumer
//! read ONE artifact and cannot drift. (Previously this parsed the five `.ags`
//! files directly; that made build.rs a second, independent reader of the spec.)

use std::collections::HashMap;
use std::env;
use std::fmt::Write as _;
use std::fs;
use std::io::Write as _;
use std::path::Path;

use serde_json::Value;

fn main() {
    let out_dir = env::var("OUT_DIR").expect("OUT_DIR set by cargo");
    let dest = Path::new(&out_dir).join("dict_data.rs");

    // The union lives in the sibling core crate. publish=false + a workspace-only
    // crate, so a workspace-relative path is safe (it is preserved in the maturin
    // sdist, which vendors path-dep crates as siblings).
    let manifest = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR set by cargo");
    let union_path = Path::new(&manifest).join("../laterite-ags4-core/data/ags_dictionary.json");
    println!("cargo:rerun-if-changed={}", union_path.display());

    let text = fs::read_to_string(&union_path)
        .unwrap_or_else(|e| panic!("read {}: {e}", union_path.display()));
    let doc: Value = serde_json::from_str(&text).expect("ags_dictionary.json must parse");

    let editions: Vec<String> = doc["editions"]
        .as_array()
        .expect("editions array")
        .iter()
        .map(|v| v.as_str().expect("edition string").to_string())
        .collect();

    let default_edition = doc["default_edition"]
        .as_str()
        .expect("default_edition string");
    let fallback_edition = doc["fallback_edition"]
        .as_str()
        .expect("fallback_edition string");

    let mut body = String::new();
    // The `DictVersion` enum + edition list + FALLBACK are generated from the
    // union too, so the validator / wasm / web never hand-copy the edition set.
    emit_dict_version(&mut body, &editions, default_edition, fallback_edition);
    for ed in &editions {
        let ident = ed.replace('.', "_"); // "4.0.3" -> "4_0_3"
        emit_version(&mut body, &ident, ed, &doc);
    }

    let mut f = fs::File::create(&dest).expect("create dict_data.rs");
    f.write_all(body.as_bytes()).expect("write dict_data.rs");
}

/// Emit the `DictVersion` enum + `as_str`/`ALL`/`from_edition`/`tables` +
/// `FALLBACK`, generated from the union's `editions`/`default_edition`/
/// `fallback_edition`. This single-sources the edition SET: the validator, wasm,
/// and web never hand-copy `["4.0.3" … "4.2"]` — they reference these.
fn emit_dict_version(out: &mut String, editions: &[String], default_ed: &str, fallback_ed: &str) {
    let variant = |ed: &str| format!("V{}", ed.replace('.', "_"));

    writeln!(
        out,
        "/// Which bundled standard dictionary to validate against."
    )
    .unwrap();
    writeln!(
        out,
        "/// GENERATED from ags_dictionary.json `editions` — add an edition to the"
    )
    .unwrap();
    writeln!(
        out,
        "/// official source dictionaries + regenerate; do not hand-edit."
    )
    .unwrap();
    writeln!(out, "#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]").unwrap();
    writeln!(out, "pub enum DictVersion {{").unwrap();
    for ed in editions {
        if ed == default_ed {
            writeln!(out, "    #[default]").unwrap();
        }
        writeln!(out, "    {},", variant(ed)).unwrap();
    }
    writeln!(out, "}}\n").unwrap();

    // The five generated statics for one edition, in `Dictionary` field order.
    writeln!(out, "type DictTables = (").unwrap();
    writeln!(out, "    &'static phf::Map<&'static str, DictEntry>,").unwrap();
    writeln!(out, "    &'static phf::Map<&'static str, GroupMeta>,").unwrap();
    writeln!(
        out,
        "    &'static phf::Map<&'static str, &'static [&'static str]>,"
    )
    .unwrap();
    writeln!(out, "    &'static phf::Map<&'static str, &'static str>,").unwrap();
    writeln!(out, "    &'static str,").unwrap();
    writeln!(out, ");\n").unwrap();

    writeln!(out, "impl DictVersion {{").unwrap();
    writeln!(out, "    pub fn as_str(self) -> &'static str {{").unwrap();
    writeln!(out, "        match self {{").unwrap();
    for ed in editions {
        writeln!(out, "            DictVersion::{} => {ed:?},", variant(ed)).unwrap();
    }
    writeln!(out, "        }}").unwrap();
    writeln!(out, "    }}").unwrap();

    let all = editions
        .iter()
        .map(|e| format!("DictVersion::{}", variant(e)))
        .collect::<Vec<_>>()
        .join(", ");
    writeln!(out, "    /// All bundled editions, oldest → newest.").unwrap();
    writeln!(out, "    pub const ALL: &'static [DictVersion] = &[{all}];").unwrap();

    writeln!(
        out,
        "    /// Parse an exact edition label (e.g. \"4.2\"); `None` if unrecognised."
    )
    .unwrap();
    writeln!(
        out,
        "    pub fn from_edition(s: &str) -> Option<DictVersion> {{"
    )
    .unwrap();
    writeln!(out, "        match s {{").unwrap();
    for ed in editions {
        writeln!(
            out,
            "            {ed:?} => Some(DictVersion::{}),",
            variant(ed)
        )
        .unwrap();
    }
    writeln!(out, "            _ => None,").unwrap();
    writeln!(out, "        }}").unwrap();
    writeln!(out, "    }}").unwrap();

    writeln!(
        out,
        "    /// The five compiled lookup tables for this edition."
    )
    .unwrap();
    writeln!(out, "    fn tables(self) -> DictTables {{").unwrap();
    writeln!(out, "        match self {{").unwrap();
    for ed in editions {
        let id = ed.replace('.', "_");
        writeln!(
            out,
            "            DictVersion::{} => (&DICT_{id}_HEADINGS, &DICT_{id}_GROUPS, &DICT_{id}_GROUP_HEADINGS, &DICT_{id}_ABBRS, DICT_{id}_TRAN_AGS),",
            variant(ed)
        )
        .unwrap();
    }
    writeln!(out, "        }}").unwrap();
    writeln!(out, "    }}").unwrap();
    writeln!(out, "}}\n").unwrap();

    writeln!(
        out,
        "/// Auto-selection fallback (python-parity `fallback_edition`). GENERATED."
    )
    .unwrap();
    writeln!(
        out,
        "pub const FALLBACK: DictVersion = DictVersion::{};\n",
        variant(fallback_ed)
    )
    .unwrap();
}

/// Project one edition out of the union and append its five statics to `out`.
/// The projection mirrors `tools/gen_dictionary.py::reconstruct` exactly (flat
/// fields are the latest-edition value; `by_ed` overlays older editions; `eds`
/// gates membership; `order_by_ed`/`*_by_ed` override group meta + order), so the
/// emitted tables are byte-identical to the old direct-`.ags` parse — which is
/// what keeps validation behaviour (and python-ags4 parity) unchanged.
fn emit_version(out: &mut String, ident: &str, ed: &str, doc: &Value) {
    // Trim to match the old `.ags` reader (which trimmed every cell); the union
    // values are clean, so this is a no-op in practice but guarantees parity.
    let s = |v: &Value| v.as_str().unwrap_or("").trim().to_string();
    // Edition membership: a missing/null `eds` means "every edition this entry
    // spans" (the heading is in all of its group's editions; the abbr in all).
    let in_ed = |eds: Option<&Value>| -> bool {
        match eds {
            None | Some(Value::Null) => true,
            Some(v) => v
                .as_array()
                .is_none_or(|a| a.iter().any(|x| x.as_str() == Some(ed))),
        }
    };
    // A heading/group field for this edition: `by_ed[ed][key]` if present
    // (string, or null → ""), else the flat value.
    let pick = |obj: &Value, by: Option<&Value>, key: &str| -> String {
        match by.and_then(|o| o.get(key)) {
            Some(v) => s(v),
            None => s(obj.get(key).unwrap_or(&Value::Null)),
        }
    };

    // group code -> (parent, desc)
    let mut groups: HashMap<String, (String, String)> = HashMap::new();
    // "GROUP\u{1f}HEADING" -> (ags_type, unit, status, desc)
    let mut headings: HashMap<String, (String, String, String, String)> = HashMap::new();
    // group code -> heading names in dictionary order (Rule 7)
    let mut group_hdng_order: HashMap<String, Vec<String>> = HashMap::new();

    for (code, g) in doc["groups"].as_object().expect("groups object") {
        if !in_ed(g.get("eds")) {
            continue;
        }
        // group meta: *_by_ed[ed] overrides the flat value; null parent / "-" = root
        let g_by_parent = g.get("parent_by_ed").and_then(|o| o.get(ed));
        let mut parent = match g_by_parent {
            Some(v) => s(v),
            None => s(g.get("parent").unwrap_or(&Value::Null)),
        };
        if parent == "-" {
            parent.clear();
        }
        let g_by_desc = g.get("desc_by_ed").and_then(|o| o.get(ed));
        let gdesc = match g_by_desc {
            Some(v) => s(v),
            None => s(g.get("description").unwrap_or(&Value::Null)),
        };
        groups.insert(code.clone(), (parent, gdesc));

        let mut order: Vec<String> = Vec::new();
        for h in g["headings"].as_array().expect("headings array") {
            if !in_ed(h.get("eds")) {
                continue;
            }
            let name = s(&h["name"]);
            let by = h.get("by_ed").and_then(|o| o.get(ed));
            let ags_type = pick(h, by, "type");
            let unit = pick(h, by, "unit");
            let status = pick(h, by, "status");
            let desc = pick(h, by, "description");
            headings.insert(
                format!("{code}\u{1f}{name}"),
                (ags_type, unit, status, desc),
            );
            order.push(name);
        }
        // An explicit per-edition order overrides the derived one.
        if let Some(ob) = g
            .get("order_by_ed")
            .and_then(|o| o.get(ed))
            .and_then(|v| v.as_array())
        {
            order = ob.iter().map(&s).collect();
        }
        group_hdng_order.insert(code.clone(), order);
    }

    // ABBR pick-list: "ABBR_HDNG\u{1f}ABBR_CODE" -> ABBR_DESC for this edition.
    let mut abbrs: HashMap<String, String> = HashMap::new();
    if let Some(arr) = doc.get("abbreviations").and_then(|v| v.as_array()) {
        for a in arr {
            if !in_ed(a.get("eds")) {
                continue;
            }
            let hdng = s(&a["heading"]);
            let code = s(&a["code"]);
            let desc = match a.get("by_ed").and_then(|o| o.get(ed)) {
                Some(v) => s(v),
                None => s(a.get("description").unwrap_or(&Value::Null)),
            };
            if !hdng.is_empty() && !code.is_empty() {
                abbrs.insert(format!("{hdng}\u{1f}{code}"), desc);
            }
        }
    }

    let tran_ags = s(doc
        .get("tran_ags")
        .and_then(|o| o.get(ed))
        .unwrap_or(&Value::Null));

    assert!(
        headings.len() > 1000,
        "dict {ident}: only {} headings projected — codegen likely broken",
        headings.len()
    );
    assert!(!tran_ags.is_empty(), "dict {ident}: TRAN_AGS not found");

    // Build the phf maps. Deterministic order (sorted) so the generated file is
    // reproducible / diff-stable. phf_codegen::Map::entry borrows the value
    // `&str` until `.build()`, so the formatted expression strings must outlive
    // the builder — materialise them into owned Vecs first.
    let mut hkeys: Vec<&String> = headings.keys().collect();
    hkeys.sort();
    let h_entries: Vec<(&str, String)> = hkeys
        .iter()
        .map(|k| {
            let (t, u, s, d) = &headings[*k];
            (
                k.as_str(),
                format!("DictEntry {{ ags_type: {t:?}, unit: {u:?}, status: {s:?}, desc: {d:?} }}"),
            )
        })
        .collect();
    let mut hmap = phf_codegen::Map::new();
    for (k, v) in &h_entries {
        hmap.entry(*k, v);
    }

    let mut gkeys: Vec<&String> = groups.keys().collect();
    gkeys.sort();
    let g_entries: Vec<(&str, String)> = gkeys
        .iter()
        .map(|k| {
            let (p, d) = &groups[*k];
            (
                k.as_str(),
                format!("GroupMeta {{ parent: {p:?}, desc: {d:?} }}"),
            )
        })
        .collect();
    let mut gmap = phf_codegen::Map::new();
    for (k, v) in &g_entries {
        gmap.entry(*k, v);
    }

    let mut okeys: Vec<&String> = group_hdng_order.keys().collect();
    okeys.sort();
    let o_entries: Vec<(&str, String)> = okeys
        .iter()
        .map(|k| {
            let list = &group_hdng_order[*k];
            let items: Vec<String> = list.iter().map(|h| format!("{h:?}")).collect();
            (k.as_str(), format!("&[{}]", items.join(", ")))
        })
        .collect();
    let mut omap = phf_codegen::Map::new();
    for (k, v) in &o_entries {
        omap.entry(*k, v);
    }

    let mut akeys: Vec<&String> = abbrs.keys().collect();
    akeys.sort();
    let a_entries: Vec<(&str, String)> = akeys
        .iter()
        .map(|k| (k.as_str(), format!("{:?}", abbrs[*k])))
        .collect();
    let mut amap = phf_codegen::Map::new();
    for (k, v) in &a_entries {
        amap.entry(*k, v);
    }

    writeln!(
        out,
        "static DICT_{ident}_HEADINGS: phf::Map<&'static str, DictEntry> = {};",
        hmap.build()
    )
    .unwrap();
    writeln!(
        out,
        "static DICT_{ident}_GROUPS: phf::Map<&'static str, GroupMeta> = {};",
        gmap.build()
    )
    .unwrap();
    writeln!(
        out,
        "static DICT_{ident}_GROUP_HEADINGS: \
         phf::Map<&'static str, &'static [&'static str]> = {};",
        omap.build()
    )
    .unwrap();
    writeln!(
        out,
        "static DICT_{ident}_ABBRS: phf::Map<&'static str, &'static str> = {};",
        amap.build()
    )
    .unwrap();
    writeln!(out, "static DICT_{ident}_TRAN_AGS: &str = {tran_ags:?};\n").unwrap();
}
