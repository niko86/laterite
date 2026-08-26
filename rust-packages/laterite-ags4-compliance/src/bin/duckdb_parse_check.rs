//! `duckdb-parse-check` (laterite-dev#458) — the duckdb surface's *read/parse-agreement* leg.
//!
//! The `laterite_ags4` DuckDB extension became a read-only reader in laterite-dev#446
//! (`validate_ags`/`certify_ags` removed), so it can no longer take part in the
//! cross-surface *findings*-agreement harness (`laterite-ags4-compliance`). This bin
//! instead checks the thing the extension actually does: does its `read_ags()`
//! parse the **same rows** as the canonical engine every laterite surface wraps?
//!
//! The agreement metric is the **content-addressed key set** (`_id`/`_parent_id`,
//! the deterministic keychain of #303/#144): `_id = UUIDv8(SHA-256(spec
//! key-chain))`. These are golden-tested byte-identical across all surfaces and
//! carry no float/temporal formatting to drift between a SQL result and a Rust
//! reference — so "duckdb read the same rows" is exactly "duckdb produced the
//! same `(_id, _parent_id)` set per group".
//!
//! The reference is the **core reader** ([`read_ags4_bytes`]) — the exact read
//! path the extension mirrors, so it shares duckdb's read semantics: UTF-8 with
//! invalid bytes REJECTED (not lossy-replaced), and a no-GROUP file read as an
//! empty table (not an error). Its `_id`s come from the same `keychain` the
//! Python/Node/wasm surfaces use, so a match proves duckdb agrees with the
//! shipped read path, not merely with itself. (Verified over the vendored
//! corpus: the core reader agrees with the leaf-parse reference on all 79
//! well-formed fixtures and — unlike the leaf's lossy/lenient profile — matches
//! duckdb's reject/empty behaviour on the 4 malformed ones.)
//!
//! Reads `<results>/duckdb-parse.json`, emitted by `tools/compliance/emit_duckdb.py`
//! in THIS repo, which installs the extension from the DuckDB community
//! repository — the artefact a user actually gets, rather than one built beside
//! the checkout. A local build can agree with the engine while the published one
//! does not.
//!
//! This bin was public and correct for some time and still never ran: its INPUT
//! producer was not here, so there was nothing for it to read (#719). The comment
//! above used to say the harness lived in the satellite, which was true of the
//! producer and not of this file — and it read as a reason the DuckDB surface
//! could not be checked here at all.
//!
//! Self-skips (exit 0) if that file is absent: there is no community build for
//! this DuckDB version yet, so there is nothing to check. That is a real state
//! after a DuckDB release, not a defect in either engine.
//!
//! Runs in `nightly.yml`'s `docs-vs-released-duckdb` leg — the job that already
//! checks the documented examples still run, which until now was the only thing
//! this repo asked of the extension.
//!
//! **This leg is blind to a read defect in any group the bundled registry does
//! not know — and that blindness is the extension's own premise, not merely a
//! gap in coverage** (#742). Both sides equate "dictionary group" with "registry
//! group"; AGS4 Rule 18 does not, because a group declared in a file's own DICT
//! is a dictionary group whose parent and KEY status come from `DICT_PGRP` and
//! `DICT_STAT`. A check that shares the premise which produces a bug cannot fail
//! on that bug, which is how full agreement and a live read defect sit together
//! without contradiction. So the skip line below NAMES the groups it dropped and
//! says which of them the file declared, rather than reporting a bare count: the
//! dropped set is not incidental, it is where a defect is most likely to be.
//!
//! Exit 1 on any disagreement (a missing/extra registry group, an `_id`-set
//! split, or a read-error disagreement) EXCEPT a documented inherent divergence
//! (see [`known_divergence`] for a group, [`known_read_error`] for a whole file).

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use serde::Deserialize;

use laterite_ags4_core::ags4_codec::read_ags4_bytes;
use laterite_ags4_core::keychain::group_row_ids;
use laterite_ags4_core::registry::{Registry, registry};

/// duckdb-parse.json — schema 2 (`kind: "parse-agreement"`).
#[derive(Debug, Deserialize)]
struct DuckParse {
    #[serde(default)]
    version: Option<String>,
    parses: Vec<FixtureParse>,
}

#[derive(Debug, Deserialize)]
struct FixtureParse {
    fixture: String,
    /// Present (a read-error message) when `read_ags`/`ags_groups` refused the
    /// file — the reference must also fail to read it (both-fail = agree).
    #[serde(default)]
    read_error: Option<String>,
    #[serde(default)]
    groups: Vec<GroupParse>,
}

#[derive(Debug, Deserialize)]
struct GroupParse {
    group: String,
    /// `[[_id, _parent_id], …]` — `_parent_id` is JSON null for a root group.
    ids: Vec<(String, Option<String>)>,
}

/// The reference key-set per registry group: `code -> sorted [(id, parent)]`.
/// `Err` if the core reader can't read the file at all (invalid UTF-8, a
/// structural violation) — which must line up with duckdb's read-error. A
/// no-GROUP file is read as an empty table (`read_ags4_bytes` maps `NotAgs4` to
/// an empty parse), matching duckdb. Custom/passthrough groups (not in the
/// registry) have no spec keys, so they're omitted here and left un-key-checked.
fn file_declared_groups(pa: &laterite_ags4_core::ags4_codec::ParsedAgs4) -> BTreeSet<String> {
    pa.get("DICT")
        .map(|d| {
            d.rows
                .iter()
                .filter_map(|r| r.get("DICT_GRP"))
                .map(|v| v.trim().to_string())
                .filter(|v| !v.is_empty())
                .collect()
        })
        .unwrap_or_default()
}

fn reference(reg: &Registry, bytes: &[u8]) -> Result<Reference, String> {
    let pa = read_ags4_bytes(bytes).map_err(|e| format!("{e:?}"))?;
    let file_declared = file_declared_groups(&pa);
    let mut groups: BTreeMap<String, Vec<(String, Option<String>)>> = BTreeMap::new();
    let mut non_registry = Vec::new();
    for code in &pa.order {
        let g = &pa.groups[code]; // `pa.order` holds trimmed codes; `groups` is keyed by them
        if reg.get(code).is_none() {
            non_registry.push((code.clone(), file_declared.contains(code)));
            continue;
        }
        let mut ids = group_row_ids(reg, code, &g.headings, g.rows.len(), |col, row| {
            // core's rows are name-keyed maps; a short/ragged row lacks the
            // heading → None → `group_row_ids` treats it as "" (same as duckdb's
            // padded short rows).
            g.rows
                .get(row)
                .and_then(|r| r.get(g.headings[col].as_str()))
                .map(String::as_str)
        });
        ids.sort();
        groups.insert(code.clone(), ids);
    }
    Ok(Reference {
        groups,
        non_registry,
    })
}

/// Why duckdb refuses `DuplicateHeaders.ags`, in the extension's own terms: a
/// repeated HEADING cannot become a SQL column without silently merging two of
/// them, so `read_ags` refuses rather than lose data, and points at recovery
/// mode (which keeps both, suffixed). The core reader has no such constraint —
/// it collapses the duplicates into its name-keyed row map. Shared by both
/// allowlists below so the group-scoped and file-scoped excuses cannot come to
/// describe the same divergence differently.
const DUPLICATE_HEADINGS: &str = "a repeated heading cannot be a SQL column \
     without silently merging two (AGS4 Rule 7), so duckdb's read_ags refuses \
     and offers recovery mode; the core reader collapses the duplicate \
     headings into its name-keyed row map";

/// Known, inherent duckdb read/parse divergences — a real, understood difference
/// between duckdb's `read_ags` and the core reader that is EXPECTED, so it's
/// tolerated (reported, not failed). The analog of the findings harness's O-N
/// reconciliation. A NEW/unexplained divergence still fails the gate. Keyed
/// tightly by `(fixture, group)` so a *different* group of the same fixture is
/// not silently excused.
fn known_divergence(fixture: &str, group: &str) -> Option<&'static str> {
    match (fixture, group) {
        // Kept for the group-scoped shape of this divergence. The extension
        // currently refuses the whole file (see `known_read_error`), so this arm
        // is not the one that fires today — it is what fires if `read_ags` ever
        // narrows the refusal back to the offending group.
        ("DuplicateHeaders.ags", "SAMP") => Some(DUPLICATE_HEADINGS),
        _ => None,
    }
}

/// The same idea one level up: duckdb declining to read a fixture AT ALL, where
/// the reference reads it, for a reason we have accepted.
///
/// Keyed on the fixture AND the reason, never the fixture alone. `read_ags`
/// refuses a file for many reasons, and a blanket per-fixture excuse would
/// swallow the next, unrelated one silently — the failure mode the whole
/// harness exists to catch. Matching the extension's own words is what keeps
/// "this refusal" from widening into "any refusal of this file".
fn known_read_error(fixture: &str, err: &str) -> Option<&'static str> {
    match fixture {
        "DuplicateHeaders.ags" if err.contains("duplicate heading") => Some(DUPLICATE_HEADINGS),
        _ => None,
    }
}

struct Reference {
    groups: BTreeMap<String, Vec<(String, Option<String>)>>,
    /// The groups this leg could not key-check, each flagged with whether the
    /// file's own DICT declares it. The flag carries the meaning: a declared
    /// group IS a dictionary group (Rule 18), so "absent from the registry" and
    /// "absent from the dictionary" are different claims and only the first
    /// holds. An undeclared one is a different case and is reported apart.
    non_registry: Vec<(String, bool)>,
}

#[derive(Default)]
struct Report {
    groups_agree: usize,
    groups_checked: usize,
    non_registry_skipped: usize,
    /// `fixture/GROUP` for each skipped group the file's own DICT declares —
    /// dictionary groups under Rule 18, and the ones a read defect is most
    /// likely to hide in.
    non_registry_declared: BTreeSet<String>,
    /// `fixture/GROUP` for each skipped group nothing declares. A different
    /// case, kept apart so the two cannot be read as one number.
    non_registry_undeclared: BTreeSet<String>,
    /// Documented inherent divergences (see [`known_divergence`]) — reported,
    /// not failed.
    known: Vec<String>,
    mismatches: Vec<String>,
}

/// Compare duckdb's read output against the core reference, fixture by fixture.
/// `read_fixture` yields each fixture's bytes (disk in `main`, in-memory in the
/// tests) so the comparison is unit-testable without the real extension.
fn compare(
    reg: &Registry,
    duck: &DuckParse,
    read_fixture: impl Fn(&str) -> Result<Vec<u8>, String>,
) -> Report {
    let mut r = Report::default();
    for fp in &duck.parses {
        let bytes = match read_fixture(&fp.fixture) {
            Ok(b) => b,
            Err(e) => {
                r.mismatches.push(format!(
                    "{}: fixture unreadable for reference: {e}",
                    fp.fixture
                ));
                continue;
            }
        };
        let refr = reference(reg, &bytes);

        // read-error agreement: if duckdb couldn't read the file, the reference
        // must also fail (a hard-error fixture agrees by both failing).
        if let Some(err) = fp.read_error.as_deref() {
            if refr.is_ok() {
                match known_read_error(&fp.fixture, err) {
                    Some(why) => r
                        .known
                        .push(format!("{}: not read by duckdb at all — {why}", fp.fixture)),
                    // The error text, not just the fact of one: a bare "duckdb
                    // read-error but the reference parsed it" sends the next
                    // reader to rebuild the extension to find out WHY, which is
                    // a CI log that reports a failure without its evidence.
                    None => r.mismatches.push(format!(
                        "{}: duckdb read-error but the reference parsed it — {err}",
                        fp.fixture
                    )),
                }
            }
            continue;
        }
        let refr = match refr {
            Ok(r2) => r2,
            Err(e) => {
                r.mismatches.push(format!(
                    "{}: reference read-error ({e}) but duckdb parsed it",
                    fp.fixture
                ));
                continue;
            }
        };
        r.non_registry_skipped += refr.non_registry.len();
        for (code, declared) in &refr.non_registry {
            let entry = format!("{}/{code}", fp.fixture);
            if *declared {
                r.non_registry_declared.insert(entry);
            } else {
                r.non_registry_undeclared.insert(entry);
            }
        }

        // duckdb's registry groups keyed by trimmed code (non-registry groups
        // aren't in the reference, so they're ignored here, not failed).
        let duck_groups: BTreeMap<&str, &Vec<(String, Option<String>)>> =
            fp.groups.iter().map(|g| (g.group.trim(), &g.ids)).collect();

        for (code, ref_ids) in &refr.groups {
            r.groups_checked += 1;
            match duck_groups.get(code.as_str()) {
                None => match known_divergence(&fp.fixture, code) {
                    Some(why) => r.known.push(format!(
                        "{} / {}: not read by duckdb — {why}",
                        fp.fixture, code
                    )),
                    None => r.mismatches.push(format!(
                        "{} / {}: in the reference but not read by duckdb",
                        fp.fixture, code
                    )),
                },
                Some(duck_ids) => {
                    // duckdb's list is already sorted (emit_duckdb sorts it), but
                    // sort defensively so the comparison is order-independent.
                    let mut d = (*duck_ids).clone();
                    d.sort();
                    if &d == ref_ids {
                        r.groups_agree += 1;
                    } else if let Some(why) = known_divergence(&fp.fixture, code) {
                        r.known.push(format!(
                            "{} / {}: _id set differs — {why}",
                            fp.fixture, code
                        ));
                    } else {
                        r.mismatches.push(format!(
                            "{} / {}: _id set differs (reference {} rows, duckdb {} rows)",
                            fp.fixture,
                            code,
                            ref_ids.len(),
                            d.len()
                        ));
                    }
                }
            }
        }
    }
    r
}

fn main() {
    let mut args = std::env::args().skip(1);
    let fixtures_dir = args.next().map_or_else(
        || PathBuf::from("../ags-python-library/tests/test_files"),
        PathBuf::from,
    );
    let results_dir = args
        .next()
        .map_or_else(|| PathBuf::from("output/compliance-results"), PathBuf::from);

    let dfile = results_dir.join("duckdb-parse.json");
    if !dfile.is_file() {
        println!(
            "duckdb parse-agreement: no {} (extension not built) — skipping",
            dfile.display()
        );
        std::process::exit(0);
    }

    let duck: DuckParse = match std::fs::read_to_string(&dfile)
        .map_err(|e| e.to_string())
        .and_then(|s| serde_json::from_str(&s).map_err(|e| e.to_string()))
    {
        Ok(d) => d,
        Err(e) => {
            eprintln!("failed to read {}: {e}", dfile.display());
            std::process::exit(3);
        }
    };

    let report = compare(registry(), &duck, |name| {
        std::fs::read(fixtures_dir.join(name)).map_err(|e| e.to_string())
    });

    let ver = duck.version.as_deref().unwrap_or("?");
    println!("# duckdb read/parse agreement (laterite-dev#458) — extension v{ver}\n");
    println!("fixtures checked: {}", duck.parses.len());
    println!(
        "registry groups agreeing: {}/{}",
        report.groups_agree, report.groups_checked
    );
    if report.non_registry_skipped > 0 {
        println!(
            "non-standard groups not key-checked: {} ({} declared in the file's own \
             DICT, {} declared nowhere)",
            report.non_registry_skipped,
            report.non_registry_declared.len(),
            report.non_registry_undeclared.len()
        );
        if !report.non_registry_declared.is_empty() {
            println!(
                "  declared in the file's DICT — dictionary groups under Rule 18, whose \
                 parent and KEY status come from DICT_PGRP/DICT_STAT rather than the \
                 registry, so this leg is blind to a read defect in exactly them:"
            );
            for g in &report.non_registry_declared {
                println!("    · {g}");
            }
        }
        if !report.non_registry_undeclared.is_empty() {
            println!("  present but declared nowhere (not a dictionary group):");
            for g in &report.non_registry_undeclared {
                println!("    · {g}");
            }
        }
    }
    if !report.known.is_empty() {
        println!(
            "\nknown inherent divergences ({}, tolerated — see `known_divergence`):",
            report.known.len()
        );
        for k in &report.known {
            println!("  · {k}");
        }
    }

    if report.mismatches.is_empty() {
        println!("\nduckdb read/parse agreement: OK");
        std::process::exit(0);
    }
    println!("\n!! DISAGREEMENTS ({}):", report.mismatches.len());
    for m in &report.mismatches {
        println!("  - {m}");
    }
    std::process::exit(1);
}

#[cfg(test)]
mod tests {
    use super::*;

    // A minimal 2-group AGS4 file: PROJ (root, KEY PROJ_ID) + LOCA (child,
    // KEY LOCA_ID, carrying PROJ_ID denormalised) — enough that `group_row_ids`
    // mints real `_id`/`_parent_id`s and `child._parent_id == parent._id`.
    const AGS: &[u8] = b"\"GROUP\",\"PROJ\"\r\n\
\"HEADING\",\"PROJ_ID\",\"PROJ_NAME\"\r\n\
\"UNIT\",\"\",\"\"\r\n\
\"TYPE\",\"ID\",\"X\"\r\n\
\"DATA\",\"P1\",\"Test\"\r\n\
\"GROUP\",\"LOCA\"\r\n\
\"HEADING\",\"LOCA_ID\",\"PROJ_ID\"\r\n\
\"UNIT\",\"\",\"\"\r\n\
\"TYPE\",\"ID\",\"ID\"\r\n\
\"DATA\",\"BH1\",\"P1\"\r\n\
\"DATA\",\"BH2\",\"P1\"\r\n";

    // `AGS` plus a SAMP group (child of LOCA) — used to exercise a "duckdb
    // couldn't read this group" case against the known-divergence allowlist.
    const AGS_SAMP: &[u8] = b"\"GROUP\",\"PROJ\"\r\n\
\"HEADING\",\"PROJ_ID\",\"PROJ_NAME\"\r\n\
\"UNIT\",\"\",\"\"\r\n\
\"TYPE\",\"ID\",\"X\"\r\n\
\"DATA\",\"P1\",\"Test\"\r\n\
\"GROUP\",\"LOCA\"\r\n\
\"HEADING\",\"LOCA_ID\",\"PROJ_ID\"\r\n\
\"UNIT\",\"\",\"\"\r\n\
\"TYPE\",\"ID\",\"ID\"\r\n\
\"DATA\",\"BH1\",\"P1\"\r\n\
\"GROUP\",\"SAMP\"\r\n\
\"HEADING\",\"LOCA_ID\",\"SAMP_TOP\",\"SAMP_REF\",\"SAMP_TYPE\",\"SAMP_ID\",\"PROJ_ID\"\r\n\
\"UNIT\",\"\",\"m\",\"\",\"\",\"\",\"\"\r\n\
\"TYPE\",\"ID\",\"2DP\",\"ID\",\"PA\",\"ID\",\"ID\"\r\n\
\"DATA\",\"BH1\",\"1.00\",\"S1\",\"D\",\"BH1_1.00\",\"P1\"\r\n";

    /// duckdb-parse.json for a fixture, minting the reference key-sets and
    /// optionally DROPPING one group (to simulate duckdb failing to read it).
    fn duck_for(name: &str, bytes: &[u8], drop_group: Option<&str>) -> DuckParse {
        let refr = reference(registry(), bytes).unwrap();
        let groups = refr
            .groups
            .iter()
            .filter(|(g, _)| Some(g.as_str()) != drop_group)
            .map(|(g, ids)| GroupParse {
                group: g.clone(),
                ids: ids.clone(),
            })
            .collect();
        DuckParse {
            version: Some("0.7.0".into()),
            parses: vec![FixtureParse {
                fixture: name.into(),
                read_error: None,
                groups,
            }],
        }
    }

    /// Build the duckdb-parse.json a *correct* extension would emit for `AGS`
    /// (its `_id`s are byte-identical to the reference by construction — #303),
    /// so `compare` should see full agreement.
    fn duck_matching() -> DuckParse {
        let refr = reference(registry(), AGS).unwrap();
        let groups = refr
            .groups
            .iter()
            .map(|(g, ids)| GroupParse {
                group: g.clone(),
                ids: ids.clone(),
            })
            .collect();
        DuckParse {
            version: Some("0.7.0".into()),
            parses: vec![FixtureParse {
                fixture: "t.ags".into(),
                read_error: None,
                groups,
            }],
        }
    }

    fn run(duck: &DuckParse) -> Report {
        compare(registry(), duck, |_| Ok(AGS.to_vec()))
    }

    #[test]
    fn agrees_when_duckdb_matches_the_reference() {
        let r = run(&duck_matching());
        assert!(r.mismatches.is_empty(), "{:?}", r.mismatches);
        assert_eq!(r.groups_checked, 2); // PROJ + LOCA
        assert_eq!(r.groups_agree, 2);
    }

    #[test]
    fn bites_when_an_id_differs() {
        // Flip one character of one `_id` — a corrupted read must be caught.
        let mut duck = duck_matching();
        let bad = &mut duck.parses[0].groups[0].ids[0].0;
        let last = bad.pop().unwrap();
        bad.push(if last == 'a' { 'b' } else { 'a' });
        let r = run(&duck);
        assert_eq!(r.mismatches.len(), 1, "a flipped _id must disagree");
    }

    #[test]
    fn bites_when_a_group_is_missing() {
        // duckdb didn't read a registry group the reference has.
        let mut duck = duck_matching();
        duck.parses[0].groups.truncate(1);
        let r = run(&duck);
        assert!(
            r.mismatches
                .iter()
                .any(|m| m.contains("not read by duckdb")),
            "a missing registry group must disagree: {:?}",
            r.mismatches
        );
    }

    #[test]
    fn read_error_agrees_only_when_both_fail() {
        // duckdb reports a read-error but the reference parses `AGS` fine → split.
        let mut duck = duck_matching();
        duck.parses[0].read_error = Some("Binder Error: not AGS4".into());
        duck.parses[0].groups.clear();
        let r = run(&duck);
        assert_eq!(r.mismatches.len(), 1);
        assert!(r.mismatches[0].contains("read-error but the reference parsed"));
    }

    #[test]
    fn tolerates_a_documented_whole_file_refusal() {
        // What the live gate hit (laterite-dev#659): the extension refuses
        // `DuplicateHeaders.ags` outright — `ags_groups` itself raises — so the
        // divergence never reaches the group arms. Documented reason, so it is
        // reported and tolerated rather than failing the leg.
        let mut duck = duck_for("DuplicateHeaders.ags", AGS_SAMP, None);
        duck.parses[0].read_error = Some(
            "Binder Error: read_ags: did not parse as AGS4 (duplicate heading \
             \"SAMP_BASE\" in group \"SAMP\" (AGS4 Rule 7) — reading it would \
             silently merge two columns; re-read in recovery mode …)"
                .into(),
        );
        duck.parses[0].groups.clear();
        let r = compare(registry(), &duck, |_| Ok(AGS_SAMP.to_vec()));
        assert!(
            r.mismatches.is_empty(),
            "a documented whole-file refusal must not fail: {:?}",
            r.mismatches
        );
        assert_eq!(r.known.len(), 1, "it must still be REPORTED, not silent");
    }

    #[test]
    fn a_different_refusal_of_the_same_fixture_still_fails() {
        // The excuse is keyed on the reason, not the filename. Any other reason
        // duckdb might refuse this same file is a new divergence, and the
        // message must carry duckdb's own words so CI says why.
        let mut duck = duck_for("DuplicateHeaders.ags", AGS_SAMP, None);
        duck.parses[0].read_error = Some("IO Error: file vanished mid-read".into());
        duck.parses[0].groups.clear();
        let r = compare(registry(), &duck, |_| Ok(AGS_SAMP.to_vec()));
        assert_eq!(r.mismatches.len(), 1, "an undocumented refusal must fail");
        assert!(
            r.mismatches[0].contains("file vanished mid-read"),
            "the failure must carry duckdb's reason: {:?}",
            r.mismatches
        );
        assert!(r.known.is_empty());
    }

    #[test]
    fn tolerates_a_known_inherent_divergence() {
        // DuplicateHeaders.ags / SAMP: duckdb can't build a SQL table with
        // duplicate columns, so it omits SAMP — allowlisted, not a failure.
        let duck = duck_for("DuplicateHeaders.ags", AGS_SAMP, Some("SAMP"));
        let r = compare(registry(), &duck, |_| Ok(AGS_SAMP.to_vec()));
        assert!(
            r.mismatches.is_empty(),
            "a known divergence must not fail: {:?}",
            r.mismatches
        );
        assert_eq!(
            r.known.len(),
            1,
            "the SAMP divergence should be recorded as known"
        );
    }

    #[test]
    fn an_unlisted_missing_group_still_fails() {
        // The SAME shape on a fixture NOT in the allowlist stays a real mismatch —
        // the allowlist is tight (per fixture+group), it doesn't blanket-excuse.
        let duck = duck_for("SomeOther.ags", AGS_SAMP, Some("SAMP"));
        let r = compare(registry(), &duck, |_| Ok(AGS_SAMP.to_vec()));
        assert_eq!(r.mismatches.len(), 1, "an unlisted missing group must fail");
        assert!(r.known.is_empty());
    }

    // `AGS` plus a DICT group that declares XLOG — a group the bundled registry
    // has never seen, whose parent (PROJ) and KEY heading (XLOG_ID) exist only
    // in DICT_PGRP/DICT_STAT. This is the AGS4 Rule 18 case the leg cannot
    // key-check, and the one the extension's binder refuses outright (#742).
    const AGS_FILE_DICT: &[u8] = b"\"GROUP\",\"PROJ\"\r\n\
\"HEADING\",\"PROJ_ID\",\"PROJ_NAME\"\r\n\
\"UNIT\",\"\",\"\"\r\n\
\"TYPE\",\"ID\",\"X\"\r\n\
\"DATA\",\"P1\",\"Test\"\r\n\
\"GROUP\",\"DICT\"\r\n\
\"HEADING\",\"DICT_TYPE\",\"DICT_GRP\",\"DICT_HDNG\",\"DICT_STAT\",\"DICT_DTYP\",\"DICT_DESC\",\"DICT_PGRP\"\r\n\
\"UNIT\",\"\",\"\",\"\",\"\",\"\",\"\",\"\"\r\n\
\"TYPE\",\"X\",\"X\",\"X\",\"X\",\"PA\",\"X\",\"X\"\r\n\
\"DATA\",\"GROUP\",\"XLOG\",\"\",\"\",\"\",\"Custom log\",\"PROJ\"\r\n\
\"DATA\",\"HEADING\",\"XLOG\",\"XLOG_ID\",\"KEY\",\"ID\",\"Custom id\",\"\"\r\n\
\"GROUP\",\"XLOG\"\r\n\
\"HEADING\",\"XLOG_ID\",\"PROJ_ID\"\r\n\
\"UNIT\",\"\",\"\"\r\n\
\"TYPE\",\"ID\",\"ID\"\r\n\
\"DATA\",\"X1\",\"P1\"\r\n";

    // The same shape WITHOUT the declaration: PQRS is present and nothing in
    // the file says what it is. Genuinely un-key-checkable, and a different
    // statement from the one above — which is why the report keeps them apart.
    const AGS_UNDECLARED: &[u8] = b"\"GROUP\",\"PROJ\"\r\n\
\"HEADING\",\"PROJ_ID\",\"PROJ_NAME\"\r\n\
\"UNIT\",\"\",\"\"\r\n\
\"TYPE\",\"ID\",\"X\"\r\n\
\"DATA\",\"P1\",\"Test\"\r\n\
\"GROUP\",\"PQRS\"\r\n\
\"HEADING\",\"PQRS_ID\",\"PROJ_ID\"\r\n\
\"UNIT\",\"\",\"\"\r\n\
\"TYPE\",\"ID\",\"ID\"\r\n\
\"DATA\",\"Q1\",\"P1\"\r\n";

    #[test]
    fn a_file_declared_group_is_named_and_marked_declared() {
        let duck = duck_for("dict.ags", AGS_FILE_DICT, None);
        let r = compare(registry(), &duck, |_| Ok(AGS_FILE_DICT.to_vec()));
        assert!(r.mismatches.is_empty(), "{:?}", r.mismatches);
        assert_eq!(r.non_registry_skipped, 1);
        assert!(
            r.non_registry_declared.contains("dict.ags/XLOG"),
            "the skip must NAME the group and say the file declared it: {:?}",
            r.non_registry_declared
        );
        assert!(
            r.non_registry_undeclared.is_empty(),
            "a DICT-declared group is not an undeclared one: {:?}",
            r.non_registry_undeclared
        );
    }

    #[test]
    fn an_undeclared_group_is_reported_apart_from_a_declared_one() {
        let duck = duck_for("undeclared.ags", AGS_UNDECLARED, None);
        let r = compare(registry(), &duck, |_| Ok(AGS_UNDECLARED.to_vec()));
        assert!(r.mismatches.is_empty(), "{:?}", r.mismatches);
        assert_eq!(r.non_registry_skipped, 1);
        assert!(
            r.non_registry_undeclared.contains("undeclared.ags/PQRS"),
            "{:?}",
            r.non_registry_undeclared
        );
        // The pair of assertions is what makes the classification falsifiable:
        // hardcode the flag either way and exactly one of these two tests goes
        // red. A single test would pass against a constant.
        assert!(
            r.non_registry_declared.is_empty(),
            "nothing declares PQRS: {:?}",
            r.non_registry_declared
        );
    }
}
