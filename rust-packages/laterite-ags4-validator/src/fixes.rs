//! Safe automatic fixes for a subset of AGS4 rule violations.
//!
//! This is a **separate** surface from [`crate::findings`] on purpose:
//! the `Finding`/`Location` JSON is a byte-faithful oracle (cross-checked
//! against python-ags4 and guarded by `line_only_finding_serializes_minimally`),
//! so a fix is never a field on a finding. Instead [`compute_fixes`] reads
//! the parsed file + the findings and emits a parallel `Vec<Fix>`; the UI
//! previews them and hands the user-selected subset back to [`apply_fixes`].
//! The validator engine (and its parity tests) are wholly untouched.
//!
//! **Scope = the full *safe* fix set.** A fix is included only when its
//! correct rewrite is unambiguous from the file alone:
//!
//!  * **Rule 2a** — normalise every line ending to CRLF (byte-level).
//!  * **Rule 1 (BOM)** — strip a leading UTF-8 BOM (byte-level, safe).
//!  * **Rule 1 (non-ASCII)** — RISKY transliterate: `ascii_fold` (deunicode)
//!    folds every non-ASCII char to a sensible ASCII equivalent (µ→"u",
//!    °→"deg", ß→"ss", accents→base) and the un-representable — incl. the
//!    U+FFFD corruption marker — to "?". A guess, so opt-in only.
//!  * **Rule 6** — delete an embedded CR inside a row.
//!  * **Rule 7** — rename a duplicate HEADING `X` to `X_1` (or `X_2` …).
//!    *Conditionally* safe: it can surface a Rule 9 unknown-heading
//!    finding, so the label flags that — we don't hide it.
//!  * **Rule 11a/11b** — insert the spec-default `TRAN_DLIM` (`|`) /
//!    `TRAN_RCON` (`+`) when missing.
//!  * **Rule 8** — reformat a numeric cell (nDP/nSCI/nSF) to its declared
//!    precision, but *only* when the cell actually parses as `f64` (never
//!    touch a non-numeric cell — that's a different, unsafe defect). A DT cell
//!    in an ISO-declared column whose value parses as a recognisable date but
//!    isn't ISO gets a canonicalisation to ISO 8601 instead, **Safe** when the
//!    day-first reading is unambiguous (day > 12, day == month, ISO/year-first,
//!    or a spelled month) and **Risky/opt-in** only when a numeric day-month
//!    value is genuinely mm/dd-ambiguous (day ≤ 12 and day ≠ month — a guess).
//!
//! Apply ordering + the expected-value guard live in [`apply_fixes`]: a
//! span carries the text it *expects* to find, so a stale/over-applied
//! edit is skipped rather than corrupting the file.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::findings::{Findings, Target};
use crate::parse::{ParsedFile, field_span};
use crate::rules::typed_values::{format_ndp, format_nsci, format_nsf};

/// One in-line text edit: replace the char range `[start, end)` on a
/// 1-based source `line` with `replacement`. `expected` is the text the
/// span is believed to currently hold — [`apply_fixes`] compares the live
/// slice against it and skips the edit if they differ, so a fix computed
/// against a now-stale line can never silently corrupt content. Char
/// offsets are Unicode scalars (not bytes), matching [`field_span`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpanEdit {
    pub line: u32,
    pub start: u32,
    pub end: u32,
    pub replacement: String,
    pub expected: String,
}

/// Which kind of fix this is — drives the UI grouping/label and tells the
/// applier whether the work is byte-level (BOM / CRLF, no spans) or a set
/// of in-line [`SpanEdit`]s. `snake_case` in JSON to match the TS union.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FixKind {
    NormalizeCrlf,
    StripBom,
    StripEmbeddedCr,
    RenameDuplicateHeading,
    InsertTranDlim,
    InsertTranRcon,
    ReformatNumeric,
    CanonicalizeDatetime,
    NormalizeTypography,
    PadShortRow,
}

/// How confident the fix is. `Safe` rewrites are unambiguous from the file
/// alone and apply with `fix-all-safe`; `Risky` ones *guess intent* (a lossy
/// or potentially-surprising rewrite) and are excluded from the bulk action —
/// the UI surfaces them separately for explicit opt-in. The field is what
/// lets the engine offer fixes that were previously skipped as "unsafe to
/// auto-apply" (e.g. typographic→ASCII substitution) instead of withholding
/// them entirely. `snake_case` in JSON to match the TS union.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FixRisk {
    Safe,
    Risky,
}

/// One proposed fix. `edits` is empty for the two byte-level kinds
/// (`NormalizeCrlf`/`StripBom` operate on the whole document, not a span).
/// `rule` is the exact rule-label const (`"AGS Format Rule 8"`, …) so the
/// UI can cross-link back to the originating finding section. `line` is
/// the anchor line for ordering/preview (`None` for whole-file kinds).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Fix {
    pub kind: FixKind,
    pub label: String,
    pub rule: String,
    pub line: Option<u32>,
    /// Safe (bulk-applicable) vs risky (opt-in only) — see [`FixRisk`].
    pub risk: FixRisk,
    pub edits: Vec<SpanEdit>,
}

pub type Fixes = Vec<Fix>;

// Rule-label consts. Kept local (the rule modules' own consts are
// private) — these must stay string-identical to them; the compute tests
// exercise each so drift is caught.
const RULE_1: &str = "AGS Format Rule 1";
const RULE_2A: &str = "AGS Format Rule 2a";
const RULE_4: &str = "AGS Format Rule 4";
const RULE_6: &str = "AGS Format Rule 6";
const RULE_7: &str = "AGS Format Rule 7";
const RULE_8: &str = "AGS Format Rule 8";
const RULE_11A: &str = "AGS Format Rule 11a";
const RULE_11B: &str = "AGS Format Rule 11b";

/// The rule suffixes the fix engine can repair (the `RULE_*` consts above, sans
/// the `"AGS Format Rule "` prefix) — the single source for the catalogue's
/// `fixable` flag (`crate::catalogue`). The `fixable_labels_match_rule_consts`
/// test keeps it in lock-step with the consts, so a new fix can't leave the
/// catalogue's `fixable` stale.
pub const FIXABLE_RULE_LABELS: &[&str] = &["1", "2a", "4", "6", "7", "8", "11a", "11b"];

/// Walk the findings + the parsed file and emit one [`Fix`] per fixable
/// finding. Pure read — never mutates `parsed` or `found` (the oracle
/// stays intact). Findings whose fix would be ambiguous/unsafe (Rule 1
/// non-ASCII, any rule not listed) are simply skipped.
pub fn compute_fixes(parsed: &ParsedFile, found: &Findings) -> Fixes {
    // Raw-line text by 1-based number, so a finding → its on-line span is
    // O(1). `field_span` runs over this raw text (quotes + tag intact).
    let line_text: HashMap<u32, &str> = parsed
        .raw_lines
        .iter()
        .map(|rl| (rl.number, rl.text.as_str()))
        .collect();

    let mut fixes: Fixes = Vec::new();

    // -- Rule 2a: one whole-file CRLF normalisation if ANY line missed it
    //    (an LF-only line, or an unterminated final line). The per-line
    //    findings collapse to a single byte-level fix.
    if found.contains_key(RULE_2A) && parsed.raw_lines.iter().any(|rl| !rl.had_crlf) {
        fixes.push(Fix {
            kind: FixKind::NormalizeCrlf,
            label: "Normalise all line endings to CRLF (Rule 2a)".to_string(),
            rule: RULE_2A.to_string(),
            line: None,
            risk: FixRisk::Safe,
            edits: Vec::new(),
        });
    }

    // -- Rule 1: BOM only. The non-ASCII >255 arm is unfixable; we tell
    //    the two apart by the flag, not by parsing the desc.
    if parsed.has_bom && found.contains_key(RULE_1) {
        fixes.push(Fix {
            kind: FixKind::StripBom,
            label: "Strip the UTF-8 byte-order mark (Rule 1)".to_string(),
            rule: RULE_1.to_string(),
            line: None,
            risk: FixRisk::Safe,
            edits: Vec::new(),
        });
    }

    // -- Rule 6: delete each embedded CR/LF (the finding carries its span).
    //    Rule 6 bans both CR and LF within a row, and the quote-aware splitter
    //    now keeps an embedded LF in the field too (#422) — so delete exactly
    //    the char that's there (`expected` must match on apply, else the LF
    //    left after stripping a CR of a `\r\n` pair would never converge). The
    //    per-char fix iterates: `\r\n` strips over two passes (well within the
    //    bounded-fixpoint budget).
    if let Some(items) = found.get(RULE_6) {
        for f in items {
            let (Some(line), Some((s, e))) = (f.line, f.location.char_span) else {
                continue;
            };
            let Some(ch) = line_text
                .get(&line)
                .and_then(|t| t.chars().nth(s as usize))
                .filter(|c| *c == '\r' || *c == '\n')
            else {
                continue; // span doesn't point at a CR/LF (stale) → skip
            };
            fixes.push(Fix {
                kind: FixKind::StripEmbeddedCr,
                label: format!("Delete embedded CR/LF on line {line} (Rule 6)"),
                rule: RULE_6.to_string(),
                line: Some(line),
                risk: FixRisk::Safe,
                edits: vec![SpanEdit {
                    line,
                    start: s,
                    end: e,
                    replacement: String::new(),
                    expected: ch.to_string(),
                }],
            });
        }
    }

    // -- Rule 7: rename the LATER duplicate heading. The finding carries
    //    the duplicate name (Target::Heading + heading) but no field
    //    index, so we re-derive the offending field from the HEADING row.
    if let Some(items) = found.get(RULE_7) {
        for f in items {
            if f.location.target != Target::Heading {
                continue; // the order-facet findings (Rule 7_2) aren't fixable here
            }
            let (Some(line), Some(dup)) = (f.line, f.location.heading.as_deref()) else {
                continue;
            };
            // Find the group whose HEADING row is on this line, to know the
            // sibling names (for an X_1-taken collision) + the field index.
            let Some(group) = parsed
                .groups
                .values()
                .find(|g| g.heading_line == Some(line))
            else {
                continue;
            };
            // The LATER occurrence is the second match of `dup`.
            let mut seen = 0usize;
            let Some(ci) = group.headings.iter().position(|h| {
                if h == dup {
                    seen += 1;
                    seen == 2
                } else {
                    false
                }
            }) else {
                continue;
            };
            // Pick X_1, or X_2 … if a sibling already owns the lower index.
            let mut suffix = 1u32;
            let new_name = loop {
                let cand = format!("{dup}_{suffix}");
                if !group.headings.iter().any(|h| h == &cand) {
                    break cand;
                }
                suffix += 1;
            };
            let Some(raw) = line_text.get(&line) else {
                continue;
            };
            // `ci` is a heading's column index within one AGS4 group — bounded
            // by that group's heading count (dictionary-bounded, a few dozen
            // at most), nowhere near u32::MAX.
            #[allow(clippy::cast_possible_truncation)]
            let Some((s, e)) = field_span(raw, ci as u32) else {
                continue;
            };
            fixes.push(Fix {
                kind: FixKind::RenameDuplicateHeading,
                label: format!(
                    "Rename duplicate heading {dup}→{new_name} \
                     (may surface a Rule 9 unknown-heading finding)"
                ),
                rule: RULE_7.to_string(),
                line: Some(line),
                // Risky: the rename can introduce a fresh Rule 9 finding (the
                // renamed heading isn't in the dictionary), so it's opt-in,
                // not part of fix-all-safe.
                risk: FixRisk::Risky,
                edits: vec![SpanEdit {
                    line,
                    start: s,
                    end: e,
                    replacement: new_name,
                    expected: dup.to_string(),
                }],
            });
        }
    }

    // -- Rule 11a / 11b: insert the spec-default delimiter / concatenator
    //    into the TRAN DATA row's TRAN_DLIM / TRAN_RCON cell. The findings
    //    are line-only (whole-row), so re-derive the exact cell span from
    //    the ParsedFile, matching how rule_11 itself locates the column.
    if found.contains_key(RULE_11A) || found.contains_key(RULE_11B) {
        if let Some(tran) = parsed.groups.get("TRAN") {
            if let Some(data) = tran.rows.first() {
                let raw = line_text.get(&data.line).copied();
                let mut insert = |col: &str, repl: &str, rule: &str, kind: FixKind, label: &str| {
                    let (Some(ci), Some(raw)) = (tran.headings.iter().position(|h| h == col), raw)
                    else {
                        return;
                    };
                    // Only propose when the cell is actually empty (the
                    // finding fired). field_span gives the inner span — for
                    // an empty `""` field that's a zero-width point just
                    // inside the quotes, exactly where the value belongs.
                    let cur = data.values.get(ci).map_or("", String::as_str);
                    if !cur.is_empty() {
                        return;
                    }
                    // `ci` is TRAN_DLIM/TRAN_RCON's column index within the
                    // TRAN group — bounded by TRAN's heading count
                    // (dictionary-bounded), nowhere near u32::MAX.
                    #[allow(clippy::cast_possible_truncation)]
                    let Some((s, e)) = field_span(raw, ci as u32) else {
                        return;
                    };
                    fixes.push(Fix {
                        kind,
                        label: label.to_string(),
                        rule: rule.to_string(),
                        line: Some(data.line),
                        risk: FixRisk::Safe,
                        edits: vec![SpanEdit {
                            line: data.line,
                            start: s,
                            end: e,
                            replacement: repl.to_string(),
                            expected: String::new(), // empty cell
                        }],
                    });
                };
                if found.contains_key(RULE_11A) {
                    insert(
                        "TRAN_DLIM",
                        "|",
                        RULE_11A,
                        FixKind::InsertTranDlim,
                        "Insert the default record-link delimiter \"|\" into TRAN_DLIM (Rule 11a)",
                    );
                }
                if found.contains_key(RULE_11B) {
                    insert(
                        "TRAN_RCON",
                        "+",
                        RULE_11B,
                        FixKind::InsertTranRcon,
                        "Insert the default record-link concatenator \"+\" into TRAN_RCON (Rule 11b)",
                    );
                }
            }
        }
    }

    // -- Rule 8: reformat a numeric cell to its declared precision, only
    //    when the cell parses as f64 (never touch a non-numeric defect).
    if let Some(items) = found.get(RULE_8) {
        for f in items {
            if f.location.target != Target::Cell {
                continue;
            }
            let (Some(line), Some(ci)) = (f.line, f.location.field_index) else {
                continue;
            };
            let Some(group) = parsed.groups.get(&f.group) else {
                continue;
            };
            let ty = group.types.get(ci as usize).map_or("", String::as_str);
            let raw = match line_text.get(&line) {
                Some(r) => *r,
                None => continue,
            };
            let Some((s, e)) = field_span(raw, ci) else {
                continue;
            };
            // The current cell value, sliced from the raw line so the
            // expected-guard matches exactly what's on disk.
            let cur: String = raw
                .chars()
                .skip(s as usize)
                .take((e - s) as usize)
                .collect();

            // Two reformats hang off a Rule 8 cell finding. A numeric cell that
            // parses as f64 gets a SAFE reprecision. A DT cell whose value parses
            // as a recognisable date but isn't ISO gets an ISO canonicalisation
            // whose risk is PER-VALUE: Safe when the day-first reading is
            // unambiguous (day > 12, day == month, ISO/year-first, or a spelled
            // month), Risky only when a numeric day-month value is genuinely
            // mm/dd-ambiguous (day <= 12 && day != month) — see `datetime_to_iso`.
            let (new_val, kind, risk) = if let Some(reformat) = numeric_reformat(ty) {
                let Ok(v) = cur.trim().parse::<f64>() else {
                    continue; // not numeric → unsafe to reformat, skip
                };
                (reformat(v), FixKind::ReformatNumeric, FixRisk::Safe)
            } else if ty == "DT" {
                let unit = group.units.get(ci as usize).map_or("", String::as_str);
                let Some((iso, risk)) = datetime_to_iso(unit, cur.trim()) else {
                    continue; // non-ISO declared UNIT, or value isn't a real date
                };
                (iso, FixKind::CanonicalizeDatetime, risk)
            } else {
                continue; // ID/T/U/YN/DMS/X/… have no value reformat
            };
            if new_val == cur {
                continue; // already correct (cheap guard)
            }
            let label = if kind == FixKind::CanonicalizeDatetime {
                format!("Canonicalise datetime {cur:?} → {new_val:?} (ISO 8601, Rule 8)")
            } else {
                format!("Reformat {cur:?} → {new_val:?} to match TYPE {ty} (Rule 8)")
            };
            fixes.push(Fix {
                kind,
                label,
                rule: RULE_8.to_string(),
                line: Some(line),
                risk,
                edits: vec![SpanEdit {
                    line,
                    start: s,
                    end: e,
                    replacement: new_val,
                    expected: cur,
                }],
            });
        }
    }

    // -- Rule 1 (non-ASCII): RISKY transliterate→ASCII. The non-ASCII arm of
    //    Rule 1 was previously left unfixed — there's no *safe* rewrite for an
    //    arbitrary >127 character without guessing intent. The risk field lets
    //    us offer it opt-in instead of withholding it: `ascii_fold` (deunicode)
    //    picks a sensible ASCII equivalent for every non-ASCII char (µ→"u",
    //    °→"deg", ß→"ss", accents→base) and folds the un-representable — incl.
    //    the U+FFFD corruption marker — to "?". Each substitution is its own
    //    expected-guarded SpanEdit, so a stale line simply no-ops.
    if let Some(items) = found.get(RULE_1) {
        for f in items {
            let Some(line) = f.line else { continue };
            let Some(raw) = line_text.get(&line) else {
                continue;
            };
            // `i` is a char offset within ONE AGS4 line — bounded by that
            // line's length, which cannot realistically reach u32::MAX
            // chars for real geotechnical data.
            #[allow(clippy::cast_possible_truncation)]
            let edits: Vec<SpanEdit> = raw
                .chars()
                .enumerate()
                .filter_map(|(i, c)| {
                    ascii_fold(c).map(|repl| SpanEdit {
                        line,
                        start: i as u32,
                        end: i as u32 + 1,
                        // A fold that yields `"` (e.g. a curly double-quote →
                        // straight quote) sits INSIDE a quoted AGS field, where a
                        // bare `"` reads as an early field terminator — truncating
                        // the cell and dropping everything after it. AGS4 escapes a
                        // literal quote by doubling it, so double any `"` produced.
                        replacement: repl.replace('"', "\"\""),
                        expected: c.to_string(),
                    })
                })
                .collect();
            if edits.is_empty() {
                continue; // line has no non-ASCII char → nothing to fold
            }
            let n = edits.len();
            fixes.push(Fix {
                kind: FixKind::NormalizeTypography,
                label: format!(
                    "Replace {n} non-ASCII character{} with ASCII on line {line} (Rule 1)",
                    if n == 1 { "" } else { "s" }
                ),
                rule: RULE_1.to_string(),
                line: Some(line),
                risk: FixRisk::Risky,
                edits,
            });
        }
    }

    // -- Rule 4: pad a DATA row that has FEWER fields than its HEADING row
    //    with empty fields so it conforms. SAFE — it only appends empty cells
    //    (existing data untouched, no value invented). A row with TOO MANY
    //    fields is left alone (dropping data would be lossy), as are short
    //    UNIT/TYPE/GROUP rows (their fix is structural, not a blank cell).
    if let Some(items) = found.get(RULE_4) {
        for f in items {
            let Some(line) = f.line else { continue };
            let Some(group) = parsed.groups.get(&f.group) else {
                continue;
            };
            let Some(row) = group.rows.iter().find(|r| r.line == line) else {
                continue; // not a DATA row (UNIT/TYPE/GROUP arity) → skip
            };
            let want = group.headings.len();
            let have = row.values.len();
            if have >= want {
                continue; // exact, or too many — padding doesn't apply
            }
            let Some(raw) = line_text.get(&line) else {
                continue;
            };
            // A malformed row (unterminated quote, or stray chars between a
            // closing quote and the next comma) makes `have` unreliable: the
            // tokenizer SKIPS the dropped content, so the count is short for a
            // reason padding won't fix — appending blanks would paper over lost
            // data. Don't offer the safe pad for those; the Rule 4/7 finding
            // still flags the row for manual repair.
            if !row_is_clean(raw) {
                continue;
            }
            // Append at the end of the line body (CR already stripped in
            // raw_lines), as an empty-slice replacement guarded by "".
            // A char count of ONE AGS4 line — bounded well under u32::MAX.
            #[allow(clippy::cast_possible_truncation)]
            let end = raw.chars().count() as u32;
            let missing = want - have;
            // A trailing comma already opens the slot for one empty field that
            // `split_ags_line` doesn't count — so the FIRST appended cell must
            // omit its leading comma, else `"DATA","BH01",` over-pads to
            // `"DATA","BH01",,""` (one field too many).
            let replacement = if raw.ends_with(',') {
                let mut s = String::from("\"\"");
                s.push_str(&",\"\"".repeat(missing - 1));
                s
            } else {
                ",\"\"".repeat(missing)
            };
            fixes.push(Fix {
                kind: FixKind::PadShortRow,
                label: format!(
                    "Pad DATA row on line {line} with {missing} empty field{} to match the {want}-column HEADING (Rule 4)",
                    if missing == 1 { "" } else { "s" }
                ),
                rule: RULE_4.to_string(),
                line: Some(line),
                risk: FixRisk::Safe,
                edits: vec![SpanEdit {
                    line,
                    start: end,
                    end,
                    replacement,
                    expected: String::new(),
                }],
            });
        }
    }

    fixes
}

/// Walk a raw DATA line the way [`crate::parse::split_ags_line`] does,
/// reporting whether the tokenizer would have to TOLERATE a malformation —
/// an unterminated quote, or stray characters between a closing quote and the
/// next comma. When it does, the dropped content means the parsed field count
/// understates the row, so blindly padding to the heading width would mask the
/// loss; the caller withholds the safe pad for such rows.
fn row_is_clean(line: &str) -> bool {
    let mut chars = line.chars().peekable();
    loop {
        match chars.peek() {
            None => return true,
            Some('"') => {
                chars.next(); // opening quote
                let mut terminated = false;
                loop {
                    match chars.next() {
                        None => break, // ran off the end inside the quote
                        Some('"') => {
                            if chars.peek() == Some(&'"') {
                                chars.next(); // escaped ""
                            } else {
                                terminated = true;
                                break; // closing quote
                            }
                        }
                        Some(_) => {}
                    }
                }
                if !terminated {
                    return false; // unterminated quote
                }
                match chars.peek() {
                    Some(',') => {
                        chars.next();
                    }
                    None => return true,
                    Some(_) => return false, // stray chars after a closing quote
                }
            }
            Some(_) => {
                // Unquoted field — read to the next comma.
                while let Some(&c) = chars.peek() {
                    if c == ',' {
                        break;
                    }
                    chars.next();
                }
                if chars.peek() == Some(&',') {
                    chars.next();
                } else {
                    return true;
                }
            }
        }
    }
}

/// Map a common typographic Unicode character (code point > 255) to its
/// plain-ASCII intent, or `None` for any other non-ASCII char (no
/// unambiguous substitution — left untouched). Drives the RISKY
/// Fold one character to ASCII for the Rule-1 [`FixKind::NormalizeTypography`]
/// fix. `None` for characters already ASCII (nothing to do). Otherwise
/// `deunicode` picks a sensible ASCII equivalent for a *real* character —
/// `µ`→"u", `°`→"deg", `±`→"+-", `ß`→"ss", `Ø`→"O", accents→base letter, even
/// CJK→romanised — with no corpus-biased hand map to keep current. A character
/// with no ASCII form folds to "?", including the `U+FFFD` replacement marker
/// that stands for *already-lost* data (deunicode maps `U+FFFD`→"?" itself). So
/// the file can be folded Rule-1-clean while leaving a visible breadcrumb
/// wherever a value was corrupted upstream. A bare combining mark folds to ""
/// (dropped), which is correct — its base letter is handled on its own.
///
/// The result is guaranteed line-break-free: `deunicode` maps the Unicode
/// line/paragraph separators (U+2028 / U+2029, and NEL U+0085) to `"\n"`, but
/// this fold is spliced INTO a single AGS record where a raw newline would split
/// the line — so any `\r`/`\n` it would produce is folded to a space instead
/// (they are whitespace separators). The produced-`"` case is escaped at the
/// `SpanEdit`; between them the fold can never break a record's structure.
fn ascii_fold(c: char) -> Option<&'static str> {
    if c.is_ascii() {
        return None;
    }
    let r = deunicode::deunicode_char(c).unwrap_or("?");
    if r.contains('\n') || r.contains('\r') {
        return Some(" ");
    }
    Some(r)
}

/// Build the right numeric-reformat closure for a declared TYPE code, or
/// `None` for non-numeric / non-reformattable types. Mirrors the
/// `classify` precision parsing in `typed_values` (SCI before SF before
/// DP so `1SCI` isn't read as a bare digit code).
fn numeric_reformat(code: &str) -> Option<Box<dyn Fn(f64) -> String>> {
    let c = code.trim();
    for (suffix, build) in [
        ("SCI", 0u8), // 0 = sci
        ("SF", 1),    // 1 = sf
        ("DP", 2),    // 2 = dp
    ] {
        if let Some(prefix) = c.strip_suffix(suffix) {
            if !prefix.is_empty() && prefix.bytes().all(|b| b.is_ascii_digit()) {
                if let Ok(n) = prefix.parse::<usize>() {
                    return Some(match build {
                        0 => Box::new(move |v| format_nsci(v, n)),
                        1 => Box::new(move |v| format_nsf(v, n)),
                        _ => Box::new(move |v| format_ndp(v, n)),
                    });
                }
            }
        }
    }
    None
}

/// Which family of layout a loose datetime parsed as — decides whether an ISO
/// canonicalisation is a *guess* (see [`datetime_is_ambiguous`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DateLayout {
    /// Numeric day-first (`%d/%m/%y`, `%d-%m-%Y`, `%d.%m.%Y`, …) — the ONLY
    /// family whose day/month could have been transposed (a mm/dd reading).
    DayFirstNumeric,
    /// ISO / year-first (`%Y-%m-%d`, `%Y/%m/%d`, `%Y-%m-%dT…`) — field order is
    /// fixed by shape, never transposable.
    YearFirst,
    /// Spelled-month (`%d-%b-%Y`, `%d %B %Y`) — the month is a name, so the day
    /// can never be confused with it.
    TextualMonth,
}

/// Whether canonicalising this parsed date is a GUESS. Genuinely mm/dd-ambiguous
/// IFF it came from a numeric day-first layout, `day <= 12` (the day could equally
/// read as a month) and `day != month` (swapping yields a *different* date). A
/// `day > 12` forces day-first; `day == month` is transpose-invariant; ISO and
/// spelled-month layouts are never transposable. The swap is always VALID when
/// `day <= 12` — the month field is already in `1..=12`, a valid day for any
/// month — so no extra range check is needed. Everything this returns `false` for
/// is SAFE to auto-apply.
fn datetime_is_ambiguous(layout: DateLayout, day: u32, month: u32) -> bool {
    layout == DateLayout::DayFirstNumeric && day <= 12 && day != month
}

/// Canonicalise a DT cell value to ISO 8601, tagged with its per-value fix risk.
///
/// Only fires when the column's UNIT declares an ISO pattern (`yyyy-mm-dd`,
/// optionally with a time part): the value is *meant* to be ISO but isn't. The
/// value is parsed by trying a battery of common layouts, re-emitted as
/// `yyyy-mm-dd` (or `yyyy-mm-ddThh:mm:ss` when the UNIT carries a time), and
/// classified [`FixRisk::Safe`] — so `fix()` applies it by default — UNLESS the
/// source was a genuinely mm/dd-ambiguous numeric day-first value
/// ([`datetime_is_ambiguous`]), which stays [`FixRisk::Risky`] (opt-in) because
/// the day-first reading is then a guess. `None` when the UNIT isn't ISO, the
/// value doesn't parse as a real date, or it's already canonical.
fn datetime_to_iso(unit: &str, value: &str) -> Option<(String, FixRisk)> {
    use chrono::{Datelike, Timelike};
    let unit = unit.trim();
    if !unit.starts_with("yyyy-mm-dd") {
        return None; // a non-ISO declared column is out of scope for this fix
    }
    let (dt, layout) = parse_loose_datetime(value)?;
    let out = if unit.len() > "yyyy-mm-dd".len() {
        format!(
            "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}",
            dt.year(),
            dt.month(),
            dt.day(),
            dt.hour(),
            dt.minute(),
            dt.second()
        )
    } else {
        format!("{:04}-{:02}-{:02}", dt.year(), dt.month(), dt.day())
    };
    if out == value {
        return None; // already canonical → no fix regardless of risk
    }
    let risk = if datetime_is_ambiguous(layout, dt.day(), dt.month()) {
        FixRisk::Risky
    } else {
        FixRisk::Safe
    };
    Some((out, risk))
}

/// Parse a datetime that FAILED Rule 8 by trying common non-ISO layouts, also
/// reporting which [`DateLayout`] family matched (so the fixer can tell an
/// ambiguous dd/mm guess from an unambiguous one). Datetime layouts (with a time)
/// are tried before date-only ones; a slash/dash/dotted numeric date is day-first
/// (`%d/%m/%Y`). chrono validates the fields, so an impossible date (month 13,
/// day 32) yields `None` — we never invent a bogus "canonical" value to offer.
fn parse_loose_datetime(s: &str) -> Option<(chrono::NaiveDateTime, DateLayout)> {
    use DateLayout::{DayFirstNumeric, TextualMonth, YearFirst};
    use chrono::{NaiveDate, NaiveDateTime};
    // Ordering matters: a day-first / 2-digit-year (`%y`) layout is tried
    // before the year-first / 4-digit (`%Y`) one for the same separator, so
    // e.g. "18/08/20" matches "%d/%m/%y" (→ 2020-08-18) rather than being read
    // as year 18 by a greedy "%Y". chrono's parse_from_str requires the WHOLE
    // string to match, which is what lets "18/08/2020" fall through to the %Y
    // variant while "18/08/20" stops at %y.
    const WITH_TIME: &[(&str, DateLayout)] = &[
        ("%d/%m/%y %H:%M:%S", DayFirstNumeric),
        ("%d/%m/%y %H:%M", DayFirstNumeric),
        ("%d/%m/%Y %H:%M:%S", DayFirstNumeric),
        ("%d/%m/%Y %H:%M", DayFirstNumeric),
        ("%d-%m-%Y %H:%M:%S", DayFirstNumeric),
        ("%d-%m-%Y %H:%M", DayFirstNumeric),
        ("%Y-%m-%dT%H:%M:%S", YearFirst),
        ("%Y-%m-%d %H:%M:%S", YearFirst),
        ("%Y-%m-%dT%H:%M", YearFirst),
        ("%Y-%m-%d %H:%M", YearFirst),
    ];
    const DATE_ONLY: &[(&str, DateLayout)] = &[
        ("%d/%m/%y", DayFirstNumeric),
        ("%d/%m/%Y", DayFirstNumeric),
        ("%d-%m-%y", DayFirstNumeric),
        ("%d-%m-%Y", DayFirstNumeric),
        ("%d.%m.%y", DayFirstNumeric),
        ("%d.%m.%Y", DayFirstNumeric),
        ("%d-%b-%y", TextualMonth),
        ("%d-%b-%Y", TextualMonth),
        ("%d %b %Y", TextualMonth),
        ("%d %B %Y", TextualMonth),
        ("%Y-%m-%d", YearFirst),
        ("%Y/%m/%d", YearFirst),
    ];
    for (f, layout) in WITH_TIME {
        if let Ok(dt) = NaiveDateTime::parse_from_str(s, f) {
            return Some((dt, *layout));
        }
    }
    for (f, layout) in DATE_ONLY {
        if let Ok(d) = NaiveDate::parse_from_str(s, f) {
            return Some((d.and_hms_opt(0, 0, 0)?, *layout));
        }
    }
    None
}

/// Apply the user-selected `fixes` to `text`, returning the new text.
///
/// Order matters and is deliberate:
///  1. In-line [`SpanEdit`]s run FIRST, grouped by line and applied
///     **right-to-left** (descending `start`) so an earlier edit can't
///     shift a later edit's offsets.
///  2. Each edit is guarded: if the live `[start, end)` slice ≠ its
///     `expected`, the edit is SKIPPED (a stale/overlapping fix never
///     corrupts content). Overlapping edits on one line therefore resolve
///     to "first wins, the rest no-op" — a re-run recomputes the deferred
///     ones against the now-current text.
///  3. Byte-level kinds run LAST: strip the BOM (drop a leading U+FEFF),
///     then normalise every line ending to CRLF.
///
/// Char slicing is code-point correct throughout (`chars()` collected to a
/// `Vec<char>`), so a multibyte line (`°`, `±`) can never be split.
#[must_use]
pub fn apply_fixes(text: &str, has_bom: bool, selected: &[Fix]) -> String {
    // Group in-line edits by line.
    let mut by_line: HashMap<u32, Vec<&SpanEdit>> = HashMap::new();
    let mut strip_bom = false;
    let mut normalize_crlf = false;
    for fix in selected {
        match fix.kind {
            FixKind::StripBom => strip_bom = true,
            FixKind::NormalizeCrlf => normalize_crlf = true,
            _ => {
                for ed in &fix.edits {
                    by_line.entry(ed.line).or_default().push(ed);
                }
            }
        }
    }

    // Rebuild the text line-by-line so each edit's char offsets stay local to
    // its own line (offsets are per-line, as field_span yields). Re-split with
    // the SAME quote-aware line model the parser used (`parse::line_spans`), so
    // a fix's line number lands on the same line here even for `\r`/lone-`\n`-
    // terminated or embedded-newline files — a plain `split('\n')` would number
    // those lines differently and misplace the edit (#422). Each line's
    // ORIGINAL terminator is re-emitted verbatim, so a well-formed file
    // round-trips byte-for-byte; the terminator is never inside `body`, so a CR
    // strip only ever touches embedded content.
    let mut result = String::with_capacity(text.len());
    let src = text.as_bytes();
    for (i, span) in crate::parse::line_spans(src).enumerate() {
        // A 1-based line number for one input file — reaching u32::MAX
        // lines would need ~40+ GB of file content, far beyond any real or
        // stress-test AGS4 file (the perf matrix tops out at ~1GB).
        #[allow(clippy::cast_possible_truncation)]
        let number = (i + 1) as u32;
        // Every terminator/delimiter is ASCII, so `start..body_end` is a valid
        // char boundary of `text`.
        let body = &text[span.start..span.body_end];

        let edited = match by_line.get(&number) {
            None => body.to_string(),
            Some(edits) => {
                let mut chars: Vec<char> = body.chars().collect();
                // Right-to-left so earlier edits don't shift later offsets.
                let mut sorted: Vec<&&SpanEdit> = edits.iter().collect();
                sorted.sort_by_key(|e| std::cmp::Reverse(e.start));
                // Overlap guard: track the lowest already-edited start; an
                // edit overlapping a prior (higher-start) one is deferred.
                let mut min_applied = u32::MAX;
                for ed in sorted {
                    let (s, e) = (ed.start as usize, ed.end as usize);
                    if ed.end > min_applied {
                        continue; // overlaps a later edit already applied → defer
                    }
                    if e > chars.len() || s > e {
                        continue; // out of range → skip defensively
                    }
                    let live: String = chars[s..e].iter().collect();
                    if live != ed.expected {
                        continue; // stale span → skip (don't corrupt)
                    }
                    let repl: Vec<char> = ed.replacement.chars().collect();
                    chars.splice(s..e, repl);
                    min_applied = ed.start;
                }
                chars.into_iter().collect()
            }
        };
        result.push_str(&edited);
        result.push_str(span.term.as_str());
    }

    // Byte-level fixes last. NOTE on the BOM: callers (the wasm layer)
    // hand us text already decoded via `encoding_rs`, which transparently
    // *strips* a leading BOM — so `text` here normally has none. `has_bom`
    // records that the original input carried one. If the user did NOT
    // select the StripBom fix we must re-prepend it so "keep the BOM" is
    // honoured; selecting StripBom (or never having one) leaves it off.
    if has_bom && !strip_bom && !result.starts_with('\u{feff}') {
        result.insert(0, '\u{feff}');
    }
    if strip_bom {
        result = result
            .strip_prefix('\u{feff}')
            .unwrap_or(&result)
            .to_string();
    }
    if normalize_crlf {
        // Split on any of CRLF / CR / LF, rejoin with CRLF, and guarantee a
        // single trailing CRLF (the unterminated-final-line Rule 2a case).
        let normalized = result
            .split("\r\n")
            .flat_map(|s| s.split(['\r', '\n']))
            .collect::<Vec<_>>()
            .join("\r\n");
        // `split` above leaves a trailing "" for an already-terminated file
        // → the join already ends in "\r\n" then ""? No: a trailing "\r\n"
        // produces a final empty segment, so join yields "...\r\n" + "" =
        // "...\r\n". An unterminated file has no final empty segment, so we
        // add one CRLF. Normalise both to exactly one trailing CRLF.
        result = format!("{}\r\n", normalized.trim_end_matches("\r\n"));
    }
    result
}

/// The outcome of [`fix_document`]: the repaired `fixed` bytes (always valid
/// UTF-8 — a UTF-8 source with nothing to fix is returned verbatim, a non-UTF-8
/// source is transcoded even when nothing else changed), the `residual`
/// findings that remain *after* the fixes, the `applied` fixes, and the edition
/// the residual was validated against.
pub struct FixOutcome {
    pub fixed: Vec<u8>,
    pub residual: Findings,
    pub applied: Fixes,
    pub dict_version: crate::DictVersion,
    pub resolution: crate::DictResolution,
    /// How many *risky* fixes (after any `only`/`exclude` selection) were withheld
    /// because `include_risky` was false — i.e. how many more `risky=true` would
    /// apply. `0` when `include_risky` is true. A discoverability signal so a
    /// caller can surface "N more fixable with risky" rather than leaving the user
    /// to guess that an opt-in tier exists.
    pub risky_available: usize,
}

/// Headless one-shot repair of a delivered AGS4 document's bytes — the single
/// orchestration shared by the PyO3 `fix()` verb and the `lat fix` CLI
/// (and the natural home for any future surface). Parses, runs the rules,
/// [`compute_fixes`], applies the **safe** set (plus the intent-guessing
/// **risky** set when `include_risky`), then re-validates the result so
/// [`FixOutcome::residual`] is what could *not* be mechanically fixed.
///
/// The applier re-emits UTF-8 (BOM-sniffed via `opts.encoding`), so repairing a
/// non-UTF-8 / BOM'd / CRLF-broken file also normalises those. The output is
/// always valid UTF-8: when no fix is applicable a UTF-8 source is returned
/// verbatim (byte-for-byte idempotent), but a non-UTF-8 source is still
/// transcoded to UTF-8 — otherwise the "output is always UTF-8" promise would
/// leak raw source bytes, and re-reading them as UTF-8 would diverge from how
/// the source was read.
pub fn fix_document(
    raw: &[u8],
    opts: &crate::CheckOptions,
    include_risky: bool,
) -> Result<FixOutcome, crate::ValidatorError> {
    fix_document_selective(raw, opts, include_risky, None, &[])
}

/// Like [`fix_document`] but restricts which rules' fixes are applied: `only`
/// (when `Some`) keeps just those rule labels, then `exclude` drops any of them.
/// Labels are the short forms (`"8"`, `"2a"`) used by [`FIXABLE_RULE_LABELS`],
/// matched against each fix's `rule` with the `"AGS Format Rule "` prefix
/// stripped. The risk gate still applies first, so a rule whose only fix is
/// risky still needs `include_risky` to apply even when named in `only`.
pub fn fix_document_selective(
    raw: &[u8],
    opts: &crate::CheckOptions,
    include_risky: bool,
    only: Option<&[String]>,
    exclude: &[String],
) -> Result<FixOutcome, crate::ValidatorError> {
    let has_bom = raw.starts_with(&[0xEF, 0xBB, 0xBF]);
    let pf = crate::parse::parse_bytes(raw, opts.encoding)?;
    let tran = crate::tran_ags_of(&pf);
    let (dv, res) = crate::resolve_dict_version(opts.dict_version, tran.as_deref())?;
    let dict = crate::Dictionary::bundled(dv);
    let mut found = Findings::new();
    crate::rules::run_all(&pf, &dict, opts, &mut found);

    let mut selected = compute_fixes(&pf, &found);
    // Per-rule selection applies to the full computed set first (short label).
    if only.is_some() || !exclude.is_empty() {
        selected.retain(|f| {
            let short = f.rule.trim_start_matches("AGS Format Rule ");
            only.is_none_or(|o| o.iter().any(|r| r == short)) && !exclude.iter().any(|r| r == short)
        });
    }
    // Risky fixes (within the selection) withheld for lack of `include_risky` —
    // surfaced on the outcome so a caller learns `risky=true` would repair more.
    let risky_available = if include_risky {
        0
    } else {
        selected.iter().filter(|f| f.risk == FixRisk::Risky).count()
    };
    if !include_risky {
        selected.retain(|f| f.risk == FixRisk::Safe);
    }
    if selected.is_empty() {
        // Even with nothing to fix the output must honor the "always UTF-8"
        // contract. An *already-valid* UTF-8 source is returned verbatim (a no-op
        // fix stays byte-for-byte idempotent — the common case). Everything else
        // is transcoded: returning its raw bytes would leak a non-UTF-8 stream and,
        // once re-read as UTF-8, make `read(enc).fix().validate()` disagree with
        // `read(enc).validate()`. Decode the way the parser did
        // (`decode_without_bom_handling`, BOM stripped) so the emitted bytes
        // re-parse to the same findings. The UTF-8-label case still needs this
        // else-branch when `raw` ISN'T valid UTF-8: the encoding sniffer resolves
        // an unlabelled non-UTF-8 file to UTF_8 and the parser lossy-decodes it, so
        // `raw.to_vec()` there would emit the invalid bytes verbatim (0x80 etc.) —
        // the exact contract breach #416 meant to close, reached via the default
        // `encoding=None` path rather than an explicit non-UTF-8 label.
        let fixed = if std::ptr::eq(opts.encoding, encoding_rs::UTF_8)
            && std::str::from_utf8(raw).is_ok()
        {
            raw.to_vec()
        } else {
            let body = raw.strip_prefix(&[0xEF, 0xBB, 0xBF]).unwrap_or(raw);
            opts.encoding
                .decode_without_bom_handling(body)
                .0
                .into_owned()
                .into_bytes()
        };
        return Ok(FixOutcome {
            fixed,
            residual: found,
            applied: Vec::new(),
            dict_version: dv,
            resolution: res,
            risky_available,
        });
    }

    let (decoded, _, _) = opts.encoding.decode(raw);
    let fixed = apply_fixes(&decoded, has_bom, &selected).into_bytes();

    // Residual findings on the fixed (UTF-8, BOM-stripped) output.
    let pf2 = crate::parse::parse_bytes(&fixed, encoding_rs::UTF_8)?;
    let tran2 = crate::tran_ags_of(&pf2);
    let (dv2, res2) = crate::resolve_dict_version(opts.dict_version, tran2.as_deref())?;
    let dict2 = crate::Dictionary::bundled(dv2);
    let mut residual = Findings::new();
    crate::rules::run_all(&pf2, &dict2, opts, &mut residual);

    Ok(FixOutcome {
        fixed,
        residual,
        applied: selected,
        dict_version: dv2,
        resolution: res2,
        risky_available,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::CheckOptions;
    use crate::dict::{Dictionary, FALLBACK};
    use crate::findings::Findings;
    use crate::parse::parse_str;
    use crate::rules;

    /// Run the full engine on `src` (FYI on) and return (parsed, findings).
    fn check(src: &str) -> (ParsedFile, Findings) {
        let parsed = parse_str(src).expect("fixture parses");
        let dict = Dictionary::bundled(FALLBACK);
        let opts = CheckOptions {
            include_fyi: true,
            ..Default::default()
        };
        let mut found = Findings::new();
        rules::run_all(&parsed, &dict, &opts, &mut found);
        (parsed, found)
    }

    fn kinds(fixes: &Fixes) -> Vec<FixKind> {
        fixes.iter().map(|f| f.kind).collect()
    }

    // A minimal compliant header so the parser accepts the fixture.
    const HEAD: &str = "\"GROUP\",\"PROJ\"\r\n\"HEADING\",\"PROJ_ID\"\r\n\
                        \"UNIT\",\"\"\r\n\"TYPE\",\"ID\"\r\n\"DATA\",\"P1\"\r\n";

    #[test]
    fn rule_2a_lf_only_yields_one_crlf_fix_and_applies() {
        // An LF-only HEADING line (line 2) is the Rule 2a miss.
        let src = "\"GROUP\",\"PROJ\"\r\n\"HEADING\",\"PROJ_ID\"\n\
                   \"UNIT\",\"\"\r\n\"TYPE\",\"ID\"\r\n\"DATA\",\"P1\"\r\n";
        let (parsed, found) = check(src);
        let fixes = compute_fixes(&parsed, &found);
        assert_eq!(kinds(&fixes), vec![FixKind::NormalizeCrlf]);
        let out = apply_fixes(src, parsed.has_bom, &fixes);
        assert!(!out.contains('\n') || out.contains("\r\n"));
        // Every \n is now preceded by \r.
        assert!(out.bytes().zip(out.bytes().skip(1)).all(|(_, _)| true));
        for (i, b) in out.bytes().enumerate() {
            if b == b'\n' {
                assert_eq!(out.as_bytes()[i - 1], b'\r', "LF at {i} not preceded by CR");
            }
        }
        assert!(out.ends_with("\r\n"));
    }

    #[test]
    fn rule_2a_missing_trailing_newline_gets_one() {
        let src = "\"GROUP\",\"PROJ\"\r\n\"HEADING\",\"PROJ_ID\"\r\n\
                   \"UNIT\",\"\"\r\n\"TYPE\",\"ID\"\r\n\"DATA\",\"P1\""; // no trailing nl
        let (parsed, found) = check(src);
        let fixes = compute_fixes(&parsed, &found);
        assert_eq!(kinds(&fixes), vec![FixKind::NormalizeCrlf]);
        let out = apply_fixes(src, parsed.has_bom, &fixes);
        assert!(out.ends_with("\"DATA\",\"P1\"\r\n"));
    }

    #[test]
    fn rule_6_embedded_cr_is_stripped() {
        let src = format!("{HEAD}\"DATA\",\"a\rb\"\r\n");
        let (parsed, found) = check(&src);
        let fixes = compute_fixes(&parsed, &found);
        assert!(kinds(&fixes).contains(&FixKind::StripEmbeddedCr));
        let out = apply_fixes(&src, parsed.has_bom, &fixes);
        // The embedded CR (between a and b) is gone; the terminator stays.
        assert!(!out.contains("\"a\rb\""));
        assert!(out.contains("\"ab\""), "got: {out:?}");
    }

    #[test]
    fn rule_7_renames_later_duplicate_to_x_1() {
        // LOCA with a duplicated LOCA_ID heading.
        let src = "\"GROUP\",\"LOCA\"\r\n\
                   \"HEADING\",\"LOCA_ID\",\"LOCA_ID\"\r\n\
                   \"UNIT\",\"\",\"\"\r\n\"TYPE\",\"ID\",\"ID\"\r\n\
                   \"DATA\",\"BH1\",\"BH1\"\r\n";
        let (parsed, found) = check(src);
        let fixes = compute_fixes(&parsed, &found);
        let f = fixes
            .iter()
            .find(|f| f.kind == FixKind::RenameDuplicateHeading)
            .expect("rename fix");
        assert_eq!(f.edits[0].replacement, "LOCA_ID_1");
        assert_eq!(f.edits[0].expected, "LOCA_ID");
        let out = apply_fixes(src, parsed.has_bom, &fixes);
        assert!(out.contains("\"LOCA_ID\",\"LOCA_ID_1\""), "got: {out:?}");
    }

    #[test]
    fn rule_7_skips_to_x_2_when_x_1_taken() {
        let src = "\"GROUP\",\"LOCA\"\r\n\
                   \"HEADING\",\"LOCA_ID\",\"LOCA_ID\",\"LOCA_ID_1\"\r\n\
                   \"UNIT\",\"\",\"\",\"\"\r\n\"TYPE\",\"ID\",\"ID\",\"ID\"\r\n\
                   \"DATA\",\"BH1\",\"BH1\",\"x\"\r\n";
        let (parsed, found) = check(src);
        let fixes = compute_fixes(&parsed, &found);
        let f = fixes
            .iter()
            .find(|f| f.kind == FixKind::RenameDuplicateHeading)
            .expect("rename fix");
        assert_eq!(f.edits[0].replacement, "LOCA_ID_2");
    }

    #[test]
    fn rule_11a_inserts_default_delimiter() {
        // TRAN with empty TRAN_DLIM, populated TRAN_RCON (isolate 11a).
        let src = "\"GROUP\",\"TRAN\"\r\n\
                   \"HEADING\",\"TRAN_DLIM\",\"TRAN_RCON\"\r\n\
                   \"UNIT\",\"\",\"\"\r\n\"TYPE\",\"X\",\"X\"\r\n\
                   \"DATA\",\"\",\"+\"\r\n";
        let (parsed, found) = check(src);
        let fixes = compute_fixes(&parsed, &found);
        let f = fixes
            .iter()
            .find(|f| f.kind == FixKind::InsertTranDlim)
            .expect("11a fix");
        assert_eq!(f.edits[0].replacement, "|");
        let out = apply_fixes(src, parsed.has_bom, &fixes);
        assert!(out.contains("\"DATA\",\"|\",\"+\""), "got: {out:?}");
    }

    #[test]
    fn rule_11b_inserts_default_concatenator() {
        let src = "\"GROUP\",\"TRAN\"\r\n\
                   \"HEADING\",\"TRAN_DLIM\",\"TRAN_RCON\"\r\n\
                   \"UNIT\",\"\",\"\"\r\n\"TYPE\",\"X\",\"X\"\r\n\
                   \"DATA\",\"|\",\"\"\r\n";
        let (parsed, found) = check(src);
        let fixes = compute_fixes(&parsed, &found);
        let f = fixes
            .iter()
            .find(|f| f.kind == FixKind::InsertTranRcon)
            .expect("11b fix");
        assert_eq!(f.edits[0].replacement, "+");
        let out = apply_fixes(src, parsed.has_bom, &fixes);
        assert!(out.contains("\"DATA\",\"|\",\"+\""), "got: {out:?}");
    }

    #[test]
    fn rule_8_reformats_ndp_nsci_nsf() {
        // Three numeric cells each mis-formatted for their declared TYPE.
        // 2DP: "1.5" → "1.50"; 1SCI: "150" → "1.5e2"; 3SF: "1234" → "1230".
        let src = "\"GROUP\",\"MOND\"\r\n\
                   \"HEADING\",\"MOND_A\",\"MOND_B\",\"MOND_C\"\r\n\
                   \"UNIT\",\"\",\"\",\"\"\r\n\"TYPE\",\"2DP\",\"1SCI\",\"3SF\"\r\n\
                   \"DATA\",\"1.5\",\"150\",\"1234\"\r\n";
        let (parsed, found) = check(src);
        let fixes = compute_fixes(&parsed, &found);
        let repls: Vec<&str> = fixes
            .iter()
            .filter(|f| f.kind == FixKind::ReformatNumeric)
            .map(|f| f.edits[0].replacement.as_str())
            .collect();
        assert!(repls.contains(&"1.50"), "nDP: {repls:?}");
        assert!(repls.contains(&"1.5e2"), "nSCI: {repls:?}");
        assert!(repls.contains(&"1230"), "nSF: {repls:?}");
        let out = apply_fixes(src, parsed.has_bom, &fixes);
        assert!(out.contains("\"1.50\""), "got: {out:?}");
        assert!(out.contains("\"1.5e2\""), "got: {out:?}");
        assert!(out.contains("\"1230\""), "got: {out:?}");
    }

    #[test]
    fn rule_8_never_touches_non_numeric_cell() {
        // A non-numeric value under a numeric TYPE: Rule 8 flags it, but
        // it must NOT yield a ReformatNumeric fix (no safe rewrite).
        let src = "\"GROUP\",\"MOND\"\r\n\
                   \"HEADING\",\"MOND_A\"\r\n\
                   \"UNIT\",\"\"\r\n\"TYPE\",\"2DP\"\r\n\
                   \"DATA\",\"abc\"\r\n";
        let (parsed, found) = check(src);
        assert!(found.contains_key(RULE_8), "Rule 8 should flag 'abc'");
        let fixes = compute_fixes(&parsed, &found);
        assert!(!kinds(&fixes).contains(&FixKind::ReformatNumeric));
    }

    #[test]
    fn apply_is_right_to_left_and_expected_guarded() {
        // Two edits on one line; the right one must apply without
        // disturbing the left one's offsets.
        let line = "\"DATA\",\"AA\",\"BB\"\r\n";
        let src = format!(
            "\"GROUP\",\"X\"\r\n\"HEADING\",\"P\",\"Q\"\r\n\
             \"UNIT\",\"\",\"\"\r\n\"TYPE\",\"X\",\"X\"\r\n{line}"
        );
        let parsed = parse_str(&src).unwrap();
        // Hand-built fixes: edit field 0 (AA→aa) and field 1 (BB→bb) on
        // line 5. field_span on the raw line gives the inner spans.
        let raw = &parsed.raw_lines[4].text;
        let (s0, e0) = field_span(raw, 0).unwrap();
        let (s1, e1) = field_span(raw, 1).unwrap();
        let fixes = vec![
            Fix {
                kind: FixKind::ReformatNumeric,
                label: String::new(),
                rule: RULE_8.to_string(),
                line: Some(5),
                risk: FixRisk::Safe,
                edits: vec![SpanEdit {
                    line: 5,
                    start: s0,
                    end: e0,
                    replacement: "aa".to_string(),
                    expected: "AA".to_string(),
                }],
            },
            Fix {
                kind: FixKind::ReformatNumeric,
                label: String::new(),
                rule: RULE_8.to_string(),
                line: Some(5),
                risk: FixRisk::Safe,
                edits: vec![SpanEdit {
                    line: 5,
                    start: s1,
                    end: e1,
                    replacement: "bb".to_string(),
                    expected: "BB".to_string(),
                }],
            },
        ];
        let out = apply_fixes(&src, false, &fixes);
        assert!(out.contains("\"DATA\",\"aa\",\"bb\""), "got: {out:?}");

        // A stale expected → the edit is skipped, content untouched.
        let stale = vec![Fix {
            kind: FixKind::ReformatNumeric,
            label: String::new(),
            rule: RULE_8.to_string(),
            line: Some(5),
            risk: FixRisk::Safe,
            edits: vec![SpanEdit {
                line: 5,
                start: s0,
                end: e0,
                replacement: "zz".to_string(),
                expected: "WRONG".to_string(),
            }],
        }];
        let out2 = apply_fixes(&src, false, &stale);
        assert!(
            out2.contains("\"DATA\",\"AA\",\"BB\""),
            "stale not skipped: {out2:?}"
        );
    }

    #[test]
    fn strip_bom_drops_it_and_keep_bom_re_prepends() {
        // The wasm layer decodes away the BOM, so `text` has none; has_bom
        // records it existed. StripBom selected → output is BOM-free.
        let src = "\"GROUP\",\"PROJ\"\r\n\"DATA\",\"P1\"\r\n";
        let strip = vec![Fix {
            kind: FixKind::StripBom,
            label: String::new(),
            rule: RULE_1.to_string(),
            line: None,
            risk: FixRisk::Safe,
            edits: Vec::new(),
        }];
        let out = apply_fixes(src, true, &strip);
        assert!(!out.starts_with('\u{feff}'), "BOM should be stripped");

        // No StripBom selected but the original had a BOM → re-prepended,
        // so "keep the BOM" is honoured and the apply is non-destructive.
        let out2 = apply_fixes(src, true, &[]);
        assert!(out2.starts_with('\u{feff}'), "BOM should be preserved");
    }

    #[test]
    fn apply_overlap_defers_second_edit() {
        // Two edits overlapping the same span: first (by descending start)
        // applies, the overlapping one is deferred (a re-run handles it).
        let src = "\"GROUP\",\"X\"\r\n\"HEADING\",\"P\"\r\n\
                   \"UNIT\",\"\"\r\n\"TYPE\",\"X\"\r\n\"DATA\",\"HELLO\"\r\n";
        let parsed = parse_str(src).unwrap();
        let raw = &parsed.raw_lines[4].text;
        let (s, e) = field_span(raw, 0).unwrap(); // "HELLO"
        // Edit A: [s, e) HELLO→hi. Edit B: [s, s+2) HE→XX overlaps A.
        let fixes = vec![Fix {
            kind: FixKind::ReformatNumeric,
            label: String::new(),
            rule: RULE_8.to_string(),
            line: Some(5),
            risk: FixRisk::Safe,
            edits: vec![
                SpanEdit {
                    line: 5,
                    start: s,
                    end: e,
                    replacement: "hi".to_string(),
                    expected: "HELLO".to_string(),
                },
                SpanEdit {
                    line: 5,
                    start: s,
                    end: s + 2,
                    replacement: "XX".to_string(),
                    expected: "HE".to_string(),
                },
            ],
        }];
        let out = apply_fixes(src, false, &fixes);
        // The full-span edit wins; the overlapping one was deferred.
        assert!(out.contains("\"hi\""), "got: {out:?}");
    }

    #[test]
    fn compute_does_not_mutate_findings_and_oracle_intact() {
        let src = "\"GROUP\",\"PROJ\"\r\n\"HEADING\",\"PROJ_ID\"\n\
                   \"UNIT\",\"\"\r\n\"TYPE\",\"ID\"\r\n\"DATA\",\"P1\"\r\n";
        let (parsed, found) = check(src);
        let before = found.clone();
        let _ = compute_fixes(&parsed, &found);
        assert_eq!(found, before, "compute_fixes must not mutate findings");
        // The line-only finding shape is still the historical minimal JSON.
        let mut f = Findings::new();
        crate::findings::add(&mut f, "AGS Format Rule 8", Some(5), "LOCA", "boom");
        assert_eq!(
            serde_json::to_string(&f["AGS Format Rule 8"][0]).unwrap(),
            r#"{"line":5,"group":"LOCA","desc":"boom"}"#
        );
    }

    #[test]
    fn nonascii_fold_is_risky_and_transliterates() {
        // A right single quote (U+2019) + em-dash (U+2014) in a DATA cell trip
        // Rule 1. The non-ASCII arm is a RISKY transliterate→ASCII fix
        // (previously withheld entirely).
        let src = "\"GROUP\",\"PROJ\"\r\n\"HEADING\",\"PROJ_ID\",\"PROJ_NAME\"\r\n\
                   \"UNIT\",\"\",\"\"\r\n\"TYPE\",\"ID\",\"X\"\r\n\
                   \"DATA\",\"P1\",\"O\u{2019}Brien em\u{2014}dash\"\r\n";
        let (parsed, found) = check(src);
        let fixes = compute_fixes(&parsed, &found);
        let fix = fixes
            .iter()
            .find(|f| f.kind == FixKind::NormalizeTypography)
            .expect("a non-ASCII fold fix");
        assert_eq!(fix.risk, FixRisk::Risky, "transliteration is opt-in only");
        assert_eq!(fix.edits.len(), 2, "the ’ and the — are both folded");
        let out = apply_fixes(src, parsed.has_bom, std::slice::from_ref(fix));
        // deunicode: ’→"'", em-dash→"--".
        assert!(out.contains("\"O'Brien em--dash\""), "got: {out:?}");
        // It must never be in the safe set fix-all-safe applies.
        assert!(
            fixes
                .iter()
                .filter(|f| f.kind == FixKind::NormalizeTypography)
                .all(|f| f.risk == FixRisk::Risky),
            "the non-ASCII fold must always be risky"
        );
    }

    #[test]
    fn ascii_fold_output_is_ascii_and_line_break_free_over_all_chars() {
        // Exhaustive: every scalar the fold maps must yield pure ASCII with no
        // line break. A fold to `\r`/`\n` would split a record; the `"`/`,`
        // producers are separately escaped at the SpanEdit (the quote test below
        // + the property suite's cell-preservation invariant cover that).
        for cp in 0u32..=0x0010_FFFF {
            let Some(c) = char::from_u32(cp) else {
                continue;
            };
            if let Some(r) = ascii_fold(c) {
                assert!(r.is_ascii(), "fold of U+{cp:04X} is not ASCII: {r:?}");
                assert!(
                    !r.contains('\r') && !r.contains('\n'),
                    "fold of U+{cp:04X} contains a line break: {r:?}"
                );
            }
        }
    }

    #[test]
    fn nonascii_fold_escapes_produced_double_quote() {
        // A curly DOUBLE quote (U+201C/U+201D) folds to a straight `"`. Since the
        // fold rewrites the raw line in place, and the character sits inside a
        // quoted AGS field, that `"` must be DOUBLED (`""`) or the tokenizer reads
        // it as an early field terminator and TRUNCATES the cell (dropping every
        // character after it). Regression for that data-loss bug.
        let src = "\"GROUP\",\"PROJ\"\r\n\"HEADING\",\"PROJ_ID\",\"PROJ_NAME\"\r\n\
                   \"UNIT\",\"\",\"\"\r\n\"TYPE\",\"ID\",\"X\"\r\n\
                   \"DATA\",\"P1\",\"say \u{201C}hi\u{201D} now\"\r\n";
        let (parsed, found) = check(src);
        let fix = compute_fixes(&parsed, &found)
            .into_iter()
            .find(|f| f.kind == FixKind::NormalizeTypography)
            .expect("a non-ASCII fold fix");
        let out = apply_fixes(src, parsed.has_bom, &[fix]);
        // The produced straight quotes are AGS-escaped (doubled) in the raw text.
        assert!(
            out.contains("\"say \"\"hi\"\" now\""),
            "quote not escaped (cell would truncate): {out:?}"
        );
        // And the field round-trips with no data loss — the trailing " now" is
        // still there (it vanished before the escape fix).
        let (reparsed, _) = check(&out);
        let proj = reparsed.groups.get("PROJ").expect("PROJ group");
        let row = proj
            .rows
            .iter()
            .find(|r| r.values.first().map(String::as_str) == Some("P1"))
            .expect("the P1 DATA row");
        assert_eq!(
            row.values[1], "say \"hi\" now",
            "cell truncated / mis-escaped"
        );
    }

    #[test]
    fn nonascii_fold_handles_symbols_accents_and_corruption() {
        // The real-world case: a micro sign, a degree sign, a German ß, an
        // accented name, and a U+FFFD replacement char (mojibake — already-lost
        // data). deunicode transliterates the real characters and folds the
        // un-representable U+FFFD to "?", so the cell becomes Rule-1 clean.
        let src = "\"GROUP\",\"PROJ\"\r\n\"HEADING\",\"PROJ_ID\",\"PROJ_NAME\"\r\n\
                   \"UNIT\",\"\",\"\"\r\n\"TYPE\",\"ID\",\"X\"\r\n\
                   \"DATA\",\"P1\",\"12\u{00B5}S 30\u{00B0}C Gro\u{00DF} caf\u{00E9} x\u{FFFD}y\"\r\n";
        let (parsed, found) = check(src);
        let fix = compute_fixes(&parsed, &found)
            .into_iter()
            .find(|f| f.kind == FixKind::NormalizeTypography)
            .expect("a non-ASCII fold fix");
        let out = apply_fixes(src, parsed.has_bom, &[fix]);
        assert!(
            out.contains("\"12uS 30degC Gross cafe x?y\""),
            "got: {out:?}"
        );
        // Nothing non-ASCII must survive the fold — the whole point.
        assert!(out.is_ascii(), "folded output must be pure ASCII: {out:?}");
    }

    #[test]
    fn rename_duplicate_heading_is_risky() {
        // Rule 7 duplicate-heading rename can surface a fresh Rule 9 finding,
        // so it's classified risky (excluded from fix-all-safe).
        let src = "\"GROUP\",\"PROJ\"\r\n\"HEADING\",\"PROJ_ID\",\"PROJ_ID\"\r\n\
                   \"UNIT\",\"\",\"\"\r\n\"TYPE\",\"ID\",\"ID\"\r\n\"DATA\",\"P1\",\"P2\"\r\n";
        let (parsed, found) = check(src);
        let fixes = compute_fixes(&parsed, &found);
        let rn = fixes
            .iter()
            .find(|f| f.kind == FixKind::RenameDuplicateHeading)
            .expect("a rename-duplicate-heading fix");
        assert_eq!(rn.risk, FixRisk::Risky);
    }

    #[test]
    fn safe_fixes_stay_safe() {
        // The byte-level + numeric fixes remain Safe so fix-all-safe keeps
        // working unchanged.
        let src = "\"GROUP\",\"PROJ\"\r\n\"HEADING\",\"PROJ_ID\"\n\
                   \"UNIT\",\"\"\r\n\"TYPE\",\"ID\"\r\n\"DATA\",\"P1\"\r\n";
        let (parsed, found) = check(src);
        let fixes = compute_fixes(&parsed, &found);
        assert!(
            fixes
                .iter()
                .filter(|f| f.kind == FixKind::NormalizeCrlf)
                .all(|f| f.risk == FixRisk::Safe),
            "CRLF normalisation stays safe"
        );
    }

    #[test]
    fn pad_short_row_appends_empty_fields() {
        // A DATA row with one value under a 2-column HEADING (Rule 4 too-few)
        // is padded with one empty field; existing data is untouched.
        let src = "\"GROUP\",\"LOCA\"\r\n\"HEADING\",\"LOCA_ID\",\"LOCA_NATE\"\r\n\
                   \"UNIT\",\"\",\"m\"\r\n\"TYPE\",\"ID\",\"2DP\"\r\n\"DATA\",\"BH01\"\r\n";
        let (parsed, found) = check(src);
        let fixes = compute_fixes(&parsed, &found);
        let pad = fixes
            .iter()
            .find(|f| f.kind == FixKind::PadShortRow)
            .expect("a pad-short-row fix");
        assert_eq!(pad.risk, FixRisk::Safe);
        let out = apply_fixes(src, parsed.has_bom, std::slice::from_ref(pad));
        assert!(out.contains("\"DATA\",\"BH01\",\"\""), "got: {out:?}");
    }

    #[test]
    fn pad_short_row_skips_a_too_long_row() {
        // A row with MORE fields than the HEADING is not padded (dropping data
        // would be lossy) — no PadShortRow fix.
        let src = "\"GROUP\",\"LOCA\"\r\n\"HEADING\",\"LOCA_ID\"\r\n\
                   \"UNIT\",\"\"\r\n\"TYPE\",\"ID\"\r\n\"DATA\",\"BH01\",\"extra\"\r\n";
        let (parsed, found) = check(src);
        let fixes = compute_fixes(&parsed, &found);
        assert!(
            !fixes.iter().any(|f| f.kind == FixKind::PadShortRow),
            "a too-long row must not be padded"
        );
    }

    #[test]
    fn pad_short_row_does_not_overshoot_a_trailing_comma() {
        // `"DATA","BH01",` ends on a delimiter: split_ags_line counts ONE
        // value, but the trailing comma already opens the second field's slot.
        // The pad must reach exactly 2 fields (→ ...,"") — not three (...,,"").
        let src = "\"GROUP\",\"LOCA\"\r\n\"HEADING\",\"LOCA_ID\",\"LOCA_NATE\"\r\n\
                   \"UNIT\",\"\",\"m\"\r\n\"TYPE\",\"ID\",\"2DP\"\r\n\"DATA\",\"BH01\",\r\n";
        let (parsed, found) = check(src);
        let fixes = compute_fixes(&parsed, &found);
        let pad = fixes
            .iter()
            .find(|f| f.kind == FixKind::PadShortRow)
            .expect("a pad-short-row fix");
        let out = apply_fixes(src, parsed.has_bom, std::slice::from_ref(pad));
        assert!(out.contains("\"DATA\",\"BH01\",\"\""), "got: {out:?}");
        assert!(!out.contains(",,"), "over-padded a trailing comma: {out:?}");
        // The padded row now re-parses to exactly the 2 heading columns.
        let reparsed = parse_str(&out).expect("padded output parses");
        let row = &reparsed.groups["LOCA"].rows[0];
        assert_eq!(row.values.len(), 2, "padded row should have 2 values");
    }

    #[test]
    fn pad_short_row_withholds_on_a_malformed_quote() {
        // A stray quote (`"a"b"`) makes the tokenizer SKIP `b`; the row counts
        // short for a reason padding won't fix. Withhold the safe pad rather
        // than paper over the dropped data.
        let src = "\"GROUP\",\"LOCA\"\r\n\"HEADING\",\"LOCA_ID\",\"LOCA_NATE\"\r\n\
                   \"UNIT\",\"\",\"m\"\r\n\"TYPE\",\"ID\",\"2DP\"\r\n\"DATA\",\"a\"b\"\r\n";
        let (parsed, found) = check(src);
        let fixes = compute_fixes(&parsed, &found);
        assert!(
            !fixes.iter().any(|f| f.kind == FixKind::PadShortRow),
            "a malformed-quote row must not be auto-padded"
        );
    }

    #[test]
    fn row_is_clean_distinguishes_malformed_rows() {
        assert!(row_is_clean("\"DATA\",\"BH01\""));
        assert!(row_is_clean("\"DATA\",\"BH01\",")); // trailing comma is clean
        assert!(row_is_clean("\"DATA\",\"a\",\"b\""));
        assert!(row_is_clean("DATA,unquoted,fields"));
        assert!(!row_is_clean("\"DATA\",\"a\"b\"")); // stray after closing quote
        assert!(!row_is_clean("\"DATA\",\"BH01")); // unterminated quote
        // An unquoted field must still be scanned to its comma and the REST of
        // the row validated — a malformed tail after it is not clean.
        assert!(!row_is_clean("AB,\"CD")); // unquoted field, then an unterminated quote
    }

    #[test]
    fn datetime_to_iso_canonicalises_common_layouts() {
        // All five are UNAMBIGUOUS → Safe: day > 12 forces dd/mm, or the layout
        // is spelled-month / year-first (never transposable).
        assert_eq!(
            datetime_to_iso("yyyy-mm-dd", "18/08/2020"),
            Some(("2020-08-18".to_string(), FixRisk::Safe))
        );
        assert_eq!(
            datetime_to_iso("yyyy-mm-dd", "18/08/20"),
            Some(("2020-08-18".to_string(), FixRisk::Safe))
        );
        assert_eq!(
            datetime_to_iso("yyyy-mm-dd", "1-Feb-2020"),
            Some(("2020-02-01".to_string(), FixRisk::Safe))
        );
        // A time-bearing ISO UNIT keeps the time.
        assert_eq!(
            datetime_to_iso("yyyy-mm-ddThh:mm:ss", "18/08/2020 13:45:00"),
            Some(("2020-08-18T13:45:00".to_string(), FixRisk::Safe))
        );
        // Missing zero-padding in an otherwise-ISO (year-first) value.
        assert_eq!(
            datetime_to_iso("yyyy-mm-dd", "2020-8-1"),
            Some(("2020-08-01".to_string(), FixRisk::Safe))
        );
    }

    #[test]
    fn datetime_to_iso_refuses_ambiguous_or_invalid_cases() {
        // Already canonical → no fix.
        assert_eq!(datetime_to_iso("yyyy-mm-dd", "2020-08-18"), None);
        // Non-ISO declared UNIT → out of scope (don't rewrite ISO→non-ISO).
        assert_eq!(datetime_to_iso("dd/mm/yyyy", "2020-08-18"), None);
        // Not a real date → never invent a canonical value.
        assert_eq!(datetime_to_iso("yyyy-mm-dd", "32/01/2020"), None);
        assert_eq!(datetime_to_iso("yyyy-mm-dd", "notadate"), None);
        assert_eq!(datetime_to_iso("yyyy-mm-dd", ""), None);
    }

    #[test]
    fn datetime_is_ambiguous_predicate() {
        use DateLayout::*;
        assert!(datetime_is_ambiguous(DayFirstNumeric, 5, 6)); // both ≤12, differ
        assert!(datetime_is_ambiguous(DayFirstNumeric, 1, 2));
        assert!(!datetime_is_ambiguous(DayFirstNumeric, 18, 8)); // day > 12 → forced
        assert!(!datetime_is_ambiguous(DayFirstNumeric, 5, 5)); // day == month → invariant
        assert!(!datetime_is_ambiguous(YearFirst, 1, 8)); // year-first never transposable
        assert!(!datetime_is_ambiguous(TextualMonth, 1, 2)); // spelled month never
    }

    #[test]
    fn datetime_to_iso_risk_matrix() {
        let iso = |v: &str| datetime_to_iso("yyyy-mm-dd", v);
        // Both components ≤ 12 and unequal → genuine dd/mm guess → Risky.
        assert_eq!(
            iso("05/06/2020"),
            Some(("2020-06-05".to_string(), FixRisk::Risky))
        );
        assert_eq!(
            iso("01-02-2020"),
            Some(("2020-02-01".to_string(), FixRisk::Risky))
        );
        // day == month is transpose-invariant → Safe.
        assert_eq!(
            iso("05/05/2020"),
            Some(("2020-05-05".to_string(), FixRisk::Safe))
        );
        // day > 12 forces day-first, year-first never ambiguous → Safe.
        assert_eq!(
            iso("18/08/20"),
            Some(("2020-08-18".to_string(), FixRisk::Safe))
        );
        assert_eq!(
            iso("2020-8-1"),
            Some(("2020-08-01".to_string(), FixRisk::Safe))
        );
        // Risk survives a time part.
        assert_eq!(
            datetime_to_iso("yyyy-mm-dd hh:mm:ss", "05/06/2020 14:30:00"),
            Some(("2020-06-05T14:30:00".to_string(), FixRisk::Risky))
        );
    }

    #[test]
    fn datetime_fix_unambiguous_is_safe_and_applies_iso() {
        // day 18 > 12 forces dd/mm (mm can't be 18) → UNAMBIGUOUS → Safe, so
        // fix-all-safe now canonicalises it BY DEFAULT (the core new behaviour).
        let src = "\"GROUP\",\"PROJ\"\r\n\"HEADING\",\"PROJ_ID\"\r\n\"UNIT\",\"\"\r\n\
                   \"TYPE\",\"ID\"\r\n\"DATA\",\"P1\"\r\n\r\n\
                   \"GROUP\",\"TRAN\"\r\n\"HEADING\",\"TRAN_DATE\"\r\n\
                   \"UNIT\",\"yyyy-mm-dd\"\r\n\"TYPE\",\"DT\"\r\n\"DATA\",\"18/08/2020\"\r\n";
        let (parsed, found) = check(src);
        let fixes = compute_fixes(&parsed, &found);
        let dt = fixes
            .iter()
            .find(|f| f.kind == FixKind::CanonicalizeDatetime)
            .expect("a datetime canonicalisation fix");
        assert_eq!(
            dt.risk,
            FixRisk::Safe,
            "day 18 > 12 → unambiguous dd/mm → safe by default"
        );
        assert_eq!(dt.edits.len(), 1);
        assert_eq!(dt.edits[0].replacement, "2020-08-18");
        assert_eq!(dt.edits[0].expected, "18/08/2020");

        // Pins the behaviour change: the safe canonicalisation lands in the
        // DEFAULT (include_risky = false) fix tier.
        let out = fix_document(src.as_bytes(), &CheckOptions::default(), false).expect("fix");
        assert!(
            !out.applied.is_empty(),
            "safe datetime fix applied by default"
        );
        assert!(
            String::from_utf8_lossy(&out.fixed).contains("2020-08-18"),
            "default fix canonicalises the unambiguous date"
        );
    }

    #[test]
    fn datetime_fix_ambiguous_stays_risky() {
        // 01/02/2020: both ≤ 12 and day != month → genuinely mm/dd-ambiguous →
        // Risky (opt-in), withheld from fix-all-safe.
        let src = "\"GROUP\",\"PROJ\"\r\n\"HEADING\",\"PROJ_ID\"\r\n\"UNIT\",\"\"\r\n\
                   \"TYPE\",\"ID\"\r\n\"DATA\",\"P1\"\r\n\r\n\
                   \"GROUP\",\"TRAN\"\r\n\"HEADING\",\"TRAN_DATE\"\r\n\
                   \"UNIT\",\"yyyy-mm-dd\"\r\n\"TYPE\",\"DT\"\r\n\"DATA\",\"01/02/2020\"\r\n";
        let (parsed, found) = check(src);
        let dt = compute_fixes(&parsed, &found)
            .into_iter()
            .find(|f| f.kind == FixKind::CanonicalizeDatetime)
            .expect("a datetime canonicalisation fix");
        assert_eq!(dt.risk, FixRisk::Risky, "01/02 is a genuine dd/mm guess");
        assert_eq!(dt.edits[0].replacement, "2020-02-01");
        assert_eq!(dt.edits[0].expected, "01/02/2020");
    }

    #[test]
    fn fix_document_noop_returns_original_bytes_verbatim() {
        // Nothing fixable in HEAD, and it is UTF-8 → fix_document returns the
        // input untouched (a no-op stays byte-for-byte idempotent). The non-UTF-8
        // no-op case still transcodes — see the test below.
        let raw = HEAD.as_bytes();
        let out = fix_document(raw, &CheckOptions::default(), false).expect("fixes");
        assert!(out.applied.is_empty());
        assert_eq!(
            out.fixed, raw,
            "a no-op fix on a UTF-8 source returns it byte-for-byte"
        );
    }

    #[test]
    fn fix_document_noop_transcodes_non_utf8_to_utf8() {
        // The no-op path must STILL honor "output is always UTF-8": a clean
        // windows-1252 source (nothing fixable — the é is FYI-only, no safe fix)
        // is transcoded to UTF-8, not passed through as raw non-UTF-8 bytes.
        // Regression for `read(enc).fix().validate()` diverging from
        // `read(enc).validate()`, and for the output silently being invalid UTF-8.
        let utf8 = "\"GROUP\",\"PROJ\"\r\n\"HEADING\",\"PROJ_ID\",\"PROJ_NAME\"\r\n\
                    \"UNIT\",\"\",\"\"\r\n\"TYPE\",\"ID\",\"X\"\r\n\
                    \"DATA\",\"P1\",\"Caf\u{00E9}\"\r\n";
        let raw = encoding_rs::WINDOWS_1252.encode(utf8).0.into_owned();
        assert!(
            std::str::from_utf8(&raw).is_err(),
            "the source is genuinely cp1252 (é = 0xE9, not valid UTF-8)"
        );
        let opts = CheckOptions {
            encoding: encoding_rs::WINDOWS_1252,
            ..CheckOptions::default()
        };
        let out = fix_document(&raw, &opts, false).expect("fixes");
        assert!(out.applied.is_empty(), "expected a no-op (é is FYI-only)");
        let s = std::str::from_utf8(&out.fixed).expect("no-op output must be valid UTF-8");
        assert!(
            s.contains("Caf\u{00E9}"),
            "decoded content preserved: {s:?}"
        );
        assert_ne!(
            out.fixed, raw,
            "a non-UTF-8 source must be transcoded, not returned verbatim"
        );
    }

    #[test]
    fn fix_document_noop_emits_valid_utf8_even_under_a_utf8_label() {
        // The gap #416 left: the no-op fast-path keyed on the encoding *label*
        // (UTF_8 → return verbatim), but the sniffer resolves an unlabelled
        // non-UTF-8 file to UTF_8 and the parser lossy-decodes it — so `raw` can
        // be invalid UTF-8 while `opts.encoding == UTF_8`. Returning it verbatim
        // leaked 0x80 into the "always UTF-8" output (caught by the Python
        // fix-chain property on the default `encoding=None` path). Now the
        // fast-path also requires `raw` to already be valid UTF-8; otherwise the
        // bytes are lossy-decoded the way the parser saw them (0x80 → U+FFFD), so
        // `fix()` output re-parses to the same findings the read did.
        // Build the invalid byte at runtime (a lone 0x80) so it isn't a literal
        // clippy can const-fold into "always errors".
        let mut raw = b"\"GROUP\",\"PROJ\"\r\n\"HEADING\",\"PROJ_ID\",\"PROJ_NAME\"\r\n\
                        \"UNIT\",\"\",\"\"\r\n\"TYPE\",\"ID\",\"X\"\r\n\
                        \"DATA\",\"P1\",\"a"
            .to_vec();
        raw.push(0x80);
        raw.extend_from_slice(b"b\"\r\n");
        assert!(
            std::str::from_utf8(&raw).is_err(),
            "0x80 is not valid UTF-8"
        );
        // encoding == UTF_8 (the default sniff), but the bytes are NOT valid UTF-8.
        let out = fix_document(&raw, &CheckOptions::default(), false).expect("fixes");
        std::str::from_utf8(&out.fixed)
            .expect("no-op output under a UTF-8 label must still be valid UTF-8");
        assert_ne!(
            out.fixed, raw,
            "invalid-UTF-8 bytes must not be returned verbatim"
        );
    }

    #[test]
    fn fix_document_repairs_lf_only_and_clears_rule_2a() {
        // First line ends LF-only → Rule 2a → NormalizeCrlf; the residual must no
        // longer carry Rule 2a once applied.
        let raw = HEAD.replacen("\r\n", "\n", 1);
        let out = fix_document(raw.as_bytes(), &CheckOptions::default(), false).expect("fixes");
        assert_eq!(kinds(&out.applied), vec![FixKind::NormalizeCrlf]);
        assert!(out.fixed.windows(2).any(|w| w == b"\r\n"), "output is CRLF");
        assert!(
            !out.residual.contains_key("AGS Format Rule 2a"),
            "the CRLF finding is resolved in the residual"
        );
    }

    #[test]
    fn fix_document_selective_filters_by_rule() {
        // LF-only line 1 → Rule 2a (NormalizeCrlf) is the one safe fix here.
        let raw = HEAD.replacen("\r\n", "\n", 1);
        let opts = CheckOptions::default();
        // `exclude` the rule → nothing applied, bytes returned verbatim.
        let excl = fix_document_selective(raw.as_bytes(), &opts, false, None, &["2a".to_string()])
            .expect("fixes");
        assert!(excl.applied.is_empty());
        assert_eq!(
            excl.fixed,
            raw.as_bytes(),
            "an excluded fix leaves the bytes untouched"
        );
        // `only` a *different* rule → the 2a fix is withheld too.
        let other =
            fix_document_selective(raw.as_bytes(), &opts, false, Some(&["8".to_string()]), &[])
                .expect("fixes");
        assert!(other.applied.is_empty());
        // `only` the matching rule → it applies.
        let matched =
            fix_document_selective(raw.as_bytes(), &opts, false, Some(&["2a".to_string()]), &[])
                .expect("fixes");
        assert_eq!(kinds(&matched.applied), vec![FixKind::NormalizeCrlf]);
    }

    #[test]
    fn fixable_labels_match_rule_consts() {
        // FIXABLE_RULE_LABELS (the catalogue's single source for `fixable`) must
        // be exactly the rule labels the fix engine attaches — keep it in
        // lock-step with the RULE_* consts compute_fixes uses.
        let from_consts: std::collections::BTreeSet<String> = [
            RULE_1, RULE_2A, RULE_4, RULE_6, RULE_7, RULE_8, RULE_11A, RULE_11B,
        ]
        .iter()
        .map(|l| l.trim_start_matches("AGS Format Rule ").to_string())
        .collect();
        let exported: std::collections::BTreeSet<String> = FIXABLE_RULE_LABELS
            .iter()
            .map(std::string::ToString::to_string)
            .collect();
        assert_eq!(exported, from_consts);
    }
}
