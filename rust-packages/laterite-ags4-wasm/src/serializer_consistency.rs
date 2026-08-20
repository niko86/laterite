//! One serializer for the whole crate, asserted against the source.
//!
//! serde-wasm-bindgen ships two that differ in what an absent `Option`
//! becomes: the default writes `undefined`, `json_compatible()` writes
//! `null`. Both are reachable, neither is wrong, and choosing the wrong one
//! is invisible from Rust — it surfaces only as a `=== null` check that never
//! fires in a browser, against a published `.d.ts` that promised `null`.
//! That is precisely what `build_ags4` and `build_ags4_ipc` did.
//!
//! `cargo test` cannot inspect the JS value — that is the boundary — so the
//! invariant is enforced where it IS visible: no source in the crate may name
//! a second serializer.

/// Every module source, so the scan cannot narrow when code moves between
/// them. This file is deliberately absent: it is the one place the banned
/// spellings are allowed to appear, and leaving it out is what lets the counts
/// below be a flat zero rather than an arithmetic exclusion.
///
/// Held to the crate's real module list by [`assert_every_module_is_scanned`] —
/// a new module that nobody adds here would otherwise silently leave its
/// serializers unexamined, which is the failure this list can have.
const SRCS: &[(&str, &str)] = &[
    ("lib.rs", include_str!("lib.rs")),
    ("boundary.rs", include_str!("boundary.rs")),
    ("build.rs", include_str!("build.rs")),
    ("censor.rs", include_str!("censor.rs")),
    ("certify.rs", include_str!("certify.rs")),
    ("dictionary.rs", include_str!("dictionary.rs")),
    ("diff.rs", include_str!("diff.rs")),
    ("excel.rs", include_str!("excel.rs")),
    ("fixes.rs", include_str!("fixes.rs")),
    ("merge.rs", include_str!("merge.rs")),
    ("metadata.rs", include_str!("metadata.rs")),
    ("read.rs", include_str!("read.rs")),
    ("resolve.rs", include_str!("resolve.rs")),
    ("testdata.rs", include_str!("testdata.rs")),
    ("ts_result_shape.rs", include_str!("ts_result_shape.rs")),
    ("validate.rs", include_str!("validate.rs")),
];

/// Every `mod` the crate root declares is in [`SRCS`].
fn assert_every_module_is_scanned() {
    for line in include_str!("lib.rs").lines() {
        let Some(name) = line.strip_prefix("mod ").and_then(|r| r.strip_suffix(';')) else {
            continue;
        };
        if name == "serializer_consistency" {
            continue; // this file — see SRCS
        }
        let file = format!("{name}.rs");
        assert!(
            SRCS.iter().any(|(f, _)| *f == file),
            "module {file} is not in SRCS, so its serializers go unscanned"
        );
    }
}

#[test]
fn every_serialisation_goes_through_the_json_compatible_serializer() {
    assert_every_module_is_scanned();
    // The default serializer, reached either by constructing it directly or
    // via the free function that wraps it. Neither may appear at all.
    for banned in ["serde_wasm_bindgen::to_value(", "Serializer::new()"] {
        for (file, src) in SRCS {
            assert_eq!(
                src.match_indices(banned).count(),
                0,
                "{banned:?} in {file} bypasses `to_js`: it writes `undefined` for an absent \
                 Option where this crate's published .d.ts promises `null`. \
                 Serialise through `to_js` instead."
            );
        }
    }
}

#[test]
fn the_build_doors_serialise_through_to_js() {
    // Belt to the above's braces, and the more direct statement: these two
    // are the doors that regressed, so name them.
    for door in ["pub fn build_ags4(", "pub fn build_ags4_ipc("] {
        let (_, src, at) = SRCS
            .iter()
            .find_map(|(f, s)| s.find(door).map(|at| (f, *s, at)))
            .unwrap_or_else(|| panic!("{door} exists"));
        let body_end = src[at..].find("\n}\n").expect("the function ends");
        assert!(
            src[at..at + body_end].contains("to_js(&report)"),
            "{door} no longer serialises through to_js"
        );
    }
}

#[test]
fn the_published_ts_still_declares_the_nullable_field_that_caught_this() {
    // `line` is `Option<u32>` on both EmitFinding and AppliedFix and is
    // declared `number | null`. If that declaration ever changes, the
    // serializer choice has to be revisited in the same breath — so fail
    // loudly rather than let the two drift apart again.
    assert_eq!(
        crate::build::TS_BUILD_RESULT
            .matches("line: number | null")
            .count(),
        2,
        "EmitFinding and AppliedFix should each declare a nullable line"
    );
}
