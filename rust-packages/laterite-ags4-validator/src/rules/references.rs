//! Cross-reference rules: AGS4.1/4.2 Rule 19b (dict-aware borrowed-
//! heading parts `19b_2/19b_3`) and Rule 20 (FILE / `FILE_FSET`).
//!
//! CLEAN-ROOM. Implemented from the AGS4 spec (`reports/AGS 4_1.pdf` &
//! `reports/AGS 4_2.pdf` §4.1.1). python-ags4 (LGPL-3.0) was read only
//! to learn its interpretation (the SPEC/TEST exceptions, the FILE
//! folder semantics) — facts about the AGS standard, not
//! copyrightable. No code, structure, or wording was copied.
//!
//! Spec text (verbatim, AGS 4.2 §4.1.1, p.157 — AGS 4.1 identical):
//!
//! * **Rule 19b** — "HEADING names shall start with the GROUP name
//!   followed by an underscore character. e.g. '`NGRP_HED1`'. Where a
//!   HEADING refers to an existing HEADING within another GROUP, the
//!   HEADING name added to the group shall bear the same name. e.g.
//!   '`CMPG_TESN`' in the 'CMPT' GROUP." (V3 covered the structural
//!   part `19b_1`; here `19b_2/19b_3` add the cross-group borrow check
//!   that needs the effective dictionary.)
//! * **Rule 20** — "Additional computer files (e.g. digital images)
//!   can be included within a data submission. Each such file shall be
//!   defined in a FILE GROUP. The additional files shall be
//!   transferred in a sub-folder named FILE. This FILE sub-folder
//!   shall contain additional sub-folders each named by the `FILE_FSET`
//!   reference. Each `FILE_FSET` named folder will contain the files
//!   listed in the FILE GROUP."
//!
//! Scope:
//! * `19b_2/19b_3` act only on a heading whose prefix names *another*
//!   group (the borrowed-heading case). A missing-underscore or
//!   structurally-bad heading is already a V3 Rule 19b finding and a
//!   V4 Rule 9 finding — we do not re-report it under Rule 19b here,
//!   unlike python-ags4 which adds it again from `19b_2` *and* `19b_3`
//!   (O-26). The `SPEC`/`TEST` prefixes are dictionary-sanctioned
//!   exceptions, skipped exactly as python-ags4 does.
//! * Rule 20 implements the **data-level** check (every used
//!   `FILE_FSET` defined in the FILE group; FILE group present when
//!   `FILE_FSET` is used). The on-disk `FILE/<fset>/<name>` existence
//!   checks python-ags4 performs are deliberately **not** done — a
//!   library validator must be deterministic and path-independent,
//!   and `db-to-ags4 --validate` checks an emitted file with no
//!   sidecar folder. Documented as a scoped variance (O-27).

use std::collections::HashSet;

use crate::dict::Dictionary;
use crate::effective_dict::EffectiveDict;
use crate::findings::{Findings, Location, Severity, Target, add, add_at};
use crate::parse::ParsedFile;

const RULE_19B: &str = "AGS Format Rule 19b";
/// Shared with [`crate::world`], which owns Rule 20's on-disk half — the two
/// halves report under the same rule key, but only one of them reads the world.
pub(crate) const RULE_20: &str = "AGS Format Rule 20";

/// Dictionary-sanctioned prefix exceptions (the standard dictionary
/// itself ships `SPEC_*`/`TEST_*` headings borrowed into other groups
/// without a matching GROUP). python-ags4 hardcodes the same pair.
const PREFIX_EXCEPTIONS: &[&str] = &["SPEC", "TEST"];

pub fn check(parsed: &ParsedFile, dict: &Dictionary, found: &mut Findings) {
    // The Rule 18 union, read by the shared `effective_dict` module (#777).
    let eff = EffectiveDict::build(parsed, *dict);

    // Headings defined under each group = standard dict ∪ file DICT.
    let merged = |group: &str| -> HashSet<String> {
        eff.headings(group)
            .iter()
            .map(|h| (*h).to_string())
            .collect()
    };

    // Every heading name defined anywhere (Rule 19b_3 global check).
    let defined_anywhere: HashSet<&str> = eff.all_heading_names().collect();

    for code in &parsed.group_order {
        let g = &parsed.groups[code];
        let Some(hl) = g.heading_line else { continue };

        // `ci` is a heading's column index within one AGS4 group — bounded
        // by that group's heading count (dictionary-bounded), nowhere near
        // u32::MAX.
        #[allow(clippy::cast_possible_truncation)]
        for (ci, h) in g.headings.iter().enumerate() {
            // The borrowed-heading rule only has something to say when
            // the prefix names a *different* group. No underscore / bad
            // shape is V3 Rule 19b + V4 Rule 9 — not re-reported here.
            let Some((prefix, _)) = h.split_once('_') else {
                continue;
            };
            if prefix == code || PREFIX_EXCEPTIONS.contains(&prefix) {
                continue;
            }
            let loc = || Location {
                target: Target::Heading,
                field_index: Some(ci as u32),
                heading: Some(h.clone()),
                ..Default::default()
            };

            let ref1 = merged(prefix);
            if ref1.is_empty() {
                // python-ags4's rule_19b_2: the prefix names a group
                // that's not defined anywhere. Wording byte-faithful
                // so compat doesn't need a translator entry.
                add_at(
                    found,
                    RULE_19B,
                    Some(hl),
                    code,
                    format!(
                        "Group {prefix} referred to in {h} could not be \
                         found in either the standard dictionary or the \
                         DICT group."
                    ),
                    loc(),
                    Severity::Error,
                );
                // Stage 9c: half-revert O-26's consolidation. python-ags4's
                // rule_19b_3 ALSO fires here when the heading itself
                // isn't defined anywhere (truly orphaned — not even a
                // legitimate cross-group borrow). The two messages
                // target different fixes — "Group X not found"
                // (rule_19b_2) hints at a prefix typo; "X doesn't
                // start with this group's name" (rule_19b_3) hints at
                // a placement mistake. Both readings useful for a
                // diagnostic report; O-26's consolidation lost the
                // second angle.
                if !defined_anywhere.contains(h.as_str()) {
                    add_at(
                        found,
                        RULE_19B,
                        Some(hl),
                        code,
                        format!(
                            "{h} does not start with the name of this \
                             group, nor is it defined in another group."
                        ),
                        loc(),
                        Severity::Error,
                    );
                }
            } else if !ref1.contains(h) {
                if merged(code).contains(h) {
                    add_at(
                        found,
                        RULE_19B,
                        Some(hl),
                        code,
                        format!(
                            "Heading {h:?} is defined under group {code} but its prefix \
                             names group {prefix}; rename it or define it under {prefix}."
                        ),
                        loc(),
                        Severity::Error,
                    );
                } else if !defined_anywhere.contains(h.as_str()) {
                    add_at(
                        found,
                        RULE_19B,
                        Some(hl),
                        code,
                        format!(
                            "Heading {h:?} neither starts with group {code} nor is \
                             defined in another group."
                        ),
                        loc(),
                        Severity::Error,
                    );
                }
                // else: defined under some third group → Rule 9's call.
            }
            // else: a valid borrow (defined under `prefix`) → fine.
        }
    }

    // Rule 20, CONTENT half only: every FILE_FSET used is defined in the FILE
    // group. Rule 20's other half stats the sibling FILE/ tree on disk — that is
    // not a function of the AGS4 bytes, so it lives in `crate::world` and is run
    // from the one door in lib.rs, never from inside the rule engine.
    rule_20(parsed, found);
}

/// Rule 20 (data level) — every `FILE_FSET` used must be defined in the
/// FILE group; the FILE group must exist when `FILE_FSET` is used.
fn rule_20(parsed: &ParsedFile, found: &mut Findings) {
    // (group, fset value, line, field_index, data_row) for every
    // alphanumeric FILE_FSET used. `ci` is the tag-stripped column index
    // of FILE_FSET; the data row's 1-based ordinal lets the UI tint the
    // exact cell.
    let mut used: Vec<(&str, &str, u32, u32, u32)> = Vec::new();
    for code in &parsed.group_order {
        let g = &parsed.groups[code];
        let Some(ci) = g.headings.iter().position(|h| h == "FILE_FSET") else {
            continue;
        };
        // `ci`/`ri` are a column/row index within one AGS4 group — bounded
        // by that group's heading count and its actual row count, both far
        // below u32::MAX for any real AGS4 file (see `laterite-ags4-core`'s
        // byte-offset casts for the same reasoning on file-scale bounds).
        #[allow(clippy::cast_possible_truncation)]
        for (ri, row) in g.rows.iter().enumerate() {
            if let Some(v) = row.values.get(ci).map(|s| s.slice(g.text())) {
                if v.chars().any(|c| c.is_ascii_alphanumeric()) {
                    used.push((code, v, row.line, ci as u32, ri as u32 + 1));
                }
            }
        }
    }
    if used.is_empty() {
        return; // no attachments referenced → FILE group not required
    }

    let Some(file_g) = parsed.groups.get("FILE") else {
        add(
            found,
            RULE_20,
            None,
            "FILE",
            "FILE group not found, but FILE_FSET entries are used elsewhere.".to_string(),
        );
        return;
    };

    let defined: HashSet<&str> = file_g
        .headings
        .iter()
        .position(|h| h == "FILE_FSET")
        .map(|ci| {
            file_g
                .rows
                .iter()
                .filter_map(|r| r.values.get(ci).map(|s| s.slice(file_g.text())))
                .collect()
        })
        .unwrap_or_default();

    for (group, fset, line, field_index, data_row) in used {
        if !defined.contains(fset) {
            add_at(
                found,
                RULE_20,
                Some(line),
                group,
                format!("FILE_FSET {fset:?} is not defined in the FILE group."),
                Location {
                    target: Target::Cell,
                    field_index: Some(field_index),
                    heading: Some("FILE_FSET".to_string()),
                    data_row: Some(data_row),
                    ..Default::default()
                },
                Severity::Error,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dict::DictVersion;
    use crate::parse::parse_str;

    fn run(src: &str) -> Findings {
        let pf = parse_str(src).expect("fixture parses");
        let d = Dictionary::bundled(DictVersion::V4_2);
        let mut f = Findings::new();
        // Content only — Rule 20's on-disk half now lives in `crate::world`.
        check(&pf, &d, &mut f);
        f
    }

    #[test]
    fn valid_borrowed_heading_is_accepted() {
        // SAMP legitimately borrows LOCA_ID (LOCA_ID is defined under
        // LOCA in the standard dictionary).
        let src = "\"GROUP\",\"SAMP\"\r\n\
                   \"HEADING\",\"LOCA_ID\",\"SAMP_ID\"\r\n\
                   \"UNIT\",\"\",\"\"\r\n\"TYPE\",\"ID\",\"ID\"\r\n\
                   \"DATA\",\"BH1\",\"S1\"\r\n";
        assert!(!run(src).contains_key(RULE_19B), "{:?}", run(src));
    }

    #[test]
    fn unknown_prefix_group_flagged() {
        // ZZZZ_FOO in SAMP — ZZZZ is not a defined group.
        let src = "\"GROUP\",\"SAMP\"\r\n\
                   \"HEADING\",\"SAMP_ID\",\"ZZZZ_FOO\"\r\n\
                   \"UNIT\",\"\",\"\"\r\n\"TYPE\",\"ID\",\"X\"\r\n\
                   \"DATA\",\"S1\",\"x\"\r\n";
        let r = run(src);
        let v = r.get(RULE_19B).expect("Rule 19b");
        assert!(v.iter().any(|x| x.desc.contains("ZZZZ")), "{v:?}");
    }

    #[test]
    fn spec_and_test_prefixes_are_exempt() {
        // SPEC_REF inside SAMP — dictionary-sanctioned, must not flag.
        let src = "\"GROUP\",\"SAMP\"\r\n\
                   \"HEADING\",\"SAMP_ID\",\"SPEC_REF\",\"TEST_STAT\"\r\n\
                   \"UNIT\",\"\",\"\",\"\"\r\n\"TYPE\",\"ID\",\"X\",\"X\"\r\n\
                   \"DATA\",\"S1\",\"R1\",\"OK\"\r\n";
        assert!(!run(src).contains_key(RULE_19B), "{:?}", run(src));
    }

    #[test]
    fn rule_20_flags_undefined_and_missing_file_group() {
        // FILE_FSET used in LOCA, no FILE group.
        let no_file = "\"GROUP\",\"LOCA\"\r\n\"HEADING\",\"LOCA_ID\",\"FILE_FSET\"\r\n\
                       \"UNIT\",\"\",\"\"\r\n\"TYPE\",\"ID\",\"X\"\r\n\
                       \"DATA\",\"BH1\",\"FS1\"\r\n";
        assert!(
            run(no_file)
                .get(RULE_20)
                .is_some_and(|v| v[0].group == "FILE")
        );

        // FILE group present but doesn't define FS9.
        let bad = "\"GROUP\",\"LOCA\"\r\n\"HEADING\",\"LOCA_ID\",\"FILE_FSET\"\r\n\
                   \"UNIT\",\"\",\"\"\r\n\"TYPE\",\"ID\",\"X\"\r\n\
                   \"DATA\",\"BH1\",\"FS9\"\r\n\r\n\
                   \"GROUP\",\"FILE\"\r\n\
                   \"HEADING\",\"FILE_FSET\",\"FILE_NAME\"\r\n\
                   \"UNIT\",\"\",\"\"\r\n\"TYPE\",\"X\",\"X\"\r\n\
                   \"DATA\",\"FS1\",\"photo.jpg\"\r\n";
        let r = run(bad);
        let r20 = r.get(RULE_20).expect("Rule 20");
        assert!(
            r20.iter()
                .any(|x| x.desc.contains("FS9") && x.group == "LOCA"),
            "{r20:?}"
        );
    }

    #[test]
    fn rule_20_clean_when_fset_defined() {
        let ok = "\"GROUP\",\"LOCA\"\r\n\"HEADING\",\"LOCA_ID\",\"FILE_FSET\"\r\n\
                  \"UNIT\",\"\",\"\"\r\n\"TYPE\",\"ID\",\"X\"\r\n\
                  \"DATA\",\"BH1\",\"FS1\"\r\n\r\n\
                  \"GROUP\",\"FILE\"\r\n\
                  \"HEADING\",\"FILE_FSET\",\"FILE_NAME\"\r\n\
                  \"UNIT\",\"\",\"\"\r\n\"TYPE\",\"X\",\"X\"\r\n\
                  \"DATA\",\"FS1\",\"photo.jpg\"\r\n";
        assert!(!run(ok).contains_key(RULE_20), "{:?}", run(ok));
    }

    /// Reference findings must carry their Location: Rule 19b (unknown prefix)
    /// targets the HEADING; Rule 20 (undefined `FILE_FSET`) targets the CELL with
    /// its column, heading name, and 1-based row.
    #[test]
    fn reference_findings_carry_locations() {
        let r = run(
            "\"GROUP\",\"SAMP\"\r\n\"HEADING\",\"SAMP_ID\",\"ZZZZ_FOO\"\r\n\
                     \"UNIT\",\"\",\"\"\r\n\"TYPE\",\"ID\",\"X\"\r\n\"DATA\",\"S1\",\"x\"\r\n",
        );
        let f19b = r
            .get(RULE_19B)
            .unwrap()
            .iter()
            .find(|x| x.desc.contains("could not be"))
            .expect("19b_2 fired");
        assert_eq!(f19b.location.target, Target::Heading);
        assert_eq!(f19b.location.field_index, Some(1));
        assert_eq!(f19b.location.heading.as_deref(), Some("ZZZZ_FOO"));

        let bad = run(
            "\"GROUP\",\"LOCA\"\r\n\"HEADING\",\"LOCA_ID\",\"FILE_FSET\"\r\n\
                       \"UNIT\",\"\",\"\"\r\n\"TYPE\",\"ID\",\"X\"\r\n\
                       \"DATA\",\"BH1\",\"FS9\"\r\n\r\n\
                       \"GROUP\",\"FILE\"\r\n\"HEADING\",\"FILE_FSET\",\"FILE_NAME\"\r\n\
                       \"UNIT\",\"\",\"\"\r\n\"TYPE\",\"X\",\"X\"\r\n\
                       \"DATA\",\"FS1\",\"photo.jpg\"\r\n",
        );
        let f20 = &bad.get(RULE_20).expect("rule 20")[0];
        assert_eq!(f20.location.target, Target::Cell);
        assert_eq!(f20.location.field_index, Some(1));
        assert_eq!(f20.location.heading.as_deref(), Some("FILE_FSET"));
        assert_eq!(f20.location.data_row, Some(1));
    }

    #[test]
    fn unknown_prefix_also_emits_19b_3_when_heading_orphaned() {
        // ZZZZ_FOO: prefix ZZZZ names no group (19b_2) AND ZZZZ_FOO is
        // not defined anywhere (19b_3) → BOTH messages fire (the Stage-9c
        // half-revert of O-26). Distinct fix hints: prefix-typo vs
        // placement-mistake.
        let src = "\"GROUP\",\"SAMP\"\r\n\
                   \"HEADING\",\"SAMP_ID\",\"ZZZZ_FOO\"\r\n\
                   \"UNIT\",\"\",\"\"\r\n\"TYPE\",\"ID\",\"X\"\r\n\
                   \"DATA\",\"S1\",\"x\"\r\n";
        let r = run(src);
        let v = r.get(RULE_19B).expect("Rule 19b");
        assert!(
            v.iter().any(|x| x.desc.contains("could not be")),
            "19b_2 (group-not-found) message: {v:?}"
        );
        assert!(
            v.iter().any(|x| x
                .desc
                .contains("does not start with the name of this group")),
            "19b_3 (orphaned-heading) message: {v:?}"
        );
    }

    #[test]
    fn heading_defined_under_own_group_but_prefixed_with_another() {
        // SAMP declares (via its own DICT) a heading "LOCA_FOO" whose
        // prefix LOCA names another group. LOCA exists (so ref1 is
        // non-empty) but doesn't contain LOCA_FOO; LOCA_FOO IS defined
        // under SAMP via the file DICT → the "defined under group SAMP
        // but its prefix names group LOCA" arm (lines 152-165).
        let src = "\"GROUP\",\"DICT\"\r\n\
                   \"HEADING\",\"DICT_TYPE\",\"DICT_GRP\",\"DICT_HDNG\",\"DICT_STAT\"\r\n\
                   \"UNIT\",\"\",\"\",\"\",\"\"\r\n\
                   \"TYPE\",\"X\",\"X\",\"X\",\"X\"\r\n\
                   \"DATA\",\"HEADING\",\"SAMP\",\"LOCA_FOO\",\"OTHER\"\r\n\r\n\
                   \"GROUP\",\"SAMP\"\r\n\
                   \"HEADING\",\"SAMP_ID\",\"LOCA_FOO\"\r\n\
                   \"UNIT\",\"\",\"\"\r\n\"TYPE\",\"ID\",\"X\"\r\n\
                   \"DATA\",\"S1\",\"x\"\r\n";
        let r = run(src);
        let v = r.get(RULE_19B).expect("Rule 19b");
        assert!(
            v.iter()
                .any(|x| x.desc.contains("LOCA_FOO") && x.desc.contains("rename it")),
            "expected the defined-here-wrong-prefix message: {v:?}"
        );
    }

    #[test]
    fn heading_with_known_prefix_group_but_undefined_everywhere() {
        // SAMP has heading "LOCA_NOPENOPE": LOCA exists (ref1 non-empty)
        // but doesn't contain it, it's NOT defined under SAMP, and it's
        // defined nowhere → the "neither starts with group SAMP nor
        // defined in another group" arm (lines 166-178).
        let src = "\"GROUP\",\"SAMP\"\r\n\
                   \"HEADING\",\"SAMP_ID\",\"LOCA_NOPENOPE\"\r\n\
                   \"UNIT\",\"\",\"\"\r\n\"TYPE\",\"ID\",\"X\"\r\n\
                   \"DATA\",\"S1\",\"x\"\r\n";
        let r = run(src);
        let v = r.get(RULE_19B).expect("Rule 19b");
        assert!(
            v.iter()
                .any(|x| x.desc.contains("LOCA_NOPENOPE") && x.desc.contains("neither starts")),
            "expected the neither-starts-nor-defined message: {v:?}"
        );
    }

    #[test]
    fn heading_with_no_underscore_is_not_a_19b_borrow_finding() {
        // A heading lacking an underscore ("BADHDNG") has no prefix to
        // borrow from → the `split_once('_')` None continue (line 97-98).
        // V3 Rule 19b / V4 Rule 9 own that defect, not this module.
        let src = "\"GROUP\",\"SAMP\"\r\n\
                   \"HEADING\",\"SAMP_ID\",\"BADHDNG\"\r\n\
                   \"UNIT\",\"\",\"\"\r\n\"TYPE\",\"ID\",\"X\"\r\n\
                   \"DATA\",\"S1\",\"x\"\r\n";
        // No 19b borrow finding mentioning BADHDNG.
        let r = run(src);
        assert!(
            r.get(RULE_19B)
                .is_none_or(|v| !v.iter().any(|x| x.desc.contains("BADHDNG"))),
            "underscore-less heading must not be a 19b borrow finding: {:?}",
            r.get(RULE_19B)
        );
    }
}
