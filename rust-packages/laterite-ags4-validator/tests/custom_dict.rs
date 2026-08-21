//! End-to-end coverage for the `--dict` custom-dictionary overlay (laterite-dev#568 Phase 3).
//!
//! Proves the whole arc a client wants: a bespoke group hung off a standard one
//! (`XTRA` off `SAMP`, borrowing the standard `SAMP_ID` KEY) is UNKNOWN to the
//! bundled dictionary — and flagged — but becomes a first-class group once the
//! same delivery is validated against the custom dictionary, in either the `.ags`
//! or JSON spelling of that dictionary.

use std::path::{Path, PathBuf};

use laterite_ags4_validator::{
    CheckOptions, DictResolution, DictVersion, check_file,
    overlay::{self, BaseSpec, DictFormat},
};

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/custom_dict")
        .join(name)
}

fn parse_fixture_dict(name: &str) -> overlay::CustomDict {
    let bytes = std::fs::read(fixture(name)).expect("read dict fixture");
    overlay::parse_dict(
        &bytes,
        DictFormat::Auto,
        CheckOptions::default().encoding,
        BaseSpec::Auto,
        name,
    )
    .expect("dict parses")
}

/// Findings on the delivery that reference the bespoke `XTRA` group.
fn xtra_findings(opts: &CheckOptions) -> usize {
    let found = check_file(&fixture("delivery_with_xtra.ags"), opts).expect("validates");
    found
        .values()
        .flatten()
        .filter(|f| f.group == "XTRA" || f.desc.contains("XTRA"))
        .count()
}

#[test]
fn additive_dict_detects_latest_base_without_replacement() {
    // The B2 regression guard: a purely-additive dict overlays the latest edition
    // and never silently becomes a replacement.
    let d = parse_fixture_dict("xtra.dict.json");
    assert_eq!(d.base_version, DictVersion::V4_2);
    assert_eq!(d.resolution, DictResolution::StructuralBase);
    assert!(d.fall_through, "additive overlay must not discard the base");
}

#[test]
fn ags_and_json_dict_fixtures_converge() {
    let j = parse_fixture_dict("xtra.dict.json");
    let a = parse_fixture_dict("xtra.dict.ags");
    assert_eq!(
        j.hash, a.hash,
        "the .ags and JSON twins are the same dictionary"
    );
}

#[test]
fn overlay_makes_a_bespoke_group_valid() {
    let with = CheckOptions {
        custom_dict: Some(parse_fixture_dict("xtra.dict.json")),
        include_warnings: true,
        include_fyi: true,
        ..Default::default()
    };
    let without = CheckOptions {
        include_warnings: true,
        include_fyi: true,
        ..Default::default()
    };

    let without_n = xtra_findings(&without);
    let with_n = xtra_findings(&with);
    assert!(
        without_n > 0,
        "the bundled dictionary must flag the unknown XTRA group"
    );
    assert_eq!(
        with_n, 0,
        "the overlay makes XTRA a recognised group ({with_n} residual XTRA findings)"
    );
}
