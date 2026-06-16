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
//!  * **Rule 1 (BOM)** — strip a leading UTF-8 BOM (byte-level). The
//!    non-ASCII >255 arm of Rule 1 is *not* fixable (no safe substitution
//!    for a smart-quote/em-dash without guessing intent) — skipped.
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
//!    isn't ISO gets a **risky** canonicalisation to ISO 8601 instead (a slash
//!    date is read dd/mm, the AGS/UK convention — a guess, hence opt-in).
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
/// of in-line [`SpanEdit`]s. snake_case in JSON to match the TS union.
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
/// them entirely. snake_case in JSON to match the TS union.
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

    // -- Rule 6: delete each embedded CR (the finding carries its span).
    if let Some(items) = found.get(RULE_6) {
        for f in items {
            let (Some(line), Some((s, e))) = (f.line, f.location.char_span) else {
                continue;
            };
            fixes.push(Fix {
                kind: FixKind::StripEmbeddedCr,
                label: format!("Delete embedded carriage return on line {line} (Rule 6)"),
                rule: RULE_6.to_string(),
                line: Some(line),
                risk: FixRisk::Safe,
                edits: vec![SpanEdit {
                    line,
                    start: s,
                    end: e,
                    replacement: String::new(),
                    expected: "\r".to_string(),
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
                    let cur = data.values.get(ci).map(String::as_str).unwrap_or("");
                    if !cur.is_empty() {
                        return;
                    }
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
            let ty = group
                .types
                .get(ci as usize)
                .map(String::as_str)
                .unwrap_or("");
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
            // parses as f64 gets a SAFE reprecision. A DT cell whose value
            // parses as a recognisable date but isn't ISO gets a RISKY
            // canonicalisation to ISO 8601 — risky because a slash date like
            // 01/02/2020 is read dd/mm (the AGS/UK convention), which is a guess.
            let (new_val, kind, risk) = if let Some(reformat) = numeric_reformat(ty) {
                let Ok(v) = cur.trim().parse::<f64>() else {
                    continue; // not numeric → unsafe to reformat, skip
                };
                (reformat(v), FixKind::ReformatNumeric, FixRisk::Safe)
            } else if ty == "DT" {
                let unit = group
                    .units
                    .get(ci as usize)
                    .map(String::as_str)
                    .unwrap_or("");
                let Some(iso) = datetime_to_iso(unit, cur.trim()) else {
                    continue; // non-ISO declared UNIT, or value isn't a real date
                };
                (iso, FixKind::CanonicalizeDatetime, FixRisk::Risky)
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

    // -- Rule 1 (non-ASCII): RISKY typographic→ASCII substitution. The
    //    non-ASCII arm of Rule 1 was previously left unfixed — there's no
    //    *safe* rewrite for an arbitrary >255 character without guessing
    //    intent. The risk field lets us offer it opt-in instead of
    //    withholding it: we touch only characters with an unambiguous ASCII
    //    intent (smart quotes, dashes, ellipsis, bullet) and leave any other
    //    non-ASCII char for the user. Each substitution is its own
    //    expected-guarded SpanEdit, so a stale line simply no-ops.
    if let Some(items) = found.get(RULE_1) {
        for f in items {
            let Some(line) = f.line else { continue };
            let Some(raw) = line_text.get(&line) else {
                continue;
            };
            let edits: Vec<SpanEdit> = raw
                .chars()
                .enumerate()
                .filter_map(|(i, c)| {
                    typographic_ascii(c).map(|repl| SpanEdit {
                        line,
                        start: i as u32,
                        end: i as u32 + 1,
                        replacement: repl.to_string(),
                        expected: c.to_string(),
                    })
                })
                .collect();
            if edits.is_empty() {
                continue; // no recognised typographic char → not auto-fixable
            }
            let n = edits.len();
            fixes.push(Fix {
                kind: FixKind::NormalizeTypography,
                label: format!(
                    "Replace {n} typographic character{} with ASCII on line {line} (Rule 1)",
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
/// [`FixKind::NormalizeTypography`] fix.
fn typographic_ascii(c: char) -> Option<&'static str> {
    Some(match c {
        '\u{2018}' | '\u{2019}' | '\u{201A}' | '\u{201B}' => "'", // ‘ ’ ‚ ‛
        '\u{201C}' | '\u{201D}' | '\u{201E}' | '\u{201F}' => "\"", // “ ” „ ‟
        '\u{2013}' | '\u{2014}' | '\u{2015}' | '\u{2212}' => "-", // – — ― −
        '\u{2026}' => "...",                                      // …
        '\u{2022}' => "*",                                        // •
        _ => return None,
    })
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

/// Canonicalise a DT cell value to ISO 8601 — the RISKY datetime fixer.
///
/// Only fires when the column's UNIT declares an ISO pattern (`yyyy-mm-dd`,
/// optionally with a time part): the value is *meant* to be ISO but isn't.
/// The value is parsed by trying a battery of common layouts (a slash/dotted
/// date is read **dd/mm**, the AGS/UK convention — that's the guess that makes
/// this risky), then re-emitted as `yyyy-mm-dd`, or `yyyy-mm-ddThh:mm:ss` when
/// the UNIT carries a time. `None` when the UNIT isn't ISO, the value doesn't
/// parse as a real date, or it's already canonical.
fn datetime_to_iso(unit: &str, value: &str) -> Option<String> {
    use chrono::{Datelike, Timelike};
    let unit = unit.trim();
    if !unit.starts_with("yyyy-mm-dd") {
        return None; // a non-ISO declared column is out of scope for this fix
    }
    let dt = parse_loose_datetime(value)?;
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
    (out != value).then_some(out)
}

/// Parse a datetime that FAILED Rule 8 by trying common non-ISO layouts.
/// Datetime layouts (with a time) are tried before date-only ones; a slash or
/// dotted date is day-first (`%d/%m/%Y`). chrono validates the fields, so an
/// impossible date (month 13, day 32) yields `None` — we never invent a bogus
/// "canonical" value to offer.
fn parse_loose_datetime(s: &str) -> Option<chrono::NaiveDateTime> {
    use chrono::{NaiveDate, NaiveDateTime};
    // Ordering matters: a day-first / 2-digit-year (`%y`) layout is tried
    // before the year-first / 4-digit (`%Y`) one for the same separator, so
    // e.g. "18/08/20" matches "%d/%m/%y" (→ 2020-08-18) rather than being read
    // as year 18 by a greedy "%Y". chrono's parse_from_str requires the WHOLE
    // string to match, which is what lets "18/08/2020" fall through to the %Y
    // variant while "18/08/20" stops at %y.
    const WITH_TIME: &[&str] = &[
        "%d/%m/%y %H:%M:%S",
        "%d/%m/%y %H:%M",
        "%d/%m/%Y %H:%M:%S",
        "%d/%m/%Y %H:%M",
        "%d-%m-%Y %H:%M:%S",
        "%d-%m-%Y %H:%M",
        "%Y-%m-%dT%H:%M:%S",
        "%Y-%m-%d %H:%M:%S",
        "%Y-%m-%dT%H:%M",
        "%Y-%m-%d %H:%M",
    ];
    for f in WITH_TIME {
        if let Ok(dt) = NaiveDateTime::parse_from_str(s, f) {
            return Some(dt);
        }
    }
    const DATE_ONLY: &[&str] = &[
        "%d/%m/%y", "%d/%m/%Y", "%d-%m-%y", "%d-%m-%Y", "%d.%m.%y", "%d.%m.%Y", "%d-%b-%y",
        "%d-%b-%Y", "%d %b %Y", "%d %B %Y", "%Y-%m-%d", "%Y/%m/%d",
    ];
    for f in DATE_ONLY {
        if let Ok(d) = NaiveDate::parse_from_str(s, f) {
            return d.and_hms_opt(0, 0, 0);
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

    // Rebuild the text line-by-line so each edit's char offsets stay
    // local to its own line (offsets are per-line, as field_span yields).
    // `split('\n')` then re-join preserves blank/CR state identically to
    // the parser; the trailing `\r` is part of the line text here so a CR
    // strip operates on raw content, never the terminator.
    let mut out_lines: Vec<String> = Vec::new();
    for (i, raw) in text.split('\n').enumerate() {
        let number = (i + 1) as u32;
        // Operate on the line WITHOUT its trailing CR terminator, so a
        // legitimate CRLF terminator survives untouched and only embedded
        // content is edited. Re-attach the CR after editing.
        let had_cr = raw.ends_with('\r');
        let body = raw.strip_suffix('\r').unwrap_or(raw);

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
        out_lines.push(if had_cr {
            format!("{edited}\r")
        } else {
            edited
        });
    }
    let mut result = out_lines.join("\n");

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
        rules::run_all(&parsed, &dict, &opts, None, &mut found);
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
        assert!(out.contains("\"a\rb\"") == false);
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
    fn typography_fix_is_risky_and_substitutes_ascii() {
        // A right single quote (U+2019) + em-dash (U+2014) — both cp > 255 —
        // in a DATA cell trip Rule 1. The non-ASCII arm is now offered as a
        // RISKY typographic→ASCII fix (previously withheld entirely).
        let src = "\"GROUP\",\"PROJ\"\r\n\"HEADING\",\"PROJ_ID\",\"PROJ_NAME\"\r\n\
                   \"UNIT\",\"\",\"\"\r\n\"TYPE\",\"ID\",\"X\"\r\n\
                   \"DATA\",\"P1\",\"O\u{2019}Brien em\u{2014}dash\"\r\n";
        let (parsed, found) = check(src);
        let fixes = compute_fixes(&parsed, &found);
        let typo = fixes
            .iter()
            .find(|f| f.kind == FixKind::NormalizeTypography)
            .expect("a typography fix");
        assert_eq!(typo.risk, FixRisk::Risky, "typography is opt-in only");
        assert_eq!(typo.edits.len(), 2, "the ’ and the — are both substituted");
        let out = apply_fixes(src, parsed.has_bom, &[typo.clone()]);
        assert!(out.contains("\"O'Brien em-dash\""), "got: {out:?}");
        // It must never be in the safe set fix-all-safe applies.
        assert!(
            fixes
                .iter()
                .filter(|f| f.kind == FixKind::NormalizeTypography)
                .all(|f| f.risk == FixRisk::Risky),
            "typography must always be risky"
        );
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
        let out = apply_fixes(src, parsed.has_bom, &[pad.clone()]);
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
        let out = apply_fixes(src, parsed.has_bom, &[pad.clone()]);
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
    }

    #[test]
    fn datetime_to_iso_canonicalises_common_layouts() {
        // dd/mm/yyyy (UK/AGS day-first) into an ISO-declared column.
        assert_eq!(
            datetime_to_iso("yyyy-mm-dd", "18/08/2020").as_deref(),
            Some("2020-08-18")
        );
        // 2-digit year, dashed, dotted, and month-name layouts.
        assert_eq!(
            datetime_to_iso("yyyy-mm-dd", "18/08/20").as_deref(),
            Some("2020-08-18")
        );
        assert_eq!(
            datetime_to_iso("yyyy-mm-dd", "1-Feb-2020").as_deref(),
            Some("2020-02-01")
        );
        // A time-bearing ISO UNIT keeps the time.
        assert_eq!(
            datetime_to_iso("yyyy-mm-ddThh:mm:ss", "18/08/2020 13:45:00").as_deref(),
            Some("2020-08-18T13:45:00")
        );
        // Missing zero-padding in an otherwise-ISO value.
        assert_eq!(
            datetime_to_iso("yyyy-mm-dd", "2020-8-1").as_deref(),
            Some("2020-08-01")
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
    fn datetime_fix_is_risky_and_applies_iso() {
        // A TRAN_DATE DT cell (UNIT yyyy-mm-dd) holding a dd/mm/yyyy value is a
        // Rule 8 miss; the canonicaliser offers an opt-in ISO rewrite.
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
            FixRisk::Risky,
            "dd/mm is a guess → opt-in, excluded from fix-all-safe"
        );
        assert_eq!(dt.edits.len(), 1);
        assert_eq!(dt.edits[0].replacement, "2020-08-18");
        assert_eq!(dt.edits[0].expected, "18/08/2020");

        let out = apply_fixes(src, parsed.has_bom, std::slice::from_ref(dt));
        assert!(out.contains("\"2020-08-18\""), "applied ISO date: {out:?}");
        assert!(!out.contains("18/08/2020"));
    }
}
