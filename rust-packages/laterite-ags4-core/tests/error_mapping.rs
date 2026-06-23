//! Phase 0 (#168 parser convergence): lock the EXACT `CliError` messages the
//! lean core terminals produce today. Phase 5's `enforce_strict_structure` must
//! reproduce the STRUCTURE messages byte-for-byte (the plan pre-greps these so a
//! string-asserting ags5db/core test doesn't break). The non-UTF-8 message is
//! the `csv` reader's OWN text and WILL change when csv is retired (the shared
//! leaf returns `ParseError::NotUtf8` → a `CliError`), so it's asserted loosely
//! (rejection + a utf-8 mention), not pinned to the csv wording — Phase 7
//! ratifies that one wording change.

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

/// Non-UTF-8 is rejected by the lean path. The exact message is the csv reader's
/// own and is expected to CHANGE when csv is retired; only the rejection + a
/// utf-8 mention are guaranteed.
#[test]
fn non_utf8_is_rejected_message_may_change() {
    let m = schema_msg(b"\xff\xff").to_lowercase();
    assert!(m.contains("utf-8") || m.contains("utf8"), "got {m:?}");
}
