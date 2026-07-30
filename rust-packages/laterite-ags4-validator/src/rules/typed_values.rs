//! Typed-value rule: AGS4.1/4.2 Rule 8.
//!
//! CLEAN-ROOM. Implemented from the AGS4 spec (`reports/AGS 4_1.pdf` &
//! `reports/AGS 4_2.pdf` §4.1.1 + the data-TYPE definitions) and
//! behavioural observation. python-ags4 (LGPL-3.0) was read only to
//! learn *which* TYPE codes it validates and *how* it interprets each
//! (facts about the AGS standard, not copyrightable). No code,
//! structure, or wording was copied. The nSF expected-form algorithm
//! is ported from this workspace's own MIT `laterite-ags4-types`
//! (`ags4_str`), which was independently fitted to python-ags4's output.
//!
//! Spec text (verbatim, AGS 4.2 §4.1.1, p.155 — AGS 4.1 identical):
//!
//! * **Rule 8** — "Data VARIABLEs shall be presented in the units of
//!   measurement and type that are described by the appropriate data
//!   field UNIT and data field TYPE defined at the start of the GROUP
//!   within the GROUP HEADER rows."
//!
//! Scope (mirrors python-ags4's `rule_8` for finding-count parity):
//! validates `nDP`, `nSCI`, `nSF`, `DT`, `T`, `U`, `YN`, `DMS`, and
//! `ID`-uniqueness (only for an ID column whose name starts with the
//! GROUP name). `MC`, `RL`, `X`, `XN` are intentionally **not**
//! validated — `RL` is Rule 11's job (V7); `X`/`XN`/`MC` are too broad
//! to constrain. A GROUP with no TYPE row is skipped (nothing to judge;
//! its absence is a Rule 2b/4 finding from V2). See OBSERVATIONS
//! O-11/O-12/O-13.

use crate::findings::{Findings, Location, Severity, Target, add_at};
use crate::parse::ParsedFile;

const RULE_8: &str = "AGS Format Rule 8";

/// What a TYPE-row code asks us to check. Unknown / deliberately
/// unvalidated codes (`X`, `XN`, `MC`, `RL`, `PA`, `PT`, `PU`, …) are
/// `Skip` — Rule 8 doesn't constrain them (python-ags4 doesn't either).
enum Check {
    Dp(usize),
    Sci(usize),
    Sf(usize),
    Dt,
    T,
    U,
    Yn,
    Dms,
    Id,
    Skip,
}

/// Classify a TYPE code precisely (python-ags4 uses loose `'DP' in
/// code` substring tests — equivalent on valid AGS codes, but exact
/// matching avoids misclassifying a stray value; see O-13).
fn classify(code: &str) -> Check {
    let c = code.trim();
    match c {
        "DT" => return Check::Dt,
        "T" => return Check::T,
        "U" => return Check::U,
        "YN" => return Check::Yn,
        "DMS" => return Check::Dms,
        "ID" => return Check::Id,
        _ => {}
    }
    // Numeric forms: a positive-integer prefix + a precision suffix.
    // Order matters — test SCI before SF/DP so "1SCI" isn't read as
    // ending in a bare digit code.
    for (suffix, is_sci, is_sf) in [
        ("SCI", true, false),
        ("SF", false, true),
        ("DP", false, false),
    ] {
        if let Some(prefix) = c.strip_suffix(suffix) {
            if !prefix.is_empty() && prefix.bytes().all(|b| b.is_ascii_digit()) {
                if let Ok(n) = prefix.parse::<usize>() {
                    return if is_sci {
                        Check::Sci(n)
                    } else if is_sf {
                        Check::Sf(n)
                    } else {
                        Check::Dp(n)
                    };
                }
            }
        }
    }
    Check::Skip
}

// `ci`/`ri` are a column/row index within one AGS4 group — both bounded far
// below u32::MAX for any real AGS4 file (dictionary-bounded heading count;
// row count bounded by what's actually held in memory).
#[allow(clippy::cast_possible_truncation)]
pub fn check(parsed: &ParsedFile, found: &mut Findings) {
    for code in &parsed.group_order {
        let g = &parsed.groups[code];
        // No TYPE row → nothing to validate (python aborts the group
        // with an IndexError-and-pass; Rule 2b/4 reports the absence).
        if g.type_line.is_none() {
            continue;
        }

        for (ci, ty) in g.types.iter().enumerate() {
            let unit = g.units.get(ci).map_or("", String::as_str);
            let heading = g.headings.get(ci).map_or("", String::as_str);
            let chk = classify(ty);
            if matches!(chk, Check::Skip) {
                continue;
            }
            if matches!(chk, Check::Id) {
                // ID-uniqueness only applies to the group's own ID
                // column (name starts with the GROUP). Other ID columns
                // are parent references — Rule 10/11 territory (V7).
                if heading.starts_with(code.as_str()) {
                    flag_duplicate_ids(g, ci, code, ty, found);
                }
                continue;
            }

            for (ri, row) in g.rows.iter().enumerate() {
                let Some(v) = row.values.get(ci) else {
                    continue;
                };
                if v.is_empty() {
                    continue; // empty cells are exempt (Rule 10b's job)
                }
                // nSF carries a useful "expected" value (the SF-rounded
                // reference) — compute it alongside the bad flag so we
                // can surface it in the finding desc. python-ags4 emits
                // " (Expected: <nsf-form>)" for the same case; this gives
                // native callers + compat callers the same diagnostic
                // hint without duplicating the rounding logic in compat.
                let mut sf_expected: Option<String> = None;
                let bad = match &chk {
                    Check::Dp(n) => !is_ndp(v, *n),
                    Check::Sci(n) => !is_nsci(v, *n),
                    Check::Sf(n) => {
                        // python skips zeros (sig-figs undefined for 0).
                        match v.trim().parse::<f64>() {
                            Ok(0.0) => false,
                            Ok(f) => {
                                let ref_form = format_nsf(f, *n);
                                if ref_form == *v {
                                    false
                                } else {
                                    sf_expected = Some(ref_form);
                                    true
                                }
                            }
                            Err(_) => true, // non-numeric under nSF
                        }
                    }
                    Check::U => v.trim().parse::<f64>().is_err(),
                    Check::Yn => !matches!(v.as_str(), "Y" | "N" | "y" | "n"),
                    Check::Dms => !is_dms(v),
                    Check::T => !is_elapsed_time(v, unit),
                    Check::Dt => !structural_dt_match(v, unit) || !dt_semantic_ok(v, unit),
                    Check::Id | Check::Skip => unreachable!(),
                };
                if bad {
                    let suffix = sf_expected
                        .as_deref()
                        .map(|e| format!(" (Expected: {e})"))
                        .unwrap_or_default();
                    add_at(
                        found,
                        RULE_8,
                        Some(row.line),
                        code,
                        format!(
                            "Value {v:?} in {heading} does not match its \
                             declared TYPE {ty:?}{}.{suffix}",
                            unit_hint(&chk, unit)
                        ),
                        Location {
                            target: Target::Cell,
                            field_index: Some(ci as u32),
                            heading: Some(heading.to_string()),
                            data_row: Some(ri as u32 + 1),
                            ..Default::default()
                        },
                        Severity::Error,
                    );
                }
            }
        }
    }
}

fn unit_hint(chk: &Check, unit: &str) -> String {
    match chk {
        Check::Dt | Check::T if !unit.is_empty() => format!(" / UNIT {unit:?}"),
        _ => String::new(),
    }
}

/// Every row whose ID value repeats within the group is flagged
/// (python uses `duplicated(keep=False)` — all occurrences, not just
/// the second).
// `ci`/`ri` are bounded the same way as in `check` above.
#[allow(clippy::cast_possible_truncation)]
fn flag_duplicate_ids(
    g: &crate::parse::ParsedGroup,
    ci: usize,
    code: &str,
    ty: &str,
    found: &mut Findings,
) {
    use std::collections::HashMap;
    let mut counts: HashMap<&str, usize> = HashMap::new();
    for row in &g.rows {
        if let Some(v) = row.values.get(ci) {
            if !v.is_empty() {
                *counts.entry(v.as_str()).or_default() += 1;
            }
        }
    }
    for (ri, row) in g.rows.iter().enumerate() {
        if let Some(v) = row.values.get(ci) {
            if !v.is_empty() && counts.get(v.as_str()).copied().unwrap_or(0) > 1 {
                add_at(
                    found,
                    RULE_8,
                    Some(row.line),
                    code,
                    format!("ID value {v:?} in this {ty} column is not unique."),
                    Location {
                        target: Target::Cell,
                        field_index: Some(ci as u32),
                        data_row: Some(ri as u32 + 1),
                        ..Default::default()
                    },
                    Severity::Error,
                );
            }
        }
    }
}

// ---- pattern checks (no `regex` dep — the AGS patterns are tiny) ----

/// `nDP`: `-?\d+\.\d{n}` for n>0; `-?\d+\.?` for 0DP. No scientific,
/// no thousands separators, integer part ≥ 1 digit.
fn is_ndp(s: &str, n: usize) -> bool {
    let b = s.strip_prefix('-').unwrap_or(s);
    if n == 0 {
        let b = b.strip_suffix('.').unwrap_or(b); // optional trailing dot
        return !b.is_empty() && b.bytes().all(|c| c.is_ascii_digit());
    }
    let Some((int, frac)) = b.split_once('.') else {
        return false;
    };
    !int.is_empty()
        && int.bytes().all(|c| c.is_ascii_digit())
        && frac.len() == n
        && frac.bytes().all(|c| c.is_ascii_digit())
}

/// `nSCI`: `-?\d\.\d{n}[eE][+-]?\d+` — exactly one digit before the
/// point, exactly n after, then an exponent.
fn is_nsci(s: &str, n: usize) -> bool {
    let b = s.strip_prefix('-').unwrap_or(s);
    let bytes = b.as_bytes();
    if bytes.len() < n + 4 {
        return false;
    }
    if !bytes[0].is_ascii_digit() || bytes[1] != b'.' {
        return false;
    }
    if !bytes[2..2 + n].iter().all(u8::is_ascii_digit) {
        return false;
    }
    let mut i = 2 + n;
    if i >= bytes.len() || (bytes[i] != b'e' && bytes[i] != b'E') {
        return false;
    }
    i += 1;
    if i < bytes.len() && (bytes[i] == b'+' || bytes[i] == b'-') {
        i += 1;
    }
    i < bytes.len() && bytes[i..].iter().all(u8::is_ascii_digit)
}

/// `DMS`: `-?\d+:[0-5]\d:[0-5]\d\.?\d*` (degrees:minutes:seconds with
/// optional fractional seconds).
fn is_dms(s: &str) -> bool {
    let b = s.strip_prefix('-').unwrap_or(s);
    let parts: Vec<&str> = b.splitn(3, ':').collect();
    if parts.len() != 3 {
        return false;
    }
    let (deg, min, secs) = (parts[0], parts[1], parts[2]);
    if deg.is_empty() || !deg.bytes().all(|c| c.is_ascii_digit()) {
        return false;
    }
    if !is_sexagesimal(min) {
        return false;
    }
    // seconds: [0-5]\d then optional `.` then optional digits
    let (sint, sfrac) = match secs.split_once('.') {
        Some((a, b)) => (a, Some(b)),
        None => (secs, None),
    };
    if !is_sexagesimal(sint) {
        return false;
    }
    match sfrac {
        Some(f) => f.bytes().all(|c| c.is_ascii_digit()),
        None => true,
    }
}

/// Exactly two digits, first in 0–5 (the `[0-5]\d` minute/second
/// field). `\d*\d\d` hour fields are handled separately.
fn is_sexagesimal(s: &str) -> bool {
    let bytes = s.as_bytes();
    bytes.len() == 2 && (b'0'..=b'5').contains(&bytes[0]) && bytes[1].is_ascii_digit()
}

/// `T` (elapsed time). Pattern depends on the UNIT: `hh:mm` →
/// `\d\d+:[0-5]\d`, `mm:ss` → `[0-5]\d:[0-5]\d`, else (incl.
/// `hh:mm:ss`) → `\d+:[0-5]\d:[0-5]\d`.
fn is_elapsed_time(s: &str, unit: &str) -> bool {
    let parts: Vec<&str> = s.split(':').collect();
    match unit.trim() {
        "hh:mm" => {
            parts.len() == 2
                && parts[0].len() >= 2
                && parts[0].bytes().all(|c| c.is_ascii_digit())
                && is_sexagesimal(parts[1])
        }
        "mm:ss" => parts.len() == 2 && is_sexagesimal(parts[0]) && is_sexagesimal(parts[1]),
        // "hh:mm:ss" and the default both want H+:MM:SS.
        _ => {
            parts.len() == 3
                && !parts[0].is_empty()
                && parts[0].bytes().all(|c| c.is_ascii_digit())
                && is_sexagesimal(parts[1])
                && is_sexagesimal(parts[2])
        }
    }
}

/// Structural DT check: every UNIT char maps to exactly one value
/// char — `y/m/d/h/s` → a digit, `+` → `+` or `-` (timezone sign),
/// anything else (`-`, `:`, `T`, space, `Z`) → that literal. Value
/// must be exactly as long as the UNIT (mirrors python-ags4's
/// per-char regex build + `fullmatch`).
fn structural_dt_match(value: &str, unit: &str) -> bool {
    if unit.is_empty() {
        // O-31: an empty UNIT on a DT field means *no declared
        // format*. python-ags4 builds an empty per-char regex and
        // `''.fullmatch(non_empty)` fails, so it flags Rule 8 ("…the
        // specified format () …"). Match that: a non-empty value with
        // no declared format is a structural failure. Non-empty values
        // are exactly what reaches here (the Rule 8 caller skips empty
        // cells), so Rule 8 now fires — closing the O-12 degenerate
        // gap. Non-empty *unrecognised* UNITs stay lenient (O-12).
        return value.is_empty();
    }
    let mut v = value.chars();
    for uc in unit.chars() {
        let Some(vc) = v.next() else { return false };
        let ok = match uc {
            'y' | 'm' | 'd' | 'h' | 's' => vc.is_ascii_digit(),
            '+' => vc == '+' || vc == '-',
            other => vc == other,
        };
        if !ok {
            return false;
        }
    }
    v.next().is_none()
}

// pandas Timestamp range — python-ags4's rule_8 uses pd.to_datetime
// (check.py:770), which returns NaT (→ Rule 8) for any date/datetime
// outside this window. Mirrored from pandas' public Timestamp.min /
// Timestamp.max docs (clean-room: a behavioural constant, NOT ported
// from check.py). AGS DT is ≤second resolution, so the real bounds'
// sub-second tail (…145224193 / …854775807) is dropped — exact for
// every realistic value (the boundary *second* never occurs in a
// geotech survey date). See O-33.
const PANDAS_MIN: &str = "1677-09-21T00:12:43";
const PANDAS_MAX: &str = "2262-04-11T23:47:16";

/// Is `dt` representable as a pandas `Timestamp` (the range python's
/// `pd.to_datetime` accepts before coercing to `NaT`)? A date-only
/// value is lifted to midnight by the caller, so `1677-09-21`
/// (00:00:00 < the 00:12:43 min) correctly fails — matching python's
/// `NaT` for that input.
fn in_pandas_range(dt: chrono::NaiveDateTime) -> bool {
    use chrono::NaiveDateTime;
    let fmt = "%Y-%m-%dT%H:%M:%S";
    let min = NaiveDateTime::parse_from_str(PANDAS_MIN, fmt).expect("const PANDAS_MIN");
    let max = NaiveDateTime::parse_from_str(PANDAS_MAX, fmt).expect("const PANDAS_MAX");
    (min..=max).contains(&dt)
}

/// Fields extracted from a DT value by lexing it through its UNIT
/// pattern. Any field the UNIT didn't ask for is `None`; the
/// caller fills defaults (year missing → no date check; month →
/// 1; day → 1; time → 00:00:00) before building a `NaiveDateTime`.
#[derive(Debug, Default, PartialEq, Eq)]
struct DtFields {
    year: Option<i32>,
    month: Option<u32>,
    day: Option<u32>,
    hour: Option<u32>,
    minute: Option<u32>,
    second: Option<u32>,
}

/// Lex `value` against `unit` token-by-token (mirroring the AGS4
/// pattern grammar): `yyyy`/`yy`/`mm`/`dd`/`hh`/`ss` consume the
/// corresponding number of digits; any other char is a literal that
/// must match exactly. `mm` is **context-sensitive** — month before
/// any `hh` has been seen, minute after.
///
/// Returns `None` if the structural shape doesn't match (digit
/// expected but got something else; value too long/short; literal
/// mismatch). A successful lex doesn't yet validate ranges — the
/// caller checks month 1-12, etc.
fn lex_unit_value(unit: &str, value: &str) -> Option<DtFields> {
    fn read_digits(v: &mut std::iter::Peekable<std::str::Chars>, n: usize) -> Option<u32> {
        let mut acc = 0u32;
        for _ in 0..n {
            let c = v.next()?;
            if !c.is_ascii_digit() {
                return None;
            }
            acc = acc.checked_mul(10)?.checked_add(c.to_digit(10)?)?;
        }
        Some(acc)
    }

    fn consume_run(u: &mut std::iter::Peekable<std::str::Chars>, ch: char) -> usize {
        let mut n = 1;
        while u.peek() == Some(&ch) {
            u.next();
            n += 1;
        }
        n
    }

    let mut u = unit.chars().peekable();
    let mut v = value.chars().peekable();
    let mut fields = DtFields::default();
    let mut seen_hh = false;

    while let Some(uc) = u.next() {
        match uc {
            'y' => {
                let n = consume_run(&mut u, 'y');
                let raw = read_digits(&mut v, n)?;
                fields.year = Some(if n == 2 {
                    // 2-digit year — python's pd.to_datetime convention
                    // (yy ≥ 70 → 19yy; yy < 70 → 20yy). Same as POSIX
                    // strptime %y in glibc.
                    let raw = raw as i32;
                    if raw < 70 { 2000 + raw } else { 1900 + raw }
                } else {
                    raw as i32
                });
            }
            'm' => {
                let n = consume_run(&mut u, 'm');
                if n != 2 {
                    return None;
                } // only `mm` (2-digit) recognised
                let raw = read_digits(&mut v, 2)?;
                if seen_hh {
                    fields.minute = Some(raw);
                } else {
                    fields.month = Some(raw);
                }
            }
            'd' => {
                let n = consume_run(&mut u, 'd');
                if n != 2 {
                    return None;
                }
                let raw = read_digits(&mut v, 2)?;
                fields.day = Some(raw);
            }
            'h' => {
                let n = consume_run(&mut u, 'h');
                if n != 2 {
                    return None;
                }
                let raw = read_digits(&mut v, 2)?;
                fields.hour = Some(raw);
                seen_hh = true;
            }
            's' => {
                let n = consume_run(&mut u, 's');
                if n != 2 {
                    return None;
                }
                let raw = read_digits(&mut v, 2)?;
                fields.second = Some(raw);
            }
            '+' => {
                // TZ-offset sign — accept `+` or `-` in the value.
                let vc = v.next()?;
                if vc != '+' && vc != '-' {
                    return None;
                }
            }
            lit => {
                let vc = v.next()?;
                if vc != lit {
                    return None;
                }
            }
        }
    }
    // Value must be exhausted to match.
    if v.next().is_some() {
        return None;
    }
    Some(fields)
}

/// Semantic DT check: is it a real calendar/clock value, by the
/// pattern its UNIT declares?
///
/// **Stage 8 (DT-format dogfood)**: previously this function had
/// special cases for `hh:mm`, `hh:mm:ss`, `yyyy-mm`, and ISO date /
/// datetime — and silently accepted everything else. That made laterite
/// lenient on European (`dd/mm/yyyy`) and US (`mm/dd/yyyy`) UNITs:
/// `32/01/2020` and `01/13/2020` both passed. python-ags4 has the
/// opposite bug (its mask1 requires `pd.to_datetime(value,
/// format='ISO8601')` to succeed, so it can't validate non-ISO UNITs
/// at all — even valid `01/12/2020` is flagged). The new lex-based
/// approach is **spec-correct for either layout**: walk the UNIT
/// token-by-token, extract calendar fields from the value, and
/// validate ranges + bound to the pandas Timestamp window (O-33).
///
/// Unrecognised UNIT shapes still fall back to lenient `true` —
/// `lex_unit_value` returns `None` only on a structural mismatch,
/// not a wholly-unknown pattern; an unknown pattern that happens to
/// match digit-by-digit gets accepted (the structural check still
/// for the parity catalogue.
///
/// `O-12` (lenient on unrecognised UNIT) is now narrower — only
/// fires when the UNIT contains characters/structure outside the
/// `y/m/d/h/s/+/literal` grammar. `O-31` (empty UNIT) and `O-33`
/// (pandas bound) still apply.
fn dt_semantic_ok(value: &str, unit: &str) -> bool {
    use chrono::NaiveDate;

    // Strip TZ from both sides — the offset (`Z(+hh:mm)`) is not
    // semantically validated; structural_dt_match enforces the shape.
    let base = value.split_once('Z').map_or(value, |(a, _)| a);
    let u = unit.trim();
    let u_base = u.split_once('Z').map_or(u, |(a, _)| a);

    let Some(f) = lex_unit_value(u_base, base) else {
        return false;
    };

    // Range checks. NaiveDate::from_ymd_opt rejects invalid (year,
    // month, day) combinations including leap-year edges; bounding
    // hour/minute/second with explicit ranges lets us tolerate
    // leap-seconds (second == 60) — chrono's %S accepts it and
    // python-ags4's pd.to_datetime does too.
    let year = f.year.unwrap_or(2000);
    let month = f.month.unwrap_or(1);
    let day = f.day.unwrap_or(1);
    let hour = f.hour.unwrap_or(0);
    let minute = f.minute.unwrap_or(0);
    let second = f.second.unwrap_or(0);

    if hour > 23 || minute > 59 || second > 60 {
        return false;
    }

    // Clamp leap-second to 59 for the chrono build (NaiveTime's range
    // is 0-59); the leap-second tolerance is already validated above.
    let s_for_build = second.min(59);

    NaiveDate::from_ymd_opt(year, month, day)
        .and_then(|d| d.and_hms_opt(hour, minute, s_for_build))
        .is_some_and(in_pandas_range)
}

// The nDP / nSCI / nSF expected forms come from the AGS type-system leaf —
// the SAME functions `laterite_ags4_types::ags4_str` uses to WRITE a typed value,
// so the form Rule 8 EXPECTS and the form `build_ags4` EMITS cannot drift
// (#528). All three were a hand-port here, kept honest only by a "ported from
// ags_types::ags4_str" comment: it agreed with the authority, but nothing
// gated it — a validator judging a value by a different formatter than the one
// that writes it. Re-exported at the old path, so Rule 8 above and the fixes
// engine (`fixes.rs`) are unchanged, and the grammar linkage still holds:
// `format_ndp` mirrors `is_ndp` (`-?\d+\.\d{n}`, or a bare integer for 0DP),
// `format_nsci` mirrors `is_nsci` (`-?\d\.\d{n}[eE][+-]?\d+`), and `format_nsf`
// is the never-scientific fixed-point form python-ags4 expects (`0.002` @3SF →
// "0.00200"). The format↔validate inverse proptests below now guard the LEAF's
// formatters against this crate's grammar. (laterite-excel keeps a
// deliberately divergent formatter — by-design, pinned in xcheck-allow.json.)
pub(crate) use laterite_ags4_types::{format_ndp, format_nsci, format_nsf};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::parse_str;

    fn run(src: &str) -> Findings {
        let pf = parse_str(src).expect("fixture parses");
        let mut f = Findings::new();
        check(&pf, &mut f);
        f
    }

    #[test]
    fn is_ndp_enforces_exact_decimal_places() {
        assert!(is_ndp("100.50", 2));
        assert!(is_ndp("-0.05", 2));
        assert!(!is_ndp("100.5", 2)); // one dp, not two
        assert!(!is_ndp("100.500", 2)); // three dp
        assert!(!is_ndp("1e2", 2));
        assert!(is_ndp("5", 0));
        assert!(is_ndp("5.", 0));
        assert!(!is_ndp("5.0", 0));
        assert!(!is_ndp(".5", 2)); // needs an integer part
    }

    #[test]
    fn is_nsci_requires_one_leading_digit_and_exponent() {
        assert!(is_nsci("1.0e2", 1));
        assert!(is_nsci("-3.14E-5", 2));
        assert!(!is_nsci("12.3e4", 1)); // two digits before the point
        assert!(!is_nsci("1.00e2", 1)); // two dp, not one
        assert!(!is_nsci("1.0", 1)); // no exponent
    }

    #[test]
    fn is_dms_and_elapsed_time() {
        assert!(is_dms("12:34:56"));
        assert!(is_dms("-1:05:09.25"));
        assert!(!is_dms("12:60:00")); // minutes out of range
        assert!(is_elapsed_time("100:30", "hh:mm"));
        assert!(is_elapsed_time("01:02:03", "hh:mm:ss"));
        assert!(!is_elapsed_time("1:99", "hh:mm"));
    }

    #[test]
    fn format_nsf_matches_python_ags4_expected_form() {
        assert_eq!(format_nsf(0.002, 3), "0.00200");
        assert_eq!(format_nsf(1234.0, 3), "1230");
        assert_eq!(format_nsf(1.23, 3), "1.23");
        assert_eq!(format_nsf(100.0, 2), "100");
    }

    #[test]
    fn dt_structural_and_semantic() {
        assert!(structural_dt_match("2023-02-22", "yyyy-mm-dd"));
        assert!(!structural_dt_match("2023-2-22", "yyyy-mm-dd")); // wrong width
        assert!(dt_semantic_ok("2023-02-22", "yyyy-mm-dd"));
        assert!(!dt_semantic_ok("2023-02-30", "yyyy-mm-dd")); // not a real date
        assert!(dt_semantic_ok("10:24", "hh:mm"));
        assert!(!dt_semantic_ok("25:00", "hh:mm"));
    }

    #[test]
    fn dt_yyyy_mm_month_precision() {
        // Stage 7c: month-precision DT values were wrongly rejected
        // because the else-clause fell back to %Y-%m-%d on a string
        // like "2023-11". Now an explicit yyyy-mm branch synthesises
        // day-01 and validates the month range.
        assert!(structural_dt_match("2023-11", "yyyy-mm"));
        assert!(dt_semantic_ok("2023-11", "yyyy-mm"));
        assert!(!dt_semantic_ok("2023-13", "yyyy-mm")); // month 13 invalid
        assert!(!dt_semantic_ok("2023-00", "yyyy-mm")); // month 0 invalid
        // Out-of-pandas-range still flags.
        assert!(!dt_semantic_ok("0018-06", "yyyy-mm"));
    }

    #[test]
    fn dt_non_iso_units_spec_correct() {
        // Stage 8 (DT-format dogfood): laterite was lenient on
        // non-ISO UNITs (`dd/mm/yyyy`, `mm/dd/yyyy`, `dd-mm-yyyy`,
        // `dd.mm.yyyy`, `mm-yyyy`) — silently accepting any value.
        // python-ags4 is overly strict in the opposite direction
        // (its mask1 requires ISO-8601 so non-ISO UNITs are
        // un-validatable). The new lex-based check is spec-correct.

        // European dd/mm/yyyy
        assert!(dt_semantic_ok("01/12/2020", "dd/mm/yyyy"));
        assert!(dt_semantic_ok("29/02/2024", "dd/mm/yyyy")); // leap-year ok
        assert!(!dt_semantic_ok("29/02/2023", "dd/mm/yyyy")); // not leap
        assert!(!dt_semantic_ok("32/01/2020", "dd/mm/yyyy")); // day 32
        assert!(!dt_semantic_ok("01/13/2020", "dd/mm/yyyy")); // month 13

        // European dd-mm-yyyy / dd.mm.yyyy
        assert!(dt_semantic_ok("01-12-2020", "dd-mm-yyyy"));
        assert!(!dt_semantic_ok("32-01-2020", "dd-mm-yyyy"));
        assert!(dt_semantic_ok("01.12.2020", "dd.mm.yyyy"));

        // US mm/dd/yyyy
        assert!(dt_semantic_ok("12/01/2020", "mm/dd/yyyy"));
        assert!(dt_semantic_ok("02/29/2024", "mm/dd/yyyy")); // leap day
        assert!(!dt_semantic_ok("13/01/2020", "mm/dd/yyyy")); // month 13
        assert!(dt_semantic_ok("12-01-2020", "mm-dd-yyyy"));

        // Month-first month-precision
        assert!(dt_semantic_ok("12-2020", "mm-yyyy"));
        assert!(dt_semantic_ok("12/2020", "mm/yyyy"));
        assert!(!dt_semantic_ok("13-2020", "mm-yyyy"));

        // 2-digit year (yy ≥ 70 → 19yy, else 20yy)
        assert!(dt_semantic_ok("01/12/20", "dd/mm/yy"));
        assert!(!dt_semantic_ok("01/12/2020", "dd/mm/yy")); // wrong shape

        // Year-only — previously laterite flagged; now accepted (matches
        // python-ags4).
        assert!(dt_semantic_ok("2020", "yyyy"));
        assert!(!dt_semantic_ok("20", "yyyy")); // wrong shape
    }

    #[test]
    fn lex_unit_value_context_sensitive_mm() {
        // `mm` before any `hh` is MONTH; after `hh` is MINUTE.
        let f = lex_unit_value("yyyy-mm-dd", "2020-12-01").unwrap();
        assert_eq!(f.month, Some(12));
        assert_eq!(f.minute, None);

        let f = lex_unit_value("yyyy-mm-ddThh:mm", "2020-12-01T10:30").unwrap();
        assert_eq!(f.month, Some(12));
        assert_eq!(f.minute, Some(30));

        // `mm` alone (no hh) is month.
        let f = lex_unit_value("mm-yyyy", "12-2020").unwrap();
        assert_eq!(f.month, Some(12));
        assert_eq!(f.minute, None);
    }

    #[test]
    fn lex_unit_value_rejects_wrong_shape() {
        // Literal mismatch.
        assert!(lex_unit_value("yyyy-mm-dd", "2020/12/01").is_none());
        // Length mismatch (too short).
        assert!(lex_unit_value("yyyy-mm-dd", "2020-12").is_none());
        // Length mismatch (too long).
        assert!(lex_unit_value("yyyy-mm-dd", "2020-12-01T10:30").is_none());
        // Non-digit where digit expected.
        assert!(lex_unit_value("yyyy", "20XX").is_none());
    }

    #[test]
    fn dt_semantic_bounds_to_pandas_range() {
        // O-33: chrono accepts any year; python's pd.to_datetime NaTs
        // anything outside pandas' Timestamp range. Match python so a
        // corrupt year flags Rule 8 while real survey dates don't.
        assert!(
            !dt_semantic_ok("0018-06-03", "yyyy-mm-dd"),
            "the dogfood defect: a year-0018 date must flag like python"
        );
        assert!(dt_semantic_ok("2025-06-08", "yyyy-mm-dd")); // normal — unchanged
        // Boundaries: pandas min is 1677-09-21 00:12:43, so the date
        // at midnight is below it (python NaTs it too); max date ok.
        assert!(dt_semantic_ok("1678-01-01", "yyyy-mm-dd"));
        assert!(!dt_semantic_ok("1677-09-21", "yyyy-mm-dd"));
        assert!(dt_semantic_ok("2262-04-11", "yyyy-mm-dd"));
        assert!(!dt_semantic_ok("9999-01-01", "yyyy-mm-dd"));
        // Datetime shape: same range gate.
        assert!(dt_semantic_ok("2025-06-08T12:30:00", "yyyy-mm-ddThh:mm:ss"));
        assert!(!dt_semantic_ok(
            "0018-06-03T12:30:00",
            "yyyy-mm-ddThh:mm:ss"
        ));
        // A still-invalid calendar date stays invalid (not masked by
        // the range gate).
        assert!(!dt_semantic_ok("2023-02-30", "yyyy-mm-dd"));
    }

    const CLEAN: &str = "\"GROUP\",\"LOCA\"\r\n\
        \"HEADING\",\"LOCA_ID\",\"LOCA_NATE\",\"LOCA_FDEP\"\r\n\
        \"UNIT\",\"\",\"m\",\"m\"\r\n\
        \"TYPE\",\"ID\",\"2DP\",\"2DP\"\r\n\
        \"DATA\",\"BH01\",\"123456.50\",\"10.00\"\r\n\
        \"DATA\",\"BH02\",\"123457.00\",\"12.50\"\r\n";

    #[test]
    fn well_typed_group_has_no_findings() {
        assert!(run(CLEAN).is_empty());
    }

    #[test]
    fn rule_8_flags_wrong_precision_and_bad_date() {
        let src = "\"GROUP\",\"LOCA\"\r\n\
            \"HEADING\",\"LOCA_ID\",\"LOCA_FDEP\",\"LOCA_ESDG\"\r\n\
            \"UNIT\",\"\",\"m\",\"yyyy-mm-dd\"\r\n\
            \"TYPE\",\"ID\",\"2DP\",\"DT\"\r\n\
            \"DATA\",\"BH01\",\"10.5\",\"2023-13-01\"\r\n";
        let r8 = run(src);
        let v = r8.get(RULE_8).expect("Rule 8");
        // "10.5" is 1dp not 2dp; "2023-13-01" is not a valid date.
        assert!(v.iter().any(|x| x.desc.contains("10.5")), "{v:?}");
        assert!(v.iter().any(|x| x.desc.contains("2023-13-01")), "{v:?}");
        assert!(v.iter().all(|x| x.line == Some(5) && x.group == "LOCA"));
    }

    #[test]
    fn rule_8_flags_non_unique_group_id() {
        let src = "\"GROUP\",\"LOCA\"\r\n\
            \"HEADING\",\"LOCA_ID\"\r\n\"UNIT\",\"\"\r\n\"TYPE\",\"ID\"\r\n\
            \"DATA\",\"BH01\"\r\n\"DATA\",\"BH01\"\r\n";
        let v = run(src);
        let r8 = v.get(RULE_8).expect("Rule 8");
        // Both duplicate rows are flagged (keep=False semantics).
        assert_eq!(
            r8.iter().filter(|x| x.desc.contains("not unique")).count(),
            2,
            "{r8:?}"
        );
    }

    #[test]
    fn rule_8_skips_unconstrained_and_empty() {
        // X/RL not validated; empty cells exempt; no TYPE row → skip.
        let src = "\"GROUP\",\"PROJ\"\r\n\
            \"HEADING\",\"PROJ_ID\",\"PROJ_MEMO\"\r\n\
            \"UNIT\",\"\",\"\"\r\n\"TYPE\",\"ID\",\"X\"\r\n\
            \"DATA\",\"P1\",\"anything at all, 1e9, ??\"\r\n\
            \"DATA\",\"P2\",\"\"\r\n";
        assert!(run(src).is_empty());
    }

    // ---- mutation-sweep additions: routing, finding location, and the
    //      exact operator boundaries the proptest suite never lands on ----

    fn grp(ty: &str, unit: &str, vals: &[&str]) -> String {
        use std::fmt::Write as _;
        let mut s = format!(
            "\"GROUP\",\"TEST\"\n\"HEADING\",\"TEST_VAL\"\n\"UNIT\",\"{unit}\"\n\"TYPE\",\"{ty}\"\n"
        );
        for v in vals {
            writeln!(s, "\"DATA\",\"{v}\"").unwrap();
        }
        s
    }

    /// Each TYPE code must route to ITS checker (the `classify` arm) and flag
    /// only bad values (the `!` in `check`). A deleted arm skips the column, so
    /// a bad value goes unflagged; a deleted `!` flags a good value.
    #[test]
    fn each_type_routes_to_its_checker_and_flags_only_bad_values() {
        assert!(!run(&grp("YN", "", &["Y", "N", "y", "n"])).contains_key(RULE_8));
        assert!(run(&grp("YN", "", &["Maybe"])).contains_key(RULE_8));
        assert!(!run(&grp("DMS", "", &["12:34:56"])).contains_key(RULE_8));
        assert!(run(&grp("DMS", "", &["12:99:00"])).contains_key(RULE_8));
        assert!(!run(&grp("T", "hh:mm", &["100:30"])).contains_key(RULE_8));
        assert!(run(&grp("T", "hh:mm", &["1:99"])).contains_key(RULE_8));
        assert!(!run(&grp("U", "", &["1.5"])).contains_key(RULE_8));
        assert!(run(&grp("U", "", &["abc"])).contains_key(RULE_8));
    }

    /// A Rule 8 finding must point at the right heading and 1-based row, and
    /// carry the UNIT hint for DT/T types (and only when the UNIT is non-empty).
    #[test]
    fn rule_8_finding_location_and_unit_hint() {
        // bad DT on the SECOND data row (ri=1 → data_row 2), non-empty UNIT.
        let f = run(&grp("DT", "yyyy-mm-dd", &["2020-01-01", "2020-13-01"]));
        let finding = &f.get(RULE_8).expect("rule 8 fired")[0];
        assert_eq!(finding.location.heading.as_deref(), Some("TEST_VAL"));
        assert_eq!(finding.location.data_row, Some(2));
        assert!(
            finding.desc.contains("/ UNIT \"yyyy-mm-dd\""),
            "{}",
            finding.desc
        );
        // empty UNIT on a DT still fires (O-31) but carries NO unit hint.
        let f = run(&grp("DT", "", &["2020-01-01"]));
        let finding = &f.get(RULE_8).expect("empty-unit DT fires")[0];
        assert!(!finding.desc.contains("/ UNIT"), "{}", finding.desc);
    }

    /// A duplicate-ID finding carries the Cell target, the column index, and the
    /// 1-based row.
    #[test]
    fn duplicate_id_finding_location() {
        let src = "\"GROUP\",\"LOCA\"\n\"HEADING\",\"LOCA_ID\"\n\"UNIT\",\"\"\n\"TYPE\",\"ID\"\n\"DATA\",\"BH1\"\n\"DATA\",\"BH1\"\n";
        let v = run(src);
        let r8 = v.get(RULE_8).expect("dup id fires");
        let second = r8
            .iter()
            .find(|x| x.location.data_row == Some(2))
            .expect("row 2 flagged");
        assert_eq!(second.location.target, Target::Cell);
        assert_eq!(second.location.field_index, Some(0));
    }

    #[test]
    fn is_nsci_boundaries() {
        assert!(is_nsci("1.5e2", 1));
        assert!(!is_nsci("a.5e2", 1)); // non-digit lead with '.' at [1]
        assert!(!is_nsci("1.5000", 1)); // no exponent marker
        assert!(!is_nsci("1.5e+", 1)); // exponent sign but no digits
    }

    #[test]
    fn is_dms_rejects_nondigit_degrees() {
        assert!(is_dms("12:34:56"));
        assert!(!is_dms("1a:05:09")); // degrees non-digit
    }

    #[test]
    fn is_elapsed_time_boundaries() {
        assert!(is_elapsed_time("12:30", "hh:mm"));
        assert!(!is_elapsed_time("ab:30", "hh:mm")); // hh non-digit
        assert!(!is_elapsed_time("12:99", "hh:mm")); // mm out of range
        assert!(is_elapsed_time("05:09", "mm:ss")); // its own arm
        assert!(!is_elapsed_time("99:30", "mm:ss")); // first field out of range
        assert!(!is_elapsed_time("05:99", "mm:ss")); // second field out of range
        assert!(is_elapsed_time("1:02:03", "hh:mm:ss"));
        assert!(!is_elapsed_time(":02:03", "hh:mm:ss")); // empty hours
        assert!(!is_elapsed_time("1:99:03", "hh:mm:ss")); // mm out
        assert!(!is_elapsed_time("1:02:99", "hh:mm:ss")); // ss out
    }

    #[test]
    fn structural_dt_tz_sign() {
        // A `+` UNIT char accepts `+` or `-` in the value, and nothing else.
        assert!(structural_dt_match("12+05", "hh+hh"));
        assert!(structural_dt_match("12-05", "hh+hh"));
        assert!(!structural_dt_match("12x05", "hh+hh"));
    }

    #[test]
    fn lex_two_digit_year_pivot() {
        // yy < 70 → 20yy; yy >= 70 → 19yy (python pd.to_datetime convention).
        assert_eq!(lex_unit_value("yy", "70").unwrap().year, Some(1970));
        assert_eq!(lex_unit_value("yy", "69").unwrap().year, Some(2069));
    }

    #[test]
    fn lex_tz_sign_accepts_plus_and_minus() {
        assert!(lex_unit_value("hh+", "12+").is_some());
        assert!(lex_unit_value("hh+", "12-").is_some());
        assert!(lex_unit_value("hh+", "12x").is_none());
    }

    #[test]
    fn dt_semantic_time_bounds() {
        assert!(dt_semantic_ok("23:00", "hh:mm")); // hour 23 boundary ok
        assert!(!dt_semantic_ok("24:00", "hh:mm")); // hour 24 rejected
        assert!(dt_semantic_ok("00:59", "hh:mm")); // minute 59 ok
        assert!(!dt_semantic_ok("00:60", "hh:mm")); // minute 60 rejected
        assert!(dt_semantic_ok("00:00:60", "hh:mm:ss")); // leap second ok
        assert!(!dt_semantic_ok("00:00:61", "hh:mm:ss")); // second 61 rejected
    }
}

/// Property-based tests for the Rule 8 typed-value helpers.
///
/// These pin the *format↔validate inverse* that Rule 8's fixes engine
/// depends on: a `format_*` helper must always emit a string its matching
/// `is_*` validator accepts — otherwise the engine would "fix" a cell into
/// a still-invalid form. Also: the pattern validators never panic on
/// arbitrary text, and the DT pandas-range gate is monotone at its bounds.
#[cfg(test)]
mod proptest_suite {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        /// Format↔validate inverse for nDP: `format_ndp(x, n)` always
        /// produces a string `is_ndp(_, n)` accepts, for any finite `f64`
        /// and `n` in the realistic 0..=15 precision range. This is the
        /// guarantee the nDP fixer relies on.
        #[test]
        fn format_ndp_is_accepted_by_is_ndp(
            x in prop::num::f64::NORMAL | prop::num::f64::ZERO | prop::num::f64::SUBNORMAL,
            n in 0usize..=15,
        ) {
            let formatted = format_ndp(x, n);
            prop_assert!(
                is_ndp(&formatted, n),
                "format_ndp({x}, {n}) = {formatted:?} rejected by is_ndp",
            );
        }

        /// Format↔validate inverse for nSCI: `format_nsci(x, n)` always
        /// produces a string `is_nsci(_, n)` accepts (Rust's `{:e}` emits
        /// exactly one mantissa digit + the AGS exponent shape).
        #[test]
        fn format_nsci_is_accepted_by_is_nsci(
            x in prop::num::f64::NORMAL | prop::num::f64::ZERO | prop::num::f64::SUBNORMAL,
            n in 1usize..=15,
        ) {
            let formatted = format_nsci(x, n);
            prop_assert!(
                is_nsci(&formatted, n),
                "format_nsci({x}, {n}) = {formatted:?} rejected by is_nsci",
            );
        }

        /// `format_nsf` is deterministic (pure fn — same f64 + n → same
        /// string) and its output is always a parseable plain decimal
        /// (never scientific — the validator rejects `1.0e2` under nSF), so
        /// Rule 8's `format_nsf(cell, n) == cell` comparison is well-formed.
        ///
        /// NOTE: a reparse-idempotence property (`format_nsf(parse(s), n)
        /// == s`) does NOT hold and is the WRONG shape for sig-fig
        /// formatting — round-tripping through f64 is lossy, and worse,
        /// rounding can cross a decade boundary (0.997 @1SF → "1.0", which
        /// reparses to 1.0 → "1"). The fixer formats the original cell once
        /// and never reparses, so this is a non-issue in production.
        #[test]
        fn format_nsf_is_deterministic_plain_decimal(
            x in -1e9f64..1e9,
            n in 1usize..=12,
        ) {
            let once = format_nsf(x, n);
            prop_assert_eq!(format_nsf(x, n), once.clone(), "non-deterministic");
            // Plain decimal: no scientific notation under nSF.
            prop_assert!(
                !once.contains('e') && !once.contains('E'),
                "format_nsf emitted scientific: {once:?}",
            );
            // Parses back to a finite number (Rule 8 reads it as a cell).
            let reparsed: f64 = once.parse().expect("plain decimal parses");
            prop_assert!(reparsed.is_finite());
        }

        /// The pattern validators never panic on arbitrary input — they
        /// read through hostile cell text on every file.
        #[test]
        fn pattern_validators_never_panic(s in ".*", n in 0usize..20) {
            let _ = is_ndp(&s, n);
            let _ = is_nsci(&s, n);
            let _ = is_dms(&s);
            let _ = is_elapsed_time(&s, "hh:mm:ss");
            prop_assert!(true);
        }

        /// `dt_semantic_ok` never panics on arbitrary value/unit pairs
        /// (it lexes the value against the unit token-by-token — a
        /// malformed pair must fail closed, not crash).
        #[test]
        fn dt_semantic_ok_never_panics(value in ".*", unit in ".*") {
            let _ = dt_semantic_ok(&value, &unit);
            prop_assert!(true);
        }

        /// DT pandas-range gate: any in-range, real calendar date under a
        /// `yyyy-mm-dd` UNIT is accepted; the same shape with a year far
        /// outside the pandas Timestamp window (≤1677 or ≥2263) is
        /// rejected. Mirrors O-33.
        #[test]
        fn dt_semantic_ok_bounds_to_pandas_range(
            y in 1679i32..=2261,
            mo in 1u32..=12,
            d in 1u32..=28,
        ) {
            let in_range = format!("{y:04}-{mo:02}-{d:02}");
            prop_assert!(
                dt_semantic_ok(&in_range, "yyyy-mm-dd"),
                "{in_range} should be in pandas range",
            );

            // Same calendar-valid date, year pushed below the window.
            let below = format!("0{:03}-{mo:02}-{d:02}", y % 1000);
            prop_assert!(
                !dt_semantic_ok(&below, "yyyy-mm-dd"),
                "{below} (year < 1677) should fail the pandas gate",
            );
        }

        /// nDP idempotence at a fixed precision: formatting an
        /// already-nDP-shaped value to the same `n` is a no-op. Unlike
        /// nSF, `{:.n$}` re-rounds the reparsed value to the same `n`
        /// fractional digits deterministically, so this holds across the
        /// whole finite f64 range (including denormals/extreme magnitudes).
        #[test]
        fn format_ndp_idempotent(
            x in prop::num::f64::NORMAL | prop::num::f64::ZERO | prop::num::f64::SUBNORMAL,
            n in 0usize..=12,
        ) {
            let once = format_ndp(x, n);
            let reparsed: f64 = once.parse().expect("format_ndp emits a parseable decimal");
            let twice = format_ndp(reparsed, n);
            prop_assert_eq!(&twice, &once, "x={} n={}", x, n);
        }
    }
}
