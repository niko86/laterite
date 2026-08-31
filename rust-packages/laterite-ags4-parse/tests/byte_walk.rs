//! The one-pass byte+line+char walk: line/group semantics carried over from
//! the validator's `parse_str`, plus the NEW source-true byte offsets.
//!
//! The TRUE-offset cases mirror the Phase-0 ground-truth oracle
//! (`laterite-ags4-core/tests/byte_offset_ground_truth.rs`): where the old csv
//! `.ags.idx` index was LOOSE (CRLF records the preceding `\n`; leading blanks
//! absorbed), this walk emits the TRUE line-start — the tightening the owner
//! ratified. (Inline content, byte-for-byte the same as those fixtures.)

use laterite_ags4_parse::{ParseError, ParseOptions, parse_bytes, parse_bytes_opts, parse_str};

// PROJ block is engineered to 65 bytes (LF) — same as the Phase-0 fixtures.
const PROJ: &str =
    "\"GROUP\",\"PROJ\"\n\"HEADING\",\"X_ID\"\n\"UNIT\",\"\"\n\"TYPE\",\"ID\"\n\"DATA\",\"v\"\n";
const TRAN: &str = "\"GROUP\",\"TRAN\"\n\"HEADING\",\"TRAN_RCON\"\n\"UNIT\",\"\"\n\"TYPE\",\"X\"\n\"DATA\",\"1\"\n";

fn lf() -> Vec<u8> {
    format!("{PROJ}{TRAN}").into_bytes()
}

// --- line semantics (parity with validator parse_str) ---------------

#[test]
fn line_numbers_and_total() {
    let pf = parse_str(&format!("{PROJ}\n{TRAN}")).unwrap();
    assert_eq!(pf.group_order, vec!["PROJ", "TRAN"]);
    let proj = &pf.groups["PROJ"];
    assert_eq!(proj.group_line, 1);
    assert_eq!(proj.heading_line, Some(2));
    assert_eq!(proj.headings, vec!["X_ID"]);
    assert_eq!(proj.rows[0].line, 5);
    assert_eq!(proj.cell(0, 0), Some("v"));
    // PROJ(5) + blank(1) + TRAN(5) = 11; no phantom trailing line.
    assert_eq!(pf.total_lines, 11);
    assert_eq!(pf.groups["TRAN"].group_line, 7);
}

#[test]
fn tracks_crlf_vs_lf_per_line() {
    let src = b"\"GROUP\",\"PROJ\"\r\n\"HEADING\",\"PROJ_ID\"\n\"DATA\",\"P1\"\r\n";
    let pf = parse_bytes(src, encoding_rs::UTF_8).unwrap();
    assert!(pf.raw_lines[0].had_crlf);
    assert!(!pf.raw_lines[1].had_crlf);
    assert!(pf.raw_lines[2].had_crlf);
}

#[test]
fn final_line_without_newline_is_not_crlf() {
    let pf = parse_str("\"GROUP\",\"PROJ\"\r\n\"DATA\",\"P1\"").unwrap();
    let last = pf.raw_lines.last().unwrap();
    assert_eq!(pf.line_text(last), "\"DATA\",\"P1\"");
    assert!(!last.had_crlf);
}

#[test]
fn non_ags4_and_ags3() {
    assert!(matches!(
        parse_str("just text\nnot ags4\n"),
        Err(ParseError::NotAgs4(_))
    ));
    let ags3 = "\"**PROJ\"\r\n\"<UNITS>\",\"\",\"\"\r\n\"P001\",\"Demo\"\r\n";
    assert!(matches!(
        parse_str(ags3),
        Err(ParseError::UnsupportedEdition { .. })
    ));
}

#[test]
fn invalid_utf8_lossy_vs_reject() {
    // A lone 0xB0 INSIDE the DATA value is invalid UTF-8 (O-32). LossyReplace
    // → U+FFFD; Reject → NotUtf8.
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"\"GROUP\",\"PROJ\"\r\n\"HEADING\",\"PROJ_ID\"\r\n");
    bytes.extend_from_slice(b"\"UNIT\",\"\"\r\n\"TYPE\",\"ID\"\r\n\"DATA\",\"P1");
    bytes.push(0xB0);
    bytes.extend_from_slice(b"\"\r\n");
    let pf = parse_bytes(&bytes, encoding_rs::UTF_8).unwrap(); // validating = lossy
    assert_eq!(pf.groups["PROJ"].cell(0, 0), Some("P1\u{FFFD}"));
    assert!(
        !pf.byte_offsets_source_true,
        "a replacement clears source-true"
    );
    assert!(matches!(
        parse_bytes_opts(&bytes, ParseOptions::lean()), // lean = reject
        Err(ParseError::NotUtf8)
    ));
}

// --- source-true byte offsets (the TRUE line-starts) ----------------

fn tran_byte(bytes: &[u8]) -> u64 {
    parse_bytes(bytes, encoding_rs::UTF_8).unwrap().groups["TRAN"].group_byte
}

#[test]
fn walk_emits_true_group_offsets() {
    // LF: TRAN at 65.
    assert_eq!(tran_byte(&lf()), 65);
    // CRLF: TRAN at 70 — the TRUE start (the csv index recorded 69, the \n).
    let crlf = format!("{PROJ}{TRAN}").replace('\n', "\r\n").into_bytes();
    assert_eq!(tran_byte(&crlf), 70);
    // BOM: +3 bytes, byte_offset counts them → 68.
    let mut bom = vec![0xEF, 0xBB, 0xBF];
    bom.extend_from_slice(&lf());
    let pf = parse_bytes(&bom, encoding_rs::UTF_8).unwrap();
    assert!(pf.has_bom);
    assert_eq!(pf.groups["PROJ"].group_byte, 0); // BOM included → still 0
    assert_eq!(pf.groups["TRAN"].group_byte, 68);
    // Leading blank lines: PROJ at the TRUE 2 (the csv index recorded 0).
    let mut leading = b"\n\n".to_vec();
    leading.extend_from_slice(&lf());
    let pf = parse_bytes(&leading, encoding_rs::UTF_8).unwrap();
    assert_eq!(pf.groups["PROJ"].group_byte, 2);
    assert_eq!(pf.groups["TRAN"].group_byte, 67);
}

#[test]
fn byte_offsets_monotonic_and_in_bounds() {
    let pf = parse_bytes(&lf(), encoding_rs::UTF_8).unwrap();
    let mut prev = 0u64;
    for rl in &pf.raw_lines {
        assert!(rl.byte_offset >= prev, "raw-line offsets must be monotonic");
        assert!(rl.byte_offset < pf.total_bytes);
        prev = rl.byte_offset;
    }
    // Three-coordinate consistency: each line's offset = sum of prior line
    // byte lengths (content + its terminator).
    let bytes = lf();
    let mut expect = 0u64;
    for rl in &pf.raw_lines {
        assert_eq!(rl.byte_offset, expect, "line {} offset", rl.number);
        let term = if rl.had_crlf { 2 } else { 1 };
        expect += rl.text.len() as u64 + term;
    }
    assert_eq!(expect, bytes.len() as u64, "lines tile the whole buffer");
}

#[test]
fn slice_reparse_recovers_the_group() {
    // 6b/6f: slice [PROJ.start .. TRAN.start) reparses to exactly PROJ.
    let bytes = lf();
    let pf = parse_bytes(&bytes, encoding_rs::UTF_8).unwrap();
    // `lf()` is a tiny literal test fixture (a handful of bytes), so these
    // offsets trivially fit usize.
    #[allow(clippy::cast_possible_truncation)]
    let (s, e) = (
        pf.groups["PROJ"].group_byte as usize,
        pf.groups["TRAN"].group_byte as usize,
    );
    let re = parse_bytes(&bytes[s..e], encoding_rs::UTF_8).unwrap();
    assert_eq!(re.group_order, vec!["PROJ"]);
    assert_eq!(re.groups["PROJ"].cell(0, 0), Some("v"));
}

#[test]
fn embedded_quoted_newline_does_not_split_record() {
    // A `\n` inside a quoted field is part of the value, not a record break.
    // Mirrors the quoted_newline fixture (TRAN at 67 = 65 + 2 extra bytes).
    let proj_qn = PROJ.replace("\"v\"", "\"a\nb\"");
    let bytes = format!("{proj_qn}{TRAN}").into_bytes();
    let pf = parse_bytes(&bytes, encoding_rs::UTF_8).unwrap();
    assert_eq!(pf.groups["TRAN"].group_byte, 67);
    // The embedded newline still bumps the physical line count, but the DATA
    // value is recovered whole via the quoted-field tokenizer on its line.
    assert_eq!(pf.group_order, vec!["PROJ", "TRAN"]);
}

#[test]
fn lean_drops_raw_lines_but_keeps_byte_offsets() {
    // 6e: lean ↔ rich differ ONLY in raw_lines retention; group/row byte
    // offsets + structure are identical.
    let bytes = lf();
    let rich = parse_bytes_opts(&bytes, ParseOptions::validating()).unwrap();
    let lean = parse_bytes_opts(&bytes, ParseOptions::lean()).unwrap();
    assert!(lean.raw_lines.is_empty());
    assert!(!rich.raw_lines.is_empty());
    assert_eq!(lean.total_lines, rich.total_lines);
    assert_eq!(
        lean.groups["TRAN"].group_byte,
        rich.groups["TRAN"].group_byte
    );
    // Rows agree by VALUE; the span offsets legitimately differ (the rich
    // profile's buffer holds every line, the lean one only the DATA bodies).
    let (lp, rp) = (&lean.groups["PROJ"], &rich.groups["PROJ"]);
    assert_eq!(lp.rows.len(), rp.rows.len());
    for (r, (lr, rr)) in lp.rows.iter().zip(&rp.rows).enumerate() {
        assert_eq!((lr.line, lr.byte_offset), (rr.line, rr.byte_offset));
        assert_eq!(lr.values.len(), rr.values.len());
        for c in 0..lr.values.len() {
            assert_eq!(lp.cell(c, r), rp.cell(c, r));
        }
    }
}

#[test]
fn parse_str_equals_parse_bytes_for_ascii() {
    // 6g: for ASCII-clean input the two entry points agree on line model.
    let s = format!("{PROJ}\n{TRAN}");
    let a = parse_str(&s).unwrap();
    let b = parse_bytes(s.as_bytes(), encoding_rs::UTF_8).unwrap();
    assert_eq!(a.total_lines, b.total_lines);
    assert_eq!(a.group_order, b.group_order);
    assert_eq!(a.raw_lines, b.raw_lines);
}
