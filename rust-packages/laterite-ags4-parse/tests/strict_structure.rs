//! #168 Phase 5: the opt-in `strict_structure` mode.
//!
//! The leaf is LENIENT by default — it silently drops a HEADING/UNIT/TYPE/DATA
//! row that has no current GROUP, so the validator's rule engine can still run
//! and report *every* problem as a finding (a strict parser would stop at the
//! first error). Core's *read* path opts IN, so a data reader fails fast with
//! the exact terminals core's csv reader used. These pin both sides.

use laterite_ags4_parse::{ParseError, ParseOptions, parse_bytes_opts};

fn strict() -> ParseOptions {
    ParseOptions {
        strict_structure: true,
        ..ParseOptions::lean()
    }
}

#[test]
fn strict_rejects_descriptor_rows_before_group() {
    for (input, msg) in [
        (
            b"\"HEADING\",\"X\"\n".as_slice(),
            "HEADING row before any GROUP",
        ),
        (b"\"UNIT\",\"\"\n".as_slice(), "UNIT row before any GROUP"),
        (b"\"TYPE\",\"X\"\n".as_slice(), "TYPE row before any GROUP"),
        (b"\"DATA\",\"X\"\n".as_slice(), "DATA row before any GROUP"),
    ] {
        assert_eq!(
            parse_bytes_opts(input, strict()).unwrap_err(),
            ParseError::Structure(msg.to_string()),
            "strict mode must reject: {msg}"
        );
    }
}

#[test]
fn strict_rejects_group_without_code() {
    let e = parse_bytes_opts(b"\"GROUP\"\n\"HEADING\",\"X\"\n", strict()).unwrap_err();
    assert_eq!(
        e,
        ParseError::Structure("GROUP row missing group code".into())
    );
}

#[test]
fn strict_accepts_a_well_formed_file() {
    let ok = b"\"GROUP\",\"PROJ\"\n\"HEADING\",\"PROJ_ID\"\n\"UNIT\",\"\"\n\"TYPE\",\"ID\"\n\"DATA\",\"P1\"\n";
    let pf = parse_bytes_opts(ok, strict()).unwrap();
    assert_eq!(pf.group_order, vec!["PROJ"]);
}

#[test]
fn lenient_default_drops_pre_group_rows_and_keeps_parsing() {
    // A stray HEADING before the first GROUP is silently dropped; the real group
    // still parses. This is exactly what lets the validator always produce a
    // findings report instead of crashing on the first structural defect.
    let pf = parse_bytes_opts(
        b"\"HEADING\",\"STRAY\"\n\"GROUP\",\"PROJ\"\n\"HEADING\",\"PROJ_ID\"\n\"UNIT\",\"\"\n\"TYPE\",\"ID\"\n\"DATA\",\"P1\"\n",
        ParseOptions::lean(),
    )
    .unwrap();
    assert_eq!(pf.group_order, vec!["PROJ"]);
    assert_eq!(pf.groups["PROJ"].headings, vec!["PROJ_ID"]);
}

#[test]
fn lenient_default_no_group_is_generic_not_ags4() {
    // With no GROUP at all, the lenient walk yields the generic NotAgs4 — it does
    // NOT raise the strict "HEADING row before any GROUP" terminal.
    let e = parse_bytes_opts(b"\"HEADING\",\"X\"\n", ParseOptions::lean()).unwrap_err();
    assert!(matches!(e, ParseError::NotAgs4(_)), "got {e:?}");
}
