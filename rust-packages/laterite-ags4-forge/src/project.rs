//! Project a cell value into a different AGS TYPE (#725).
//!
//! Retyping a column and leaving its values alone gives a file whose cells
//! contradict their own declaration. That is sometimes exactly the point — it
//! is how a type-invalid cell gets manufactured — but it is not what
//! "project this column into 3DP" means. This is the transform that keeps the
//! projected file spec-valid, so that a fault injected into it afterwards is
//! the ONLY fault in it. A projection that quietly broke the file would make
//! every verdict downstream unreadable.
//!
//! It lives in forge rather than in the type crate on purpose. The transform
//! is deliberately LOSSY — a coarser precision discards digits, and `YN`
//! discards everything but a yes or a no — and this crate is published
//! nowhere, so it cannot be reached from real data by accident. If a second
//! consumer ever appears, move it then.
//!
//! **The families disagree about what "project" may discard, and that is
//! inherited rather than chosen here.** A numeric projection rounds: asking
//! for `2DP` renders `100.006` as `100.01`, because a declared precision in
//! AGS is a rendering convention for the same measurement. A `DT` projection
//! will NOT drop a real time: `dt_to_unit_precision` truncates only when every
//! digit it discards is a zero, and refuses otherwise — a decision taken in
//! #715 for the emitter, where `None` means "write what you were given and let
//! the validity mode judge it". Here `None` means the operation is refused by
//! name instead. So a `DT` column carrying real times cannot be projected down
//! to `yyyy-mm-dd`; the caller is told which row, and can blank the times or
//! use the declaration-only path. Refusing is the reversible half of the
//! choice — a lossy `DT` can be allowed later, but a discarded time cannot be
//! recovered from a file that already shipped.
//!
//! The formatters are NOT reimplemented here. `format_ndp`, `format_nsf`,
//! `format_nsci` and the precision-aware `dt_to_unit_precision` all come from
//! the type crate, so a projected value is written by the same code that
//! writes every other value in this repo. The validator once carried a
//! hand-port of those formatters kept honest only by a comment saying where
//! it came from, and nothing checked that the two agreed.

use laterite_ags4_types::{
    CanonicalType, canonical_type, dt_to_unit_precision, format_ndp, format_nsci, format_nsf,
    parse_ags_decimal, parse_bool,
};

/// Split `3DP` / `2SF` / `1SCI` into its digit count and its family.
///
/// The suffixes share no ending with each other, so the order they are tried
/// in cannot matter — but they are tried longest-first anyway, because a
/// future suffix that IS a tail of another would fail silently otherwise.
fn numeric_spec(ags_type: &str) -> Option<(usize, &'static str)> {
    let t = ags_type.trim().to_uppercase();
    for suffix in ["SCI", "SF", "DP"] {
        if let Some(prefix) = t.strip_suffix(suffix) {
            return prefix.parse::<usize>().ok().map(|n| (n, suffix));
        }
    }
    None
}

/// Whether the AGS type system recognises this token at all.
///
/// Dictionary membership is deliberately NOT the test. A type the dictionary
/// never pairs with this heading is a fault forge exists to manufacture;
/// judging that is the validator's job. What is refused here is a token the
/// type system cannot read at all, which would produce a file whose
/// invalidity the caller did not choose.
pub(crate) fn is_known_type(ags_type: &str) -> bool {
    canonical_type(ags_type).is_some()
}

/// `value` rendered so that it satisfies `ags_type`, or `None` when it cannot
/// be.
///
/// `unit` is the heading's DECLARED unit, and it is what decides a `DT`
/// value's precision — never the shape of the value itself. A `DT` heading
/// declaring `yyyy-mm-dd` gets a date even if the cell held a full timestamp,
/// because the declaration is what the file promises its readers (#695/#715).
pub(crate) fn project(value: &str, ags_type: &str, unit: &str) -> Option<String> {
    // An empty cell is empty in every type. Projecting it into a zero, a
    // false or an epoch would invent data the file never carried.
    if value.is_empty() {
        return Some(String::new());
    }
    match canonical_type(ags_type)? {
        // Any text satisfies a text type. That deliberately includes `PA` and
        // `RL`, whose real constraints — defined in ABBR, resolvable as a
        // record link — are the validator's business, and are very often the
        // fault a projection is setting up.
        CanonicalType::String | CanonicalType::Enum => Some(value.to_string()),
        CanonicalType::Bool => parse_bool(value).map(|b| if b { "Y" } else { "N" }.to_string()),
        CanonicalType::Datetime => dt_to_unit_precision(value, unit),
        CanonicalType::Integer => parse_ags_decimal(value).map(|f| format_ndp(f, 0)),
        CanonicalType::Decimal => {
            let f = parse_ags_decimal(value)?;
            let (n, family) = numeric_spec(ags_type)?;
            Some(match family {
                "DP" => format_ndp(f, n),
                "SF" => format_nsf(f, n),
                _ => format_nsci(f, n),
            })
        }
        // No AGS4 type code maps to these today; `canonical_type` can only
        // reach them if the type system grows one. Refusing is the safe
        // answer — a projection nobody has defined is not one to guess at,
        // and the caller is told by name rather than handed a wrong value.
        CanonicalType::Date | CanonicalType::Time => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The table the spec asked for: each canonical family against a value
    /// already at the target, one that needs work, one that cannot project,
    /// an empty cell and a non-ASCII value.
    #[test]
    fn each_family_projects_or_refuses_by_value() {
        let cases: &[(&str, &str, &str, Option<&str>)] = &[
            // (value, type, unit, expected)
            ("100.00", "2DP", "m", Some("100.00")),
            ("100.006", "2DP", "m", Some("100.01")),
            // Not a rounding rule — `100.005` is not exactly representable as
            // an f64 and lands just under the halfway point, so it rounds
            // down. Pinned because it looks like a bug to the next reader and
            // is not one: the projection inherits the repo's formatters
            // rather than inventing arithmetic of its own.
            ("100.005", "2DP", "m", Some("100.00")),
            ("100.00", "0DP", "m", Some("100")),
            ("100.00", "3SF", "m", Some("100")),
            ("0.00123", "2SF", "m", Some("0.0012")),
            ("not a number", "2DP", "m", None),
            ("", "2DP", "m", Some("")),
            ("naïve", "2DP", "m", None),
            // Text takes anything, non-ASCII included.
            ("naïve", "X", "", Some("naïve")),
            ("100.00", "ID", "", Some("100.00")),
            ("SPT+U100", "PA", "", Some("SPT+U100")),
            // Booleans.
            ("Y", "YN", "", Some("Y")),
            ("N", "YN", "", Some("N")),
            ("maybe", "YN", "", None),
            ("", "YN", "", Some("")),
            // The declared UNIT decides a DT's precision, not the value.
            // Dropping an all-zero tail spells the same instant, so it goes.
            (
                "2026-08-26T00:00:00",
                "DT",
                "yyyy-mm-dd",
                Some("2026-08-26"),
            ),
            // Padding up to a finer declared precision adds only zeros.
            (
                "2026-08-26",
                "DT",
                "yyyy-mm-ddThh:mm:ss",
                Some("2026-08-26T00:00:00"),
            ),
            // Dropping a REAL time is refused — see the asymmetry note above.
            ("2026-08-26T09:15:00", "DT", "yyyy-mm-dd", None),
            ("nonsense", "DT", "yyyy-mm-dd", None),
            ("", "DT", "yyyy-mm-dd", Some("")),
        ];
        for (value, ags_type, unit, want) in cases {
            let got = project(value, ags_type, unit);
            assert_eq!(
                got.as_deref(),
                *want,
                "project({value:?}, {ags_type:?}, {unit:?})"
            );
        }
    }

    /// A token the type system cannot read is refused before anything is
    /// written. Note what is NOT refused: `9DP` is not in the dictionary and
    /// is still projectable, because dictionary membership is the validator's
    /// question and inventing one is a fault forge is allowed to make.
    #[test]
    fn an_unreadable_type_token_is_not_a_known_type() {
        for known in ["X", "ID", "2DP", "0DP", "3SF", "1SCI", "DT", "YN", "9DP"] {
            assert!(is_known_type(known), "{known} must be known");
        }
        for unknown in ["", "NOPE", "DP", "2DPX", "-1DP", "2 DP"] {
            assert!(!is_known_type(unknown), "{unknown:?} must not be known");
        }
    }

    #[test]
    fn the_numeric_families_split_on_their_suffix() {
        assert_eq!(numeric_spec("3DP"), Some((3, "DP")));
        assert_eq!(numeric_spec("2SF"), Some((2, "SF")));
        assert_eq!(numeric_spec("1SCI"), Some((1, "SCI")));
        assert_eq!(numeric_spec("DT"), None);
        assert_eq!(numeric_spec("DP"), None);
    }
}
