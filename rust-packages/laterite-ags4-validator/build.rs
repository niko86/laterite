//! Compile-time codegen: vendored AGS4 standard dictionaries (.ags) →
//! `phf` perfect-hash static tables in `OUT_DIR/dict_data.rs`.
//!
//! Runs once per build. Output is `include!`d by `src/dict.rs`, so the
//! validator pays zero startup cost and never parses the dictionary at
//! runtime. The .ags dictionary content is ©AGS reference data — see
//! `data/PROVENANCE.md`.
//!
//! Column positions are resolved by *name* from each DICT group's
//! HEADING row, not hard-coded, so a future dictionary edition that
//! reorders columns still builds.

use std::collections::HashMap;
use std::env;
use std::fmt::Write as _;
use std::fs;
use std::io::Write as _;
use std::path::Path;

fn main() {
    let out_dir = env::var("OUT_DIR").expect("OUT_DIR set by cargo");
    let dest = Path::new(&out_dir).join("dict_data.rs");

    let mut body = String::new();
    for (ident, file) in [
        ("4_0_3", "data/Standard_dictionary_v4_0_3.ags"),
        ("4_0_4", "data/Standard_dictionary_v4_0_4.ags"),
        ("4_1", "data/Standard_dictionary_v4_1.ags"),
        ("4_1_1", "data/Standard_dictionary_v4_1_1.ags"),
        ("4_2", "data/Standard_dictionary_v4_2.ags"),
    ] {
        println!("cargo:rerun-if-changed={file}");
        // The older 4.0.x dictionaries python-ags4 ships are Latin-1
        // (cp1252), not UTF-8 (`°`/`±`/`µ` in unit/desc cells). Decode
        // byte→char as ISO-8859-1: lossless, dependency-free, and
        // exactly the 0–255 tolerance the validator itself documents
        // (OBSERVATIONS O-1). Keeps the vendored files byte-identical
        // to upstream (clean provenance) rather than re-encoding them.
        let bytes = fs::read(file).unwrap_or_else(|e| panic!("read {file}: {e}"));
        let text: String = bytes.iter().map(|&b| b as char).collect();
        emit_version(&mut body, ident, &text);
    }

    let mut f = fs::File::create(&dest).expect("create dict_data.rs");
    f.write_all(body.as_bytes()).expect("write dict_data.rs");
}

/// Parse one standard dictionary and append its three statics to `out`.
fn emit_version(out: &mut String, ident: &str, text: &str) {
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .flexible(true)
        .from_reader(text.as_bytes());

    let mut current_group = String::new();
    // DICT group's column-name -> index, captured from its HEADING row.
    let mut dict_cols: HashMap<String, usize> = HashMap::new();
    // TRAN group's column-name -> index.
    let mut tran_cols: HashMap<String, usize> = HashMap::new();
    // ABBR group's column-name -> index. Stage 7e: used to surface
    // the canonical (ABBR_HDNG, ABBR_CODE) → ABBR_DESC table per
    // edition so rule_16_fyi can diff a file's ABBR rows against the
    // standard list (python-ags4's `fyi_16_1` behaviour).
    let mut abbr_cols: HashMap<String, usize> = HashMap::new();

    // group code -> (parent, desc)
    let mut groups: HashMap<String, (String, String)> = HashMap::new();
    // "GROUP\u{1f}HEADING" -> (ags_type, unit, status, desc)
    let mut headings: HashMap<String, (String, String, String, String)> = HashMap::new();
    // group code -> heading names in *dictionary order* (the phf map
    // above is unordered; Rule 7 needs the canonical order). First
    // occurrence wins, mirroring python-ags4's drop_duplicates(keep=
    // 'first') so a redefinition keeps its original slot.
    let mut group_hdng_order: HashMap<String, Vec<String>> = HashMap::new();
    let mut tran_ags = String::new();
    // "ABBR_HDNG\u{1f}ABBR_CODE" -> ABBR_DESC. Last definition wins
    // (mirrors python-ags4's drop_duplicates(keep='last')).
    let mut abbrs: HashMap<String, String> = HashMap::new();

    let col = |cols: &HashMap<String, usize>, rec: &csv::StringRecord, name: &str| -> String {
        cols.get(name)
            .and_then(|&i| rec.get(i))
            .unwrap_or("")
            .trim()
            .to_string()
    };

    for result in rdr.records() {
        let rec = match result {
            Ok(r) => r,
            Err(_) => continue,
        };
        let tag = rec.get(0).unwrap_or("").trim();
        match tag {
            "GROUP" => {
                current_group = rec.get(1).unwrap_or("").trim().to_string();
            }
            "HEADING" => {
                if current_group == "DICT" {
                    dict_cols = rec
                        .iter()
                        .enumerate()
                        .skip(1)
                        .map(|(i, h)| (h.trim().to_string(), i))
                        .collect();
                } else if current_group == "TRAN" {
                    tran_cols = rec
                        .iter()
                        .enumerate()
                        .skip(1)
                        .map(|(i, h)| (h.trim().to_string(), i))
                        .collect();
                } else if current_group == "ABBR" {
                    abbr_cols = rec
                        .iter()
                        .enumerate()
                        .skip(1)
                        .map(|(i, h)| (h.trim().to_string(), i))
                        .collect();
                }
            }
            "DATA" if current_group == "DICT" => {
                let dtype = col(&dict_cols, &rec, "DICT_TYPE");
                let grp = col(&dict_cols, &rec, "DICT_GRP");
                if grp.is_empty() {
                    continue;
                }
                if dtype == "GROUP" {
                    let mut parent = col(&dict_cols, &rec, "DICT_PGRP");
                    if parent == "-" {
                        parent.clear(); // root group
                    }
                    let desc = col(&dict_cols, &rec, "DICT_DESC");
                    groups.insert(grp, (parent, desc));
                } else if dtype == "HEADING" {
                    let hdng = col(&dict_cols, &rec, "DICT_HDNG");
                    if hdng.is_empty() {
                        continue;
                    }
                    let ags_type = col(&dict_cols, &rec, "DICT_DTYP");
                    let unit = col(&dict_cols, &rec, "DICT_UNIT");
                    let status = col(&dict_cols, &rec, "DICT_STAT");
                    let desc = col(&dict_cols, &rec, "DICT_DESC");
                    let order = group_hdng_order.entry(grp.clone()).or_default();
                    if !order.iter().any(|h| h == &hdng) {
                        order.push(hdng.clone());
                    }
                    // Last definition wins (dedupe; phf_codegen panics
                    // on duplicate keys).
                    headings.insert(format!("{grp}\u{1f}{hdng}"), (ags_type, unit, status, desc));
                }
            }
            "DATA" if current_group == "TRAN" && tran_ags.is_empty() => {
                let v = col(&tran_cols, &rec, "TRAN_AGS");
                if !v.is_empty() {
                    tran_ags = v;
                }
            }
            "DATA" if current_group == "ABBR" => {
                let hdng = col(&abbr_cols, &rec, "ABBR_HDNG");
                let code = col(&abbr_cols, &rec, "ABBR_CODE");
                let desc = col(&abbr_cols, &rec, "ABBR_DESC");
                if !hdng.is_empty() && !code.is_empty() {
                    // Composite key matches `Dictionary::heading_key`
                    // style — keeps the API symmetric for `abbr_desc()`.
                    abbrs.insert(format!("{hdng}\u{1f}{code}"), desc);
                }
            }
            _ => {}
        }
    }

    assert!(
        headings.len() > 1000,
        "dict {ident}: only {} headings parsed — codegen likely broken",
        headings.len()
    );
    assert!(!tran_ags.is_empty(), "dict {ident}: TRAN_AGS not found");

    // Build the phf maps. Deterministic order (sorted) so the generated
    // file is reproducible / diff-stable. phf_codegen::Map::entry
    // borrows the value `&str` until `.build()`, so the formatted
    // expression strings must outlive the builder — materialise them
    // into owned Vecs first.
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
