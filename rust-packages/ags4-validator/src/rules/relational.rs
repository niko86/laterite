//! Relational rules: AGS4.1/4.2 Rules 10a, 10b, 10c, 11 (→ 11a/11b)
//! and 11c.
//!
//! CLEAN-ROOM. Implemented from the AGS4 spec (`reports/AGS 4_1.pdf` &
//! `reports/AGS 4_2.pdf` §4.1.1). python-ags4 (LGPL-3.0) was read only
//! to learn *which* relational checks it performs and *how* it
//! interprets the prose (facts about the AGS standard, not
//! copyrightable). No code, structure, or wording was copied.
//!
//! Spec text (verbatim, AGS 4.2 §4.1.1, pp.155–156 — AGS 4.1 identical):
//!
//! * **Rule 10**  — "HEADINGs are defined as KEY, REQUIRED or OTHER. …"
//! * **Rule 10a** — "In every GROUP, certain HEADINGs are defined as
//!   KEY. There shall not be more than one row of data in each GROUP
//!   with the same combination of KEY field entries. KEY fields must
//!   appear in each GROUP, but may contain null data (see Rule 12)."
//! * **Rule 10b** — "Some HEADINGs are marked as REQUIRED. REQUIRED
//!   fields must appear in the data GROUPs where they are indicated in
//!   the AGS FORMAT DATA DICTIONARY. These fields require data entry
//!   and cannot be null (i.e. left blank or empty)."
//! * **Rule 10c** — "Links are made between data rows in GROUPs by the
//!   KEY fields. Every entry made in the KEY fields in any GROUP must
//!   have an equivalent entry in its PARENT GROUP. The PARENT GROUP
//!   must be included within the data file."
//! * **Rule 11**  — "HEADINGs defined as a data TYPE of 'Record Link'
//!   (RL) can be used to link data rows to entries in GROUPs outside
//!   of the defined hierarchy (Rule 10c) or DICT group for user
//!   defined GROUPs. The GROUP name followed by the KEY FIELDs
//!   defining the cross-referenced data row, in the order presented in
//!   the AGS4 DATA DICTIONARY."
//! * **Rule 11a** — "Each GROUP/KEY FIELD shall be separated by a
//!   delimiter character … defined in TRAN_DLIM. The default being
//!   '|' (ASCII 124)."
//! * **Rule 11b** — "… more than one combination … separated by a
//!   defined concatenation character … defined in TRAN_RCON. The
//!   default being '+' (ASCII 43)."
//! * **Rule 11c** — "Any heading of data TYPE 'Record Link' included
//!   in a data file shall cross-reference to the KEY FIELDs of data
//!   rows in the GROUP referred to by the heading contents."
//!
//! The KEY/REQUIRED status and parent of each group come from an
//! *effective* dictionary = the bundled standard dictionary with the
//! file's own DICT group overlaid (Rule 9/18 territory; consumed here,
//! validated in V6). Rule 10c skips a hardcoded set of parentless /
//! implicitly-linked groups exactly as python-ags4 does — see
//! OBSERVATIONS O-21/O-22/O-23/O-24.

use std::collections::{HashMap, HashSet};

use crate::dict::Dictionary;
use crate::findings::{Findings, add};
use crate::parse::{ParsedFile, ParsedGroup};

const RULE_10A: &str = "AGS Format Rule 10a";
const RULE_10B: &str = "AGS Format Rule 10b";
const RULE_10C: &str = "AGS Format Rule 10c";
const RULE_11A: &str = "AGS Format Rule 11a";
const RULE_11B: &str = "AGS Format Rule 11b";
const RULE_11C: &str = "AGS Format Rule 11c";

/// Groups with no checkable parent linkage. python-ags4 hardcodes this
/// exact list: roots (PROJ/TRAN/…) plus groups whose link to PROJ/LOCA
/// is implicit, not a repeated-KEY relation (LOCA's `DICT_PGRP` is
/// `PROJ`, yet a LOCA row carries no PROJ key — so 10c can't be run on
/// it). Replicated verbatim for parity + correctness (O-21).
const PARENTLESS: &[&str] = &[
    "PROJ", "TRAN", "ABBR", "DICT", "UNIT", "TYPE", "LOCA", "FILE", "LBSG", "PREM", "STND",
];

/// Standard dictionary + the file's own DICT group, answering the
/// status / parent questions Rules 10a–10c need. Owned `String`s keep
/// the call sites lifetime-free.
struct EffectiveDict {
    std: Dictionary,
    /// group → [(heading, DICT_STAT)] declared in the file's DICT.
    file_hdng: HashMap<String, Vec<(String, String)>>,
    /// group → raw DICT_PGRP from a file DICT `GROUP`-type row.
    file_parent: HashMap<String, String>,
}

impl EffectiveDict {
    fn build(parsed: &ParsedFile, std: Dictionary) -> Self {
        let mut file_hdng: HashMap<String, Vec<(String, String)>> = HashMap::new();
        let mut file_parent: HashMap<String, String> = HashMap::new();
        if let Some(d) = parsed.groups.get("DICT") {
            let idx = |n: &str| d.headings.iter().position(|h| h == n);
            let (ti, gi, hi, si, pi) = (
                idx("DICT_TYPE"),
                idx("DICT_GRP"),
                idx("DICT_HDNG"),
                idx("DICT_STAT"),
                idx("DICT_PGRP"),
            );
            if let (Some(ti), Some(gi)) = (ti, gi) {
                for r in &d.rows {
                    let get = |i: Option<usize>| {
                        i.and_then(|i| r.values.get(i))
                            .map(String::as_str)
                            .unwrap_or("")
                    };
                    let dtype = r.values.get(ti).map(String::as_str).unwrap_or("");
                    let grp = r.values.get(gi).map(String::as_str).unwrap_or("");
                    if grp.is_empty() {
                        continue;
                    }
                    match dtype {
                        "GROUP" => {
                            file_parent.insert(grp.to_string(), get(pi).to_string());
                        }
                        "HEADING" => {
                            let h = get(hi);
                            if !h.is_empty() {
                                file_hdng
                                    .entry(grp.to_string())
                                    .or_default()
                                    .push((h.to_string(), get(si).to_string()));
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        EffectiveDict {
            std,
            file_hdng,
            file_parent,
        }
    }

    /// Headings of `group` whose status contains `want` (case-
    /// insensitive: `"KEY"` or `"REQUIRED"`), standard dict first then
    /// file-DICT extras, de-duplicated.
    fn fields_with_status(&self, group: &str, want: &str) -> Vec<String> {
        let mut out: Vec<String> = Vec::new();
        for h in self.std.group_headings(group) {
            if let Some(e) = self.std.heading(group, h) {
                if e.status.to_ascii_uppercase().contains(want) {
                    out.push((*h).to_string());
                }
            }
        }
        if let Some(extra) = self.file_hdng.get(group) {
            for (h, st) in extra {
                if st.to_ascii_uppercase().contains(want) && !out.iter().any(|x| x == h) {
                    out.push(h.clone());
                }
            }
        }
        out
    }

    fn key_fields(&self, group: &str) -> Vec<String> {
        self.fields_with_status(group, "KEY")
    }
    fn required_fields(&self, group: &str) -> Vec<String> {
        self.fields_with_status(group, "REQUIRED")
    }

    /// `Some(parent)` (possibly `""` = blank in dictionary), or `None`
    /// if the group has no definition in either dictionary.
    fn parent(&self, group: &str) -> Option<String> {
        if let Some(m) = self.std.group(group) {
            return Some(m.parent.to_string()); // build.rs maps '-' → ""
        }
        self.file_parent
            .get(group)
            .map(|p| if p == "-" { String::new() } else { p.clone() })
    }
}

pub fn check(parsed: &ParsedFile, dict: &Dictionary, found: &mut Findings) {
    let eff = EffectiveDict::build(parsed, *dict);

    for code in &parsed.group_order {
        let g = &parsed.groups[code];
        rule_10a(g, code, &eff, found);
        rule_10b(g, code, &eff, found);
        rule_10c(parsed, g, code, &eff, found);
    }

    rule_11(parsed, found);
}

/// Column index of `name` in a group's HEADING row.
fn col(g: &ParsedGroup, name: &str) -> Option<usize> {
    g.headings.iter().position(|h| h == name)
}

/// The values of `g`'s row at the named columns, in `names` order
/// (missing column → empty string, so tuples stay positional).
fn tuple(g: &ParsedGroup, names: &[String], row: &crate::parse::DataRow) -> Vec<String> {
    names
        .iter()
        .map(|n| {
            col(g, n)
                .and_then(|i| row.values.get(i))
                .cloned()
                .unwrap_or_default()
        })
        .collect()
}

/// Rule 10a — KEY fields present; no duplicate KEY combinations.
fn rule_10a(g: &ParsedGroup, code: &str, eff: &EffectiveDict, found: &mut Findings) {
    let keys = eff.key_fields(code);
    if keys.is_empty() {
        return;
    }

    let mut all_present = true;
    for k in &keys {
        if !g.headings.iter().any(|h| h == k) {
            all_present = false;
            if let Some(hl) = g.heading_line {
                add(
                    found,
                    RULE_10A,
                    Some(hl),
                    code,
                    format!("KEY field {k} is not present."),
                );
            }
        }
    }
    if !all_present {
        return; // can't trust the combination check without all keys
    }

    let mut counts: HashMap<Vec<String>, usize> = HashMap::new();
    for row in &g.rows {
        *counts.entry(tuple(g, &keys, row)).or_default() += 1;
    }
    for row in &g.rows {
        let t = tuple(g, &keys, row);
        if counts.get(&t).copied().unwrap_or(0) > 1 {
            add(
                found,
                RULE_10A,
                Some(row.line),
                code,
                format!("Duplicate KEY field combination: {}", t.join("|")),
            );
        }
    }
}

/// Rule 10b — REQUIRED fields present and non-empty in every DATA row.
fn rule_10b(g: &ParsedGroup, code: &str, eff: &EffectiveDict, found: &mut Findings) {
    let req = eff.required_fields(code);
    if req.is_empty() {
        return;
    }

    let mut present: Vec<&String> = Vec::new();
    for r in &req {
        match g.headings.iter().any(|h| h == r) {
            true => present.push(r),
            false => {
                if let Some(hl) = g.heading_line {
                    add(
                        found,
                        RULE_10B,
                        Some(hl),
                        code,
                        format!("REQUIRED field {r} is not present."),
                    );
                }
            }
        }
    }

    // Map REQUIRED heading name → column index (skip those that aren't
    // present — `req_cols` carries only resolvable REQUIRED columns).
    let req_cols: Vec<(usize, &str)> = present
        .iter()
        .filter_map(|r| col(g, r).map(|i| (i, r.as_str())))
        .collect();

    for row in &g.rows {
        let any_empty = req_cols.iter().any(|(i, _)| {
            row.values
                .get(*i)
                .map(|v| v.trim().is_empty())
                .unwrap_or(true)
        });
        if !any_empty {
            continue;
        }
        // Reconstruct the DATA row with `|`-separator, substituting
        // `??<FIELD_NAME>??` where a REQUIRED column is empty —
        // python-ags4's `rule_10b` row format. Leading `DATA|` matches
        // the row tag; trailing empty columns stay as bare `|`s.
        let empty_at: std::collections::HashMap<usize, &str> = req_cols
            .iter()
            .copied()
            .filter(|(i, _)| {
                row.values
                    .get(*i)
                    .map(|v| v.trim().is_empty())
                    .unwrap_or(true)
            })
            .collect();
        let mut parts: Vec<String> = Vec::with_capacity(g.headings.len() + 1);
        parts.push("DATA".to_string());
        for (i, _) in g.headings.iter().enumerate() {
            let v = row.values.get(i).map(String::as_str).unwrap_or("");
            if let Some(name) = empty_at.get(&i) {
                parts.push(format!("??{name}??"));
            } else {
                parts.push(v.to_string());
            }
        }
        add(
            found,
            RULE_10B,
            Some(row.line),
            code,
            format!("Empty REQUIRED fields: {}", parts.join("|")),
        );
    }
}

/// Rule 10c — every child row must have a matching parent row, keyed
/// by the parent's KEY fields.
fn rule_10c(
    parsed: &ParsedFile,
    g: &ParsedGroup,
    code: &str,
    eff: &EffectiveDict,
    found: &mut Findings,
) {
    if PARENTLESS.contains(&code) {
        return;
    }
    let Some(parent) = eff.parent(code) else {
        add(
            found,
            RULE_10C,
            None,
            code,
            "Could not check parent entries: group not defined in the standard dictionary or DICT group."
                .to_string(),
        );
        return;
    };
    if parent.is_empty() {
        add(
            found,
            RULE_10C,
            None,
            code,
            "Parent group is left blank in the dictionary.".to_string(),
        );
        return;
    }
    let Some(pg) = parsed.groups.get(&parent) else {
        add(
            found,
            RULE_10C,
            None,
            code,
            format!("Parent group {parent} is not in the file."),
        );
        return;
    };

    let pkeys = eff.key_fields(&parent);
    if pkeys.is_empty() {
        add(
            found,
            RULE_10C,
            None,
            code,
            format!("No KEY fields are defined in the parent group ({parent})."),
        );
        return;
    }
    let ckeys = eff.key_fields(code);
    let missing: Vec<&String> = pkeys.iter().filter(|p| !ckeys.contains(p)).collect();
    if !missing.is_empty() {
        let names: Vec<&str> = missing.iter().map(|s| s.as_str()).collect();
        add(
            found,
            RULE_10C,
            None,
            code,
            format!(
                "{} defined as KEY field(s) in the parent group ({parent}) but not in the child group.",
                names.join(", ")
            ),
        );
        return;
    }

    let in_child = pkeys.iter().all(|k| col(g, k).is_some());
    let in_parent = pkeys.iter().all(|k| col(pg, k).is_some());
    if !(in_child && in_parent) {
        if let Some(hl) = g.heading_line {
            add(
                found,
                RULE_10C,
                Some(hl),
                code,
                format!(
                    "Could not check parent entries: KEY fields missing in {code} or {parent} \
                     (see Rule 10a)."
                ),
            );
        }
        return;
    }

    let parent_tuples: HashSet<Vec<String>> =
        pg.rows.iter().map(|r| tuple(pg, &pkeys, r)).collect();
    for row in &g.rows {
        let t = tuple(g, &pkeys, row);
        // O-39: skip child rows whose parent KEY cells are ALL empty
        // — those are "standalone" rows by the file's own design
        // (e.g. lab-control SAMP with no LOCA borehole, off-site
        // samples). python-ags4 reads the spec ("every entry in the
        // KEY fields must have a parent") as applying only to
        // non-empty entries. Empty cells are not "entries". A row
        // with even one non-empty parent KEY field, in contrast, IS
        // claiming a parent and gets the usual check.
        if t.iter().all(|s| s.trim().is_empty()) {
            continue;
        }
        if !parent_tuples.contains(&t) {
            add(
                found,
                RULE_10C,
                Some(row.line),
                code,
                format!(
                    "No parent entry in {parent} for KEY combination: {}",
                    t.join("|")
                ),
            );
        }
    }
}

/// Rule 11 — read TRAN_DLIM / TRAN_RCON, dispatch 11a/11b/11c.
fn rule_11(parsed: &ParsedFile, found: &mut Findings) {
    let Some(tran) = parsed.groups.get("TRAN") else {
        return; // TRAN missing → Rule 14 reports it
    };
    let Some(data) = tran.rows.first() else {
        return; // no DATA row → Rule 14
    };
    // Absent columns: python-ags4 swallows the KeyError → no finding.
    // We mirror that (TRAN_DLIM/RCON are OTHER, not REQUIRED). (O-23)
    let (Some(di), Some(ci)) = (col(tran, "TRAN_DLIM"), col(tran, "TRAN_RCON")) else {
        return;
    };
    let delim = data.values.get(di).map(String::as_str).unwrap_or("");
    let concat = data.values.get(ci).map(String::as_str).unwrap_or("");

    let mut blocked = false;
    if delim.is_empty() {
        add(
            found,
            RULE_11A,
            Some(data.line),
            "TRAN",
            "TRAN_DLIM is missing.",
        );
        blocked = true;
    }
    if concat.is_empty() {
        add(
            found,
            RULE_11B,
            Some(data.line),
            "TRAN",
            "TRAN_RCON is missing.",
        );
        blocked = true;
    }
    if blocked {
        return; // 11c needs a usable delimiter + concatenator
    }
    rule_11c(parsed, delim, concat, found);
}

/// Rule 11c — every Record-Link value must resolve to exactly one row
/// in the referenced GROUP (positional match against its leading
/// columns, the AGS4 "GROUP|key1|key2…" form).
fn rule_11c(parsed: &ParsedFile, delim: &str, concat: &str, found: &mut Findings) {
    for code in &parsed.group_order {
        let g = &parsed.groups[code];
        for (ci, ty) in g.types.iter().enumerate() {
            if ty.trim() != "RL" {
                continue;
            }
            for row in &g.rows {
                let Some(rl) = row.values.get(ci) else {
                    continue;
                };
                if rl.is_empty() {
                    continue;
                }
                if !rl.contains(delim) {
                    add(
                        found,
                        RULE_11C,
                        Some(row.line),
                        code,
                        format!(
                            "Invalid Record Link {rl:?}: {delim:?} must separate the GROUP \
                             and KEY fields."
                        ),
                    );
                    continue;
                }
                for link in rl.split(concat) {
                    let parts: Vec<&str> = link.split(delim).collect();
                    match fetch_count(parsed, &parts) {
                        0 => add(
                            found,
                            RULE_11C,
                            Some(row.line),
                            code,
                            format!("Invalid Record Link {link:?}: no such record."),
                        ),
                        1 => {}
                        _ => add(
                            found,
                            RULE_11C,
                            Some(row.line),
                            code,
                            format!("Invalid Record Link {link:?}: matches more than one record."),
                        ),
                    }
                }
            }
        }
    }
}

/// How many DATA rows in `parts[0]` match `parts[1..]` positionally
/// against the group's leading heading columns (python-ags4's
/// `fetch_record` is positional, not KEY-aware — O-24).
fn fetch_count(parsed: &ParsedFile, parts: &[&str]) -> usize {
    let Some((grp, keys)) = parts.split_first() else {
        return 0;
    };
    let Some(g) = parsed.groups.get(*grp) else {
        return 0;
    };
    if keys.is_empty() || keys.len() > g.headings.len() {
        return 0;
    }
    g.rows
        .iter()
        .filter(|r| {
            keys.iter()
                .enumerate()
                .all(|(i, k)| r.values.get(i).map(String::as_str).unwrap_or("") == *k)
        })
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dict::DictVersion;
    use crate::parse::parse_str;

    fn run(src: &str) -> Findings {
        let pf = parse_str(src).expect("fixture parses");
        let d = Dictionary::bundled(DictVersion::V4_2);
        let mut f = Findings::new();
        check(&pf, &d, &mut f);
        f
    }

    #[test]
    fn rule_10a_flags_duplicate_key_and_missing_key() {
        // LOCA KEY = LOCA_ID. Two rows with the same LOCA_ID.
        let dup = "\"GROUP\",\"LOCA\"\r\n\"HEADING\",\"LOCA_ID\",\"LOCA_FDEP\"\r\n\
                   \"UNIT\",\"\",\"m\"\r\n\"TYPE\",\"ID\",\"2DP\"\r\n\
                   \"DATA\",\"BH1\",\"1.00\"\r\n\"DATA\",\"BH1\",\"2.00\"\r\n";
        let r = run(dup);
        let r10a = r.get(RULE_10A).expect("Rule 10a");
        assert_eq!(
            r10a.iter().filter(|x| x.desc.contains("Duplicate")).count(),
            2
        );

        // LOCA without its LOCA_ID KEY column.
        let miss = "\"GROUP\",\"LOCA\"\r\n\"HEADING\",\"LOCA_FDEP\"\r\n\
                    \"UNIT\",\"m\"\r\n\"TYPE\",\"2DP\"\r\n\"DATA\",\"1.00\"\r\n";
        assert!(run(miss).get(RULE_10A).is_some_and(|v| {
            v.iter()
                .any(|x| x.desc.contains("LOCA_ID") && x.desc.contains("not present"))
        }));
    }

    #[test]
    fn rule_10b_flags_missing_and_empty_required() {
        // PROJ_ID is KEY+REQUIRED; blank in the only DATA row.
        let src = "\"GROUP\",\"PROJ\"\r\n\"HEADING\",\"PROJ_ID\",\"PROJ_NAME\"\r\n\
                   \"UNIT\",\"\",\"\"\r\n\"TYPE\",\"ID\",\"X\"\r\n\
                   \"DATA\",\"\",\"Site\"\r\n";
        let r10b = run(src);
        let v = r10b.get(RULE_10B).expect("Rule 10b");
        assert!(
            v.iter()
                .any(|x| x.desc.contains("Empty REQUIRED") && x.line == Some(5)),
            "{v:?}"
        );
    }

    #[test]
    fn rule_10c_flags_orphan_child_row() {
        // SAMP (parent LOCA, key LOCA_ID) references a LOCA that exists,
        // plus one orphan (BH9 not in LOCA).
        let src = "\"GROUP\",\"LOCA\"\r\n\"HEADING\",\"LOCA_ID\"\r\n\
                   \"UNIT\",\"\"\r\n\"TYPE\",\"ID\"\r\n\"DATA\",\"BH1\"\r\n\r\n\
                   \"GROUP\",\"SAMP\"\r\n\
                   \"HEADING\",\"LOCA_ID\",\"SAMP_TOP\",\"SAMP_REF\",\"SAMP_TYPE\",\"SAMP_ID\"\r\n\
                   \"UNIT\",\"\",\"m\",\"\",\"\",\"\"\r\n\
                   \"TYPE\",\"ID\",\"2DP\",\"X\",\"PA\",\"ID\"\r\n\
                   \"DATA\",\"BH1\",\"1.00\",\"S1\",\"B\",\"BH1S1\"\r\n\
                   \"DATA\",\"BH9\",\"2.00\",\"S2\",\"B\",\"BH9S2\"\r\n";
        let r = run(src);
        let r10c = r.get(RULE_10C).expect("Rule 10c");
        assert!(
            r10c.iter()
                .any(|x| x.desc.contains("BH9") && x.line == Some(12)),
            "expected orphan BH9 flagged: {r10c:?}"
        );
        assert!(
            !r10c.iter().any(|x| x.desc.contains("BH1")),
            "BH1 has a parent: {r10c:?}"
        );
    }

    #[test]
    fn rule_10c_skips_rows_with_all_empty_parent_keys() {
        // O-39: a SAMP row with empty LOCA_ID is "standalone" — no
        // parent claim. Don't flag (matches python-ags4's permissive
        // spec reading; "every entry made in KEY fields" = non-empty
        // entries only). A row with non-empty LOCA_ID that has no
        // match still flags as before.
        let src = "\"GROUP\",\"LOCA\"\r\n\"HEADING\",\"LOCA_ID\"\r\n\
                   \"UNIT\",\"\"\r\n\"TYPE\",\"ID\"\r\n\"DATA\",\"BH1\"\r\n\r\n\
                   \"GROUP\",\"SAMP\"\r\n\
                   \"HEADING\",\"LOCA_ID\",\"SAMP_TOP\",\"SAMP_REF\",\"SAMP_TYPE\",\"SAMP_ID\"\r\n\
                   \"UNIT\",\"\",\"m\",\"\",\"\",\"\"\r\n\
                   \"TYPE\",\"ID\",\"2DP\",\"X\",\"PA\",\"ID\"\r\n\
                   \"DATA\",\"\",\"1.00\",\"S1\",\"B\",\"LAB_CTRL\"\r\n\
                   \"DATA\",\"BH1\",\"2.00\",\"S2\",\"B\",\"BH1S2\"\r\n";
        let r = run(src);
        // Standalone row (LOCA_ID="") → no Rule 10c.
        assert!(
            r.get(RULE_10C).map_or(true, |v| v.is_empty()),
            "standalone row must not flag: {:?}",
            r.get(RULE_10C)
        );
    }

    #[test]
    fn rule_11a_11b_flag_blank_tran_delims() {
        let src = "\"GROUP\",\"TRAN\"\r\n\
                   \"HEADING\",\"TRAN_DLIM\",\"TRAN_RCON\"\r\n\
                   \"UNIT\",\"\",\"\"\r\n\"TYPE\",\"X\",\"X\"\r\n\"DATA\",\"\",\"\"\r\n";
        let r = run(src);
        assert!(r.contains_key(RULE_11A), "{r:?}");
        assert!(r.contains_key(RULE_11B), "{r:?}");
    }

    #[test]
    fn rule_11c_flags_bad_record_link() {
        // SAMP.SAMP_LINK is RL pointing at LOCA|BH404 (no such LOCA).
        let src = "\"GROUP\",\"TRAN\"\r\n\
                   \"HEADING\",\"TRAN_DLIM\",\"TRAN_RCON\"\r\n\
                   \"UNIT\",\"\",\"\"\r\n\"TYPE\",\"X\",\"X\"\r\n\"DATA\",\"|\",\"+\"\r\n\r\n\
                   \"GROUP\",\"LOCA\"\r\n\"HEADING\",\"LOCA_ID\"\r\n\
                   \"UNIT\",\"\"\r\n\"TYPE\",\"ID\"\r\n\"DATA\",\"BH1\"\r\n\r\n\
                   \"GROUP\",\"SAMP\"\r\n\"HEADING\",\"LOCA_ID\",\"SAMP_LINK\"\r\n\
                   \"UNIT\",\"\",\"\"\r\n\"TYPE\",\"ID\",\"RL\"\r\n\
                   \"DATA\",\"BH1\",\"LOCA|BH404\"\r\n";
        let r = run(src);
        let r11c = r.get(RULE_11C).expect("Rule 11c");
        assert!(
            r11c.iter().any(|x| x.desc.contains("no such record")),
            "{r11c:?}"
        );

        // A link with no delimiter at all.
        let nodelim = "\"GROUP\",\"TRAN\"\r\n\
                       \"HEADING\",\"TRAN_DLIM\",\"TRAN_RCON\"\r\n\
                       \"UNIT\",\"\",\"\"\r\n\"TYPE\",\"X\",\"X\"\r\n\"DATA\",\"|\",\"+\"\r\n\r\n\
                       \"GROUP\",\"SAMP\"\r\n\"HEADING\",\"LOCA_ID\",\"SAMP_LINK\"\r\n\
                       \"UNIT\",\"\",\"\"\r\n\"TYPE\",\"ID\",\"RL\"\r\n\
                       \"DATA\",\"BH1\",\"LOCABH1\"\r\n";
        assert!(
            run(nodelim)
                .get(RULE_11C)
                .is_some_and(|v| v.iter().any(|x| x.desc.contains("must separate")))
        );
    }
}
