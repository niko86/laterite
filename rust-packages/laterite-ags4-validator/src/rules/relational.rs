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
//!   delimiter character … defined in `TRAN_DLIM`. The default being
//!   '|' (ASCII 124)."
//! * **Rule 11b** — "… more than one combination … separated by a
//!   defined concatenation character … defined in `TRAN_RCON`. The
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
use crate::findings::{Findings, Location, Severity, Target, add, add_at};
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
struct EffectiveDict<'a> {
    std: Dictionary<'a>,
    /// group → [(heading, `DICT_STAT`)] declared in the file's DICT.
    file_hdng: HashMap<String, Vec<(String, String)>>,
    /// group → raw `DICT_PGRP` from a file DICT `GROUP`-type row.
    file_parent: HashMap<String, String>,
}

impl<'a> EffectiveDict<'a> {
    fn build(parsed: &ParsedFile, std: Dictionary<'a>) -> Self {
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
                        i.and_then(|i| r.values.get(i)).map_or("", String::as_str)
                    };
                    let dtype = r.values.get(ti).map_or("", String::as_str);
                    let grp = r.values.get(gi).map_or("", String::as_str);
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
        for h in self.std.group_headings(group).iter() {
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

pub fn check(parsed: &ParsedFile, dict: &Dictionary<'_>, found: &mut Findings) {
    let eff = EffectiveDict::build(parsed, *dict);

    // A parent's KEY-tuple set depends only on the parent group, never on which
    // child is asking, so it is memoised by parent code across the whole file.
    // Rule 10c runs once per group and many groups share a parent — SAMP alone
    // parents the entire lab-test family — so rebuilding the set per child was
    // the bulk of this rule's cost (see [[perf-campaign]] T1). The tuples borrow
    // from `parsed`, which outlives the loop.
    let mut parent_tuples: HashMap<String, HashSet<Vec<&str>>> = HashMap::new();

    for code in &parsed.group_order {
        let g = &parsed.groups[code];
        rule_10a(g, code, &eff, found);
        rule_10b(g, code, &eff, found);
        rule_10c(parsed, g, code, &eff, found, &mut parent_tuples);
    }

    rule_11(parsed, found);
}

/// Column index of `name` in a group's HEADING row.
fn col(g: &ParsedGroup, name: &str) -> Option<usize> {
    g.headings.iter().position(|h| h == name)
}

/// Column indices of `names`, resolved ONCE for a group.
///
/// The indices do not vary by row, but the per-row form of this
/// (`col()` inside `tuple()`) re-derived them for every row — and `col` is a
/// linear scan of the heading list, so a group of R rows with K keys and H
/// headings paid R*K*H string comparisons to answer the same K questions.
/// Hoisting it makes that R*K pointer reads. See [[core-perf-baseline]].
fn cols(g: &ParsedGroup, names: &[String]) -> Vec<Option<usize>> {
    names.iter().map(|n| col(g, n)).collect()
}

/// The row's values at `idx`, in `idx` order, BORROWED from the row.
///
/// Same contract as the earlier owned version — a missing column or a ragged
/// row yields `""` so tuples stay positional — but it no longer clones every
/// cell. These tuples exist only to be hashed and compared; the rows already
/// own the text, so the clone bought nothing.
fn tuple_at<'a>(idx: &[Option<usize>], row: &'a crate::parse::DataRow) -> Vec<&'a str> {
    idx.iter()
        .map(|i| i.and_then(|i| row.values.get(i)).map_or("", String::as_str))
        .collect()
}

/// Rule 10a — KEY fields present; no duplicate KEY combinations.
// `ri` is a DATA row's index within one AGS4 group — bounded by that
// group's actual row count, far below u32::MAX for any real AGS4 file.
#[allow(clippy::cast_possible_truncation)]
fn rule_10a(g: &ParsedGroup, code: &str, eff: &EffectiveDict<'_>, found: &mut Findings) {
    let keys = eff.key_fields(code);
    if keys.is_empty() {
        return;
    }

    let mut all_present = true;
    for k in &keys {
        if !g.headings.iter().any(|h| h == k) {
            all_present = false;
            if let Some(hl) = g.heading_line {
                add_at(
                    found,
                    RULE_10A,
                    Some(hl),
                    code,
                    format!("KEY field {k} is not present."),
                    Location {
                        target: Target::Heading,
                        heading: Some(k.clone()),
                        ..Default::default()
                    },
                    Severity::Error,
                );
            }
        }
    }
    if !all_present {
        return; // can't trust the combination check without all keys
    }

    let idx = cols(g, &keys);
    let mut counts: HashMap<Vec<&str>, usize> = HashMap::new();
    for row in &g.rows {
        *counts.entry(tuple_at(&idx, row)).or_default() += 1;
    }
    for (ri, row) in g.rows.iter().enumerate() {
        let t = tuple_at(&idx, row);
        if counts.get(&t).copied().unwrap_or(0) > 1 {
            add_at(
                found,
                RULE_10A,
                Some(row.line),
                code,
                format!("Duplicate KEY field combination: {}", t.join("|")),
                Location {
                    target: Target::Cell,
                    data_row: Some(ri as u32 + 1),
                    ..Default::default()
                },
                Severity::Error,
            );
        }
    }
}

/// Rule 10b — REQUIRED fields present and non-empty in every DATA row.
// `ri` is bounded the same way as in `rule_10a` above.
#[allow(clippy::cast_possible_truncation)]
fn rule_10b(g: &ParsedGroup, code: &str, eff: &EffectiveDict<'_>, found: &mut Findings) {
    let req = eff.required_fields(code);
    if req.is_empty() {
        return;
    }

    let mut present: Vec<&String> = Vec::new();
    for r in &req {
        if g.headings.iter().any(|h| h == r) {
            present.push(r);
        } else if let Some(hl) = g.heading_line {
            add_at(
                found,
                RULE_10B,
                Some(hl),
                code,
                format!("REQUIRED field {r} is not present."),
                Location {
                    target: Target::Heading,
                    heading: Some(r.clone()),
                    ..Default::default()
                },
                Severity::Error,
            );
        }
    }

    // Map REQUIRED heading name → column index (skip those that aren't
    // present — `req_cols` carries only resolvable REQUIRED columns).
    let req_cols: Vec<(usize, &str)> = present
        .iter()
        .filter_map(|r| col(g, r).map(|i| (i, r.as_str())))
        .collect();

    for (ri, row) in g.rows.iter().enumerate() {
        let any_empty = req_cols
            .iter()
            .any(|(i, _)| row.values.get(*i).is_none_or(|v| v.trim().is_empty()));
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
            .filter(|(i, _)| row.values.get(*i).is_none_or(|v| v.trim().is_empty()))
            .collect();
        let mut parts: Vec<String> = Vec::with_capacity(g.headings.len() + 1);
        parts.push("DATA".to_string());
        for (i, _) in g.headings.iter().enumerate() {
            let v = row.values.get(i).map_or("", String::as_str);
            if let Some(name) = empty_at.get(&i) {
                parts.push(format!("??{name}??"));
            } else {
                parts.push(v.to_string());
            }
        }
        add_at(
            found,
            RULE_10B,
            Some(row.line),
            code,
            format!("Empty REQUIRED fields: {}", parts.join("|")),
            Location {
                target: Target::Cell,
                data_row: Some(ri as u32 + 1),
                ..Default::default()
            },
            Severity::Error,
        );
    }
}

/// Rule 10c — every child row must have a matching parent row, keyed
/// by the parent's KEY fields.
// `ri` is bounded the same way as in `rule_10a` above.
#[allow(clippy::cast_possible_truncation)]
fn rule_10c<'p>(
    parsed: &'p ParsedFile,
    g: &'p ParsedGroup,
    code: &str,
    eff: &EffectiveDict<'_>,
    found: &mut Findings,
    parent_tuples: &mut HashMap<String, HashSet<Vec<&'p str>>>,
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

    // Child indices are per-child, so resolved here; the parent tuple SET is
    // per-parent, so built once and cached (see `check`). `entry` clones the
    // parent code once per child — cheap beside the row scan it replaces.
    let cidx = cols(g, &pkeys);
    let ptuples = parent_tuples.entry(parent.clone()).or_insert_with(|| {
        let pidx = cols(pg, &pkeys);
        pg.rows.iter().map(|r| tuple_at(&pidx, r)).collect()
    });
    for (ri, row) in g.rows.iter().enumerate() {
        let t = tuple_at(&cidx, row);
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
        if !ptuples.contains(&t) {
            add_at(
                found,
                RULE_10C,
                Some(row.line),
                code,
                format!(
                    "No parent entry in {parent} for KEY combination: {}",
                    t.join("|")
                ),
                Location {
                    target: Target::Cell,
                    data_row: Some(ri as u32 + 1),
                    ..Default::default()
                },
                Severity::Error,
            );
        }
    }
}

/// Rule 11 — read `TRAN_DLIM` / `TRAN_RCON`, dispatch 11a/11b/11c.
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
    let delim = data.values.get(di).map_or("", String::as_str);
    let concat = data.values.get(ci).map_or("", String::as_str);

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
// `ci`/`ri` are a column/row index within one AGS4 group — both bounded
// far below u32::MAX for any real AGS4 file (see `rule_10a` above).
#[allow(clippy::cast_possible_truncation)]
fn rule_11c(parsed: &ParsedFile, delim: &str, concat: &str, found: &mut Findings) {
    for code in &parsed.group_order {
        let g = &parsed.groups[code];
        for (ci, ty) in g.types.iter().enumerate() {
            if ty.trim() != "RL" {
                continue;
            }
            for (ri, row) in g.rows.iter().enumerate() {
                let Some(rl) = row.values.get(ci) else {
                    continue;
                };
                if rl.is_empty() {
                    continue;
                }
                let loc = || Location {
                    target: Target::Cell,
                    field_index: Some(ci as u32),
                    data_row: Some(ri as u32 + 1),
                    ..Default::default()
                };
                if !rl.contains(delim) {
                    add_at(
                        found,
                        RULE_11C,
                        Some(row.line),
                        code,
                        format!(
                            "Invalid Record Link {rl:?}: {delim:?} must separate the GROUP \
                             and KEY fields."
                        ),
                        loc(),
                        Severity::Error,
                    );
                    continue;
                }
                for link in rl.split(concat) {
                    let parts: Vec<&str> = link.split(delim).collect();
                    match fetch_count(parsed, &parts) {
                        0 => add_at(
                            found,
                            RULE_11C,
                            Some(row.line),
                            code,
                            format!("Invalid Record Link {link:?}: no such record."),
                            loc(),
                            Severity::Error,
                        ),
                        1 => {}
                        _ => add_at(
                            found,
                            RULE_11C,
                            Some(row.line),
                            code,
                            format!("Invalid Record Link {link:?}: matches more than one record."),
                            loc(),
                            Severity::Error,
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
                .all(|(i, k)| r.values.get(i).map_or("", String::as_str) == *k)
        })
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dict::DictVersion;
    use crate::parse::parse_str;

    fn run(src: &str) -> Findings {
        run_v(src, DictVersion::V4_2)
    }

    fn run_v(src: &str, v: DictVersion) -> Findings {
        let pf = parse_str(src).expect("fixture parses");
        let d = Dictionary::bundled(v);
        let mut f = Findings::new();
        check(&pf, &d, &mut f);
        f
    }

    /// Synthetic repro of the #222 / O-42 corpus file (no real data): PMTL's
    /// parent is **PMTD** in 4.0.3 (KEY includes `PMTD_SEQ`) but **PMTG** in
    /// 4.0.4+. This PMTL row has a blank `PMTD_SEQ`, so under 4.0.3 it orphans
    /// against PMTD (no PMTD row with that blank SEQ) while under 4.0.4 it
    /// matches its PMTG parent on the shorter key. PMTL is the only group in
    /// the dictionary with an edition-varying parent.
    const PMTL_EDITION_FIXTURE: &str = "\"GROUP\",\"LOCA\"\r\n\"HEADING\",\"LOCA_ID\"\r\n\
        \"UNIT\",\"\"\r\n\"TYPE\",\"ID\"\r\n\"DATA\",\"BH1\"\r\n\r\n\
        \"GROUP\",\"PMTG\"\r\n\"HEADING\",\"LOCA_ID\",\"PMTG_DPTH\",\"PMTG_TESN\"\r\n\
        \"UNIT\",\"\",\"m\",\"\"\r\n\"TYPE\",\"ID\",\"2DP\",\"ID\"\r\n\
        \"DATA\",\"BH1\",\"10.00\",\"T1\"\r\n\r\n\
        \"GROUP\",\"PMTD\"\r\n\
        \"HEADING\",\"LOCA_ID\",\"PMTG_DPTH\",\"PMTG_TESN\",\"PMTD_SEQ\"\r\n\
        \"UNIT\",\"\",\"m\",\"\",\"\"\r\n\"TYPE\",\"ID\",\"2DP\",\"ID\",\"ID\"\r\n\
        \"DATA\",\"BH1\",\"10.00\",\"T1\",\"1\"\r\n\r\n\
        \"GROUP\",\"PMTL\"\r\n\
        \"HEADING\",\"LOCA_ID\",\"PMTG_DPTH\",\"PMTG_TESN\",\"PMTL_LNO\",\"PMTD_SEQ\"\r\n\
        \"UNIT\",\"\",\"m\",\"\",\"\",\"\"\r\n\"TYPE\",\"ID\",\"2DP\",\"ID\",\"ID\",\"ID\"\r\n\
        \"DATA\",\"BH1\",\"10.00\",\"T1\",\"1\",\"\"\r\n";

    #[test]
    fn fields_with_status_dedups_a_std_heading_redeclared_in_file_dict() {
        // A file DICT overlay re-declares the standard KEY heading LOCA_ID with
        // the SAME status, so it appears in BOTH passes of fields_with_status
        // (standard headings first, then the file_hdng overlay). The dedup guard
        // must return it exactly once — a flipped guard double-pushes it, which
        // would corrupt the KEY tuple the relational rules build from this.
        let src = "\"GROUP\",\"DICT\"\r\n\
            \"HEADING\",\"DICT_TYPE\",\"DICT_GRP\",\"DICT_HDNG\",\"DICT_STAT\",\"DICT_PGRP\"\r\n\
            \"UNIT\",\"\",\"\",\"\",\"\",\"\"\r\n\"TYPE\",\"X\",\"X\",\"X\",\"X\",\"X\"\r\n\
            \"DATA\",\"HEADING\",\"LOCA\",\"LOCA_ID\",\"KEY\",\"\"\r\n";
        let parsed = parse_str(src).expect("fixture parses");
        let dict = Dictionary::bundled(DictVersion::V4_2);
        let eff = EffectiveDict::build(&parsed, dict);
        let keys = eff.fields_with_status("LOCA", "KEY");
        assert_eq!(
            keys.iter().filter(|h| h.as_str() == "LOCA_ID").count(),
            1,
            "LOCA_ID must appear once, not duplicated: {keys:?}"
        );
    }

    #[test]
    fn rule_10c_pmtl_parent_is_edition_dependent() {
        // #222 / O-42: the parent PMTL is checked against differs by edition.
        let pmtl_orphans = |v| {
            run_v(PMTL_EDITION_FIXTURE, v).get(RULE_10C).map_or(0, |f| {
                f.iter()
                    .filter(|x| x.group == "PMTL" && x.desc.contains("No parent entry"))
                    .count()
            })
        };
        // 4.0.3: PMTL→PMTD on a key incl. the (blank) PMTD_SEQ → orphan.
        assert_eq!(
            pmtl_orphans(DictVersion::V4_0_3),
            1,
            "4.0.3 should orphan PMTL→PMTD"
        );
        // 4.0.4+: PMTL→PMTG on the shorter key → matches, no orphan. python-ags4's
        // stale "4.0"→4.0.3 alias is what makes it over-report these (false positives).
        assert_eq!(
            pmtl_orphans(DictVersion::V4_0_4),
            0,
            "4.0.4 should match PMTL→PMTG"
        );
        assert_eq!(
            pmtl_orphans(DictVersion::V4_2),
            0,
            "4.2 keeps the 4.0.4 PMTL→PMTG parent"
        );
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
    fn rule_10c_reuses_the_parent_tuple_set_across_two_children_correctly() {
        // Two children of the SAME parent LOCA. The first (SAMP) builds and
        // caches LOCA's KEY-tuple set; the second (a file-DICT child QKID, also
        // parent LOCA) must reuse that cached set — flagging its own orphan while
        // accepting its own valid reference. If the memoisation keyed or reused
        // the set wrongly, QKID would either miss BH9 or wrongly flag BH1.
        let src = "\"GROUP\",\"LOCA\"\r\n\"HEADING\",\"LOCA_ID\"\r\n\
                   \"UNIT\",\"\"\r\n\"TYPE\",\"ID\"\r\n\
                   \"DATA\",\"BH1\"\r\n\"DATA\",\"BH2\"\r\n\r\n\
                   \"GROUP\",\"DICT\"\r\n\
                   \"HEADING\",\"DICT_TYPE\",\"DICT_GRP\",\"DICT_HDNG\",\"DICT_STAT\",\"DICT_PGRP\"\r\n\
                   \"UNIT\",\"\",\"\",\"\",\"\",\"\"\r\n\
                   \"TYPE\",\"X\",\"X\",\"X\",\"X\",\"X\"\r\n\
                   \"DATA\",\"GROUP\",\"QKID\",\"\",\"\",\"LOCA\"\r\n\
                   \"DATA\",\"HEADING\",\"QKID\",\"LOCA_ID\",\"KEY\",\"\"\r\n\
                   \"DATA\",\"HEADING\",\"QKID\",\"QKID_ID\",\"KEY\",\"\"\r\n\r\n\
                   \"GROUP\",\"SAMP\"\r\n\
                   \"HEADING\",\"LOCA_ID\",\"SAMP_TOP\",\"SAMP_REF\",\"SAMP_TYPE\",\"SAMP_ID\"\r\n\
                   \"UNIT\",\"\",\"m\",\"\",\"\",\"\"\r\n\
                   \"TYPE\",\"ID\",\"2DP\",\"X\",\"PA\",\"ID\"\r\n\
                   \"DATA\",\"BH1\",\"1.00\",\"S1\",\"B\",\"BH1S1\"\r\n\r\n\
                   \"GROUP\",\"QKID\"\r\n\"HEADING\",\"LOCA_ID\",\"QKID_ID\"\r\n\
                   \"UNIT\",\"\",\"\"\r\n\"TYPE\",\"ID\",\"ID\"\r\n\
                   \"DATA\",\"BH1\",\"K1\"\r\n\"DATA\",\"BH9\",\"K2\"\r\n";
        let r = run(src);
        let r10c = r.get(RULE_10C).expect("Rule 10c");
        // QKID's orphan BH9 flagged from the cached LOCA set...
        assert!(
            r10c.iter()
                .any(|x| x.group == "QKID" && x.desc.contains("BH9")),
            "QKID orphan BH9 should be flagged from the cached parent set: {r10c:?}"
        );
        // ...and QKID's genuinely-valid BH1 not flagged (the cache is LOCA's, not
        // empty or SAMP's).
        assert!(
            !r10c
                .iter()
                .any(|x| x.group == "QKID" && x.desc.contains("BH1")),
            "QKID's BH1 has a real LOCA parent: {r10c:?}"
        );
        // SAMP (which populated the cache) is itself clean.
        assert!(
            !r10c.iter().any(|x| x.group == "SAMP"),
            "SAMP's only row references BH1 which exists: {r10c:?}"
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
            r.get(RULE_10C).is_none_or(std::vec::Vec::is_empty),
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

    #[test]
    fn rule_11c_flags_link_matching_more_than_one_record() {
        // Two LOCA rows share leading column "BH1" (LOCA's ID column
        // isn't KEY-deduped here — fetch_record is positional, O-24), so
        // a link "LOCA|BH1" resolves to 2 rows → the `_ => matches more
        // than one` arm of rule_11c.
        let src = "\"GROUP\",\"TRAN\"\r\n\
                   \"HEADING\",\"TRAN_DLIM\",\"TRAN_RCON\"\r\n\
                   \"UNIT\",\"\",\"\"\r\n\"TYPE\",\"X\",\"X\"\r\n\"DATA\",\"|\",\"+\"\r\n\r\n\
                   \"GROUP\",\"LOCA\"\r\n\"HEADING\",\"LOCA_ID\"\r\n\
                   \"UNIT\",\"\"\r\n\"TYPE\",\"ID\"\r\n\
                   \"DATA\",\"BH1\"\r\n\"DATA\",\"BH1\"\r\n\r\n\
                   \"GROUP\",\"SAMP\"\r\n\"HEADING\",\"LOCA_ID\",\"SAMP_LINK\"\r\n\
                   \"UNIT\",\"\",\"\"\r\n\"TYPE\",\"ID\",\"RL\"\r\n\
                   \"DATA\",\"BH1\",\"LOCA|BH1\"\r\n";
        let r = run(src);
        let r11c = r.get(RULE_11C).expect("Rule 11c");
        assert!(
            r11c.iter()
                .any(|x| x.desc.contains("matches more than one record")),
            "{r11c:?}"
        );
    }

    // ---- mutation-sweep additions: finding LOCATIONS + fetch_count bounds ----
    //
    // The existing tests assert a finding *fired* and its message; these pin the
    // Location it carries (target / heading / field_index / 1-based data_row) and
    // the exact fetch_count guards — the coordinates every surface renders.

    /// Rule 10a: a missing KEY points at the HEADING row naming the field; a
    /// duplicate KEY points at the offending 1-based data row.
    #[test]
    fn rule_10a_finding_locations() {
        let dup = "\"GROUP\",\"LOCA\"\r\n\"HEADING\",\"LOCA_ID\",\"LOCA_FDEP\"\r\n\
                   \"UNIT\",\"\",\"m\"\r\n\"TYPE\",\"ID\",\"2DP\"\r\n\
                   \"DATA\",\"BH1\",\"1.00\"\r\n\"DATA\",\"BH1\",\"2.00\"\r\n";
        let r = run(dup);
        let dupf = r
            .get(RULE_10A)
            .unwrap()
            .iter()
            .find(|x| x.desc.contains("Duplicate") && x.location.data_row == Some(2))
            .expect("2nd duplicate row flagged at data_row 2");
        assert_eq!(dupf.location.target, Target::Cell);

        let miss = "\"GROUP\",\"LOCA\"\r\n\"HEADING\",\"LOCA_FDEP\"\r\n\
                    \"UNIT\",\"m\"\r\n\"TYPE\",\"2DP\"\r\n\"DATA\",\"1.00\"\r\n";
        let missf = run(miss)
            .get(RULE_10A)
            .unwrap()
            .iter()
            .find(|x| x.desc.contains("not present"))
            .cloned()
            .expect("missing KEY flagged");
        assert_eq!(missf.location.target, Target::Heading);
        assert_eq!(missf.location.heading.as_deref(), Some("LOCA_ID"));
    }

    /// Rule 10b: a missing REQUIRED field is flagged at the HEADING row (naming
    /// it — the present-check must actually compare names); an empty REQUIRED
    /// cell is flagged at its 1-based data row.
    #[test]
    fn rule_10b_finding_locations() {
        let miss = "\"GROUP\",\"PROJ\"\r\n\"HEADING\",\"PROJ_NAME\"\r\n\
                    \"UNIT\",\"\"\r\n\"TYPE\",\"X\"\r\n\"DATA\",\"Site\"\r\n";
        let missf = run(miss)
            .get(RULE_10B)
            .expect("missing REQUIRED fires")
            .iter()
            .find(|x| x.desc.contains("PROJ_ID") && x.desc.contains("not present"))
            .cloned()
            .expect("PROJ_ID missing flagged");
        assert_eq!(missf.location.target, Target::Heading);
        assert_eq!(missf.location.heading.as_deref(), Some("PROJ_ID"));

        let empty = "\"GROUP\",\"PROJ\"\r\n\"HEADING\",\"PROJ_ID\",\"PROJ_NAME\"\r\n\
                     \"UNIT\",\"\",\"\"\r\n\"TYPE\",\"ID\",\"X\"\r\n\"DATA\",\"\",\"Site\"\r\n";
        let r = run(empty);
        let emptyf = &r.get(RULE_10B).expect("empty REQUIRED fires")[0];
        assert_eq!(emptyf.location.target, Target::Cell);
        assert_eq!(emptyf.location.data_row, Some(1));
    }

    /// Rule 10c orphan finding points at the offending child row (1-based).
    #[test]
    fn rule_10c_orphan_finding_location() {
        let src = "\"GROUP\",\"LOCA\"\r\n\"HEADING\",\"LOCA_ID\"\r\n\
                   \"UNIT\",\"\"\r\n\"TYPE\",\"ID\"\r\n\"DATA\",\"BH1\"\r\n\r\n\
                   \"GROUP\",\"SAMP\"\r\n\
                   \"HEADING\",\"LOCA_ID\",\"SAMP_TOP\",\"SAMP_REF\",\"SAMP_TYPE\",\"SAMP_ID\"\r\n\
                   \"UNIT\",\"\",\"m\",\"\",\"\",\"\"\r\n\
                   \"TYPE\",\"ID\",\"2DP\",\"X\",\"PA\",\"ID\"\r\n\
                   \"DATA\",\"BH1\",\"1.00\",\"S1\",\"B\",\"BH1S1\"\r\n\
                   \"DATA\",\"BH9\",\"2.00\",\"S2\",\"B\",\"BH9S2\"\r\n";
        let orphan = run(src)
            .get(RULE_10C)
            .expect("orphan fires")
            .iter()
            .find(|x| x.desc.contains("BH9"))
            .cloned()
            .expect("BH9 orphan");
        assert_eq!(orphan.location.target, Target::Cell);
        assert_eq!(orphan.location.data_row, Some(2)); // BH9 is SAMP's 2nd row
    }

    /// Rule 11c: a bad record link points at the RL cell (column index + 1-based
    /// row); a link resolving to EXACTLY one record is not flagged.
    #[test]
    fn rule_11c_finding_location_and_valid_link_passes() {
        let bad = "\"GROUP\",\"TRAN\"\r\n\
                   \"HEADING\",\"TRAN_DLIM\",\"TRAN_RCON\"\r\n\
                   \"UNIT\",\"\",\"\"\r\n\"TYPE\",\"X\",\"X\"\r\n\"DATA\",\"|\",\"+\"\r\n\r\n\
                   \"GROUP\",\"LOCA\"\r\n\"HEADING\",\"LOCA_ID\"\r\n\
                   \"UNIT\",\"\"\r\n\"TYPE\",\"ID\"\r\n\"DATA\",\"BH1\"\r\n\r\n\
                   \"GROUP\",\"SAMP\"\r\n\"HEADING\",\"LOCA_ID\",\"SAMP_LINK\"\r\n\
                   \"UNIT\",\"\",\"\"\r\n\"TYPE\",\"ID\",\"RL\"\r\n\
                   \"DATA\",\"BH1\",\"LOCA|BH404\"\r\n";
        let rbad = run(bad);
        let f = &rbad.get(RULE_11C).expect("bad link fires")[0];
        assert_eq!(f.location.target, Target::Cell);
        assert_eq!(f.location.field_index, Some(1)); // SAMP_LINK is column 1
        assert_eq!(f.location.data_row, Some(1)); // the only SAMP row

        let good = "\"GROUP\",\"TRAN\"\r\n\
                    \"HEADING\",\"TRAN_DLIM\",\"TRAN_RCON\"\r\n\
                    \"UNIT\",\"\",\"\"\r\n\"TYPE\",\"X\",\"X\"\r\n\"DATA\",\"|\",\"+\"\r\n\r\n\
                    \"GROUP\",\"LOCA\"\r\n\"HEADING\",\"LOCA_ID\"\r\n\
                    \"UNIT\",\"\"\r\n\"TYPE\",\"ID\"\r\n\"DATA\",\"BH1\"\r\n\r\n\
                    \"GROUP\",\"SAMP\"\r\n\"HEADING\",\"LOCA_ID\",\"SAMP_LINK\"\r\n\
                    \"UNIT\",\"\",\"\"\r\n\"TYPE\",\"ID\",\"RL\"\r\n\
                    \"DATA\",\"BH1\",\"LOCA|BH1\"\r\n";
        assert!(
            !run(good).contains_key(RULE_11C),
            "a link matching exactly one record must not be flagged"
        );
    }

    /// `fetch_count` guards: empty keys must NOT match every row, and fewer keys
    /// than columns is the normal (matching) case.
    #[test]
    fn fetch_count_key_guards() {
        let pf = parse_str(
            "\"GROUP\",\"LOCA\"\r\n\"HEADING\",\"LOCA_ID\",\"LOCA_FDEP\"\r\n\
             \"UNIT\",\"\",\"m\"\r\n\"TYPE\",\"ID\",\"2DP\"\r\n\
             \"DATA\",\"BH1\",\"1.00\"\r\n\"DATA\",\"BH2\",\"2.00\"\r\n",
        )
        .unwrap();
        assert_eq!(fetch_count(&pf, &["LOCA"]), 0); // no keys → not "all rows"
        assert_eq!(fetch_count(&pf, &["LOCA", "BH1"]), 1); // fewer keys than cols
        assert_eq!(fetch_count(&pf, &["NOPE", "x"]), 0); // unknown group
    }

    #[test]
    fn rule_11_silent_when_tran_has_no_data_row() {
        // TRAN group with the delimiter headings but zero DATA rows →
        // rule_11's `tran.rows.first()` None early return (no 11a/11b/c).
        let src = "\"GROUP\",\"PROJ\"\r\n\"HEADING\",\"PROJ_ID\"\r\n\
                   \"UNIT\",\"\"\r\n\"TYPE\",\"ID\"\r\n\"DATA\",\"P1\"\r\n\r\n\
                   \"GROUP\",\"TRAN\"\r\n\"HEADING\",\"TRAN_DLIM\",\"TRAN_RCON\"\r\n\
                   \"UNIT\",\"\",\"\"\r\n\"TYPE\",\"X\",\"X\"\r\n";
        let r = run(src);
        assert!(!r.contains_key(RULE_11A), "{r:?}");
        assert!(!r.contains_key(RULE_11B), "{r:?}");
        assert!(!r.contains_key(RULE_11C), "{r:?}");
    }

    #[test]
    fn rule_11_silent_when_tran_lacks_delim_columns() {
        // TRAN present, has a DATA row, but no TRAN_DLIM/TRAN_RCON
        // columns → the `(Some, Some)` column-resolve None arm.
        let src = "\"GROUP\",\"TRAN\"\r\n\"HEADING\",\"TRAN_AGS\"\r\n\
                   \"UNIT\",\"\"\r\n\"TYPE\",\"X\"\r\n\"DATA\",\"4.2\"\r\n";
        let r = run(src);
        assert!(!r.contains_key(RULE_11A), "{r:?}");
        assert!(!r.contains_key(RULE_11B), "{r:?}");
    }

    // ---- Rule 10c error branches (no/blank/absent parent, key mismatch) ----

    #[test]
    fn rule_10c_flags_parent_group_absent_from_file() {
        // SAMP's parent LOCA is in the standard dictionary but not in the
        // file → the "Parent group … is not in the file" arm.
        let src = "\"GROUP\",\"SAMP\"\r\n\
                   \"HEADING\",\"LOCA_ID\",\"SAMP_TOP\",\"SAMP_REF\",\"SAMP_TYPE\",\"SAMP_ID\"\r\n\
                   \"UNIT\",\"\",\"m\",\"\",\"\",\"\"\r\n\
                   \"TYPE\",\"ID\",\"2DP\",\"X\",\"PA\",\"ID\"\r\n\
                   \"DATA\",\"BH1\",\"1.00\",\"S1\",\"B\",\"BH1S1\"\r\n";
        let r = run(src);
        let r10c = r.get(RULE_10C).expect("Rule 10c");
        assert!(
            r10c.iter()
                .any(|x| x.group == "SAMP" && x.desc.contains("is not in the file")),
            "{r10c:?}"
        );
    }

    #[test]
    fn rule_10c_flags_group_undefined_anywhere() {
        // A custom group "QZZZ" with no standard- or DICT-defined parent
        // → the `eff.parent(code) = None` "group not defined" arm. It's
        // not in PARENTLESS, so 10c runs and can't find a parent at all.
        let src = "\"GROUP\",\"QZZZ\"\r\n\"HEADING\",\"QZZZ_ID\"\r\n\
                   \"UNIT\",\"\"\r\n\"TYPE\",\"ID\"\r\n\"DATA\",\"X1\"\r\n";
        let r = run(src);
        let r10c = r.get(RULE_10C).expect("Rule 10c");
        assert!(
            r10c.iter()
                .any(|x| x.group == "QZZZ" && x.desc.contains("not defined")),
            "{r10c:?}"
        );
    }

    #[test]
    fn rule_10c_uses_file_dict_to_define_a_custom_parent_chain() {
        // A file DICT defines QZZZ (parent PROJ, KEY=QZZZ_ID) and child
        // QCHD (parent QZZZ, KEY borrowing QZZZ_ID). QCHD's data row
        // references a QZZZ_ID that doesn't exist → orphan flagged via
        // the file-DICT overlay path (EffectiveDict::build with a DICT
        // group, exercising file_parent + file_hdng + fields_with_status
        // extras). PROJ is the QZZZ parent (PARENTLESS-exempt LOCA-style
        // not used).
        let src = "\"GROUP\",\"PROJ\"\r\n\"HEADING\",\"PROJ_ID\"\r\n\
                   \"UNIT\",\"\"\r\n\"TYPE\",\"ID\"\r\n\"DATA\",\"P1\"\r\n\r\n\
                   \"GROUP\",\"DICT\"\r\n\
                   \"HEADING\",\"DICT_TYPE\",\"DICT_GRP\",\"DICT_HDNG\",\"DICT_STAT\",\"DICT_PGRP\"\r\n\
                   \"UNIT\",\"\",\"\",\"\",\"\",\"\"\r\n\
                   \"TYPE\",\"X\",\"X\",\"X\",\"X\",\"X\"\r\n\
                   \"DATA\",\"GROUP\",\"QZZZ\",\"\",\"\",\"PROJ\"\r\n\
                   \"DATA\",\"HEADING\",\"QZZZ\",\"QZZZ_ID\",\"KEY\",\"\"\r\n\
                   \"DATA\",\"GROUP\",\"QCHD\",\"\",\"\",\"QZZZ\"\r\n\
                   \"DATA\",\"HEADING\",\"QCHD\",\"QZZZ_ID\",\"KEY\",\"\"\r\n\
                   \"DATA\",\"HEADING\",\"QCHD\",\"QCHD_ID\",\"KEY\",\"\"\r\n\r\n\
                   \"GROUP\",\"QZZZ\"\r\n\"HEADING\",\"QZZZ_ID\"\r\n\
                   \"UNIT\",\"\"\r\n\"TYPE\",\"ID\"\r\n\"DATA\",\"Z1\"\r\n\r\n\
                   \"GROUP\",\"QCHD\"\r\n\"HEADING\",\"QZZZ_ID\",\"QCHD_ID\"\r\n\
                   \"UNIT\",\"\",\"\"\r\n\"TYPE\",\"ID\",\"ID\"\r\n\
                   \"DATA\",\"Z9\",\"C1\"\r\n";
        let r = run(src);
        let r10c = r.get(RULE_10C).expect("Rule 10c");
        // QCHD row references QZZZ_ID "Z9" which has no QZZZ parent (Z1).
        assert!(
            r10c.iter()
                .any(|x| x.group == "QCHD" && x.desc.contains("Z9")),
            "file-DICT-defined parent chain must drive the orphan check: {r10c:?}"
        );
    }

    #[test]
    fn rule_10b_uses_file_dict_required_status() {
        // A file DICT marks QZZZ_REF as REQUIRED for a custom group;
        // an empty cell there → Rule 10b fires via the file_hdng overlay
        // (fields_with_status extras branch).
        let src = "\"GROUP\",\"DICT\"\r\n\
                   \"HEADING\",\"DICT_TYPE\",\"DICT_GRP\",\"DICT_HDNG\",\"DICT_STAT\",\"DICT_PGRP\"\r\n\
                   \"UNIT\",\"\",\"\",\"\",\"\",\"\"\r\n\
                   \"TYPE\",\"X\",\"X\",\"X\",\"X\",\"X\"\r\n\
                   \"DATA\",\"GROUP\",\"QZZZ\",\"\",\"\",\"-\"\r\n\
                   \"DATA\",\"HEADING\",\"QZZZ\",\"QZZZ_ID\",\"KEY\",\"\"\r\n\
                   \"DATA\",\"HEADING\",\"QZZZ\",\"QZZZ_REF\",\"REQUIRED\",\"\"\r\n\r\n\
                   \"GROUP\",\"QZZZ\"\r\n\"HEADING\",\"QZZZ_ID\",\"QZZZ_REF\"\r\n\
                   \"UNIT\",\"\",\"\"\r\n\"TYPE\",\"ID\",\"X\"\r\n\
                   \"DATA\",\"Z1\",\"\"\r\n";
        let r = run(src);
        let r10b = r.get(RULE_10B).expect("Rule 10b");
        assert!(
            r10b.iter()
                .any(|x| x.group == "QZZZ" && x.desc.contains("QZZZ_REF")),
            "file-DICT REQUIRED status must drive Rule 10b: {r10b:?}"
        );
    }

    #[test]
    fn rule_10c_flags_parent_with_no_key_fields() {
        // File-DICT: parent QPAR has NO KEY heading; child QCHD names
        // QPAR as parent. pkeys is empty → the "No KEY fields are defined
        // in the parent group" arm (lines 393-401).
        let src = "\"GROUP\",\"DICT\"\r\n\
                   \"HEADING\",\"DICT_TYPE\",\"DICT_GRP\",\"DICT_HDNG\",\"DICT_STAT\",\"DICT_PGRP\"\r\n\
                   \"UNIT\",\"\",\"\",\"\",\"\",\"\"\r\n\
                   \"TYPE\",\"X\",\"X\",\"X\",\"X\",\"X\"\r\n\
                   \"DATA\",\"GROUP\",\"QPAR\",\"\",\"\",\"-\"\r\n\
                   \"DATA\",\"HEADING\",\"QPAR\",\"QPAR_VAL\",\"OTHER\",\"\"\r\n\
                   \"DATA\",\"GROUP\",\"QCHD\",\"\",\"\",\"QPAR\"\r\n\
                   \"DATA\",\"HEADING\",\"QCHD\",\"QCHD_ID\",\"KEY\",\"\"\r\n\r\n\
                   \"GROUP\",\"QPAR\"\r\n\"HEADING\",\"QPAR_VAL\"\r\n\
                   \"UNIT\",\"\"\r\n\"TYPE\",\"X\"\r\n\"DATA\",\"v\"\r\n\r\n\
                   \"GROUP\",\"QCHD\"\r\n\"HEADING\",\"QCHD_ID\"\r\n\
                   \"UNIT\",\"\"\r\n\"TYPE\",\"ID\"\r\n\"DATA\",\"C1\"\r\n";
        let r = run(src);
        let r10c = r.get(RULE_10C).expect("Rule 10c");
        assert!(
            r10c.iter()
                .any(|x| x.group == "QCHD" && x.desc.contains("No KEY fields")),
            "{r10c:?}"
        );
    }

    #[test]
    fn rule_10c_flags_parent_key_not_declared_in_child() {
        // File-DICT: parent QPAR has KEY=QPAR_ID; child QCHD names QPAR
        // as parent but does NOT declare QPAR_ID as one of its KEYs. So
        // a parent KEY is "missing from the child" → lines 404-417.
        let src = "\"GROUP\",\"DICT\"\r\n\
                   \"HEADING\",\"DICT_TYPE\",\"DICT_GRP\",\"DICT_HDNG\",\"DICT_STAT\",\"DICT_PGRP\"\r\n\
                   \"UNIT\",\"\",\"\",\"\",\"\",\"\"\r\n\
                   \"TYPE\",\"X\",\"X\",\"X\",\"X\",\"X\"\r\n\
                   \"DATA\",\"GROUP\",\"QPAR\",\"\",\"\",\"-\"\r\n\
                   \"DATA\",\"HEADING\",\"QPAR\",\"QPAR_ID\",\"KEY\",\"\"\r\n\
                   \"DATA\",\"GROUP\",\"QCHD\",\"\",\"\",\"QPAR\"\r\n\
                   \"DATA\",\"HEADING\",\"QCHD\",\"QCHD_ID\",\"KEY\",\"\"\r\n\r\n\
                   \"GROUP\",\"QPAR\"\r\n\"HEADING\",\"QPAR_ID\"\r\n\
                   \"UNIT\",\"\"\r\n\"TYPE\",\"ID\"\r\n\"DATA\",\"P1\"\r\n\r\n\
                   \"GROUP\",\"QCHD\"\r\n\"HEADING\",\"QCHD_ID\"\r\n\
                   \"UNIT\",\"\"\r\n\"TYPE\",\"ID\"\r\n\"DATA\",\"C1\"\r\n";
        let r = run(src);
        let r10c = r.get(RULE_10C).expect("Rule 10c");
        assert!(
            r10c.iter().any(|x| x.group == "QCHD"
                && x.desc.contains("QPAR_ID")
                && x.desc.contains("not in the child group")),
            "{r10c:?}"
        );
    }

    #[test]
    fn rule_10c_flags_key_column_absent_from_heading_row() {
        // File-DICT: both QPAR and QCHD declare QPAR_ID as KEY (so the
        // effective-dict KEY sets line up and `missing` is empty), but
        // the child's actual HEADING row OMITS the QPAR_ID column → the
        // `!(in_child && in_parent)` arm (lines 420-435), reported at the
        // child's HEADING line.
        let src = "\"GROUP\",\"DICT\"\r\n\
                   \"HEADING\",\"DICT_TYPE\",\"DICT_GRP\",\"DICT_HDNG\",\"DICT_STAT\",\"DICT_PGRP\"\r\n\
                   \"UNIT\",\"\",\"\",\"\",\"\",\"\"\r\n\
                   \"TYPE\",\"X\",\"X\",\"X\",\"X\",\"X\"\r\n\
                   \"DATA\",\"GROUP\",\"QPAR\",\"\",\"\",\"-\"\r\n\
                   \"DATA\",\"HEADING\",\"QPAR\",\"QPAR_ID\",\"KEY\",\"\"\r\n\
                   \"DATA\",\"GROUP\",\"QCHD\",\"\",\"\",\"QPAR\"\r\n\
                   \"DATA\",\"HEADING\",\"QCHD\",\"QPAR_ID\",\"KEY\",\"\"\r\n\
                   \"DATA\",\"HEADING\",\"QCHD\",\"QCHD_ID\",\"KEY\",\"\"\r\n\r\n\
                   \"GROUP\",\"QPAR\"\r\n\"HEADING\",\"QPAR_ID\"\r\n\
                   \"UNIT\",\"\"\r\n\"TYPE\",\"ID\"\r\n\"DATA\",\"P1\"\r\n\r\n\
                   \"GROUP\",\"QCHD\"\r\n\"HEADING\",\"QCHD_ID\"\r\n\
                   \"UNIT\",\"\"\r\n\"TYPE\",\"ID\"\r\n\"DATA\",\"C1\"\r\n";
        let r = run(src);
        let r10c = r.get(RULE_10C).expect("Rule 10c");
        assert!(
            r10c.iter()
                .any(|x| x.group == "QCHD" && x.desc.contains("KEY fields missing")),
            "{r10c:?}"
        );
    }

    #[test]
    fn rule_11c_ignores_empty_record_link_cell() {
        // An RL-typed column whose value is empty must be skipped (the
        // `rl.is_empty()` continue in rule_11c) — no Rule 11c finding.
        let src = "\"GROUP\",\"TRAN\"\r\n\
                   \"HEADING\",\"TRAN_DLIM\",\"TRAN_RCON\"\r\n\
                   \"UNIT\",\"\",\"\"\r\n\"TYPE\",\"X\",\"X\"\r\n\"DATA\",\"|\",\"+\"\r\n\r\n\
                   \"GROUP\",\"LOCA\"\r\n\"HEADING\",\"LOCA_ID\"\r\n\
                   \"UNIT\",\"\"\r\n\"TYPE\",\"ID\"\r\n\"DATA\",\"BH1\"\r\n\r\n\
                   \"GROUP\",\"SAMP\"\r\n\"HEADING\",\"LOCA_ID\",\"SAMP_LINK\"\r\n\
                   \"UNIT\",\"\",\"\"\r\n\"TYPE\",\"ID\",\"RL\"\r\n\
                   \"DATA\",\"BH1\",\"\"\r\n";
        assert!(!run(src).contains_key(RULE_11C), "{:?}", run(src));
    }

    #[test]
    fn rule_11c_unknown_group_in_link_is_no_such_record() {
        // Link references group "NOPE" that isn't in the file →
        // fetch_count's `parsed.groups.get == None` returns 0 → "no such
        // record". Also covers a link whose key count exceeds the target
        // group's heading count (fetch_count's `keys.len() > headings`).
        let src = "\"GROUP\",\"TRAN\"\r\n\
                   \"HEADING\",\"TRAN_DLIM\",\"TRAN_RCON\"\r\n\
                   \"UNIT\",\"\",\"\"\r\n\"TYPE\",\"X\",\"X\"\r\n\"DATA\",\"|\",\"+\"\r\n\r\n\
                   \"GROUP\",\"LOCA\"\r\n\"HEADING\",\"LOCA_ID\"\r\n\
                   \"UNIT\",\"\"\r\n\"TYPE\",\"ID\"\r\n\"DATA\",\"BH1\"\r\n\r\n\
                   \"GROUP\",\"SAMP\"\r\n\"HEADING\",\"LOCA_ID\",\"SAMP_LINK\"\r\n\
                   \"UNIT\",\"\",\"\"\r\n\"TYPE\",\"ID\",\"RL\"\r\n\
                   \"DATA\",\"BH1\",\"NOPE|x\"\r\n\"DATA\",\"BH1\",\"LOCA|a|b|c\"\r\n";
        let r = run(src);
        let r11c = r.get(RULE_11C).expect("Rule 11c");
        // Both bad links resolve to 0 records → "no such record".
        assert!(
            r11c.iter()
                .filter(|x| x.desc.contains("no such record"))
                .count()
                >= 2,
            "{r11c:?}"
        );
    }

    #[test]
    fn rule_10c_flags_blank_parent_in_file_dict() {
        // A file-DICT GROUP row whose DICT_PGRP is blank ("") for a
        // non-PARENTLESS custom group → eff.parent returns Some("") →
        // the "Parent group is left blank" arm.
        let src = "\"GROUP\",\"DICT\"\r\n\
                   \"HEADING\",\"DICT_TYPE\",\"DICT_GRP\",\"DICT_HDNG\",\"DICT_STAT\",\"DICT_PGRP\"\r\n\
                   \"UNIT\",\"\",\"\",\"\",\"\",\"\"\r\n\
                   \"TYPE\",\"X\",\"X\",\"X\",\"X\",\"X\"\r\n\
                   \"DATA\",\"GROUP\",\"QZZZ\",\"\",\"\",\"\"\r\n\
                   \"DATA\",\"HEADING\",\"QZZZ\",\"QZZZ_ID\",\"KEY\",\"\"\r\n\r\n\
                   \"GROUP\",\"QZZZ\"\r\n\"HEADING\",\"QZZZ_ID\"\r\n\
                   \"UNIT\",\"\"\r\n\"TYPE\",\"ID\"\r\n\"DATA\",\"Z1\"\r\n";
        let r = run(src);
        let r10c = r.get(RULE_10C).expect("Rule 10c");
        assert!(
            r10c.iter()
                .any(|x| x.group == "QZZZ" && x.desc.contains("left blank")),
            "{r10c:?}"
        );
    }
}
