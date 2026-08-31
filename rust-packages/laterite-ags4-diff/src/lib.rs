//! Revision diff — a KEY-aware, type-aware comparison of two AGS4 files.
//!
//! A KEY-aware, type-aware comparison of two AGS4 files (a baseline `a` and a
//! revision `b`). Rows within a group are matched by the group's *dictionary*
//! KEY headings, not by line order — so a re-sorted or re-numbered file still
//! pairs the same boreholes/samples. Matched cells are compared through
//! `laterite_ags4_types::parse_value`, so a formatting-only change ("1.0" → "1.00",
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
use laterite_ags4_types::parse_value;
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

/// The `diff --json` document: [`RevisionDelta`] as pretty JSON (2-space,
/// field order = declaration order, non-ASCII raw, no trailing newline).
///
/// This is the launcher contract's ONE machine render for the diff verb
/// (`ags-wiki/design/dec-launcher-contract.md`): the `lat` binary, the wheel's
/// `_cli` and the npx launcher all print this string verbatim, so `--json` is
/// byte-exact across them by construction rather than by three hand-kept
/// serialisers agreeing (#542 — Python's round trip escaped non-ASCII).
pub fn delta_json(delta: &RevisionDelta) -> String {
    serde_json::to_string_pretty(delta).unwrap_or_default()
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
    buf: &str,
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
                    .map_or_else(String::new, |s| s.slice(buf).to_string())
            })
            .collect()
    } else {
        row.values
            .iter()
            .map(|s| s.slice(buf).to_string())
            .collect()
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
    buf_a: &str,
    idx_a: &std::collections::HashMap<&str, usize>,
    types_a: &[String],
    row_b: &DataRow,
    buf_b: &str,
    idx_b: &std::collections::HashMap<&str, usize>,
    types_b: &[String],
    dict: &Dictionary,
) -> Vec<CellDelta> {
    let mut out = Vec::new();
    for h in common {
        let a = idx_a
            .get(h.as_str())
            .and_then(|&i| row_a.values.get(i))
            .map(|s| s.slice(buf_a));
        let b = idx_b
            .get(h.as_str())
            .and_then(|&i| row_b.values.get(i))
            .map(|s| s.slice(buf_b));
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
            .entry(row_key(row, gb.text(), &idx_b, &key_headings, keyed))
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
        let k = row_key(row_a, ga.text(), &idx_a, &key_headings, keyed);
        if let Some(bi) = b_by_key
            .get_mut(&k)
            .and_then(std::collections::VecDeque::pop_front)
        {
            matched_b[bi] = true;
            let row_b = &gb.rows[bi];
            let cells = changed_cells(
                code,
                &common,
                row_a,
                ga.text(),
                &idx_a,
                &ga.types,
                row_b,
                gb.text(),
                &idx_b,
                &gb.types,
                dict,
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
                    key: row_key(row_b, gb.text(), &idx_b, &key_headings, keyed),
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

    /// An identical PROJ on both sides — present so the file-level tests have a
    /// group that must NOT be reported as different.
    fn proj() -> &'static str {
        "\"GROUP\",\"PROJ\"\r\n\
\"HEADING\",\"PROJ_ID\"\r\n\
\"UNIT\",\"\"\r\n\
\"TYPE\",\"ID\"\r\n\
\"DATA\",\"P1\"\r\n"
    }

    #[test]
    fn delta_json_is_the_wire_render_and_keeps_non_ascii_raw() {
        // The properties every launcher leans on when printing this string
        // verbatim (#542): declaration-order fields, 2-space pretty layout,
        // non-ASCII bytes untouched, no trailing newline.
        let a = "\"GROUP\",\"LOCA\"\r\n\
\"HEADING\",\"LOCA_ID\",\"LOCA_REM\"\r\n\
\"UNIT\",\"\",\"\"\r\n\
\"TYPE\",\"ID\",\"X\"\r\n\
\"DATA\",\"BH01\",\"sec — argile\"\r\n";
        let b = a.replace("sec — argile", "humide — boué");
        let pa = parse_str(a).unwrap();
        let pb = parse_str(&b).unwrap();
        let dict = Dictionary::bundled(DictVersion::V4_1_1);
        let json = delta_json(&diff_parsed(&pa, &pb, &dict, None));
        assert!(json.starts_with("{\n  \"groups\""));
        assert!(!json.ends_with('\n'));
        assert!(json.contains("humide — boué"), "non-ASCII must stay raw");
        assert!(!json.contains("\\u00"), "no ensure_ascii-style escapes");
        let back: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(back["total_changed"], 1);
    }

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

    /// One LOCA group, `n` rows, ids `BH{i:02}` and eastings that differ from
    /// `base` by row — enough to make every row *changed* between two calls.
    fn loca(n: usize, base: f64) -> String {
        let mut s = String::from(
            "\"GROUP\",\"LOCA\"\r\n\
\"HEADING\",\"LOCA_ID\",\"LOCA_NATE\"\r\n\
\"UNIT\",\"\",\"m\"\r\n\
\"TYPE\",\"ID\",\"2DP\"\r\n",
        );
        for i in 1..=n {
            use std::fmt::Write as _;
            let _ = writeln!(s, "\"DATA\",\"BH{i:02}\",\"{:.2}\"\r", base + i as f64);
        }
        s
    }

    /// `diff_parsed` is the crate's ONLY public function — every surface
    /// (`lat diff`, Python, Node, wasm) enters here — and until this test it had
    /// no coverage at all: the suite exercised the private helpers underneath it
    /// and nothing walked the file-level union, the totals, or the decision to
    /// include a group. A mutation sweep found 18 survivors in this one function
    /// (laterite#127).
    #[test]
    fn diff_parsed_unions_groups_and_totals_across_the_file() {
        // A has LOCA + ABBR; B has LOCA + SAMP. So SAMP is added, ABBR removed,
        // LOCA changed — one of each arm, plus a group present and IDENTICAL on
        // both sides (PROJ) that must NOT be reported.
        let a = format!(
            "{}{}{}",
            proj(),
            loca(2, 100.0),
            "\"GROUP\",\"ABBR\"\r\n\
\"HEADING\",\"ABBR_HDNG\",\"ABBR_CODE\"\r\n\
\"UNIT\",\"\",\"\"\r\n\
\"TYPE\",\"X\",\"X\"\r\n\
\"DATA\",\"LOCA_TYPE\",\"TP\"\r\n"
        );
        let b = format!(
            "{}{}{}",
            proj(),
            loca(2, 200.0),
            "\"GROUP\",\"SAMP\"\r\n\
\"HEADING\",\"SAMP_ID\"\r\n\
\"UNIT\",\"\"\r\n\
\"TYPE\",\"ID\"\r\n\
\"DATA\",\"S1\"\r\n"
        );
        let pa = parse_str(&a).unwrap();
        let pb = parse_str(&b).unwrap();
        let dict = Dictionary::bundled(DictVersion::V4_1_1);
        let d = diff_parsed(&pa, &pb, &dict, None);

        assert_eq!(d.groups_added, vec!["SAMP".to_string()], "SAMP is B-only");
        assert_eq!(
            d.groups_removed,
            vec!["ABBR".to_string()],
            "ABBR is A-only — reached through the `!b.groups.contains_key` arm"
        );

        // Totals are the SUM across groups, not any single group's count.
        assert_eq!(d.total_changed, 2, "both LOCA rows changed easting");
        assert_eq!(d.total_added, 0);
        assert_eq!(d.total_removed, 0);

        // An unchanged group is omitted; a changed one is kept.
        let codes: Vec<&str> = d.groups.iter().map(|g| g.code.as_str()).collect();
        assert_eq!(
            codes,
            vec!["LOCA"],
            "only groups with a difference are included — PROJ is identical"
        );
    }

    /// Totals must ACCUMULATE across groups, and the include-predicate must be a
    /// sum rather than any other combination of the three counts.
    ///
    /// Deliberately built so that zero is never the witness: the first sweep left
    /// six survivors here precisely because the earlier test had
    /// `total_added == total_removed == 0`, and 0 is a fixed point of both `-=`
    /// and `*=`. A count only proves an operator when the count is non-zero and
    /// the operands differ.
    ///
    /// - `LOCA` gains one row and loses one (added 1, removed 1) — under
    ///   `added - removed` the group nets to zero and DISAPPEARS from the report.
    /// - `ZZZZ` only gains rows (added 2, removed 0) — under `added * removed`
    ///   it nets to zero and disappears instead.
    ///
    /// One of the two vanishes under each wrong operator, so asserting both are
    /// present pins the arithmetic from both sides.
    #[test]
    fn totals_accumulate_and_the_include_predicate_is_a_sum() {
        let a = "\"GROUP\",\"LOCA\"\r\n\
\"HEADING\",\"LOCA_ID\"\r\n\
\"UNIT\",\"\"\r\n\
\"TYPE\",\"ID\"\r\n\
\"DATA\",\"BH01\"\r\n\
\"DATA\",\"BH02\"\r\n\
\"GROUP\",\"ZZZZ\"\r\n\
\"HEADING\",\"ZZZZ_A\"\r\n\
\"UNIT\",\"\"\r\n\
\"TYPE\",\"X\"\r\n\
\"DATA\",\"keep\"\r\n";
        let b = "\"GROUP\",\"LOCA\"\r\n\
\"HEADING\",\"LOCA_ID\"\r\n\
\"UNIT\",\"\"\r\n\
\"TYPE\",\"ID\"\r\n\
\"DATA\",\"BH02\"\r\n\
\"DATA\",\"BH03\"\r\n\
\"GROUP\",\"ZZZZ\"\r\n\
\"HEADING\",\"ZZZZ_A\"\r\n\
\"UNIT\",\"\"\r\n\
\"TYPE\",\"X\"\r\n\
\"DATA\",\"keep\"\r\n\
\"DATA\",\"new1\"\r\n\
\"DATA\",\"new2\"\r\n";
        let pa = parse_str(a).unwrap();
        let pb = parse_str(b).unwrap();
        let dict = Dictionary::bundled(DictVersion::V4_1_1);
        let d = diff_parsed(&pa, &pb, &dict, None);

        // 1 (BH03) + 2 (new1, new2); a `*=` accumulator would leave this 0.
        assert_eq!(d.total_added, 3, "adds must SUM across groups");
        assert_eq!(d.total_removed, 1, "BH01, from LOCA only");
        assert_eq!(d.total_changed, 0);

        let codes: Vec<&str> = d.groups.iter().map(|g| g.code.as_str()).collect();
        assert!(
            codes.contains(&"LOCA"),
            "LOCA nets 1 added + 1 removed and must still be reported: {codes:?}"
        );
        assert!(
            codes.contains(&"ZZZZ"),
            "ZZZZ nets 2 added + 0 removed and must still be reported: {codes:?}"
        );
    }

    /// A group whose ROWS are identical but whose HEADINGS differ must still be
    /// reported. Guards the second arm of the include-predicate: counting rows
    /// alone would drop a schema-only change entirely.
    #[test]
    fn diff_parsed_includes_a_group_changed_only_by_its_headings() {
        let a = format!("{}{}", proj(), loca(1, 100.0));
        let b = format!(
            "{}{}",
            proj(),
            "\"GROUP\",\"LOCA\"\r\n\
\"HEADING\",\"LOCA_ID\",\"LOCA_NATE\",\"LOCA_GL\"\r\n\
\"UNIT\",\"\",\"m\",\"m\"\r\n\
\"TYPE\",\"ID\",\"2DP\",\"2DP\"\r\n\
\"DATA\",\"BH01\",\"101.00\",\"\"\r\n"
        );
        let pa = parse_str(&a).unwrap();
        let pb = parse_str(&b).unwrap();
        let dict = Dictionary::bundled(DictVersion::V4_1_1);
        let d = diff_parsed(&pa, &pb, &dict, None);

        assert_eq!(d.total_added + d.total_removed + d.total_changed, 0);
        let loca_delta = d
            .groups
            .iter()
            .find(|g| g.code == "LOCA")
            .expect("a heading-only change must still surface the group");
        assert_eq!(loca_delta.headings_added, vec!["LOCA_GL".to_string()]);
        assert!(loca_delta.headings_removed.is_empty());
    }

    /// `headings_removed` is the mirror arm, and inverting either filter is
    /// silent without an assertion on both.
    #[test]
    fn heading_added_and_removed_are_not_interchangeable() {
        let a = "\"GROUP\",\"LOCA\"\r\n\
\"HEADING\",\"LOCA_ID\",\"LOCA_GONE\"\r\n\
\"UNIT\",\"\",\"\"\r\n\
\"TYPE\",\"ID\",\"X\"\r\n\
\"DATA\",\"BH01\",\"x\"\r\n";
        let b = "\"GROUP\",\"LOCA\"\r\n\
\"HEADING\",\"LOCA_ID\",\"LOCA_NEW\"\r\n\
\"UNIT\",\"\",\"\"\r\n\
\"TYPE\",\"ID\",\"X\"\r\n\
\"DATA\",\"BH01\",\"x\"\r\n";
        let pa = parse_str(a).unwrap();
        let pb = parse_str(b).unwrap();
        let dict = Dictionary::bundled(DictVersion::V4_1_1);
        let d = diff_group("LOCA", &pa.groups["LOCA"], &pb.groups["LOCA"], &dict, None);

        assert_eq!(
            d.headings_added,
            vec!["LOCA_NEW".to_string()],
            "added = in B, not in A"
        );
        assert_eq!(
            d.headings_removed,
            vec!["LOCA_GONE".to_string()],
            "removed = in A, not in B"
        );
    }

    /// A KEY heading present on only ONE side cannot index the other, so it must
    /// not be used as a key. Requiring both sides is an `&&`; an `||` here would
    /// build a key from a heading half the rows do not have.
    #[test]
    fn a_key_heading_missing_from_one_side_is_not_used_as_a_key() {
        // SAMP's KEY tuple includes LOCA_ID + SAMP_TOP + SAMP_REF + SAMP_TYPE;
        // B drops SAMP_TOP, so the surviving key set must exclude it.
        let a = "\"GROUP\",\"SAMP\"\r\n\
\"HEADING\",\"LOCA_ID\",\"SAMP_TOP\",\"SAMP_REF\"\r\n\
\"UNIT\",\"\",\"m\",\"\"\r\n\
\"TYPE\",\"ID\",\"2DP\",\"X\"\r\n\
\"DATA\",\"BH01\",\"1.00\",\"R1\"\r\n";
        let b = "\"GROUP\",\"SAMP\"\r\n\
\"HEADING\",\"LOCA_ID\",\"SAMP_REF\"\r\n\
\"UNIT\",\"\",\"\"\r\n\
\"TYPE\",\"ID\",\"X\"\r\n\
\"DATA\",\"BH01\",\"R1\"\r\n";
        let pa = parse_str(a).unwrap();
        let pb = parse_str(b).unwrap();
        let dict = Dictionary::bundled(DictVersion::V4_1_1);
        let d = diff_group("SAMP", &pa.groups["SAMP"], &pb.groups["SAMP"], &dict, None);

        assert!(
            !d.key_headings.iter().any(|h| h == "SAMP_TOP"),
            "SAMP_TOP is absent from B and cannot key it: {:?}",
            d.key_headings
        );
        for h in &d.key_headings {
            assert!(
                pa.groups["SAMP"].headings.contains(h) && pb.groups["SAMP"].headings.contains(h),
                "every key heading must exist on BOTH sides, {h} does not"
            );
        }
    }

    /// `max_rows_per_group` caps the SERIALIZED rows only — the counts stay true
    /// totals. Both halves matter: a cap that also truncated the counts would
    /// under-report a diff, and the doc comment promises it does not.
    #[test]
    fn the_row_cap_limits_serialized_rows_but_not_the_counts() {
        let pa = parse_str(&loca(5, 100.0)).unwrap();
        let pb = parse_str(&loca(5, 200.0)).unwrap();
        let dict = Dictionary::bundled(DictVersion::V4_1_1);

        let uncapped = diff_group("LOCA", &pa.groups["LOCA"], &pb.groups["LOCA"], &dict, None);
        assert_eq!(uncapped.changed, 5);
        assert_eq!(uncapped.rows.len(), 5);

        let capped = diff_group(
            "LOCA",
            &pa.groups["LOCA"],
            &pb.groups["LOCA"],
            &dict,
            Some(2),
        );
        assert_eq!(
            capped.changed, 5,
            "the cap must not touch the count — it is the true total"
        );
        assert_eq!(
            capped.rows.len(),
            2,
            "exactly `cap` rows are serialized, not cap-1 and not cap+1"
        );
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

// The README's example is a doctest, not a second copy of one. `cfg(doctest)`
// means this module exists only while rustdoc collects doctests: it is absent
// from a normal build and from the rendered docs.rs page, so the crate's own
// `//!` docs are untouched and nothing is duplicated. The README is the single
// source, and `cargo test --workspace` already compiles it.
//
// The example is written out in full — no rustdoc `# ` hidden lines. A README is
// also read as plain Markdown on crates.io, where `# let x = …` renders as an
// <h1>. Visible boilerplate is the price of a page that is checked AND readable.
#[cfg(doctest)]
#[doc = include_str!("../README.md")]
mod readme_doctests {}
