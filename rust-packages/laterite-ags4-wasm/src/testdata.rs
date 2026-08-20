//! Fixtures shared by more than one verb's tests.
//!
//! Here rather than duplicated per module: two copies of an AGS fixture drift,
//! and a test asserting against the stale one still passes.

pub(crate) const CLEAN: &[u8] =
    include_bytes!("../../laterite-ags4-validator/tests/fixtures/clean_minimal.ags");

/// Two groups, one keyed child, one heading typed `2DP` — enough to exercise
/// edition resolution, KEY matching and a type clash.
pub(crate) const LOCA_A: &[u8] = b"\"GROUP\",\"PROJ\"\r\n\
\"HEADING\",\"PROJ_ID\"\r\n\
\"UNIT\",\"\"\r\n\
\"TYPE\",\"ID\"\r\n\
\"DATA\",\"P1\"\r\n\
\r\n\
\"GROUP\",\"LOCA\"\r\n\
\"HEADING\",\"LOCA_ID\",\"LOCA_NATE\"\r\n\
\"UNIT\",\"\",\"m\"\r\n\
\"TYPE\",\"ID\",\"2DP\"\r\n\
\"DATA\",\"BH01\",\"100.00\"\r\n\
\"DATA\",\"BH02\",\"200.00\"\r\n";

/// `LOCA_A` with BH01 moved, BH02 gone and BH03 new — one of each verdict.
#[cfg(any(feature = "diff", feature = "merge"))]
pub(crate) const LOCA_B: &[u8] = b"\"GROUP\",\"PROJ\"\r\n\
\"HEADING\",\"PROJ_ID\"\r\n\
\"UNIT\",\"\"\r\n\
\"TYPE\",\"ID\"\r\n\
\"DATA\",\"P1\"\r\n\
\r\n\
\"GROUP\",\"LOCA\"\r\n\
\"HEADING\",\"LOCA_ID\",\"LOCA_NATE\"\r\n\
\"UNIT\",\"\",\"m\"\r\n\
\"TYPE\",\"ID\",\"2DP\"\r\n\
\"DATA\",\"BH01\",\"999.00\"\r\n\
\"DATA\",\"BH03\",\"300.00\"\r\n";

pub(crate) fn err(r: Result<impl Sized, String>) -> String {
    match r {
        Ok(_) => panic!("expected an error, got Ok"),
        Err(m) => m,
    }
}
