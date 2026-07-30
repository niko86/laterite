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
    /// `laterite_ags4_types::arrow_cols` — keeps typing out of the parse leaf.
    pub fn cell(&self, col: usize, row: usize) -> Option<&str> {
        self.rows
            .get(row)
            .and_then(|r| r.values.get(col))
            .map(String::as_str)
    }

    /// Column index of a heading by name (de-dups the `position()` dance
    /// the rules each inline).
    #[must_use]
    pub fn col(&self, name: &str) -> Option<usize> {
        self.headings.iter().position(|h| h == name)
    }
}

/// One `"GROUP",…` record as it appears in the source — EVERY occurrence, including
/// a code's second and later declarations.
///
/// [`ParsedFile::groups`] is first-seen-wins, which is right for the typed view (a
/// redeclared group's rows all attach to the one entry). It is wrong for a *locator*:
/// the byte index built from it spanned only the FIRST section, so slicing a
/// redeclared group re-parsed part of the file and silently returned fewer rows than
/// the whole-file parse — no error, no warning. A cert cannot vouch for a location it
/// cannot state, so the index needs every span, and that means every record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GroupRecord {
    pub code: String,
    /// Byte offset of the `"GROUP"` record in the ORIGINAL bytes.
    pub byte_offset: u64,
    /// 1-indexed source line of the record.
    pub line: u32,
}

/// A parsed AGS4 file. The validator's `ParsedFile` plus byte fields,
/// `total_bytes`, and `byte_offsets_source_true`.
#[derive(Debug, Clone)]
pub struct ParsedFile {
    /// First-seen wins on duplicate GROUP codes.
    pub groups: BTreeMap<String, ParsedGroup>,
    /// GROUP codes in appearance order.
    pub group_order: Vec<String>,
    /// EVERY `"GROUP"` record, in source order — duplicates included. `group_order`
    /// is the de-duplicated view of this; anything that needs to LOCATE bytes must
    /// read this instead (see [`GroupRecord`]).
    pub group_records: Vec<GroupRecord>,
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
    /// Locate GROUP records only — skip HEADING/UNIT/TYPE/DATA materialisation.
    ///
    /// The cert/index path needs `group_records` + `group_order` and nothing
    /// else, but it used to pay for the whole row model and discard it: on a
    /// 25 MB file that is ~418k lines tokenised into owned `String`s to keep
    /// ~123 GROUP records. Under this flag the walk is IDENTICAL — same line
    /// spans, same decode, same UTF-8 rejection, same AGS3 sniff, same
    /// first-seen-wins `group_order` — it just stops building what the caller
    /// then drops. `groups` comes back with empty headings/units/types/rows,
    /// so it is NOT a read profile; `index_ags4_bytes` is the only caller.
    pub locate_only: bool,
}

impl ParseOptions {
    /// Lean: no raw-line retention, UTF-8, reject invalid bytes loudly, lenient
    /// structure (the caller opts into `strict_structure` if it wants hard-fails).
    #[must_use]
    pub fn lean() -> Self {
        ParseOptions {
            retain_raw_lines: false,
            encoding: encoding_rs::UTF_8,
            on_invalid_utf8: InvalidUtf8::Reject,
            strict_structure: false,
            locate_only: false,
        }
    }
    /// Validating: keep raw lines, UTF-8, lossy-replace, lenient structure (the
    /// validator twin — never crashes, reports problems as findings).
    #[must_use]
    pub fn validating() -> Self {
        ParseOptions {
            retain_raw_lines: true,
            encoding: encoding_rs::UTF_8,
            on_invalid_utf8: InvalidUtf8::LossyReplace,
            strict_structure: false,
            locate_only: false,
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

/// Resolve a caller-supplied encoding label to an `encoding_rs` encoding — the
/// single source of truth every surface (`read`/`validate`/`fix` on Python,
/// Node, and the browser) shares, so a label means the same thing everywhere.
///
/// `None` or empty ⇒ UTF-8. The common legacy-producer spellings are accepted
/// explicitly — the ones WHATWG's `for_label` does NOT know — and anything else
/// flows through `for_label`. Returns `None` for a genuinely unknown label.
///
/// **An unknown label is an ERROR on every surface. Never a fallback.** A caller
/// who asks for `cp1252x` has made a typo, and quietly decoding as UTF-8 instead is
/// a corruption vector: the bytes `C3 A9` are `é` in UTF-8 and `Ã©` in cp1252, both
/// decode cleanly, and the file then validates "clean" with the wrong text in it.
/// Node and the browser used to fall back to UTF-8 here while Python raised.
///
/// The explicit arms exist because `for_label` rejects these spellings (probed, not
/// assumed):
///   * `latin-1` (hyphenated) — WHATWG knows `latin1`, not `latin-1`.
///   * `latin9` / `latin-9` — WHATWG knows `l9`, `iso-8859-15`, `csisolatin9`, but
///     neither of these. They were only accepted by a private table inside the `lat`
///     CLI, so `--encoding latin-9` worked on the binary and was rejected by the
///     Python library. Promoted here so one label means one thing everywhere.
#[must_use]
pub fn resolve_encoding(label: Option<&str>) -> Option<&'static encoding_rs::Encoding> {
    let Some(label) = label else {
        return Some(encoding_rs::UTF_8);
    };
    match label.trim().to_ascii_lowercase().as_str() {
        "" | "utf-8" | "utf8" => Some(encoding_rs::UTF_8),
        // Latin-1 ≈ Windows-1252 except 0x80–0x9F; for AGS4 we treat them as the
        // same, cp1252 being the strict superset python-ags4 uses.
        "cp1252" | "windows-1252" | "latin1" | "latin-1" | "iso-8859-1" => {
            Some(encoding_rs::WINDOWS_1252)
        }
        "latin9" | "latin-9" => Some(encoding_rs::ISO_8859_15),
        other => encoding_rs::Encoding::for_label(other.as_bytes()),
    }
}

// --- the one-pass byte+line+char walk (§3.4) ------------------------

const BOM: &[u8] = &[0xEF, 0xBB, 0xBF];

/// Decode one line's bytes (BOM already stripped by the caller for line 1).
/// `LossyReplace` keeps `U+FFFD`; `Reject` fails on any invalid byte. For
/// the single-byte encodings the validator accepts (UTF-8/cp1252/latin1)
/// per-line decode equals whole-buffer decode (a record start is a `\n`
/// byte — decode-invariant). Returns `(text, had_replacement, borrowed)`.
fn decode_line<'a>(
    body: &'a [u8],
    encoding: &'static encoding_rs::Encoding,
    on_invalid: InvalidUtf8,
) -> Result<(Cow<'a, str>, bool, bool), ParseError> {
    let (cow, had_repl) = encoding.decode_without_bom_handling(body);
    if on_invalid == InvalidUtf8::Reject && had_repl {
        return Err(ParseError::NotUtf8);
    }
    let borrowed = matches!(cow, Cow::Borrowed(_));
    // Deliberately NOT `into_owned()`. Every line used to allocate a String
    // here even when the caller only needed a `&str` to tokenize and then
    // dropped it — 418k allocations on a 25 MB file. The one caller that
    // genuinely needs ownership (`retain_raw_lines`) takes it at the push.
    Ok((cow, had_repl, borrowed))
}

// --- the shared quote-aware line splitter (#422) --------------------

/// How a line was terminated. AGS4 Rule 2a mandates `Crlf`; `Lf` (Unix) and
/// `Cr` (classic-Mac) are non-conforming terminators the reader still splits on
/// but the validator flags. `Unterminated` is a final line with no terminator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineTerminator {
    /// `\r\n` — the only AGS4-conforming terminator.
    Crlf,
    /// lone `\n` (Unix) — Rule 2a.
    Lf,
    /// lone `\r` (classic Mac) — Rule 2a. Recognised as a terminator here so a
    /// lone-CR row separator is not mistaken for an *embedded* CR (#422).
    Cr,
    /// End of input with no terminator (unterminated final line).
    Unterminated,
}

impl LineTerminator {
    /// The terminator as a string (all terminators are ASCII).
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            LineTerminator::Crlf => "\r\n",
            LineTerminator::Lf => "\n",
            LineTerminator::Cr => "\r",
            LineTerminator::Unterminated => "",
        }
    }
}

/// One line located by the quote-aware walk: the byte range of its body (the
/// terminator excluded) in the buffer, how it was terminated, and where to
/// resume. `start..body_end` is always a valid slice; for a `&str` buffer it is
/// also a char boundary (every delimiter is ASCII).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LineSpan {
    /// Byte offset of the line body start.
    pub start: usize,
    /// Byte offset one past the body (before any terminator).
    pub body_end: usize,
    pub term: LineTerminator,
    /// Byte offset to resume the walk from (past the terminator).
    pub next: usize,
}

/// The five data descriptors as they open a row (`"GROUP"`, …). Used only by the
/// unterminated-quote recovery backstop below.
const QUOTED_DESCRIPTORS: [&[u8]; 5] = [
    b"\"GROUP\"",
    b"\"HEADING\"",
    b"\"UNIT\"",
    b"\"TYPE\"",
    b"\"DATA\"",
];

/// Does the buffer at `at` begin a new row (a quoted data descriptor)? The
/// recovery backstop: an *unterminated* quote would otherwise swallow every
/// following row as embedded content; AGS4's fixed descriptor vocabulary lets us
/// resync at the next obvious row boundary, bounding the runaway to one row.
fn starts_with_descriptor(bytes: &[u8], at: usize) -> bool {
    let rest = &bytes[at.min(bytes.len())..];
    QUOTED_DESCRIPTORS.iter().any(|d| rest.starts_with(d))
}

/// Byte offset just past the terminator at `j` (`\r\n` is two bytes, else one).
fn terminator_end(bytes: &[u8], j: usize) -> usize {
    if bytes[j] == b'\r' && bytes.get(j + 1) == Some(&b'\n') {
        j + 2
    } else {
        j + 1
    }
}

/// Build the `LineSpan` ending at the terminator byte `j` (in `start..`).
fn terminate_at(bytes: &[u8], start: usize, j: usize) -> LineSpan {
    let term = if bytes[j] == b'\r' {
        if bytes.get(j + 1) == Some(&b'\n') {
            LineTerminator::Crlf
        } else {
            LineTerminator::Cr
        }
    } else {
        LineTerminator::Lf
    };
    LineSpan {
        start,
        body_end: j,
        term,
        next: terminator_end(bytes, j),
    }
}

/// The states of AGS4's CSV grammar, mirroring [`split_ags_line`]: a `"` only
/// *opens* a field at a field boundary; after a field's closing `"` we skip junk
/// to the next comma **quote-blind**. Tracking the same states here means a CR/LF
/// is classed as a line terminator in exactly the positions where `split_ags_line`
/// treats it as between/after fields — so line boundaries and field boundaries
/// never disagree (the property `split_ags_line` ⇄ splitter agreement pins).
#[derive(Clone, Copy)]
enum QState {
    /// At the start of a field (line start, or just after a comma).
    FieldStart,
    /// Inside a quoted field — a CR/LF here is embedded content, not a boundary.
    Quoted,
    /// After a quoted field closed; skipping junk to the next comma (quote-blind).
    AfterClose,
    /// Inside an unquoted field; reading to the next comma.
    Unquoted,
}

/// Find the next line starting at `start`, **quote-aware and universal-newline**:
/// a CR/LF *outside* a quoted field terminates the line (`\r\n`/`\n`/lone `\r`);
/// a CR/LF *inside* a quoted field is embedded content and does NOT split —
/// UNLESS the next line begins with a data descriptor (the unterminated-quote
/// backstop). `memchr3` keeps the scan at SIMD speed on the common (long,
/// delimiter-sparse) runs.
fn next_line(bytes: &[u8], start: usize) -> LineSpan {
    let n = bytes.len();
    let eof = || LineSpan {
        start,
        body_end: n,
        term: LineTerminator::Unterminated,
        next: n,
    };
    let mut i = start;
    let mut state = QState::FieldStart;
    loop {
        match state {
            QState::FieldStart => {
                if i >= n {
                    return eof();
                }
                match bytes[i] {
                    b'"' => {
                        state = QState::Quoted;
                        i += 1;
                    }
                    b'\r' | b'\n' => return terminate_at(bytes, start, i),
                    // Any other byte begins an unquoted field; reprocess it there.
                    _ => state = QState::Unquoted,
                }
            }
            QState::Quoted => match memchr::memchr3(b'"', b'\r', b'\n', &bytes[i..]) {
                None => return eof(),
                Some(off) => {
                    let j = i + off;
                    if bytes[j] == b'"' {
                        if bytes.get(j + 1) == Some(&b'"') {
                            i = j + 2; // doubled "" — escaped quote, stay in the field
                        } else {
                            state = QState::AfterClose; // closing quote
                            i = j + 1;
                        }
                    } else {
                        // CR/LF inside quotes → embedded content, unless the next
                        // line is clearly a new row (unterminated-quote recovery).
                        if starts_with_descriptor(bytes, terminator_end(bytes, j)) {
                            return terminate_at(bytes, start, j);
                        }
                        i = j + 1;
                    }
                }
            },
            // AfterClose and Unquoted scan identically (quote-blind to the next
            // comma/CR/LF) — they only differ in how they were entered, not in
            // what to do next, so clippy's match_same_arms is right to merge them.
            QState::AfterClose | QState::Unquoted => {
                match memchr::memchr3(b',', b'\r', b'\n', &bytes[i..]) {
                    None => return eof(),
                    Some(off) => {
                        let j = i + off;
                        if bytes[j] == b',' {
                            state = QState::FieldStart;
                            i = j + 1;
                        } else {
                            return terminate_at(bytes, start, j); // CR/LF outside quotes
                        }
                    }
                }
            }
        }
    }
}

/// Iterator over the quote-aware [`LineSpan`]s of a buffer — the ONE line model
/// the parser and `apply_fixes` share, so their line numbering agrees by
/// construction (fix edits carry parser line numbers and must land on the same
/// line the fixer reconstructs). A buffer that ends exactly at a terminator
/// yields no phantom trailing blank line.
pub struct LineSpans<'a> {
    bytes: &'a [u8],
    pos: usize,
    done: bool,
}

impl Iterator for LineSpans<'_> {
    type Item = LineSpan;
    fn next(&mut self) -> Option<LineSpan> {
        if self.done || self.pos > self.bytes.len() {
            return None;
        }
        if self.pos == self.bytes.len() {
            // An empty buffer yields nothing; a non-empty one has already
            // emitted its last line (its terminator ended at len).
            self.done = true;
            return None;
        }
        let span = next_line(self.bytes, self.pos);
        self.pos = span.next;
        Some(span)
    }
}

/// Quote-aware line spans over raw bytes (the parser's entry).
#[must_use]
pub fn line_spans(bytes: &[u8]) -> LineSpans<'_> {
    LineSpans {
        bytes,
        pos: 0,
        done: false,
    }
}

/// The unified parser. Drives [`split_ags_line`] over the raw bytes,
/// decoding each line, so every record carries its absolute source-byte
/// offset while line/char positions are against the decoded text.
/// Take everything after the leading tag, MOVING the values out of `fields`.
///
/// This used to be `fields[1..].to_vec()`, which clones every `String` the
/// tokenizer just allocated — so each cell was heap-allocated twice, once by
/// `split_ags_line` and once here, on every descriptor and DATA row. Nothing
/// reads `fields` after its arm, so the tail can simply be handed over.
fn rest(fields: &mut Vec<String>) -> Vec<String> {
    if fields.len() > 1 {
        fields.split_off(1)
    } else {
        Vec::new()
    }
}

pub fn parse_bytes_opts(bytes: &[u8], opts: ParseOptions) -> Result<ParsedFile, ParseError> {
    let has_bom = bytes.starts_with(BOM); // sniff BEFORE any strip (Rule 1)
    let total_bytes = bytes.len() as u64;

    let mut groups: BTreeMap<String, ParsedGroup> = BTreeMap::new();
    let mut group_order: Vec<String> = Vec::new();
    let mut group_records: Vec<GroupRecord> = Vec::new();
    let mut raw_lines: Vec<RawLine> = Vec::new();
    let mut current: Option<String> = None;
    let mut looks_ags3 = false;
    let mut source_true = true;

    let mut number = 0u32;
    for span in line_spans(bytes) {
        let byte_offset = span.start as u64; // absolute, BOM included
        number += 1;
        // `had_crlf` stays "was this CRLF-terminated" (Rule 2a). A lone `\r`
        // (classic Mac) or lone `\n` (Unix) terminator is now a genuine split
        // point like `\r\n`, but reported as improper rather than swallowed as
        // an embedded CR — #422. An embedded CR/LF *inside* a quoted field is
        // NOT a terminator, so it stays in `body` for Rule 6 to flag (O-2).
        let had_crlf = span.term == LineTerminator::Crlf;
        let mut body = &bytes[span.start..span.body_end];
        // Strip a leading BOM for DECODE only; byte_offset stays 0.
        if number == 1 && body.starts_with(BOM) {
            body = &body[BOM.len()..];
        }
        let (text, had_repl, borrowed) = decode_line(body, opts.encoding, opts.on_invalid_utf8)?;
        if had_repl || !borrowed {
            source_true = false;
        }

        // No phantom trailing blank: `line_spans` stops when the buffer ends
        // exactly at a terminator (a file ending in `\r\n`/`\n`/`\r` yields no
        // extra empty line), matching the old memchr walk's behaviour.

        let trimmed_empty = text.trim().is_empty();
        if !trimmed_empty {
            // Read the tag by BORROWING field 0, then split only when the tag
            // says the rest of the line is wanted. The old order — split every
            // line into owned Strings, then look at [0] — paid for the whole
            // row on lines whose tag alone decides the outcome.
            //
            // `first_field` cannot unescape (a borrowed slice can't shrink), so
            // it differs from `split_ags_line[0]` only on a tag containing `""`.
            // No AGS4 descriptor and no AGS3 marker (`**`, `<UNITS>`, `<CONT>`)
            // contains a quote, so every arm below reads the same either way.
            //
            // Non-empty after trim ⇒ non-empty field 0, so this never skips a
            // line the old `!fields.is_empty()` guard would have processed.
            if let Some(tag) = scan::first_field(&text) {
                // A locator needs GROUP and nothing else; the full walk needs
                // every descriptor row. An unrecognised tag needs neither — it
                // is only sniffed for AGS3 markers, which live in the tag.
                let needs_fields = match tag {
                    "GROUP" => true,
                    "HEADING" | "UNIT" | "TYPE" | "DATA" => !opts.locate_only,
                    _ => false,
                };
                let mut fields = if needs_fields {
                    split_ags_line(&text)
                } else {
                    Vec::new()
                };
                match tag {
                    "GROUP" => {
                        let code = fields.get(1).cloned().unwrap_or_default();
                        if opts.strict_structure && code.is_empty() {
                            return Err(ParseError::Structure(
                                "GROUP row missing group code".into(),
                            ));
                        }
                        // EVERY occurrence, before the first-seen-wins insert below —
                        // this is what a locator must be built from.
                        group_records.push(GroupRecord {
                            code: code.clone(),
                            byte_offset,
                            line: number,
                        });
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
                    // Locate-only stops here: the descriptor rows are exactly
                    // the model this profile exists not to build. Matched
                    // BEFORE the real arms (and not folded into `_`) so an
                    // AGS3 sniff can never see a descriptor tag.
                    "HEADING" | "UNIT" | "TYPE" | "DATA" if opts.locate_only => {
                        // Skipping the model must not skip the GUARD. The
                        // orphan-row check reads only `current`, which this
                        // profile still maintains, so it costs nothing here —
                        // and the message stays byte-identical to the full
                        // walk's (core's `error_mapping` pins these strings).
                        if opts.strict_structure
                            && current.as_ref().and_then(|c| groups.get(c)).is_none()
                        {
                            return Err(ParseError::Structure(format!(
                                "{tag} row before any GROUP"
                            )));
                        }
                    }
                    "HEADING" => {
                        if let Some(g) = current.as_ref().and_then(|c| groups.get_mut(c)) {
                            g.heading_line = Some(number);
                            g.headings = rest(&mut fields);
                        } else if opts.strict_structure {
                            return Err(ParseError::Structure(
                                "HEADING row before any GROUP".into(),
                            ));
                        }
                    }
                    "UNIT" => {
                        if let Some(g) = current.as_ref().and_then(|c| groups.get_mut(c)) {
                            g.unit_line = Some(number);
                            g.units = rest(&mut fields);
                        } else if opts.strict_structure {
                            return Err(ParseError::Structure("UNIT row before any GROUP".into()));
                        }
                    }
                    "TYPE" => {
                        if let Some(g) = current.as_ref().and_then(|c| groups.get_mut(c)) {
                            g.type_line = Some(number);
                            g.types = rest(&mut fields);
                        } else if opts.strict_structure {
                            return Err(ParseError::Structure("TYPE row before any GROUP".into()));
                        }
                    }
                    "DATA" => {
                        if let Some(g) = current.as_ref().and_then(|c| groups.get_mut(c)) {
                            g.rows.push(DataRow {
                                line: number,
                                byte_offset,
                                values: rest(&mut fields),
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
                // The one site that genuinely needs ownership — so it is the
                // one site that pays for it.
                text: text.into_owned(),
                had_crlf,
                byte_offset,
            });
        }
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
        group_records,
        raw_lines,
        total_lines,
        has_bom,
        total_bytes,
        byte_offsets_source_true: source_true,
    })
}

pub mod scan;

// --- tokenizer + char-span tracker (lifted verbatim from the validator) ---

/// Split one AGS4 line into fields. AGS4 wraps every field in double quotes
/// and doubles embedded quotes (`""`). Tolerant: an unquoted field is read
/// up to the next comma; an unterminated quote consumes to end-of-line.
/// Returns owned, unescaped values.
#[must_use]
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
                if let Some(',') = chars.peek() {
                    chars.next();
                    continue;
                }
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
#[must_use]
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
// --- offset-preserving field tokenizer (#533) -----------------------
//
// RETIRED 2026-07-24. `AgsSpan` + `tokenize_spans` were a third hand-written
// implementation of the AGS4 line grammar; `scan::RawField` is a strict
// superset of what they returned (the same four bounds, plus `quoted`,
// `has_escape` and `had_comma`), so callers use `scan::scan_line(line, DISPLAY)`
// directly. See `scan.rs` for why the value policy is a parameter.
//
// The one deliberate change: bounds are BYTES, not code points. Rust callers
// index `&str` by byte and previously could not use the offsets at all — which
// is exactly why `AgsSpan` carried an owned `text` copy of every field. The
// browser still needs code points, so that conversion now happens in the wasm
// adapter that actually has the requirement, instead of every consumer paying
// for a materialised string it mostly did not want.
