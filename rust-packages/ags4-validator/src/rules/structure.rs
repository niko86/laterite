//! Group-structure rules: AGS4.1/4.2 Rules 2, 2a, 2b, 4.
//!
//! CLEAN-ROOM. Implemented from the AGS4 spec (`reports/AGS 4_1.pdf` &
//! `reports/AGS 4_2.pdf` §4.1.1). python-ags4 (LGPL-3.0) was read only
//! to learn the de-facto interpretation of the spec's wording (facts
//! about the AGS standard, not copyrightable); no code/structure was
//! copied. python-ags4 spreads these across `rule_2` (pandas table
//! scan), `rule_2a`/`rule_4_1` (per raw line) and `rule_2b`/`rule_4_2`
//! (table scan); we operate uniformly over the already-parsed
//! `ParsedFile`, which is simpler and single-pass.
//!
//! Spec text (verbatim, for the implementer):
//!
//! * **Rule 2** — "Each data file shall contain one or more data
//!   GROUPs. Each data GROUP shall comprise a number of GROUP HEADER
//!   rows and must have one or more DATA rows." → every group needs
//!   ≥ 1 DATA row. (≥ 1 GROUP in the file is enforced at parse time:
//!   a fileless of GROUPs is `ValidatorError::NotAgs4`.)
//! * **Rule 2a** — "Each row is located on a separate line, delimited
//!   by … carriage return (13) and … line feed (10)." → every line
//!   CR+LF terminated.
//! * **Rule 2b** — "As a minimum, the GROUP HEADER rows comprise
//!   GROUP, HEADING, UNIT and TYPE rows presented in that order." →
//!   UNIT present + immediately below HEADING; TYPE present +
//!   immediately below UNIT. (HEADING-missing is attributed to Rule 4,
//!   matching python-ags4's attribution so per-rule counts line up.)
//! * **Rule 4** — "The GROUP row contains only one DATA item, the
//!   GROUP name… All other rows in the GROUP have a number of DATA
//!   items defined by the HEADING row." → GROUP row has exactly the
//!   descriptor + name; UNIT/TYPE/DATA field counts equal the HEADING
//!   field count.
//!
//! Attribution choice: python-ags4 reports an unquoted DATA field
//! under Rule 4 (field-count mismatch), not Rule 5 — see OBSERVATIONS
//! O-3. We keep Rule 4's field-count check faithful so that path
//! agrees; the Rule 5 strict-quote check (V1) is additive, not a
//! reattribution.

use crate::findings::{Findings, add};
use crate::parse::{ParsedFile, split_ags_line};

const RULE_2: &str = "AGS Format Rule 2";
const RULE_2A: &str = "AGS Format Rule 2a";
const RULE_2B: &str = "AGS Format Rule 2b";
const RULE_4: &str = "AGS Format Rule 4";

pub fn check(parsed: &ParsedFile, found: &mut Findings) {
    rule_2a(parsed, found);
    for code in &parsed.group_order {
        let g = &parsed.groups[code];
        rule_2(g, found);
        rule_2b(g, found);
        rule_4(parsed, g, found);
    }
}

/// Rule 2a — CR+LF line termination, every line.
fn rule_2a(parsed: &ParsedFile, found: &mut Findings) {
    for rl in &parsed.raw_lines {
        if !rl.had_crlf {
            add(
                found,
                RULE_2A,
                Some(rl.number),
                "",
                "Line is not terminated by a carriage return + line feed (CR LF).",
            );
        }
    }
}

/// Rule 2 — each group must have ≥ 1 DATA row.
fn rule_2(g: &crate::parse::ParsedGroup, found: &mut Findings) {
    if g.rows.is_empty() {
        add(
            found,
            RULE_2,
            Some(g.group_line),
            &g.code,
            "Group has no DATA rows (a group must contain one or more).",
        );
    }
}

/// Rule 2b — UNIT/TYPE present and contiguous, in order, right after
/// HEADING. (Missing HEADING is a Rule 4 finding, not 2b — keeps
/// per-rule attribution aligned with python-ags4.)
fn rule_2b(g: &crate::parse::ParsedGroup, found: &mut Findings) {
    let at = g.group_line;

    match g.unit_line {
        None => add(
            found,
            RULE_2B,
            Some(at),
            &g.code,
            "UNIT row missing from group.",
        ),
        Some(ul) => {
            // UNIT must sit immediately below HEADING. If HEADING is
            // absent, Rule 4 reports that; we only judge placement
            // when we have a HEADING to anchor to.
            if let Some(hl) = g.heading_line {
                if ul != hl + 1 {
                    add(
                        found,
                        RULE_2B,
                        Some(ul),
                        &g.code,
                        "UNIT row is misplaced — it must be immediately below the HEADING row.",
                    );
                }
            }
        }
    }

    match g.type_line {
        None => add(
            found,
            RULE_2B,
            Some(at),
            &g.code,
            "TYPE row missing from group.",
        ),
        Some(tl) => {
            if let Some(ul) = g.unit_line {
                if tl != ul + 1 {
                    add(
                        found,
                        RULE_2B,
                        Some(tl),
                        &g.code,
                        "TYPE row is misplaced — it must be immediately below the UNIT row.",
                    );
                }
            }
        }
    }
}

/// Rule 4 — GROUP-row arity + UNIT/TYPE/DATA field-count == HEADING.
fn rule_4(parsed: &ParsedFile, g: &crate::parse::ParsedGroup, found: &mut Findings) {
    // 4.1: the GROUP row carries only the descriptor + the group name.
    // Re-split the raw GROUP line (raw_lines is line-ordered, 1-indexed
    // contiguous, so index = line-1).
    if let Some(rl) = parsed.raw_lines.get((g.group_line - 1) as usize) {
        let fields = split_ags_line(&rl.text);
        if fields.len() > 2 {
            add(
                found,
                RULE_4,
                Some(g.group_line),
                &g.code,
                "GROUP row has more than one data field (only the group name is allowed).",
            );
        } else if fields.len() < 2 {
            add(
                found,
                RULE_4,
                Some(g.group_line),
                &g.code,
                "GROUP row is malformed (missing the group name).",
            );
        }
    }

    // 4.2: every non-GROUP header/data row's field count must equal the
    // HEADING row's. Missing HEADING is reported once per group.
    if g.heading_line.is_none() {
        add(
            found,
            RULE_4,
            None,
            &g.code,
            "HEADING row missing from group.",
        );
        return; // nothing to compare counts against
    }
    let want = g.headings.len();

    if g.unit_line.is_some() && g.units.len() != want {
        add(
            found,
            RULE_4,
            g.unit_line,
            &g.code,
            "UNIT row field count does not match the HEADING row.",
        );
    }
    if g.type_line.is_some() && g.types.len() != want {
        add(
            found,
            RULE_4,
            g.type_line,
            &g.code,
            "TYPE row field count does not match the HEADING row.",
        );
    }
    for row in &g.rows {
        if row.values.len() != want {
            add(
                found,
                RULE_4,
                Some(row.line),
                &g.code,
                "DATA row field count does not match the HEADING row.",
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::parse_str;

    fn run(src: &str) -> Findings {
        let pf = parse_str(src).expect("fixture parses");
        let mut f = Findings::new();
        check(&pf, &mut f);
        f
    }

    const CLEAN: &str = "\"GROUP\",\"PROJ\"\r\n\"HEADING\",\"PROJ_ID\",\"PROJ_NAME\"\r\n\
                          \"UNIT\",\"\",\"\"\r\n\"TYPE\",\"ID\",\"X\"\r\n\
                          \"DATA\",\"P1\",\"x\"\r\n";

    #[test]
    fn clean_group_has_no_structure_findings() {
        let f = run(CLEAN);
        assert!(f.is_empty(), "unexpected: {f:?}");
    }

    #[test]
    fn rule_2_flags_group_with_no_data() {
        // PROJ ok; add a header-only LOCA group.
        let src = format!(
            "{CLEAN}\r\n\"GROUP\",\"LOCA\"\r\n\"HEADING\",\"LOCA_ID\"\r\n\
             \"UNIT\",\"\"\r\n\"TYPE\",\"ID\"\r\n"
        );
        let f = run(&src);
        let r2 = f.get(RULE_2).expect("Rule 2");
        assert_eq!(r2.len(), 1);
        assert_eq!(r2[0].group, "LOCA");
    }

    #[test]
    fn rule_2a_flags_lf_only_line() {
        // HEADING row is LF-only.
        let src = "\"GROUP\",\"PROJ\"\r\n\"HEADING\",\"PROJ_ID\"\n\
                   \"UNIT\",\"\"\r\n\"TYPE\",\"ID\"\r\n\"DATA\",\"P1\"\r\n";
        let f = run(src);
        let r2a = f.get(RULE_2A).expect("Rule 2a");
        assert!(r2a.iter().any(|x| x.line == Some(2)), "{r2a:?}");
    }

    #[test]
    fn rule_2b_flags_missing_unit() {
        let src = "\"GROUP\",\"PROJ\"\r\n\"HEADING\",\"PROJ_ID\"\r\n\
                   \"TYPE\",\"ID\"\r\n\"DATA\",\"P1\"\r\n";
        let f = run(src);
        let r2b = f.get(RULE_2B).expect("Rule 2b");
        assert!(
            r2b.iter().any(|x| x.desc.contains("UNIT row missing")),
            "{r2b:?}"
        );
    }

    #[test]
    fn rule_2b_flags_type_before_unit() {
        // TYPE then UNIT — TYPE not immediately below UNIT.
        let src = "\"GROUP\",\"PROJ\"\r\n\"HEADING\",\"PROJ_ID\"\r\n\
                   \"TYPE\",\"ID\"\r\n\"UNIT\",\"\"\r\n\"DATA\",\"P1\"\r\n";
        let f = run(src);
        let r2b = f.get(RULE_2B).expect("Rule 2b");
        assert!(
            r2b.iter().any(|x| x.desc.contains("misplaced")),
            "expected a misplacement finding: {r2b:?}"
        );
    }

    #[test]
    fn rule_4_flags_fat_group_row() {
        let src = "\"GROUP\",\"PROJ\",\"EXTRA\"\r\n\"HEADING\",\"PROJ_ID\"\r\n\
                   \"UNIT\",\"\"\r\n\"TYPE\",\"ID\"\r\n\"DATA\",\"P1\"\r\n";
        let f = run(src);
        let r4 = f.get(RULE_4).expect("Rule 4");
        assert!(r4.iter().any(|x| x.line == Some(1)), "{r4:?}");
    }

    #[test]
    fn rule_4_flags_data_field_count_mismatch() {
        // HEADING has 2 fields; DATA row has 3.
        let src = "\"GROUP\",\"PROJ\"\r\n\"HEADING\",\"PROJ_ID\",\"PROJ_NAME\"\r\n\
                   \"UNIT\",\"\",\"\"\r\n\"TYPE\",\"ID\",\"X\"\r\n\
                   \"DATA\",\"P1\",\"x\",\"extra\"\r\n";
        let f = run(src);
        let r4 = f.get(RULE_4).expect("Rule 4");
        assert!(r4.iter().any(|x| x.line == Some(5)), "{r4:?}");
    }

    #[test]
    fn rule_4_reports_missing_heading_once() {
        let src = "\"GROUP\",\"PROJ\"\r\n\"UNIT\",\"\"\r\n\"TYPE\",\"ID\"\r\n\
                   \"DATA\",\"P1\"\r\n";
        let f = run(src);
        let r4 = f.get(RULE_4).expect("Rule 4");
        let missing: Vec<_> = r4
            .iter()
            .filter(|x| x.desc.contains("HEADING row missing"))
            .collect();
        assert_eq!(missing.len(), 1, "{r4:?}");
        assert_eq!(missing[0].line, None);
    }

    #[test]
    fn rule_2b_flags_missing_type_row() {
        // HEADING/UNIT present, TYPE absent → the `None` arm of the TYPE
        // match (distinct from the misplacement arm, exercised above).
        let src = "\"GROUP\",\"PROJ\"\r\n\"HEADING\",\"PROJ_ID\"\r\n\
                   \"UNIT\",\"\"\r\n\"DATA\",\"P1\"\r\n";
        let f = run(src);
        let r2b = f.get(RULE_2B).expect("Rule 2b");
        assert!(
            r2b.iter().any(|x| x.desc.contains("TYPE row missing")),
            "{r2b:?}"
        );
    }

    #[test]
    fn rule_2b_flags_misplaced_unit_below_a_gap() {
        // HEADING on line 2, but UNIT not immediately below it (a stray
        // descriptor line between them) → UNIT-misplaced arm.
        // GROUP(1) HEADING(2) TYPE(3) UNIT(4) DATA(5): UNIT at line 4 is
        // not hl+1 (=3), so the placement check fires for UNIT too.
        let src = "\"GROUP\",\"PROJ\"\r\n\"HEADING\",\"PROJ_ID\"\r\n\
                   \"TYPE\",\"ID\"\r\n\"UNIT\",\"\"\r\n\"DATA\",\"P1\"\r\n";
        let f = run(src);
        let r2b = f.get(RULE_2B).expect("Rule 2b");
        assert!(
            r2b.iter().any(|x| x.desc.contains("UNIT row is misplaced")),
            "expected a UNIT-misplacement finding: {r2b:?}"
        );
    }

    #[test]
    fn rule_4_flags_thin_group_row_missing_name() {
        // A GROUP row carrying only the descriptor (no group name) →
        // the `fields.len() < 2` malformed-GROUP arm. The bare "GROUP"
        // line still parses (the parser tolerates it); structure flags it.
        let src = "\"GROUP\"\r\n\"HEADING\",\"PROJ_ID\"\r\n\
                   \"UNIT\",\"\"\r\n\"TYPE\",\"ID\"\r\n\"DATA\",\"P1\"\r\n";
        let pf = parse_str(src).expect("parses");
        let mut f = Findings::new();
        // The parser may name the group ""; drive rule_4 over whatever
        // group it produced.
        check(&pf, &mut f);
        let r4 = f.get(RULE_4).expect("Rule 4");
        assert!(
            r4.iter().any(|x| x.desc.contains("missing the group name")),
            "{r4:?}"
        );
    }

    #[test]
    fn rule_4_flags_unit_and_type_field_count_mismatch() {
        // HEADING has 2 fields; UNIT has 1 and TYPE has 3 — both the
        // UNIT-count and TYPE-count arms fire (lines 182-197).
        let src = "\"GROUP\",\"PROJ\"\r\n\"HEADING\",\"PROJ_ID\",\"PROJ_NAME\"\r\n\
                   \"UNIT\",\"\"\r\n\"TYPE\",\"ID\",\"X\",\"X\"\r\n\
                   \"DATA\",\"P1\",\"Site\"\r\n";
        let f = run(src);
        let r4 = f.get(RULE_4).expect("Rule 4");
        assert!(
            r4.iter().any(|x| x.desc.contains("UNIT row field count")),
            "expected UNIT field-count mismatch: {r4:?}"
        );
        assert!(
            r4.iter().any(|x| x.desc.contains("TYPE row field count")),
            "expected TYPE field-count mismatch: {r4:?}"
        );
    }
}
