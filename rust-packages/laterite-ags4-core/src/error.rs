//! Typed CLI errors with exit codes that mirror the Python `_cli.py` contract.
//!
//! The Python CLI documents exit codes 0-8 in its `--help` epilog. Agents
//! and CI rely on those for self-correction without parsing prose. This
//! enum is the Rust mirror: each variant maps to a documented exit code.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum CliError {
    #[error("file not found: {0}")]
    FileNotFound(String),

    #[error("schema error: {0}")]
    Schema(String),

    /// A group declared the same heading twice (AGS4 Rule 7). Fatal on the read
    /// path by default: rows are keyed by heading name, so continuing would
    /// return the second column's values for the first column's position — a
    /// wrong answer that looks like a complete one. The recovery mode
    /// (`DuplicateHeadings::Recover`) reads the file with the repeats suffixed.
    #[error(
        "duplicate heading {heading:?} in group {group:?} (AGS4 Rule 7) — \
         reading it would silently merge two columns; re-read in recovery mode \
         to keep both, suffixed __2, __3, …"
    )]
    DuplicateHeading { group: String, heading: String },

    /// A DATA row split into MORE fields than its group declares headings, so
    /// the excess cannot be bound to anything and the value it belongs to
    /// cannot be determined. Sibling of [`Self::DuplicateHeading`] and fatal
    /// for the same reason: continuing hands back a row that looks complete
    /// and is not.
    ///
    /// The usual cause is a value containing a comma whose quotes were lost
    /// (AGS4 Rule 5) — `Acme, Bloggs and Co` unquoted splits in two, and
    /// nothing can say which half belongs to the heading.
    #[error(
        "row at line {line} in group {group:?} has {found} field(s) but {group:?} \
         declares {declared} heading(s) (AGS4 Rules 4/5) — the extra value(s) \
         belong to no heading, so this row cannot be read faithfully; the usual \
         cause is a value containing a comma that lost its quotes. Re-read with \
         ExcessFields::Truncate to discard them deliberately."
    )]
    ExcessFields {
        group: String,
        line: u32,
        found: usize,
        declared: usize,
    },
}

impl CliError {
    #[must_use]
    pub fn exit_code(&self) -> i32 {
        match self {
            Self::FileNotFound(_) => 3,
            // Same class as Schema: the file is structurally unusable as read.
            Self::Schema(_) | Self::DuplicateHeading { .. } | Self::ExcessFields { .. } => 6,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The exit codes are the CLI/CI self-correction contract (`_cli.py`'s
    /// documented 0-8), so pin the exact numbers, not just "non-zero". A missing
    /// file is its own recoverable class (3); anything structural is 6.
    #[test]
    fn exit_codes_mirror_the_documented_cli_contract() {
        assert_eq!(CliError::FileNotFound("x".into()).exit_code(), 3);
        assert_eq!(CliError::Schema("boom".into()).exit_code(), 6);
        assert_eq!(
            CliError::DuplicateHeading {
                group: "LOCA".into(),
                heading: "LOCA_ID".into(),
            }
            .exit_code(),
            6,
        );
    }

    #[test]
    fn display_names_the_offending_file_group_and_heading() {
        assert_eq!(
            CliError::FileNotFound("d.ags".into()).to_string(),
            "file not found: d.ags"
        );
        assert_eq!(
            CliError::Schema("bad row".into()).to_string(),
            "schema error: bad row"
        );
        let dup = CliError::DuplicateHeading {
            group: "LOCA".into(),
            heading: "LOCA_ID".into(),
        }
        .to_string();
        assert!(dup.contains("LOCA_ID") && dup.contains("LOCA"), "{dup}");
        assert!(dup.contains("Rule 7"), "{dup}");
    }
}
