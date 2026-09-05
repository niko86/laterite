//! AGS4 codec — read the plain-text AGS4 transfer format into core's
//! name-keyed [`ParsedAgs4`].
//!
//! AGS4 files are CSV-like: every field is double-quoted, embedded
//! quotes become `""`, and each row's first field is a TAG that
//! disambiguates its purpose:
//!
//! ```text
//! "GROUP","LOCA"
//! "HEADING","LOCA_ID","LOCA_TYPE",...
//! "UNIT","","","",...
//! "TYPE","ID","PA","2DP",...
//! "DATA","BH01","CP","100.50",...
//! <blank line>
//! "GROUP","SAMP"
//! ...
//! ```
//!
//! Blank lines separate group sections. Since #168 Phase 5 the actual parsing
//! is the shared leaf ([`laterite_ags4_parse`]) — one tokenizer + one
//! source-true walk for the whole toolchain; `from_shared` projects the
//! leaf's positional, *raw* `ParsedFile` into core's name-keyed, *trimmed*
//! shape (re-applying core's trims + UNIT/TYPE padding). The read path opts
//! into the leaf's strict-structure mode (a data reader fails fast on a
//! HEADING/DATA-before-GROUP or a code-less GROUP; the validator keeps the
//! lenient default and reports those as findings instead).

use std::collections::HashMap;
use std::path::Path;

use laterite_ags4_parse::{ParseError, ParseOptions, ParsedFile, ParsedGroup, parse_bytes_opts};

use crate::error::CliError;

/// One AGS4 group section after parsing.
///
/// Since #900 the rows are **span-backed**: the group holds the parse leaf's
/// arena (spans over the shared decoded buffer, the M6 layout) and serves
/// cells through the accessors below, trimmed on read — the whole-file
/// `String`-per-cell materialisation the old `rows` field forced was most
/// of the read door's peak (priced by the #893 diagnosis; the campaign
/// ledger's M8 row carries the numbers, and
/// `ags-wiki/design/dec-read-projection-representation.md` is the shape's
/// record). Construction is private on purpose: this family has changed
/// layout three times (M4, M6, #900), and the constructor tax is paid once
/// so the next change is not another break.
#[derive(Clone)]
pub struct AgsGroup {
    pub(crate) code: String,
    /// Heading names in declaration order, trimmed, duplicate policy applied.
    pub(crate) headings: Vec<String>,
    /// Units row aligned with `headings` when the UNIT row was present
    /// (padded with empty strings); empty when it was absent.
    pub(crate) units: Vec<String>,
    /// TYPE row aligned with `headings` — AGS4 type codes (X / ID / 2DP / …).
    pub(crate) types: Vec<String>,
    pub(crate) repr: GroupRepr,
}

/// Where a group's cells live. Exactly two shapes, by design
/// (dec-read-projection-representation): the span-backed parse, and the ONE
/// owned positional shape that copy-on-write and the builders construct.
#[derive(Clone)]
pub(crate) enum GroupRepr {
    /// Cells resolve from the leaf group's span arena against the shared
    /// decoded buffer, trimmed on read. Nothing per-cell is owned.
    Spans(ParsedGroup),
    /// Owned positional rows, each padded to the heading count. Values are
    /// held **verbatim** (a materialised cell was trimmed when it was copied
    /// out of the spans; a mutated cell keeps exactly what the caller set).
    Owned(Vec<Vec<String>>),
}

impl AgsGroup {
    /// A group built from owned data — the builders' door (excel's
    /// `from_excel`, hand-built test groups). Each row is padded/truncated
    /// to the heading count; values are stored verbatim.
    #[must_use]
    pub fn from_owned_rows(
        code: String,
        headings: Vec<String>,
        units: Vec<String>,
        types: Vec<String>,
        mut rows: Vec<Vec<String>>,
    ) -> AgsGroup {
        for r in &mut rows {
            r.resize(headings.len(), String::new());
        }
        AgsGroup {
            code,
            headings,
            units,
            types,
            repr: GroupRepr::Owned(rows),
        }
    }

    /// The 4-letter group code (trimmed).
    #[must_use]
    pub fn code(&self) -> &str {
        &self.code
    }

    /// Heading names in declaration order.
    #[must_use]
    pub fn headings(&self) -> &[String] {
        &self.headings
    }

    /// Units row aligned with [`Self::headings`] when the UNIT row was
    /// present (padded); empty when it was absent.
    #[must_use]
    pub fn units(&self) -> &[String] {
        &self.units
    }

    /// TYPE row aligned with [`Self::headings`].
    #[must_use]
    pub fn types(&self) -> &[String] {
        &self.types
    }

    /// How many DATA rows.
    #[must_use]
    pub fn n_rows(&self) -> usize {
        match &self.repr {
            GroupRepr::Spans(pg) => pg.rows.len(),
            GroupRepr::Owned(rows) => rows.len(),
        }
    }

    /// Column index of a heading by name.
    #[must_use]
    pub fn col(&self, heading: &str) -> Option<usize> {
        self.headings.iter().position(|h| h == heading)
    }

    /// The cell at `(row, col)` — `Some("")` for a short row's missing tail
    /// (the positional contract), `None` when `row` or `col` is out of range.
    /// A parsed cell comes back trimmed; a mutated one verbatim.
    #[must_use]
    pub fn cell(&self, row: usize, col: usize) -> Option<&str> {
        if col >= self.headings.len() {
            return None;
        }
        match &self.repr {
            GroupRepr::Spans(pg) => pg
                .rows
                .get(row)
                .map(|r| pg.value_at(r, col).unwrap_or("").trim()),
            GroupRepr::Owned(rows) => rows.get(row)?.get(col).map(String::as_str).or(Some("")),
        }
    }

    /// [`Self::cell`] by heading name.
    #[must_use]
    pub fn cell_named(&self, row: usize, heading: &str) -> Option<&str> {
        self.cell(row, self.col(heading)?)
    }

    /// One row's cells, padded to the heading count — the borrowing
    /// projection every read surface renders from. Yields nothing for an
    /// out-of-range row.
    pub fn row_cells(&self, row: usize) -> impl Iterator<Item = &str> + '_ {
        let n = if row < self.n_rows() {
            self.headings.len()
        } else {
            0
        };
        (0..n).map(move |c| self.cell(row, c).unwrap_or(""))
    }

    /// One row's cells as owned `String`s — the ONE owned convenience, for
    /// the callers that hand rows to an owning boundary (the render writers,
    /// the bindings' array builders). Everything else should borrow.
    #[must_use]
    pub fn padded_row(&self, row: usize) -> Vec<String> {
        self.row_cells(row).map(str::to_string).collect()
    }

    /// Overwrite one cell. The first mutation of any kind materialises the
    /// group into its owned shape (copy-on-write — one group's worth, only
    /// on the paths that asked). Returns `false` when `row`/`col` is out of
    /// range, mutating nothing.
    pub fn set_cell(&mut self, row: usize, col: usize, value: String) -> bool {
        if row >= self.n_rows() || col >= self.headings.len() {
            return false;
        }
        self.owned_rows_mut()[row][col] = value;
        true
    }

    /// Append a row (positional; padded/truncated to the heading count).
    /// Materialises like [`Self::set_cell`] — there is no spans-plus-tail
    /// hybrid, by decision.
    pub fn push_row(&mut self, mut cells: Vec<String>) {
        cells.resize(self.headings.len(), String::new());
        self.owned_rows_mut().push(cells);
    }

    /// The one mutation seam: materialise (idempotent) and hand back the
    /// owned rows.
    fn owned_rows_mut(&mut self) -> &mut Vec<Vec<String>> {
        self.materialise();
        let GroupRepr::Owned(rows) = &mut self.repr else {
            unreachable!("materialise() always leaves the owned shape");
        };
        rows
    }

    /// Spans → the one owned shape, applying the read trim exactly as the
    /// old eager conversion did. Idempotent.
    fn materialise(&mut self) {
        if let GroupRepr::Spans(pg) = &self.repr {
            let n = self.headings.len();
            let rows: Vec<Vec<String>> = pg
                .rows
                .iter()
                .map(|r| {
                    (0..n)
                        .map(|c| pg.value_at(r, c).unwrap_or("").trim().to_string())
                        .collect()
                })
                .collect();
            self.repr = GroupRepr::Owned(rows);
        }
    }
}

/// Equality over the logical projection — code, resolved metadata, and every
/// projected cell — so a span-backed group and its materialised twin compare
/// equal, which is the contract the representation split must not leak.
impl PartialEq for AgsGroup {
    fn eq(&self, other: &Self) -> bool {
        self.code == other.code
            && self.headings == other.headings
            && self.units == other.units
            && self.types == other.types
            && self.n_rows() == other.n_rows()
            && (0..self.n_rows()).all(|r| self.row_cells(r).eq(other.row_cells(r)))
    }
}

/// Shape, never contents: a derived Debug would print every cell of a
/// delivery file into panic messages and logs.
#[allow(clippy::missing_fields_in_debug)] // omitting `units`/`types`/`repr` is the point
impl std::fmt::Debug for AgsGroup {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AgsGroup")
            .field("code", &self.code)
            .field("headings", &self.headings)
            .field("rows", &self.n_rows())
            .finish()
    }
}

/// Whole-file parse result. `order` preserves the group-section order from
/// the input file — useful for round-trip emission.
#[derive(Debug, Clone)]
pub struct ParsedAgs4 {
    pub(crate) groups: HashMap<String, AgsGroup>,
    pub(crate) order: Vec<String>,
}

impl ParsedAgs4 {
    /// Assemble from owned groups, preserving their sequence as the file
    /// order (first-seen wins on a duplicate code) — the constructors' door
    /// for callers that build a document from parts (the facade's sliced
    /// path).
    #[must_use]
    pub fn from_groups(groups: Vec<AgsGroup>) -> ParsedAgs4 {
        let mut map = HashMap::with_capacity(groups.len());
        let mut order = Vec::with_capacity(groups.len());
        for g in groups {
            if !map.contains_key(&g.code) {
                order.push(g.code.clone());
                map.insert(g.code.clone(), g);
            }
        }
        ParsedAgs4 { groups: map, order }
    }

    #[must_use]
    pub fn get(&self, code: &str) -> Option<&AgsGroup> {
        self.groups.get(code)
    }

    #[must_use]
    pub fn get_mut(&mut self, code: &str) -> Option<&mut AgsGroup> {
        self.groups.get_mut(code)
    }

    /// The group codes in file order.
    #[must_use]
    pub fn order(&self) -> &[String] {
        &self.order
    }

    /// Remove and return one group.
    pub fn take_group(&mut self, code: &str) -> Option<AgsGroup> {
        let g = self.groups.remove(code);
        if g.is_some() {
            self.order.retain(|c| c != code);
        }
        g
    }

    /// Keep only these groups, dropping the rest (the facade's `only`
    /// filter).
    pub fn retain_only(&mut self, codes: &[String]) {
        self.groups.retain(|code, _| codes.contains(code));
        self.order.retain(|code| codes.contains(code));
    }
}

/// What the reader does when a group declares the same heading name twice.
///
/// AGS4 forbids it (Rule 7), and the validator raises it at error severity —
/// but the *read* surfaces (`lat read`, `laterite-ags4-excel`, node,
/// `read_groups_raw`) never run the rule engine. Left unhandled the collision is
/// not merely lossy, it is **wrong**: rows are keyed by heading name, so the
/// second occurrence overwrites the first, and consumers that walk `headings`
/// positionally then read the survivor's value at *both* positions. The first
/// column's data is gone and the second's is duplicated into its place, leaving
/// a column that looks fully populated and is not.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DuplicateHeadings {
    /// Refuse the file, naming the offending heading. The default: a reader that
    /// cannot represent the file faithfully should say so rather than hand back
    /// a plausible-looking wrong answer.
    #[default]
    Error,
    /// Disambiguate instead of colliding — the 2nd..nth occurrence of a name
    /// become `NAME__2`, `NAME__3`, … in **both** `headings` and the row keys,
    /// so positional reads line up again and no cell is lost.
    ///
    /// The result is deliberately **not valid AGS4** — a suffixed heading is not
    /// a spec heading. This exists to recover data from a broken file, not to
    /// round-trip one.
    Recover,
}

/// What to do with a DATA row that split into MORE fields than its group
/// declares headings. The excess binds to nothing, so the value it belongs to
/// cannot be determined — the same shape as [`DuplicateHeadings`], one axis
/// over: there a name collided, here a position has no name.
///
/// The usual cause is AGS4 Rule 5 — a value containing a comma whose quotes
/// were lost, so `Acme, Bloggs and Co` splits in two.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ExcessFields {
    /// Refuse the file, naming the group, the line and both counts. The
    /// default, for the reason [`DuplicateHeadings::Error`] gives: a reader
    /// that cannot represent the file faithfully should say so rather than
    /// hand back a plausible-looking wrong answer.
    ///
    /// This one is worth refusing even more than a duplicate heading, because
    /// the wrong answer survives everything downstream that might have caught
    /// it. The truncated row satisfies Rule 4, the file then validates clean,
    /// and `certify` mints an `.ags.idx` asserting `errors: measured, count 0`
    /// over a value that is no longer there (#776).
    #[default]
    Error,
    /// Discard the excess, which is what every read did before #776. Kept as an
    /// opt-in for salvaging data from a file that cannot be repaired at source.
    ///
    /// The result is deliberately **lossy**: a value that lost its quotes is
    /// silently shortened. Do not round-trip or certify it — that is the exact
    /// path #776 exists to close.
    Truncate,
}

/// Per-read behaviour switches. Defaults are the strict, faithful choices; a
/// caller opts *into* leniency, never out of it by accident.
#[derive(Debug, Clone, Copy, Default)]
pub struct ReadOptions {
    pub duplicate_headings: DuplicateHeadings,
    pub excess_fields: ExcessFields,
}

impl ReadOptions {
    /// The host bindings' two caller-facing booleans, mapped once. Every
    /// surface exposes exactly `recover_duplicate_headings` /
    /// `truncate_excess_fields` and each used to restate this bool→enum
    /// mapping locally (#923) — the mapping belongs to the type, not to the
    /// callers.
    #[must_use]
    pub fn from_flags(recover_duplicate_headings: bool, truncate_excess_fields: bool) -> Self {
        ReadOptions {
            duplicate_headings: if recover_duplicate_headings {
                DuplicateHeadings::Recover
            } else {
                DuplicateHeadings::Error
            },
            excess_fields: if truncate_excess_fields {
                ExcessFields::Truncate
            } else {
                ExcessFields::Error
            },
        }
    }
}

/// Read an AGS4 file from a path — slurps the bytes and delegates to
/// [`read_ags4_bytes`] (the shared leaf walks the whole buffer at once).
pub fn read_ags4(path: &Path) -> Result<ParsedAgs4, CliError> {
    read_ags4_with(path, ReadOptions::default())
}

/// [`read_ags4`] with explicit [`ReadOptions`].
pub fn read_ags4_with(path: &Path, opts: ReadOptions) -> Result<ParsedAgs4, CliError> {
    let bytes =
        std::fs::read(path).map_err(|e| CliError::Schema(format!("open AGS4 file: {e}")))?;
    read_ags4_bytes_with(&bytes, opts)
}

/// Read AGS4 from an in-memory byte buffer through the shared parse leaf, then
/// project to core's name-keyed [`ParsedAgs4`] (#168 Phase 5). Uses the leaf's
/// lean profile with `strict_structure` ON, so the locked structure terminals
/// (`error_mapping.rs`) are raised exactly as the retired csv reader did. The
/// leaf also strips a leading BOM, so a sliced read of a BOM file is clean.
///
/// A file with **no GROUP rows** is NOT an error for the reader — the retired
/// csv reader returned an empty parse for it, and downstream consumers
/// (e.g. the excel exporter's "No valid AGS4 data found" check) rely on that.
/// So the leaf's `NotAgs4` is mapped back to an empty `ParsedAgs4`; structural
/// violations (pre-GROUP rows, a code-less GROUP) still propagate as errors.
pub fn read_ags4_bytes(bytes: &[u8]) -> Result<ParsedAgs4, CliError> {
    read_ags4_bytes_with(bytes, ReadOptions::default())
}

/// [`read_ags4_bytes`] with explicit [`ReadOptions`].
pub fn read_ags4_bytes_with(bytes: &[u8], read_opts: ReadOptions) -> Result<ParsedAgs4, CliError> {
    let opts = ParseOptions {
        strict_structure: true,
        ..ParseOptions::lean()
    };
    match parse_bytes_opts(bytes, opts) {
        Ok(parsed) => from_shared(parsed, read_opts),
        Err(ParseError::NotAgs4(_)) => Ok(ParsedAgs4 {
            groups: HashMap::new(),
            order: Vec::new(),
        }),
        Err(e) => Err(map_parse_err(e)),
    }
}

/// Resolve a group's heading list against [`DuplicateHeadings`].
///
/// Returns the names to key rows by. Under `Error` a repeat is a hard stop;
/// under `Recover` the nth occurrence (n ≥ 2) becomes `NAME__n`. The suffix is
/// applied to the *trimmed* name and counted per distinct name, so a third
/// `LOCA_ID` is `LOCA_ID__3`, not `LOCA_ID__2__2`.
///
/// A generated name could in principle collide with a real heading that already
/// ends `__2`, so the result is re-checked; that is a malformed file either way,
/// and silently merging two columns is the one outcome this function exists to
/// prevent.
fn resolve_headings(
    code: &str,
    headings: Vec<String>,
    policy: DuplicateHeadings,
) -> Result<Vec<String>, CliError> {
    let mut seen: HashMap<String, usize> = HashMap::with_capacity(headings.len());
    let mut out: Vec<String> = Vec::with_capacity(headings.len());
    for h in headings {
        let n = seen.entry(h.clone()).or_insert(0);
        *n += 1;
        if *n == 1 {
            out.push(h);
            continue;
        }
        match policy {
            DuplicateHeadings::Error => {
                return Err(CliError::DuplicateHeading {
                    group: code.to_string(),
                    heading: h,
                });
            }
            DuplicateHeadings::Recover => out.push(format!("{h}__{n}")),
        }
    }
    if policy == DuplicateHeadings::Recover {
        let mut check: std::collections::HashSet<&str> = std::collections::HashSet::new();
        for h in &out {
            if !check.insert(h.as_str()) {
                return Err(CliError::DuplicateHeading {
                    group: code.to_string(),
                    heading: h.clone(),
                });
            }
        }
    }
    Ok(out)
}

/// Project the shared leaf's positional, RAW [`ParsedFile`] into core's
/// name-keyed, TRIMMED [`ParsedAgs4`] (#168 Phase 5) — **policy eagerly,
/// strings lazily** (#900). The trims core has always applied to
/// heading/unit/type happen here as before; cell values stay in the leaf's
/// span arena and are trimmed by the accessors on read, so no whole-file
/// owned materialisation exists on any read path (the #893-measured slab).
/// First-seen wins on a duplicate (trimmed) code, matching the csv reader
/// this replaced.
///
/// Duplicate heading NAMES within a group are governed by
/// [`ReadOptions::duplicate_headings`] — see [`resolve_headings`]. A row
/// with MORE fields than headings is governed by
/// [`ReadOptions::excess_fields`] and refused by default (#776) — the walk
/// below reads span COUNTS, allocating nothing, so the refusal still fires
/// at read time, never deferred to first access. Under `Truncate` the
/// excess simply never projects: every accessor is heading-count wide, so
/// truncation is a property of the projection now, not an edit.
fn from_shared(pf: ParsedFile, read_opts: ReadOptions) -> Result<ParsedAgs4, CliError> {
    // Taken BY VALUE: each group's descriptor rows are moved (not cloned)
    // into the resolved metadata, and the group's span structure moves whole
    // into the group it backs.
    let ParsedFile {
        groups: mut pgroups,
        group_order,
        ..
    } = pf;
    let mut groups: HashMap<String, AgsGroup> = HashMap::with_capacity(group_order.len());
    let mut order: Vec<String> = Vec::with_capacity(group_order.len());
    for raw_code in &group_order {
        let code = raw_code.trim().to_string();
        if groups.contains_key(&code) {
            continue; // first-seen wins on the trimmed code (csv-reader parity)
        }
        let Some(mut pg) = pgroups.remove(raw_code) else {
            continue; // group_order and groups are built together; defensive.
        };
        // The leaf group's own descriptor copies are dead weight once moved
        // out — the retained `pg` serves spans only.
        let headings: Vec<String> = std::mem::take(&mut pg.headings)
            .into_iter()
            .map(trim_owned)
            .collect();
        // Resolve BEFORE the UNIT/TYPE pad below, so those still align with the
        // heading count — `Recover` renames headings, it never adds or drops one.
        let headings = resolve_headings(&code, headings, read_opts.duplicate_headings)?;
        // Pad/truncate to the heading count — but ONLY when the row was actually
        // present (the csv reader resized inside its UNIT/TYPE arm; a group with
        // no UNIT row kept an empty vec, never padded). `unit_line`/`type_line`
        // are `Some` iff the leaf saw that descriptor row.
        let mut units: Vec<String> = std::mem::take(&mut pg.units)
            .into_iter()
            .map(trim_owned)
            .collect();
        if pg.unit_line.is_some() {
            units.resize(headings.len(), String::new());
        }
        let mut types: Vec<String> = std::mem::take(&mut pg.types)
            .into_iter()
            .map(trim_owned)
            .collect();
        if pg.type_line.is_some() {
            types.resize(headings.len(), "X".to_string());
        }
        // The eager arity walk (#776): a value bound to no heading cannot be
        // represented, so the default read still refuses it HERE — in group
        // order, first offending row first, exactly as the eager conversion
        // did — reading span counts only.
        if read_opts.excess_fields == ExcessFields::Error {
            for r in &pg.rows {
                if r.n_values() > headings.len() {
                    return Err(CliError::ExcessFields {
                        group: code,
                        line: r.line,
                        found: r.n_values(),
                        declared: headings.len(),
                    });
                }
            }
        }
        order.push(code.clone());
        groups.insert(
            code.clone(),
            AgsGroup {
                code,
                headings,
                units,
                types,
                repr: GroupRepr::Spans(pg),
            },
        );
    }
    Ok(ParsedAgs4 { groups, order })
}

/// Trim a value WITHOUT reallocating it. `s.trim().to_string()` allocates even
/// when there is nothing to trim, which is the overwhelmingly common case for
/// quoted AGS4 fields; this shrinks in place and copies nothing.
fn trim_owned(mut s: String) -> String {
    let lead = s.len() - s.trim_start().len();
    let end = s.trim_end().len();
    if lead == 0 && end == s.len() {
        return s;
    }
    s.drain(..lead);
    // `end` is the trimmed length measured from the ORIGINAL start; after draining
    // `lead` leading bytes the survivor is `end - lead` long. For an all-whitespace
    // value `trim_start` consumes everything, so `lead == s.len()` while `end == 0`
    // — and `end - lead` underflows. Saturate: the result is the empty string, and
    // `end >= lead` in every other case, so this is a plain subtraction there.
    s.truncate(end.saturating_sub(lead));
    s
}

/// Map the shared leaf's `ParseError` into core's `CliError`. The structure
/// terminals carry their exact messages verbatim (`error_mapping.rs` pins them);
/// the non-UTF-8 wording is the leaf's — the csv reader's old wording changes
/// (asserted loosely in `error_mapping.rs`, ratified at Phase 7).
pub(crate) fn map_parse_err(e: ParseError) -> CliError {
    let msg = match e {
        ParseError::NotAgs4(m) | ParseError::Structure(m) => m,
        ParseError::UnsupportedEdition { found } => format!("unsupported AGS edition {found:?}"),
        ParseError::NotUtf8 => "input is not valid UTF-8".to_string(),
        ParseError::TooLarge => {
            "file too large: decoded text exceeds the 4 GiB span space".to_string()
        }
    };
    CliError::Schema(msg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn write_fixture(path: &Path, content: &str) {
        let mut f = std::fs::File::create(path).unwrap();
        f.write_all(content.as_bytes()).unwrap();
    }

    const FIXTURE: &str = r#""GROUP","PROJ"
"HEADING","PROJ_ID","PROJ_NAME"
"UNIT","",""
"TYPE","X","X"
"DATA","P1","Test project"

"GROUP","LOCA"
"HEADING","LOCA_ID","LOCA_TYPE","LOCA_NATE"
"UNIT","","","m"
"TYPE","ID","PA","2DP"
"DATA","BH01","CP","100.50"
"DATA","BH02","TP","200.75"
"#;

    #[test]
    fn parses_two_groups_in_order() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.ags");
        write_fixture(&path, FIXTURE);

        let parsed = read_ags4(&path).unwrap();
        assert_eq!(parsed.order, vec!["PROJ", "LOCA"]);

        let proj = parsed.get("PROJ").unwrap();
        assert_eq!(proj.headings, vec!["PROJ_ID", "PROJ_NAME"]);
        assert_eq!(proj.types, vec!["X", "X"]);
        assert_eq!(proj.n_rows(), 1);
        assert_eq!(proj.cell_named(0, "PROJ_ID"), Some("P1"));

        let loca = parsed.get("LOCA").unwrap();
        assert_eq!(loca.headings, vec!["LOCA_ID", "LOCA_TYPE", "LOCA_NATE"]);
        assert_eq!(loca.units, vec!["", "", "m"]);
        assert_eq!(loca.n_rows(), 2);
        assert_eq!(loca.cell_named(1, "LOCA_NATE"), Some("200.75"));
    }

    #[test]
    fn read_ags4_bytes_matches_path() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.ags");
        write_fixture(&path, FIXTURE);

        let from_path = read_ags4(&path).unwrap();
        let from_bytes = read_ags4_bytes(FIXTURE.as_bytes()).unwrap();

        assert_eq!(from_path.order, from_bytes.order);
        for code in &from_path.order {
            let a = from_path.get(code).unwrap();
            let b = from_bytes.get(code).unwrap();
            // PartialEq is the logical projection: metadata + every cell.
            assert_eq!(a, b);
        }
    }

    #[test]
    fn missing_unit_row_leaves_units_empty_not_padded() {
        // A group with NO UNIT row keeps an empty units vec — the retired csv
        // reader only padded inside its UNIT arm, so an absent row was never
        // padded. Regression guard for the from_shared padding fix (#168 Phase 5).
        let f = "\"GROUP\",\"PROJ\"\n\"HEADING\",\"PROJ_ID\",\"PROJ_NAME\"\n\"TYPE\",\"ID\",\"X\"\n\"DATA\",\"P1\",\"n\"\n";
        let parsed = read_ags4_bytes(f.as_bytes()).unwrap();
        let proj = parsed.get("PROJ").unwrap();
        assert!(
            proj.units.is_empty(),
            "no UNIT row → empty units, not padded"
        );
        // a present TYPE row IS padded/aligned to the heading count
        assert_eq!(proj.types, vec!["ID", "X"]);
        assert_eq!(proj.cell_named(0, "PROJ_ID"), Some("P1"));
    }

    #[test]
    fn read_path_opts_into_strict_structure() {
        // core's reader fails fast on a HEADING before any GROUP (the leaf's
        // strict mode), with the exact locked message error_mapping.rs pins.
        let err = read_ags4_bytes(b"\"HEADING\",\"X\"\n").unwrap_err();
        assert!(
            matches!(err, CliError::Schema(ref m) if m == "HEADING row before any GROUP"),
            "got {err:?}"
        );
    }

    #[test]
    fn no_group_file_reads_as_empty_not_an_error() {
        // A file with no GROUP rows is an EMPTY parse for the reader, not an
        // error — matching the retired csv reader. Downstream consumers (the
        // excel exporter's "No valid AGS4 data found" check) rely on this.
        // Regression guard: Phase 5's leaf returns NotAgs4 here, mapped back to
        // an empty parse (caught the empty-file→xlsx python-ags4 parity test).
        for empty in [b"".as_slice(), b"\r\n\r\n".as_slice()] {
            let parsed = read_ags4_bytes(empty).unwrap();
            assert!(parsed.order.is_empty() && parsed.groups.is_empty());
        }
    }

    /// A group whose HEADING row repeats a name. `LOCA_ID` appears at columns 0
    /// and 2 with DIFFERENT values, which is what makes the old behaviour
    /// detectable rather than merely lossy.
    const DUP_HEADING: &[u8] = b"\"GROUP\",\"LOCA\"\n\
\"HEADING\",\"LOCA_ID\",\"LOCA_GL\",\"LOCA_ID\"\n\
\"UNIT\",\"\",\"m\",\"\"\n\
\"TYPE\",\"ID\",\"2DP\",\"ID\"\n\
\"DATA\",\"FIRST\",\"1.00\",\"SECOND\"\n";

    #[test]
    fn duplicate_heading_is_fatal_by_default() {
        let err = read_ags4_bytes(DUP_HEADING).unwrap_err();
        let CliError::DuplicateHeading { group, heading } = err else {
            panic!("expected DuplicateHeading, got {err:?}");
        };
        assert_eq!((group.as_str(), heading.as_str()), ("LOCA", "LOCA_ID"));
    }

    /// The regression this exists to prevent. Rows are keyed by heading name, so
    /// before the guard the second `LOCA_ID` overwrote the first and a
    /// positional read — `headings[i]` → `row[&headings[i]]`, the shape
    /// `laterite-ags4-excel`, node and `read_groups_raw` all use — returned
    /// `["SECOND", "1.00", "SECOND"]`. `FIRST` was gone AND `SECOND` was
    /// duplicated into its column, so the column looked populated and was wrong.
    /// Recovery must keep both, in their own positions.
    #[test]
    fn recovery_keeps_every_cell_and_fixes_the_positional_read() {
        let opts = ReadOptions {
            duplicate_headings: DuplicateHeadings::Recover,
            ..ReadOptions::default()
        };
        let parsed = read_ags4_bytes_with(DUP_HEADING, opts).expect("recovers");
        let g = parsed.get("LOCA").expect("LOCA");
        assert_eq!(g.headings, ["LOCA_ID", "LOCA_GL", "LOCA_ID__2"]);

        let positional: Vec<&str> = g.row_cells(0).collect();
        assert_eq!(positional, ["FIRST", "1.00", "SECOND"]);

        // UNIT/TYPE still align: renaming must not change the column count.
        assert_eq!(g.units, ["", "m", ""]);
        assert_eq!(g.types, ["ID", "2DP", "ID"]);
    }

    #[test]
    fn recovery_numbers_each_repeat_from_two() {
        let src = b"\"GROUP\",\"LOCA\"\n\
\"HEADING\",\"A\",\"A\",\"A\",\"B\"\n\
\"DATA\",\"1\",\"2\",\"3\",\"4\"\n";
        let opts = ReadOptions {
            duplicate_headings: DuplicateHeadings::Recover,
            ..ReadOptions::default()
        };
        let parsed = read_ags4_bytes_with(src, opts).expect("recovers");
        let g = parsed.get("LOCA").expect("LOCA");
        // Counted per distinct name, so the third A is A__3 — not A__2__2.
        assert_eq!(g.headings, ["A", "A__2", "A__3", "B"]);
    }

    /// A file that is already fine must be byte-identical under both policies —
    /// the guard costs nothing and changes nothing for the 99.99% case.
    #[test]
    fn a_clean_file_reads_identically_under_either_policy() {
        let src = b"\"GROUP\",\"LOCA\"\n\"HEADING\",\"LOCA_ID\"\n\"DATA\",\"BH01\"\n";
        let strict = read_ags4_bytes(src).expect("clean");
        let recover = read_ags4_bytes_with(
            src,
            ReadOptions {
                duplicate_headings: DuplicateHeadings::Recover,
                ..ReadOptions::default()
            },
        )
        .expect("clean");
        assert_eq!(strict.order, recover.order);
        assert_eq!(
            strict.get("LOCA").unwrap().headings,
            recover.get("LOCA").unwrap().headings
        );
    }

    /// The #776 fixture. `PROJ_CLNT` was `"Acme, Bloggs and Co"` and lost its
    /// quotes, so AGS4 Rule 5's separator split one authored value into two
    /// fields — and the second binds to no heading at all. Every other line is
    /// well-formed, which is exactly what made this survive: the file looks
    /// ordinary and the loss is one field wide.
    const EXCESS_FIELDS: &[u8] = b"\"GROUP\",\"PROJ\"\n\
\"HEADING\",\"PROJ_ID\",\"PROJ_CLNT\"\n\
\"UNIT\",\"\",\"\"\n\
\"TYPE\",\"ID\",\"X\"\n\
\"DATA\",\"P1\",Acme, Bloggs and Co\n";

    /// Worth refusing more than a duplicate heading, because nothing downstream
    /// catches it: the shortened row still satisfies Rule 4, so the file
    /// validates clean and `certify` will mint an index asserting zero errors
    /// over a value that is no longer there.
    #[test]
    fn excess_fields_are_fatal_by_default() {
        let err = read_ags4_bytes(EXCESS_FIELDS).unwrap_err();
        let CliError::ExcessFields {
            group,
            line,
            found,
            declared,
        } = err
        else {
            panic!("expected ExcessFields, got {err:?}");
        };
        assert_eq!((group.as_str(), line, found, declared), ("PROJ", 5, 3, 2));
    }

    /// The opt-in keeps the pre-#776 behaviour, and the assertion states plainly
    /// what that behaviour costs: `Bloggs and Co` is gone, and `PROJ_CLNT` reads
    /// as a complete-looking `Acme`.
    #[test]
    fn truncate_discards_the_unbindable_field() {
        let opts = ReadOptions {
            excess_fields: ExcessFields::Truncate,
            ..ReadOptions::default()
        };
        let parsed = read_ags4_bytes_with(EXCESS_FIELDS, opts).expect("truncates");
        let g = parsed.get("PROJ").expect("PROJ");
        assert_eq!(g.headings, ["PROJ_ID", "PROJ_CLNT"]);
        assert_eq!(g.cell_named(0, "PROJ_CLNT"), Some("Acme"));
    }

    /// The control that says the guard keys on the LOST QUOTES, not on the
    /// comma. Quoted, the same authored value is one field and reads whole
    /// under the strict default.
    #[test]
    fn a_quoted_comma_is_one_field_and_still_reads() {
        let src = b"\"GROUP\",\"PROJ\"\n\
\"HEADING\",\"PROJ_ID\",\"PROJ_CLNT\"\n\
\"DATA\",\"P1\",\"Acme, Bloggs and Co\"\n";
        let g = read_ags4_bytes(src).expect("clean");
        let g = g.get("PROJ").expect("PROJ");
        assert_eq!(g.cell_named(0, "PROJ_CLNT"), Some("Acme, Bloggs and Co"));
    }

    /// The other direction is NOT symmetrical and must not become so. A row with
    /// FEWER fields than headings loses nothing — the missing tail is knowable
    /// (it is empty) — so it still pads to `""` as it always has. Only the
    /// unbindable direction is fatal.
    #[test]
    fn a_short_row_still_pads_rather_than_failing() {
        let src = b"\"GROUP\",\"PROJ\"\n\
\"HEADING\",\"PROJ_ID\",\"PROJ_CLNT\"\n\
\"DATA\",\"P1\"\n";
        let parsed = read_ags4_bytes(src).expect("short rows are fine");
        let g = parsed.get("PROJ").expect("PROJ");
        assert_eq!(g.cell_named(0, "PROJ_CLNT"), Some(""));
    }

    /// `trim_owned`'s in-place path only runs when there is whitespace to strip —
    /// and quoted AGS values almost never have any, so the fixtures above exercise
    /// only its no-op fast path. These call it directly across all four cases so
    /// the guard (`lead == 0 && end == len`) and the `end - lead` truncation are
    /// each pinned by a value that changes if either is wrong.
    #[test]
    fn trim_owned_strips_leading_trailing_and_both() {
        assert_eq!(trim_owned("x".to_string()), "x"); // fast path: nothing to do
        assert_eq!(trim_owned("  x".to_string()), "x"); // leading only
        assert_eq!(trim_owned("x  ".to_string()), "x"); // trailing only
        // Leading AND trailing of DIFFERENT widths — `end - lead` must be the new
        // length; `end + lead` leaves trailing space, `end / lead` truncates short.
        assert_eq!(trim_owned("  hello   ".to_string()), "hello");
        assert_eq!(trim_owned("   ".to_string()), ""); // all whitespace
    }

    /// The re-check branch `resolve_headings` runs only under `Recover`: a
    /// generated `NAME__2` can collide with a real heading that already ends
    /// `__2`. Nothing else exercises it — a normal Recover file never hits the
    /// collision — so without this the whole `policy == Recover` guard reads clean.
    #[test]
    fn recovery_rejects_a_synthetic_suffix_that_collides_with_a_real_heading() {
        // Two LOCA_ID (the 2nd → LOCA_ID__2) plus a real LOCA_ID__2 already present.
        let src = b"\"GROUP\",\"LOCA\"\n\
\"HEADING\",\"LOCA_ID\",\"LOCA_ID\",\"LOCA_ID__2\"\n\
\"DATA\",\"a\",\"b\",\"c\"\n";
        let opts = ReadOptions {
            duplicate_headings: DuplicateHeadings::Recover,
            ..ReadOptions::default()
        };
        let err = read_ags4_bytes_with(src, opts).unwrap_err();
        let CliError::DuplicateHeading { group, heading } = err else {
            panic!("expected DuplicateHeading from the re-check, got {err:?}");
        };
        assert_eq!((group.as_str(), heading.as_str()), ("LOCA", "LOCA_ID__2"));
    }

    // ---- the #900 machinery: accessor edges + the copy-on-write seam ----

    /// `cell`'s three range outcomes are load-bearing distinctions: `None`
    /// only for a coordinate outside the projection, `Some("")` for a short
    /// row's missing tail (the positional contract — a real row, an absent
    /// value).
    #[test]
    fn cell_distinguishes_out_of_range_from_short_row_tail() {
        // LOCA has 3 headings; give it a 1-value short row.
        let src = b"\"GROUP\",\"LOCA\"\n\
\"HEADING\",\"LOCA_ID\",\"LOCA_TYPE\",\"LOCA_NATE\"\n\
\"DATA\",\"BH01\"\n";
        let parsed = read_ags4_bytes(src).unwrap();
        let g = parsed.get("LOCA").unwrap();
        assert_eq!(g.cell(0, 0), Some("BH01"));
        assert_eq!(g.cell(0, 2), Some(""), "short-row tail is present-empty");
        assert_eq!(g.cell(0, 3), None, "column beyond the headings");
        assert_eq!(g.cell(1, 0), None, "row beyond the data");
        assert_eq!(g.row_cells(1).count(), 0, "OOB row yields nothing");
        assert_eq!(
            g.padded_row(0),
            vec!["BH01".to_string(), String::new(), String::new()]
        );
    }

    /// The copy-on-write seam: the first mutation flips a parsed group to the
    /// owned shape, and that flip must apply the read trim to every cell it
    /// copies — otherwise an unrelated `set_cell` would change what a
    /// neighbouring cell reads back as.
    #[test]
    fn set_cell_materialises_without_disturbing_neighbours() {
        // `  P1  ` reads trimmed from spans; it must STAY trimmed once an
        // unrelated cell's mutation materialises the group.
        let src = b"\"GROUP\",\"PROJ\"\n\
\"HEADING\",\"PROJ_ID\",\"PROJ_NAME\"\n\
\"DATA\",\"  P1  \",\"Test project\"\n";
        let mut parsed = read_ags4_bytes(src).unwrap();
        let g = parsed.get_mut("PROJ").unwrap();
        assert_eq!(g.cell(0, 0), Some("P1"));

        assert!(g.set_cell(0, 1, "Renamed".into()));
        assert_eq!(g.cell(0, 1), Some("Renamed"));
        assert_eq!(g.cell(0, 0), Some("P1"), "materialise keeps the trim");

        // A mutated cell is verbatim — the trim is a READ projection of the
        // parse, not a normalisation of caller data.
        assert!(g.set_cell(0, 0, "  spaced  ".into()));
        assert_eq!(g.cell(0, 0), Some("  spaced  "));
    }

    /// An out-of-range `set_cell` refuses without mutating — and without
    /// materialising, though only the refusal is observable.
    #[test]
    fn set_cell_out_of_range_refuses_and_changes_nothing() {
        let src = b"\"GROUP\",\"PROJ\"\n\
\"HEADING\",\"PROJ_ID\"\n\
\"DATA\",\"P1\"\n";
        let mut parsed = read_ags4_bytes(src).unwrap();
        let g = parsed.get_mut("PROJ").unwrap();
        assert!(!g.set_cell(1, 0, "x".into()), "row out of range");
        assert!(!g.set_cell(0, 1, "x".into()), "col out of range");
        assert_eq!(g.cell(0, 0), Some("P1"));
    }

    /// `push_row` materialises too (no spans-plus-tail hybrid, by decision)
    /// and pads/truncates the new row to the heading count.
    #[test]
    fn push_row_materialises_pads_and_truncates() {
        let src = b"\"GROUP\",\"LOCA\"\n\
\"HEADING\",\"LOCA_ID\",\"LOCA_TYPE\"\n\
\"DATA\",\"BH01\",\"CP\"\n";
        let mut parsed = read_ags4_bytes(src).unwrap();
        let g = parsed.get_mut("LOCA").unwrap();
        g.push_row(vec!["BH02".into()]); // short: padded
        g.push_row(vec!["BH03".into(), "TP".into(), "surplus".into()]); // long: truncated
        assert_eq!(g.n_rows(), 3);
        assert_eq!(g.cell(0, 0), Some("BH01"), "parsed row survives the flip");
        assert_eq!(g.cell(1, 1), Some(""));
        assert_eq!(g.cell(2, 1), Some("TP"));
        assert_eq!(g.cell(2, 2), None, "the surplus did not widen the group");
    }

    /// `PartialEq` is the logical projection, so a span-backed group and its
    /// materialised twin compare equal — the representation split must not
    /// leak through equality.
    #[test]
    fn a_materialised_twin_stays_equal() {
        let src = b"\"GROUP\",\"PROJ\"\n\
\"HEADING\",\"PROJ_ID\",\"PROJ_NAME\"\n\
\"DATA\",\"  P1  \",\"Test project\"\n";
        let parsed = read_ags4_bytes(src).unwrap();
        let spans = parsed.get("PROJ").unwrap();

        let mut twin = spans.clone();
        // Setting a cell to the value it already projects materialises the
        // twin while changing nothing logical.
        let kept = twin.cell(0, 1).unwrap().to_string();
        assert!(twin.set_cell(0, 1, kept));
        assert_eq!(spans, &twin);
    }

    /// `from_owned_rows` is the builders' door: rows resize to the heading
    /// count and values are stored verbatim (no trim — the trim belongs to
    /// the parse projection).
    #[test]
    fn from_owned_rows_resizes_and_keeps_values_verbatim() {
        let g = AgsGroup::from_owned_rows(
            "LOCA".into(),
            vec!["LOCA_ID".into(), "LOCA_TYPE".into()],
            vec![String::new(), String::new()],
            vec!["ID".into(), "PA".into()],
            vec![
                vec!["  BH01  ".into()],                          // short: padded
                vec!["BH02".into(), "TP".into(), "extra".into()], // long: truncated
            ],
        );
        assert_eq!(g.n_rows(), 2);
        assert_eq!(g.cell(0, 0), Some("  BH01  "), "verbatim, not trimmed");
        assert_eq!(g.cell(0, 1), Some(""));
        assert_eq!(g.cell(1, 1), Some("TP"));
        assert_eq!(g.cell(1, 2), None);
    }
}
