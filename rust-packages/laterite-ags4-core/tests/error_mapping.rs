//! Lock the EXACT `CliError` messages the lean core terminals produce (#168
//! parser convergence). The STRUCTURE messages survived the convergence
//! byte-for-byte (Phase 5's `enforce_strict_structure` reproduces them). The
//! non-UTF-8 message was the retired `csv` reader's own text; it is now the shared
//! leaf's `ParseError::NotUtf8` wording, **pinned here at #168 Phase 7 (O-46)** —
//! the lean read path rejects non-UTF-8, where the validator decodes it lossily
//! and flags Rule 1 (O-32).

use laterite_ags4_core::ags4_codec::read_ags4_bytes;
use laterite_ags4_core::error::CliError;

fn schema_msg(bytes: &[u8]) -> String {
    match read_ags4_bytes(bytes) {
        Err(CliError::Schema(m)) => m,
        Err(e) => panic!("expected CliError::Schema, got {e:?}"),
        Ok(_) => panic!("expected an error, got Ok"),
    }
}

/// Structure terminals — these messages MUST survive the convergence verbatim.
#[test]
fn structure_error_messages_locked() {
    assert_eq!(
        schema_msg(b"\"HEADING\",\"X\"\n"),
        "HEADING row before any GROUP"
    );
    assert_eq!(schema_msg(b"\"DATA\",\"X\"\n"), "DATA row before any GROUP");
    assert_eq!(schema_msg(b"\"GROUP\"\n"), "GROUP row missing group code");
}

/// Non-UTF-8 is rejected by the lean read path with the shared leaf's exact
/// wording — pinned at #168 Phase 7 (was asserted loosely while the convergence
/// settled the message from the csv reader's own text). The validator path does
/// NOT reject; it decodes lossily and reports Rule 1 (O-32 / O-46).
#[test]
fn non_utf8_is_rejected_with_pinned_message() {
    assert_eq!(schema_msg(b"\xff\xff"), "input is not valid UTF-8");
}
