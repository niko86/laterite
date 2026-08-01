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

/// The result of validating a file.
pub struct Report {
    pub(crate) findings: Vec<Finding>,
}

impl Report {
    /// Does the file break no error-severity rule?
    ///
    /// Warnings and FYIs do not make a file invalid, so this stays true when
    /// they are present — which is why it is a distinct question from
    /// "did the run produce findings".
    #[must_use]
    pub fn is_valid(&self) -> bool {
        !self.findings.iter().any(|f| f.severity == Severity::Error)
    }

    /// Every finding, ordered by rule then by the order the engine emitted them,
    /// so two runs over the same file are diffable.
    #[must_use]
    pub fn findings(&self) -> &[Finding] {
        &self.findings
    }

    /// How many findings at this severity.
    #[must_use]
    pub fn count(&self, severity: Severity) -> usize {
        self.findings
            .iter()
            .filter(|f| f.severity == severity)
            .count()
    }

    /// Were there no findings at all, at any severity?
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.findings.is_empty()
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
