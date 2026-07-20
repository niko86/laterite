//! WORLD checks — the rules that read state the AGS4 bytes do not contain.
//!
//! Everything in `rules/` is a pure function of the parsed bytes: same bytes in,
//! same findings out, forever. This module is the one deliberate exception, and
//! it is a module rather than a rule so that the exception has an address.
//!
//! Why the partition is load-bearing: an `.ags.idx` certificate records a verdict
//! against a SHA-256 of the file. That lets a later run skip the rule engine —
//! sound, because content findings cannot change while the bytes don't. It says
//! nothing whatsoever about the sibling `FILE/` tree, which someone can delete
//! without touching a byte of the `.ags`. So a WORLD check may never be skipped
//! on the strength of a certificate, and must never be *silently* skipped for any
//! other reason either (see [`WorldScope`]).
//!
//! Today Rule 20's on-disk half is the only inhabitant. The external `--dict`
//! override (O-28) is the obvious next one if it is ever implemented.

use std::path::{Path, PathBuf};

use crate::findings::{Findings, add};
use crate::parse::ParsedFile;
use crate::rules::references::RULE_20;

/// What a check is allowed to look at beyond the bytes.
///
/// Deliberately **not** `(check_files: bool, source: Option<&Path>)` — that pair
/// has a fourth state, "asked for the check, had nothing to check it against",
/// and the engine used to answer it by quietly reporting Rule 20 clean. Every
/// bytes/text read (and wasm, always) took that path. Here the world cannot be
/// requested without supplying the thing to check against, so the incoherent
/// state is not representable; a caller who *wants* to ask and has no path gets
/// [`crate::ValidatorError::WorldCheckRequiresSource`] instead of a false clean.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorldScope {
    /// Content-only. The file's bytes are the whole of the evidence.
    None,
    /// The `.ags` file's own path — Rule 20 looks for `FILE/` beside it.
    OnDisk(PathBuf),
}

/// Run every WORLD check in `scope`. Called from the one door in
/// [`crate::check_parsed`], *outside* any certificate-skip branch — the whole
/// point is that this runs even when the content half is vouched for.
pub fn run(parsed: &ParsedFile, scope: &WorldScope, found: &mut Findings) {
    match scope {
        WorldScope::None => {}
        WorldScope::OnDisk(source) => rule_20_on_disk(parsed, source, found),
    }
}

/// Rule 20 (on-disk half). The sidecar `FILE/<FILE_FSET>/<FILE_NAME>` tree must
/// exist beside the `.ags`. `std::fs` only — no new dependency. Messages are
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
            .map_or("", String::as_str);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::parse_str;

    #[test]
    fn flags_missing_fset_subfolder_and_file() {
        // FILE/ root exists but the declared FS1 sub-folder is absent → the
        // "sub-folder … is missing on disk" arm. A second FILE row (FS2) has the
        // sub-folder but not the named file → the "Declared file …" arm.
        let src = "\"GROUP\",\"LOCA\"\r\n\"HEADING\",\"LOCA_ID\",\"FILE_FSET\"\r\n\
                   \"UNIT\",\"\",\"\"\r\n\"TYPE\",\"ID\",\"X\"\r\n\
                   \"DATA\",\"BH1\",\"FS1\"\r\n\"DATA\",\"BH2\",\"FS2\"\r\n\r\n\
                   \"GROUP\",\"FILE\"\r\n\
                   \"HEADING\",\"FILE_FSET\",\"FILE_NAME\"\r\n\
                   \"UNIT\",\"\",\"\"\r\n\"TYPE\",\"X\",\"X\"\r\n\
                   \"DATA\",\"FS1\",\"a.jpg\"\r\n\"DATA\",\"FS2\",\"b.jpg\"\r\n";
        let pf = parse_str(src).expect("parses");
        let tmp = tempfile::tempdir().expect("tempdir");
        let ags = tmp.path().join("site.ags");
        std::fs::create_dir_all(tmp.path().join("FILE").join("FS2")).unwrap();

        let mut f = Findings::new();
        run(&pf, &WorldScope::OnDisk(ags.clone()), &mut f);
        let r20 = f.get(RULE_20).expect("Rule 20 on-disk");
        assert!(
            r20.iter()
                .any(|x| x.desc.contains("FILE/FS1") && x.desc.contains("missing on disk")),
            "missing FS1 sub-folder must flag: {r20:?}"
        );
        assert!(
            r20.iter()
                .any(|x| x.desc.contains("FILE/FS2/b.jpg") && x.desc.contains("missing on disk")),
            "missing file under present FS2 sub-folder must flag: {r20:?}"
        );
    }

    #[test]
    fn silent_without_file_group() {
        // No FILE group at all → early return (nothing on-disk to assert).
        let src = "\"GROUP\",\"LOCA\"\r\n\"HEADING\",\"LOCA_ID\"\r\n\
                   \"UNIT\",\"\"\r\n\"TYPE\",\"ID\"\r\n\"DATA\",\"BH1\"\r\n";
        let pf = parse_str(src).expect("parses");
        let tmp = tempfile::tempdir().expect("tempdir");
        let ags = tmp.path().join("site.ags");
        let mut f = Findings::new();
        run(&pf, &WorldScope::OnDisk(ags), &mut f);
        assert!(
            !f.contains_key(RULE_20),
            "no FILE group → no Rule 20: {f:?}"
        );
    }

    #[test]
    fn none_scope_is_path_independent_and_on_disk_is_live() {
        // Data-level-clean file with a FILE group (FS1/photo.jpg).
        let src = "\"GROUP\",\"LOCA\"\r\n\"HEADING\",\"LOCA_ID\",\"FILE_FSET\"\r\n\
                   \"UNIT\",\"\",\"\"\r\n\"TYPE\",\"ID\",\"X\"\r\n\
                   \"DATA\",\"BH1\",\"FS1\"\r\n\r\n\
                   \"GROUP\",\"FILE\"\r\n\
                   \"HEADING\",\"FILE_FSET\",\"FILE_NAME\"\r\n\
                   \"UNIT\",\"\",\"\"\r\n\"TYPE\",\"X\",\"X\"\r\n\
                   \"DATA\",\"FS1\",\"photo.jpg\"\r\n";
        let pf = parse_str(src).expect("parses");
        let tmp = tempfile::tempdir().expect("tempdir");
        let ags = tmp.path().join("site.ags"); // need not exist on disk

        // Scope::None: path-independent — no Rule 20 even with no tree.
        let mut f = Findings::new();
        run(&pf, &WorldScope::None, &mut f);
        assert!(
            !f.contains_key(RULE_20),
            "WorldScope::None must stay path-independent: {f:?}"
        );

        // OnDisk, tree absent → fires.
        let mut f = Findings::new();
        run(&pf, &WorldScope::OnDisk(ags.clone()), &mut f);
        assert!(
            f.get(RULE_20)
                .is_some_and(|v| v.iter().any(|x| x.group == "FILE")),
            "missing FILE/ tree must flag Rule 20: {f:?}"
        );

        // Materialise FILE/FS1/photo.jpg → the same scope is now clean. This is
        // the whole reason the check can't be cached: the verdict changed and
        // not one byte of the .ags moved.
        let leaf = tmp.path().join("FILE").join("FS1");
        std::fs::create_dir_all(&leaf).unwrap();
        std::fs::write(leaf.join("photo.jpg"), b"x").unwrap();
        let mut f = Findings::new();
        run(&pf, &WorldScope::OnDisk(ags), &mut f);
        assert!(
            !f.contains_key(RULE_20),
            "tree present → Rule 20 clean: {f:?}"
        );
    }
}
