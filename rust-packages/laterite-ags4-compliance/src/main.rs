//! `laterite-ags4-compliance` (#169) — cross-surface findings-agreement harness.
//!
//! Reads one finding-set JSON per validation surface (`<surface>.json` in a
//! results dir, emitted by the per-surface runners that PULL the PUBLISHED
//! artifacts — `uv pip`/`npm`/duckdb `INSTALL`) and checks two invariants over
//! python-ags4's 84 fixtures:
//!
//!   1. **4-laterite identity (hard).** Every findings-emitting laterite surface
//!      (rust / python-laterite / node / wasm) wraps the SAME engine, so over
//!      any fixture their finding FLOOR must be IDENTICAL — compared as full
//!      TUPLES (rule, line, group, desc, `field_index`) in a count-sensitive
//!      multiset (#555 part 1), not the deduplicated rule-label SET it used to
//!      be: two surfaces agreeing on which rules fired can still disagree on
//!      where, how many, or what they said. A split is a binding / serialization
//!      / build bug — there is no O-N escape hatch. (`duckdb` became a read-only
//!      reader in #446 → it does read/parse-agreement via the `duckdb-parse-
//!      check` bin instead, not findings; #458.)
//!   2. **python-ags4 agreement modulo O-N (dogfood).** The canonical laterite
//!      floor vs python-ags4's floor through [`laterite_ags4_parity::classify`] —
//!      VERBATIM, so the documented O-2/O-3/O-26/O-30/O-34 divergences reconcile
//!      and only an unexplained difference is an action.
//!
//! The two legs read DIFFERENT fields for a reason: the 4-laterite check uses
//! `findings` (the full tuples above), while the python-ags4 leg stays on the
//! `"AGS Format Rule N"` LABEL floor (`rules`) — python-ags4 emits only labels
//! and the O-N reconciliation is defined over them, so tuples have no peer there.
//! FYIs are carried separately and compared among all four surfaces (all are
//! FYI-capable now duckdb has left findings — #194/#458). Both validators emit
//! identical FYI labels, so the floor strip is symmetric, not a reconciliation
//! of a real divergence.

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use laterite_ags4_parity::{Parity, RustResult, classify};
use serde::Deserialize;

/// The laterite surfaces (same engine; must be byte-identical on the floor),
/// in canonical order — the first present is the canonical for the python leg.
/// `duckdb` was dropped here in #458: the extension became a read-only reader
/// (#446 removed `validate_ags`), so it emits no findings — its cross-surface
/// agreement is checked by the `duckdb-parse-check` bin (read/parse-agreement)
/// instead. See `tools/compliance/README.md`.
const LATERITE: &[&str] = &["rust", "python-laterite", "node", "wasm"];
/// Surfaces that can emit FYI — the same set (duckdb, the only non-FYI surface,
/// no longer participates in findings-agreement at all; #194/#458).
const FYI_CAPABLE: &[&str] = &["rust", "python-laterite", "node", "wasm"];
const REFERENCE: &str = "python-ags4";

#[derive(Debug, Deserialize)]
struct SurfaceFile {
    surface: String,
    /// The surface's own reported version. COMPARED across surfaces since #556 —
    /// a version split means "same engine" is false and the identity claim is
    /// vacuous. It was collected and printed and never checked, which is how
    /// `wasm v0.5.1` sat beside three `v0.7.0` surfaces under the heading
    /// "4-laterite floor identical".
    #[serde(default)]
    version: Option<String>,
    /// RESIDUAL, named rather than left to look implemented: **no runner has ever
    /// populated this.** It is `-` in every report ever printed.
    ///
    /// It is also the check we actually want: `version` is a PROXY for "the same
    /// engine decided these", and a proxy is what this whole arc keeps finding
    /// (#549). Two surfaces can share a version and still embed different engine
    /// builds — exactly the false clean #550's `ENGINE_FINGERPRINT` exists to
    /// prevent, one layer out. Populating it means each surface exposing that
    /// fingerprint, which none does today.
    ///
    /// Kept, not deleted: it names a real gap, and a field that reads `-` is at
    /// least visibly empty. Do NOT read its presence as evidence of anything.
    #[serde(default)]
    engine_sha: Option<String>,
    results: Vec<FixtureFindings>,
}

#[derive(Debug, Clone, Deserialize)]
struct FixtureFindings {
    #[allow(dead_code)]
    fixture: String,
    /// The `"AGS Format Rule N"` floor as bare LABELS. Kept because the
    /// python-ags4 leg ([`classify`]) compares label sets — python-ags4 emits
    /// only labels, and the O-N reconciliation is defined over them. The
    /// 4-laterite identity check does NOT use this; it uses `findings` (below).
    #[serde(default)]
    rules: Vec<String>,
    /// The `"AGS Format Rule N"` floor as full TUPLES (rule, line, group, desc,
    /// `field_index`) — the value the 4-laterite identity check actually compares
    /// (#555 part 1). `rules` is a projection of this (its deduplicated labels);
    /// carrying the tuples is the point — two surfaces agreeing on WHICH rules
    /// fired can still disagree on WHERE / HOW MANY / WHAT they said, and that
    /// split lived under the label set where no comparator could reach it. FYI
    /// findings are excluded, exactly like `rules`.
    #[serde(default)]
    findings: Vec<FindingTuple>,
    #[serde(default)]
    fyi: Vec<String>,
    /// A hard error variant string (`NotAgs4` / `UnsupportedEdition` / …) for a
    /// laterite surface that couldn't validate at all.
    #[serde(default)]
    hard_error: Option<String>,
    /// A python-ags4-side error string (the reference surface only).
    #[serde(default)]
    error: Option<String>,
}

/// One floor finding as the 4-laterite check compares it. `line`/`field_index`
/// are `Option` because whole-group findings (Rule 13/14/…) attach to neither.
/// Derives `Ord` so a fixture's findings sort into a canonical multiset — the
/// comparison is order-insensitive but count- and content-SENSITIVE (a surface
/// emitting Rule 8 twice where another emits it once is a split, which a
/// deduplicated label set silently hid). Field order here IS the sort key:
/// rule, then line, group, desc, `field_index`.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
struct FindingTuple {
    rule: String,
    #[serde(default)]
    line: Option<u32>,
    group: String,
    desc: String,
    #[serde(default)]
    field_index: Option<u32>,
}

/// Floor [`RustResult`] for a laterite surface: a hard-error variant, else the
/// rule-floor LABEL set (`Clean` when empty). FYI is excluded by construction.
/// Used ONLY for the python-ags4 leg (`classify`), which is label-based; the
/// 4-laterite identity check uses [`tuple_floor`].
fn floor_result(f: &FixtureFindings) -> RustResult {
    if let Some(he) = &f.hard_error {
        return RustResult::HardError(he.clone());
    }
    let set: BTreeSet<String> = f.rules.iter().cloned().collect();
    if set.is_empty() {
        RustResult::Clean
    } else {
        RustResult::Rules(set)
    }
}

/// What the 4-laterite identity check compares: a hard-error variant, else the
/// full finding TUPLES as a canonical (sorted) MULTISET. Unlike [`floor_result`]
/// this keeps `line`/`group`/`desc`/`field_index` and DUPLICATES — the same
/// engine wrapped four ways must produce not just the same rule labels but the
/// same findings, the same number of times, saying the same thing.
#[derive(Debug, Clone, PartialEq, Eq)]
enum TupleFloor {
    HardError(String),
    Findings(Vec<FindingTuple>),
}

fn tuple_floor(f: &FixtureFindings) -> TupleFloor {
    if let Some(he) = &f.hard_error {
        return TupleFloor::HardError(he.clone());
    }
    let mut v = f.findings.clone();
    v.sort();
    TupleFloor::Findings(v)
}

/// python-ags4's floor as `classify` wants it: `Err(reason)` on a python error,
/// else the rule-floor set.
fn py_floor(f: &FixtureFindings) -> Result<BTreeSet<String>, String> {
    match &f.error {
        Some(e) => Err(e.clone()),
        None => Ok(f.rules.iter().cloned().collect()),
    }
}

fn fyi_set(f: &FixtureFindings) -> BTreeSet<String> {
    f.fyi.iter().cloned().collect()
}

type Index = BTreeMap<String, BTreeMap<String, FixtureFindings>>;

#[derive(Default, Debug)]
struct Report {
    fixtures: usize,
    /// floor-identical across the present laterite surfaces
    identical: usize,
    /// fixture → [(surface, tuple floor)] where the laterite surfaces split
    binding_splits: Vec<(String, Vec<(String, TupleFloor)>)>,
    /// fixtures where the FYI-capable surfaces disagree on the FYI set
    fyi_splits: Vec<String>,
    /// LATERITE surfaces that produced no results file at all (#556). The
    /// invariant above says "4-laterite identity (HARD)" — but a surface whose
    /// runner died was silently filtered out of `present_lat` and the remaining
    /// three agreed with each other, so the gate reported COMPLIANCE OK having
    /// quietly stopped checking the thing it names. Absence of evidence read as
    /// evidence of agreement.
    missing_surfaces: Vec<String>,
    /// fixture → the present surfaces that didn't report it (#556). The subtler
    /// half: a surface can be present yet skip a fixture, and `lat.iter().all()`
    /// over ONE element is vacuously true — so a fixture only one surface
    /// reported counted as "identical across four". Cross-surface agreement with
    /// nothing to compare against.
    partial_fixtures: Vec<(String, Vec<String>)>,
    py_agree: usize,
    py_known: Vec<(String, String)>,
    py_error: usize,
    /// fixture → the action verdict (a real, undocumented divergence)
    actions: Vec<(String, Parity)>,
}

/// The comparison core — pure over the loaded index, so it is unit-tested
/// without touching the filesystem or any surface.
fn compare(idx: &Index, fixtures: &BTreeSet<String>) -> Report {
    let present_lat: Vec<&str> = LATERITE
        .iter()
        .copied()
        .filter(|s| idx.contains_key(*s))
        .collect();
    let canonical = present_lat.first().copied();
    let mut rep = Report {
        fixtures: fixtures.len(),
        ..Default::default()
    };
    // A surface that never reported is not "agreement", it is a missing witness.
    // This is the whole invariant: the name says FOUR-laterite identity.
    rep.missing_surfaces = LATERITE
        .iter()
        .filter(|s| !idx.contains_key(**s))
        .map(std::string::ToString::to_string)
        .collect();
    for fx in fixtures {
        // (1) 4-laterite identity over the present surfaces — on the full
        // finding TUPLES, not the deduplicated label set (#555 part 1).
        let lat: Vec<(String, TupleFloor)> = present_lat
            .iter()
            .filter_map(|s| idx[*s].get(fx).map(|f| (s.to_string(), tuple_floor(f))))
            .collect();
        // Which present surfaces stayed silent about this fixture? `all()` over one
        // element is vacuously true, so without this a fixture that only `rust`
        // reported would count as "identical across four" — the comparator agreeing
        // with itself. Recorded per-fixture rather than counted, because WHICH
        // surface went quiet is the diagnostic.
        let silent: Vec<String> = present_lat
            .iter()
            .filter(|s| !idx[**s].contains_key(fx))
            .map(std::string::ToString::to_string)
            .collect();
        if !silent.is_empty() {
            rep.partial_fixtures.push((fx.clone(), silent));
        }
        match lat.first() {
            Some((_, first)) if lat.iter().all(|(_, r)| r == first) => rep.identical += 1,
            Some(_) => rep.binding_splits.push((fx.clone(), lat.clone())),
            None => {}
        }

        // FYI identity among the FYI-capable present surfaces (secondary).
        let fyis: Vec<BTreeSet<String>> = FYI_CAPABLE
            .iter()
            .filter_map(|s| idx.get(*s).and_then(|m| m.get(fx)).map(fyi_set))
            .collect();
        if let Some(first) = fyis.first() {
            if !fyis.iter().all(|s| s == first) {
                rep.fyi_splits.push(fx.clone());
            }
        }

        // (2) python-ags4 leg via classify (verbatim).
        if let (Some(can), Some(pyf)) = (canonical, idx.get(REFERENCE).and_then(|m| m.get(fx))) {
            if let Some(cf) = idx[can].get(fx) {
                let verdict = classify(&floor_result(cf), &py_floor(pyf));
                match &verdict {
                    Parity::Agree => rep.py_agree += 1,
                    Parity::KnownDivergence { observation, .. } => {
                        rep.py_known.push((fx.clone(), observation.clone()));
                    }
                    Parity::PythonError { .. } => rep.py_error += 1,
                    _ => rep.actions.push((fx.clone(), verdict)),
                }
            }
        }
    }
    rep
}

fn load(dir: &PathBuf) -> Result<BTreeMap<String, SurfaceFile>, String> {
    let mut out = BTreeMap::new();
    let rd = std::fs::read_dir(dir).map_err(|e| format!("read {}: {e}", dir.display()))?;
    for entry in rd {
        let p = entry.map_err(|e| e.to_string())?.path();
        if p.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        // `*-parse.json` is the duckdb read/parse-agreement artifact (#458),
        // a different schema read by the `duckdb-parse-check` bin — never a
        // findings surface, so skip it here or it'd fail to deserialize.
        if p.file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|n| n.ends_with("-parse.json"))
        {
            continue;
        }
        let f = std::fs::File::open(&p).map_err(|e| format!("open {}: {e}", p.display()))?;
        let sf: SurfaceFile =
            serde_json::from_reader(f).map_err(|e| format!("parse {}: {e}", p.display()))?;
        out.insert(sf.surface.clone(), sf);
    }
    Ok(out)
}

fn main() {
    // FATAL BY DEFAULT, opt OUT explicitly — the polarity matters. An
    // opt-IN `--require-surfaces` would leave the gate advisory unless someone
    // remembered the flag, which is the RC-5 shape this change exists to remove:
    // the default must be the guarantee, and relaxing it must be a visible act.
    // For a local run of one surface, not for CI.
    let allow_missing = std::env::args().any(|a| a == "--allow-missing-surfaces");
    let dir = std::env::args()
        .nth(1)
        .filter(|a| !a.starts_with("--"))
        .map_or_else(|| PathBuf::from("output/compliance-results"), PathBuf::from);
    let surfaces = match load(&dir) {
        Ok(s) if !s.is_empty() => s,
        Ok(_) => {
            eprintln!("no <surface>.json found in {}", dir.display());
            std::process::exit(3);
        }
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(3);
        }
    };
    let idx: Index = surfaces
        .iter()
        .map(|(s, sf)| {
            (
                s.clone(),
                sf.results
                    .iter()
                    .map(|r| (r.fixture.clone(), r.clone()))
                    .collect(),
            )
        })
        .collect();
    let fixtures: BTreeSet<String> = idx.values().flat_map(|m| m.keys().cloned()).collect();
    let rep = compare(&idx, &fixtures);

    let present_lat: Vec<&str> = LATERITE
        .iter()
        .copied()
        .filter(|s| idx.contains_key(*s))
        .collect();
    println!("# laterite-ags4-compliance — cross-surface findings agreement\n");
    println!("surfaces present:");
    for (name, sf) in &surfaces {
        let v = sf.version.as_deref().unwrap_or("?");
        let sha = sf.engine_sha.as_deref().unwrap_or("-");
        println!(
            "  - {name:<16} v{v}  engine_sha={sha}  ({} fixtures)",
            sf.results.len()
        );
    }
    println!(
        "\nlaterite surfaces compared: {} ({})",
        present_lat.len(),
        present_lat.join(", ")
    );
    println!(
        "4-laterite floor identical: {}/{}",
        rep.identical, rep.fixtures
    );
    if !rep.missing_surfaces.is_empty() {
        println!(
            "\n!! MISSING SURFACES ({}) — these produced no results at all: {}",
            rep.missing_surfaces.len(),
            rep.missing_surfaces.join(", ")
        );
        println!(
            "   The invariant is FOUR-laterite identity. Comparing the survivors and\n\
                reporting OK would be agreement by attrition — the surfaces that did\n\
                report cannot vouch for the one that didn't."
        );
    }
    if !rep.partial_fixtures.is_empty() {
        println!(
            "\n!! PARTIAL COVERAGE ({} fixtures) — a present surface skipped them:",
            rep.partial_fixtures.len()
        );
        for (fx, silent) in rep.partial_fixtures.iter().take(10) {
            println!("   {fx}  silent: {}", silent.join(", "));
        }
        if rep.partial_fixtures.len() > 10 {
            println!("   … +{} more", rep.partial_fixtures.len() - 10);
        }
        println!(
            "   A fixture only ONE surface reported is vacuously 'identical' — the\n\
                comparator agreeing with itself."
        );
    }
    if allow_missing && (!rep.missing_surfaces.is_empty() || !rep.partial_fixtures.is_empty()) {
        println!("\n(--allow-missing-surfaces: the above is NOT failing this run.)");
    }

    // The whole invariant rests on "same engine, so the floor must match". The
    // harness collected each surface's VERSION, printed it, and never compared
    // them — the same computed-then-ignored shape as fyi_splits, in the same file.
    // It showed `wasm v0.5.1` beside three `v0.7.0` surfaces and still called the
    // result 4-laterite identity. If the versions disagree, the premise is false
    // and "identical" means nothing: four surfaces agreeing is only evidence when
    // they are four builds of the same thing. (#556)
    let versions: Vec<(&str, &str)> = LATERITE
        .iter()
        .filter_map(|s| {
            surfaces
                .get(*s)
                .map(|sf| (*s, sf.version.as_deref().unwrap_or("?")))
        })
        .collect();
    let version_split = versions
        .iter()
        .any(|(_, v)| Some(*v) != versions.first().map(|(_, v0)| *v0));
    if version_split {
        println!("\n!! VERSION SPLIT — the laterite surfaces are not the same build:");
        for (name, v) in &versions {
            println!("   {name:<16} v{v}");
        }
        println!(
            "   'Same engine, so the floor must match' is the premise of this gate.\n\
                Four surfaces agreeing is evidence only if they are four builds of\n\
                the SAME thing."
        );
    }
    if !rep.binding_splits.is_empty() {
        println!(
            "\n!! BINDING SPLITS ({}) — laterite surfaces disagree on the finding floor:",
            rep.binding_splits.len()
        );
        for (fx, surfs) in &rep.binding_splits {
            println!("   {fx}");
            for (s, r) in surfs {
                match r {
                    TupleFloor::HardError(e) => println!("      {s:<16} HARD_ERROR {e}"),
                    TupleFloor::Findings(fs) => {
                        println!("      {s:<16} {} finding(s)", fs.len());
                        for t in fs {
                            let line = t.line.map_or_else(|| "-".into(), |l| l.to_string());
                            let fi = t
                                .field_index
                                .map(|i| format!(" field[{i}]"))
                                .unwrap_or_default();
                            println!("         {} L{line} {}{fi}  {}", t.rule, t.group, t.desc);
                        }
                    }
                }
            }
        }
    }
    if !rep.fyi_splits.is_empty() {
        println!(
            "\n!! FYI splits ({}): {}",
            rep.fyi_splits.len(),
            rep.fyi_splits.join(", ")
        );
    }
    if idx.contains_key(REFERENCE) {
        println!(
            "\npython-ags4: AGREE {} | KNOWN_DIVERGENCE {} | ACTION {} | PYTHON_ERROR {}",
            rep.py_agree,
            rep.py_known.len(),
            rep.actions.len(),
            rep.py_error
        );
        if !rep.py_known.is_empty() {
            let mut by_obs: BTreeMap<String, usize> = BTreeMap::new();
            for (_, o) in &rep.py_known {
                *by_obs.entry(o.clone()).or_default() += 1;
            }
            println!("  reconciled: {by_obs:?}");
        }
        if !rep.actions.is_empty() {
            println!(
                "\n!! python-ags4 ACTIONS ({}) — undocumented divergences:",
                rep.actions.len()
            );
            for (fx, v) in &rep.actions {
                println!("   {fx}  {}", v.tag());
            }
        }
    }

    // #556 (RC-5, advisory by default): this line used to read
    //     rep.binding_splits.is_empty() && rep.actions.is_empty()
    // while the harness ALSO computed fyi_splits (detected, printed, unit-tested by
    // a function named `..._is_flagged`, and called "a binding FYI bug" in its own
    // test) and silently tolerated a surface that never reported. Three answers
    // computed correctly and then not acted on. `git log -S"fyi_splits"` shows the
    // exclusion was never recorded as a decision either way — it was neither
    // enforced nor justified, which is how it survived: nobody had to defend it.
    let missing = !rep.missing_surfaces.is_empty() || !rep.partial_fixtures.is_empty();
    let ok = rep.binding_splits.is_empty()
        && rep.actions.is_empty()
        && rep.fyi_splits.is_empty()
        && !version_split
        && (!missing || allow_missing);
    println!(
        "\n{}",
        if ok {
            "COMPLIANCE OK"
        } else {
            "COMPLIANCE FAIL (binding split, FYI split, version split, missing surface, or undocumented divergence)"
        }
    );
    std::process::exit(i32::from(!ok));
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a fixture whose tuple floor mirrors its label list — one
    /// placeholder tuple per rule, so the tuple-based 4-laterite check sees the
    /// same shape the label list describes. Tests that care about tuple CONTENT
    /// (location, count) beyond the label build `FixtureFindings` directly (see
    /// `same_labels_different_location_is_a_binding_split`).
    fn ff(fixture: &str, rules: &[&str], fyi: &[&str]) -> FixtureFindings {
        let findings = rules
            .iter()
            .map(|r| FindingTuple {
                rule: r.to_string(),
                line: None,
                group: String::new(),
                desc: String::new(),
                field_index: None,
            })
            .collect();
        FixtureFindings {
            fixture: fixture.to_string(),
            rules: rules.iter().map(std::string::ToString::to_string).collect(),
            findings,
            fyi: fyi.iter().map(std::string::ToString::to_string).collect(),
            hard_error: None,
            error: None,
        }
    }

    fn idx_of(pairs: &[(&str, FixtureFindings)]) -> Index {
        let mut idx: Index = BTreeMap::new();
        for (surface, f) in pairs {
            idx.entry(surface.to_string())
                .or_default()
                .insert(f.fixture.clone(), f.clone());
        }
        idx
    }

    fn fxs(idx: &Index) -> BTreeSet<String> {
        idx.values().flat_map(|m| m.keys().cloned()).collect()
    }

    /// All four present and reporting the same fixture — the baseline the two
    /// tests below mutate. Without this, "missing is detected" could pass because
    /// the detector fires on everything.
    fn all_four(fixture: &str) -> Vec<(&'static str, FixtureFindings)> {
        LATERITE
            .iter()
            .map(|s| (*s, ff(fixture, &["AGS Format Rule 1"], &[])))
            .collect()
    }

    #[test]
    fn all_four_present_reports_nothing_missing() {
        let idx = idx_of(&all_four("a.ags"));
        let r = compare(&idx, &fxs(&idx));
        assert!(r.missing_surfaces.is_empty(), "{:?}", r.missing_surfaces);
        assert!(r.partial_fixtures.is_empty(), "{:?}", r.partial_fixtures);
        assert_eq!(r.identical, 1);
    }

    #[test]
    fn a_surface_that_never_reported_is_not_agreement() {
        // wasm's runner died. The other three agree with each other, and before
        // #556 that printed COMPLIANCE OK — the gate quietly stopped checking the
        // fourth surface while still calling itself 4-laterite identity.
        let pairs: Vec<_> = all_four("a.ags")
            .into_iter()
            .filter(|(s, _)| *s != "wasm")
            .collect();
        let idx = idx_of(&pairs);
        let r = compare(&idx, &fxs(&idx));
        assert_eq!(r.missing_surfaces, vec!["wasm".to_string()]);
        // The survivors still agree — which is exactly why agreement alone is not
        // a safe exit condition.
        assert!(r.binding_splits.is_empty());
        assert_eq!(r.identical, 1);
    }

    #[test]
    fn a_fixture_only_one_surface_reported_is_not_identical_across_four() {
        // The subtler half: every surface is PRESENT, but only rust reported
        // b.ags. `lat.iter().all()` over one element is vacuously true, so b.ags
        // counted as "identical" — the comparator agreeing with itself.
        let mut pairs = all_four("a.ags");
        pairs.push(("rust", ff("b.ags", &["AGS Format Rule 2"], &[])));
        let idx = idx_of(&pairs);
        let r = compare(&idx, &fxs(&idx));
        assert!(r.missing_surfaces.is_empty());
        assert_eq!(r.partial_fixtures.len(), 1);
        let (fx, silent) = &r.partial_fixtures[0];
        assert_eq!(fx, "b.ags");
        assert_eq!(
            silent,
            &vec![
                "python-laterite".to_string(),
                "node".to_string(),
                "wasm".to_string()
            ]
        );
    }

    #[test]
    fn laterite_surfaces_agree_and_python_agrees() {
        // rust == python-laterite on the floor; python-ags4 same → AGREE.
        let idx = idx_of(&[
            ("rust", ff("a.ags", &["AGS Format Rule 8"], &[])),
            ("python-laterite", ff("a.ags", &["AGS Format Rule 8"], &[])),
            ("python-ags4", ff("a.ags", &["AGS Format Rule 8"], &[])),
        ]);
        let r = compare(&idx, &fxs(&idx));
        assert_eq!(r.identical, 1);
        assert!(r.binding_splits.is_empty());
        assert_eq!(r.py_agree, 1);
        assert!(r.actions.is_empty());
    }

    #[test]
    fn binding_split_is_flagged() {
        // wasm drops a finding the others report → a binding bug.
        let idx = idx_of(&[
            (
                "rust",
                ff("a.ags", &["AGS Format Rule 8", "AGS Format Rule 16"], &[]),
            ),
            (
                "python-laterite",
                ff("a.ags", &["AGS Format Rule 8", "AGS Format Rule 16"], &[]),
            ),
            ("wasm", ff("a.ags", &["AGS Format Rule 8"], &[])),
        ]);
        let r = compare(&idx, &fxs(&idx));
        assert_eq!(r.identical, 0);
        assert_eq!(r.binding_splits.len(), 1);
    }

    #[test]
    fn same_labels_different_location_is_a_binding_split() {
        // The #555 part-1 capability: two surfaces agree on WHICH rule fired
        // (both "AGS Format Rule 8") but disagree on WHERE. The old deduplicated
        // LABEL floor saw {Rule 8} == {Rule 8} and called it identical; the tuple
        // floor sees line 5 != line 6 and splits.
        let mk = |line| FixtureFindings {
            fixture: "a.ags".to_string(),
            rules: vec!["AGS Format Rule 8".to_string()],
            findings: vec![FindingTuple {
                rule: "AGS Format Rule 8".to_string(),
                line: Some(line),
                group: "LOCA".to_string(),
                desc: "bad".to_string(),
                field_index: Some(1),
            }],
            fyi: vec![],
            hard_error: None,
            error: None,
        };
        let idx = idx_of(&[("rust", mk(5)), ("python-laterite", mk(6))]);
        let r = compare(&idx, &fxs(&idx));
        assert_eq!(r.identical, 0);
        assert_eq!(r.binding_splits.len(), 1, "a different line must split");
    }

    #[test]
    fn same_label_different_count_is_a_binding_split() {
        // Rule 8 firing twice on one surface, once on another — identical label
        // SETS, different finding COUNTS. The deduplicated floor collapsed both
        // to {Rule 8}; the sorted MULTISET does not.
        let t = |line| FindingTuple {
            rule: "AGS Format Rule 8".to_string(),
            line: Some(line),
            group: "LOCA".to_string(),
            desc: "bad".to_string(),
            field_index: None,
        };
        let mk = |tuples: Vec<FindingTuple>| FixtureFindings {
            fixture: "a.ags".to_string(),
            rules: vec!["AGS Format Rule 8".to_string()],
            findings: tuples,
            fyi: vec![],
            hard_error: None,
            error: None,
        };
        let idx = idx_of(&[("rust", mk(vec![t(5), t(6)])), ("wasm", mk(vec![t(5)]))]);
        let r = compare(&idx, &fxs(&idx));
        assert_eq!(r.binding_splits.len(), 1, "a different count must split");
    }

    #[test]
    fn o3_unquoted_field_reconciles_not_an_action() {
        // O-3: laterite Rule 5, python-ags4 Rule 4 → KnownDivergence(O-3).
        let idx = idx_of(&[
            ("python-laterite", ff("a.ags", &["AGS Format Rule 5"], &[])),
            ("python-ags4", ff("a.ags", &["AGS Format Rule 4"], &[])),
        ]);
        let r = compare(&idx, &fxs(&idx));
        assert!(r.actions.is_empty(), "{:?}", r.actions);
        assert_eq!(r.py_known, vec![("a.ags".to_string(), "O-3".to_string())]);
    }

    #[test]
    fn fyi_floor_is_symmetric_extended_ascii_agrees() {
        // D1: laterite emits a Rule-1 FYI on extended-ASCII; python-ags4's floor
        // (FYI-stripped by its wrapper) has none. On the rule FLOOR both are
        // Clean → AGREE; the FYI is NOT a python-ags4 action.
        let idx = idx_of(&[
            (
                "python-laterite",
                ff("ext.ags", &[], &["FYI (Related to Rule 1)"]),
            ),
            ("python-ags4", ff("ext.ags", &[], &[])),
        ]);
        let r = compare(&idx, &fxs(&idx));
        assert_eq!(r.py_agree, 1, "extended-ASCII must AGREE on the floor");
        assert!(r.actions.is_empty());
    }

    #[test]
    fn fyi_split_among_capable_surfaces_is_flagged() {
        // rust + python-laterite disagree on the FYI set (a binding FYI bug),
        // even though their rule floor matches.
        let idx = idx_of(&[
            (
                "rust",
                ff(
                    "a.ags",
                    &["AGS Format Rule 8"],
                    &["FYI (Related to Rule 16)"],
                ),
            ),
            ("python-laterite", ff("a.ags", &["AGS Format Rule 8"], &[])),
        ]);
        let r = compare(&idx, &fxs(&idx));
        assert_eq!(r.identical, 1, "rule floor still matches");
        assert_eq!(r.fyi_splits, vec!["a.ags".to_string()]);
    }
}
