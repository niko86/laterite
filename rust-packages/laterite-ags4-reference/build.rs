//! Compile-time codegen: the consolidated union dictionary
//! (this leaf's own `data/ags_dictionary.json`) → `phf` perfect-hash static
//! tables in `OUT_DIR/dict_data.rs`.
//!
//! Runs once per build. Output is `include!`d by `src/dict.rs`, so this leaf
//! (and every consumer of its `dict` module — the validator, wasm, node)
//! pays zero startup cost and never parses the dictionary at runtime. Moved
//! here from `laterite-ags4-validator/build.rs` (#475 PR2) so a consumer that
//! only wants the compiled dictionary needn't depend on the whole rule engine.
//!
//! SINGLE SOURCE OF TRUTH: the union JSON is the one machine-readable form of the
//! AGS standard dictionary, generated from the official `.ags` files by
//! `tools/gen_dictionary.py` (the sole `.ags` reader). This build projects each
//! edition out of that union — the same per-edition reconstruction
//! `gen_dictionary.reconstruct` does — so the validator and every other consumer
//! read ONE artifact and cannot drift. (Previously this parsed the five `.ags`
//! files directly; that made build.rs a second, independent reader of the spec.)

use std::collections::{BTreeMap, HashMap};
use std::env;
use std::fmt::Write as _;
use std::fs;
use std::io::Write as _;
use std::path::Path;

use serde_json::Value;

/// `(parent, desc)` — one group's metadata as projected.
type GroupVal = (String, String);
/// `(ags_type, unit, status, desc)` — one heading's definition as projected.
type HeadingVal = (String, String, String, String);
/// The same table, projected once per edition, tagged with the edition label.
/// This is `fold`'s input shape.
type PerEdition<V> = Vec<(String, HashMap<String, V>)>;

/// One edition's projection of the union — the four tables plus `TRAN_AGS`.
/// Built per edition exactly as before, then folded into the shared tables.
struct Projection {
    /// group code -> (parent, desc)
    groups: HashMap<String, GroupVal>,
    /// `"GROUP\u{1f}HEADING"` -> (`ags_type`, unit, status, desc)
    headings: HashMap<String, HeadingVal>,
    /// group code -> heading names in dictionary order (Rule 7)
    order: HashMap<String, Vec<String>>,
    /// `"ABBR_HDNG\u{1f}ABBR_CODE"` -> `ABBR_DESC`
    abbrs: HashMap<String, String>,
    tran_ags: String,
}

/// Fold five per-edition maps into one keyed by the union of their keys, each
/// value a list of `(edition mask, value)` variants.
///
/// This is the whole repack. The five editions overlap almost entirely — 89% of
/// heading keys carry an identical value in every edition they appear in — so
/// emitting five complete tables stored the same tuple up to five times. Here a
/// key appears once and its value once per DISTINCT value.
///
/// Masks for a given key are disjoint by construction: each edition contributes
/// at most one value, so an edition's bit lands in exactly one variant. That is
/// what lets the lookup stop at the first match.
///
/// `BTreeMap` for a deterministic, diff-stable emit — `phf` iteration order is a
/// property of the hash, not of insertion, but the *generated source* should not
/// churn between builds.
fn fold<V: Clone + PartialEq>(
    per_ed: &[(String, HashMap<String, V>)],
) -> BTreeMap<String, Vec<(u32, V)>> {
    let mut out: BTreeMap<String, Vec<(u32, V)>> = BTreeMap::new();
    for (i, (_ed, table)) in per_ed.iter().enumerate() {
        let bit = 1u32 << i;
        let mut keys: Vec<&String> = table.keys().collect();
        keys.sort();
        for k in keys {
            let v = &table[k];
            let variants = out.entry(k.clone()).or_default();
            match variants.iter_mut().find(|(_, existing)| existing == v) {
                Some((mask, _)) => *mask |= bit,
                None => variants.push((bit, v.clone())),
            }
        }
    }
    out
}

/// Emit one folded table as `phf::Map<&str, &[(u8, V)]>`, with `fmt` rendering
/// one value as a Rust expression.
fn emit_table<V>(
    out: &mut String,
    name: &str,
    ty: &str,
    folded: &BTreeMap<String, Vec<(u32, V)>>,
    fmt: impl Fn(&V) -> String,
) {
    let entries: Vec<(&str, String)> = folded
        .iter()
        .map(|(k, variants)| {
            let items: Vec<String> = variants
                .iter()
                .map(|(mask, v)| format!("({mask}, {})", fmt(v)))
                .collect();
            (k.as_str(), format!("&[{}]", items.join(", ")))
        })
        .collect();
    let mut map = phf_codegen::Map::new();
    for (k, v) in &entries {
        map.entry(*k, v);
    }
    writeln!(
        out,
        "pub(super) static {name}: phf::Map<&'static str, &'static [(EdMask, {ty})]> = {};",
        map.build()
    )
    .unwrap();
}

fn main() {
    let out_dir = env::var("OUT_DIR").expect("OUT_DIR set by cargo");
    let dest = Path::new(&out_dir).join("dict_data.rs");

    // The union JSON now lives in this leaf's own data/ (#475 PR2 relocated it
    // out of laterite-ags4-core).
    let manifest = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR set by cargo");
    let union_path = Path::new(&manifest).join("data/ags_dictionary.json");
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

    // One `EdMask` bit per edition. Five today; the width is chosen rather than
    // assumed so adding editions is a data change, not a silent truncation.
    let mask_ty = match editions.len() {
        0..=8 => "u8",
        9..=16 => "u16",
        17..=32 => "u32",
        n => panic!("{n} editions exceeds the 32-bit edition mask"),
    };

    let mut body = String::new();
    writeln!(
        body,
        "/// One bit per bundled edition, in `DictVersion::ALL` order. GENERATED.\n\
         pub(super) type EdMask = {mask_ty};\n"
    )
    .unwrap();

    // The `DictVersion` enum + edition list + FALLBACK are generated from the
    // union too, so the validator / wasm / web never hand-copy the edition set.
    emit_dict_version(&mut body, &editions, default_edition, fallback_edition);

    // Project every edition exactly as before, then fold. The projection logic
    // is untouched — only the STORAGE changes — which is what lets
    // `tests/dict_projection_pin.rs` reproduce its digests byte for byte.
    let projections: Vec<(String, Projection)> = editions
        .iter()
        .map(|ed| (ed.clone(), project(ed, &doc)))
        .collect();

    for (ed, p) in &projections {
        assert!(
            p.headings.len() > 1000,
            "dict {ed}: only {} headings projected — codegen likely broken",
            p.headings.len()
        );
        assert!(!p.tran_ags.is_empty(), "dict {ed}: TRAN_AGS not found");
    }

    let by =
        |f: fn(&Projection) -> &HashMap<String, String>| -> Vec<(String, HashMap<String, String>)> {
            projections
                .iter()
                .map(|(e, p)| (e.clone(), f(p).clone()))
                .collect()
        };

    let groups: PerEdition<GroupVal> = projections
        .iter()
        .map(|(e, p)| (e.clone(), p.groups.clone()))
        .collect();
    let headings: PerEdition<HeadingVal> = projections
        .iter()
        .map(|(e, p)| (e.clone(), p.headings.clone()))
        .collect();
    let order: PerEdition<Vec<String>> = projections
        .iter()
        .map(|(e, p)| (e.clone(), p.order.clone()))
        .collect();
    let abbrs = by(|p| &p.abbrs);

    emit_table(
        &mut body,
        "HEADINGS",
        "DictEntry",
        &fold(&headings),
        |(t, u, s, d)| {
            format!("DictEntry {{ ags_type: {t:?}, unit: {u:?}, status: {s:?}, desc: {d:?} }}")
        },
    );
    emit_table(
        &mut body,
        "GROUPS",
        "GroupMeta",
        &fold(&groups),
        |(p, d)| format!("GroupMeta {{ parent: {p:?}, desc: {d:?} }}"),
    );
    emit_table(
        &mut body,
        "GROUP_HEADINGS",
        "&'static [&'static str]",
        &fold(&order),
        |list| {
            let items: Vec<String> = list.iter().map(|h| format!("{h:?}")).collect();
            format!("&[{}]", items.join(", "))
        },
    );
    emit_table(&mut body, "ABBRS", "&'static str", &fold(&abbrs), |d| {
        format!("{d:?}")
    });

    // TRAN_AGS is one short string per edition — indexed, not masked, because
    // there is nothing to share and an index is cheaper than a scan.
    let trans: Vec<String> = projections
        .iter()
        .map(|(_, p)| format!("{:?}", p.tran_ags))
        .collect();
    writeln!(
        body,
        "\npub(super) static TRAN_AGS: [&str; {}] = [{}];",
        trans.len(),
        trans.join(", ")
    )
    .unwrap();

    let mut f = fs::File::create(&dest).expect("create dict_data.rs");
    f.write_all(body.as_bytes()).expect("write dict_data.rs");
}

/// Emit the `DictVersion` enum + `as_str`/`ALL`/`from_edition`/`tables` +
/// `FALLBACK`, generated from the union's `editions`/`default_edition`/
/// `fallback_edition`. These are the AUTHORITY for the edition set.
///
/// This comment used to *assert* that nothing hand-copies `["4.0.3" … "4.2"]`. It was
/// aspiration, not fact: the set was hand-written in ~9 places, including a match in
/// the `lat` CLI whose rejection MESSAGE was generated from `ALL` while its arms were
/// not — so a new edition would have shipped a CLI rejecting `4.3` with a message
/// advertising `4.3`. The three `lat` launchers now all reach `from_edition`/`ALL`,
/// and the surface census (`tools/gen_census.py`) checks that they still do.
///
/// Not yet true of the **web app**, which still hand-lists the editions in four TS
/// files. It is not a `lat` launcher, so the census cannot probe it — that
/// convergence is its own change. Don't restore the blanket claim until it is done.
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

    // `pub(super)`: dict.rs (chore/clippy-pedantic) wraps this generated code
    // in a private `dict_data` submodule so `#[allow(clippy::pedantic)]` scopes
    // correctly; `BundledDict` (the sole caller, in the parent module) still
    // needs to reach these.
    writeln!(
        out,
        "    /// This edition's position in `ALL` — the index into `TRAN_AGS`."
    )
    .unwrap();
    writeln!(out, "    pub(super) fn index(self) -> usize {{").unwrap();
    writeln!(out, "        match self {{").unwrap();
    for (i, ed) in editions.iter().enumerate() {
        writeln!(out, "            DictVersion::{} => {i},", variant(ed)).unwrap();
    }
    writeln!(out, "        }}").unwrap();
    writeln!(out, "    }}").unwrap();

    writeln!(
        out,
        "    /// This edition's bit in an `EdMask`. A table entry belongs to this\n    \
             /// edition when its variant's mask has this bit set."
    )
    .unwrap();
    writeln!(out, "    pub(super) fn bit(self) -> EdMask {{").unwrap();
    writeln!(out, "        1 << self.index()").unwrap();
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

/// Project one edition out of the union.
///
/// The projection mirrors `tools/gen_dictionary.py::reconstruct` exactly (flat
/// fields are the latest-edition value; `by_ed` overlays older editions; `eds`
/// gates membership; `order_by_ed`/`*_by_ed` override group meta + order), so the
/// projected values are byte-identical to the old direct-`.ags` parse — which is
/// what keeps validation behaviour (and python-ags4 parity) unchanged.
///
/// This function is deliberately unchanged by the repack: it still answers "what
/// does edition X believe", one edition at a time. Only what `main` does with
/// five of these changed — they are folded into shared tables instead of emitted
/// as five complete ones.
fn project(ed: &str, doc: &Value) -> Projection {
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

    Projection {
        groups,
        headings,
        order: group_hdng_order,
        abbrs,
        tran_ags,
    }
}
