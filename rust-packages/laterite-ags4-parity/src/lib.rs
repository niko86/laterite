//! `laterite-ags4-parity` — the shared Rust↔python-ags4 parity model.
//!
//! This is the de-duplication of the verdict/oracle logic that lived
//! in `laterite-ags4-corpus-qa/src/parity.rs`: the *same* `classify`/`reconcile`
//! (encoded against the OBSERVATIONS O-2/O-3/O-26/O-30/O-34 arms) and
//! the *same* `PyOracle` subprocess bridge, now consumed by **both**
//! `laterite-ags4-corpus-qa` and `laterite-ags4-forge` instead of one copy + a re-derive
//! (which would drift the clean-room "Rust ≡ python except enumerated
//! O-Ns" claim across two maintenance points). It also hosts the
//! seedable `Rng`/`reservoir` so deterministic sampling is shared.
//!
//! Depends *on* `laterite-ags4-validator`, never the reverse — the validator
//! library's lean dep-graph guarantee is untouched.

pub mod oracle;
pub mod rng;
pub mod verdict;

pub use oracle::{EXPECTED_PYAGS4, OracleError, PyOracle, SelfCheck};
pub use rng::{Rng, reservoir};
pub use verdict::{Parity, RustResult, classify};
