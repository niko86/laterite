//! Dictionary-aware rules: AGS4.1/4.2 Rules 7 and 9.
//!
//! CLEAN-ROOM. Implemented from the AGS4 spec (`reports/AGS 4_1.pdf` &
//! `reports/AGS 4_2.pdf` §4.1.1). python-ags4 (LGPL-3.0) was read only
//! to learn its interpretation (which facets it splits each rule into,
//! how it builds the effective dictionary) — facts about the AGS
//! standard, not copyrightable. No code/structure/wording was copied.
//!
//! Spec text (verbatim, AGS 4.2 §4.1.1, p.155 — AGS 4.1 prose
//! materially identical; the rule set is edition-stable, cf. O-6/O-7):
//!
//! * **Rule 7** — "The order of data FIELDs in each line within a
//!   GROUP is defined at the start of each GROUP in the HEADING row.
//!   HEADINGs shall be in the order described in the AGS FORMAT DATA
//!   DICTIONARY."
//! * **Rule 9** — "Data HEADING and GROUP names shall be taken from
//!   the AGS FORMAT DATA DICTIONARY. In cases where there is no
//!   suitable entry, a user-defined GROUP and/or HEADING may be used
//!   in accordance with Rule 18. Any user-defined HEADINGs shall be
//!   included at the end of the HEADING row after the standard
//!   HEADINGs in the order defined in the DICT group (see Rule 18a)."
//!
//! **Effective dictionary.** Rule 9 explicitly admits user-defined
//! headings declared in the file's own DICT group, and Rule 7's order
//! is "standard headings first, then user-defined in DICT order". So
//! both rules run against an *effective* dictionary = the bundled
//! standard dictionary, with the file's DICT-group rows appended
//! (first-occurrence wins, so a standard heading keeps its canonical
//! slot). This V4 phase only *consumes* the DICT group; *validating*
//! its structure/completeness is Rules 15/16/17/18 (V6).
//!
//! python-ags4 splits Rule 7 into a per-line duplicate-heading check
//! (`7_1`) and the order check (`7_2`). The duplicate check is *inferred*
//! — the prose only mandates dictionary order — but a HEADING row with
//! repeated names has no well-defined order or field identity, so we
//! keep it (attributed to Rule 7, matching python's count). See
//! OBSERVATIONS O-8/O-9/O-10.

use std::collections::{BTreeSet, HashMap};

use crate::dict::Dictionary;
use crate::findings::{Findings, Location, Severity, Target, add, add_at};
use crate::parse::ParsedFile;

const RULE_7: &str = "AGS Format Rule 7";
const RULE_9: &str = "AGS Format Rule 9";

pub fn check(parsed: &ParsedFile, dict: &Dictionary, found: &mut Findings) {
    // User-defined headings declared in the file's own DICT group,
    // per group, in file order. Consumed here, validated in V6.
    let file_dict = collect_file_dict(parsed);

    for code in &parsed.group_order {
        let g = &parsed.groups[code];
        // No HEADING row → nothing to order/resolve; its absence is
        // already a Rule 4 finding (V2).
        let Some(hl) = g.heading_line else { continue };

        rule_7_1(&g.headings, hl, found);

        // Effective order = standard order, then any DICT-group
        // headings for this group not already present (Rule 9's
        // "user-defined ... at the end after the standard HEADINGs").
        let mut order: Vec<&str> = dict.group_headings(code).to_vec();
        if let Some(extra) = file_dict.get(code.as_str()) {
            for h in extra {
                if !order.contains(&h.as_str()) {
                    order.push(h.as_str());
                }
            }
        }
        let known: BTreeSet<&str> = order.iter().copied().collect();

        // Rule 9 — every heading must be defined somewhere. `ci` is the
        // tag-stripped heading index; the UI resolves the raw field as
        // `field_index + 1` (field 0 is the HEADING tag).
        // `ci` is a column index within one AGS4 group's heading row —
        // bounded by that group's heading count (dictionary-bounded, a few
        // dozen at most), nowhere near u32::MAX.
        #[allow(clippy::cast_possible_truncation)]
        for (ci, h) in g.headings.iter().enumerate() {
            if !known.contains(h.as_str()) {
                add_at(
                    found,
                    RULE_9,
                    Some(hl),
                    code,
                    format!(
                        "Heading {h:?} is not in the standard dictionary \
                         or the file's DICT group."
                    ),
                    Location {
                        target: Target::Heading,
                        field_index: Some(ci as u32),
                        heading: Some(h.clone()),
                        ..Default::default()
                    },
                    Severity::Error,
                );
            }
        }

        rule_7_2(&g.headings, &order, &known, hl, code, found);
    }
}

/// Rule 7 (duplicate-heading facet) — a HEADING row must not repeat a
/// field name. Independent of the dictionary; attributed to Rule 7
/// with an empty group, matching python-ags4's per-line attribution.
fn rule_7_1(headings: &[String], line: u32, found: &mut Findings) {
    let mut seen = BTreeSet::new();
    // The first heading that repeats names the offending field for the
    // UI; the set-level finding (`field_index: None`) still spans the
    // whole HEADING row.
    let dup = headings.iter().find(|h| !seen.insert(h.as_str()));
    if let Some(dup) = dup {
        add_at(
            found,
            RULE_7,
            Some(line),
            "",
            "HEADING row contains duplicate field names.",
            Location {
                target: Target::Heading,
                heading: Some(dup.clone()),
                ..Default::default()
            },
            Severity::Error,
        );
    }
}

/// Rule 7 (order facet) — the headings actually present must appear in
/// the same relative order as the effective dictionary defines them.
/// If any heading is unknown the order is unverifiable (it's a Rule 9
/// finding); we say so rather than guess.
fn rule_7_2(
    headings: &[String],
    order: &[&str],
    known: &BTreeSet<&str>,
    line: u32,
    group: &str,
    found: &mut Findings,
) {
    if !headings.iter().all(|h| known.contains(h.as_str())) {
        add(
            found,
            RULE_7,
            Some(line),
            group,
            "Heading order cannot be checked: one or more headings are \
             not in the standard dictionary or DICT group (see Rule 9).",
        );
        return;
    }

    // Expected = the dictionary order filtered to the headings used.
    let used: BTreeSet<&str> = headings.iter().map(String::as_str).collect();
    let expected: Vec<&str> = order.iter().copied().filter(|h| used.contains(h)).collect();

    for (i, h) in headings.iter().enumerate() {
        match expected.get(i) {
            Some(e) if *e == h.as_str() => {}
            Some(_) => {
                let rest = expected[i..].join("|");
                add(
                    found,
                    RULE_7,
                    Some(line),
                    group,
                    format!(
                        "Headings out of order from {h:?}. Expected \
                         dictionary order from here: {rest}"
                    ),
                );
                return;
            }
            // `headings` longer than the de-duped expected list — only
            // possible with a duplicate heading, which rule_7_1 already
            // reported. python-ags4 indexes unconditionally here and
            // raises IndexError on this input (O-8); we stop cleanly.
            None => return,
        }
    }
}

/// Pull the file's own DICT group into `group -> [user heading, …]`
/// (file order). Columns are resolved by name from the DICT HEADING
/// row, so column reordering doesn't break it. Mirrors python-ags4's
/// `combine_DICT_tables` (standard dict first, file DICT appended);
/// here we only read it — DICT *validation* is V6.
pub(crate) fn collect_file_dict(parsed: &ParsedFile) -> HashMap<String, Vec<String>> {
    let mut out: HashMap<String, Vec<String>> = HashMap::new();
    let Some(dictg) = parsed.groups.get("DICT") else {
        return out;
    };
    let col = |name: &str| dictg.headings.iter().position(|h| h == name);
    let (Some(gi), Some(hi)) = (col("DICT_GRP"), col("DICT_HDNG")) else {
        return out; // malformed DICT — Rule 18 (V6) reports that
    };
    for row in &dictg.rows {
        let grp = row.values.get(gi).map_or("", String::as_str);
        let hdng = row.values.get(hi).map_or("", String::as_str);
        if grp.is_empty() || hdng.is_empty() {
            continue; // GROUP-type rows / blanks contribute no heading
        }
        let v = out.entry(grp.to_string()).or_default();
        if !v.iter().any(|x| x == hdng) {
            v.push(hdng.to_string());
        }
    }
    out
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

    // PROJ_ID before PROJ_NAME = dictionary order; both standard.
    const CLEAN: &str = "\"GROUP\",\"PROJ\"\r\n\
                          \"HEADING\",\"PROJ_ID\",\"PROJ_NAME\"\r\n\
                          \"UNIT\",\"\",\"\"\r\n\"TYPE\",\"ID\",\"X\"\r\n\
                          \"DATA\",\"P1\",\"x\"\r\n";

    #[test]
    fn clean_standard_headings_in_order_have_no_findings() {
        assert!(run(CLEAN).is_empty());
    }

    #[test]
    fn rule_7_flags_headings_out_of_dictionary_order() {
        // PROJ_NAME before PROJ_ID — reversed vs the dictionary.
        let src = "\"GROUP\",\"PROJ\"\r\n\
                   \"HEADING\",\"PROJ_NAME\",\"PROJ_ID\"\r\n\
                   \"UNIT\",\"\",\"\"\r\n\"TYPE\",\"X\",\"ID\"\r\n\
                   \"DATA\",\"x\",\"P1\"\r\n";
        let f = run(src);
        let r7 = f.get(RULE_7).expect("Rule 7");
        assert!(
            r7.iter().any(|x| x.line == Some(2) && x.group == "PROJ"),
            "expected an order finding on the HEADING line: {r7:?}"
        );
        // No Rule 9 — both names are standard.
        assert!(!f.contains_key(RULE_9), "{f:?}");
    }

    #[test]
    fn rule_7_flags_duplicate_heading() {
        let src = "\"GROUP\",\"PROJ\"\r\n\
                   \"HEADING\",\"PROJ_ID\",\"PROJ_ID\"\r\n\
                   \"UNIT\",\"\",\"\"\r\n\"TYPE\",\"ID\",\"ID\"\r\n\
                   \"DATA\",\"P1\",\"P1\"\r\n";
        let r7 = run(src);
        let v = r7.get(RULE_7).expect("Rule 7");
        assert!(v.iter().any(|x| x.desc.contains("duplicate")), "{v:?}");
    }

    #[test]
    fn rule_9_flags_unknown_heading_and_rule_7_defers() {
        let src = "\"GROUP\",\"PROJ\"\r\n\
                   \"HEADING\",\"PROJ_ID\",\"PROJ_ZZZZ\"\r\n\
                   \"UNIT\",\"\",\"\"\r\n\"TYPE\",\"ID\",\"X\"\r\n\
                   \"DATA\",\"P1\",\"q\"\r\n";
        let f = run(src);
        let r9 = f.get(RULE_9).expect("Rule 9");
        assert!(
            r9.iter()
                .any(|x| x.desc.contains("PROJ_ZZZZ") && x.line == Some(2)),
            "{r9:?}"
        );
        // Order can't be checked when a heading is unknown.
        let r7 = f.get(RULE_7).expect("Rule 7 defer note");
        assert!(
            r7.iter().any(|x| x.desc.contains("cannot be checked")),
            "{r7:?}"
        );
    }

    #[test]
    fn file_dict_user_heading_satisfies_rules_7_and_9() {
        // PROJ uses a user heading PROJ_XX declared in the file's DICT
        // group, placed after the standard PROJ_ID — legal per Rule 9.
        let src = "\"GROUP\",\"PROJ\"\r\n\
                   \"HEADING\",\"PROJ_ID\",\"PROJ_XX\"\r\n\
                   \"UNIT\",\"\",\"\"\r\n\"TYPE\",\"ID\",\"X\"\r\n\
                   \"DATA\",\"P1\",\"v\"\r\n\r\n\
                   \"GROUP\",\"DICT\"\r\n\
                   \"HEADING\",\"DICT_TYPE\",\"DICT_GRP\",\"DICT_HDNG\"\r\n\
                   \"UNIT\",\"\",\"\",\"\"\r\n\"TYPE\",\"X\",\"X\",\"X\"\r\n\
                   \"DATA\",\"HEADING\",\"PROJ\",\"PROJ_XX\"\r\n";
        let f = run(src);
        // PROJ_XX is now known → no Rule 9 for it, order holds.
        let r9 = f
            .get(RULE_9)
            .map(|v| v.iter().filter(|x| x.desc.contains("PROJ_XX")).count());
        assert_eq!(r9.unwrap_or(0), 0, "PROJ_XX wrongly flagged: {f:?}");
        assert!(!f.contains_key(RULE_7), "unexpected Rule 7: {f:?}");
    }
}
