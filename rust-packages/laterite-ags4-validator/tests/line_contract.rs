//! Phase 0 (#168 parser convergence): snapshot the validator parser's LINE
//! CONTRACT — `total_lines`, the last line's `(number, had_crlf)`, `has_bom`,
//! and the per-group descriptor line numbers (`group/heading/unit/type_line`) —
//! across the EOF terminators and descriptor edge cases that the convergence's
//! `split('\n')` → `memchr` rewrite of the hottest loop must preserve byte-for-
//! byte. This is the "before" the new walk is held to: the trailing-newline /
//! phantom-final-blank-line trap (plan §3.4) and the Rule-2/2b/7 line anchors.

use laterite_ags4_validator::parse::{parse_bytes, parse_str};

fn full5(sep: &str) -> String {
    format!(
        "\"GROUP\",\"PROJ\"{sep}\"HEADING\",\"X_ID\"{sep}\"UNIT\",\"\"{sep}\"TYPE\",\"ID\"{sep}\"DATA\",\"v\"{sep}"
    )
}

/// `total_lines` + the last line's `(number, had_crlf)` must not change across
/// the four EOF terminators — in particular a trailing newline must NOT
/// fabricate a phantom final blank line (total stays 5, not 6).
#[test]
fn eof_terminators_total_lines_and_last_crlf() {
    let last = |s: &str| {
        let p = parse_str(s).unwrap();
        (
            p.total_lines,
            p.raw_lines.last().map(|l| (l.number, l.had_crlf)),
        )
    };
    assert_eq!(last(&full5("\n")), (5, Some((5, false)))); // LF-only last line
    assert_eq!(last(&full5("\r\n")), (5, Some((5, true))));

    let no_term = full5("\n").trim_end_matches('\n').to_string();
    assert_eq!(last(&no_term), (5, Some((5, false)))); // no trailing terminator

    // A lone trailing CR (no LF) is currently treated as had_crlf=true — locked.
    assert_eq!(last(&(no_term + "\r")), (5, Some((5, true))));
}

/// The per-group descriptor line numbers for missing / out-of-order descriptor
/// rows. Tuple = `(group_line, heading_line, unit_line, type_line, total_lines)`.
#[test]
fn descriptor_line_numbers() {
    let g = |s: &str| {
        let p = parse_str(s).unwrap();
        let g = p.groups.get("PROJ").unwrap();
        (
            g.group_line,
            g.heading_line,
            g.unit_line,
            g.type_line,
            p.total_lines,
        )
    };
    // missing TYPE row → type_line None
    assert_eq!(
        g("\"GROUP\",\"PROJ\"\n\"HEADING\",\"X_ID\"\n\"UNIT\",\"\"\n\"DATA\",\"v\"\n"),
        (1, Some(2), Some(3), None, 4)
    );
    // missing UNIT row → unit_line None
    assert_eq!(
        g("\"GROUP\",\"PROJ\"\n\"HEADING\",\"X_ID\"\n\"TYPE\",\"ID\"\n\"DATA\",\"v\"\n"),
        (1, Some(2), None, Some(3), 4)
    );
    // TYPE before HEADING → each recorded at its own line (type 2, heading 3)
    assert_eq!(
        g("\"GROUP\",\"PROJ\"\n\"TYPE\",\"ID\"\n\"HEADING\",\"X_ID\"\n\"DATA\",\"v\"\n"),
        (1, Some(3), None, Some(2), 4)
    );
    // two HEADING rows → the LAST occurrence wins (line 3)
    assert_eq!(
        g(
            "\"GROUP\",\"PROJ\"\n\"HEADING\",\"X_ID\"\n\"HEADING\",\"Y_ID\"\n\"UNIT\",\"\"\n\"TYPE\",\"ID\"\n\"DATA\",\"v\"\n"
        ),
        (1, Some(3), Some(4), Some(5), 6)
    );
}

/// `has_bom` is set from the raw bytes (`parse_bytes`), defaulting false for a
/// BOM-free buffer (and for `parse_str`, which never sees a BOM).
#[test]
fn has_bom_from_bytes() {
    let body = full5("\n");
    let mut bom = vec![0xEF, 0xBB, 0xBF];
    bom.extend_from_slice(body.as_bytes());
    assert!(parse_bytes(&bom, encoding_rs::UTF_8).unwrap().has_bom);
    assert!(
        !parse_bytes(body.as_bytes(), encoding_rs::UTF_8)
            .unwrap()
            .has_bom
    );
}
