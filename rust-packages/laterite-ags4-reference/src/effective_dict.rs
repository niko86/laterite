//! The Rule 18 **effective dictionary** — the standard dictionary ∪ the
//! delivery file's own DICT group — as the one shared implementation (#777).
//!
//! AGS4 Rule 18 lets a file declare user-defined groups and headings in its
//! DICT group; Rules 7/9/10a–c/19b all validate against that union, and a
//! read-only consumer needs the same union to bind a custom group's columns
//! (its types come from the dictionary, not the file's TYPE row alone). This
//! module used to exist twice, privately, inside `laterite-ags4-validator`
//! (O-25/O-29 recorded the duplication as deferred debt); it lives HERE
//! because this leaf already owns [`crate::dict::Dictionary`] and the parse
//! leaf, so the validator gains no new dependency edge and a reader that
//! takes only `laterite-ags4-core` reaches it through core's re-export.
//!
//! **This is a read of DICT, not a check of it.** Malformed DICT content
//! contributes nothing rather than erroring — reporting it is Rule 18's job,
//! in the validator. Contrast [`crate::overlay::parse_dict`], which ingests a
//! *standalone* `--dict` dictionary file and is deliberately strict: a bad
//! dictionary FILE is refused, a bad DICT group inside a delivery file is a
//! finding on that file, never a refusal to read it.
//!
//! CLEAN-ROOM. python-ags4 (LGPL-3.0) was read only to learn its
//! interpretation — facts about the AGS standard, not copyrightable:
//! heading membership is type-agnostic (its Rule 7/9/10a/10b lookups filter
//! on `DICT_GRP` alone, never `DICT_TYPE`), parentage comes from
//! `DICT_TYPE = "GROUP"` rows only, and duplicate declarations resolve
//! first-wins. No code, structure, or wording was copied.
//!
//! Values are never trimmed HERE — the overlay carries exactly what the
//! caller's parse handed over. Through [`FileDict::from_parsed`] that means
//! RAW, the parse leaf's policy (the validator judges whitespace, so a
//! reader must not repair it silently); an adapter over a trimming parse —
//! core's codec route is one — inherits that parse's policy instead.

use std::collections::{BTreeSet, HashMap, HashSet};

use laterite_ags4_parse::ParsedFile;

use crate::dict::{Dictionary, HeadingRef};

/// The one `DICT_STAT` classification predicate: case-insensitive containment,
/// so `KEY`, `key` and `KEY+REQUIRED` all count as KEY. Every reader of the
/// file half's status — the union's `fields_with_status`, `FileDict::
/// key_headings`, and through them the keychain's effective minting — goes
/// through here, so "what counts as a KEY declaration" exists exactly once.
pub(crate) fn status_contains(status: &str, want: &str) -> bool {
    status.to_ascii_uppercase().contains(want)
}

/// One heading declared by the delivery file's own DICT group, with every
/// column a `HEADING`-carrying row can contribute. Field vocabulary mirrors
/// [`crate::dict::DictEntry`] so union lookups read the same either side.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileHeading {
    /// `DICT_HDNG`.
    pub heading: String,
    /// `DICT_STAT` — raw; match case-insensitively (`KEY` / `REQUIRED`).
    pub status: String,
    /// `DICT_DTYP` — the AGS data type a reader binds columns with.
    pub ags_type: String,
    /// `DICT_UNIT`.
    pub unit: String,
    /// `DICT_DESC` — carried so a union lookup can fill [`HeadingRef::desc`].
    pub desc: String,
}

/// One DICT row, borrowed — the funnel every constructor feeds so the
/// classification semantics exist exactly once. Surfaces holding a parse
/// shape this crate does not know (core's name-keyed codec rows) adapt into
/// this and call [`FileDict::from_rows`].
#[derive(Debug, Clone, Copy)]
pub struct DictRow<'a> {
    /// `DICT_TYPE` — decides parentage (`"GROUP"` rows), never membership.
    pub dict_type: &'a str,
    /// `DICT_GRP`.
    pub group: &'a str,
    /// `DICT_HDNG`.
    pub heading: &'a str,
    /// `DICT_STAT`.
    pub status: &'a str,
    /// `DICT_DTYP`.
    pub ags_type: &'a str,
    /// `DICT_UNIT`.
    pub unit: &'a str,
    /// `DICT_PGRP`.
    pub parent: &'a str,
    /// `DICT_DESC`.
    pub desc: &'a str,
}

/// The delivery file's own DICT group, read tolerantly (never validated).
///
/// Two independent maps, because a row contributes on two independent axes:
/// any row naming a group and a heading declares that heading (regardless of
/// `DICT_TYPE` — python-ags4's membership lookups do the same), while only a
/// `DICT_TYPE = "GROUP"` row declares parentage. First occurrence wins on
/// both axes, matching python-ags4's first-row reads; per-group heading
/// order is file order, which is the Rule 18a order Rule 7 appends by.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FileDict {
    headings: HashMap<String, Vec<FileHeading>>,
    parents: HashMap<String, String>,
}

impl FileDict {
    /// Read the DICT group out of a parsed delivery file. No DICT group, or
    /// one missing its `DICT_GRP` column, yields an empty overlay — the
    /// standard dictionary alone is then the effective dictionary.
    #[must_use]
    pub fn from_parsed(parsed: &ParsedFile) -> Self {
        let mut out = Self::default();
        let Some(dictg) = parsed.groups.get("DICT") else {
            return out;
        };
        // Columns resolved by NAME from the HEADING row, so column
        // reordering does not break the read; a missing optional column
        // degrades per-row to "" and that row contributes what it can.
        let (ti, gi, hi, si, yi, ui, pi, di) = (
            dictg.col("DICT_TYPE"),
            dictg.col("DICT_GRP"),
            dictg.col("DICT_HDNG"),
            dictg.col("DICT_STAT"),
            dictg.col("DICT_DTYP"),
            dictg.col("DICT_UNIT"),
            dictg.col("DICT_PGRP"),
            dictg.col("DICT_DESC"),
        );
        if gi.is_none() {
            return out; // nothing can be placed without a group column
        }
        for row in &dictg.rows {
            let get =
                |i: Option<usize>| i.and_then(|i| row.values.get(i)).map_or("", String::as_str);
            out.insert(&DictRow {
                dict_type: get(ti),
                group: get(gi),
                heading: get(hi),
                status: get(si),
                ags_type: get(yi),
                unit: get(ui),
                parent: get(pi),
                desc: get(di),
            });
        }
        out
    }

    /// Build from already-extracted rows — the adapter entry for surfaces
    /// whose parse shape this crate does not know.
    #[must_use]
    pub fn from_rows<'a>(rows: impl IntoIterator<Item = DictRow<'a>>) -> Self {
        let mut out = Self::default();
        for row in rows {
            out.insert(&row);
        }
        out
    }

    fn insert(&mut self, row: &DictRow<'_>) {
        if !row.group.is_empty() && !row.heading.is_empty() {
            let v = self.headings.entry(row.group.to_string()).or_default();
            if !v.iter().any(|h| h.heading == row.heading) {
                v.push(FileHeading {
                    heading: row.heading.to_string(),
                    status: row.status.to_string(),
                    ags_type: row.ags_type.to_string(),
                    unit: row.unit.to_string(),
                    desc: row.desc.to_string(),
                });
            }
        }
        if row.dict_type == "GROUP" && !row.group.is_empty() {
            self.parents
                .entry(row.group.to_string())
                .or_insert_with(|| row.parent.to_string());
        }
    }

    /// Headings declared for `group`, in file (Rule 18a) order.
    #[must_use]
    pub fn headings(&self, group: &str) -> &[FileHeading] {
        self.headings.get(group).map_or(&[], Vec::as_slice)
    }

    /// The declaration of one heading under one group, if the file made one.
    #[must_use]
    pub fn heading(&self, group: &str, name: &str) -> Option<&FileHeading> {
        self.headings(group).iter().find(|h| h.heading == name)
    }

    /// Raw `DICT_PGRP` from the group's `GROUP`-type row. `Some("")` means
    /// declared parentless; `None` means no `GROUP`-type row declared it.
    #[must_use]
    pub fn parent(&self, group: &str) -> Option<&str> {
        self.parents.get(group).map(String::as_str)
    }

    /// Every group the file's DICT touches (by a `GROUP`-type row or by a
    /// heading declaration), sorted — the set a reader enumerates to find
    /// file-declared groups.
    #[must_use]
    pub fn groups(&self) -> BTreeSet<&str> {
        self.parents
            .keys()
            .chain(self.headings.keys())
            .map(String::as_str)
            .collect()
    }

    /// True when the file declared nothing (no DICT group, or an unreadable
    /// one) — the effective dictionary is then the standard one alone.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.headings.is_empty() && self.parents.is_empty()
    }

    /// The group's declared KEY headings, in declaration order — `DICT_STAT`
    /// matched by the one shared predicate (`status_contains`), so
    /// `KEY+REQUIRED` counts. The file half of `EffectiveDict::key_fields`,
    /// exposed on its own for the keychain's effective-dictionary minting
    /// (#815), where only a group the standard registry does not know ever
    /// reads it.
    pub fn key_headings(&self, group: &str) -> impl Iterator<Item = &FileHeading> {
        self.headings(group)
            .iter()
            .filter(|h| status_contains(&h.status, "KEY"))
    }

    /// Every file-declared heading name, across all groups (duplicates
    /// across groups possible — collect into a set for membership).
    pub fn all_heading_names(&self) -> impl Iterator<Item = &str> {
        self.headings.values().flatten().map(|h| h.heading.as_str())
    }
}

/// The union view Rule 18 defines: standard dictionary first, file DICT
/// appended — a standard heading keeps its canonical slot and definition
/// even when the file re-declares it.
#[derive(Debug, Clone)]
pub struct EffectiveDict<'a> {
    std: Dictionary<'a>,
    file: FileDict,
}

impl<'a> EffectiveDict<'a> {
    /// The one-call form: read the file's DICT and layer it over `std`.
    #[must_use]
    pub fn build(parsed: &ParsedFile, std: Dictionary<'a>) -> Self {
        Self::new(std, FileDict::from_parsed(parsed))
    }

    /// Layer an already-collected overlay over `std`.
    #[must_use]
    pub fn new(std: Dictionary<'a>, file: FileDict) -> Self {
        EffectiveDict { std, file }
    }

    /// The standard side of the union.
    #[must_use]
    pub fn std(&self) -> Dictionary<'a> {
        self.std
    }

    /// The file side of the union.
    #[must_use]
    pub fn file(&self) -> &FileDict {
        &self.file
    }

    /// Effective heading order for `group`: the standard order, then
    /// file-declared extras in file order (Rule 9's "user-defined … at the
    /// end after the standard HEADINGs", Rule 18a's ordering).
    #[must_use]
    pub fn headings(&self, group: &str) -> Vec<&str> {
        let mut out: Vec<&str> = self.std.group_headings(group).to_vec();
        for h in self.file.headings(group) {
            if !out.contains(&h.heading.as_str()) {
                out.push(&h.heading);
            }
        }
        out
    }

    /// One heading's effective definition — standard first, file DICT else.
    #[must_use]
    pub fn heading(&self, group: &str, name: &str) -> Option<HeadingRef<'_>> {
        if let Some(e) = self.std.heading(group, name) {
            return Some(e);
        }
        self.file.heading(group, name).map(|h| HeadingRef {
            ags_type: &h.ags_type,
            unit: &h.unit,
            status: &h.status,
            desc: &h.desc,
        })
    }

    /// Every heading name defined anywhere in the union (Rule 19b's
    /// defined-anywhere membership). May repeat a name — collect into a set.
    pub fn all_heading_names(&self) -> impl Iterator<Item = &str> {
        self.std
            .all_heading_names()
            .chain(self.file.all_heading_names())
    }

    /// Shared walk behind [`Self::key_fields`]/[`Self::required_fields`]:
    /// headings of `group` whose status contains `want`, standard dict first
    /// then file-DICT extras, de-duplicated. Private on purpose — only the
    /// status side is uppercased, so `want` must already be an uppercase
    /// literal. The two wrappers pass exactly that; a public caller would
    /// sooner or later pass `"key"` and silently get nothing.
    fn fields_with_status(&self, group: &str, want: &str) -> Vec<String> {
        let mut out: Vec<String> = Vec::new();
        for h in self.std.group_headings(group).iter() {
            if let Some(e) = self.std.heading(group, h) {
                if e.status.to_ascii_uppercase().contains(want) {
                    out.push((*h).to_string());
                }
            }
        }
        for h in self.file.headings(group) {
            if status_contains(&h.status, want) && !out.contains(&h.heading) {
                out.push(h.heading.clone());
            }
        }
        out
    }

    /// The group's KEY fields, in effective-dictionary order.
    #[must_use]
    pub fn key_fields(&self, group: &str) -> Vec<String> {
        self.fields_with_status(group, "KEY")
    }

    /// The group's REQUIRED fields, in effective-dictionary order.
    #[must_use]
    pub fn required_fields(&self, group: &str) -> Vec<String> {
        self.fields_with_status(group, "REQUIRED")
    }

    /// `Some(parent)` (possibly `""` = parentless), or `None` if the group
    /// has no definition in either dictionary. The file side's conventional
    /// `"-"` (no parent) normalizes to `""`, as the bundled tables already
    /// do at build time.
    #[must_use]
    pub fn parent(&self, group: &str) -> Option<&str> {
        if let Some(m) = self.std.group(group) {
            return Some(m.parent);
        }
        self.file
            .parent(group)
            .map(|p| if p == "-" { "" } else { p })
    }

    /// `group` plus every group above it on the declared parent chain.
    ///
    /// Cycle-guarded, because half this chain is file-authored: a DICT that
    /// declares A's parent B and B's parent A is malformed but parses, and an
    /// unguarded walk would spin on it forever.
    #[must_use]
    pub fn ancestry(&self, group: &str) -> HashSet<String> {
        let mut seen: HashSet<String> = HashSet::new();
        let mut cur = group.to_string();
        while seen.insert(cur.clone()) {
            match self.parent(&cur) {
                Some(p) if !p.is_empty() => cur = p.to_string(),
                _ => break,
            }
        }
        seen
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dict::DictVersion;
    use laterite_ags4_parse::parse_str;

    fn file_dict(src: &str) -> FileDict {
        FileDict::from_parsed(&parse_str(src).expect("fixture parses"))
    }

    fn eff(src: &str) -> EffectiveDict<'static> {
        EffectiveDict::new(Dictionary::bundled(DictVersion::V4_2), file_dict(src))
    }

    /// A DICT declaring one custom group with all five columns + parentage.
    const FULL: &str = "\"GROUP\",\"DICT\"\r\n\
        \"HEADING\",\"DICT_TYPE\",\"DICT_GRP\",\"DICT_HDNG\",\"DICT_STAT\",\"DICT_DTYP\",\"DICT_UNIT\",\"DICT_PGRP\",\"DICT_DESC\"\r\n\
        \"UNIT\",\"\",\"\",\"\",\"\",\"\",\"\",\"\",\"\"\r\n\
        \"TYPE\",\"X\",\"X\",\"X\",\"X\",\"X\",\"X\",\"X\",\"X\"\r\n\
        \"DATA\",\"GROUP\",\"XXXX\",\"\",\"\",\"\",\"\",\"LOCA\",\"Custom group\"\r\n\
        \"DATA\",\"HEADING\",\"XXXX\",\"XXXX_ID\",\"KEY\",\"ID\",\"\",\"\",\"Ident\"\r\n\
        \"DATA\",\"HEADING\",\"XXXX\",\"XXXX_DPTH\",\"REQUIRED\",\"2DP\",\"m\",\"\",\"Depth\"\r\n";

    #[test]
    fn captures_all_five_columns_and_parent() {
        let d = file_dict(FULL);
        let hs = d.headings("XXXX");
        assert_eq!(hs.len(), 2);
        assert_eq!(
            hs[1],
            FileHeading {
                heading: "XXXX_DPTH".into(),
                status: "REQUIRED".into(),
                ags_type: "2DP".into(),
                unit: "m".into(),
                desc: "Depth".into(),
            }
        );
        assert_eq!(d.parent("XXXX"), Some("LOCA"));
        assert_eq!(d.groups().into_iter().collect::<Vec<_>>(), ["XXXX"]);
    }

    #[test]
    fn no_dict_group_yields_empty_overlay() {
        let d = file_dict(
            "\"GROUP\",\"PROJ\"\r\n\"HEADING\",\"PROJ_ID\"\r\n\
             \"UNIT\",\"\"\r\n\"TYPE\",\"ID\"\r\n\"DATA\",\"P1\"\r\n",
        );
        assert!(d.is_empty());
    }

    /// Rows with no group code or no heading contribute no heading; a
    /// heading declared twice for the same group keeps its FIRST declaration.
    #[test]
    fn skips_empty_and_first_declaration_wins() {
        let d = file_dict(
            "\"GROUP\",\"DICT\"\r\n\
             \"HEADING\",\"DICT_TYPE\",\"DICT_GRP\",\"DICT_HDNG\",\"DICT_STAT\"\r\n\
             \"UNIT\",\"\",\"\",\"\",\"\"\r\n\"TYPE\",\"X\",\"X\",\"X\",\"X\"\r\n\
             \"DATA\",\"GROUP\",\"XXXX\",\"\",\"\"\r\n\
             \"DATA\",\"HEADING\",\"XXXX\",\"XXXX_ONE\",\"KEY\"\r\n\
             \"DATA\",\"HEADING\",\"XXXX\",\"XXXX_ONE\",\"REQUIRED\"\r\n",
        );
        let hs = d.headings("XXXX");
        assert_eq!(hs.len(), 1);
        assert_eq!(hs[0].status, "KEY");
    }

    /// Membership is type-agnostic — python-ags4's Rule 7/9/10a/10b lookups
    /// filter on `DICT_GRP` alone — so a row with a blank or junk `DICT_TYPE`
    /// still declares its heading, and a DICT with no `DICT_TYPE` column
    /// still contributes headings (though it can declare no parentage).
    #[test]
    fn heading_membership_ignores_dict_type() {
        let d = file_dict(
            "\"GROUP\",\"DICT\"\r\n\
             \"HEADING\",\"DICT_GRP\",\"DICT_HDNG\"\r\n\
             \"UNIT\",\"\",\"\"\r\n\"TYPE\",\"X\",\"X\"\r\n\
             \"DATA\",\"XXXX\",\"XXXX_ONE\"\r\n",
        );
        assert_eq!(d.headings("XXXX").len(), 1);
        assert_eq!(d.parent("XXXX"), None);
    }

    /// Duplicate GROUP-type rows: the first declaration wins, matching
    /// python-ags4's first-row parent read.
    #[test]
    fn first_group_row_wins_for_parent() {
        let d = file_dict(
            "\"GROUP\",\"DICT\"\r\n\
             \"HEADING\",\"DICT_TYPE\",\"DICT_GRP\",\"DICT_PGRP\"\r\n\
             \"UNIT\",\"\",\"\",\"\"\r\n\"TYPE\",\"X\",\"X\",\"X\"\r\n\
             \"DATA\",\"GROUP\",\"XXXX\",\"LOCA\"\r\n\
             \"DATA\",\"GROUP\",\"XXXX\",\"SAMP\"\r\n",
        );
        assert_eq!(d.parent("XXXX"), Some("LOCA"));
    }

    #[test]
    fn effective_order_is_standard_then_file_extras() {
        // PROJ_XX is file-declared; the standard PROJ headings keep their
        // canonical slots and the extra appends after them.
        let e = eff("\"GROUP\",\"DICT\"\r\n\
             \"HEADING\",\"DICT_TYPE\",\"DICT_GRP\",\"DICT_HDNG\"\r\n\
             \"UNIT\",\"\",\"\",\"\"\r\n\"TYPE\",\"X\",\"X\",\"X\"\r\n\
             \"DATA\",\"HEADING\",\"PROJ\",\"PROJ_XX\"\r\n\
             \"DATA\",\"HEADING\",\"PROJ\",\"PROJ_ID\"\r\n");
        let hs = e.headings("PROJ");
        assert_eq!(hs.last(), Some(&"PROJ_XX"));
        assert_eq!(hs.first(), Some(&"PROJ_ID"), "std order first: {hs:?}");
        // The re-declared PROJ_ID must not appear twice.
        assert_eq!(hs.iter().filter(|h| **h == "PROJ_ID").count(), 1);
    }

    #[test]
    fn heading_lookup_prefers_standard_definition() {
        // The file re-declares LOCA_ID with a different type; the standard
        // definition wins (a standard heading keeps its canonical slot AND
        // its canonical definition).
        let e = eff("\"GROUP\",\"DICT\"\r\n\
             \"HEADING\",\"DICT_TYPE\",\"DICT_GRP\",\"DICT_HDNG\",\"DICT_DTYP\"\r\n\
             \"UNIT\",\"\",\"\",\"\",\"\"\r\n\"TYPE\",\"X\",\"X\",\"X\",\"X\"\r\n\
             \"DATA\",\"HEADING\",\"LOCA\",\"LOCA_ID\",\"2DP\"\r\n\
             \"DATA\",\"HEADING\",\"LOCA\",\"LOCA_XYZ\",\"3SF\"\r\n");
        assert_eq!(e.heading("LOCA", "LOCA_ID").expect("std").ags_type, "ID");
        let custom = e.heading("LOCA", "LOCA_XYZ").expect("file-declared");
        assert_eq!(custom.ags_type, "3SF");
        assert!(e.heading("LOCA", "LOCA_NOPE").is_none());
    }

    #[test]
    fn parent_falls_through_and_normalizes_dash() {
        let e = eff("\"GROUP\",\"DICT\"\r\n\
             \"HEADING\",\"DICT_TYPE\",\"DICT_GRP\",\"DICT_PGRP\"\r\n\
             \"UNIT\",\"\",\"\",\"\"\r\n\"TYPE\",\"X\",\"X\",\"X\"\r\n\
             \"DATA\",\"GROUP\",\"XXXX\",\"-\"\r\n\
             \"DATA\",\"GROUP\",\"YYYY\",\"XXXX\"\r\n");
        // Standard group: parent from the bundled tables.
        assert_eq!(e.parent("SAMP"), Some("LOCA"));
        // File-declared root: "-" normalizes to "" (declared, parentless).
        assert_eq!(e.parent("XXXX"), Some(""));
        assert_eq!(e.parent("YYYY"), Some("XXXX"));
        // Undeclared anywhere: None.
        assert_eq!(e.parent("ZZZZ"), None);
    }

    #[test]
    fn ancestry_walks_and_survives_a_cycle() {
        let e = eff("\"GROUP\",\"DICT\"\r\n\
             \"HEADING\",\"DICT_TYPE\",\"DICT_GRP\",\"DICT_PGRP\"\r\n\
             \"UNIT\",\"\",\"\",\"\"\r\n\"TYPE\",\"X\",\"X\",\"X\"\r\n\
             \"DATA\",\"GROUP\",\"XXXX\",\"LOCA\"\r\n\
             \"DATA\",\"GROUP\",\"AAAA\",\"BBBB\"\r\n\
             \"DATA\",\"GROUP\",\"BBBB\",\"AAAA\"\r\n");
        let chain = e.ancestry("XXXX");
        assert!(chain.contains("XXXX") && chain.contains("LOCA") && chain.contains("PROJ"));
        // A file-authored A↔B parent cycle terminates.
        let cycle = e.ancestry("AAAA");
        assert_eq!(cycle.len(), 2, "{cycle:?}");
    }

    #[test]
    fn all_heading_names_spans_both_sides() {
        let e = eff(FULL);
        let names: HashSet<&str> = e.all_heading_names().collect();
        assert!(names.contains("XXXX_DPTH"), "file-declared");
        assert!(names.contains("LOCA_ID"), "standard");
    }

    #[test]
    fn from_rows_matches_from_parsed() {
        let via_rows = FileDict::from_rows([
            DictRow {
                dict_type: "GROUP",
                group: "XXXX",
                heading: "",
                status: "",
                ags_type: "",
                unit: "",
                parent: "LOCA",
                desc: "Custom group",
            },
            DictRow {
                dict_type: "HEADING",
                group: "XXXX",
                heading: "XXXX_ID",
                status: "KEY",
                ags_type: "ID",
                unit: "",
                parent: "",
                desc: "Ident",
            },
            DictRow {
                dict_type: "HEADING",
                group: "XXXX",
                heading: "XXXX_DPTH",
                status: "REQUIRED",
                ags_type: "2DP",
                unit: "m",
                parent: "",
                desc: "Depth",
            },
        ]);
        assert_eq!(via_rows, file_dict(FULL));
    }
}
