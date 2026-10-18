//! What a validation run found.

/// How serious a finding is.
///
/// `#[non_exhaustive]`: the engine's severity set is a format-work concern and
/// may gain a level, which must not break a consumer's `match`.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Severity {
    /// The file breaks the standard.
    Error,
    /// Suspicious but permitted — a malformed DICT, a nonstandard abbreviation.
    Warning,
    /// Informational only.
    Fyi,
}

impl Severity {
    /// The stable wire token, shared with the Python, Node and `lat` surfaces.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Fyi => "fyi",
        }
    }
}

/// One thing wrong with a file.
pub struct Finding {
    pub(crate) rule: String,
    pub(crate) group: String,
    pub(crate) description: String,
    pub(crate) line: Option<u32>,
    pub(crate) severity: Severity,
}

impl Finding {
    /// The rule label, e.g. `"AGS Format Rule 8"`.
    #[must_use]
    pub fn rule(&self) -> &str {
        &self.rule
    }

    /// The group the finding is about; empty when it is about the file itself.
    #[must_use]
    pub fn group(&self) -> &str {
        &self.group
    }

    /// Human-readable detail.
    #[must_use]
    pub fn description(&self) -> &str {
        &self.description
    }

    /// 1-indexed line in the source file, when the finding has one.
    #[must_use]
    pub fn line(&self) -> Option<u32> {
        self.line
    }

    /// How serious.
    #[must_use]
    pub fn severity(&self) -> Severity {
        self.severity
    }
}

/// A rule that [`Validate::max_findings_per_rule`](super::Validate::max_findings_per_rule)
/// cut: it kept its first [`shown`](Truncation::shown) findings of
/// [`total`](Truncation::total).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Truncation {
    pub(crate) rule: String,
    pub(crate) shown: usize,
    pub(crate) total: usize,
}

impl Truncation {
    /// The rule label, e.g. `"AGS Format Rule 2a"`.
    #[must_use]
    pub fn rule(&self) -> &str {
        &self.rule
    }

    /// How many of its findings the report holds.
    #[must_use]
    pub fn shown(&self) -> usize {
        self.shown
    }

    /// How many findings the rule made.
    #[must_use]
    pub fn total(&self) -> usize {
        self.total
    }
}

/// The result of validating a file.
pub struct Report {
    pub(crate) findings: Vec<Finding>,
    pub(crate) truncated: Vec<Truncation>,
    /// Every finding by tier, kept or not: errors, warnings, fyi.
    pub(crate) counts: [usize; 3],
    pub(crate) certified: bool,
    pub(crate) revalidate_reason: Option<String>,
}

impl Report {
    /// Counted by the engine's one verdict, so a finding a cap did not keep still
    /// counts here.
    pub(crate) fn new(
        found: laterite_ags4_validator::findings::Collected,
        certified: bool,
        revalidate_reason: Option<String>,
    ) -> Report {
        let v =
            laterite_ags4_validator::verdict::Verdict::of(&found.findings, &found.truncated, false);
        Report {
            truncated: found
                .truncated
                .iter()
                .map(|(rule, t)| Truncation {
                    rule: rule.clone(),
                    shown: t.shown,
                    total: t.total,
                })
                .collect(),
            counts: [v.errors, v.warnings, v.fyi],
            findings: super::convert(found.findings),
            certified,
            revalidate_reason,
        }
    }

    /// Does the file break no error-severity rule?
    ///
    /// Warnings and FYIs do not make a file invalid, so this stays true when
    /// they are present — which is why it is a distinct question from
    /// "did the run produce findings".
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.count(Severity::Error) == 0
    }

    /// Every finding the report holds, ordered by rule then by the order the
    /// engine emitted them, so two runs over the same file are diffable. Under
    /// [`Validate::max_findings_per_rule`](super::Validate::max_findings_per_rule)
    /// that is each rule's first findings; [`Report::truncated`] says which rules
    /// had more.
    #[must_use]
    pub fn findings(&self) -> &[Finding] {
        &self.findings
    }

    /// The rules a per-rule cap cut, in rule order. Empty when nothing was cut,
    /// which is always the case without a cap.
    #[must_use]
    pub fn truncated(&self) -> &[Truncation] {
        &self.truncated
    }

    /// Did an offered certificate stand in for the rule engine?
    ///
    /// Always `false` when no certificate was offered. It never means "nothing
    /// was checked": if `check_files` was on, the on-disk half ran regardless —
    /// a certificate is a statement about bytes, and a directory is not part of
    /// them.
    #[must_use]
    pub fn certified(&self) -> bool {
        self.certified
    }

    /// Why an offered certificate was NOT used, if one was offered and rejected.
    ///
    /// `None` when none was offered, or when it was accepted. Worth surfacing
    /// rather than swallowing: without it, a stale certificate looks exactly
    /// like a working one from the outside — same verdict, quietly paying for a
    /// full validation every time.
    #[must_use]
    pub fn revalidate_reason(&self) -> Option<&str> {
        self.revalidate_reason.as_deref()
    }

    /// How many findings at this severity — every one the run made, including
    /// any a per-rule cap did not keep.
    #[must_use]
    pub fn count(&self, severity: Severity) -> usize {
        match severity {
            Severity::Error => self.counts[0],
            Severity::Warning => self.counts[1],
            Severity::Fyi => self.counts[2],
        }
    }

    /// Were there no findings at all, at any severity?
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.counts.iter().all(|n| *n == 0)
    }
}

impl std::fmt::Debug for Finding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Finding")
            .field("rule", &self.rule)
            .field("group", &self.group)
            .field("line", &self.line)
            .field("severity", &self.severity)
            .field("description", &self.description)
            .finish()
    }
}

/// Counts, not the findings themselves: a report over a bad file can carry
/// thousands, and a panic message is not where anyone wants them.
impl std::fmt::Debug for Report {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Report")
            .field("valid", &self.is_valid())
            .field("errors", &self.count(Severity::Error))
            .field("warnings", &self.count(Severity::Warning))
            .field("fyi", &self.count(Severity::Fyi))
            .finish()
    }
}
