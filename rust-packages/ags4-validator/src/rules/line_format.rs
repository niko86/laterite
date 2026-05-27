//! Line-level lexical rules: AGS4.1 Rules 1, 3, 5, 6.
//!
//! CLEAN-ROOM. Implemented from the AGS4.1 specification
//! (`reports/AGS 4_1.pdf` §4.1.1). python-ags4 (LGPL-3.0) was read only
//! to learn how the spec's deliberately-loose wording is interpreted in
//! practice — those interpretations are facts about the AGS standard,
//! not copyrightable expression. No python-ags4 code, structure, or
//! algorithm was copied; this design operates over the pre-parsed
//! `ParsedFile` rather than python-ags4's stream-with-mutable-dict
//! approach.
//!
//! Spec text (verbatim, for the implementer):
//!
//! * **Rule 1** — "The data file shall be entirely composed of ASCII
//!   characters." In practice the AGS toolchain treats code points
//!   128–255 (extended/Latin-1, e.g. `°`) as a tolerated FYI, and only
//!   code points > 255 (smart quotes, em-dashes, …) as a hard Rule 1
//!   error. We follow that interpretation: the FYI is suppressed unless
//!   `include_fyi`, matching `ags4_cli check`'s default of errors-only.
//!   (We're UTF-8-only and reject non-UTF-8 at parse time, so a line
//!   here is always valid Unicode; the only question is code-point
//!   range.)
//! * **Rule 3** — every non-blank row must start with one of the five
//!   data descriptors GROUP / HEADING / UNIT / TYPE / DATA.
//! * **Rule 5** — every field enclosed in `"..."`; an embedded quote
//!   doubled (`""`). A line ending in `","` has an empty *unquoted*
//!   trailing field → violation.
//! * **Rule 6** — comma separator; no CR/LF inside a row. python-ags4
//!   no-ops this (says 2a/4b/5 subsume it). We add one cheap,
//!   independent check the spec explicitly demands and python skips: a
//!   stray CR (U+000D) *inside* the line content (our parser strips the
//!   legitimate line-terminator CR, so any remaining CR is embedded and
//!   illegal).

use crate::CheckOptions;
use crate::findings::{Findings, add};
use crate::parse::{ParsedFile, split_ags_line};

const RULE_1: &str = "AGS Format Rule 1";
const RULE_1_FYI: &str = "FYI (Related to Rule 1)";
const RULE_3: &str = "AGS Format Rule 3";
const RULE_5: &str = "AGS Format Rule 5";
const RULE_6: &str = "AGS Format Rule 6";

const DESCRIPTORS: [&str; 5] = ["GROUP", "HEADING", "UNIT", "TYPE", "DATA"];

pub fn check(parsed: &ParsedFile, opts: &CheckOptions, found: &mut Findings) {
    // BOM finding fires once at line 1 if the file started with EF BB BF
    // (`parse_file_with_encoding` sets the flag — see parse.rs). python-ags4
    // emits a Rule 1 error AND a FYI (Related to Rule 1) for the BOM case;
    // we mirror both. The per-line non-ASCII loop below still runs (a BOM-
    // bearing file may also have extended-ASCII content downstream).
    if parsed.has_bom {
        add(
            found,
            RULE_1,
            Some(1),
            "",
            "Has Non-ASCII character(s) (assuming that file encoding \
             is 'utf-8') and/or a byte-order-mark (BOM).",
        );
        if opts.include_fyi {
            add(
                found,
                RULE_1_FYI,
                Some(1),
                "",
                "If a BOM is present, then it is highly recommended \
                 that the file be saved without BOM encoding to avoid \
                 issues with other software.",
            );
        }
    }
    for rl in &parsed.raw_lines {
        let line = rl.text.as_str();
        let n = rl.number;

        rule_1(line, n, opts, found);

        // Blank / whitespace-only lines are legitimate group separators
        // (Rule 2a context); the structural rules below don't apply.
        if line.trim().is_empty() {
            continue;
        }

        rule_3(line, n, found);
        rule_5(line, n, found);
        rule_6(line, n, found);
    }
}

/// Rule 1 — character set.
fn rule_1(line: &str, n: u32, opts: &CheckOptions, found: &mut Findings) {
    let mut max_cp = 0u32;
    for c in line.chars() {
        let cp = c as u32;
        if cp > max_cp {
            max_cp = cp;
        }
    }
    if max_cp <= 127 {
        return; // pure ASCII — compliant
    }
    if max_cp > 255 {
        let desc = if n == 1 {
            "Line contains non-ASCII character(s) (code point > 255), or a \
             byte-order mark. Save the file as plain UTF-8 ASCII."
        } else {
            "Line contains non-ASCII character(s) (code point > 255)."
        };
        add(found, RULE_1, Some(n), "", desc);
    } else if opts.include_fyi {
        // 128..=255: extended/Latin-1. Tolerated; informational only.
        add(
            found,
            RULE_1_FYI,
            Some(n),
            "",
            "Line contains extended-ASCII character(s) (code point 128–255). \
             Permitted but plain ASCII is preferred.",
        );
    }
}

/// Rule 3 — leading data descriptor.
fn rule_3(line: &str, n: u32, found: &mut Findings) {
    let fields = split_ags_line(line);
    let first = fields.first().map(String::as_str).unwrap_or("");
    if !DESCRIPTORS.contains(&first) {
        add(
            found,
            RULE_3,
            Some(n),
            "",
            "Row does not start with a valid data descriptor \
             (GROUP / HEADING / UNIT / TYPE / DATA).",
        );
    }
}

/// Rule 5 — every field double-quote enclosed; embedded quotes doubled.
///
/// python-ags4 emits two distinct messages depending on which Rule 5
/// sub-violation tripped — and its own test_rule_5_{1,2} asserts on
/// the exact wording. We classify the deviation type and emit
/// matching distinct descs so `compat.check_file` can parity-match
/// without a translator branch.
fn rule_5(line: &str, n: u32, found: &mut Findings) {
    let desc = match check_quoting(line) {
        QuotingDeviation::Ok => return,
        QuotingDeviation::EmbeddedQuote => {
            "Row has an embedded double-quote that is not doubled \
             (\"\") — a quote inside a data field must be written as \
             two consecutive quotes."
        }
        QuotingDeviation::NotEnclosed => "Row has field(s) not enclosed in double quotes.",
    };
    add(found, RULE_5, Some(n), "", desc);
}

#[derive(Debug, PartialEq, Eq)]
enum QuotingDeviation {
    Ok,
    /// A field's closing quote was followed by non-comma, non-EOL text
    /// — the author probably intended an embedded quote that should
    /// have been doubled (`""`). Distinguishes rule5-1 (`"ACME "Gas
    /// Works" Redevelopment"`) from rule5-2 (open field never closes).
    EmbeddedQuote,
    /// A field opened but never closed, or the line doesn't start with
    /// a quote at all. rule5-2 territory.
    NotEnclosed,
}

/// Rule 6 — embedded carriage return inside row content. (Comma
/// separation and inter-field newlines surface via Rule 5 / structural
/// rules; this is the one independent Rule 6 check worth making.)
fn rule_6(line: &str, n: u32, found: &mut Findings) {
    if line.contains('\r') {
        add(
            found,
            RULE_6,
            Some(n),
            "",
            "Row contains an embedded carriage return (ASCII 13). \
             CR/LF are not allowed within or between data variables.",
        );
    }
}

/// Strict AGS4 quoting grammar:  `"f"` ( `,` `"f"` )*  where each `f`
/// may contain `""` (escaped quote) but no lone `"`. Returns the
/// **deviation type** on the first violation so Rule 5 can emit the
/// right desc — see [`QuotingDeviation`]. Strict counterpart to the
/// tolerant `split_ags_line` (same grammar, opposite intent).
fn check_quoting(line: &str) -> QuotingDeviation {
    let mut chars = line.chars().peekable();
    loop {
        // Each field must open with a quote.
        if chars.next() != Some('"') {
            return QuotingDeviation::NotEnclosed;
        }
        // Consume field body until the closing quote.
        loop {
            match chars.next() {
                None => return QuotingDeviation::NotEnclosed, // unterminated
                Some('"') => {
                    if chars.peek() == Some(&'"') {
                        chars.next(); // doubled quote — stays in field
                    } else {
                        break; // closing quote
                    }
                }
                Some(_) => {}
            }
        }
        // After a closing quote: either end-of-line or a comma + more.
        // A non-comma non-EOL char here indicates the previous quote
        // wasn't actually a field-closer — the author probably embedded
        // a lone `"` mid-field and meant to double it. python-ags4 emits
        // a distinct message for this case (test_rule_5_1) vs the
        // "not enclosed" case (test_rule_5_2).
        match chars.next() {
            None => return QuotingDeviation::Ok,
            Some(',') => continue,
            Some(_) => return QuotingDeviation::EmbeddedQuote,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::parse_str;

    fn run(src: &str, fyi: bool) -> Findings {
        let pf = parse_str(src).expect("fixture parses");
        let opts = CheckOptions {
            include_fyi: fyi,
            ..Default::default()
        };
        let mut f = Findings::new();
        check(&pf, &opts, &mut f);
        f
    }

    // A minimal compliant 2-group file every test can prepend so the
    // parser accepts the input (needs ≥1 GROUP).
    const HEAD: &str = "\"GROUP\",\"PROJ\"\r\n\"HEADING\",\"PROJ_ID\"\r\n\
                        \"UNIT\",\"\"\r\n\"TYPE\",\"ID\"\r\n\"DATA\",\"P1\"\r\n";

    #[test]
    fn clean_file_has_no_line_findings() {
        let f = run(HEAD, true);
        assert!(f.is_empty(), "unexpected findings: {f:?}");
    }

    #[test]
    fn rule_1_flags_codepoint_over_255_as_error() {
        // U+2019 RIGHT SINGLE QUOTATION MARK (a "smart quote").
        let src = format!("{HEAD}\"GROUP\",\"NOTE\u{2019}S\"\r\n");
        let pf = parse_str(&src).unwrap();
        let mut f = Findings::new();
        check(&pf, &CheckOptions::default(), &mut f);
        let r1 = f.get(RULE_1).expect("Rule 1 error");
        assert_eq!(r1.len(), 1);
        assert_eq!(r1[0].line, Some(6));
        assert!(!f.contains_key(RULE_1_FYI));
    }

    #[test]
    fn rule_1_extended_ascii_is_fyi_only_and_suppressed_by_default() {
        // U+00B0 DEGREE SIGN — extended ASCII (176).
        let src = format!("{HEAD}\"GROUP\",\"TEMP\u{00b0}\"\r\n");

        // Default: FYI suppressed, no Rule 1 error.
        let pf = parse_str(&src).unwrap();
        let mut f = Findings::new();
        check(&pf, &CheckOptions::default(), &mut f);
        assert!(
            !f.contains_key(RULE_1),
            "extended ASCII must not be a Rule 1 error"
        );
        assert!(!f.contains_key(RULE_1_FYI), "FYI suppressed by default");

        // With include_fyi: surfaces as FYI, still not an error.
        let f2 = run(&src, true);
        assert!(!f2.contains_key(RULE_1));
        assert_eq!(f2[RULE_1_FYI].len(), 1);
    }

    #[test]
    fn rule_3_flags_bad_descriptor() {
        let src = format!("{HEAD}\"WIBBLE\",\"x\"\r\n");
        let f = run(&src, false);
        let r3 = f.get(RULE_3).expect("Rule 3");
        assert_eq!(r3.len(), 1);
        assert_eq!(r3[0].line, Some(6));
    }

    #[test]
    fn rule_3_ignores_blank_separator_lines() {
        let src = format!(
            "{HEAD}\r\n\"GROUP\",\"LOCA\"\r\n\"HEADING\",\"LOCA_ID\"\r\n\
                           \"UNIT\",\"\"\r\n\"TYPE\",\"ID\"\r\n\"DATA\",\"BH1\"\r\n"
        );
        let f = run(&src, false);
        assert!(!f.contains_key(RULE_3), "blank line wrongly flagged: {f:?}");
    }

    #[test]
    fn rule_5_flags_unquoted_field() {
        let src = format!("{HEAD}\"DATA\",unquoted\r\n");
        let f = run(&src, false);
        assert!(f.contains_key(RULE_5));
        assert_eq!(f[RULE_5][0].line, Some(6));
    }

    #[test]
    fn rule_5_flags_trailing_empty_unquoted_field() {
        // Ends in `","` — a trailing unquoted empty field.
        let src = format!("{HEAD}\"DATA\",\"x\",\r\n");
        let f = run(&src, false);
        assert!(f.contains_key(RULE_5), "trailing `\",\"` not caught: {f:?}");
    }

    #[test]
    fn rule_5_accepts_doubled_embedded_quote() {
        let src = format!("{HEAD}\"DATA\",\"he said \"\"hi\"\"\"\r\n");
        let f = run(&src, false);
        assert!(
            !f.contains_key(RULE_5),
            "valid doubled quote flagged: {f:?}"
        );
    }

    #[test]
    fn rule_6_flags_embedded_cr() {
        // Build raw lines with an embedded CR that is NOT the line
        // terminator. parse_str strips a *trailing* \r; an interior one
        // survives in raw_lines.text.
        let src = format!("{HEAD}\"DATA\",\"a\rb\"\r\n");
        let f = run(&src, false);
        assert!(f.contains_key(RULE_6), "embedded CR not caught: {f:?}");
        assert_eq!(f[RULE_6][0].line, Some(6));
    }
}
