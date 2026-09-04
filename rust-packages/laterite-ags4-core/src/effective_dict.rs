//! The Rule 18 effective dictionary — re-exported from the
//! `laterite-ags4-reference` leaf, plus the adapter for this crate's own
//! read codec (#777).
//!
//! The implementation lives in the reference leaf beside the standard
//! dictionary it unions with (same move as [`crate::registry`]); this module
//! preserves the "one engine crate" contract for read-only consumers — a
//! reader that depends on `laterite-ags4-core` alone can build the union a
//! file-declared group needs to bind its columns, without pulling the rule
//! engine. [`Dictionary`] and [`DictVersion`] ride along so the standard
//! side is constructible from here too.
//!
//! Two routes in, one semantic: [`FileDict::from_parsed`] for the parse
//! leaf's [`laterite_ags4_parse::ParsedFile`], and [`file_dict_of`] for this
//! crate's [`ParsedAgs4`]. The codec trims cell whitespace where the parse
//! leaf keeps values raw — each route is faithful to what its parse holds.

pub use laterite_ags4_reference::dict::{DictVersion, Dictionary};
pub use laterite_ags4_reference::effective_dict::*;

use crate::ags4_codec::ParsedAgs4;

/// The file's DICT overlay, read out of the codec's parse (`read_ags4*`).
/// Tolerant like every route: no DICT group, or malformed content, yields an
/// empty overlay — core reads DICT, it never validates it.
#[must_use]
pub fn file_dict_of(parsed: &ParsedAgs4) -> FileDict {
    let Some(d) = parsed.get("DICT") else {
        return FileDict::default();
    };
    // Name→column resolves once per group, positional after that — the
    // accessor contract's pattern, not a per-row name scan.
    let [typ, grp, hdng, stat, dtyp, unit, pgrp, desc] = [
        "DICT_TYPE",
        "DICT_GRP",
        "DICT_HDNG",
        "DICT_STAT",
        "DICT_DTYP",
        "DICT_UNIT",
        "DICT_PGRP",
        "DICT_DESC",
    ]
    .map(|n| d.col(n));
    FileDict::from_rows((0..d.n_rows()).map(|i| {
        let get = |col: Option<usize>| col.and_then(|col| d.cell(i, col)).unwrap_or("");
        DictRow {
            dict_type: get(typ),
            group: get(grp),
            heading: get(hdng),
            status: get(stat),
            ags_type: get(dtyp),
            unit: get(unit),
            parent: get(pgrp),
            desc: get(desc),
        }
    }))
}

/// The union (standard ∪ the file's own DICT group) for a codec parse — the
/// one-call form of [`EffectiveDict::new`] over [`file_dict_of`].
#[must_use]
pub fn effective_dict_of<'a>(parsed: &ParsedAgs4, std: Dictionary<'a>) -> EffectiveDict<'a> {
    EffectiveDict::new(std, file_dict_of(parsed))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ags4_codec::read_ags4_bytes;
    use laterite_ags4_parse::parse_str;

    const SRC: &str = "\"GROUP\",\"PROJ\"\r\n\
        \"HEADING\",\"PROJ_ID\"\r\n\"UNIT\",\"\"\r\n\"TYPE\",\"ID\"\r\n\
        \"DATA\",\"P1\"\r\n\r\n\
        \"GROUP\",\"DICT\"\r\n\
        \"HEADING\",\"DICT_TYPE\",\"DICT_GRP\",\"DICT_HDNG\",\"DICT_STAT\",\"DICT_DTYP\",\"DICT_UNIT\",\"DICT_PGRP\"\r\n\
        \"UNIT\",\"\",\"\",\"\",\"\",\"\",\"\",\"\"\r\n\
        \"TYPE\",\"X\",\"X\",\"X\",\"X\",\"X\",\"X\",\"X\"\r\n\
        \"DATA\",\"GROUP\",\"XXXX\",\"\",\"\",\"\",\"\",\"LOCA\"\r\n\
        \"DATA\",\"HEADING\",\"XXXX\",\"XXXX_DPTH\",\"KEY\",\"2DP\",\"m\",\"\"\r\n";

    /// Both parse routes must yield the same overlay for the same bytes.
    #[test]
    fn codec_route_matches_parse_leaf_route() {
        let via_codec = file_dict_of(&read_ags4_bytes(SRC.as_bytes()).expect("codec reads"));
        let via_leaf = FileDict::from_parsed(&parse_str(SRC).expect("leaf parses"));
        assert_eq!(via_codec, via_leaf);
        assert!(!via_codec.is_empty());
    }

    /// The union answers the question a read-only consumer asks: a
    /// file-declared group's column types, statuses, units and parent.
    #[test]
    fn union_binds_a_file_declared_group() {
        let parsed = read_ags4_bytes(SRC.as_bytes()).expect("codec reads");
        let eff = effective_dict_of(&parsed, Dictionary::bundled(DictVersion::V4_2));
        let h = eff.heading("XXXX", "XXXX_DPTH").expect("file-declared");
        assert_eq!((h.ags_type, h.unit, h.status), ("2DP", "m", "KEY"));
        assert_eq!(eff.parent("XXXX"), Some("LOCA"));
        // The standard side answers through the same union.
        assert_eq!(eff.heading("PROJ", "PROJ_ID").expect("std").ags_type, "ID");
    }
}
