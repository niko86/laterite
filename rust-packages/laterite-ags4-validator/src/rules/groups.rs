//! Mandatory- and definition-group rules: AGS4.1/4.2 Rules 12, 13,
//! 14, 15, 16 (+16a), 17, 18 (+18a).
//!
//! CLEAN-ROOM. Implemented from the AGS4 spec (`reports/AGS 4_1.pdf` &
//! `reports/AGS 4_2.pdf` §4.1.1). python-ags4 (LGPL-3.0) was read only
//! to learn *which* checks it performs and *how* it interprets the
//! prose (facts about the AGS standard, not copyrightable). No code,
//! structure, or wording was copied.
//!
//! Spec text (verbatim, AGS 4.2 §4.1.1, pp.156–157 — AGS 4.1 identical):
//!
//! * **Rule 12** — "Data does not have to be included against each
//!   HEADING unless REQUIRED (Rule 10b). The data FIELD can be null; a
//!   null entry is defined as \"\" (two quotes together)." → wholly
//!   subsumed by Rule 10b (V7); python-ags4's `rule_12` is a no-op and
//!   so is ours. No finding originates here. (O-15)
//! * **Rule 13** — "Each data file shall contain the PROJ GROUP which
//!   shall contain only one data row and, as a minimum, shall contain
//!   data under the headings defined as REQUIRED (Rule 10b)."
//! * **Rule 14** — "Each data file shall contain the TRAN GROUP which
//!   shall contain only one data row …" (REQUIRED-fill → Rule 10b/V7).
//! * **Rule 15** — "Each data file shall contain the UNIT GROUP to
//!   list all units used within the data file. Every unit of
//!   measurement entered in the UNIT row of a GROUP or data entered in
//!   a FIELD where the field TYPE is defined as \"PU\" … shall be
//!   listed and defined in the UNIT GROUP."
//! * **Rule 16** — "Each data file shall contain the ABBR GROUP when
//!   abbreviations have been included in the data file. The
//!   abbreviations listed in the ABBR GROUP shall include definitions
//!   for all abbreviations entered in a FIELD where the data TYPE is
//!   defined as \"PA\" …"
//! * **Rule 16a** — multi-abbreviation FIELDs are split on the
//!   TRAN_RCON concatenator (default `+`); each part must be defined
//!   separately in ABBR.
//! * **Rule 17** — "Each data file shall contain the TYPE GROUP to
//!   define the field TYPEs used within the data file. Every data type
//!   entered in the TYPE row of a GROUP shall be listed and defined in
//!   the TYPE GROUP."
//! * **Rule 18** — "Each data file shall contain the DICT GROUP where
//!   non-standard GROUP and HEADING names have been included …" →
//!   follow-on to Rule 9 (V4): if a non-standard heading exists but no
//!   DICT group, flag it. (O-17)
//! * **Rule 18a** — DICT order defines heading append/Record-Link
//!   order → already enforced by V4's effective-dict ordering (Rule 7)
//!   and V7's Rule 11; nothing extra here. (O-18)
//!
//! Note: Rules 13/14 also fire on a zero-DATA-row PROJ/TRAN, which
//! Rule 2 (V2) reports too — an intentional double-report kept for
//! python-ags4 finding-count parity (O-16). The "REQUIRED fields
//! filled" half of Rules 13/14 is Rule 10b's job (V7).

use std::collections::{BTreeMap, BTreeSet};

use crate::CheckOptions;
use crate::dict::Dictionary;
use crate::findings::{Findings, Location, Severity, add, add_at};
use crate::parse::{ParsedFile, ParsedGroup};

const RULE_13: &str = "AGS Format Rule 13";
const RULE_14: &str = "AGS Format Rule 14";
const RULE_15: &str = "AGS Format Rule 15";
const RULE_16: &str = "AGS Format Rule 16";
const RULE_17: &str = "AGS Format Rule 17";
const RULE_18: &str = "AGS Format Rule 18";
// Top-level FYI bucket (no "Related to Rule N" suffix) — python-ags4
// uses this for the unrecognised-TRAN_AGS hint, distinct from
// "FYI (Related to Rule 1)" / "FYI (Related to Rule 16)".
const FYI: &str = "FYI";
const RULE_16_FYI: &str = "FYI (Related to Rule 16)";

const KNOWN_TRAN_AGS: &[&str] = &["4.0", "4.0.3", "4.0.4", "4.1", "4.1.1", "4.2"];

pub fn check(parsed: &ParsedFile, dict: &Dictionary, opts: &CheckOptions, found: &mut Findings) {
    single_row_group(parsed, "PROJ", RULE_13, found);
    single_row_group(parsed, "TRAN", RULE_14, found);
    rule_15(parsed, found);
    rule_16(parsed, found);
    rule_17(parsed, found);
    rule_18(parsed, found);
    if opts.include_fyi {
        tran_ags_fyi(parsed, found);
        rule_16_fyi(parsed, dict, found);
    }
}

/// FYI emit: for each abbreviation defined in the file's ABBR group,
/// compare its `ABBR_DESC` against the bundled standard ABBR table
/// for the active edition. Emits one finding per mismatch:
/// `'HDNG: Description of abbreviation "CODE" is "X" but it should
/// be "Y" according to the standard abbreviations list.'`
/// Mirrors python-ags4 `check.fyi_16_1`.
fn rule_16_fyi(parsed: &ParsedFile, dict: &Dictionary, found: &mut Findings) {
    let Some(abbr) = parsed.groups.get("ABBR") else {
        return;
    };
    let (Some(hi), Some(ci), Some(di)) = (
        col(abbr, "ABBR_HDNG"),
        col(abbr, "ABBR_CODE"),
        col(abbr, "ABBR_DESC"),
    ) else {
        return; // malformed ABBR — main Rule 16 / Rule 9 report it
    };
    for row in &abbr.rows {
        let hdng = row.values.get(hi).map(String::as_str).unwrap_or("");
        let code = row.values.get(ci).map(String::as_str).unwrap_or("");
        let file_desc = row.values.get(di).map(String::as_str).unwrap_or("");
        if hdng.is_empty() || code.is_empty() {
            continue;
        }
        let Some(std_desc) = dict.abbr_desc(hdng, code) else {
            continue; // not in the standard ABBR — main Rule 16 covers
            // "abbreviation used but not defined anywhere".
        };
        // Case-insensitive comparison — python-ags4's `fyi_16_1` does
        // `str.lower().eq(...)`, so e.g. "Other Field" vs the standard
        // "Other field" is not flagged (the casing varies in real
        // dictionaries and tooling round-trips). Matching that here so
        // the FYI doesn't fire noise on legitimate variation.
        if std_desc.eq_ignore_ascii_case(file_desc) {
            continue;
        }
        add_at(
            found,
            RULE_16_FYI,
            Some(row.line),
            "ABBR",
            format!(
                "{hdng}: Description of abbreviation {code:?} is \
                 {file_desc:?} but it should be {std_desc:?} \
                 according to the standard abbreviations list."
            ),
            Location::default(),
            Severity::Fyi,
        );
    }
}

/// FYI emit: TRAN_AGS value present but not a recognised AGS4 edition.
/// python-ags4 emits the same FYI; useful to flag custom/typo'd
/// edition strings without raising a Rule 14 error (TRAN_AGS being
/// *present* is what Rule 14 requires; *recognised* isn't a rule).
fn tran_ags_fyi(parsed: &ParsedFile, found: &mut Findings) {
    let Some(tran) = parsed.groups.get("TRAN") else {
        return;
    };
    let Some(ci) = tran.headings.iter().position(|h| h == "TRAN_AGS") else {
        return;
    };
    let Some(v) = tran.rows.first().and_then(|r| r.values.get(ci)) else {
        return;
    };
    let t = v.trim();
    if t.is_empty() || KNOWN_TRAN_AGS.contains(&t) {
        return;
    }
    add_at(
        found,
        FYI,
        None,
        "TRAN",
        format!(
            "TRAN_AGS is not a recognized AGS4 version: {t:?}. The \
             standard editions are 4.0.3 / 4.0.4 / 4.1 / 4.1.1 / 4.2."
        ),
        Location::default(),
        Severity::Fyi,
    );
}

/// Index of a heading within a group's HEADING row, by name.
fn col(g: &ParsedGroup, name: &str) -> Option<usize> {
    g.headings.iter().position(|h| h == name)
}

/// Distinct non-empty DATA values of one column.
fn column_values(g: &ParsedGroup, ci: usize) -> BTreeSet<&str> {
    g.rows
        .iter()
        .filter_map(|r| r.values.get(ci).map(String::as_str))
        .filter(|v| !v.is_empty())
        .collect()
}

/// Rules 13/14 — PROJ/TRAN must exist with exactly one DATA row.
fn single_row_group(parsed: &ParsedFile, code: &str, rule: &str, found: &mut Findings) {
    let Some(g) = parsed.groups.get(code) else {
        add(found, rule, None, code, format!("{code} group not found."));
        return;
    };
    match g.rows.len() {
        0 => add(
            found,
            rule,
            Some(g.group_line),
            code,
            format!("The {code} group must contain at least one DATA row."),
        ),
        1 => {}
        _ => {
            // First row is allowed; every subsequent one is the defect.
            for row in &g.rows[1..] {
                add(
                    found,
                    rule,
                    Some(row.line),
                    code,
                    format!("The {code} group must contain only one DATA row."),
                );
            }
        }
    }
}

/// Rule 15 — every unit used (UNIT rows + `PU`-typed DATA) must be
/// defined in the UNIT group's `UNIT_UNIT` column.
fn rule_15(parsed: &ParsedFile, found: &mut Findings) {
    let Some(unit_g) = parsed.groups.get("UNIT") else {
        add(found, RULE_15, None, "UNIT", "UNIT group not found.");
        return;
    };

    // First-seen location per used unit (for a helpful message).
    let mut used: BTreeMap<String, String> = BTreeMap::new();
    let note = |u: &str, loc: String, m: &mut BTreeMap<String, String>| {
        if !u.is_empty() && u != "UNIT" {
            m.entry(u.to_string()).or_insert(loc);
        }
    };

    for code in &parsed.group_order {
        let g = &parsed.groups[code];
        for u in &g.units {
            note(u, format!("the UNIT row of {code}"), &mut used);
        }
        for (ci, ty) in g.types.iter().enumerate() {
            if ty.trim() == "PU" {
                let hd = g.headings.get(ci).map(String::as_str).unwrap_or("?");
                for v in column_values(g, ci) {
                    note(v, format!("column {hd} of {code}"), &mut used);
                }
            }
        }
    }

    let defined: BTreeSet<&str> = match col(unit_g, "UNIT_UNIT") {
        Some(ci) => column_values(unit_g, ci),
        None => BTreeSet::new(), // missing UNIT_UNIT → Rule 10a/4 reports it
    };

    for (unit, loc) in &used {
        if !defined.contains(unit.as_str()) {
            add(
                found,
                RULE_15,
                None,
                "UNIT",
                format!("Unit {unit:?} (first used in {loc}) is not defined in the UNIT group."),
            );
        }
    }
}

/// Rule 16/16a — every abbreviation in a `PA`-typed FIELD (split on the
/// TRAN_RCON concatenator) must be defined in ABBR for that heading.
fn rule_16(parsed: &ParsedFile, found: &mut Findings) {
    // Does the file use any PA column at all?
    let has_pa = parsed
        .group_order
        .iter()
        .any(|c| parsed.groups[c].types.iter().any(|t| t.trim() == "PA"));
    if !has_pa {
        return; // ABBR not required
    }

    let Some(abbr) = parsed.groups.get("ABBR") else {
        add(found, RULE_16, None, "ABBR", "ABBR group not found.");
        return;
    };

    // (ABBR_HDNG, ABBR_CODE) pairs that are defined.
    let defined: BTreeSet<(&str, &str)> = match (col(abbr, "ABBR_HDNG"), col(abbr, "ABBR_CODE")) {
        (Some(hi), Some(ci)) => abbr
            .rows
            .iter()
            .filter_map(|r| Some((r.values.get(hi)?.as_str(), r.values.get(ci)?.as_str())))
            .collect(),
        _ => return, // malformed ABBR — Rule 10a/4 reports it
    };

    // Concatenator from TRAN_RCON (Rule 16a). Absent/empty → no split.
    let concat = parsed
        .groups
        .get("TRAN")
        .and_then(|t| col(t, "TRAN_RCON").map(|ci| (t, ci)))
        .and_then(|(t, ci)| t.rows.first().and_then(|r| r.values.get(ci)))
        .map(String::as_str)
        .filter(|s| !s.is_empty());

    for code in &parsed.group_order {
        let g = &parsed.groups[code];
        for (ci, ty) in g.types.iter().enumerate() {
            if ty.trim() != "PA" {
                continue;
            }
            let hd = g.headings.get(ci).map(String::as_str).unwrap_or("");
            for v in column_values(g, ci) {
                let parts: Vec<&str> = match concat {
                    Some(sep) => v.split(sep).collect(),
                    None => vec![v],
                };
                for p in parts {
                    if !p.is_empty() && !defined.contains(&(hd, p)) {
                        add(
                            found,
                            RULE_16,
                            None,
                            code,
                            format!(
                                "Abbreviation {p:?} under {hd} is not defined in the ABBR group."
                            ),
                        );
                    }
                }
            }
        }
    }
}

/// Rule 17 — every TYPE code used in any group's TYPE row must be
/// defined in the TYPE group's `TYPE_TYPE` column.
fn rule_17(parsed: &ParsedFile, found: &mut Findings) {
    let Some(type_g) = parsed.groups.get("TYPE") else {
        add(found, RULE_17, None, "TYPE", "TYPE group not found.");
        return;
    };

    let defined: BTreeSet<&str> = match col(type_g, "TYPE_TYPE") {
        Some(ci) => column_values(type_g, ci),
        None => BTreeSet::new(),
    };

    let mut used: BTreeSet<&str> = BTreeSet::new();
    for code in &parsed.group_order {
        for t in &parsed.groups[code].types {
            let t = t.trim();
            // Skip "" (not a data type) and the "TYPE" descriptor — a
            // deliberate refinement; python-ags4 only excludes "TYPE"
            // (O-19), but an empty cell is never a real data type.
            if !t.is_empty() && t != "TYPE" {
                used.insert(t);
            }
        }
    }

    for t in &used {
        if !defined.contains(t) {
            add(
                found,
                RULE_17,
                None,
                "TYPE",
                format!("Data type {t:?} is not defined in the TYPE group."),
            );
        }
    }
}

/// Rule 18 — a non-standard heading with no DICT group to define it.
/// Follow-on to Rule 9 (V4): `dictionary::check` runs first in
/// `run_all`, so the presence of a Rule 9 finding *is* "the file uses
/// a non-standard heading" (python-ags4 wires this the same way, by
/// inspecting rule_9's output).
fn rule_18(parsed: &ParsedFile, found: &mut Findings) {
    let non_standard = found.contains_key("AGS Format Rule 9");
    if non_standard && !parsed.groups.contains_key("DICT") {
        add(
            found,
            RULE_18,
            None,
            "DICT",
            "DICT group not found, but the file uses non-standard headings \
             (see Rule 9) that must be defined in a DICT group."
                .to_string(),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dict::DictVersion;
    use crate::parse::parse_str;

    fn run(src: &str) -> Findings {
        let pf = parse_str(src).expect("fixture parses");
        let mut f = Findings::new();
        check(
            &pf,
            &Dictionary::bundled(DictVersion::V4_1),
            &CheckOptions::default(),
            &mut f,
        );
        f
    }

    /// FYI-enabled runner — the FYI emitters (`tran_ags_fyi`,
    /// `rule_16_fyi`) only run under `include_fyi`.
    fn run_fyi(src: &str) -> Findings {
        let pf = parse_str(src).expect("fixture parses");
        let mut f = Findings::new();
        check(
            &pf,
            &Dictionary::bundled(DictVersion::V4_2),
            &CheckOptions {
                include_fyi: true,
                ..Default::default()
            },
            &mut f,
        );
        f
    }

    #[test]
    fn rule_13_14_flag_missing_proj_and_tran() {
        // A lone LOCA group — no PROJ, no TRAN.
        let src = "\"GROUP\",\"LOCA\"\r\n\"HEADING\",\"LOCA_ID\"\r\n\
                   \"UNIT\",\"\"\r\n\"TYPE\",\"ID\"\r\n\"DATA\",\"BH1\"\r\n";
        let f = run(src);
        assert!(f.get(RULE_13).is_some_and(|v| v[0].group == "PROJ"));
        assert!(f.get(RULE_14).is_some_and(|v| v[0].group == "TRAN"));
    }

    #[test]
    fn rule_13_flags_extra_proj_data_rows() {
        let src = "\"GROUP\",\"PROJ\"\r\n\"HEADING\",\"PROJ_ID\"\r\n\
                   \"UNIT\",\"\"\r\n\"TYPE\",\"ID\"\r\n\
                   \"DATA\",\"P1\"\r\n\"DATA\",\"P2\"\r\n";
        let f = run(src);
        let r13 = f.get(RULE_13).expect("Rule 13");
        // Only the *second* DATA row (line 6) is the defect.
        assert!(r13.iter().any(|x| x.line == Some(6)), "{r13:?}");
        assert!(!r13.iter().any(|x| x.line == Some(5)), "{r13:?}");
    }

    #[test]
    fn rule_15_flags_undefined_unit() {
        // LOCA_FDEP declared in 'm', but UNIT group only defines 'mm'.
        let src = "\"GROUP\",\"LOCA\"\r\n\"HEADING\",\"LOCA_ID\",\"LOCA_FDEP\"\r\n\
                   \"UNIT\",\"\",\"m\"\r\n\"TYPE\",\"ID\",\"2DP\"\r\n\
                   \"DATA\",\"BH1\",\"1.00\"\r\n\r\n\
                   \"GROUP\",\"UNIT\"\r\n\"HEADING\",\"UNIT_UNIT\",\"UNIT_DESC\"\r\n\
                   \"UNIT\",\"\",\"\"\r\n\"TYPE\",\"X\",\"X\"\r\n\
                   \"DATA\",\"mm\",\"millimetres\"\r\n";
        let f = run(src);
        let r15 = f.get(RULE_15).expect("Rule 15");
        assert!(r15.iter().any(|x| x.desc.contains("\"m\"")), "{r15:?}");
    }

    #[test]
    fn rule_16_flags_undefined_abbreviation_and_missing_abbr() {
        // PA column with value "XX" and no ABBR group.
        let no_abbr = "\"GROUP\",\"LOCA\"\r\n\"HEADING\",\"LOCA_ID\",\"LOCA_TYPE\"\r\n\
                       \"UNIT\",\"\",\"\"\r\n\"TYPE\",\"ID\",\"PA\"\r\n\
                       \"DATA\",\"BH1\",\"XX\"\r\n";
        assert!(
            run(no_abbr)
                .get(RULE_16)
                .is_some_and(|v| v[0].group == "ABBR")
        );

        // ABBR present but doesn't define "XX" for LOCA_TYPE.
        let bad = "\"GROUP\",\"LOCA\"\r\n\"HEADING\",\"LOCA_ID\",\"LOCA_TYPE\"\r\n\
                   \"UNIT\",\"\",\"\"\r\n\"TYPE\",\"ID\",\"PA\"\r\n\
                   \"DATA\",\"BH1\",\"XX\"\r\n\r\n\
                   \"GROUP\",\"ABBR\"\r\n\
                   \"HEADING\",\"ABBR_HDNG\",\"ABBR_CODE\",\"ABBR_DESC\"\r\n\
                   \"UNIT\",\"\",\"\",\"\"\r\n\"TYPE\",\"X\",\"X\",\"X\"\r\n\
                   \"DATA\",\"LOCA_TYPE\",\"TP\",\"Trial pit\"\r\n";
        let f = run(bad);
        let r16 = f.get(RULE_16).expect("Rule 16");
        assert!(r16.iter().any(|x| x.desc.contains("\"XX\"")), "{r16:?}");
    }

    #[test]
    fn rule_17_flags_undefined_type_and_missing_type_group() {
        let src = "\"GROUP\",\"LOCA\"\r\n\"HEADING\",\"LOCA_ID\",\"LOCA_FDEP\"\r\n\
                   \"UNIT\",\"\",\"m\"\r\n\"TYPE\",\"ID\",\"2DP\"\r\n\
                   \"DATA\",\"BH1\",\"1.00\"\r\n";
        // No TYPE group at all.
        let f = run(src);
        assert!(
            f.get(RULE_17)
                .is_some_and(|v| v[0].desc.contains("not found"))
        );
    }

    #[test]
    fn rule_18_follows_rule_9() {
        // Simulate Rule 9 having fired, no DICT group.
        let pf = parse_str(
            "\"GROUP\",\"LOCA\"\r\n\"HEADING\",\"LOCA_ID\"\r\n\
             \"UNIT\",\"\"\r\n\"TYPE\",\"ID\"\r\n\"DATA\",\"BH1\"\r\n",
        )
        .unwrap();
        let mut f = Findings::new();
        add(
            &mut f,
            "AGS Format Rule 9",
            Some(2),
            "LOCA",
            "X not in dict.",
        );
        check(
            &pf,
            &Dictionary::bundled(DictVersion::V4_1),
            &CheckOptions::default(),
            &mut f,
        );
        assert!(
            f.contains_key(RULE_18),
            "Rule 18 should follow Rule 9: {f:?}"
        );
    }

    #[test]
    fn rule_18_silent_without_rule_9() {
        let pf = parse_str(
            "\"GROUP\",\"LOCA\"\r\n\"HEADING\",\"LOCA_ID\"\r\n\
             \"UNIT\",\"\"\r\n\"TYPE\",\"ID\"\r\n\"DATA\",\"BH1\"\r\n",
        )
        .unwrap();
        let mut f = Findings::new();
        check(
            &pf,
            &Dictionary::bundled(DictVersion::V4_1),
            &CheckOptions::default(),
            &mut f,
        );
        assert!(!f.contains_key(RULE_18), "no Rule 9 → no Rule 18: {f:?}");
    }

    #[test]
    fn rule_13_flags_zero_data_row_proj() {
        // PROJ present with HEADING/UNIT/TYPE but no DATA row → the
        // `0 => …"at least one DATA row"` arm of single_row_group.
        let src = "\"GROUP\",\"PROJ\"\r\n\"HEADING\",\"PROJ_ID\"\r\n\
                   \"UNIT\",\"\"\r\n\"TYPE\",\"ID\"\r\n";
        let f = run(src);
        let r13 = f.get(RULE_13).expect("Rule 13");
        assert!(
            r13.iter().any(|x| x.desc.contains("at least one DATA row")),
            "{r13:?}"
        );
    }

    #[test]
    fn rule_15_flags_unit_used_in_pu_typed_column() {
        // A `PU`-typed DATA cell carries a unit *value* ("kPa") that must
        // be defined in UNIT — the PU branch of rule_15 (distinct from
        // the UNIT-row source). UNIT defines only "mm".
        let src = "\"GROUP\",\"LOCA\"\r\n\
                   \"HEADING\",\"LOCA_ID\",\"LOCA_UNIT\"\r\n\
                   \"UNIT\",\"\",\"\"\r\n\"TYPE\",\"ID\",\"PU\"\r\n\
                   \"DATA\",\"BH1\",\"kPa\"\r\n\r\n\
                   \"GROUP\",\"UNIT\"\r\n\"HEADING\",\"UNIT_UNIT\"\r\n\
                   \"UNIT\",\"\"\r\n\"TYPE\",\"X\"\r\n\"DATA\",\"mm\"\r\n";
        let f = run(src);
        let r15 = f.get(RULE_15).expect("Rule 15");
        assert!(
            r15.iter()
                .any(|x| x.desc.contains("\"kPa\"") && x.desc.contains("column LOCA_UNIT")),
            "PU-typed unit value must be checked: {r15:?}"
        );
    }

    #[test]
    fn rule_16_splits_on_tran_rcon_concatenator() {
        // A PA cell "TP+CP" with TRAN_RCON "+" splits into TP, CP; CP is
        // undefined in ABBR → the concat-split branch (rule 16a).
        let src = "\"GROUP\",\"TRAN\"\r\n\"HEADING\",\"TRAN_RCON\"\r\n\
                   \"UNIT\",\"\"\r\n\"TYPE\",\"X\"\r\n\"DATA\",\"+\"\r\n\r\n\
                   \"GROUP\",\"SAMP\"\r\n\"HEADING\",\"SAMP_ID\",\"SAMP_TYPE\"\r\n\
                   \"UNIT\",\"\",\"\"\r\n\"TYPE\",\"ID\",\"PA\"\r\n\
                   \"DATA\",\"S1\",\"TP+CP\"\r\n\r\n\
                   \"GROUP\",\"ABBR\"\r\n\
                   \"HEADING\",\"ABBR_HDNG\",\"ABBR_CODE\",\"ABBR_DESC\"\r\n\
                   \"UNIT\",\"\",\"\",\"\"\r\n\"TYPE\",\"X\",\"X\",\"X\"\r\n\
                   \"DATA\",\"SAMP_TYPE\",\"TP\",\"Trial pit\"\r\n";
        let f = run(src);
        let r16 = f.get(RULE_16).expect("Rule 16");
        // TP is defined, CP is not → only CP flagged (proves the split).
        assert!(
            r16.iter().any(|x| x.desc.contains("\"CP\"")),
            "undefined split-part CP must flag: {r16:?}"
        );
        assert!(
            !r16.iter().any(|x| x.desc.contains("\"TP\"")),
            "defined split-part TP must not flag: {r16:?}"
        );
    }

    #[test]
    fn rule_16_silent_when_no_pa_column() {
        // No PA-typed column anywhere → ABBR not required, rule_16 early
        // return (the `!has_pa` guard).
        let src = "\"GROUP\",\"LOCA\"\r\n\"HEADING\",\"LOCA_ID\"\r\n\
                   \"UNIT\",\"\"\r\n\"TYPE\",\"ID\"\r\n\"DATA\",\"BH1\"\r\n";
        assert!(!run(src).contains_key(RULE_16));
    }

    #[test]
    fn tran_ags_fyi_flags_unrecognised_edition_string() {
        // TRAN_AGS = "9.9" — present (so Rule 14 is satisfied) but not a
        // known edition → the top-level FYI bucket (include_fyi only).
        let src = "\"GROUP\",\"TRAN\"\r\n\"HEADING\",\"TRAN_AGS\"\r\n\
                   \"UNIT\",\"\"\r\n\"TYPE\",\"X\"\r\n\"DATA\",\"9.9\"\r\n";
        let f = run_fyi(src);
        let fyi = f.get(FYI).expect("top-level FYI");
        assert!(
            fyi.iter()
                .any(|x| x.group == "TRAN" && x.desc.contains("9.9")),
            "{fyi:?}"
        );
        // Default (no FYI) must stay silent.
        assert!(!run(src).contains_key(FYI));
    }

    #[test]
    fn tran_ags_fyi_silent_for_recognised_edition() {
        // A recognised edition string ("4.2") is NOT flagged even with
        // FYI on — the `KNOWN_TRAN_AGS.contains` early return.
        let src = "\"GROUP\",\"TRAN\"\r\n\"HEADING\",\"TRAN_AGS\"\r\n\
                   \"UNIT\",\"\"\r\n\"TYPE\",\"X\"\r\n\"DATA\",\"4.2\"\r\n";
        assert!(!run_fyi(src).contains_key(FYI));
    }

    #[test]
    fn rule_16_fyi_flags_nonstandard_abbr_description() {
        // ABBR defines ARTW_TYPE/DRY with a description that differs from
        // the bundled-4.2 standard ("Dry test") → rule_16_fyi emits one
        // mismatch finding. A PA column is present so the main Rule 16
        // path also runs; ARTW_TYPE/DRY *is* defined so main Rule 16 is
        // silent on it, leaving the FYI as the only signal.
        let src = "\"GROUP\",\"FROM\"\r\n\"HEADING\",\"FROM_ID\",\"ARTW_TYPE\"\r\n\
                   \"UNIT\",\"\",\"\"\r\n\"TYPE\",\"ID\",\"PA\"\r\n\
                   \"DATA\",\"X1\",\"DRY\"\r\n\r\n\
                   \"GROUP\",\"ABBR\"\r\n\
                   \"HEADING\",\"ABBR_HDNG\",\"ABBR_CODE\",\"ABBR_DESC\"\r\n\
                   \"UNIT\",\"\",\"\",\"\"\r\n\"TYPE\",\"X\",\"X\",\"X\"\r\n\
                   \"DATA\",\"ARTW_TYPE\",\"DRY\",\"Totally wrong wording\"\r\n";
        let f = run_fyi(src);
        let fyi = f.get(RULE_16_FYI).expect("Rule 16 FYI");
        assert!(
            fyi.iter()
                .any(|x| x.group == "ABBR" && x.desc.contains("DRY")),
            "{fyi:?}"
        );
    }

    #[test]
    fn rule_16_fyi_silent_when_description_matches_case_insensitively() {
        // Same (HDNG, CODE) but description equal to the standard up to
        // case → the `eq_ignore_ascii_case` continue; no FYI.
        let src = "\"GROUP\",\"FROM\"\r\n\"HEADING\",\"FROM_ID\",\"ARTW_TYPE\"\r\n\
                   \"UNIT\",\"\",\"\"\r\n\"TYPE\",\"ID\",\"PA\"\r\n\
                   \"DATA\",\"X1\",\"DRY\"\r\n\r\n\
                   \"GROUP\",\"ABBR\"\r\n\
                   \"HEADING\",\"ABBR_HDNG\",\"ABBR_CODE\",\"ABBR_DESC\"\r\n\
                   \"UNIT\",\"\",\"\",\"\"\r\n\"TYPE\",\"X\",\"X\",\"X\"\r\n\
                   \"DATA\",\"ARTW_TYPE\",\"DRY\",\"DRY TEST\"\r\n";
        assert!(!run_fyi(src).contains_key(RULE_16_FYI));
    }
}
