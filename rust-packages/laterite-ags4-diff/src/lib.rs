//! Revision diff — a KEY-aware, type-aware comparison of two AGS4 files.
//!
//! A KEY-aware, type-aware comparison of two AGS4 files (a baseline `a` and a
//! revision `b`). Rows within a group are matched by the group's *dictionary*
//! KEY headings, not by line order — so a re-sorted or re-numbered file still
//! pairs the same boreholes/samples. Matched cells are compared through
//! `laterite_types::parse_value`, so a formatting-only change ("1.0" → "1.00",
//! trailing whitespace, an equivalent datetime spelling) is NOT reported —
//! only a genuine typed change is. This is the engine-consistent diff a
//! plain line diff can't be: it understands the data model.
//!
//! Fallback: a group with no dictionary KEY headings present in both files
//! (a custom/passthrough group) is matched on its whole row tuple, so a
//! changed row there shows as a remove + add pair (and `keyed` is false).

use laterite_ags4_parse::{DataRow, ParsedFile, ParsedGroup};
use laterite_ags4_reference::dict::Dictionary;
use laterite_ags4_reference::keychain::key_heading_names;
use laterite_ags4_reference::union::registry;
use laterite_types::parse_value;
use serde::Serialize;

/// One changed cell of a matched row.
#[derive(Serialize)]
pub struct CellDelta {
    pub heading: String,
    #[serde(rename = "type")]
    pub ags_type: String,
    /// raw value in the baseline / revision (`null` if the row is shorter
    /// than the heading list on that side).
    pub a: Option<String>,
    pub b: Option<String>,
}

/// One row's verdict: added (only in `b`), removed (only in `a`), or changed
/// (matched by KEY, ≥1 cell differs).
#[derive(Serialize)]
pub struct RowDelta {
    pub kind: &'static str,
    /// the KEY values (or whole-row tuple, when unkeyed) identifying the row.
    pub key: Vec<String>,
    pub line_a: Option<u32>,
    pub line_b: Option<u32>,
    /// changed cells — populated only for `kind == "changed"`.
    pub cells: Vec<CellDelta>,
}

#[derive(Serialize)]
pub struct GroupDelta {
    pub code: String,
    /// true totals (independent of any `rows` cap).
    pub added: usize,
    pub removed: usize,
    pub changed: usize,
    /// headings present only in `b` / only in `a` (structural change).
    pub headings_added: Vec<String>,
    pub headings_removed: Vec<String>,
    /// false ⇒ matched on whole-row tuple (no dictionary KEY headings).
    pub keyed: bool,
    /// the KEY heading names used to match rows + label them.
    pub key_headings: Vec<String>,
    /// the per-row deltas (capped by `max_rows_per_group`).
    pub rows: Vec<RowDelta>,
}

#[derive(Serialize)]
pub struct RevisionDelta {
    /// groups with ≥1 row/heading change, in `b`'s file order then `a`-only.
    pub groups: Vec<GroupDelta>,
    pub groups_added: Vec<String>,
    pub groups_removed: Vec<String>,
    pub total_added: usize,
    pub total_removed: usize,
    pub total_changed: usize,
}

/// heading name → column index, for O(1) cell lookup by name on each side.
fn heading_index(headings: &[String]) -> std::collections::HashMap<&str, usize> {
    headings
        .iter()
        .enumerate()
        .map(|(i, h)| (h.as_str(), i))
        .collect()
}

/// Composite match-key for a row: the KEY-heading cell values (keyed), else
/// the whole row tuple (unkeyed fallback).
fn row_key(
    row: &DataRow,
    idx: &std::collections::HashMap<&str, usize>,
    key_headings: &[String],
    keyed: bool,
) -> Vec<String> {
    if keyed {
        key_headings
            .iter()
            .map(|h| {
                idx.get(h.as_str())
                    .and_then(|&i| row.values.get(i))
                    .cloned()
                    .unwrap_or_default()
            })
            .collect()
    } else {
        row.values.clone()
    }
}

/// Cells of a matched row that genuinely differ. A cell counts as changed
/// only when its raw values differ AND they don't canonicalise to the same
/// non-null typed value (so "1.0"/"1.00" is suppressed). Compared over the
/// headings common to both files; structural heading adds/removes are
/// reported at the group level instead.
#[allow(clippy::too_many_arguments)]
fn changed_cells(
    code: &str,
    common: &[String],
    row_a: &DataRow,
    idx_a: &std::collections::HashMap<&str, usize>,
    types_a: &[String],
    row_b: &DataRow,
    idx_b: &std::collections::HashMap<&str, usize>,
    types_b: &[String],
    dict: &Dictionary,
) -> Vec<CellDelta> {
    let mut out = Vec::new();
    for h in common {
        let a = idx_a
            .get(h.as_str())
            .and_then(|&i| row_a.values.get(i))
            .map(String::as_str);
        let b = idx_b
            .get(h.as_str())
            .and_then(|&i| row_b.values.get(i))
            .map(String::as_str);
        // AGS type, resolved INDEPENDENTLY per side (own file TYPE row, then the
        // dictionary, then opaque "X"), so a heading two files typed differently is
        // compared on each side's real type rather than cross-contaminating through
        // one shared type. (When both files agree on TYPE — the common case —
        // ty_a == ty_b and this is identical to the old single-type comparison.)
        let ty_for = |idx: &std::collections::HashMap<&str, usize>, types: &[String]| -> String {
            idx.get(h.as_str())
                .and_then(|&i| types.get(i))
                .map(String::as_str)
                .or_else(|| dict.heading(code, h).map(|e| e.ags_type))
                .unwrap_or("X")
                .to_string()
        };
        let ty_a = ty_for(idx_a, types_a);
        let ty_b = ty_for(idx_b, types_b);
        let va = parse_value(a, &ty_a);
        let vb = parse_value(b, &ty_b);
        let typed_equal = !va.is_null() && va == vb;
        if a != b && !typed_equal {
            out.push(CellDelta {
                heading: h.clone(),
                ags_type: ty_b.clone(),
                a: a.map(str::to_string),
                b: b.map(str::to_string),
            });
        }
    }
    out
}

fn diff_group(
    code: &str,
    ga: &ParsedGroup,
    gb: &ParsedGroup,
    dict: &Dictionary,
    cap: Option<usize>,
) -> GroupDelta {
    let set_a: std::collections::BTreeSet<&str> = ga.headings.iter().map(String::as_str).collect();
    let set_b: std::collections::BTreeSet<&str> = gb.headings.iter().map(String::as_str).collect();
    let headings_added: Vec<String> = gb
        .headings
        .iter()
        .filter(|h| !set_a.contains(h.as_str()))
        .cloned()
        .collect();
    let headings_removed: Vec<String> = ga
        .headings
        .iter()
        .filter(|h| !set_b.contains(h.as_str()))
        .cloned()
        .collect();
    let common: Vec<String> = gb
        .headings
        .iter()
        .filter(|h| set_a.contains(h.as_str()))
        .cloned()
        .collect();

    // KEY headings that exist on BOTH sides (so they can index either row), from
    // the ONE shared definition (`key_heading_names`) that `laterite-ags4-merge`
    // and the content-addressed `_id` also consume — diff no longer re-derives
    // "what identifies a row" from per-edition status. Proven equivalent across
    // every bundled edition by `union_key_headings_agree_with_per_edition_status`.
    let key_headings: Vec<String> = registry()
        .get(code)
        .map(|g| {
            key_heading_names(g)
                .iter()
                .filter(|h| set_a.contains(**h) && set_b.contains(**h))
                .map(std::string::ToString::to_string)
                .collect()
        })
        .unwrap_or_default();
    let keyed = !key_headings.is_empty();

    let idx_a = heading_index(&ga.headings);
    let idx_b = heading_index(&gb.headings);

    // Index B rows by key → queue of row indices (a queue pairs duplicate
    // keys in file order rather than collapsing them).
    let mut b_by_key: std::collections::HashMap<Vec<String>, std::collections::VecDeque<usize>> =
        std::collections::HashMap::new();
    for (i, row) in gb.rows.iter().enumerate() {
        b_by_key
            .entry(row_key(row, &idx_b, &key_headings, keyed))
            .or_default()
            .push_back(i);
    }

    let mut rows: Vec<RowDelta> = Vec::new();
    let mut added = 0usize;
    let mut removed = 0usize;
    let mut changed = 0usize;
    let mut matched_b = vec![false; gb.rows.len()];
    let under_cap = |rows: &Vec<RowDelta>| cap.is_none_or(|c| rows.len() < c);

    for row_a in &ga.rows {
        let k = row_key(row_a, &idx_a, &key_headings, keyed);
        if let Some(bi) = b_by_key
            .get_mut(&k)
            .and_then(std::collections::VecDeque::pop_front)
        {
            matched_b[bi] = true;
            let row_b = &gb.rows[bi];
            let cells = changed_cells(
                code, &common, row_a, &idx_a, &ga.types, row_b, &idx_b, &gb.types, dict,
            );
            if !cells.is_empty() {
                changed += 1;
                if under_cap(&rows) {
                    rows.push(RowDelta {
                        kind: "changed",
                        key: k,
                        line_a: Some(row_a.line),
                        line_b: Some(row_b.line),
                        cells,
                    });
                }
            }
        } else {
            removed += 1;
            if under_cap(&rows) {
                rows.push(RowDelta {
                    kind: "removed",
                    key: k,
                    line_a: Some(row_a.line),
                    line_b: None,
                    cells: Vec::new(),
                });
            }
        }
    }
    for (i, row_b) in gb.rows.iter().enumerate() {
        if !matched_b[i] {
            added += 1;
            if under_cap(&rows) {
                rows.push(RowDelta {
                    kind: "added",
                    key: row_key(row_b, &idx_b, &key_headings, keyed),
                    line_a: None,
                    line_b: Some(row_b.line),
                    cells: Vec::new(),
                });
            }
        }
    }

    GroupDelta {
        code: code.to_string(),
        added,
        removed,
        changed,
        headings_added,
        headings_removed,
        keyed,
        key_headings,
        rows,
    }
}

/// Compare two parsed AGS4 files into a [`RevisionDelta`]. Pure: the caller
/// resolves the dictionary (the edition KEY headings drive row matching) and
/// parses both files; this neither parses nor serialises. `max_rows_per_group`
/// caps how many per-row deltas each group carries (the `added`/`removed`/
/// `changed` counts stay the true totals); `None` keeps everything.
#[must_use]
pub fn diff_parsed(
    a: &ParsedFile,
    b: &ParsedFile,
    dict: &Dictionary,
    max_rows_per_group: Option<usize>,
) -> RevisionDelta {
    // Union of group codes: B's file order, then groups only in A.
    let mut codes: Vec<String> = b.group_order.clone();
    for c in &a.group_order {
        if !b.groups.contains_key(c) {
            codes.push(c.clone());
        }
    }

    let mut groups: Vec<GroupDelta> = Vec::new();
    let mut groups_added: Vec<String> = Vec::new();
    let mut groups_removed: Vec<String> = Vec::new();
    let (mut total_added, mut total_removed, mut total_changed) = (0usize, 0usize, 0usize);

    for code in &codes {
        match (a.groups.get(code), b.groups.get(code)) {
            (None, Some(_)) => groups_added.push(code.clone()),
            (Some(_), None) => groups_removed.push(code.clone()),
            (Some(ga), Some(gb)) => {
                let d = diff_group(code, ga, gb, dict, max_rows_per_group);
                total_added += d.added;
                total_removed += d.removed;
                total_changed += d.changed;
                if d.added + d.removed + d.changed > 0
                    || !d.headings_added.is_empty()
                    || !d.headings_removed.is_empty()
                {
                    groups.push(d);
                }
            }
            (None, None) => {}
        }
    }

    RevisionDelta {
        groups,
        groups_added,
        groups_removed,
        total_added,
        total_removed,
        total_changed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use laterite_ags4_parse::parse_str;
    use laterite_ags4_reference::dict::DictVersion;

    #[test]
    fn diff_group_is_key_aware_and_type_aware() {
        // Baseline: BH01..BH03. Revision: BH01 unchanged-but-reformatted
        // (523145.67 -> 523145.670), BH02 a real value change, BH03 removed,
        // BH04 added. Matched by the dictionary KEY heading LOCA_ID, NOT by
        // row order.
        let a = "\"GROUP\",\"LOCA\"\r\n\
\"HEADING\",\"LOCA_ID\",\"LOCA_NATE\"\r\n\
\"UNIT\",\"\",\"m\"\r\n\
\"TYPE\",\"ID\",\"2DP\"\r\n\
\"DATA\",\"BH01\",\"523145.67\"\r\n\
\"DATA\",\"BH02\",\"523200.00\"\r\n\
\"DATA\",\"BH03\",\"523300.00\"\r\n";
        let b = "\"GROUP\",\"LOCA\"\r\n\
\"HEADING\",\"LOCA_ID\",\"LOCA_NATE\"\r\n\
\"UNIT\",\"\",\"m\"\r\n\
\"TYPE\",\"ID\",\"2DP\"\r\n\
\"DATA\",\"BH02\",\"523200.50\"\r\n\
\"DATA\",\"BH01\",\"523145.670\"\r\n\
\"DATA\",\"BH04\",\"523400.00\"\r\n";
        let pa = parse_str(a).unwrap();
        let pb = parse_str(b).unwrap();
        let dict = Dictionary::bundled(DictVersion::V4_1_1);
        let d = diff_group("LOCA", &pa.groups["LOCA"], &pb.groups["LOCA"], &dict, None);

        assert!(d.keyed, "LOCA_ID is a dictionary KEY heading");
        assert_eq!(d.key_headings, vec!["LOCA_ID".to_string()]);
        assert_eq!(d.added, 1, "BH04 added");
        assert_eq!(d.removed, 1, "BH03 removed");
        assert_eq!(
            d.changed, 1,
            "only BH02 — BH01's 523145.67 -> 523145.670 is a 2DP no-op"
        );

        let changed: Vec<_> = d.rows.iter().filter(|r| r.kind == "changed").collect();
        assert_eq!(changed.len(), 1);
        assert_eq!(changed[0].key, vec!["BH02".to_string()]);
        assert_eq!(changed[0].cells.len(), 1);
        assert_eq!(changed[0].cells[0].heading, "LOCA_NATE");
        assert_eq!(changed[0].cells[0].a.as_deref(), Some("523200.00"));
        assert_eq!(changed[0].cells[0].b.as_deref(), Some("523200.50"));
    }

    #[test]
    fn diff_group_unkeyed_falls_back_to_whole_row() {
        // A custom group with no dictionary KEY headings: a changed row can't
        // be paired, so it shows as a remove + add (keyed = false).
        let a = "\"GROUP\",\"ZZZZ\"\r\n\
\"HEADING\",\"ZZZZ_A\",\"ZZZZ_B\"\r\n\
\"UNIT\",\"\",\"\"\r\n\
\"TYPE\",\"X\",\"X\"\r\n\
\"DATA\",\"p\",\"q\"\r\n";
        let b = "\"GROUP\",\"ZZZZ\"\r\n\
\"HEADING\",\"ZZZZ_A\",\"ZZZZ_B\"\r\n\
\"UNIT\",\"\",\"\"\r\n\
\"TYPE\",\"X\",\"X\"\r\n\
\"DATA\",\"p\",\"r\"\r\n";
        let pa = parse_str(a).unwrap();
        let pb = parse_str(b).unwrap();
        let dict = Dictionary::bundled(DictVersion::V4_1_1);
        let d = diff_group("ZZZZ", &pa.groups["ZZZZ"], &pb.groups["ZZZZ"], &dict, None);

        assert!(!d.keyed);
        assert_eq!(d.changed, 0);
        assert_eq!(d.added, 1);
        assert_eq!(d.removed, 1);
    }

    // 0c: each side of a matched row is typed by its OWN file, so a heading two
    // files typed differently is compared on each side's real type rather than
    // cross-contaminating through one shared type.
    #[test]
    fn changed_cells_type_each_side_independently() {
        // LOCA_XY: baseline types it ID ("1.0" → the string "1.0"), revision types
        // it 2DP ("1.00" → the number 1.0). One shared type (the old code) would
        // parse BOTH under 2DP and suppress the change; per-side typing surfaces it.
        let a = "\"GROUP\",\"LOCA\"\r\n\"HEADING\",\"LOCA_ID\",\"LOCA_XY\"\r\n\"UNIT\",\"\",\"\"\r\n\"TYPE\",\"ID\",\"ID\"\r\n\"DATA\",\"BH1\",\"1.0\"\r\n";
        let b = "\"GROUP\",\"LOCA\"\r\n\"HEADING\",\"LOCA_ID\",\"LOCA_XY\"\r\n\"UNIT\",\"\",\"\"\r\n\"TYPE\",\"ID\",\"2DP\"\r\n\"DATA\",\"BH1\",\"1.00\"\r\n";
        let pa = parse_str(a).unwrap();
        let pb = parse_str(b).unwrap();
        let dict = Dictionary::bundled(DictVersion::V4_1_1);
        let d = diff_group("LOCA", &pa.groups["LOCA"], &pb.groups["LOCA"], &dict, None);
        assert_eq!(
            d.changed, 1,
            "the ID→2DP retype of LOCA_XY is a real change"
        );
        let changed: Vec<_> = d.rows.iter().filter(|r| r.kind == "changed").collect();
        assert_eq!(changed[0].cells[0].heading, "LOCA_XY");
    }

    // 0b migration gate: before repointing diff's KEY derivation at the shared
    // `key_heading_names`, PROVE the union's KEY classification agrees with every
    // bundled edition's own `status.contains("KEY")` for the headings that edition
    // declares — a genuine old-vs-new comparison, not the function-vs-itself
    // tautology. If this ever fails, the switch is NOT behaviour-neutral.
    #[test]
    fn union_key_headings_agree_with_per_edition_status() {
        use laterite_ags4_reference::keychain::key_heading_names;
        use laterite_ags4_reference::union::registry;
        use std::collections::BTreeSet;

        let reg = registry();
        let mut mismatches: Vec<String> = Vec::new();
        for &ed in DictVersion::ALL {
            let d = Dictionary::bundled(ed);
            let mut codes: Vec<&str> = d.group_codes().collect();
            codes.sort_unstable();
            for code in codes {
                let union_keys: BTreeSet<&str> = reg
                    .get(code)
                    .map(|g| key_heading_names(g).into_iter().collect())
                    .unwrap_or_default();
                for &h in d.group_headings(code).iter() {
                    let old = d.heading(code, h).is_some_and(|e| e.status.contains("KEY"));
                    let new = union_keys.contains(h);
                    if old != new {
                        mismatches.push(format!(
                            "{}/{code}/{h}: edition KEY={old} union KEY={new}",
                            ed.as_str()
                        ));
                    }
                }
            }
        }
        assert!(
            mismatches.is_empty(),
            "union vs per-edition KEY divergence:\n{}",
            mismatches.join("\n")
        );
    }
}
