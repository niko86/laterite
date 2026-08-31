//! Runtime `.ags` DICT-group reader for the `--dict` override (laterite-dev#568 Phase 2).
//!
//! The bundled dictionaries are compiled from `ags_dictionary.json` at build
//! time; this is the FIRST place we read an AGS4 DICT group at *runtime*, so a
//! client can hand a bespoke dictionary as a plain `.ags` file. It reuses the
//! shared `laterite-ags4-parse` tokenizer (no second parser) and reconstructs
//! the same [`DictionaryFile`] shape the JSON path deserialises into — so both
//! `--dict` formats converge on one internal representation before
//! `detect_base` / `build_delta` ever see them.
//!
//! Per-heading semantic checks (status/type tokens) run inline here *with line
//! numbers*, because the format we pitch for hand-authoring is this one; the
//! JSON path re-checks the same content without lines (serde owns its syntax
//! errors). Parent existence is NOT checked here — it needs the base edition,
//! so `build_delta` owns it.

use std::collections::HashMap;

use laterite_ags4_parse::{InvalidUtf8, ParseError, ParseOptions, parse_bytes_opts};

use crate::overlay::{DictError, valid_status, valid_type};
use crate::union::{DictGroup, DictHeading, DictionaryFile};

/// Read a custom dictionary from AGS4 bytes: parse, locate the DICT group, and
/// fold its `GROUP` / `HEADING` rows into a [`DictionaryFile`].
pub(crate) fn read_ags_dict(
    bytes: &[u8],
    enc: &'static encoding_rs::Encoding,
) -> Result<DictionaryFile, DictError> {
    let opts = ParseOptions {
        encoding: enc,
        on_invalid_utf8: InvalidUtf8::LossyReplace,
        strict_structure: false,
        // Needs the DICT group's HEADING/DATA rows, not just its location.
        locate_only: false,
    };
    let parsed = parse_bytes_opts(bytes, opts).map_err(|e| match e {
        // No GROUP rows at all → this isn't a dictionary we can read.
        ParseError::NotAgs4(_) => DictError::NotADictionary,
        other => DictError::Parse(format!("{other:?}")),
    })?;

    let dict = parsed.groups.get("DICT").ok_or(DictError::NotADictionary)?;

    // Column indices. DICT_GRP is mandatory (every row is keyed by its group);
    // the rest are optional — a minimal DICT may omit UNIT/DESC/PGRP.
    let ci_grp = dict.col("DICT_GRP").ok_or(DictError::NotADictionary)?;
    let ci_type = dict.col("DICT_TYPE");
    let ci_hdng = dict.col("DICT_HDNG");
    let ci_stat = dict.col("DICT_STAT");
    let ci_dtyp = dict.col("DICT_DTYP");
    let ci_unit = dict.col("DICT_UNIT");
    let ci_desc = dict.col("DICT_DESC");
    let ci_pgrp = dict.col("DICT_PGRP");

    let mut groups: HashMap<String, DictGroup> = HashMap::new();

    for row in &dict.rows {
        let cell = |ci: Option<usize>| -> &str {
            ci.and_then(|i| row.values.get(i))
                .map_or("", |s| s.slice(dict.text()).trim())
        };
        let grp = cell(Some(ci_grp));
        if grp.is_empty() {
            continue; // a row with no group code carries nothing we can place
        }
        match cell(ci_type) {
            "GROUP" => {
                let entry = groups
                    .entry(grp.to_string())
                    .or_insert_with(DictGroup::empty);
                let parent = cell(ci_pgrp);
                entry.parent = (!parent.is_empty()).then(|| parent.to_string());
                let desc = cell(ci_desc);
                if !desc.is_empty() {
                    entry.description = Some(desc.to_string());
                }
            }
            "HEADING" => {
                let name = cell(ci_hdng);
                if name.is_empty() {
                    return Err(DictError::BadGroupRowArity { line: row.line });
                }
                let status = cell(ci_stat);
                if !valid_status(status) {
                    return Err(DictError::UnknownStatus {
                        line: row.line,
                        token: status.to_string(),
                    });
                }
                let ags_type = cell(ci_dtyp);
                if !valid_type(ags_type) {
                    return Err(DictError::UnknownType {
                        line: row.line,
                        token: ags_type.to_string(),
                    });
                }
                let unit = cell(ci_unit);
                let entry = groups
                    .entry(grp.to_string())
                    .or_insert_with(DictGroup::empty);
                if entry.headings.iter().any(|h| h.name == name) {
                    return Err(DictError::DuplicateHeading {
                        group: grp.to_string(),
                        heading: name.to_string(),
                    });
                }
                entry.headings.push(DictHeading {
                    name: name.to_string(),
                    status: status.to_string(),
                    ags_type: ags_type.to_string(),
                    unit: (!unit.is_empty()).then(|| unit.to_string()),
                    description: cell(ci_desc).to_string(),
                });
            }
            // Any other DICT_TYPE (blank, UNIT-only rows, …) contributes nothing.
            _ => {}
        }
    }

    if groups.is_empty() {
        return Err(DictError::Empty);
    }
    Ok(DictionaryFile::from_groups(groups))
}
