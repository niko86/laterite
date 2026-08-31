//! The `from_shared` trim projection — the one surviving assertion from the old
//! `parse_parity.rs` two-parser gate (#168 Phase 7).
//!
//! Since the parser convergence (#168), core's `ags4_codec` and the validator's
//! `parse` are the SAME shared leaf ([`laterite_ags4_parse`]): the validator
//! re-exports it, and core's read path is `from_shared(...)` over it. So the old
//! gate's "the two parsers agree on structure" is now tautological (one parser) —
//! it was retired here.
//!
//! What is NOT tautological, and is why this file remains: `from_shared` **re-trims**
//! each field to stay byte-identical to core's pre-convergence output, while the
//! leaf (and thus the validator) keeps values **verbatim**. That is #168 fork 1 —
//! the trim lives in core's projection, not the shared parse — and this asserts it
//! holds, so a future refactor can't silently move (or drop) the trim.

use laterite_ags4_core::ags4_codec::read_ags4_bytes;
use laterite_ags4_validator::parse::parse_str;

// A single DATA value with interior whitespace inside the quotes: `"  P1  "`.
// core's `from_shared` trims it to `P1`; the shared leaf keeps `  P1  `.
const INTERIOR_WS: &str = concat!(
    "\"GROUP\",\"PROJ\"\r\n",
    "\"HEADING\",\"PROJ_ID\"\r\n",
    "\"UNIT\",\"\"\r\n",
    "\"TYPE\",\"ID\"\r\n",
    "\"DATA\",\"  P1  \"\r\n",
);

#[test]
fn from_shared_trims_where_the_leaf_keeps_verbatim() {
    let core = read_ags4_bytes(INTERIOR_WS.as_bytes()).unwrap();
    let val = parse_str(INTERIOR_WS).unwrap();
    let cv = core.get("PROJ").unwrap().rows[0]
        .get("PROJ_ID")
        .map(String::as_str);
    let vv = val.groups.get("PROJ").unwrap().cell(0, 0);
    assert_eq!(
        cv,
        Some("P1"),
        "core's from_shared trims interior-quoted whitespace"
    );
    assert_eq!(vv, Some("  P1  "), "the shared leaf preserves it verbatim");
    assert_ne!(
        cv, vv,
        "the trim lives in core's projection, not the shared parse (#168 fork 1)"
    );
}
