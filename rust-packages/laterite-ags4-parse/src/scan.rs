//! PROTOTYPE — one AGS4 line scanner, in bytes.
//!
//! The grammar (quote-wrapped fields, `""` escapes, comma separators) is
//! currently implemented THREE times in `lib.rs`: `split_ags_line` (owned
//! unescaped values), `field_span` (char span of one field) and
//! `tokenize_spans` (all spans, lossless reassembly). Three hand-written
//! machines over one grammar, which is how they came to disagree on four
//! behaviours — empty-line field count, unquoted trimming, field indexing and
//! whether `""` stays escaped.
//!
//! This is the shared core they can all sit on. Two design choices make that
//! possible without taxing the hot path:
//!
//! * **Bytes, not code points.** `"` and `,` are ASCII, and UTF-8 guarantees an
//!   ASCII byte never appears inside a multi-byte sequence, so a byte scan is
//!   exactly equivalent to the char scan and needs no decoding. The browser's
//!   code-point offsets are then a CONVERSION applied by that adapter only —
//!   the validator's 418k-line walk stops paying for a requirement it never had.
//! * **No allocation.** The scanner yields bounds; adapters allocate only what
//!   their own contract demands (`split_ags_line` must, because unescaping `""`
//!   produces a value shorter than its source slice and no span can express
//!   that).
//!
//! State machine mirrors `tokenize_spans`, the most complete of the three: a
//! token runs to and INCLUDES its trailing comma, so concatenating every token
//! reproduces the line.
//!
//! # Why the value policy is a parameter, not a decision
//!
//! What was duplicated three times — and where all five behavioural divergences
//! came from — is the STATE MACHINE. Consolidating it is pure win.
//!
//! What legitimately differs is how a token's inner VALUE is resolved, and those
//! differences are needs rather than accidents: a browser editor wants
//! display-trimmed bounds to highlight; a validator must judge the raw bytes,
//! because on an unquoted field the surrounding whitespace IS the Rule 5
//! violation. Collapsing them would let a UI concern define what the validator
//! considers a field's value.
//!
//! So the scan is shared and the interpretation is the caller's.

/// How a token's inner value is resolved. The scan is identical either way;
/// only these two judgements differ between callers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ValuePolicy {
    /// Trim ASCII spaces from an unquoted field's content.
    ///
    /// FALSE for anything that judges the bytes: on an unquoted field the
    /// surrounding whitespace is part of what Rule 5 is complaining about, so
    /// trimming it would hide the violation. TRUE for display, where the editor
    /// wants the content bounds to highlight.
    pub trim_unquoted: bool,
    /// An unterminated quote runs to end-of-line (TRUE) rather than yielding an
    /// empty value (FALSE).
    ///
    /// TRUE preserves the tolerant readers' "salvage what is there" behaviour;
    /// FALSE preserves the display tokenizer's.
    pub unterminated_to_eol: bool,
}

/// The judging policy: raw bytes, nothing trimmed, unterminated content
/// salvaged. What `split_ags_line` and `field_span` have always done.
pub const RAW: ValuePolicy = ValuePolicy {
    trim_unquoted: false,
    unterminated_to_eol: true,
};

/// The display policy: unquoted content trimmed, an unterminated quote
/// yielding an empty inner span. What `tokenize_spans` has always done.
pub const DISPLAY: ValuePolicy = ValuePolicy {
    trim_unquoted: true,
    unterminated_to_eol: false,
};

/// One field's bounds within a line, in BYTE offsets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RawField {
    /// Token start — includes any leading whitespace and the opening quote.
    pub token_start: usize,
    /// One past token end — INCLUDES the trailing comma when one closed it.
    pub token_end: usize,
    /// Inner value start: just inside the opening quote when quoted; for an
    /// unquoted field the space-trimmed content start.
    pub value_start: usize,
    /// One past the inner value's end.
    pub value_end: usize,
    /// The field was quote-wrapped.
    pub quoted: bool,
    /// The inner value contains a doubled quote (`""`) and so needs unescaping
    /// to become its logical value. Lets a caller borrow the common case and
    /// allocate only for the rare one.
    pub has_escape: bool,
    /// A comma closed this token (so `token_end - 1` is that comma).
    pub had_comma: bool,
}

/// Scan `line` into field bounds. Allocation-free; the `Vec` is the caller's.
#[must_use]
pub fn scan_line(line: &str, policy: ValuePolicy) -> Vec<RawField> {
    let b = line.as_bytes();
    let n = b.len();
    let mut out: Vec<RawField> = Vec::new();

    let mut i = 0usize;
    let mut token_start = 0usize;
    let mut in_quotes = false;
    let mut value_start: Option<usize> = None;
    let mut value_end = 0usize;
    let mut has_escape = false;
    let mut closed = false;

    while i < n {
        let c = b[i];
        if in_quotes {
            if c == b'"' {
                // A doubled quote is an escaped literal, not a close.
                if b.get(i + 1) == Some(&b'"') {
                    has_escape = true;
                    i += 2;
                    continue;
                }
                in_quotes = false;
                closed = true;
                value_end = i; // content ends just before the closing quote
            }
            i += 1;
            continue;
        }
        if c == b'"' {
            in_quotes = true;
            closed = false;
            i += 1;
            value_start = Some(i);
            value_end = i; // an empty quoted field is a zero-width inner span
        } else if c == b',' {
            i += 1;
            out.push(finish(
                b,
                token_start,
                i,
                true,
                value_start,
                value_end,
                has_escape,
                closed,
                policy,
            ));
            value_start = None;
            value_end = 0;
            has_escape = false;
            closed = false;
            token_start = i;
        } else {
            i += 1;
        }
    }

    // Trailing token (after the last comma, or the whole line if commaless).
    if token_start < n || out.is_empty() {
        out.push(finish(
            b,
            token_start,
            n,
            false,
            value_start,
            value_end,
            has_escape,
            closed,
            policy,
        ));
    }
    out
}

/// Field 0's inner value — what a descriptor check (Rule 3) actually needs.
///
/// The whole reason this module exists: `rule_3` called `split_ags_line`,
/// allocating a `Vec<String>` of every field (~21 allocations on a 20-column
/// DATA row) to read one of them, on every line of every file.
#[must_use]
pub fn first_field(line: &str) -> Option<&str> {
    first_field_with(line, RAW)
}

/// [`first_field`] under an explicit policy.
#[must_use]
pub fn first_field_with(line: &str, policy: ValuePolicy) -> Option<&str> {
    let b = line.as_bytes();
    let n = b.len();
    if n == 0 {
        return None;
    }
    let mut i = 0usize;
    let mut in_quotes = false;
    let mut value_start: Option<usize> = None;
    let mut value_end = 0usize;
    let mut closed = false;

    while i < n {
        let c = b[i];
        if in_quotes {
            if c == b'"' {
                if b.get(i + 1) == Some(&b'"') {
                    i += 2;
                    continue;
                }
                in_quotes = false;
                closed = true;
                value_end = i;
            }
            i += 1;
            continue;
        }
        if c == b'"' {
            in_quotes = true;
            closed = false;
            i += 1;
            value_start = Some(i);
            value_end = i;
        } else if c == b',' {
            break;
        } else {
            i += 1;
        }
    }
    let f = finish(
        b,
        0,
        i,
        false,
        value_start,
        value_end,
        false,
        closed,
        policy,
    );
    line.get(f.value_start..f.value_end)
}

/// Resolve one token's inner value bounds. Unquoted (no opening quote seen)
/// trims ASCII spaces from the content, excluding any trailing comma — the
/// `tokenize_spans` rule, kept in one place instead of three.
#[allow(clippy::too_many_arguments)]
fn finish(
    b: &[u8],
    token_start: usize,
    end: usize,
    had_comma: bool,
    value_start: Option<usize>,
    value_end: usize,
    has_escape: bool,
    closed: bool,
    policy: ValuePolicy,
) -> RawField {
    let quoted = value_start.is_some();
    let (vs, ve) = if let Some(vs) = value_start {
        // An unterminated quote: salvage to end-of-line, or yield the empty
        // inner span — the callers genuinely disagree, so the policy decides.
        if !closed && policy.unterminated_to_eol {
            let content_end = if had_comma { end - 1 } else { end };
            (vs, content_end.max(vs))
        } else {
            (vs, value_end)
        }
    } else {
        let content_end = if had_comma { end - 1 } else { end };
        let mut vs = token_start;
        let mut ve = content_end;
        if policy.trim_unquoted {
            while vs < ve && b[vs] == b' ' {
                vs += 1;
            }
            while ve > vs && b[ve - 1] == b' ' {
                ve -= 1;
            }
        }
        (vs, ve)
    };
    RawField {
        token_start,
        token_end: end,
        value_start: vs,
        value_end: ve,
        quoted,
        has_escape,
        had_comma,
    }
}

#[cfg(test)]
mod differential {
    use super::*;
    use crate::{split_ags_line, tokenize_spans};

    /// Every line of the real fixture, plus edge cases, through both the new
    /// core and each incumbent. This is the evidence that consolidation is
    /// behaviour-preserving — or the list of places it isn't.
    fn corpus() -> Vec<String> {
        let mut v: Vec<String> = vec![
            String::new(),
            ",".into(),
            "\"\"".into(),
            "\"DATA\"".into(),
            "\"DATA\",".into(),
            "\"DATA\",\"BH1\"".into(),
            " DATA ,x".into(),
            "\"a\"\"b\",\"c\"".into(),
            "\"unterminated".into(),
            "no,quotes,here".into(),
            "\"astral\",\"emoji-x\"".into(),
        ];
        let p = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../output/bench-fixtures/small.ags");
        if let Ok(t) = std::fs::read_to_string(p) {
            v.extend(t.lines().map(ToString::to_string));
        }
        v
    }

    #[test]
    fn field_count_matches_tokenize_spans() {
        let mut mismatches = 0usize;
        for l in corpus() {
            if scan_line(&l, DISPLAY).len() != tokenize_spans(&l).len() {
                mismatches += 1;
                if mismatches <= 5 {
                    eprintln!("COUNT differs on {l:?}");
                }
            }
        }
        assert_eq!(mismatches, 0, "{mismatches} field-count mismatches");
    }

    #[test]
    fn value_bounds_match_tokenize_spans() {
        let mut mismatches = 0usize;
        for l in corpus() {
            let mine = scan_line(&l, DISPLAY);
            let theirs = tokenize_spans(&l);
            if mine.len() != theirs.len() {
                continue;
            }
            for (m, t) in mine.iter().zip(theirs.iter()) {
                let my_val = &l[m.value_start..m.value_end];
                let their_val: String = l
                    .chars()
                    .skip(t.value_start as usize)
                    .take((t.value_end - t.value_start) as usize)
                    .collect();
                if my_val != their_val {
                    mismatches += 1;
                    if mismatches <= 5 {
                        eprintln!("VALUE differs on {l:?}: mine={my_val:?} theirs={their_val:?}");
                    }
                }
            }
        }
        assert_eq!(mismatches, 0, "{mismatches} value mismatches");
    }

    /// `first_field` returns a BORROWED slice, so it cannot unescape: a `""`
    /// becomes a single `"` only by allocating, and a span of the source cannot
    /// express a value shorter than itself. So the contract is "raw inner
    /// slice", and `RawField::has_escape` tells a caller when that differs from
    /// the logical value.
    ///
    /// This is sound for its caller: Rule 3 compares against GROUP / HEADING /
    /// UNIT / TYPE / DATA, none of which contain a quote — a field with an
    /// escape is not a descriptor under either reading.
    #[test]
    fn first_field_matches_split_ags_line_where_unescaped() {
        let mut mismatches = 0usize;
        for l in corpus() {
            // Skip only the documented escape divergence, and prove it IS the
            // escape rather than assuming: the flag must agree.
            if scan_line(&l, RAW).first().is_some_and(|f| f.has_escape) {
                continue;
            }
            let mine = first_field(&l);
            let theirs = split_ags_line(&l);
            let theirs0 = theirs.first().map(String::as_str);
            if mine != theirs0 {
                mismatches += 1;
                if mismatches <= 8 {
                    eprintln!("FIRST differs on {l:?}: mine={mine:?} theirs={theirs0:?}");
                }
            }
        }
        assert_eq!(mismatches, 0, "{mismatches} first-field mismatches");
    }
}
