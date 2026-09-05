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
//! | *(none — #930 worked both rows on 2026-09-05: the staged-write body is `laterite_ags4_hostopts::staged_write_io`, the read-options body is `ags4_codec::ReadOptions::from_flags`)* | | |
//!
//! An empty table is still a ledger: the CONSTRAINT above is permanent, so
//! the next time this crate wants engine API the registry does not carry
//! yet, the copy gets a row here first.
