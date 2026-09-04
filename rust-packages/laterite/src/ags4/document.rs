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
    /// The bytes exactly as they arrived — **before** any transcode.
    ///
    /// Kept for [`Document::certify`], whose SHA-256 must be over the bytes a
    /// later reader will hash. Storing the decoded UTF-8 instead would look
    /// right and mint a certificate that is never fresh for the file on disk:
    /// a cp1252 delivery read with `encoding` differs from its own decoded
    /// form in exactly the `°` and `±` cells that made someone pass the option.
    /// The Python surface mints over the original bytes for the same reason and
    /// records the encoding alongside, which is what this pair reproduces.
    pub(crate) source_bytes: Vec<u8>,
    /// The encoding label the source was read with, recorded into the
    /// certificate so a verifier decodes the way the minter did.
    pub(crate) encoding: Option<String>,
    /// Was this built by slicing named groups out of the source using a
    /// certificate's byte index, rather than by parsing the whole file?
    pub(crate) sliced: bool,
}

impl Document {
    pub(crate) fn new(
        parsed: ParsedAgs4,
        source_bytes: Vec<u8>,
        encoding: Option<String>,
    ) -> Document {
        Document {
            parsed,
            source_bytes,
            encoding,
            sliced: false,
        }
    }

    /// Did a certificate's byte index let this read skip the rest of the file?
    ///
    /// `false` whenever the whole file was parsed — including when a certificate
    /// was offered and declined, which is the case worth being able to see. An
    /// index that quietly stops applying is otherwise indistinguishable from one
    /// that is working, since the document is identical either way.
    #[must_use]
    pub fn sliced(&self) -> bool {
        self.sliced
    }

    /// Keep only these groups, dropping the rest. The filter half of
    /// [`crate::ags4::Read::only`].
    pub(crate) fn retain_only(&mut self, codes: &[String]) {
        self.parsed.retain_only(codes);
    }

    /// The group codes, in file order.
    #[must_use]
    pub fn codes(&self) -> Vec<&str> {
        self.parsed.order().iter().map(String::as_str).collect()
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
            .order()
            .iter()
            .filter_map(|code| self.group(code))
            .collect()
    }

    /// One group by its 4-letter code, e.g. `"LOCA"`.
    #[must_use]
    pub fn group(&self, code: &str) -> Option<Group<'_>> {
        self.parsed.get(code).map(|g| Group { inner: g })
    }

    /// Is this group present?
    #[must_use]
    pub fn contains(&self, code: &str) -> bool {
        self.parsed.get(code).is_some()
    }

    /// How many groups.
    #[must_use]
    pub fn len(&self) -> usize {
        self.parsed.order().len()
    }

    /// Are there no groups at all?
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.parsed.order().is_empty()
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
        let g = self.parsed.get_mut(group).ok_or_else(|| {
            Error::new(
                ErrorKind::InvalidArgument,
                format!("no group `{group}` in this document"),
            )
        })?;
        let Some(col) = g.col(heading) else {
            return Err(Error::new(
                ErrorKind::InvalidArgument,
                format!("group `{group}` has no heading `{heading}`"),
            ));
        };
        let n = g.n_rows();
        if row >= n {
            return Err(Error::new(
                ErrorKind::InvalidArgument,
                format!("group `{group}` has {n} row(s); no row {row}"),
            ));
        }
        // Copy-on-write underneath: the group's first mutation materialises
        // it (one group's worth); bounds were checked above, so this cannot
        // refuse.
        g.set_cell(row, col, value.into());
        Ok(())
    }

    /// Append a row to a group, as `(heading, value)` pairs.
    ///
    /// Headings not named are written empty. A pair naming a heading the group
    /// does not have is an error for the same reason as [`Document::set_cell`].
    pub fn push_row(&mut self, group: &str, cells: &[(&str, &str)]) -> Result<(), Error> {
        let g = self.parsed.get_mut(group).ok_or_else(|| {
            Error::new(
                ErrorKind::InvalidArgument,
                format!("no group `{group}` in this document"),
            )
        })?;
        if let Some((bad, _)) = cells
            .iter()
            .find(|(h, _)| !g.headings().iter().any(|x| x == h))
        {
            return Err(Error::new(
                ErrorKind::InvalidArgument,
                format!("group `{group}` has no heading `{bad}`"),
            ));
        }
        let row: Vec<String> = g
            .headings()
            .iter()
            .map(|h| {
                cells
                    .iter()
                    .find(|(name, _)| name == h)
                    .map_or(String::new(), |(_, v)| (*v).to_string())
            })
            .collect();
        g.push_row(row);
        Ok(())
    }

    /// Remove a group entirely. Returns whether it was there.
    pub fn remove_group(&mut self, code: &str) -> bool {
        self.parsed.take_group(code).is_some()
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
        self.inner.code()
    }

    /// Heading names in declaration order.
    #[must_use]
    pub fn headings(&self) -> Vec<&'a str> {
        self.inner.headings().iter().map(String::as_str).collect()
    }

    /// The UNIT row, aligned with [`Group::headings`].
    #[must_use]
    pub fn units(&self) -> Vec<&'a str> {
        self.inner.units().iter().map(String::as_str).collect()
    }

    /// The TYPE row (AGS4 type codes), aligned with [`Group::headings`].
    #[must_use]
    pub fn types(&self) -> Vec<&'a str> {
        self.inner.types().iter().map(String::as_str).collect()
    }

    /// How many DATA rows.
    #[must_use]
    pub fn len(&self) -> usize {
        self.inner.n_rows()
    }

    /// Are there no DATA rows? (Which is itself an AGS4 rule violation.)
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.inner.n_rows() == 0
    }

    /// One row by index.
    #[must_use]
    pub fn row(&self, i: usize) -> Option<Row<'a>> {
        (i < self.inner.n_rows()).then_some(Row {
            group: self.inner,
            idx: i,
        })
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
    group: &'a laterite_ags4_core::ags4_codec::AgsGroup,
    idx: usize,
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
        self.group.cell_named(self.idx, heading)
    }

    /// How many cells this row carries.
    #[must_use]
    pub fn len(&self) -> usize {
        self.group.headings().len()
    }

    /// Does the row carry no cells at all?
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.group.headings().is_empty()
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
            .field("groups", &self.parsed.order().len())
            .field("codes", &self.codes())
            // Length, never contents: this is the whole source file, and a
            // `dbg!` of a document should not print a delivery.
            .field("source_bytes", &self.source_bytes.len())
            .field("encoding", &self.encoding)
            .field("sliced", &self.sliced)
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
