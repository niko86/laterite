//! `Send`/`Sync` for the opaque types this crate returns.
//!
//! The public-API snapshot (`tools/release/public-api/`) already carries the
//! auto-trait impls of every NAMED public type, and
//! `tools/check_public_api.py` fails the build if one loses `Send` or `Sync`.
//! It cannot do that for `-> impl Trait`: rustdoc renders the declared bounds
//! and stops, so an opaque return type's auto traits appear nowhere in the
//! rendering — while leaking to the consumer exactly as if they had been
//! written down.
//!
//! `check_impl_trait_is_asserted` requires every `-> impl` in this crate's
//! snapshot to be named in this file — a new opaque return cannot be added
//! without landing here first.

use laterite_ags4_core::ags4_codec::AgsGroup;

/// Takes the value by reference so the assertion binds to the type the API
/// actually hands back, not to one written out here — an opaque type cannot be
/// named, which is the whole difficulty.
fn assert_send_sync<T: Send + Sync>(_: &T) {}

/// `AgsGroup::row_cells`. The iterator is one type across both internal
/// representations (span-backed and owned), so binding it on either arm pins
/// both; `from_owned_rows` is the public door that needs no fixture.
#[test]
fn row_cells_is_send_and_sync() {
    let g = AgsGroup::from_owned_rows(
        "PROJ".into(),
        vec!["PROJ_ID".into()],
        vec!["".into()],
        vec!["ID".into()],
        vec![vec!["P1".into()]],
    );
    assert_send_sync(&g.row_cells(0));
}
