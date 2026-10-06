//! The order a group's DATA rows are written in (#1008).
//!
//! AGS4 requires no row order, so this is about the reader, not validity:
//! a merge writes rows in arrival order, which interleaves the locations and
//! samples of several deliveries, and a person reading or diffing the file
//! has to re-sort it. The engine already knows each group's KEY headings and
//! their declared TYPEs, which is everything a correct sort needs and what a
//! caller would otherwise have to rebuild.
//!
//! The comparator is written here rather than borrowed from a crate because
//! its rules are AGS rules: which TYPEs compare as numbers, that a numeric
//! value compares exactly (never through `f64`, whose rounding can tie two
//! distinct values), and where a blank or unparsable key goes.

use std::cmp::Ordering;

use laterite_ags4_reference::keychain::key_heading_names;
use laterite_ags4_reference::union::registry;
use laterite_ags4_validator::effective_dict::{DictRow, FileDict};

/// The order each group's DATA rows are written in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RowOrder {
    /// The order the rows were given in (for a merge, first-seen order across
    /// the inputs). The default: the output is byte-identical to a build that
    /// never heard of this option, because some producers rely on their own.
    #[default]
    Input,
    /// Sorted ascending by the group's KEY headings — inherited parent keys
    /// first, then its own, in dictionary order (the row identity merge and
    /// diff already use). A key declared `nDP`, `nSF`, `nSCI` or `U` on the
    /// group's TYPE line compares by exact decimal value; any other compares
    /// in natural order (`BH2` before `BH10`). Blank keys sort last, the sort
    /// is stable, and the metadata groups (TRAN, TYPE, UNIT, ABBR, DICT) and
    /// groups without KEY headings are left in input order.
    Key,
}

impl RowOrder {
    /// The wire name — the exact token every surface accepts and reports.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            RowOrder::Input => "input",
            RowOrder::Key => "key",
        }
    }

    /// Every accepted token, default first — the one list the CLI's value
    /// enum, the `.pyi` `Literal` and the TS unions derive from or are pinned
    /// to, as [`crate::DictRows::ALL`] is for `dict_rows`.
    pub const ALL: [RowOrder; 2] = [RowOrder::Input, RowOrder::Key];
}

impl std::str::FromStr for RowOrder {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        RowOrder::ALL
            .into_iter()
            .find(|m| m.as_str() == s.trim().to_ascii_lowercase())
            .ok_or_else(|| {
                let allowed: Vec<&str> = RowOrder::ALL.iter().map(|m| m.as_str()).collect();
                format!(
                    "unknown row_order {s:?}; expected one of {}",
                    allowed.join(", ")
                )
            })
    }
}

/// Groups whose rows are catalogues or records of the file itself rather than
/// data: their order carries no reading benefit, and DICT's order is the Rule
/// 18a order Rule 7 appends user headings by, so moving it could add a finding.
const UNSORTED: [&str; 5] = ["TRAN", "TYPE", "UNIT", "ABBR", "DICT"];

/// The DICT rows known so far, kept owned so the effective dictionary of a
/// group the standard registry does not know can be read when that group
/// arrives; a standard group never needs them. The one-call doors hold every
/// group, so they read the DICT ahead ([`DictSeen::looked_ahead`]) — merge
/// writes its groups alphabetically, which puts DICT after any user group
/// coded before it. A session door sees no group in advance, so there only a
/// DICT pushed BEFORE such a group can key it.
#[derive(Default)]
pub(crate) struct DictSeen {
    rows: Vec<[String; 8]>,
    /// Set once a door has recorded every DICT up front, so the DICT's own
    /// push does not record its rows a second time.
    pub(crate) looked_ahead: bool,
}

impl DictSeen {
    pub(crate) fn record(&mut self, headings: &[String], rows: &[Vec<String>]) {
        let col = |name: &str| headings.iter().position(|h| h == name);
        let cols = [
            col("DICT_TYPE"),
            col("DICT_GRP"),
            col("DICT_HDNG"),
            col("DICT_STAT"),
            col("DICT_DTYP"),
            col("DICT_UNIT"),
            col("DICT_PGRP"),
            col("DICT_DESC"),
        ];
        for row in rows {
            self.rows.push(cols.map(|c| {
                c.and_then(|i| row.get(i))
                    .map_or_else(String::new, |v| v.trim().to_string())
            }));
        }
    }

    fn file_dict(&self) -> FileDict {
        FileDict::from_rows(self.rows.iter().map(|r| DictRow {
            dict_type: &r[0],
            group: &r[1],
            heading: &r[2],
            status: &r[3],
            ags_type: &r[4],
            unit: &r[5],
            parent: &r[6],
            desc: &r[7],
        }))
    }
}

/// The names of `code`'s KEY headings, parent keys first: the registry's for
/// a standard group (a file's DICT never re-keys one, as with row identity),
/// otherwise the file's own declarations with its declared parent's first.
fn key_names(code: &str, seen: &DictSeen) -> Vec<String> {
    let reg = registry();
    if let Some(g) = reg.get(code) {
        return key_heading_names(g)
            .into_iter()
            .map(str::to_string)
            .collect();
    }
    if seen.rows.is_empty() {
        return Vec::new();
    }
    let fd = seen.file_dict();
    let declared = |group: &str| -> Vec<String> {
        match reg.get(group) {
            Some(g) => key_heading_names(g)
                .into_iter()
                .map(str::to_string)
                .collect(),
            None => fd.key_headings(group).map(|h| h.heading.clone()).collect(),
        }
    };
    let mut names: Vec<String> = fd
        .parent(code)
        .filter(|p| !p.is_empty() && *p != "-" && *p != code)
        .map(&declared)
        .unwrap_or_default();
    for own in declared(code) {
        if !names.contains(&own) {
            names.push(own);
        }
    }
    names
}

/// Sort `rows` in place by the group's KEY headings under [`RowOrder::Key`].
/// `types` is the group's TYPE line as it will be written. A group whose rows
/// are not all as wide as its HEADING row is left alone: its cells cannot be
/// matched to their headings, and a reorder would only guess.
pub(crate) fn sort_by_keys(
    code: &str,
    headings: &[String],
    types: &[String],
    rows: &mut Vec<Vec<String>>,
    seen: &DictSeen,
) {
    if rows.len() < 2 || UNSORTED.contains(&code) {
        return;
    }
    let width = headings.len();
    if rows.iter().any(|r| r.len() != width) {
        return;
    }
    let keys: Vec<(usize, bool)> = key_names(code, seen)
        .iter()
        .filter_map(|k| headings.iter().position(|h| h == k))
        .map(|i| (i, types.get(i).is_some_and(|t| is_numeric_type(t))))
        .collect();
    if keys.is_empty() {
        return;
    }
    // Each numeric cell is parsed once, not once per comparison.
    let parsed: Vec<Vec<Option<Decimal>>> = keys
        .iter()
        .map(|&(col, numeric)| {
            if numeric {
                rows.iter().map(|r| Decimal::parse(&r[col])).collect()
            } else {
                Vec::new()
            }
        })
        .collect();
    let mut order: Vec<usize> = (0..rows.len()).collect();
    // `sort_by` is stable: rows that tie on every key keep their input order.
    order.sort_by(|&a, &b| {
        for (k, &(col, numeric)) in keys.iter().enumerate() {
            let (va, vb) = (rows[a][col].as_str(), rows[b][col].as_str());
            let o = if numeric {
                cmp_numeric(va, vb, parsed[k][a].as_ref(), parsed[k][b].as_ref())
            } else {
                cmp_text(va, vb)
            };
            if o != Ordering::Equal {
                return o;
            }
        }
        Ordering::Equal
    });
    if order.iter().enumerate().all(|(to, &from)| to == from) {
        return; // already sorted — no row moves
    }
    let mut old: Vec<Option<Vec<String>>> = std::mem::take(rows).into_iter().map(Some).collect();
    *rows = order
        .into_iter()
        .map(|i| old[i].take().expect("each row index appears once"))
        .collect();
}

/// The numeric AGS TYPEs: `nDP`, `nSF`, `nSCI` (any precision) and `U`.
fn is_numeric_type(t: &str) -> bool {
    let t = t.trim();
    if t == "U" {
        return true;
    }
    ["SCI", "DP", "SF"].iter().any(|suffix| {
        t.strip_suffix(suffix)
            .is_some_and(|n| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()))
    })
}

fn is_blank(s: &str) -> bool {
    s.trim().is_empty()
}

/// A numeric key: parsed values by exact value, then the unparsable ones in
/// natural order, then blanks. Equal values (`1.0`, `1.00`) tie, so the next
/// key decides.
fn cmp_numeric(a: &str, b: &str, da: Option<&Decimal>, db: Option<&Decimal>) -> Ordering {
    match (is_blank(a), is_blank(b)) {
        (true, true) => return Ordering::Equal,
        (true, false) => return Ordering::Greater,
        (false, true) => return Ordering::Less,
        (false, false) => {}
    }
    match (da, db) {
        (Some(x), Some(y)) => x.cmp(y),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => natural_cmp(a, b),
    }
}

/// Any other key: natural order, blanks last.
fn cmp_text(a: &str, b: &str) -> Ordering {
    match (is_blank(a), is_blank(b)) {
        (true, true) => Ordering::Equal,
        (true, false) => Ordering::Greater,
        (false, true) => Ordering::Less,
        (false, false) => natural_cmp(a, b),
    }
}

/// Natural order: digit runs compare as whole numbers (`BH2` < `BH10`,
/// `BUILD 16` < `BUILD 183`), other runs case-insensitively, a digit run
/// before a text run at the same position. A tie on all of that (`BH02` and
/// `BH2`, `bh1` and `BH1`) falls to the raw strings, so the order is total
/// and never depends on the input's.
fn natural_cmp(left: &str, right: &str) -> Ordering {
    let (mut lruns, mut rruns) = (Runs(left), Runs(right));
    loop {
        match (lruns.next(), rruns.next()) {
            (None, None) => return left.cmp(right),
            (None, Some(_)) => return Ordering::Less,
            (Some(_), None) => return Ordering::Greater,
            (Some(lrun), Some(rrun)) => {
                let order = cmp_run(lrun, rrun);
                if order != Ordering::Equal {
                    return order;
                }
            }
        }
    }
}

fn cmp_run(x: &str, y: &str) -> Ordering {
    let digits = |s: &str| s.as_bytes()[0].is_ascii_digit();
    match (digits(x), digits(y)) {
        (true, true) => {
            // Exact for a run of any length: no integer type to overflow.
            let (x, y) = (x.trim_start_matches('0'), y.trim_start_matches('0'));
            x.len().cmp(&y.len()).then_with(|| x.cmp(y))
        }
        (true, false) => Ordering::Less,
        (false, true) => Ordering::Greater,
        (false, false) => x
            .chars()
            .flat_map(char::to_lowercase)
            .cmp(y.chars().flat_map(char::to_lowercase)),
    }
}

/// Alternating runs of ASCII digits and of everything else. Splitting on an
/// ASCII byte always lands on a char boundary, so the runs are valid `&str`s.
struct Runs<'a>(&'a str);

impl<'a> Iterator for Runs<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<&'a str> {
        let bytes = self.0.as_bytes();
        let first = *bytes.first()?;
        let digit = first.is_ascii_digit();
        let end = bytes
            .iter()
            .position(|b| b.is_ascii_digit() != digit)
            .unwrap_or(bytes.len());
        let (run, rest) = self.0.split_at(end);
        self.0 = rest;
        Some(run)
    }
}

/// An exact decimal: `±0.d₁d₂…dₙ × 10^exp`, with no leading or trailing zero
/// digit, so equal values have one representation (`1.0` and `1.00` and
/// `1E0` are the same). Zero has no digits. Never an `f64`: rounding would
/// let two distinct values tie, or order them by an artefact of the float.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Decimal {
    negative: bool,
    exp: i64,
    digits: Vec<u8>,
}

impl Decimal {
    /// `[+-]digits[.digits][(e|E)[+-]digits]`, surrounding whitespace allowed,
    /// at least one mantissa digit. Anything else is `None` (unparsable).
    pub(crate) fn parse(s: &str) -> Option<Decimal> {
        let s = s.trim();
        let (negative, rest) = match s.as_bytes().first()? {
            b'-' => (true, &s[1..]),
            b'+' => (false, &s[1..]),
            _ => (false, s),
        };
        let (mantissa, exponent) = match rest.find(['e', 'E']) {
            Some(i) => (&rest[..i], Some(&rest[i + 1..])),
            None => (rest, None),
        };
        let (int, frac) = mantissa.split_once('.').unwrap_or((mantissa, ""));
        let all_digits = |p: &str| p.bytes().all(|b| b.is_ascii_digit());
        if (int.is_empty() && frac.is_empty()) || !all_digits(int) || !all_digits(frac) {
            return None;
        }
        let e10: i64 = match exponent {
            None => 0,
            Some(e) => {
                let unsigned = e.strip_prefix(['+', '-']).unwrap_or(e);
                if unsigned.is_empty() || !all_digits(unsigned) {
                    return None;
                }
                e.parse().ok()?
            }
        };
        let all: Vec<u8> = int.bytes().chain(frac.bytes()).map(|b| b - b'0').collect();
        let Some(lead) = all.iter().position(|&d| d != 0) else {
            return Some(Decimal {
                negative: false,
                exp: 0,
                digits: Vec::new(),
            });
        };
        let tail = all.iter().rposition(|&d| d != 0).map_or(0, |i| i + 1);
        let int_len = i64::try_from(int.len()).ok()?;
        let lead_i = i64::try_from(lead).ok()?;
        Some(Decimal {
            negative,
            exp: int_len.checked_sub(lead_i)?.checked_add(e10)?,
            digits: all[lead..tail].to_vec(),
        })
    }

    fn sign(&self) -> i8 {
        match (self.digits.is_empty(), self.negative) {
            (true, _) => 0,
            (false, true) => -1,
            (false, false) => 1,
        }
    }
}

impl Ord for Decimal {
    fn cmp(&self, other: &Self) -> Ordering {
        self.sign().cmp(&other.sign()).then_with(|| {
            // Same sign: compare magnitudes (a longer digit string with an
            // equal prefix is the larger, which `Vec`'s order already says).
            let magnitude = self
                .exp
                .cmp(&other.exp)
                .then_with(|| self.digits.cmp(&other.digits));
            if self.sign() < 0 {
                magnitude.reverse()
            } else {
                magnitude
            }
        })
    }
}

impl PartialOrd for Decimal {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sorted(mut v: Vec<&str>) -> Vec<&str> {
        v.sort_by(|a, b| cmp_text(a, b));
        v
    }

    fn dec(s: &str) -> Decimal {
        Decimal::parse(s).unwrap_or_else(|| panic!("{s:?} should parse"))
    }

    #[test]
    fn digit_runs_compare_as_numbers() {
        assert_eq!(sorted(vec!["BH10", "BH2", "BH1"]), ["BH1", "BH2", "BH10"]);
        assert_eq!(
            sorted(vec!["BUILD 183", "BUILD 2", "BUILD 16"]),
            ["BUILD 2", "BUILD 16", "BUILD 183"]
        );
        // A run longer than any integer type still compares exactly.
        assert_eq!(
            natural_cmp("X99999999999999999999999", "X100000000000000000000000"),
            Ordering::Less
        );
    }

    #[test]
    fn text_runs_ignore_case_and_a_full_tie_falls_to_the_raw_strings() {
        assert_eq!(sorted(vec!["bh2", "BH1", "Bh3"]), ["BH1", "bh2", "Bh3"]);
        // Equal ignoring case: the raw bytes decide, so the order is total.
        assert_eq!(natural_cmp("BH1", "bh1"), Ordering::Less);
        assert_eq!(natural_cmp("bh1", "BH1"), Ordering::Greater);
        // Equal as numbers: the raw bytes decide again.
        assert_eq!(natural_cmp("BH02", "BH2"), Ordering::Less);
        assert_eq!(natural_cmp("BH2", "BH02"), Ordering::Greater);
        assert_eq!(natural_cmp("BH2", "BH2"), Ordering::Equal);
    }

    #[test]
    fn blank_sorts_last() {
        // Blanks tie with each other, so the stable sort keeps their order.
        assert_eq!(sorted(vec![" ", "B", "", "A"]), ["A", "B", " ", ""]);
        let parsed = |s: &str| Decimal::parse(s);
        let mut v = vec!["", "x", "2.00", "1.00"];
        v.sort_by(|a, b| cmp_numeric(a, b, parsed(a).as_ref(), parsed(b).as_ref()));
        assert_eq!(v, ["1.00", "2.00", "x", ""]);
    }

    #[test]
    fn decimals_compare_exactly() {
        assert!(dec("10.00") > dec("9.50"));
        assert_eq!(dec("1.0"), dec("1.00"));
        assert_eq!(dec("1E0"), dec("1"));
        assert_eq!(dec("-0.0"), dec("0"));
        assert!(dec("-2") < dec("-1.5"));
        assert!(dec("-0.001") < dec("0"));
        assert!(dec("0.15") < dec("0.151"));
        assert!(dec("1.5E-3") < dec(".0016"));
        assert!(dec("2.0E+3") > dec("1999.999"));
        // Distinct in exact arithmetic, equal once both are an f64.
        assert!(dec("0.10000000000000000001") > dec("0.1"));
        for bad in [
            "", "-", ".", "1,000", "1.2.3", "NaN", "inf", "1e", "1e+", "0x10",
        ] {
            assert!(Decimal::parse(bad).is_none(), "{bad:?} should not parse");
        }
    }

    #[test]
    fn numeric_types_are_recognised() {
        for t in ["2DP", "0DP", "3SF", "2SCI", "U", " 1DP "] {
            assert!(is_numeric_type(t), "{t}");
        }
        for t in [
            "X", "ID", "PA", "DT", "T", "RL", "DP", "MC", "XN", "YN", "SCI",
        ] {
            assert!(!is_numeric_type(t), "{t}");
        }
    }

    #[test]
    fn row_order_tokens_round_trip_and_unknown_is_refused() {
        for m in RowOrder::ALL {
            assert_eq!(m.as_str().parse::<RowOrder>(), Ok(m));
        }
        assert_eq!(RowOrder::default(), RowOrder::Input);
        let msg = "sorted".parse::<RowOrder>().unwrap_err();
        assert!(msg.contains("input, key"), "{msg}");
    }
}
