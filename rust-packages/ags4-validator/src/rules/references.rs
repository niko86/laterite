//! Cross-reference rules: AGS4.1/4.2 Rule 19b (dict-aware borrowed-
//! heading parts 19b_2/19b_3) and Rule 20 (FILE / FILE_FSET).
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
//!   followed by an underscore character. e.g. 'NGRP_HED1'. Where a
//!   HEADING refers to an existing HEADING within another GROUP, the
//!   HEADING name added to the group shall bear the same name. e.g.
//!   'CMPG_TESN' in the 'CMPT' GROUP." (V3 covered the structural
//!   part 19b_1; here 19b_2/19b_3 add the cross-group borrow check
//!   that needs the effective dictionary.)
//! * **Rule 20** — "Additional computer files (e.g. digital images)
//!   can be included within a data submission. Each such file shall be
//!   defined in a FILE GROUP. The additional files shall be
//!   transferred in a sub-folder named FILE. This FILE sub-folder
//!   shall contain additional sub-folders each named by the FILE_FSET
//!   reference. Each FILE_FSET named folder will contain the files
//!   listed in the FILE GROUP."
//!
//! Scope:
//! * 19b_2/19b_3 act only on a heading whose prefix names *another*
//!   group (the borrowed-heading case). A missing-underscore or
//!   structurally-bad heading is already a V3 Rule 19b finding and a
//!   V4 Rule 9 finding — we do not re-report it under Rule 19b here,
//!   unlike python-ags4 which adds it again from 19b_2 *and* 19b_3
//!   (O-26). The `SPEC`/`TEST` prefixes are dictionary-sanctioned
//!   exceptions, skipped exactly as python-ags4 does.
//! * Rule 20 implements the **data-level** check (every used
//!   FILE_FSET defined in the FILE group; FILE group present when
//!   FILE_FSET is used). The on-disk `FILE/<fset>/<name>` existence
//!   checks python-ags4 performs are deliberately **not** done — a
//!   library validator must be deterministic and path-independent,
//!   and `db-to-ags4 --validate` checks an emitted file with no
//!   sidecar folder. Documented as a scoped variance (O-27).

use std::collections::HashSet;
use std::path::Path;

use crate::dict::Dictionary;
use crate::findings::{Findings, add};
use crate::parse::ParsedFile;
use crate::rules::dictionary::collect_file_dict;

const RULE_19B: &str = "AGS Format Rule 19b";
const RULE_20: &str = "AGS Format Rule 20";

/// Dictionary-sanctioned prefix exceptions (the standard dictionary
/// itself ships `SPEC_*`/`TEST_*` headings borrowed into other groups
/// without a matching GROUP). python-ags4 hardcodes the same pair.
const PREFIX_EXCEPTIONS: &[&str] = &["SPEC", "TEST"];

pub fn check(
    parsed: &ParsedFile,
    dict: &Dictionary,
    source: Option<&Path>,
    check_files: bool,
    found: &mut Findings,
) {
    let file_dict = collect_file_dict(parsed);

    // Headings defined under each group = standard dict ∪ file DICT.
    let merged = |group: &str| -> HashSet<String> {
        let mut s: HashSet<String> = dict
            .group_headings(group)
            .iter()
            .map(|h| (*h).to_string())
            .collect();
        if let Some(extra) = file_dict.get(group) {
            s.extend(extra.iter().cloned());
        }
        s
    };

    // Every heading name defined anywhere (Rule 19b_3 global check).
    let mut defined_anywhere: HashSet<&str> = dict.all_heading_names().collect();
    for hs in file_dict.values() {
        for h in hs {
            defined_anywhere.insert(h.as_str());
        }
    }

    for code in &parsed.group_order {
        let g = &parsed.groups[code];
        let Some(hl) = g.heading_line else { continue };

        for h in &g.headings {
            // The borrowed-heading rule only has something to say when
            // the prefix names a *different* group. No underscore / bad
            // shape is V3 Rule 19b + V4 Rule 9 — not re-reported here.
            let Some((prefix, _)) = h.split_once('_') else {
                continue;
            };
            if prefix == code || PREFIX_EXCEPTIONS.contains(&prefix) {
                continue;
            }

            let ref1 = merged(prefix);
            if ref1.is_empty() {
                // python-ags4's rule_19b_2: the prefix names a group
                // that's not defined anywhere. Wording byte-faithful
                // so compat doesn't need a translator entry.
                add(
                    found,
                    RULE_19B,
                    Some(hl),
                    code,
                    format!(
                        "Group {prefix} referred to in {h} could not be \
                         found in either the standard dictionary or the \
                         DICT group."
                    ),
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
                    add(
                        found,
                        RULE_19B,
                        Some(hl),
                        code,
                        format!(
                            "{h} does not start with the name of this \
                             group, nor is it defined in another group."
                        ),
                    );
                }
            } else if !ref1.contains(h) {
                if merged(code).contains(h) {
                    add(
                        found,
                        RULE_19B,
                        Some(hl),
                        code,
                        format!(
                            "Heading {h:?} is defined under group {code} but its prefix \
                             names group {prefix}; rename it or define it under {prefix}."
                        ),
                    );
                } else if !defined_anywhere.contains(h.as_str()) {
                    add(
                        found,
                        RULE_19B,
                        Some(hl),
                        code,
                        format!(
                            "Heading {h:?} neither starts with group {code} nor is \
                             defined in another group."
                        ),
                    );
                }
                // else: defined under some third group → Rule 9's call.
            }
            // else: a valid borrow (defined under `prefix`) → fine.
        }
    }

    rule_20(parsed, found);
    // Opt-in (default off, O-27): the on-disk sidecar tree. Library /
    // `db-to-ags4 --validate` callers stay path-independent; the
    // corpus-qa dogfood + `ags4-check --check-files` enable it to match
    // python-ags4's always-on filesystem stat.
    if check_files {
        if let Some(src) = source {
            rule_20_on_disk(parsed, src, found);
        }
    }
}

/// Rule 20 (on-disk, opt-in via [`crate::CheckOptions::check_files`]).
/// The sidecar `FILE/<FILE_FSET>/<FILE_NAME>` tree must exist beside
/// the `.ags`. `std::fs` only — no new dependency, no lifetime ripple
/// (the `&Path` is borrowed strictly inside one call). Messages are
/// clean-room; the dogfood compares rule-key presence, not wording.
fn rule_20_on_disk(parsed: &ParsedFile, source: &Path, found: &mut Findings) {
    // No FILE group → the data-level pass already spoke (or there are
    // no attachments at all). Nothing on-disk to assert.
    let Some(file_g) = parsed.groups.get("FILE") else {
        return;
    };
    let dir = source.parent().unwrap_or_else(|| Path::new("."));
    let file_root = dir.join("FILE");
    if !file_root.is_dir() {
        add(
            found,
            RULE_20,
            None,
            "FILE",
            "Sidecar 'FILE' folder not found next to the AGS4 file; \
             files declared in the FILE group cannot be located on disk."
                .to_string(),
        );
        return; // no root → probing sub-folders adds only noise
    }
    let Some(fci) = file_g.headings.iter().position(|h| h == "FILE_FSET") else {
        return; // FILE group without FILE_FSET → data-level territory
    };
    let nci = file_g.headings.iter().position(|h| h == "FILE_NAME");

    for row in &file_g.rows {
        let Some(fset) = row.values.get(fci).map(String::as_str) else {
            continue;
        };
        if fset.is_empty() {
            continue;
        }
        let fset_dir = file_root.join(fset);
        if !fset_dir.is_dir() {
            add(
                found,
                RULE_20,
                Some(row.line),
                "FILE",
                format!("Declared FILE_FSET sub-folder 'FILE/{fset}' is missing on disk."),
            );
            continue;
        }
        let name = nci
            .and_then(|c| row.values.get(c))
            .map(String::as_str)
            .unwrap_or("");
        if name.is_empty() {
            continue;
        }
        // FILE_NAME may carry sub-paths; normalise either separator.
        let rel: std::path::PathBuf = name.split(['/', '\\']).collect();
        if !fset_dir.join(&rel).is_file() {
            add(
                found,
                RULE_20,
                Some(row.line),
                "FILE",
                format!("Declared file 'FILE/{fset}/{name}' is missing on disk."),
            );
        }
    }
}

/// Rule 20 (data level) — every FILE_FSET used must be defined in the
/// FILE group; the FILE group must exist when FILE_FSET is used.
fn rule_20(parsed: &ParsedFile, found: &mut Findings) {
    // (group, fset value, line) for every alphanumeric FILE_FSET used.
    let mut used: Vec<(&str, &str, u32)> = Vec::new();
    for code in &parsed.group_order {
        let g = &parsed.groups[code];
        let Some(ci) = g.headings.iter().position(|h| h == "FILE_FSET") else {
            continue;
        };
        for row in &g.rows {
            if let Some(v) = row.values.get(ci) {
                if v.chars().any(|c| c.is_ascii_alphanumeric()) {
                    used.push((code, v, row.line));
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
                .filter_map(|r| r.values.get(ci).map(String::as_str))
                .collect()
        })
        .unwrap_or_default();

    for (group, fset, line) in used {
        if !defined.contains(fset) {
            add(
                found,
                RULE_20,
                Some(line),
                group,
                format!("FILE_FSET {fset:?} is not defined in the FILE group."),
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
        // No source path / check_files off → on-disk half skipped
        // (the default; data-level behaviour is unchanged).
        check(&pf, &d, None, false, &mut f);
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

    #[test]
    fn rule_20_on_disk_opt_in_and_default_off() {
        // Data-level-clean file with a FILE group (FS1/photo.jpg).
        let src = "\"GROUP\",\"LOCA\"\r\n\"HEADING\",\"LOCA_ID\",\"FILE_FSET\"\r\n\
                   \"UNIT\",\"\",\"\"\r\n\"TYPE\",\"ID\",\"X\"\r\n\
                   \"DATA\",\"BH1\",\"FS1\"\r\n\r\n\
                   \"GROUP\",\"FILE\"\r\n\
                   \"HEADING\",\"FILE_FSET\",\"FILE_NAME\"\r\n\
                   \"UNIT\",\"\",\"\"\r\n\"TYPE\",\"X\",\"X\"\r\n\
                   \"DATA\",\"FS1\",\"photo.jpg\"\r\n";
        let pf = parse_str(src).expect("parses");
        let d = Dictionary::bundled(DictVersion::V4_2);
        let tmp = tempfile::tempdir().expect("tempdir");
        let ags = tmp.path().join("site.ags"); // need not exist on disk

        // Default OFF: path-independent — no Rule 20 even with no tree.
        let mut f = Findings::new();
        check(&pf, &d, Some(ags.as_path()), false, &mut f);
        assert!(
            !f.contains_key(RULE_20),
            "default must stay path-independent: {f:?}"
        );

        // Opt-in, tree absent → on-disk Rule 20 fires.
        let mut f = Findings::new();
        check(&pf, &d, Some(ags.as_path()), true, &mut f);
        assert!(
            f.get(RULE_20)
                .is_some_and(|v| v.iter().any(|x| x.group == "FILE")),
            "check_files + missing FILE/ tree must flag Rule 20: {f:?}"
        );

        // Materialise FILE/FS1/photo.jpg → opt-in now clean.
        let leaf = tmp.path().join("FILE").join("FS1");
        std::fs::create_dir_all(&leaf).unwrap();
        std::fs::write(leaf.join("photo.jpg"), b"x").unwrap();
        let mut f = Findings::new();
        check(&pf, &d, Some(ags.as_path()), true, &mut f);
        assert!(
            !f.contains_key(RULE_20),
            "tree present → Rule 20 clean: {f:?}"
        );
    }
}
