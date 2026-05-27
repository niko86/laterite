//! Fuzzy "did you mean" suggestions for unknown group/recipe codes.
//!
//! Wraps `strsim::normalized_levenshtein`. Mirrors Python's
//! `_cli_groups.suggest_codes` (which uses difflib's ratio-based matcher).
//! Cutoff 0.5 is permissive enough to catch one-character typos without
//! firing on unrelated strings.

/// Return up to `n` closest matches to `typed` from `candidates`.
///
/// `uppercase` controls whether the input + candidates get upper-cased before
/// comparison. Group codes are upper-case; recipe slugs are lower-case.
pub fn suggest<S: AsRef<str>>(
    typed: &str,
    candidates: &[S],
    n: usize,
    uppercase: bool,
) -> Vec<String> {
    let probe = if uppercase {
        typed.to_uppercase()
    } else {
        typed.to_lowercase()
    };
    let mut scored: Vec<(String, f64)> = candidates
        .iter()
        .map(|c| {
            let cand = c.as_ref();
            let key = if uppercase {
                cand.to_uppercase()
            } else {
                cand.to_lowercase()
            };
            let score = strsim::normalized_levenshtein(&probe, &key);
            (cand.to_string(), score)
        })
        .filter(|(_, s)| *s >= 0.5)
        .collect();
    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    scored.into_iter().take(n).map(|(c, _)| c).collect()
}
