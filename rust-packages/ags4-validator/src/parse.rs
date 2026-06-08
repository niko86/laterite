//! Line-aware AGS4 parser.
//!
//! Clean-room — written from the AGS4.1 spec (reports/AGS 4_1.pdf
//! §4.1.1 Rules 1–7), not derived from python-ags4. The validator's
//! findings need 1-indexed source line numbers for every per-row
//! violation, so unlike `ags5db`'s `ags4_codec` (which discards
//! them) this parser retains:
//!
//!   * every raw line + its number (line-level Rules 1/3/5/6),
//!   * which line carried GROUP / HEADING / UNIT / TYPE per group
//!     (structural Rules 2/2b),
//!   * each DATA row's line + positional values.
//!
//! The split is deliberately *tolerant*: a malformed line yields
//! best-effort fields rather than aborting, because reporting the
//! malformation is a rule's job, not the parser's. Only a file with
//! zero GROUP rows is rejected outright (it isn't AGS4 at all).

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use crate::error::ValidatorError;

#[derive(Debug, Clone)]
pub struct RawLine {
    /// 1-indexed, matching how editors + the AGS4 validator report.
    pub number: u32,
    /// Line content with the trailing CR/LF stripped. Empty string for
    /// blank separator lines.
    pub text: String,
    /// Whether this line was CR+LF terminated in the source. `false`
    /// for an LF-only line or the final line if the file has no
    /// trailing newline — both are AGS4 Rule 2a violations. Captured
    /// here because `text` has the CR stripped, losing the evidence.
    pub had_crlf: bool,
}

#[derive(Debug, Clone)]
pub struct DataRow {
    pub line: u32,
    /// Field values *after* the leading tag, unquoted + unescaped,
    /// positionally aligned with `ParsedGroup::headings`.
    pub values: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ParsedGroup {
    pub code: String,
    pub group_line: u32,
    pub heading_line: Option<u32>,
    pub unit_line: Option<u32>,
    pub type_line: Option<u32>,
    pub headings: Vec<String>,
    pub units: Vec<String>,
    pub types: Vec<String>,
    pub rows: Vec<DataRow>,
}

#[derive(Debug, Clone)]
pub struct ParsedFile {
    /// First-seen wins on duplicate GROUP codes (Rule 2 flags the dup
    /// separately); the map holds the first occurrence.
    pub groups: BTreeMap<String, ParsedGroup>,
    /// GROUP codes in the order they appear in the file.
    pub group_order: Vec<String>,
    /// Every line, including blanks + structurally-unattached lines, so
    /// line-level rules don't have to re-read the file.
    pub raw_lines: Vec<RawLine>,
    pub total_lines: u32,
    /// True iff the source file began with a UTF-8 byte-order mark
    /// (`EF BB BF`). Set by `parse_file_with_encoding`; defaults to
    /// `false` for `parse_str` (no BOM in an in-memory string the
    /// caller built). Rule 1 emits a dedicated finding + FYI when set.
    pub has_bom: bool,
}

/// Parse an AGS4 file from disk.
///
/// Invalid UTF-8 is decoded *lossily* (WHATWG maximal-subpart →
/// `U+FFFD`) rather than rejected. This is the exact behavioural twin
/// of python-ags4's `open(…, encoding='utf-8', errors="replace")`
/// (`AGS4.py:771`): a real default `ags4 check` invocation never
/// hard-fails on encoding — it lets the replacement char surface as a
/// Rule 1 finding (`U+FFFD` is code point 65533 > 255, so `rule_1`'s
/// non-ASCII arm reports it, just as python does). Refusing the input
/// outright was the *only* real divergence from the reference and
/// turned 12/12503 dogfood files into zero-rules-evaluated black
/// holes; decoding lossily keeps the structure parseable so every
/// later rule still runs. Valid UTF-8 takes `from_utf8_lossy`'s
/// `Cow::Borrowed` fast path — byte-identical, no char-by-char
/// rebuild. See O-32.
pub fn parse_file(path: &Path) -> Result<ParsedFile, ValidatorError> {
    parse_file_with_encoding(path, encoding_rs::UTF_8)
}

/// Like [`parse_file`] but decodes via the given encoding. UTF-8 is
/// the historical default (`from_utf8_lossy` semantics); pass other
/// encodings for legacy files (`cp1252` / `latin1`). Stage 7b: this
/// is the shared library entry for the `--encoding` CLI flag and
/// `compat.check_file(encoding=...)`. `encoding_rs::Encoding::decode`
/// strips the BOM where present (matching python's `open(encoding=)`
/// behaviour) and inserts U+FFFD on undefined bytes (matching
/// python's `errors='replace'`).
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

/// Parse AGS4 from in-memory bytes with the given encoding — the
/// filesystem-free core of [`parse_file_with_encoding`]. Used by
/// callers that already hold the bytes and have no path: the browser
/// **wasm** wrapper (no filesystem at all) reads the user's file into
/// memory and hands it straight here. BOM sniff + lossy decode are
/// identical to the on-disk path, so Rule 1 (and every later rule)
/// behaves the same whether the bytes came from disk or a `FileReader`.
pub fn parse_bytes(
    bytes: &[u8],
    encoding: &'static encoding_rs::Encoding,
) -> Result<ParsedFile, ValidatorError> {
    // BOM sniff before decode: encoding_rs::decode strips the BOM
    // transparently (correct UTF-16/UTF-8 behaviour), so we have to
    // notice it here if Rule 1 is to emit a BOM-specific finding.
    // Only the UTF-8 BOM is meaningful for AGS4 (UTF-16-encoded AGS4
    // is out of spec; encoding_rs would still strip the BOM in those
    // cases, but the file would also fail other rules first).
    let has_bom = bytes.starts_with(&[0xEF, 0xBB, 0xBF]);
    let (text, _enc, _had_replacements) = encoding.decode(bytes);
    let mut parsed = parse_str(&text)?;
    parsed.has_bom = has_bom;
    Ok(parsed)
}

/// Parse from an in-memory string (used by unit tests + the `--dict`
/// loader). Splits on `\n`; a trailing `\r` is trimmed so CRLF and LF
/// files parse identically (Rule 6 checks the line *ending* separately
/// off `raw_lines` if needed).
pub fn parse_str(text: &str) -> Result<ParsedFile, ValidatorError> {
    let mut groups: BTreeMap<String, ParsedGroup> = BTreeMap::new();
    let mut group_order: Vec<String> = Vec::new();
    let mut raw_lines: Vec<RawLine> = Vec::new();
    let mut current: Option<String> = None;
    // AGS3 uses `**GROUP` / `*HEADING` / `<UNITS>` / `<CONT>` — none
    // of which are AGS4 `GROUP` rows, so an AGS3 file otherwise looks
    // like "no GROUP rows". Track the unambiguous AGS3 markers so we
    // can report it as the recognised (unsupported) edition it is,
    // not a generic "not AGS4" (O-30).
    let mut looks_ags3 = false;

    // Materialise segments once (split('\n') is O(n); recomputing its
    // length per iteration was O(n²)).
    let segments: Vec<&str> = text.split('\n').collect();
    let n_segments = segments.len();

    for (i, raw) in segments.iter().enumerate() {
        let number = (i + 1) as u32;
        let had_crlf = raw.ends_with('\r');
        let line = raw.strip_suffix('\r').unwrap_or(raw);

        // `split('\n')` yields a trailing "" for a file ending in \n.
        // Don't fabricate a phantom final blank line.
        if i + 1 == n_segments && line.is_empty() && text.ends_with('\n') {
            break;
        }
        raw_lines.push(RawLine {
            number,
            text: line.to_string(),
            had_crlf,
        });

        if line.trim().is_empty() {
            continue; // group separator / blank
        }
        let fields = split_ags_line(line);
        if fields.is_empty() {
            continue;
        }
        let tag = fields[0].as_str();
        let rest = || fields[1..].to_vec();

        match tag {
            "GROUP" => {
                let code = fields.get(1).cloned().unwrap_or_default();
                if !groups.contains_key(&code) {
                    group_order.push(code.clone());
                    groups.insert(
                        code.clone(),
                        ParsedGroup {
                            code: code.clone(),
                            group_line: number,
                            heading_line: None,
                            unit_line: None,
                            type_line: None,
                            headings: Vec::new(),
                            units: Vec::new(),
                            types: Vec::new(),
                            rows: Vec::new(),
                        },
                    );
                }
                // A duplicate GROUP still re-points "current" so its
                // following HEADING/DATA rows attach somewhere; Rule 2
                // reports the duplicate from group_order vs groups.
                current = Some(code);
            }
            "HEADING" => {
                if let Some(g) = current.as_ref().and_then(|c| groups.get_mut(c)) {
                    g.heading_line = Some(number);
                    g.headings = rest();
                }
            }
            "UNIT" => {
                if let Some(g) = current.as_ref().and_then(|c| groups.get_mut(c)) {
                    g.unit_line = Some(number);
                    g.units = rest();
                }
            }
            "TYPE" => {
                if let Some(g) = current.as_ref().and_then(|c| groups.get_mut(c)) {
                    g.type_line = Some(number);
                    g.types = rest();
                }
            }
            "DATA" => {
                if let Some(g) = current.as_ref().and_then(|c| groups.get_mut(c)) {
                    g.rows.push(DataRow {
                        line: number,
                        values: rest(),
                    });
                }
            }
            // Unknown tag: keep the raw line (Rule 3 flags it later),
            // don't attach structurally. `**X` (AGS3 group),
            // `<UNITS>`/`<CONT>` (AGS3 markers) are unambiguous AGS3.
            _ => {
                if tag.starts_with("**") || tag == "<UNITS>" || tag == "<CONT>" {
                    looks_ags3 = true;
                }
            }
        }
    }

    if group_order.is_empty() {
        if looks_ags3 {
            // A recognised edition we deliberately don't support —
            // surface it as such, not a vague "not AGS4" (O-30).
            return Err(ValidatorError::UnsupportedEdition {
                found: "3.x (AGS3 format)".to_string(),
            });
        }
        return Err(ValidatorError::NotAgs4("no GROUP rows found".to_string()));
    }

    let total_lines = raw_lines.len() as u32;
    Ok(ParsedFile {
        groups,
        group_order,
        raw_lines,
        total_lines,
        has_bom: false, // parse_str — caller built the string; set by
                        // parse_file_with_encoding when we see the raw bytes.
    })
}

/// Split one AGS4 line into fields. AGS4 wraps every field in double
/// quotes and doubles embedded quotes (`""`). Tolerant: an unquoted
/// field is read up to the next comma; an unterminated quote consumes
/// to end-of-line. Returns owned, unescaped values.
pub fn split_ags_line(line: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut chars = line.chars().peekable();

    loop {
        // Skip nothing — we're positioned at the start of a field.
        match chars.peek() {
            None => {
                // Trailing comma produced an implicit empty final field
                // only if the line had at least one field already and
                // ended on a separator; handled by the comma arm below.
                break;
            }
            Some('"') => {
                chars.next(); // consume opening quote
                let mut field = String::new();
                loop {
                    match chars.next() {
                        None => break, // unterminated — tolerate
                        Some('"') => {
                            if chars.peek() == Some(&'"') {
                                chars.next();
                                field.push('"'); // escaped quote
                            } else {
                                break; // closing quote
                            }
                        }
                        Some(c) => field.push(c),
                    }
                }
                out.push(field);
                // Expect a comma or end after a closing quote.
                match chars.peek() {
                    Some(',') => {
                        chars.next();
                        continue;
                    }
                    _ => {
                        // Skip any stray chars until comma/EOL (lenient).
                        while let Some(&c) = chars.peek() {
                            if c == ',' {
                                chars.next();
                                break;
                            }
                            chars.next();
                        }
                        if chars.peek().is_none() {
                            break;
                        }
                    }
                }
            }
            Some(_) => {
                // Unquoted field — read to next comma.
                let mut field = String::new();
                while let Some(&c) = chars.peek() {
                    if c == ',' {
                        break;
                    }
                    field.push(c);
                    chars.next();
                }
                out.push(field);
                if chars.peek() == Some(&',') {
                    chars.next();
                    continue;
                }
                break;
            }
        }
    }
    out
}

/// Char-offset span of the **content inside the quotes** of the raw-line
/// field at position `field_index + 1` — the `+1` skips the leading
/// `DATA`/`HEADING` tag, so `field_index` is the *tag-stripped* index the
/// rules carry. Returns a half-open `(start, end)` pair counted in
/// `char`s (Unicode scalars), **not** bytes, so a multibyte line lights
/// the right columns in a JS `Array.from`/spread slice.
///
/// **Span convention: the value BETWEEN the surrounding quotes** —
/// the quotes themselves and the comma delimiter are excluded. For an
/// unquoted field the span is the raw field content (commas excluded).
/// `None` if the line has fewer fields than requested.
///
/// Mirrors [`split_ags_line`]'s quote / escaped-`""` state machine, but
/// tracks the running `char` position and records the inner-content span
/// of the target field rather than materialising every field's value.
pub fn field_span(line: &str, field_index: u32) -> Option<(u32, u32)> {
    let target = field_index as usize + 1; // +1 skips the leading tag.
    let mut pos: u32 = 0; // running char offset into `line`.
    let mut field = 0usize; // which field we're about to read.
    let mut chars = line.chars().peekable();

    loop {
        match chars.peek().copied() {
            None => return None, // ran out of fields before reaching target
            Some('"') => {
                chars.next();
                pos += 1;
                let start = pos; // first char inside the quotes
                let mut end = pos;
                loop {
                    match chars.next() {
                        None => break, // unterminated — span to EOL content
                        Some('"') => {
                            if chars.peek() == Some(&'"') {
                                chars.next();
                                pos += 2;
                                end = pos; // escaped quote stays inside
                            } else {
                                pos += 1; // closing quote
                                break;
                            }
                        }
                        Some(_) => {
                            pos += 1;
                            end = pos;
                        }
                    }
                }
                if field == target {
                    return Some((start, end));
                }
                // Skip stray chars up to and including the next comma.
                while let Some(&c) = chars.peek() {
                    chars.next();
                    pos += 1;
                    if c == ',' {
                        break;
                    }
                }
                field += 1;
            }
            Some(_) => {
                // Unquoted field — content is everything up to the comma.
                let start = pos;
                let mut end = pos;
                while let Some(&c) = chars.peek() {
                    if c == ',' {
                        break;
                    }
                    chars.next();
                    pos += 1;
                    end = pos;
                }
                if field == target {
                    return Some((start, end));
                }
                if chars.peek() == Some(&',') {
                    chars.next();
                    pos += 1;
                }
                field += 1;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `field_span` must point at the inner value of the field at
    /// `field_index + 1`, in CHAR offsets, matching `split_ags_line`'s
    /// notion of each field's content. The cases mirror
    /// `splits_quoted_fields_and_unescapes` plus an empty field and a
    /// multibyte (`°`) line proving char-not-byte counting.
    #[test]
    fn field_span_points_at_inner_value_char_offsets() {
        // `"DATA","BH01","100.50"` — field_index 0 is BH01, 1 is 100.50.
        let line = r#""DATA","BH01","100.50""#;
        let chars: Vec<char> = line.chars().collect();
        let slice = |(s, e): (u32, u32)| chars[s as usize..e as usize].iter().collect::<String>();
        assert_eq!(slice(field_span(line, 0).unwrap()), "BH01");
        assert_eq!(slice(field_span(line, 1).unwrap()), "100.50");
        assert_eq!(field_span(line, 2), None); // only 3 fields incl tag

        // Escaped `""` stays inside the inner span. The span is over the
        // RAW line, so the doubled quotes are part of the lit region
        // (highlighting paints what's physically on the line, not the
        // unescaped value).
        let esc = r#""DATA","he said ""hi""","x""#;
        let echars: Vec<char> = esc.chars().collect();
        let span = field_span(esc, 0).unwrap();
        let got: String = echars[span.0 as usize..span.1 as usize].iter().collect();
        assert_eq!(got, r#"he said ""hi"""#);

        // Empty field — zero-width span just inside the quotes.
        let empty = r#""HEADING","",""#;
        let (s, e) = field_span(empty, 0).unwrap();
        assert_eq!(s, e, "empty field is a zero-width span");

        // Multibyte: a `°` (2 UTF-8 bytes, 1 char) ahead of the target
        // field must shift the span by ONE char, not two bytes.
        let mb = r#""DATA","°C","42""#;
        let mchars: Vec<char> = mb.chars().collect();
        let mslice = |(s, e): (u32, u32)| mchars[s as usize..e as usize].iter().collect::<String>();
        assert_eq!(mslice(field_span(mb, 0).unwrap()), "°C");
        assert_eq!(mslice(field_span(mb, 1).unwrap()), "42");
    }

    #[test]
    fn splits_quoted_fields_and_unescapes() {
        assert_eq!(
            split_ags_line(r#""DATA","BH01","100.50""#),
            vec!["DATA", "BH01", "100.50"],
        );
        assert_eq!(
            split_ags_line(r#""DATA","he said ""hi""","x""#),
            vec!["DATA", r#"he said "hi""#, "x"],
        );
        assert_eq!(split_ags_line(r#""HEADING","",""#), vec!["HEADING", "", ""],);
    }

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
        // No phantom trailing blank line.
        assert_eq!(pf.total_lines, 11);
    }

    #[test]
    fn tracks_crlf_vs_lf_per_line() {
        // Line 1 CRLF, line 2 LF-only, then a CRLF DATA row.
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
        assert!(matches!(
            parse_str("just some text\nnot ags4\n"),
            Err(ValidatorError::NotAgs4(_)),
        ));
    }

    #[test]
    fn ags3_is_unsupported_edition_not_generic_notags4() {
        // O-30: an AGS3 file (`**GROUP` / `<UNITS>`) is a recognised
        // edition we deliberately refuse — surface it as such, not
        // the vague NotAgs4("no GROUP rows found"). (No fixture file:
        // the corpus-QA e2e crawls tests/fixtures/ and asserts that
        // corpus is hard-error-free — keep AGS3 a parse-level test.)
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
        // A genuinely structureless file is still the generic error.
        assert!(matches!(
            parse_str("nope\nstill nope\n"),
            Err(ValidatorError::NotAgs4(_)),
        ));
    }

    /// Helper: write `bytes` to a temp `.ags` file and `parse_file` it.
    /// Temp (not tests/fixtures/) because corpus-qa's e2e crawls the
    /// fixture dir and asserts hard_error==0 — a non-UTF-8 fixture
    /// would also defeat the very behaviour under test.
    fn parse_bytes(bytes: &[u8]) -> Result<ParsedFile, ValidatorError> {
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
        // O-32: a lone 0xB0 (cp1252 `°`) is a UTF-8 continuation byte
        // with no lead → invalid → `from_utf8_lossy` substitutes
        // U+FFFD, exactly as python-ags4's `errors="replace"` does.
        // The file MUST still parse (no NotUtf8 black hole) so every
        // later rule runs; the replacement char is what Rule 1 then
        // reports (cp 65533 > 255).
        let pf = parse_bytes(&minimal_proj(&[0xB0])).expect("must not hard-error");
        assert_eq!(pf.group_order, vec!["PROJ"]);
        let v = &pf.groups["PROJ"].rows[0].values[0];
        assert_eq!(v, "P1\u{FFFD}", "lone 0xB0 → U+FFFD (not U+00B0)");
        assert!(!v.contains('\u{00B0}'), "must NOT be Latin-1-decoded");
    }

    #[test]
    fn valid_utf8_extended_char_is_byte_faithful() {
        // Byte-identical guard: a *correctly* UTF-8-encoded `°`
        // (0xC2 0xB0) is valid → `from_utf8_lossy` takes the
        // `Cow::Borrowed` fast path, no re-encode → the value is
        // exactly "P1°" (U+00B0, ≤255 → Rule 1 FYI/clean, not the
        // >255 error). This is the correct-encoding-is-rewarded side
        // of O-32.
        let pf = parse_bytes(&minimal_proj("°".as_bytes())).unwrap();
        let v = &pf.groups["PROJ"].rows[0].values[0];
        assert_eq!(v, "P1\u{00B0}");
        assert!(!v.contains('\u{FFFD}'), "valid UTF-8 must not be mangled");
    }
}

/// Property-based tests for the AGS4 field splitter + span tracker.
///
/// `split_ags_line` is the lenient field parser every rule reads through;
/// `field_span` mirrors its state machine to light up editor columns. The
/// properties pin the two contracts the examples only sample: the
/// quote/escape splitter is the exact inverse of a well-formed encoder
/// (round-trip identity), `field_span` counts in stable Unicode CHARS
/// regardless of line terminator, and neither panics on arbitrary text.
#[cfg(test)]
mod proptest_suite {
    use super::*;
    use proptest::prelude::*;

    /// Encode one field the way a *well-formed* AGS4 writer does: wrap in
    /// double quotes, double every embedded quote. Inverse of the quoted
    /// branch of `split_ags_line`.
    fn encode_field(field: &str) -> String {
        format!("\"{}\"", field.replace('"', "\"\""))
    }

    /// Join fields into a well-formed AGS4 line (no trailing comma, every
    /// field quoted). The empty-vec case yields the empty string, which
    /// `split_ags_line` reads back as `[]`.
    fn encode_line(fields: &[String]) -> String {
        fields
            .iter()
            .map(|f| encode_field(f))
            .collect::<Vec<_>>()
            .join(",")
    }

    /// Field values for the round-trip generator. Excludes the line
    /// terminators `\r` / `\n` (a real AGS line carries none — they ARE
    /// the line break), but deliberately includes quotes, commas, empty
    /// strings, ASCII control-ish chars, and multibyte UTF-8 so the
    /// quote/escape and comma-delimiting paths are all exercised.
    fn field_value() -> impl Strategy<Value = String> {
        prop::collection::vec(
            prop_oneof![
                // common AGS payload chars + the two delimiters
                prop::char::range(' ', '~'),
                Just('"'),
                Just(','),
                // multibyte coverage (Latin-1 supplement, CJK, emoji)
                prop::char::range('\u{00a1}', '\u{017f}'),
                prop::char::range('\u{4e00}', '\u{4e80}'),
                Just('°'),
                Just('🦀'),
            ]
            // No CR/LF — those terminate a line, not live inside a field.
            .prop_filter("no line terminators", |c| *c != '\r' && *c != '\n'),
            0..12,
        )
        .prop_map(|cs| cs.into_iter().collect())
    }

    proptest! {
        /// Round-trip identity: encoding an arbitrary `Vec<String>` of
        /// field values into a well-formed AGS4 line and splitting it back
        /// recovers the EXACT originals — the splitter is the precise
        /// inverse of the quote+escape encoder. Covers empty fields,
        /// embedded quotes/commas, and multibyte UTF-8.
        #[test]
        fn split_ags_line_round_trips_well_formed(fields in prop::collection::vec(field_value(), 0..8)) {
            let line = encode_line(&fields);
            let got = split_ags_line(&line);
            prop_assert_eq!(got, fields, "line={:?}", line);
        }

        /// `split_ags_line` never panics on arbitrary `&str` — including
        /// unterminated quotes, stray bytes after a closing quote, and
        /// adversarial multibyte. (The lenient parser tolerates malformed
        /// input by design; this guards that tolerance can't crash.)
        #[test]
        fn split_ags_line_never_panics(line in ".*") {
            let _ = split_ags_line(&line);
            prop_assert!(true);
        }

        /// `field_span` never panics on arbitrary `(line, index)`.
        #[test]
        fn field_span_never_panics(line in ".*", idx in 0u32..32) {
            let _ = field_span(&line, idx);
            prop_assert!(true);
        }

        /// Char-offset stability across line terminators: the span for a
        /// given field is IDENTICAL whether the encoded line ends in CRLF,
        /// LF, or has no terminator. The trailing terminator is past every
        /// field, so it must never shift an inner-content span. (Indices
        /// are tag-stripped: `field_index` 0 is the SECOND encoded field.)
        #[test]
        fn field_span_stable_across_line_terminator(
            fields in prop::collection::vec(field_value(), 2..6),
            idx in 0u32..4,
        ) {
            let base = encode_line(&fields);
            prop_assume!((idx as usize + 1) < fields.len());

            let bare = field_span(&base, idx);
            let with_lf = field_span(&format!("{base}\n"), idx);
            let with_crlf = field_span(&format!("{base}\r\n"), idx);

            prop_assert_eq!(bare, with_lf, "LF shifted the span");
            prop_assert_eq!(bare, with_crlf, "CRLF shifted the span");
        }

        /// `field_span` agrees with `split_ags_line`: for a well-formed
        /// line, slicing the RAW line by the returned char span yields the
        /// field's content with embedded quotes RE-DOUBLED (the span is
        /// over the physical line, so an escaped `""` shows as two chars).
        /// Re-splitting that one-field reconstruction recovers the value.
        #[test]
        fn field_span_inner_slice_matches_field(
            fields in prop::collection::vec(field_value(), 2..6),
            idx in 0u32..4,
        ) {
            prop_assume!((idx as usize + 1) < fields.len());
            let line = encode_line(&fields);
            let chars: Vec<char> = line.chars().collect();
            let (s, e) = field_span(&line, idx).expect("field exists");
            let raw: String = chars[s as usize..e as usize].iter().collect();
            // `raw` is the physical inner content (quotes doubled). Wrap it
            // back in quotes and split → the un-escaped original value.
            let reparsed = split_ags_line(&format!("\"{raw}\""));
            prop_assert_eq!(&reparsed, &vec![fields[idx as usize + 1].clone()]);
        }
    }
}
