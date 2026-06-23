//! The shared AGS4 parse leaf (#168) — one tolerant tokenizer + one
//! source-true byte/line/char walk, below both `laterite-ags4-core` and
//! `laterite-ags4-validator`.
//!
//! Clean-room, written from the AGS4.1 spec (§4.1.1 Rules 1–7). The two
//! historical parsers converge here: the validator's line-aware
//! `parse` (rich — keeps every line + per-group descriptor lines) and
//! core's `ags4_codec` (lean — byte-offset index for the `.ags.idx`
//! cert). This leaf carries BOTH coordinate systems in a single pass:
//! each record's absolute **byte** offset in the original buffer, its
//! 1-indexed **line** number, and (via [`field_span`]) **char** spans —
//! none back-derived from another.
//!
//! Phase 1 of the convergence: the crate stands alone — no consumer
//! reaches it yet. The validator adopts it in Phase 2 (`pub use`), core
//! in Phase 5; the legacy [`parse_bytes`] / [`parse_str`] signatures are
//! preserved verbatim so that adoption is import-only.
//!
//! **Trim policy:** values/units/types/headings are RAW (untrimmed) — the
//! validator's behaviour. Core's lean projection re-applies `.trim()` in
//! its own `from_shared` (Phase 5), keeping its byte-identical conveniences.

use std::borrow::Cow;
use std::collections::BTreeMap;

// --- output types (§3.3) --------------------------------------------

/// One physical source line (rich overlay; retained only when
/// [`ParseOptions::retain_raw_lines`]).
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RawLine {
    /// 1-indexed, matching how editors + the AGS4 validator report.
    pub number: u32,
    /// Line content with the trailing CR/LF stripped.
    pub text: String,
    /// Whether the line was CR+LF terminated (the `text` loses the CR, so
    /// the evidence is captured here — Rule 2a).
    pub had_crlf: bool,
    /// Absolute byte offset of this line's start in the ORIGINAL bytes
    /// (BOM included — byte 0 is genuinely byte 0).
    pub byte_offset: u64,
}

/// One DATA row.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DataRow {
    /// 1-indexed physical line (the validator's findings/fixes join key).
    pub line: u32,
    /// Record start byte offset in source bytes (record-granular index).
    pub byte_offset: u64,
    /// Field values after the leading tag, unquoted + unescaped,
    /// positionally aligned with [`ParsedGroup::headings`]. RAW (untrimmed).
    pub values: Vec<String>,
}

/// One GROUP and its descriptor rows + data.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ParsedGroup {
    pub code: String,
    pub group_line: u32,
    /// THE cert datum: byte offset of the `"GROUP",…` record start.
    pub group_byte: u64,
    pub heading_line: Option<u32>,
    pub unit_line: Option<u32>,
    pub type_line: Option<u32>,
    pub headings: Vec<String>,
    /// NOT padded — raw `rest()` (Rule 4/8 arity needs the real length).
    pub units: Vec<String>,
    /// NOT padded.
    pub types: Vec<String>,
    pub rows: Vec<DataRow>,
}

impl ParsedGroup {
    /// Raw value at `(col, row)` by position, or `None` for a short/ragged
    /// row. Borrowing accessor every typed-Arrow host feeds to
    /// `laterite_types::arrow_cols` — keeps typing out of the parse leaf.
    pub fn cell(&self, col: usize, row: usize) -> Option<&str> {
        self.rows
            .get(row)
            .and_then(|r| r.values.get(col))
            .map(String::as_str)
    }

    /// Column index of a heading by name (de-dups the `position()` dance
    /// the rules each inline).
    pub fn col(&self, name: &str) -> Option<usize> {
        self.headings.iter().position(|h| h == name)
    }
}

/// A parsed AGS4 file. The validator's `ParsedFile` plus byte fields,
/// `total_bytes`, and `byte_offsets_source_true`.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ParsedFile {
    /// First-seen wins on duplicate GROUP codes.
    pub groups: BTreeMap<String, ParsedGroup>,
    /// GROUP codes in appearance order.
    pub group_order: Vec<String>,
    /// Every line — empty unless [`ParseOptions::retain_raw_lines`].
    pub raw_lines: Vec<RawLine>,
    pub total_lines: u32,
    /// File began with a UTF-8 BOM (`EF BB BF`).
    pub has_bom: bool,
    /// Source length — the index's last-section EOF bound.
    pub total_bytes: u64,
    /// Every record's `byte_offset` indexes the ORIGINAL bytes (no decode
    /// substitution / non-borrow shifted a record start). Guards the cert
    /// against corruption — note: NOT gated on `!has_bom` (the BOM's bytes
    /// are counted, so offsets stay true through it).
    pub byte_offsets_source_true: bool,
}

// --- options + errors (§3.2) ----------------------------------------

/// How to decode bytes that aren't valid in `encoding`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InvalidUtf8 {
    /// Hard-fail with [`ParseError::NotUtf8`] (core's lean strict path).
    Reject,
    /// Substitute `U+FFFD` and carry on (the validator's path — keeps the
    /// structure parseable so later rules still run; see O-32).
    LossyReplace,
}

/// Parse knobs. The legacy entry points pin these; consumers can opt into
/// the lean profile.
#[derive(Debug, Clone, Copy)]
pub struct ParseOptions {
    /// Retain `raw_lines` (the heavy per-line `String` overlay). Off in the
    /// lean cert/sliced-read path.
    pub retain_raw_lines: bool,
    pub encoding: &'static encoding_rs::Encoding,
    pub on_invalid_utf8: InvalidUtf8,
    /// Hard-fail on a structural violation — a HEADING/UNIT/TYPE/DATA row before
    /// any GROUP, or a GROUP row with no code ([`ParseError::Structure`]).
    /// OFF by default: the lenient walk skips orphan rows so the validator's
    /// rule engine can still run and report *every* problem (a strict parser
    /// stops at the first error). Core's *read* path opts IN — a data reader
    /// fails fast on structurally-broken input (#168 Phase 5, fork 3).
    pub strict_structure: bool,
}

impl ParseOptions {
    /// Lean: no raw-line retention, UTF-8, reject invalid bytes loudly, lenient
    /// structure (the caller opts into `strict_structure` if it wants hard-fails).
    pub fn lean() -> Self {
        ParseOptions {
            retain_raw_lines: false,
            encoding: encoding_rs::UTF_8,
            on_invalid_utf8: InvalidUtf8::Reject,
            strict_structure: false,
        }
    }
    /// Validating: keep raw lines, UTF-8, lossy-replace, lenient structure (the
    /// validator twin — never crashes, reports problems as findings).
    pub fn validating() -> Self {
        ParseOptions {
            retain_raw_lines: true,
            encoding: encoding_rs::UTF_8,
            on_invalid_utf8: InvalidUtf8::LossyReplace,
            strict_structure: false,
        }
    }
}

/// The leaf's parse failures. Payloads mirror the validator's terminals so
/// `impl From<ParseError> for ValidatorError` (Phase 2) is lossless.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    /// No GROUP rows — not an AGS4 file.
    NotAgs4(String),
    /// A recognised-but-unsupported edition (AGS3 markers).
    UnsupportedEdition { found: String },
    /// `Reject` mode hit invalid bytes for `encoding`.
    NotUtf8,
    /// A strict-structure violation, raised ONLY under
    /// [`ParseOptions::strict_structure`] (core's read profile — #168 Phase 5):
    /// a HEADING/UNIT/TYPE/DATA row before any GROUP, or a GROUP row with no
    /// code. Carries the exact message core's csv reader used (`error_mapping.rs`
    /// pins these). The lenient default never raises this — the validator keeps
    /// parsing so its rule engine can report *every* problem as a finding.
    Structure(String),
}

// --- legacy entry points (signatures preserved verbatim) -------------

/// Parse from in-memory bytes with the given encoding — the validating
/// profile (lossy decode, raw lines retained). Legacy signature, used by
/// every current `parse_bytes(&bytes, encoding)` call site.
pub fn parse_bytes(
    bytes: &[u8],
    encoding: &'static encoding_rs::Encoding,
) -> Result<ParsedFile, ParseError> {
    parse_bytes_opts(
        bytes,
        ParseOptions {
            encoding,
            ..ParseOptions::validating()
        },
    )
}

/// Parse from an in-memory string. Equivalent to `parse_bytes` over the
/// string's UTF-8 bytes with the validating profile.
pub fn parse_str(text: &str) -> Result<ParsedFile, ParseError> {
    parse_bytes_opts(text.as_bytes(), ParseOptions::validating())
}

// --- the one-pass byte+line+char walk (§3.4) ------------------------

const BOM: &[u8] = &[0xEF, 0xBB, 0xBF];

/// Decode one line's bytes (BOM already stripped by the caller for line 1).
/// `LossyReplace` keeps `U+FFFD`; `Reject` fails on any invalid byte. For
/// the single-byte encodings the validator accepts (UTF-8/cp1252/latin1)
/// per-line decode equals whole-buffer decode (a record start is a `\n`
/// byte — decode-invariant). Returns `(text, had_replacement, borrowed)`.
fn decode_line(
    body: &[u8],
    encoding: &'static encoding_rs::Encoding,
    on_invalid: InvalidUtf8,
) -> Result<(String, bool, bool), ParseError> {
    let (cow, had_repl) = encoding.decode_without_bom_handling(body);
    if on_invalid == InvalidUtf8::Reject && had_repl {
        return Err(ParseError::NotUtf8);
    }
    let borrowed = matches!(cow, Cow::Borrowed(_));
    Ok((cow.into_owned(), had_repl, borrowed))
}

/// The unified parser. Drives [`split_ags_line`] over the raw bytes,
/// decoding each line, so every record carries its absolute source-byte
/// offset while line/char positions are against the decoded text.
pub fn parse_bytes_opts(bytes: &[u8], opts: ParseOptions) -> Result<ParsedFile, ParseError> {
    let has_bom = bytes.starts_with(BOM); // sniff BEFORE any strip (Rule 1)
    let total_bytes = bytes.len() as u64;

    let mut groups: BTreeMap<String, ParsedGroup> = BTreeMap::new();
    let mut group_order: Vec<String> = Vec::new();
    let mut raw_lines: Vec<RawLine> = Vec::new();
    let mut current: Option<String> = None;
    let mut looks_ags3 = false;
    let mut source_true = true;

    let mut pos = 0usize;
    let mut number = 0u32;
    while pos < bytes.len() {
        let rel = memchr::memchr(b'\n', &bytes[pos..]);
        let (line_bytes, next) = match rel {
            Some(i) => (&bytes[pos..pos + i], pos + i + 1),
            None => (&bytes[pos..], bytes.len()),
        };
        let byte_offset = pos as u64; // absolute, BOM included
        number += 1;
        let had_crlf = line_bytes.last() == Some(&b'\r');
        let mut body = if had_crlf {
            &line_bytes[..line_bytes.len() - 1]
        } else {
            line_bytes
        };
        // Strip a leading BOM for DECODE only; byte_offset stays 0.
        if number == 1 && body.starts_with(BOM) {
            body = &body[BOM.len()..];
        }
        let (text, had_repl, borrowed) = decode_line(body, opts.encoding, opts.on_invalid_utf8)?;
        if had_repl || !borrowed {
            source_true = false;
        }

        // (parse_str parity) skip a phantom trailing blank — but the
        // memchr walk never emits one: a file ending in `\n` leaves
        // pos == len after its last real line, so the loop just exits.

        let trimmed_empty = text.trim().is_empty();
        if !trimmed_empty {
            let fields = split_ags_line(&text);
            if !fields.is_empty() {
                let tag = fields[0].as_str();
                match tag {
                    "GROUP" => {
                        let code = fields.get(1).cloned().unwrap_or_default();
                        if opts.strict_structure && code.is_empty() {
                            return Err(ParseError::Structure(
                                "GROUP row missing group code".into(),
                            ));
                        }
                        groups.entry(code.clone()).or_insert_with(|| {
                            group_order.push(code.clone());
                            ParsedGroup {
                                code: code.clone(),
                                group_line: number,
                                group_byte: byte_offset,
                                heading_line: None,
                                unit_line: None,
                                type_line: None,
                                headings: Vec::new(),
                                units: Vec::new(),
                                types: Vec::new(),
                                rows: Vec::new(),
                            }
                        });
                        // A duplicate GROUP still re-points "current" so its
                        // rows attach somewhere; the dup is reported from
                        // group_order vs groups by the validator.
                        current = Some(code);
                    }
                    "HEADING" => {
                        if let Some(g) = current.as_ref().and_then(|c| groups.get_mut(c)) {
                            g.heading_line = Some(number);
                            g.headings = fields[1..].to_vec();
                        } else if opts.strict_structure {
                            return Err(ParseError::Structure(
                                "HEADING row before any GROUP".into(),
                            ));
                        }
                    }
                    "UNIT" => {
                        if let Some(g) = current.as_ref().and_then(|c| groups.get_mut(c)) {
                            g.unit_line = Some(number);
                            g.units = fields[1..].to_vec();
                        } else if opts.strict_structure {
                            return Err(ParseError::Structure("UNIT row before any GROUP".into()));
                        }
                    }
                    "TYPE" => {
                        if let Some(g) = current.as_ref().and_then(|c| groups.get_mut(c)) {
                            g.type_line = Some(number);
                            g.types = fields[1..].to_vec();
                        } else if opts.strict_structure {
                            return Err(ParseError::Structure("TYPE row before any GROUP".into()));
                        }
                    }
                    "DATA" => {
                        if let Some(g) = current.as_ref().and_then(|c| groups.get_mut(c)) {
                            g.rows.push(DataRow {
                                line: number,
                                byte_offset,
                                values: fields[1..].to_vec(),
                            });
                        } else if opts.strict_structure {
                            return Err(ParseError::Structure("DATA row before any GROUP".into()));
                        }
                    }
                    _ => {
                        if tag.starts_with("**") || tag == "<UNITS>" || tag == "<CONT>" {
                            looks_ags3 = true;
                        }
                    }
                }
            }
        }

        if opts.retain_raw_lines {
            raw_lines.push(RawLine {
                number,
                text,
                had_crlf,
                byte_offset,
            });
        }
        pos = next;
    }

    if group_order.is_empty() {
        if looks_ags3 {
            return Err(ParseError::UnsupportedEdition {
                found: "3.x (AGS3 format)".to_string(),
            });
        }
        return Err(ParseError::NotAgs4("no GROUP rows found".to_string()));
    }

    let total_lines = number; // every line counted (raw_lines may be empty)
    Ok(ParsedFile {
        groups,
        group_order,
        raw_lines,
        total_lines,
        has_bom,
        total_bytes,
        byte_offsets_source_true: source_true,
    })
}

// --- tokenizer + char-span tracker (lifted verbatim from the validator) ---

/// Split one AGS4 line into fields. AGS4 wraps every field in double quotes
/// and doubles embedded quotes (`""`). Tolerant: an unquoted field is read
/// up to the next comma; an unterminated quote consumes to end-of-line.
/// Returns owned, unescaped values.
pub fn split_ags_line(line: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut chars = line.chars().peekable();

    loop {
        match chars.peek() {
            None => break,
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
                match chars.peek() {
                    Some(',') => {
                        chars.next();
                        continue;
                    }
                    _ => {
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

/// Char-offset span of the content INSIDE the quotes of the raw-line field
/// at position `field_index + 1` (the `+1` skips the leading tag). Half-open
/// `(start, end)` in CHARS (Unicode scalars), not bytes. `None` if the line
/// has fewer fields than requested.
pub fn field_span(line: &str, field_index: u32) -> Option<(u32, u32)> {
    let target = field_index as usize + 1;
    let mut pos: u32 = 0;
    let mut field = 0usize;
    let mut chars = line.chars().peekable();

    loop {
        match chars.peek().copied() {
            None => return None,
            Some('"') => {
                chars.next();
                pos += 1;
                let start = pos;
                let mut end = pos;
                loop {
                    match chars.next() {
                        None => break,
                        Some('"') => {
                            if chars.peek() == Some(&'"') {
                                chars.next();
                                pos += 2;
                                end = pos;
                            } else {
                                pos += 1;
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
