//! The parsed file, and the handles for walking it.
//!
//! Everything here borrows from one owned [`Document`]. The engine's own parse
//! result is held privately: it gains fields as the format work needs them, and
//! none of that reaches a consumer of this crate.

use laterite_ags4_core::ags4_codec::ParsedAgs4;

use crate::{Error, ErrorKind};

/// A parsed AGS4 file.
///
/// Groups keep the order they appeared in, which matters for writing: an AGS4
/// file's group order is meaningful to the people who read it, and a round-trip
/// that reorders them is not a round-trip.
pub struct Document {
    pub(crate) parsed: ParsedAgs4,
}

impl Document {
    pub(crate) fn new(parsed: ParsedAgs4) -> Document {
        Document { parsed }
    }

    /// The group codes, in file order.
    #[must_use]
    pub fn codes(&self) -> Vec<&str> {
        self.parsed.order.iter().map(String::as_str).collect()
    }

    /// Every group, in file order.
    ///
    /// Returns a `Vec` of borrowed handles rather than `impl Iterator`. An
    /// opaque return type cannot be named by a caller who wants to store it, and
    /// its auto traits are invisible in the rendered API — on a surface that is
    /// frozen at publish, both are worth more than avoiding one small
    /// allocation. A file has at most a few hundred groups.
    #[must_use]
    pub fn groups(&self) -> Vec<Group<'_>> {
        self.parsed
            .order
            .iter()
            .filter_map(|code| self.group(code))
            .collect()
    }

    /// One group by its 4-letter code, e.g. `"LOCA"`.
    #[must_use]
    pub fn group(&self, code: &str) -> Option<Group<'_>> {
        self.parsed.groups.get(code).map(|g| Group { inner: g })
    }

    /// Is this group present?
    #[must_use]
    pub fn contains(&self, code: &str) -> bool {
        self.parsed.groups.contains_key(code)
    }

    /// How many groups.
    #[must_use]
    pub fn len(&self) -> usize {
        self.parsed.order.len()
    }

    /// Are there no groups at all?
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.parsed.order.is_empty()
    }

    /// Overwrite one cell.
    ///
    /// Errors if the group, row or heading does not exist, rather than creating
    /// what was asked for: a typo in a heading name is far more likely than an
    /// intent to add a column, and silently adding one produces a file that is
    /// wrong in a way the writer will happily emit.
    pub fn set_cell(
        &mut self,
        group: &str,
        row: usize,
        heading: &str,
        value: impl Into<String>,
    ) -> Result<(), Error> {
        let g = self.parsed.groups.get_mut(group).ok_or_else(|| {
            Error::new(
                ErrorKind::InvalidArgument,
                format!("no group `{group}` in this document"),
            )
        })?;
        if !g.headings.iter().any(|h| h == heading) {
            return Err(Error::new(
                ErrorKind::InvalidArgument,
                format!("group `{group}` has no heading `{heading}`"),
            ));
        }
        let n = g.rows.len();
        let r = g.rows.get_mut(row).ok_or_else(|| {
            Error::new(
                ErrorKind::InvalidArgument,
                format!("group `{group}` has {n} row(s); no row {row}"),
            )
        })?;
        // Reuse the group's existing `Arc<str>` key rather than allocating a new
        // one — the whole point of the shared-key representation underneath.
        let key = r
            .keys()
            .find(|k| &***k == heading)
            .cloned()
            .unwrap_or_else(|| heading.into());
        r.insert(key, value.into());
        Ok(())
    }

    /// Append a row to a group, as `(heading, value)` pairs.
    ///
    /// Headings not named are written empty. A pair naming a heading the group
    /// does not have is an error for the same reason as [`Document::set_cell`].
    pub fn push_row(&mut self, group: &str, cells: &[(&str, &str)]) -> Result<(), Error> {
        let g = self.parsed.groups.get_mut(group).ok_or_else(|| {
            Error::new(
                ErrorKind::InvalidArgument,
                format!("no group `{group}` in this document"),
            )
        })?;
        if let Some((bad, _)) = cells
            .iter()
            .find(|(h, _)| !g.headings.iter().any(|x| x == h))
        {
            return Err(Error::new(
                ErrorKind::InvalidArgument,
                format!("group `{group}` has no heading `{bad}`"),
            ));
        }
        let mut row = std::collections::HashMap::with_capacity(g.headings.len());
        for h in &g.headings {
            let v = cells
                .iter()
                .find(|(name, _)| name == h)
                .map_or("", |(_, v)| *v);
            row.insert(std::sync::Arc::<str>::from(h.as_str()), v.to_string());
        }
        g.rows.push(row);
        Ok(())
    }

    /// Remove a group entirely. Returns whether it was there.
    pub fn remove_group(&mut self, code: &str) -> bool {
        let existed = self.parsed.groups.remove(code).is_some();
        self.parsed.order.retain(|c| c != code);
        existed
    }
}

/// One group section — its headings, units, types and rows.
#[derive(Clone, Copy)]
pub struct Group<'a> {
    inner: &'a laterite_ags4_core::ags4_codec::AgsGroup,
}

impl<'a> Group<'a> {
    /// The 4-letter group code, e.g. `"LOCA"`.
    #[must_use]
    pub fn code(&self) -> &'a str {
        &self.inner.code
    }

    /// Heading names in declaration order.
    #[must_use]
    pub fn headings(&self) -> Vec<&'a str> {
        self.inner.headings.iter().map(String::as_str).collect()
    }

    /// The UNIT row, aligned with [`Group::headings`].
    #[must_use]
    pub fn units(&self) -> Vec<&'a str> {
        self.inner.units.iter().map(String::as_str).collect()
    }

    /// The TYPE row (AGS4 type codes), aligned with [`Group::headings`].
    #[must_use]
    pub fn types(&self) -> Vec<&'a str> {
        self.inner.types.iter().map(String::as_str).collect()
    }

    /// How many DATA rows.
    #[must_use]
    pub fn len(&self) -> usize {
        self.inner.rows.len()
    }

    /// Are there no DATA rows? (Which is itself an AGS4 rule violation.)
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.inner.rows.is_empty()
    }

    /// One row by index.
    #[must_use]
    pub fn row(&self, i: usize) -> Option<Row<'a>> {
        self.inner.rows.get(i).map(|r| Row { cells: r })
    }

    /// Every row, in file order.
    #[must_use]
    pub fn rows(&self) -> Rows<'a> {
        Rows {
            group: *self,
            next: 0,
        }
    }
}

/// Iterator over a group's rows.
///
/// A named type, not `impl Iterator`: a caller can put this in a struct field
/// or a function signature, which an opaque type forbids.
pub struct Rows<'a> {
    group: Group<'a>,
    next: usize,
}

impl<'a> Iterator for Rows<'a> {
    type Item = Row<'a>;

    fn next(&mut self) -> Option<Row<'a>> {
        let row = self.group.row(self.next)?;
        self.next += 1;
        Some(row)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let left = self.group.len().saturating_sub(self.next);
        (left, Some(left))
    }
}

impl ExactSizeIterator for Rows<'_> {}

/// One DATA row.
#[derive(Clone, Copy)]
pub struct Row<'a> {
    cells: &'a std::collections::HashMap<std::sync::Arc<str>, String>,
}

impl<'a> Row<'a> {
    /// The value under `heading`, or `None` if the row has no such heading.
    ///
    /// Values are returned **verbatim** — exactly the characters the file
    /// carried, with no coercion. AGS4 is a text interchange format and the file
    /// is frequently the contractual artefact, so `write(read(x)) == x` must
    /// hold. Typed access is a 0.2 addition and will be a separate method rather
    /// than a change to this one.
    #[must_use]
    pub fn cell(&self, heading: &str) -> Option<&'a str> {
        self.cells.get(heading).map(String::as_str)
    }

    /// How many cells this row carries.
    #[must_use]
    pub fn len(&self) -> usize {
        self.cells.len()
    }

    /// Does the row carry no cells at all?
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.cells.is_empty()
    }
}

// --- Debug -------------------------------------------------------------
//
// Written by hand, not derived. A derived `Debug` on `Document` would print
// every cell of a delivery file — which turns an innocuous `dbg!` or a
// `.unwrap()` panic message into a dump of someone's site data, and quietly
// makes the engine's internal field names part of what consumers see. These
// print shape, not contents.

impl std::fmt::Debug for Document {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Document")
            .field("groups", &self.parsed.order.len())
            .field("codes", &self.codes())
            .finish()
    }
}

impl std::fmt::Debug for Group<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Group")
            .field("code", &self.code())
            .field("headings", &self.headings())
            .field("rows", &self.len())
            .finish()
    }
}

impl std::fmt::Debug for Row<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Row").field("cells", &self.len()).finish()
    }
}

impl std::fmt::Debug for Rows<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Rows")
            .field("group", &self.group.code())
            .field("remaining", &self.len())
            .finish()
    }
}
