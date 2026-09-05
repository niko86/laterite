//! The wait-state ledger: engine API this crate wants and cannot import yet.
//!
//! This crate is published, and `cargo package --verify` builds its tarball
//! against the **crates.io** versions of its engine deps — path deps are
//! stripped at packaging — so a sibling's NEW API is unusable here until the
//! nightly engine cut publishes it (#929 is the CI run that proved it, #930
//! the standing example). Each row below is one deliberate local copy
//! carrying that wait. The per-file comments point here; this table is the
//! whole inventory, so "we bumped the engine pins" means: work this table,
//! delete each copy, call the shared item, and remove its row — in the SAME
//! PR as the version-req bump.
//!
//! `tools/check_package_contents.py --verify-buildable` is the gate that
//! proves a swap safe BEFORE pushing; a plain `cargo test` proves nothing
//! here, because it sees the path deps this constraint is about.
//!
//! | waiting on (registry) | the local copy | replaced by |
//! |---|---|---|
//! | `laterite-ags4-hostopts` on the registry at all (#947 extracted it; its FIRST publish is a token run) | `ags4/build.rs::staged_write` + `staging_dir` | `laterite_ags4_hostopts::staged_write` (#930) |
//! | `laterite-ags4-core` newer than 0.14.0 | the body of `ags4/mod.rs::read_options` | `ags4_codec::ReadOptions::from_flags` (#930) |
