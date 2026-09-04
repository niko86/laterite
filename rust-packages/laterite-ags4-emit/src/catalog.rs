//! The catalog-collection rule: which UNITs and TYPEs a file's groups
//! actually use (Rules 15 / 17).
//!
//! Two synthesizers need this answer — the shipped emitter (minting a
//! missing UNIT/TYPE catalog over a caller's real groups) and forge's
//! whole-file dogfood minting — and until #924 each owned its own copy.
//! Reliquary row 14 recorded them as deliberately separate, "converge only
//! if they drift"; they drifted: forge tested `!u.is_empty()` on the
//! untrimmed value where emit trims and excludes the header literals, and
//! forge never learned emit's PU harvest at all — so the dogfood corpus
//! could not manufacture the picklist-of-units case the shipped emitter
//! handles. One rule, two adapters; what stays per-consumer is the group
//! *shape* the rule reads through [`GroupView`], and everything each
//! synthesizer builds AROUND the set (row minting, gap-filling vs
//! whole-file).
//!
//! Home is THIS crate, not the `laterite-ags4-types` leaf the ticket also
//! nominated: emit is published, and its tarball verifies against the
//! types version on crates.io — a new types module is unreachable from a
//! packaged emit until the next engine cut publishes it (the same
//! constraint that deferred the facade's hostopts adoption, #930). Forge,
//! the other consumer, is never packaged, so it can call new emit API
//! through its path dependency immediately.
use std::collections::BTreeSet;

/// The slice of a group the collection rule reads: the UNIT/TYPE header
/// rows plus cell access for the PU harvest. Implemented by emit's
/// `OwnedGroup` and forge's `Group` — two real adapters, which is what
/// makes this seam real rather than hypothetical.
pub trait GroupView {
    /// The UNIT row, one value per column.
    fn units(&self) -> &[String];
    /// The TYPE row, one value per column.
    fn types(&self) -> &[String];
    /// Number of DATA rows.
    fn row_count(&self) -> usize;
    /// The bare value at (`row`, `col`), if the row has that column.
    fn cell(&self, row: usize, col: usize) -> Option<&str>;
}

/// Distinct non-empty units used (Rule 15): every group's UNIT-row value
/// plus every distinct value in a `PU`-typed data column — a
/// picklist-of-units column's cells ARE units, and a catalog that omits
/// them fails the very rule it was minted to satisfy. Values are trimmed;
/// blanks and the literal `"UNIT"` (the header row's own label, present
/// when a caller round-trips a parsed file) are excluded.
pub fn units_used<'a, G, I>(groups: I) -> BTreeSet<String>
where
    G: GroupView + 'a,
    I: IntoIterator<Item = &'a G>,
{
    let mut units = BTreeSet::new();
    for g in groups {
        for u in g.units() {
            let u = u.trim();
            if !u.is_empty() && u != "UNIT" {
                units.insert(u.to_string());
            }
        }
        for (ci, ty) in g.types().iter().enumerate() {
            if ty.trim() == "PU" {
                for row in 0..g.row_count() {
                    if let Some(v) = g.cell(row, ci).map(str::trim) {
                        if !v.is_empty() {
                            units.insert(v.to_string());
                        }
                    }
                }
            }
        }
    }
    units
}

/// Distinct non-empty type codes used (Rule 17): every group's TYPE-row
/// value, trimmed; blanks and the literal `"TYPE"` are excluded.
pub fn types_used<'a, G, I>(groups: I) -> BTreeSet<String>
where
    G: GroupView + 'a,
    I: IntoIterator<Item = &'a G>,
{
    let mut types = BTreeSet::new();
    for g in groups {
        for t in g.types() {
            let t = t.trim();
            if !t.is_empty() && t != "TYPE" {
                types.insert(t.to_string());
            }
        }
    }
    types
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A minimal in-crate view — the two REAL adapters live with their
    /// structs (emit's `OwnedGroup`, forge's `Group`) and each pins the
    /// same expected sets over the same logical input there.
    struct Probe {
        units: Vec<String>,
        types: Vec<String>,
        rows: Vec<Vec<String>>,
    }

    impl GroupView for Probe {
        fn units(&self) -> &[String] {
            &self.units
        }
        fn types(&self) -> &[String] {
            &self.types
        }
        fn row_count(&self) -> usize {
            self.rows.len()
        }
        fn cell(&self, row: usize, col: usize) -> Option<&str> {
            self.rows.get(row)?.get(col).map(String::as_str)
        }
    }

    fn probe() -> Probe {
        Probe {
            // " m " padded: the divergence that fired row 14 — forge kept
            // the padded spelling as a distinct unit, emit trimmed it.
            units: vec![" m ".into(), "UNIT".into(), String::new()],
            types: vec!["PU".into(), "X".into(), "TYPE".into()],
            rows: vec![
                vec!["kPa".into(), "notpu".into(), "x".into()],
                vec!["  ".into(), "blankpu".into(), "y".into()],
            ],
        }
    }

    #[test]
    fn units_trim_exclude_the_header_literal_and_harvest_pu() {
        let units = units_used(std::iter::once(&probe()));
        let got: Vec<&str> = units.iter().map(String::as_str).collect();
        // Trimmed "m" (not " m "), the PU column's "kPa"; the literal
        // "UNIT", the blank, the whitespace-only PU cell and every non-PU
        // data cell are out.
        assert_eq!(got, vec!["kPa", "m"]);
    }

    #[test]
    fn types_trim_and_exclude_the_header_literal() {
        let types = types_used(std::iter::once(&probe()));
        let got: Vec<&str> = types.iter().map(String::as_str).collect();
        assert_eq!(got, vec!["PU", "X"]);
    }

    #[test]
    fn a_short_row_contributes_what_it_has() {
        // A row shorter than the PU column index must not panic — `cell`
        // answers None and the harvest moves on.
        let g = Probe {
            units: vec![String::new(), String::new()],
            types: vec!["X".into(), "PU".into()],
            rows: vec![vec!["only-one".into()]],
        };
        assert!(units_used(std::iter::once(&g)).is_empty());
    }
}
