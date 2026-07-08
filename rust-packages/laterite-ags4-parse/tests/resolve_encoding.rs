//! The shared encoding-label resolver — the single source of truth every
//! surface uses, so a label means the same thing on Python, Node and the
//! browser. Guards the `latin-1` divergence that motivated it.

use encoding_rs::{UTF_8, WINDOWS_1252};
use laterite_ags4_parse::resolve_encoding;

#[test]
fn none_and_empty_and_utf8_are_utf8() {
    assert_eq!(resolve_encoding(None), Some(UTF_8));
    assert_eq!(resolve_encoding(Some("")), Some(UTF_8));
    assert_eq!(resolve_encoding(Some("utf-8")), Some(UTF_8));
    assert_eq!(resolve_encoding(Some("UTF8")), Some(UTF_8));
}

#[test]
fn the_windows1252_family_all_resolve_the_same() {
    // Every legacy-producer spelling maps to Windows-1252 — crucially the
    // hyphenated `latin-1`, which is NOT a WHATWG label (so `for_label` alone
    // rejected it, the divergence this unifies).
    for label in ["latin-1", "latin1", "iso-8859-1", "windows-1252", "cp1252"] {
        assert_eq!(
            resolve_encoding(Some(label)),
            Some(WINDOWS_1252),
            "label {label:?} should resolve to Windows-1252"
        );
    }
}

#[test]
fn labels_are_trimmed_and_case_insensitive() {
    assert_eq!(resolve_encoding(Some("  Latin-1 ")), Some(WINDOWS_1252));
    assert_eq!(resolve_encoding(Some("Windows-1252")), Some(WINDOWS_1252));
}

#[test]
fn other_whatwg_labels_still_flow_through() {
    assert_eq!(
        resolve_encoding(Some("shift_jis")),
        Some(encoding_rs::SHIFT_JIS)
    );
}

#[test]
fn a_genuinely_unknown_label_is_none() {
    // None, not a silent fallback — the caller (library vs UI) picks the policy.
    assert_eq!(resolve_encoding(Some("totally-bogus-encoding")), None);
}
