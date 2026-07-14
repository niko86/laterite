//! AGS4 rule checks.
//!
//! Each submodule (added V1–V8) implements one rule family, clean-room
//! from `reports/AGS 4_1.pdf` §4.1.1. `run_all` is the single dispatch
//! point `check_file` calls — phases wire their rules in here.
//!
//! V1: line-level lexical rules (1, 3, 5, 6).
//! V2: group-structure rules (2, 2a, 2b, 4).
//! V3: name-format rules (19, 19a, 19b structural).
//! V4: dictionary-aware rules (7, 9) — first phase to consult `dict`.
//! V5: typed-value rule (8) — value vs declared TYPE/UNIT.
//! V6: mandatory/definition groups (12, 13, 14, 15, 16, 17, 18).
//! V7: relational rules (10a, 10b, 10c, 11/11a/11b, 11c).
//! V8: cross-reference rules (19b_2/19b_3 dict-aware, 20).
//! All eight phases now wired; `run_all` is feature-complete.

pub mod dictionary;
pub mod groups;
pub mod line_format;
pub mod naming;
pub mod references;
pub mod relational;
pub mod structure;
pub mod typed_values;

use crate::CheckOptions;
use crate::dict::Dictionary;
use crate::findings::Findings;
use crate::parse::ParsedFile;

/// Run every enabled rule against `parsed`, appending to `found`.
/// Phases wire their family in here; ordering follows the AGS4 rule
/// numbering so a report reads top-to-bottom like the spec.
///
/// **CONTENT only.** Every rule here is a pure function of `parsed` (and the
/// dictionary + tier flags): same bytes in, same findings out. Nothing in this
/// module may touch the filesystem, the clock, or the environment — the rules
/// that must are in [`crate::world`], and they run from [`crate::check_parsed`],
/// never from here. That purity is what makes an `.ags.idx` certificate able to
/// stand in for this function's result.
///
/// `pub(crate)`, not `pub`: every out-of-crate caller goes through
/// [`crate::check_parsed`], which is the only place that can refuse an
/// incoherent request. Before, four surfaces called this directly with
/// `check_files: true` in their options and got a silent false clean.
pub(crate) fn run_all(
    parsed: &ParsedFile,
    dict: &Dictionary,
    opts: &CheckOptions,
    found: &mut Findings,
) {
    line_format::check(parsed, opts, found); // Rules 1, 3, 5, 6
    structure::check(parsed, found); //          Rules 2, 2a, 2b, 4
    naming::check(parsed, found); //             Rules 19, 19a, 19b
    dictionary::check(parsed, dict, found); //   Rules 7, 9
    typed_values::check(parsed, found); //       Rule 8
    relational::check(parsed, dict, found); //   Rules 10a–10c, 11a–11c
    references::check(parsed, dict, found); //   Rules 19b_2/3, 20 (data level)
    // Rule 18 reads Rule 9's output — must run after dictionary::check.
    // Groups takes opts (for FYI gates) AND dict (for Rule 16's FYI
    // variant that compares the file's ABBR descs against the
    // bundled standard list).
    groups::check(parsed, dict, opts, found); // Rules 13–18 (12 = no-op)
}
