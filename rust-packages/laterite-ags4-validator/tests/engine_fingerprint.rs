//! The two facts about `ENGINE_FINGERPRINT` that no other test in the tree can
//! state (#550).
//!
//! Why they live *here*, of all places: `decide()` is `laterite-ags4-core`'s, and
//! core takes the fingerprint as a **parameter** — it cannot see the real one, so
//! every fingerprint test it owns necessarily *fabricates* a value
//! (`"ffffffffffffffff"`, `"0000deadbeef0000"`). Meanwhile this crate *defines*
//! the real fingerprint but never touches a certificate. Each crate holds half the
//! fact and neither can assert it. This file is the one place both are visible —
//! core is a dev-dep here — which is exactly why the gap survived: it wasn't
//! overlooked, it was **unreachable from either side**.
//!
//! Together they close a gap that shape-only assertions leave wide open. The
//! existing tests prove the fingerprint LOOKS right (16 lowercase hex, != VERSION)
//! and that COMPARING two fingerprints works. Neither constrains **what goes into
//! one** — a digest over an empty file list is still 16 lowercase hex chars, and
//! still compares fine.

use std::path::PathBuf;

use laterite_ags4_core::index::{Decision, EngineId, Question, RevalidateReason, Sidecar};
use laterite_ags4_validator::ENGINE_FINGERPRINT;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

/// The engine identity of THIS build — the same one every surface stamps.
fn current_engine() -> EngineId {
    EngineId {
        validator: "laterite_ags4".to_string(),
        compat: None,
        fingerprint: ENGINE_FINGERPRINT.to_string(),
    }
}

/// The question the golden cert was minted under: a plain `lat certify` — errors
/// only, no forced edition, UTF-8.
fn as_minted() -> Question {
    Question {
        want_warnings: false,
        want_fyi: false,
        forced_edition: None,
        encoding: "UTF-8".to_string(),
        custom_dict: None,
    }
}

/// A REAL certificate, minted by a REAL older engine, must not be vouched for.
///
/// `engine_golden.ags.idx` was minted by the pre-#550 engine (fingerprint
/// `5f725e53c36f7074`) — not hand-written, not fabricated: `lat certify` produced
/// it, and that value is independently reproducible from that engine's file list.
/// #550 widened the covered set, so the fingerprint moved and this cert became the
/// artefact of a genuinely older engine. It stays one forever: returning to that
/// exact digest would mean reverting the coverage AND every covered byte.
///
/// This is the first assertion in the tree that spans two versions of ourselves.
/// Every other cert test is a conversation between one build and itself.
#[test]
fn a_cert_minted_by_an_older_engine_is_not_vouched() {
    let bytes = std::fs::read(fixture("engine_golden.ags")).expect("golden source");
    let cert = std::fs::read(fixture("engine_golden.ags.idx")).expect("golden cert");
    let sidecar = Sidecar::from_json(&cert).expect("the golden cert still parses");

    // Guard the guard: if the source bytes ever drift, `decide` short-circuits on
    // SizeChanged/ContentChanged BEFORE it ever compares engines, and this test
    // would pass while testing nothing. That is the failure mode this whole file
    // exists to prevent, so it must not be the failure mode of the file itself.
    assert_eq!(
        sidecar.file.size,
        bytes.len() as u64,
        "engine_golden.ags changed — re-mint the cert or this test stops reaching the engine check"
    );

    let decision = sidecar.decide(&bytes, &as_minted(), &current_engine());

    assert_eq!(
        decision,
        Decision::Revalidate(RevalidateReason::DifferentEngine),
        "a cert from the pre-#550 engine ({}) must not be trusted by this engine ({ENGINE_FINGERPRINT})",
        sidecar.validation.engine,
    );
}

/// The same cert IS vouched for by the engine that minted it — so the test above
/// is failing for the reason it claims, not because the fixture is broken in some
/// other way.
///
/// Without this, `a_cert_minted_by_an_older_engine_is_not_vouched` passes just as
/// happily against a corrupt cert, a mismatched question, or a `decide()` that
/// returns `Revalidate` unconditionally.
#[test]
fn the_golden_cert_is_vouched_by_the_engine_that_minted_it() {
    let bytes = std::fs::read(fixture("engine_golden.ags")).expect("golden source");
    let cert = std::fs::read(fixture("engine_golden.ags.idx")).expect("golden cert");
    let sidecar = Sidecar::from_json(&cert).expect("the golden cert still parses");

    let minting_engine = EngineId {
        validator: "laterite_ags4".to_string(),
        compat: None,
        fingerprint: sidecar.validation.engine.clone(),
    };

    assert_eq!(
        sidecar.decide(&bytes, &as_minted(), &minting_engine),
        Decision::Vouched,
        "the golden cert must be a VALID cert that only the engine change rejects",
    );
}

/// The fingerprint must cover every crate the verdict is expressed through — not
/// just the crate that hosts the rules.
///
/// The floor is hand-written on purpose, and that is not a contradiction of #550's
/// "derive, don't list". `build.rs` DERIVES the set; this pins the MINIMUM that set
/// must contain, with the reason each entry decides a verdict. Narrow the
/// derivation and this fails. A derivation with nothing holding it to a floor is
/// how the fingerprint came to cover three-quarters of its own engine while looking
/// entirely healthy.
#[test]
fn the_fingerprint_covers_every_crate_the_verdict_runs_through() {
    let covered: Vec<&str> = env!("LATERITE_ENGINE_FINGERPRINT_FILES")
        .split(';')
        .collect();

    // (file, why it decides a verdict)
    let must_cover = [
        (
            "laterite-ags4-types/src/lib.rs",
            "owns format_nsf — the formatter that COMPUTES Rule 8's verdict (#528 routed \
             the validator's hand-port through it)",
        ),
        (
            "laterite-ags4-parse/src/lib.rs",
            "the shared tokenizer — it DECIDES field boundaries, and this crate's parse.rs \
             is a `pub use` over it (#168)",
        ),
        (
            "laterite-ags4-reference/src/dict.rs",
            "the dictionary a file is judged against",
        ),
        (
            "laterite-ags4-reference/src/union.rs",
            "projects the multi-edition union — which headings are standard in an edition",
        ),
        (
            "laterite-ags4-reference/build.rs",
            "GENERATES the per-edition phf tables: hashing the dictionary JSON while leaving \
             the code that projects it uncovered was the original hole",
        ),
        (
            "laterite-ags4-reference/data/ags_dictionary.json",
            "the dictionary itself",
        ),
        (
            "laterite-ags4-validator/src/lib.rs",
            "hosts check_parsed + the edition-resolution policy",
        ),
        (
            "laterite-ags4-validator/src/verdict.rs",
            "decides which tiers are fatal, and therefore whether a file PASSES — the \
             hand-written list in build.rs did not grow when this module arrived (#321), \
             which is exactly how the list came to miss three-quarters of the engine before",
        ),
    ];

    for (file, why) in must_cover {
        assert!(
            covered.contains(&file),
            "ENGINE_FINGERPRINT does not cover {file} — {why}.\nA cert minted before a change \
             to it would still read Vouched.\nCovered set ({}): {}",
            covered.len(),
            covered.join(", "),
        );
    }

    // The rules themselves, in bulk — the set must not silently collapse to a stub.
    let rules = covered
        .iter()
        .filter(|f| f.starts_with("laterite-ags4-validator/src/rules/"))
        .count();
    assert!(
        rules >= 5,
        "only {rules} rule source(s) covered — the walk over src/rules is not finding them",
    );
}

/// A dev-dependency cannot change a verdict, so following one would only invalidate
/// certificates for no reason — and `laterite-ags4-core` is a dev-dep of this crate,
/// so following it would walk back into a crate that depends on this one.
#[test]
fn the_covered_set_stops_at_dev_dependencies() {
    let covered: Vec<&str> = env!("LATERITE_ENGINE_FINGERPRINT_FILES")
        .split(';')
        .collect();
    assert!(
        !covered.iter().any(|f| f.starts_with("laterite-ags4-core/")),
        "laterite-ags4-core is a DEV-dep — it cannot reach a verdict and must not be covered",
    );
}
