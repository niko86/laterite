//! The validator's AGS4 parse module — now a thin adapter over the shared
//! `laterite-ags4-parse` leaf (#168 Phase 2).
//!
//! The tolerant tokenizer (`split_ags_line` / `field_span`), the line-aware
//! types (`ParsedFile` / `ParsedGroup` / `RawLine` / `DataRow`), and the
//! one-pass byte/line/char walk all live in the leaf now; this module
//! `pub use`s them so every `rules/*.rs`, `fixes.rs`, binding, and `emit`
//! that names `parse::ParsedFile` / `cell` / `split_ags_line` / … compiles
//! unchanged.
//!
//! The four entry points are kept here as thin wrappers that return
//! `ValidatorError` (the leaf returns `ParseError`, mapped via
//! `From<ParseError> for ValidatorError`), so every caller's error handling
//! is unchanged. The two `parse_file*` wrappers additionally do the `fs::read`
//! the leaf deliberately doesn't. Validator callers parse via the leaf's
//! VALIDATING profile (retain raw lines + lossy decode), preserving the rich
//! 420 MB line profile and O-32's lossy-not-reject behaviour.

use std::fs;
use std::path::Path;

use crate::error::ValidatorError;

pub use laterite_ags4_parse::{
    DataRow, ParseError, ParsedFile, ParsedGroup, RawLine, field_span, line_spans, split_ags_line,
};

/// Parse an AGS4 file from disk (UTF-8). Invalid UTF-8 is decoded lossily
/// (O-32) — see the leaf's `parse_bytes`. FS wrapper: the leaf is FS-free.
pub fn parse_file(path: &Path) -> Result<ParsedFile, ValidatorError> {
    parse_file_with_encoding(path, encoding_rs::UTF_8)
}

/// Like [`parse_file`] but decodes via the given encoding (cp1252 / latin1 for
/// legacy files; the `--encoding` CLI flag + `compat.check_file(encoding=)`).
pub fn parse_file_with_encoding(
    path: &Path,
    encoding: &'static encoding_rs::Encoding,
) -> Result<ParsedFile, ValidatorError> {
    if !path.exists() {
        return Err(ValidatorError::NotFound(path.to_path_buf()));
    }
    let bytes = fs::read(path).map_err(|e| ValidatorError::Io {
        path: path.to_path_buf(),
        source: e,
    })?;
    parse_bytes(&bytes, encoding)
}

/// Parse in-memory bytes with the given encoding — the filesystem-free entry
/// the wasm wrapper and bindings use. Wraps the leaf's `parse_bytes`
/// (validating profile); `ParseError` → `ValidatorError` via `?`.
pub fn parse_bytes(
    bytes: &[u8],
    encoding: &'static encoding_rs::Encoding,
) -> Result<ParsedFile, ValidatorError> {
    Ok(laterite_ags4_parse::parse_bytes(bytes, encoding)?)
}

/// Parse from an in-memory string (unit tests + the `--dict` loader).
pub fn parse_str(text: &str) -> Result<ParsedFile, ValidatorError> {
    Ok(laterite_ags4_parse::parse_str(text)?)
}

#[cfg(test)]
mod tests {
    //! Validator-level integration: the behaviour the rules depend on, now
    //! exercised *through the re-export* (the tokenizer + walk's own unit /
    //! proptest suite lives in the leaf, `laterite-ags4-parse/tests/`).
    use super::*;

    #[test]
    fn parses_groups_with_line_numbers() {
        let src = "\"GROUP\",\"PROJ\"\r\n\
                   \"HEADING\",\"PROJ_ID\"\r\n\
                   \"UNIT\",\"\"\r\n\
                   \"TYPE\",\"ID\"\r\n\
                   \"DATA\",\"P1\"\r\n\
                   \r\n\
                   \"GROUP\",\"LOCA\"\r\n\
                   \"HEADING\",\"LOCA_ID\"\r\n\
                   \"UNIT\",\"\"\r\n\
                   \"TYPE\",\"ID\"\r\n\
                   \"DATA\",\"BH01\"\r\n";
        let pf = parse_str(src).unwrap();
        assert_eq!(pf.group_order, vec!["PROJ", "LOCA"]);
        let proj = &pf.groups["PROJ"];
        assert_eq!(proj.group_line, 1);
        assert_eq!(proj.heading_line, Some(2));
        assert_eq!(proj.headings, vec!["PROJ_ID"]);
        assert_eq!(proj.rows.len(), 1);
        assert_eq!(proj.rows[0].line, 5);
        assert_eq!(proj.rows[0].values, vec!["P1"]);
        let loca = &pf.groups["LOCA"];
        assert_eq!(loca.group_line, 7);
        assert_eq!(loca.rows[0].line, 11);
        assert_eq!(pf.total_lines, 11); // no phantom trailing blank line
    }

    #[test]
    fn tracks_crlf_vs_lf_per_line() {
        let src = "\"GROUP\",\"PROJ\"\r\n\"HEADING\",\"PROJ_ID\"\n\"DATA\",\"P1\"\r\n";
        let pf = parse_str(src).unwrap();
        assert!(pf.raw_lines[0].had_crlf, "line 1 should be CRLF");
        assert!(!pf.raw_lines[1].had_crlf, "line 2 is LF-only");
        assert!(pf.raw_lines[2].had_crlf, "line 3 should be CRLF");
    }

    #[test]
    fn final_line_without_newline_is_not_crlf() {
        let src = "\"GROUP\",\"PROJ\"\r\n\"DATA\",\"P1\""; // no trailing newline
        let pf = parse_str(src).unwrap();
        let last = pf.raw_lines.last().unwrap();
        assert_eq!(last.text, "\"DATA\",\"P1\"");
        assert!(!last.had_crlf, "unterminated final line is a Rule 2a miss");
    }

    #[test]
    fn non_ags4_input_is_rejected() {
        // The leaf's ParseError::NotAgs4 maps to ValidatorError::NotAgs4.
        assert!(matches!(
            parse_str("just some text\nnot ags4\n"),
            Err(ValidatorError::NotAgs4(_)),
        ));
    }

    #[test]
    fn ags3_is_unsupported_edition_not_generic_notags4() {
        // O-30: AGS3 markers map to ValidatorError::UnsupportedEdition.
        let ags3 = "\"**PROJ\"\r\n\
                     \"*PROJ_ID\",\"*PROJ_NAME\",\"*PROJ_AGS\"\r\n\
                     \"<UNITS>\",\"\",\"\"\r\n\
                     \"P001\",\"Demo\",\"3.1\"\r\n";
        match parse_str(ags3) {
            Err(ValidatorError::UnsupportedEdition { found }) => {
                assert!(found.contains('3'), "should name AGS3, got {found:?}");
            }
            other => panic!("expected UnsupportedEdition, got {other:?}"),
        }
        assert!(matches!(
            parse_str("nope\nstill nope\n"),
            Err(ValidatorError::NotAgs4(_)),
        ));
    }

    /// Helper: write `bytes` to a temp `.ags` file and `parse_file` it. Temp
    /// (not tests/fixtures/) because corpus-qa's e2e crawls the fixture dir
    /// and asserts hard_error==0 — a non-UTF-8 fixture would defeat the test.
    fn parse_bytes_via_file(bytes: &[u8]) -> Result<ParsedFile, ValidatorError> {
        use std::io::Write;
        let mut f = tempfile::Builder::new().suffix(".ags").tempfile().unwrap();
        f.write_all(bytes).unwrap();
        f.flush().unwrap();
        parse_file(f.path())
    }

    fn minimal_proj(value_bytes: &[u8]) -> Vec<u8> {
        let mut b = Vec::new();
        b.extend_from_slice(b"\"GROUP\",\"PROJ\"\r\n");
        b.extend_from_slice(b"\"HEADING\",\"PROJ_ID\"\r\n");
        b.extend_from_slice(b"\"UNIT\",\"\"\r\n");
        b.extend_from_slice(b"\"TYPE\",\"ID\"\r\n");
        b.extend_from_slice(b"\"DATA\",\"P1");
        b.extend_from_slice(value_bytes);
        b.extend_from_slice(b"\"\r\n");
        b
    }

    #[test]
    fn invalid_utf8_input_is_decoded_lossily_not_rejected() {
        // O-32: a lone 0xB0 → U+FFFD via the lossy (validating) path; the file
        // MUST still parse so every later rule runs.
        let pf = parse_bytes_via_file(&minimal_proj(&[0xB0])).expect("must not hard-error");
        assert_eq!(pf.group_order, vec!["PROJ"]);
        let v = &pf.groups["PROJ"].rows[0].values[0];
        assert_eq!(v, "P1\u{FFFD}", "lone 0xB0 → U+FFFD (not U+00B0)");
        assert!(!v.contains('\u{00B0}'), "must NOT be Latin-1-decoded");
    }

    #[test]
    fn valid_utf8_extended_char_is_byte_faithful() {
        let pf = parse_bytes_via_file(&minimal_proj("°".as_bytes())).unwrap();
        let v = &pf.groups["PROJ"].rows[0].values[0];
        assert_eq!(v, "P1\u{00B0}");
        assert!(!v.contains('\u{FFFD}'), "valid UTF-8 must not be mangled");
    }
}
