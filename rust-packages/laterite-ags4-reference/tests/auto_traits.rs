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
//! That makes them the one part of the surface where a major-version break is
//! genuinely invisible. Adding a `Cell` to a captured variable inside one of
//! these iterators would drop `Sync` from a consumer's type with no signature
//! change anywhere. So they are asserted here, on real returned values, at
//! compile time.
//!
//! `check_impl_trait_is_asserted` requires every `-> impl` in this crate's
//! snapshot to be named in this file — a new opaque return cannot be added
//! without landing here first.

use laterite_ags4_reference::dict::{Dictionary, FALLBACK};
use laterite_ags4_reference::union::registry;

/// Takes the value by reference so the assertion binds to the type the API
/// actually hands back, not to one written out here — an opaque type cannot be
/// named, which is the whole difficulty.
fn assert_send_sync<T: Send + Sync>(_: &T) {}

/// `Registry::iter` and `GroupDescriptor::key_headings`.
#[test]
fn registry_iterators_are_send_and_sync() {
    let reg = registry();
    assert_send_sync(&reg.iter());

    let group = reg
        .iter()
        .next()
        .expect("the bundled registry is not empty");
    assert_send_sync(&group.key_headings());
}

/// `Dictionary::group_codes` and `Dictionary::all_heading_names`.
///
/// Through `Dictionary::bundled`, which is the `'static` arm: the `layered`
/// arm borrows an `OwnedDelta`, and a borrow of a `Sync` value is `Send`, so
/// the bundled case is the one that can actually regress on its own.
#[test]
fn dictionary_iterators_are_send_and_sync() {
    let dict = Dictionary::bundled(FALLBACK);
    assert_send_sync(&dict.group_codes());
    assert_send_sync(&dict.all_heading_names());
}
