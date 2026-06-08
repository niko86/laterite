//! Name-format rules: AGS4.1/4.2 Rules 19, 19a, 19b (the per-line,
//! structural parts — 19b_1). The cross-group "borrowed heading must
//! exist in the referenced GROUP" semantic (python-ags4's 19b_2/19b_3)
//! is dictionary-dependent and lands in V8.
//!
//! CLEAN-ROOM. Implemented from the AGS4 spec (`reports/AGS 4_1.pdf` &
//! `reports/AGS 4_2.pdf` §4.1.1). python-ags4 (LGPL-3.0) was read only
//! to learn its interpretation. No code/structure copied.
//!
//! Spec text (verbatim, AGS4.2 §4.1.1):
//!
//! * **Rule 19** — "A GROUP name shall not be more than 4 characters
//!   long and shall consist of uppercase letters and numbers only."
//! * **Rule 19a** — "A HEADING name shall not be more than 9
//!   characters long and shall consist of uppercase letters, numbers
//!   or the underscore character only."
//! * **Rule 19b** — "HEADING names shall start with the GROUP name
//!   followed by an underscore character. e.g. 'NGRP_HED1'. Where a
//!   HEADING refers to an existing HEADING within another GROUP, the
//!   HEADING name added to the group shall bear the same name. e.g.
//!   'CMPG_TESN' in the 'CMPT' GROUP."
//!
//! **Enforcement policy: de-facto, not literal prose** (user decision,
//! see OBSERVATIONS O-6/O-7). The spec *prose* is looser than the AGS
//! format's own published dictionary: across AGS4.1 (148 groups / 1879
//! headings) and AGS4.2 (171 / 2320), **every** group name is exactly
//! 4 uppercase letters and **every** heading's field part is 1–4
//! chars — zero exceptions. The prose's "not more than 4" / "letters
//! and numbers" allowances are never exercised. We enforce the
//! convention the dictionary actually follows (and that python-ags4
//! effectively enforces), and flag the prose as a spec shortcoming:
//!
//!   * Rule 19   — GROUP name must be **exactly 4 uppercase letters
//!     `[A-Z]`** (not "≤4", not digits).
//!   * Rule 19b  — field part (after the first `_`) must be **1–4
//!     chars**; prefix exactly 4 chars + `_`.
//!
//! Rule 19a follows the prose (≤9, `[A-Z0-9_]`) — there the prose and
//! the dictionary agree. The prefix==GROUP / valid cross-group borrow
//! check is dictionary-dependent → deferred to V8 (python's 19b_2/3).

use crate::findings::{Findings, Location, Severity, Target, add, add_at};
use crate::parse::ParsedFile;

const RULE_19: &str = "AGS Format Rule 19";
const RULE_19A: &str = "AGS Format Rule 19a";
const RULE_19B: &str = "AGS Format Rule 19b";

pub fn check(parsed: &ParsedFile, found: &mut Findings) {
    for code in &parsed.group_order {
        let g = &parsed.groups[code];

        rule_19(code, g.group_line, found);

        // 19a / 19b only have something to judge if a HEADING row
        // exists; its absence is already a Rule 4 finding (V2).
        if let Some(hl) = g.heading_line {
            if g.headings.is_empty() {
                add(
                    found,
                    RULE_19A,
                    Some(hl),
                    code,
                    "HEADING row has no field names.",
                );
            }
            for (ci, h) in g.headings.iter().enumerate() {
                rule_19a(h, ci, hl, code, found);
                rule_19b(h, ci, hl, code, found);
            }
        }
    }
}

/// Rule 19 — GROUP name: enforce the de-facto convention — **exactly
/// 4 uppercase letters `[A-Z]`**. (Spec prose says "≤4 / letters and
/// numbers" but 0 of 319 standard groups deviate; see O-6.)
fn rule_19(code: &str, line: u32, found: &mut Findings) {
    let ok = code.chars().count() == 4 && code.chars().all(|c| c.is_ascii_uppercase());
    if !ok {
        add_at(
            found,
            RULE_19,
            Some(line),
            code,
            "GROUP name must be exactly 4 uppercase letters (A–Z).",
            Location {
                target: Target::Group,
                ..Default::default()
            },
            Severity::Error,
        );
    }
}

/// Rule 19a — HEADING name: ≤9 chars, `[A-Z0-9_]` only. Length and
/// charset are reported as separate findings (a heading can fail both),
/// matching python-ags4's granularity for count parity.
fn rule_19a(h: &str, ci: usize, line: u32, group: &str, found: &mut Findings) {
    let loc = || Location {
        target: Target::Heading,
        field_index: Some(ci as u32),
        heading: Some(h.to_string()),
        ..Default::default()
    };
    if h.chars().count() > 9 {
        add_at(
            found,
            RULE_19A,
            Some(line),
            group,
            format!("Heading {h:?} is more than 9 characters long."),
            loc(),
            Severity::Error,
        );
    }
    if !h
        .chars()
        .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_')
    {
        add_at(
            found,
            RULE_19A,
            Some(line),
            group,
            format!("Heading {h:?} must contain only uppercase letters, digits, and underscore."),
            loc(),
            Severity::Error,
        );
    }
}

/// Rule 19b_1 — de-facto structural shape `AAAA_BBBB`:
///   * a **4 uppercase-letter** group-name prefix `[A-Z]{4}`,
///   * a single underscore,
///   * a **1–4 char** field part, `[A-Z0-9]` only (no further `_`).
///
/// This enforces the convention every standard heading follows
/// (0 / 4199 deviate across both dictionaries — O-7), which is
/// stricter than the spec prose. The prefix-equals-GROUP (or valid
/// cross-group borrow, e.g. `FILE_FSET` inside LOCA) check needs the
/// dictionary and is deferred to V8 (python's 19b_2/3).
fn rule_19b(h: &str, ci: usize, line: u32, group: &str, found: &mut Findings) {
    let loc = || Location {
        target: Target::Heading,
        field_index: Some(ci as u32),
        heading: Some(h.to_string()),
        ..Default::default()
    };
    let Some((prefix, field)) = h.split_once('_') else {
        add_at(
            found,
            RULE_19B,
            Some(line),
            group,
            format!("Heading {h:?} must be GROUP_FIELD (4-letter group + underscore + field)."),
            loc(),
            Severity::Error,
        );
        return;
    };

    let prefix_ok = prefix.chars().count() == 4 && prefix.chars().all(|c| c.is_ascii_uppercase());
    let field_len = field.chars().count();
    let field_ok = (1..=4).contains(&field_len)
        && field
            .chars()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit());

    if !prefix_ok || !field_ok {
        add_at(
            found,
            RULE_19B,
            Some(line),
            group,
            format!(
                "Heading {h:?} must be a 4-letter group-name prefix + underscore + a \
                 1–4 character field (uppercase letters/digits)."
            ),
            loc(),
            Severity::Error,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::parse_str;

    fn run(src: &str) -> Findings {
        let pf = parse_str(src).expect("parses");
        let mut f = Findings::new();
        check(&pf, &mut f);
        f
    }

    const CLEAN: &str = "\"GROUP\",\"PROJ\"\r\n\"HEADING\",\"PROJ_ID\",\"PROJ_NAME\"\r\n\
                          \"UNIT\",\"\",\"\"\r\n\"TYPE\",\"ID\",\"X\"\r\n\
                          \"DATA\",\"P1\",\"x\"\r\n";

    #[test]
    fn clean_names_have_no_findings() {
        assert!(run(CLEAN).is_empty());
    }

    #[test]
    fn rule_19_short_name_flagged_de_facto() {
        // "AB" — spec prose permits ≤4, but every real group is
        // exactly 4 letters. We enforce the de-facto rule → flag it.
        let src = "\"GROUP\",\"AB\"\r\n\"HEADING\",\"PROJ_ID\"\r\n\
                   \"UNIT\",\"\"\r\n\"TYPE\",\"X\"\r\n\"DATA\",\"v\"\r\n";
        assert!(
            run(src).contains_key(RULE_19),
            "short group name not flagged"
        );
    }

    #[test]
    fn rule_19_digits_flagged_de_facto() {
        // "12AB" — spec prose says "letters and numbers", but 0/319
        // standard groups contain a digit. De-facto = letters only.
        let src = "\"GROUP\",\"12AB\"\r\n\"HEADING\",\"PROJ_ID\"\r\n\
                   \"UNIT\",\"\"\r\n\"TYPE\",\"X\"\r\n\"DATA\",\"v\"\r\n";
        assert!(
            run(src).contains_key(RULE_19),
            "digit in group name not flagged"
        );
    }

    #[test]
    fn rule_19_flags_too_long_and_punctuation() {
        let long = "\"GROUP\",\"TOOLONG\"\r\n\"HEADING\",\"TOOL_X\"\r\n\
                    \"UNIT\",\"\"\r\n\"TYPE\",\"X\"\r\n\"DATA\",\"v\"\r\n";
        assert_eq!(run(long).get(RULE_19).map(Vec::len), Some(1));
        let punct = "\"GROUP\",\"LO-A\"\r\n\"HEADING\",\"PROJ_ID\"\r\n\
                     \"UNIT\",\"\"\r\n\"TYPE\",\"X\"\r\n\"DATA\",\"v\"\r\n";
        assert!(
            run(punct).contains_key(RULE_19),
            "punctuation in GROUP not flagged"
        );
    }

    #[test]
    fn rule_19a_flags_long_and_bad_charset_separately() {
        let src = "\"GROUP\",\"PROJ\"\r\n\
                   \"HEADING\",\"PROJ_TOOLONGNAME\",\"PROJ_lc\"\r\n\
                   \"UNIT\",\"\",\"\"\r\n\"TYPE\",\"X\",\"X\"\r\n\"DATA\",\"a\",\"b\"\r\n";
        let f = run(src);
        let r = f.get(RULE_19A).expect("19a");
        // PROJ_TOOLONGNAME: >9 chars (all upper) → 1 length finding.
        // PROJ_lc: lowercase → 1 charset finding. 2 total under 19a.
        assert_eq!(r.len(), 2, "{r:?}");
    }

    #[test]
    fn rule_19b_flags_missing_underscore_and_bad_prefix() {
        let no_us = "\"GROUP\",\"PROJ\"\r\n\"HEADING\",\"NOUS\"\r\n\
                     \"UNIT\",\"\"\r\n\"TYPE\",\"X\"\r\n\"DATA\",\"v\"\r\n";
        // "NOUS": no '_' → 19b. (4 chars so 19a is clean — isolates 19b.)
        assert!(run(no_us).contains_key(RULE_19B));

        let bad_prefix = "\"GROUP\",\"PROJ\"\r\n\"HEADING\",\"PRJ_ID\"\r\n\
                          \"UNIT\",\"\"\r\n\"TYPE\",\"X\"\r\n\"DATA\",\"v\"\r\n";
        // "PRJ" prefix is 3 chars, not 4 → 19b.
        assert!(
            run(bad_prefix).contains_key(RULE_19B),
            "3-char prefix not flagged"
        );
    }

    #[test]
    fn rule_19b_enforces_de_facto_field_length() {
        // "PROJ_FIELDX" — prefix 4 ok, field part "FIELDX" = 6 chars.
        // De-facto: every standard heading's field part is 1–4 → flag.
        let src = "\"GROUP\",\"PROJ\"\r\n\"HEADING\",\"PROJ_FIELDX\"\r\n\
                   \"UNIT\",\"\"\r\n\"TYPE\",\"X\"\r\n\"DATA\",\"v\"\r\n";
        let f = run(src);
        assert!(
            f.contains_key(RULE_19B),
            "long field part not flagged: {f:?}"
        );
    }

    #[test]
    fn rule_19b_accepts_borrowed_heading_shape() {
        // "FILE_FSET" inside a non-FILE group: prefix 4 letters, field
        // "FSET" (4) — structurally valid. (Whether the borrow is a
        // *real* FILE heading is a V8 dict check, not 19b_1.)
        let src = "\"GROUP\",\"LOCA\"\r\n\"HEADING\",\"LOCA_ID\",\"FILE_FSET\"\r\n\
                   \"UNIT\",\"\",\"\"\r\n\"TYPE\",\"ID\",\"X\"\r\n\"DATA\",\"BH1\",\"FS1\"\r\n";
        let f = run(src);
        assert!(
            !f.contains_key(RULE_19B),
            "valid borrowed-heading shape flagged: {f:?}"
        );
    }
}
