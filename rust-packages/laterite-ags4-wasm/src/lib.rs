//! Browser wasm wrapper around the clean-room AGS4 validator.
//!
//! `validate()` replicates the body of `laterite_ags4_validator::check_file_with_dict`
//! (`lib.rs`) but from in-memory bytes with `source = None`, so it runs
//! the entire rule engine **client-side** with no filesystem and nothing
//! uploaded. Rule *violations* come back as data in the report; only
//! un-validatable inputs (not AGS4, unsupported edition, bad arguments)
//! populate `report.error` — nothing throws across the wasm boundary.
//!
//! `read()` returns typed Arrow IPC for the DuckDB-wasm data explorer; the
//! crate began as the validator wrapper alone and grew that surface after.
//!
//! # What is always here, and what is a feature (#330)
//!
//! Six surfaces are behind cargo features — `excel`, `arrow`, `certify`,
//! `diff`, `merge`, `censor` — all ON by `default = full`, so a plain
//! `wasm-pack build` is unchanged. Turning them off is what the published slim
//! artifact does, and it is worth megabytes: our own Rust is ~7.6% of the
//! binary, `arrow` alone ~35% of its code section.
//!
//! **Ungated, therefore the slim surface**: `validate`, `read` (+
//! `group_codes` / `meta` / `rows_json`), `build_ags4`, `compute_fixes` /
//! `apply_fixes`, `list_rules`, `dictionary`, `version` / `engine_version` /
//! `engine_fingerprint`. Read → validate → fix → write is one chain and no
//! shipped build breaks it in the middle.
//!
//! The gates take exports away; they never change what a surviving one
//! *returns*. The one asymmetry is that `read`'s two row doors are not both
//! always present — see [`ParsedDataset::rows_json`].
//!
//! # Layout
//!
//! One module per verb — `validate`, `build`, `read`, `diff`, `merge`, … — each
//! carrying the tests for the code beside it, with the two shared leaves
//! (`resolve`, `boundary`) holding what no single verb owns. The root is
//! declarations and re-exports: `#[wasm_bindgen]` exports are flat in the
//! generated JS whatever module they were written in, so this shape is
//! invisible to a consumer.
//!
//! # Why the cores are extracted
//!
//! Almost every export here names a JS type — `JsValue`, `JsError`, one of the
//! `*Js` aliases — and this crate has no `wasm-bindgen-test` lane, so `cargo
//! test` cannot call it at all. (The exceptions are `metadata`'s four plain-Rust
//! identity doors, which is why they are tested end to end.) That is a measurement problem only if the
//! exports are thin. They were not: option folding, edition resolution, the
//! encoding decision, the leak-safe censor default and four separate "return
//! nothing rather than the wrong thing" arms all lived inside signatures no test
//! could enter. So each door is `decode → core → marshal`, the core is plain
//! Rust, and the tests hold the core. What is left at the boundary is covered by
//! the `wasm-engine` xcheck leg instead.

/// Declare a block of TypeScript for the generated `.d.ts`, keeping a plain
/// `const` copy the tests can read.
///
/// Both are needed, and the duplication is only apparent. wasm-bindgen's
/// `typescript_custom_section` **consumes** the item it decorates — the const is
/// gone by the time the rest of the crate compiles — and its parser matches on
/// `syn::Lit::Str`, so it will not accept a reference to a const defined
/// elsewhere. Emitting the same literal token into both positions from one
/// `$src` is what lets `ts_result_shape_tests` read the exact string that ships,
/// rather than a second copy that could drift from it.
///
/// The readable copy is `#[cfg(test)]`: nothing but the tests reads it, so
/// outside them it is dead weight in a binary whose whole point is being small.
///
/// The attribute list applies to **both** items, which matters now that six
/// surfaces are feature-gated (#330): a `#[cfg(feature = "diff")]` that reached
/// only the test const would leave the shipped `.d.ts` declaring `RevisionDelta`
/// in a build with no `diff` export — the generated types are the published API
/// reference, so that is a lie the compiler would never catch.
macro_rules! ts_section {
    ($(#[$meta:meta])* $name:ident, $section:ident, $src:literal) => {
        $(#[$meta])*
        #[cfg(test)]
        pub(crate) const $name: &str = $src;

        $(#[$meta])*
        #[wasm_bindgen(typescript_custom_section)]
        const $section: &'static str = $src;
    };
}

// The modules come AFTER `ts_section!` deliberately: `macro_rules!` is scoped
// textually, so a module declared above the macro cannot see it.
//
// The feature gates are on the `mod` lines as well as on the items inside. The
// inline gates are the real record of what each feature takes away and stay
// exactly where they were written; the gate here is what keeps a module's own
// `use` lines honest when the feature is off, with no import left over to warn
// about under `-D warnings`.
mod boundary;
mod build;
#[cfg(feature = "censor")]
mod censor;
#[cfg(feature = "certify")]
mod certify;
mod dictionary;
#[cfg(feature = "diff")]
mod diff;
#[cfg(feature = "excel")]
mod excel;
mod fixes;
#[cfg(feature = "merge")]
mod merge;
mod metadata;
mod read;
mod resolve;
mod validate;

// The published surface, flat at the crate root — the shape it had when this
// was one file, and the shape `wasm-pack` emits either way.
pub use build::*;
#[cfg(feature = "censor")]
pub use censor::*;
#[cfg(feature = "certify")]
pub use certify::*;
pub use dictionary::*;
#[cfg(feature = "diff")]
pub use diff::*;
#[cfg(feature = "excel")]
pub use excel::*;
pub use fixes::*;
#[cfg(feature = "merge")]
pub use merge::*;
pub use metadata::*;
pub use read::*;
pub use validate::*;

// Fixtures more than one verb's tests read.
#[cfg(test)]
mod testdata;

// The two crate-WIDE test modules, which no verb can own: one holds every
// published TS interface to the struct it claims to describe, the other holds
// the whole crate to a single serializer.
//
// Named without a `_tests` suffix on purpose: `cargo llvm-cov` skips files
// matching that pattern, and both were measured like any other code while they
// lived in `lib.rs`. A `_tests.rs` name here would quietly take them out of the
// crate's coverage denominator — changing what the nightly floor means, by
// filename.
#[cfg(test)]
mod serializer_consistency;
#[cfg(test)]
mod ts_result_shape;
