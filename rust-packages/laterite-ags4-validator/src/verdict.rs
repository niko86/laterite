//! The one place a validation verdict is computed.
//!
//! Until now the formula was trivial — "any finding at all means invalid" —
//! and so it was written out at every surface boundary: `laterite-cli`'s
//! `validate` command and `laterite-py`'s `run_check` each held their own
//! `i32::from(count != 0)`. They agreed because there was nothing to disagree
//! about.
//!
//! Separating the DISPLAY tier from the VERDICT (#321) ends that. A warning is
//! now shown by default and does **not** fail, `--warnings-as-errors` opts back
//! into failure, and the verdict has to know which tier each finding is in.
//! That is a rule, and a rule copied per surface is one that eventually
//! diverges — silently, because a wrong exit code in CI looks like a passing
//! build. Same reasoning that gave [`crate::error::ValidatorError`] a single
//! `exit_code()`.
//!
//! [`Verdict::exit_code`] is *derived from* [`Verdict::is_valid`] rather than
//! computed beside it, so the two cannot contradict each other. The invariant
//! `is_valid == (exit_code == 0)` holds by construction, not by test.

use crate::findings::{Findings, Severity, Truncated};

/// What a validation run concluded, and the tier counts it concluded it from.
///
/// Build with [`Verdict::of`]. The counts are of findings **the run made** — a
/// tier the caller did not ask for was never collected, so it counts zero here
/// for the same reason it shows nothing there. A finding a per-rule cap did not
/// keep still counts (#1089): the cap bounds what a report holds, not what it
/// concludes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Verdict {
    /// Findings at [`Severity::Error`]. These always fail.
    pub errors: usize,
    /// Findings at [`Severity::Warning`]. Fail only under
    /// `warnings_as_errors`.
    pub warnings: usize,
    /// Findings at [`Severity::Fyi`]. Never fail — the tier exists to be read,
    /// not to gate.
    pub fyi: usize,
    /// Whether the caller asked for warnings to be fatal.
    warnings_are_fatal: bool,
}

impl Verdict {
    /// Count `found`, plus what `truncated` says a cap did not keep, by tier, and
    /// record whether warnings were asked to be fatal.
    ///
    /// `truncated` is a required argument, not a default, so a surface that
    /// offers a cap cannot forget to count what it cut: under a cap of `0`,
    /// `found` alone is empty and would read as a pass.
    ///
    /// `warnings_as_errors` is the caller's *request*, not a property of the
    /// findings, which is why it is stored rather than folded into the counts:
    /// a report and its verdict are different questions, and the tier counts
    /// stay meaningful under either answer.
    #[must_use]
    pub fn of(found: &Findings, truncated: &Truncated, warnings_as_errors: bool) -> Self {
        let mut v = Self {
            errors: 0,
            warnings: 0,
            fyi: 0,
            warnings_are_fatal: warnings_as_errors,
        };
        for finding in found.values().flatten() {
            match finding.severity {
                Severity::Error => v.errors += 1,
                Severity::Warning => v.warnings += 1,
                Severity::Fyi => v.fyi += 1,
            }
        }
        for cut in truncated.values() {
            v.errors += cut.dropped(Severity::Error);
            v.warnings += cut.dropped(Severity::Warning);
            v.fyi += cut.dropped(Severity::Fyi);
        }
        v
    }

    /// Every finding in the report, whatever its tier.
    ///
    /// This is what the report *shows*; it is deliberately NOT what the verdict
    /// keys off. Kept because "how many findings are there" is a real question
    /// a caller asks, and answering it with the error count would be a lie of a
    /// different shape.
    #[must_use]
    pub fn total(&self) -> usize {
        self.errors + self.warnings + self.fyi
    }

    /// Did the file pass?
    ///
    /// Errors always fail. Warnings fail only when the caller asked them to.
    /// FYI never fails.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.errors == 0 && !(self.warnings_are_fatal && self.warnings > 0)
    }

    /// The process exit code for this verdict: `0` pass, `1` fail.
    ///
    /// Derived from [`Self::is_valid`] rather than recomputed, so the two
    /// cannot drift.
    #[must_use]
    pub fn exit_code(&self) -> i32 {
        i32::from(!self.is_valid())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::findings::{Location, add_at};

    fn one(sev: Severity) -> Findings {
        let mut f = Findings::new();
        add_at(
            &mut f,
            "AGS Format Rule 1",
            Some(1),
            "PROJ",
            "x",
            Location::default(),
            sev,
        );
        f
    }

    #[test]
    fn a_clean_file_passes() {
        let v = Verdict::of(&Findings::new(), &Truncated::new(), false);
        assert!(v.is_valid());
        assert_eq!(v.exit_code(), 0);
        assert_eq!(v.total(), 0);
    }

    #[test]
    fn an_error_always_fails() {
        for fatal in [false, true] {
            let v = Verdict::of(&one(Severity::Error), &Truncated::new(), fatal);
            assert!(!v.is_valid(), "an error must fail (fatal={fatal})");
            assert_eq!(v.exit_code(), 1);
        }
    }

    #[test]
    fn a_warning_is_shown_but_does_not_fail() {
        // The whole point of #321: the finding is in the report (total counts
        // it) and the file still passes.
        let v = Verdict::of(&one(Severity::Warning), &Truncated::new(), false);
        assert_eq!(v.warnings, 1);
        assert_eq!(v.total(), 1);
        assert!(v.is_valid());
        assert_eq!(v.exit_code(), 0);
    }

    #[test]
    fn warnings_as_errors_makes_the_same_warning_fatal() {
        let v = Verdict::of(&one(Severity::Warning), &Truncated::new(), true);
        assert_eq!(v.warnings, 1);
        assert!(!v.is_valid());
        assert_eq!(v.exit_code(), 1);
    }

    #[test]
    fn fyi_never_fails_even_under_warnings_as_errors() {
        // `--warnings-as-errors` says warnings, and means it. Escalating fyi
        // too would make the flag's name a lie and the tier unusable.
        for fatal in [false, true] {
            let v = Verdict::of(&one(Severity::Fyi), &Truncated::new(), fatal);
            assert_eq!(v.fyi, 1);
            assert!(v.is_valid(), "fyi must never fail (fatal={fatal})");
            assert_eq!(v.exit_code(), 0);
        }
    }

    #[test]
    fn is_valid_and_exit_code_can_never_disagree() {
        // The invariant every surface is held to. It holds structurally here
        // (exit_code derives from is_valid); this pins it against a future
        // refactor that computes them separately.
        let mut mixed = Findings::new();
        for sev in [Severity::Error, Severity::Warning, Severity::Fyi] {
            add_at(
                &mut mixed,
                "AGS Format Rule 1",
                Some(1),
                "PROJ",
                "x",
                Location::default(),
                sev,
            );
        }
        for found in [Findings::new(), one(Severity::Warning), mixed] {
            for fatal in [false, true] {
                let v = Verdict::of(&found, &Truncated::new(), fatal);
                assert_eq!(v.is_valid(), v.exit_code() == 0);
            }
        }
    }

    #[test]
    fn tier_counts_sum_to_total() {
        let mut f = Findings::new();
        for sev in [
            Severity::Error,
            Severity::Warning,
            Severity::Warning,
            Severity::Fyi,
        ] {
            add_at(
                &mut f,
                "AGS Format Rule 1",
                Some(1),
                "PROJ",
                "x",
                Location::default(),
                sev,
            );
        }
        let v = Verdict::of(&f, &Truncated::new(), false);
        assert_eq!((v.errors, v.warnings, v.fyi), (1, 2, 1));
        assert_eq!(v.total(), 4);
    }
}
