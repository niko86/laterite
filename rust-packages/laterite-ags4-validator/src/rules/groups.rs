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
//!   `TRAN_RCON` concatenator (default `+`); each part must be defined
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
// WARNING-tier label for Rule 18 structural defects (#200). A separate label
// (NOT "AGS Format Rule 18") so the compat severity classifier — which keys off
// the rule-label substring — never miscounts it as an error, and the error-tier
// Rule 18 bucket stays byte-stable. Mirrors the "FYI (Related to Rule N)" scheme.
const RULE_18_WARN: &str = "Warning (Related to Rule 18)";
// WARNING-tier label for an unrecognised TRAN_AGS edition string (#203). laterite
// is deliberately STRICTER than python-ags4 here: an unrecognised edition means we
// fall back to a default dictionary (4.1.1) and may validate against the WRONG
// schema, which warrants a WARNING the user sees by default — not a buried FYI
// (OBSERVATIONS O-44). `compat` still mirrors python-ags4's FYI tier (see `check`).
// Same "Warning (...)" scheme as RULE_18_WARN so the compat severity classifier
// never miscounts it as an error.
const RULE_14_WARN: &str = "Warning (Related to Rule 14)";

const KNOWN_TRAN_AGS: &[&str] = &["4.0", "4.0.3", "4.0.4", "4.1", "4.1.1", "4.2"];

pub fn check(parsed: &ParsedFile, dict: &Dictionary, opts: &CheckOptions, found: &mut Findings) {
    single_row_group(parsed, "PROJ", RULE_13, found);
    single_row_group(parsed, "TRAN", RULE_14, found);
    rule_15(parsed, found);
    rule_16(parsed, found);
    rule_17(parsed, found);
    rule_18(parsed, found);
    if opts.include_warnings {
        rule_18_structure(parsed, found);
        // Native (warnings-on) view: an unrecognised TRAN_AGS is a WARNING — the
        // schema-fallback risk should be visible by default (O-44).
        tran_ags_unrecognised(parsed, found, RULE_14_WARN, Severity::Warning);
    }
    if opts.include_fyi {
        // FYI-only mode (i.e. `compat`, which runs include_fyi without
        // include_warnings) mirrors python-ags4: it emits the same finding at the
        // FYI tier. Guarded so a caller asking for BOTH tiers sees it once (as the
        // stricter WARNING above), never duplicated.
        if !opts.include_warnings {
            tran_ags_unrecognised(parsed, found, FYI, Severity::Fyi);
        }
        rule_16_fyi(parsed, dict, found);
        rule_16_fyi_nonstandard_abbr(parsed, dict, found);
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
        let hdng = row.values.get(hi).map_or("", String::as_str);
        let code = row.values.get(ci).map_or("", String::as_str);
        let file_desc = row.values.get(di).map_or("", String::as_str);
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

/// FYI emit: an abbreviation the file SELF-DECLARES in its ABBR group that is
/// not a recognised standard abbreviation for its heading. The file is
/// spec-legal — Rule 16 only requires a `PA` value be defined in ABBR, which it
/// is — but a non-standard or mistyped code (e.g. `"Borng"` for `"Boring"`)
/// doesn't interoperate with tooling that expects the standard picklist, so we
/// surface it as an FYI (the owner's deliberate choice over a WARNING, since the
/// file breaks no rule). Bounded to headings that HAVE a bundled standard
/// picklist; a genuinely custom / DICT-defined `PA` heading has no standard set
/// to judge against and is skipped, keeping the FYI quiet on bespoke schemas.
///
/// This is a laterite-originated check — python-ags4 has no equivalent (its
/// `fyi_16_1` only flags description drift on an *otherwise-standard* code; its
/// Warnings section is unimplemented). See OBSERVATIONS O-43. Complementary to
/// [`rule_16_fyi`]: that fires when a standard code's description differs; this
/// fires when the code itself isn't standard (the case `rule_16_fyi` skips).
fn rule_16_fyi_nonstandard_abbr(parsed: &ParsedFile, dict: &Dictionary, found: &mut Findings) {
    let Some(abbr) = parsed.groups.get("ABBR") else {
        return;
    };
    let (Some(hi), Some(ci)) = (col(abbr, "ABBR_HDNG"), col(abbr, "ABBR_CODE")) else {
        return; // malformed ABBR — main Rule 16 / Rule 9 report it
    };
    for row in &abbr.rows {
        let hdng = row.values.get(hi).map_or("", String::as_str);
        let code = row.values.get(ci).map_or("", String::as_str);
        if hdng.is_empty() || code.is_empty() {
            continue;
        }
        // Only a heading with a bundled standard picklist can have a
        // "non-standard" judgement; a custom / DICT-defined PA heading has no
        // standard set to compare against, so skip it (bounds the FYI).
        if dict.abbr_codes(hdng).is_empty() {
            continue;
        }
        // `abbr_desc` resolves Some(desc) iff the code IS in the standard
        // picklist; None means the file declared a non-standard code.
        if dict.abbr_desc(hdng, code).is_some() {
            continue;
        }
        add_at(
            found,
            RULE_16_FYI,
            Some(row.line),
            "ABBR",
            format!(
                "{hdng}: abbreviation {code:?} is declared in the ABBR group but is \
                 not a recognised standard abbreviation for {hdng}."
            ),
            Location::default(),
            Severity::Fyi,
        );
    }
}

/// `TRAN_AGS` value present but not a recognised AGS4 edition — laterite then falls
/// back to a default dictionary and may validate against the wrong schema. Not a
/// Rule 14 error (`TRAN_AGS` being *present* is what Rule 14 requires; *recognised*
/// isn't a rule). Emitted at the caller's chosen tier: a WARNING for the native
/// (warnings-on) view, or python-ags4's FYI tier for `compat` — see `check`.
fn tran_ags_unrecognised(
    parsed: &ParsedFile,
    found: &mut Findings,
    label: &'static str,
    severity: Severity,
) {
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
        label,
        None,
        "TRAN",
        format!(
            "TRAN_AGS is not a recognized AGS4 version: {t:?}. The \
             standard editions are 4.0.3 / 4.0.4 / 4.1 / 4.1.1 / 4.2."
        ),
        Location::default(),
        severity,
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
                let hd = g.headings.get(ci).map_or("?", String::as_str);
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
/// `TRAN_RCON` concatenator) must be defined in ABBR for that heading.
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
            let hd = g.headings.get(ci).map_or("", String::as_str);
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
/// inspecting `rule_9`'s output).
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

/// WARNING emit (#200, O-44): structural defects in the file's OWN DICT group.
/// The engine only *consumes* DICT to extend the effective dictionary (Rules
/// 7/9) — a malformed DICT silently degrades every downstream check, so the
/// clearest defects are surfaced as opt-in WARNINGs. Bounded first cut: a
/// missing required DICT column (`DICT_TYPE` / `DICT_GRP` / `DICT_HDNG`), a row
/// with a blank `DICT_GRP`, and a `HEADING`-type row with a blank `DICT_HDNG`.
/// Softer "REQUIRED where HEADING/GROUP" cells (`DICT_STAT` / `DICT_UNIT` /
/// `DICT_PGRP`) are deliberately deferred to avoid false positives.
///
/// WARNING, not Error: python-ags4 does NO DICT structural validation, so an
/// error here would break the parity baseline; opt-in (`include_warnings`)
/// leaves the default verdict and the compat path untouched. The label is
/// [`RULE_18_WARN`] (not `"AGS Format Rule 18"`) so the compat severity
/// classifier never miscounts it as an error. Clean-room — laterite-originated.
fn rule_18_structure(parsed: &ParsedFile, found: &mut Findings) {
    let Some(dictg) = parsed.groups.get("DICT") else {
        return; // a missing DICT is the error-tier `rule_18`'s job, not this.
    };
    let idx = |n: &str| dictg.headings.iter().position(|h| h == n);
    let (ti, gi, hi) = (idx("DICT_TYPE"), idx("DICT_GRP"), idx("DICT_HDNG"));

    // (1) Missing required columns — without these the DICT can't be interpreted.
    let head_line = dictg.heading_line.unwrap_or(dictg.group_line);
    for (name, present) in [
        ("DICT_TYPE", ti.is_some()),
        ("DICT_GRP", gi.is_some()),
        ("DICT_HDNG", hi.is_some()),
    ] {
        if !present {
            add_at(
                found,
                RULE_18_WARN,
                Some(head_line),
                "DICT",
                format!(
                    "DICT group is missing the required {name} column; its \
                     non-standard definitions can't be validated."
                ),
                Location::default(),
                Severity::Warning,
            );
        }
    }
    // Need DICT_GRP to attribute rows; the missing-column warning already fired.
    let Some(gi) = gi else { return };

    // (2) Per-row defects.
    for row in &dictg.rows {
        let grp = row.values.get(gi).map_or("", String::as_str);
        if grp.is_empty() {
            add_at(
                found,
                RULE_18_WARN,
                Some(row.line),
                "DICT",
                "DICT row has a blank DICT_GRP — every DICT definition must name \
                 the group it belongs to."
                    .to_string(),
                Location::default(),
                Severity::Warning,
            );
            continue;
        }
        // A HEADING-type row must name the heading it defines (a GROUP-type row
        // legitimately has a blank DICT_HDNG, so branch on DICT_TYPE first).
        let dtype = ti
            .and_then(|ti| row.values.get(ti))
            .map_or("", String::as_str);
        let hdng = hi
            .and_then(|hi| row.values.get(hi))
            .map_or("", String::as_str);
        if dtype.eq_ignore_ascii_case("HEADING") && hdng.is_empty() {
            add_at(
                found,
                RULE_18_WARN,
                Some(row.line),
                "DICT",
                format!("DICT row defines a HEADING for group {grp:?} but DICT_HDNG is blank."),
                Location::default(),
                Severity::Warning,
            );
        }
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

    /// FYI-enabled runner — the FYI emitters (`rule_16_fyi`, and the
    /// `tran_ags_unrecognised` FYI tier when warnings are off) only run under
    /// `include_fyi`.
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

    /// WARNING-enabled runner — `rule_18_structure` only runs under
    /// `include_warnings`.
    fn run_warn(src: &str) -> Findings {
        let pf = parse_str(src).expect("fixture parses");
        let mut f = Findings::new();
        check(
            &pf,
            &Dictionary::bundled(DictVersion::V4_2),
            &CheckOptions {
                include_warnings: true,
                ..Default::default()
            },
            &mut f,
        );
        f
    }

    // A PROJ + DICT scaffold; `dict_rows` is the DICT group's HEADING + DATA
    // body (so each test supplies just the DICT shape under test).
    fn dict_fixture(dict_block: &str) -> String {
        format!(
            "\"GROUP\",\"PROJ\"\r\n\"HEADING\",\"PROJ_ID\"\r\n\
             \"UNIT\",\"\"\r\n\"TYPE\",\"ID\"\r\n\"DATA\",\"P1\"\r\n\r\n\
             \"GROUP\",\"DICT\"\r\n{dict_block}"
        )
    }

    #[test]
    fn rule_18_structure_flags_blank_dict_hdng_on_heading_row() {
        // A HEADING-type row that names no heading.
        let src = dict_fixture(
            "\"HEADING\",\"DICT_TYPE\",\"DICT_GRP\",\"DICT_HDNG\"\r\n\
             \"UNIT\",\"\",\"\",\"\"\r\n\"TYPE\",\"X\",\"X\",\"X\"\r\n\
             \"DATA\",\"HEADING\",\"LOCA\",\"\"\r\n",
        );
        let w = run_warn(&src)
            .get(RULE_18_WARN)
            .cloned()
            .expect("a Rule 18 warning");
        assert!(
            w.iter().any(|x| x.desc.contains("DICT_HDNG is blank")),
            "{w:?}"
        );
    }

    #[test]
    fn rule_18_structure_flags_blank_dict_grp() {
        let src = dict_fixture(
            "\"HEADING\",\"DICT_TYPE\",\"DICT_GRP\",\"DICT_HDNG\"\r\n\
             \"UNIT\",\"\",\"\",\"\"\r\n\"TYPE\",\"X\",\"X\",\"X\"\r\n\
             \"DATA\",\"HEADING\",\"\",\"LOCA_XX\"\r\n",
        );
        let w = run_warn(&src)
            .get(RULE_18_WARN)
            .cloned()
            .expect("a Rule 18 warning");
        assert!(w.iter().any(|x| x.desc.contains("blank DICT_GRP")), "{w:?}");
    }

    #[test]
    fn rule_18_structure_flags_missing_required_column() {
        // No DICT_TYPE column at all.
        let src = dict_fixture(
            "\"HEADING\",\"DICT_GRP\",\"DICT_HDNG\"\r\n\
             \"UNIT\",\"\",\"\"\r\n\"TYPE\",\"X\",\"X\"\r\n\
             \"DATA\",\"LOCA\",\"LOCA_XX\"\r\n",
        );
        let w = run_warn(&src)
            .get(RULE_18_WARN)
            .cloned()
            .expect("a Rule 18 warning");
        assert!(
            w.iter()
                .any(|x| x.desc.contains("missing the required DICT_TYPE")),
            "{w:?}"
        );
    }

    #[test]
    fn rule_18_structure_silent_for_group_row_blank_hdng() {
        // A GROUP-type row legitimately carries a blank DICT_HDNG — must NOT warn.
        let src = dict_fixture(
            "\"HEADING\",\"DICT_TYPE\",\"DICT_GRP\",\"DICT_HDNG\",\"DICT_PGRP\"\r\n\
             \"UNIT\",\"\",\"\",\"\",\"\"\r\n\"TYPE\",\"X\",\"X\",\"X\",\"X\"\r\n\
             \"DATA\",\"GROUP\",\"LOCX\",\"\",\"PROJ\"\r\n",
        );
        let f = run_warn(&src);
        assert!(
            f.get(RULE_18_WARN)
                .is_none_or(|w| !w.iter().any(|x| x.desc.contains("DICT_HDNG is blank"))),
            "a GROUP row must not be flagged for a blank DICT_HDNG: {:?}",
            f.get(RULE_18_WARN)
        );
    }

    #[test]
    fn rule_18_structure_off_by_default() {
        let src = dict_fixture(
            "\"HEADING\",\"DICT_TYPE\",\"DICT_GRP\",\"DICT_HDNG\"\r\n\
             \"UNIT\",\"\",\"\",\"\"\r\n\"TYPE\",\"X\",\"X\",\"X\"\r\n\
             \"DATA\",\"HEADING\",\"LOCA\",\"\"\r\n",
        );
        assert!(!run(&src).contains_key(RULE_18_WARN));
    }

    #[test]
    fn rule_18_structure_clean_dict_no_warning() {
        // A well-formed DICT (one HEADING row, one GROUP row) → no warning.
        let src = dict_fixture(
            "\"HEADING\",\"DICT_TYPE\",\"DICT_GRP\",\"DICT_HDNG\",\"DICT_PGRP\"\r\n\
             \"UNIT\",\"\",\"\",\"\",\"\"\r\n\"TYPE\",\"X\",\"X\",\"X\",\"X\"\r\n\
             \"DATA\",\"GROUP\",\"LOCX\",\"\",\"PROJ\"\r\n\
             \"DATA\",\"HEADING\",\"LOCX\",\"LOCX_ID\",\"\"\r\n",
        );
        assert!(!run_warn(&src).contains_key(RULE_18_WARN));
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

    // A PROJ + ABBR scaffold; `{hdng}`/`{code}` is the one ABBR declaration
    // under test. (Other rules fire too — the tests only inspect RULE_16_FYI.)
    fn abbr_fixture(hdng: &str, code: &str) -> String {
        format!(
            "\"GROUP\",\"PROJ\"\r\n\"HEADING\",\"PROJ_ID\"\r\n\
             \"UNIT\",\"\"\r\n\"TYPE\",\"ID\"\r\n\"DATA\",\"P1\"\r\n\r\n\
             \"GROUP\",\"ABBR\"\r\n\
             \"HEADING\",\"ABBR_HDNG\",\"ABBR_CODE\",\"ABBR_DESC\"\r\n\
             \"UNIT\",\"\",\"\",\"\"\r\n\"TYPE\",\"X\",\"X\",\"X\"\r\n\
             \"DATA\",{hdng:?},{code:?},\"Self-declared\"\r\n"
        )
    }

    #[test]
    fn rule_16_fyi_flags_declared_nonstandard_abbr() {
        // SAMP_TYPE has a bundled standard picklist; "ZZ" is not in it but is
        // self-declared in ABBR (so the error Rule 16 stays silent) → one FYI.
        let f = run_fyi(&abbr_fixture("SAMP_TYPE", "ZZ"));
        let fyi = f
            .get(RULE_16_FYI)
            .expect("an FYI (Related to Rule 16) bucket");
        assert!(
            fyi.iter().any(|x| {
                x.desc.contains("\"ZZ\"") && x.desc.contains("not a recognised standard")
            }),
            "{fyi:?}"
        );
        // It is an FYI, never an error Rule 16 (the code IS in the file's ABBR).
        assert!(!run_fyi(&abbr_fixture("SAMP_TYPE", "ZZ")).contains_key(RULE_16));
    }

    #[test]
    fn rule_16_fyi_silent_for_standard_abbr() {
        // "U" IS a standard SAMP_TYPE code → no non-standard FYI fires.
        let f = run_fyi(&abbr_fixture("SAMP_TYPE", "U"));
        let no_nonstd = f.get(RULE_16_FYI).is_none_or(|v| {
            !v.iter()
                .any(|x| x.desc.contains("not a recognised standard"))
        });
        assert!(no_nonstd, "U is standard — must not be flagged");
    }

    #[test]
    fn rule_16_fyi_nonstandard_off_without_include_fyi() {
        // Default opts (include_fyi = false) → the FYI never fires.
        let f = run(&abbr_fixture("SAMP_TYPE", "ZZ"));
        assert!(!f.contains_key(RULE_16_FYI));
    }

    #[test]
    fn rule_16_fyi_skips_heading_without_standard_picklist() {
        // A heading with no bundled standard picklist can't be judged
        // non-standard — bespoke / DICT-defined PA headings stay quiet.
        let f = run_fyi(&abbr_fixture("XXXX_TYPE", "ZZ"));
        let none = f
            .get(RULE_16_FYI)
            .is_none_or(|v| !v.iter().any(|x| x.desc.contains("XXXX_TYPE")));
        assert!(none, "a picklist-less heading must not be flagged");
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
    fn tran_ags_unrecognised_is_warning_natively_and_fyi_in_compat() {
        // TRAN_AGS = "9.9" — present (so Rule 14 is satisfied) but not a known
        // edition. Native (warnings-on) view → a WARNING (RULE_14_WARN, visible by
        // default); FYI-only view (compat, mirroring python-ags4) → the top-level
        // FYI; errors-only default → silent on both (#203 / O-44).
        let src = "\"GROUP\",\"TRAN\"\r\n\"HEADING\",\"TRAN_AGS\"\r\n\
                   \"UNIT\",\"\"\r\n\"TYPE\",\"X\"\r\n\"DATA\",\"9.9\"\r\n";
        let w = run_warn(src);
        let warn = w.get(RULE_14_WARN).expect("Rule 14 WARNING");
        assert!(
            warn.iter()
                .any(|x| x.group == "TRAN" && x.desc.contains("9.9")),
            "{warn:?}"
        );
        // The warnings view must NOT also emit the FYI (seen once, as the warning).
        assert!(!w.contains_key(FYI), "{w:?}");
        // FYI-only (compat) → python-ags4's FYI tier, NOT the warning.
        let f = run_fyi(src);
        let fyi = f.get(FYI).expect("top-level FYI");
        assert!(
            fyi.iter()
                .any(|x| x.group == "TRAN" && x.desc.contains("9.9")),
            "{fyi:?}"
        );
        assert!(!f.contains_key(RULE_14_WARN), "{f:?}");
        // Errors-only default → silent on both tiers.
        let d = run(src);
        assert!(!d.contains_key(FYI) && !d.contains_key(RULE_14_WARN));
    }

    #[test]
    fn tran_ags_recognised_edition_is_silent_on_every_tier() {
        // A recognised edition string ("4.2") is NOT flagged on any tier — the
        // `KNOWN_TRAN_AGS.contains` early return.
        let src = "\"GROUP\",\"TRAN\"\r\n\"HEADING\",\"TRAN_AGS\"\r\n\
                   \"UNIT\",\"\"\r\n\"TYPE\",\"X\"\r\n\"DATA\",\"4.2\"\r\n";
        assert!(!run_fyi(src).contains_key(FYI));
        assert!(!run_warn(src).contains_key(RULE_14_WARN));
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
