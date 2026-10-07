//! How close one ABBR code is to another — the ranking behind the "did you
//! mean" hints on Rule 16's FYIs (#1024).
//!
//! It only ever *suggests*. Rule 16 looks codes up exactly, and nothing here
//! changes that or rewrites a code: a rewrite happens only when a caller names
//! it ([`crate::recode`]). A wrong suggestion costs a reader a glance; a wrong
//! rewrite would silently change their data, which is why the two are kept
//! apart.
//!
//! Three levels, nearest first:
//!
//! 1. equal ignoring letter case — the same comparison
//!    [`Dictionary::abbr_codes_ignoring_case`](crate::dict::Dictionary::abbr_codes_ignoring_case)
//!    makes, through [`same_ignoring_case`], so the two can never disagree
//!    about what a case variant is;
//! 2. equal once case and the separators `-`, `_`, `.` and `/` are ignored
//!    (`Un-disturbed`);
//! 3. a small Damerau–Levenshtein distance, ignoring case — and only between
//!    codes of at least four characters, because between short codes one edit
//!    is most of the code: `U` and `D` are one substitution apart and mean
//!    different things. Up to six characters one edit is allowed, beyond that
//!    two.

/// The most suggestions any hint names. A longer list stops being a hint.
pub const MAX_SUGGESTIONS: usize = 3;

/// Characters some producers put inside a code and others leave out.
const SEPARATORS: [char; 4] = ['-', '_', '.', '/'];

/// How near a candidate is. Orders nearest first: by `level`, then by
/// `distance` (the case-insensitive edit distance, `0` at level 1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Closeness {
    pub level: u8,
    pub distance: usize,
}

/// Whether two codes are equal once letter case is ignored. Unicode
/// lowercasing, as the case-only collision checks fold.
#[must_use]
pub fn same_ignoring_case(a: &str, b: &str) -> bool {
    a == b || a.to_lowercase() == b.to_lowercase()
}

/// How close `candidate` is to `code`, or `None` when it is not close enough
/// to suggest. A candidate equal to `code` as written is `None` too: it is the
/// code, not a suggestion for it.
#[must_use]
pub fn closeness(code: &str, candidate: &str) -> Option<Closeness> {
    if code == candidate {
        return None;
    }
    if same_ignoring_case(code, candidate) {
        return Some(Closeness {
            level: 1,
            distance: 0,
        });
    }
    let (a, b) = (code.to_lowercase(), candidate.to_lowercase());
    let bare = |s: &str| -> String { s.chars().filter(|c| !SEPARATORS.contains(c)).collect() };
    let bare_a = bare(&a);
    if !bare_a.is_empty() && bare_a == bare(&b) {
        return Some(Closeness {
            level: 2,
            distance: edit_distance(&a, &b),
        });
    }
    let (la, lb) = (code.chars().count(), candidate.chars().count());
    if la.min(lb) < 4 {
        return None;
    }
    let limit = if la.max(lb) <= 6 { 1 } else { 2 };
    // Each edit changes the length by at most one, so a longer gap cannot fit.
    if la.abs_diff(lb) > limit {
        return None;
    }
    let distance = edit_distance(&a, &b);
    (distance <= limit).then_some(Closeness { level: 3, distance })
}

/// The candidates close to `code`, nearest first — by level, then distance,
/// then in dictionary (lexicographic) order — at most [`MAX_SUGGESTIONS`],
/// each named once.
#[must_use]
#[inline(never)]
pub fn nearest<'c>(code: &str, candidates: &[&'c str]) -> Vec<&'c str> {
    // Kept in order by hand rather than by `sort`: the list never holds more
    // than three, and every sort instantiation is code the browser engine
    // downloads (the tier-1 wasm ceiling, `tools/release/check-wasm-tier1.mjs`).
    let mut best: Vec<(Closeness, &'c str)> = Vec::with_capacity(MAX_SUGGESTIONS + 1);
    for &c in candidates {
        let Some(k) = closeness(code, c) else {
            continue;
        };
        if best.iter().any(|(_, b)| *b == c) {
            continue;
        }
        let at = best.iter().position(|e| (k, c) < *e).unwrap_or(best.len());
        best.insert(at, (k, c));
        best.truncate(MAX_SUGGESTIONS);
    }
    best.into_iter().map(|(_, c)| c).collect()
}

/// Damerau–Levenshtein distance in its optimal-string-alignment form: inserts,
/// deletes, substitutions and adjacent transpositions (`STRUBED` → `STURBED`
/// is one), counted over characters. Codes are short, so the full table is
/// cheaper to reason about than a banded one.
#[inline(never)]
fn edit_distance(left: &str, right: &str) -> usize {
    let a: Vec<char> = left.chars().collect();
    let b: Vec<char> = right.chars().collect();
    let width = b.len() + 1;
    // `table[row * width + col]`: the distance between the first `row`
    // characters of `a` and the first `col` of `b`.
    let mut table: Vec<usize> = vec![0; (a.len() + 1) * width];
    for (row, cell) in table.iter_mut().step_by(width).enumerate() {
        *cell = row;
    }
    for (col, cell) in table.iter_mut().take(width).enumerate() {
        *cell = col;
    }
    for row in 1..=a.len() {
        for col in 1..=b.len() {
            let cost = usize::from(a[row - 1] != b[col - 1]);
            let mut best = (table[(row - 1) * width + col] + 1)
                .min(table[row * width + col - 1] + 1)
                .min(table[(row - 1) * width + col - 1] + cost);
            if row > 1 && col > 1 && a[row - 1] == b[col - 2] && a[row - 2] == b[col - 1] {
                best = best.min(table[(row - 2) * width + col - 2] + 1);
            }
            table[row * width + col] = best;
        }
    }
    table[a.len() * width + b.len()]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn level(code: &str, candidate: &str) -> Option<u8> {
        closeness(code, candidate).map(|k| k.level)
    }

    #[test]
    fn the_three_levels_on_the_issue_examples() {
        assert_eq!(level("Undisturbed", "UNDISTURBED"), Some(1));
        assert_eq!(level("Un-disturbed", "UNDISTURBED"), Some(2));
        assert_eq!(level("UNDISTRUBED", "UNDISTURBED"), Some(3));
        assert_eq!(closeness("UNDISTRUBED", "UNDISTURBED").unwrap().distance, 1);
    }

    #[test]
    fn single_letter_codes_are_never_suggested_for_each_other() {
        assert_eq!(level("U", "D"), None);
        assert_eq!(level("D", "U"), None);
        // Nor a short code for a long one, however the edits fall.
        assert_eq!(level("UND", "UNDI"), None);
    }

    #[test]
    fn a_five_character_code_matches_at_one_edit_but_not_two() {
        assert_eq!(level("ABCDE", "ABCDX"), Some(3));
        assert_eq!(level("ABCDE", "ABCXY"), None);
        // Beyond six characters two edits are allowed, three are not.
        assert_eq!(level("ABCDEFG", "ABCDEXY"), Some(3));
        assert_eq!(level("ABCDEFG", "ABCDWXY"), None);
    }

    #[test]
    fn the_code_itself_is_not_a_suggestion() {
        assert_eq!(closeness("UNDISTURBED", "UNDISTURBED"), None);
    }

    #[test]
    fn separators_alone_do_not_make_a_match() {
        assert_eq!(level("-", "/"), None);
        assert_eq!(level("A-B", "A/B"), Some(2));
    }

    #[test]
    fn at_most_three_nearest_first_then_in_dictionary_order() {
        let got = nearest(
            "Undisturbed",
            &[
                "UNDISTURBXX",  // level 3, distance 2
                "UNDISTURBED",  // level 1
                "Un-Disturbed", // level 2
                "UNDISTURBE",   // level 3, distance 1
                "AUNDISTURBED", // level 3, distance 1, later in dictionary order
                "ZZZZ",         // not close
            ],
        );
        assert_eq!(got, vec!["UNDISTURBED", "Un-Disturbed", "AUNDISTURBED"]);
        // Without the two nearer levels, distance outranks dictionary order.
        let got = nearest(
            "Undisturbed",
            &["UNDISTURBXX", "UNDISTURBE", "AUNDISTURBED"],
        );
        assert_eq!(got, vec!["AUNDISTURBED", "UNDISTURBE", "UNDISTURBXX"]);
        assert!(nearest("U", &["D", "X"]).is_empty());
    }

    #[test]
    fn a_candidate_listed_twice_is_named_once() {
        assert_eq!(
            nearest("Undisturbed", &["UNDISTURBED", "UNDISTURBED"]),
            vec!["UNDISTURBED"]
        );
    }

    #[test]
    fn edit_distance_counts_an_adjacent_swap_as_one() {
        assert_eq!(edit_distance("strubed", "sturbed"), 1);
        assert_eq!(edit_distance("", "abc"), 3);
        assert_eq!(edit_distance("kitten", "sitting"), 3);
    }
}
