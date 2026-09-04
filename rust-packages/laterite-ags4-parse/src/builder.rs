//! Assemble a [`ParsedFile`] from a byte-authoring writer's own records —
//! the leaf half of the writer-built verdict
//! (`ags-wiki/design/dec-emit-streamed-verdict.md`).
//!
//! An emitter that authors every byte it writes — each quote, separator and
//! terminator — already knows everything `parse_bytes` would re-derive from
//! those bytes. This builder lets it say so: the writer records each row it
//! writes (tag, body span, cell spans), and [`ParsedFileBuilder::finish`]
//! assembles the same retained structure the parse walk builds, over the
//! written bytes adopted as the retained buffer. No tokenizer runs; the spans
//! come from the writer's own emission, and the pieces the walk owns —
//! first-seen-wins group identity, descriptor overwrite on a redeclared
//! group, the `""` unescape into a fix-up region, the `u32` span-space guard
//! — are THIS crate's, shared with the walk rather than restated by the
//! caller.
//!
//! The result matches [`parse_bytes`](crate::parse_bytes) of the same bytes
//! under the validating profile on every field the rule engine reads (the
//! emit crate's differential test holds that equality); the two deliberately
//! differ in buffer LAYOUT — the builder's buffer is the written bytes
//! verbatim (terminators included, fix-ups appended after the end), where the
//! walk's drops terminators and interleaves fix-ups — which spans absorb, and
//! only coordinate-comparing code could see.
//!
//! [`ParsedFileBuilder::finish`] returns the emitted length beside the file:
//! the retained buffer PREFIX up to it is exactly the written bytes, so a
//! caller wanting them back truncates the (refcount-1) buffer there and pays
//! no copy.

use std::collections::BTreeMap;
use std::sync::Arc;

use crate::{
    DataRow, GroupRecord, ParseError, ParsedFile, ParsedGroup, RawLine, append_text, span_at,
    unescape_doubled,
};

/// One written cell, located by the writer: the byte range of its LOGICAL
/// value in the written bytes — inside the quotes, escapes included — and
/// whether that range still carries doubled `""` escapes (in which case the
/// builder unescapes it once into the buffer's fix-up region, exactly as the
/// walk does).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecordedCell {
    pub start: usize,
    pub end: usize,
    pub has_escape: bool,
}

/// Which descriptor row a record is — the same five tags the walk matches,
/// plus the blank separator line between group sections.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RowTag {
    Group,
    Heading,
    Unit,
    Type,
    Data,
    /// A section-separator line: empty body, no cells.
    Blank,
}

/// A deferred `""` unescape: the cell's span cannot point at the escaped
/// bytes, so it is patched at [`ParsedFileBuilder::finish`] to point at the
/// once-unescaped copy in the fix-up region.
struct Fixup {
    code: String,
    arena_idx: usize,
    start: usize,
    end: usize,
}

/// Builds a [`ParsedFile`] over bytes the caller writes into [`Self::buf`].
///
/// Protocol: write one physical line's bytes (body + `\r\n`) into the
/// buffer, then [`Self::record_row`] it with the body's byte range and its
/// cells' value ranges (the cells AFTER the leading tag). Rows must be
/// recorded in written order — line numbers are assigned by arrival.
/// [`Self::finish`] adopts the buffer as [`ParsedFile::text`].
#[derive(Default)]
pub struct ParsedFileBuilder {
    out: Vec<u8>,
    line: u32,
    raw_lines: Vec<RawLine>,
    groups: BTreeMap<String, ParsedGroup>,
    group_order: Vec<String>,
    group_records: Vec<GroupRecord>,
    current: Option<String>,
    fixups: Vec<Fixup>,
}

impl ParsedFileBuilder {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// The output buffer the caller writes rows into. The builder never
    /// writes to it before [`Self::finish`] — every byte is the caller's.
    pub fn buf(&mut self) -> &mut Vec<u8> {
        &mut self.out
    }

    /// Bytes written so far — where the next row's body will start.
    #[must_use]
    pub fn written(&self) -> usize {
        self.out.len()
    }

    /// Materialise one recorded cell's logical value from the written bytes.
    fn cell_value(&self, c: &RecordedCell) -> Result<String, ParseError> {
        let raw =
            std::str::from_utf8(&self.out[c.start..c.end]).map_err(|_| ParseError::NotUtf8)?;
        Ok(if c.has_escape {
            unescape_doubled(raw)
        } else {
            raw.to_string()
        })
    }

    /// Record one written row: `body` is the byte range of the line's body
    /// (tag included, terminator excluded) and `cells` locate the values
    /// AFTER the leading tag. Mirrors the walk's per-tag arms — including
    /// first-seen-wins on a redeclared group code, whose descriptor rows
    /// overwrite and whose DATA rows append.
    pub fn record_row(
        &mut self,
        tag: RowTag,
        body: std::ops::Range<usize>,
        cells: &[RecordedCell],
    ) -> Result<(), ParseError> {
        debug_assert!(
            self.out[body.end..].starts_with(b"\r\n"),
            "a recorded row must already be CRLF-terminated"
        );
        // The walk's u32 span-space guard, at the same moment: refuse before
        // an offset could truncate.
        if u32::try_from(body.end).is_err() {
            return Err(ParseError::TooLarge);
        }
        self.line += 1;
        let number = self.line;
        self.raw_lines.push(RawLine {
            number,
            text: span_at(body.start, body.end),
            had_crlf: true,
        });
        match tag {
            RowTag::Blank => {}
            RowTag::Group => {
                let code = cells
                    .first()
                    .map(|c| self.cell_value(c))
                    .transpose()?
                    .unwrap_or_default();
                self.group_records.push(GroupRecord {
                    code: code.clone(),
                    byte_offset: body.start as u64,
                    line: number,
                });
                self.groups.entry(code.clone()).or_insert_with(|| {
                    self.group_order.push(code.clone());
                    ParsedGroup {
                        // Placeholder until finish, exactly as in the walk.
                        buf: Arc::default(),
                        code: code.clone(),
                        group_line: number,
                        group_byte: body.start as u64,
                        heading_line: None,
                        unit_line: None,
                        type_line: None,
                        headings: Vec::new(),
                        units: Vec::new(),
                        types: Vec::new(),
                        rows: Vec::new(),
                        spans: Vec::new(),
                        row_byte_offsets: Vec::new(),
                    }
                });
                self.current = Some(code);
            }
            RowTag::Heading | RowTag::Unit | RowTag::Type => {
                let values: Vec<String> = cells
                    .iter()
                    .map(|c| self.cell_value(c))
                    .collect::<Result<_, _>>()?;
                if let Some(g) = self.current.as_ref().and_then(|c| self.groups.get_mut(c)) {
                    match tag {
                        RowTag::Heading => {
                            g.heading_line = Some(number);
                            g.headings = values;
                        }
                        RowTag::Unit => {
                            g.unit_line = Some(number);
                            g.units = values;
                        }
                        _ => {
                            g.type_line = Some(number);
                            g.types = values;
                        }
                    }
                }
            }
            RowTag::Data => {
                let current = self.current.clone();
                if let Some(g) = current.as_ref().and_then(|c| self.groups.get_mut(c)) {
                    let first = u32::try_from(g.spans.len()).map_err(|_| ParseError::TooLarge)?;
                    for c in cells {
                        if c.has_escape {
                            // Patched at finish to the unescaped copy in the
                            // fix-up region — the walk's discipline, deferred
                            // because the region lives past the written bytes.
                            let arena_idx = g.spans.len();
                            g.spans.push(span_at(0, 0));
                            self.fixups.push(Fixup {
                                code: current.clone().unwrap_or_default(),
                                arena_idx,
                                start: c.start,
                                end: c.end,
                            });
                        } else {
                            g.spans.push(span_at(c.start, c.end));
                        }
                    }
                    // Fits u32: bounded by the arena growth guarded above.
                    #[allow(clippy::cast_possible_truncation)]
                    let n = (g.spans.len() - first as usize) as u32;
                    g.rows.push(DataRow {
                        line: number,
                        first,
                        n,
                    });
                }
            }
        }
        Ok(())
    }

    /// Adopt the written bytes as the retained buffer and hand back the
    /// assembled [`ParsedFile`] plus the written length (the buffer prefix
    /// that IS the output — anything past it is the fix-up region).
    ///
    /// Errors as the walk would: no GROUP row recorded is
    /// [`ParseError::NotAgs4`] with the walk's exact message.
    pub fn finish(mut self) -> Result<(ParsedFile, usize), ParseError> {
        if self.group_order.is_empty() {
            return Err(ParseError::NotAgs4("no GROUP rows found".to_string()));
        }
        let emitted_len = self.out.len();
        let mut buf = String::from_utf8(self.out).map_err(|_| ParseError::NotUtf8)?;
        for fx in self.fixups {
            let fixed = unescape_doubled(&buf[fx.start..fx.end]);
            let base = append_text(&mut buf, &fixed)? as usize;
            let g = self
                .groups
                .get_mut(&fx.code)
                .expect("a fixup names the group that recorded it");
            g.spans[fx.arena_idx] = span_at(base, base + fixed.len());
        }
        // ONE buffer, adopted (not copied) into the Arc and shared into each
        // group by refcount — the walk's exact ending.
        let text = Arc::new(buf);
        for g in self.groups.values_mut() {
            g.buf = Arc::clone(&text);
        }
        Ok((
            ParsedFile {
                text,
                groups: self.groups,
                group_order: self.group_order,
                group_records: self.group_records,
                raw_lines: self.raw_lines,
                // Validating-profile parity: the per-line/per-row source
                // offsets are absent (dec-parse-structure-layout); the
                // group-level coordinates above are populated, as every
                // profile's are.
                line_byte_offsets: Vec::new(),
                total_lines: self.line,
                has_bom: false,
                total_bytes: emitted_len as u64,
                // The written bytes ARE the source: nothing was decoded or
                // substituted, so every offset is source-true.
                byte_offsets_source_true: true,
            },
            emitted_len,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse_str;

    /// Hand-record a small file and hold the assembled structure equal to a
    /// real parse of the same bytes on every field the rule engine reads.
    /// (The emit crate's differential test does this over the real writer;
    /// this pins the builder's own arms, fix-up region included.)
    #[test]
    fn assembled_file_matches_a_real_parse_of_the_same_bytes() {
        let mut b = ParsedFileBuilder::new();
        // "GROUP","PROJ"
        let rows: &[(&str, RowTag, &[&str])] = &[
            ("\"GROUP\",\"PROJ\"", RowTag::Group, &["PROJ"]),
            (
                "\"HEADING\",\"PROJ_ID\",\"PROJ_NAME\"",
                RowTag::Heading,
                &["PROJ_ID", "PROJ_NAME"],
            ),
            ("\"UNIT\",\"\",\"\"", RowTag::Unit, &["", ""]),
            ("\"TYPE\",\"ID\",\"X\"", RowTag::Type, &["ID", "X"]),
            (
                "\"DATA\",\"P1\",\"say \"\"hi\"\"\"",
                RowTag::Data,
                &["P1", "say \"\"hi\"\""],
            ),
            ("", RowTag::Blank, &[]),
            ("\"GROUP\",\"LOCA\"", RowTag::Group, &["LOCA"]),
            ("\"HEADING\",\"LOCA_ID\"", RowTag::Heading, &["LOCA_ID"]),
            ("\"UNIT\",\"\"", RowTag::Unit, &[""]),
            ("\"TYPE\",\"ID\"", RowTag::Type, &["ID"]),
            ("\"DATA\",\"BH01\"", RowTag::Data, &["BH01"]),
        ];
        for (line, tag, cells) in rows {
            let start = b.written();
            b.buf().extend_from_slice(line.as_bytes());
            let end = b.written();
            b.buf().extend_from_slice(b"\r\n");
            // Locate each cell's value bytes inside the line we just wrote —
            // the test plays the writer, which knows where it put things.
            // A moving cursor from just past the tag keeps repeated values
            // (e.g. two empty cells) landing on their own occurrence.
            let mut cursor = line.find(',').unwrap_or(line.len());
            let recs: Vec<RecordedCell> = cells
                .iter()
                .map(|c| {
                    let off = line[cursor..].find(&format!("\"{c}\"")).unwrap() + cursor + 1;
                    cursor = off + c.len() + 1;
                    RecordedCell {
                        start: start + off,
                        end: start + off + c.len(),
                        has_escape: c.contains("\"\""),
                    }
                })
                .collect();
            b.record_row(*tag, start..end, &recs).unwrap();
        }
        let (built, emitted_len) = b.finish().unwrap();
        let source = String::from_utf8(built.text.as_bytes()[..emitted_len].to_vec()).unwrap();
        let parsed = parse_str(&source).unwrap();

        assert_eq!(built.group_order, parsed.group_order);
        assert_eq!(built.total_lines, parsed.total_lines);
        assert_eq!(built.has_bom, parsed.has_bom);
        assert_eq!(built.total_bytes, parsed.total_bytes);
        assert_eq!(built.group_records, parsed.group_records);
        assert_eq!(built.raw_lines.len(), parsed.raw_lines.len());
        for (bl, pl) in built.raw_lines.iter().zip(&parsed.raw_lines) {
            assert_eq!(bl.number, pl.number);
            assert_eq!(bl.had_crlf, pl.had_crlf);
            assert_eq!(built.line_text(bl), parsed.line_text(pl));
        }
        for code in &parsed.group_order {
            let bg = &built.groups[code];
            let pg = &parsed.groups[code];
            assert_eq!(bg.group_line, pg.group_line);
            assert_eq!(bg.group_byte, pg.group_byte);
            assert_eq!(bg.heading_line, pg.heading_line);
            assert_eq!(bg.unit_line, pg.unit_line);
            assert_eq!(bg.type_line, pg.type_line);
            assert_eq!(bg.headings, pg.headings);
            assert_eq!(bg.units, pg.units);
            assert_eq!(bg.types, pg.types);
            assert_eq!(bg.rows.len(), pg.rows.len());
            for (br, pr) in bg.rows.iter().zip(&pg.rows) {
                assert_eq!(br.line, pr.line);
                assert_eq!(br.n_values(), pr.n_values());
                for i in 0..pr.n_values() {
                    assert_eq!(bg.value_at(br, i), pg.value_at(pr, i));
                }
            }
        }
        // The escaped cell resolved through the fix-up region.
        let proj = &built.groups["PROJ"];
        assert_eq!(proj.cell(1, 0), Some("say \"hi\""));
    }

    /// A redeclared group code keeps ONE entry: descriptor rows overwrite,
    /// DATA rows append — the walk's first-seen-wins, which the emit door can
    /// reach when a caller passes two groups under one code.
    #[test]
    fn a_redeclared_group_overwrites_descriptors_and_appends_rows() {
        let src = "\"GROUP\",\"PROJ\"\r\n\"HEADING\",\"PROJ_ID\"\r\n\"DATA\",\"P1\"\r\n\r\n\
                   \"GROUP\",\"PROJ\"\r\n\"HEADING\",\"PROJ_NAME\"\r\n\"DATA\",\"P2\"\r\n";
        let mut b = ParsedFileBuilder::new();
        let mut at = 0usize;
        let plan: &[(RowTag, &str, &[&str])] = &[
            (RowTag::Group, "\"GROUP\",\"PROJ\"", &["PROJ"]),
            (RowTag::Heading, "\"HEADING\",\"PROJ_ID\"", &["PROJ_ID"]),
            (RowTag::Data, "\"DATA\",\"P1\"", &["P1"]),
            (RowTag::Blank, "", &[]),
            (RowTag::Group, "\"GROUP\",\"PROJ\"", &["PROJ"]),
            (RowTag::Heading, "\"HEADING\",\"PROJ_NAME\"", &["PROJ_NAME"]),
            (RowTag::Data, "\"DATA\",\"P2\"", &["P2"]),
        ];
        for (tag, line, cells) in plan {
            let start = at;
            b.buf().extend_from_slice(line.as_bytes());
            b.buf().extend_from_slice(b"\r\n");
            let recs: Vec<RecordedCell> = cells
                .iter()
                .map(|c| {
                    let off = line.rfind(&format!("\"{c}\"")).unwrap() + 1;
                    RecordedCell {
                        start: start + off,
                        end: start + off + c.len(),
                        has_escape: false,
                    }
                })
                .collect();
            b.record_row(*tag, start..start + line.len(), &recs)
                .unwrap();
            at += line.len() + 2;
        }
        let (built, _) = b.finish().unwrap();
        let parsed = parse_str(src).unwrap();
        assert_eq!(built.group_order, vec!["PROJ"]);
        let bg = &built.groups["PROJ"];
        let pg = &parsed.groups["PROJ"];
        assert_eq!(bg.headings, pg.headings, "second HEADING row wins");
        assert_eq!(bg.rows.len(), 2, "both sections' rows attach");
        assert_eq!(bg.group_line, pg.group_line, "first section's identity");
        for (i, pr) in pg.rows.iter().enumerate() {
            assert_eq!(bg.rows[i].line, pr.line);
            assert_eq!(bg.value_at(&bg.rows[i], 0), pg.value_at(pr, 0));
        }
    }

    /// No GROUP recorded fails exactly as the walk does.
    #[test]
    fn an_empty_build_is_not_ags4() {
        let b = ParsedFileBuilder::new();
        assert!(matches!(b.finish(), Err(ParseError::NotAgs4(_))));
    }
}
