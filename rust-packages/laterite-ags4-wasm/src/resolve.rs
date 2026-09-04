//! Turning what a caller *said* into what the engine takes.
//!
//! An encoding label, a `dictVersion` string and a custom-dictionary blob all
//! arrive from JS as text and bytes, and every door that accepts them has to
//! reach the same verdict about what they mean — including what to say when
//! they mean nothing. Shared here rather than per-verb so a label cannot be
//! tolerated at one door and refused at the next.
use laterite_ags4_validator::{DictVersion, ValidatorError, overlay};

/// Map a `ValidatorError` to a `(kind, message)`. In the wasm path only
/// `NotAgs4` / `UnsupportedEdition` are actually reachable — there is no
/// filesystem (so no `NotFound`/`Io`), decode is lossy (non-UTF-8 surfaces
/// as a Rule 1 finding), and we never set `custom_dict` (so no `BadDict`) —
/// but we map every arm so the `match` is total and future-proof.
pub(crate) fn classify(e: &ValidatorError) -> (&'static str, String) {
    // Delegate to the single producer `ValidatorError::kind()`, except the
    // deliberate, allowlisted divergence: with no filesystem, `NotFound`/`Io` are
    // unreachable, so they collapse to `"io"` here (vs the producer's
    // `"not_found"`) purely to keep the match total. Gated in the tests below.
    let kind = match e {
        ValidatorError::NotFound(_) | ValidatorError::Io { .. } => "io",
        other => other.kind(),
    };
    (kind, e.to_string())
}

/// Resolve a UI encoding label to an `encoding_rs` encoding, via the shared label
/// table in the parse leaf so a label means the same thing on every surface.
///
/// An unknown label is an ERROR, not a fallback. It used to return UTF-8 — which
/// reads like leniency and behaves like corruption: `C3 A9` decodes cleanly as `é`
/// in UTF-8 and `Ã©` in cp1252, so a caller who asked for the wrong label got the
/// wrong text and a clean bill of health. Python raised on the same input. The
/// browser's own select only offers UTF-8 / Windows-1252, so the UI cannot trip
/// this; the wasm API is public, and a caller who names a charset we do not know
/// deserves to be told so.
pub(crate) fn resolve_encoding(
    label: Option<&str>,
) -> std::result::Result<&'static encoding_rs::Encoding, String> {
    laterite_ags4_parse::resolve_encoding(label)
        .ok_or_else(|| format!("unknown encoding {:?}", label.unwrap_or("")))
}

/// Map a UI dict-version string to a forced edition. `None` / `"auto"`
/// ⇒ `Ok(None)` (auto-detect from `TRAN_AGS`). An unrecognised string
/// returns `Err(message)` (the caller turns it into a `bad_args`
/// report); we return the short message rather than the whole report so
/// the `Err` variant stays small (clippy `result_large_err`).
/// The parse is `hostopts` (#923) — one copy per workspace, not per surface.
pub(crate) fn resolve_dict_override(s: Option<&str>) -> Result<Option<DictVersion>, String> {
    laterite_ags4_emit::hostopts::edition(s).map_err(|e| e.message)
}

/// Build the runtime custom-dictionary overlay (laterite-dev#568) from browser-supplied bytes.
///
/// The wasm sandbox has no filesystem, so — unlike the CLI/Python/Node twins — this
/// has no path arm: a custom dict always arrives as raw bytes (a `Uint8Array` the UI
/// read from a file). `over` forces a base edition (from `dict_version`),
/// `dict_replace` drops the base entirely, and the two cannot both hold (a forced
/// base contradicts a full replacement). `enc` is the caller's already-resolved
/// source encoding — the same one it hands `CheckOptions`.
///
/// Returns `Ok(None)` when no dict was supplied. The error is a short message the
/// caller surfaces on the same channel a bad `dict_version` uses.
pub(crate) fn build_custom_dict(
    dict_bytes: Option<&[u8]>,
    dict_replace: bool,
    over: Option<DictVersion>,
    enc: &'static encoding_rs::Encoding,
) -> std::result::Result<Option<overlay::CustomDict>, String> {
    // The ladder is `hostopts` (#923); this surface has no path arm (the wasm
    // sandbox has no filesystem), so `dict_path` is structurally `None` and a
    // custom dict always arrives as bytes under the neutral advisory label.
    laterite_ags4_emit::hostopts::custom_dict(
        None,
        dict_bytes,
        dict_replace,
        over,
        enc,
        laterite_ags4_emit::hostopts::DictFlags {
            source: "dict",
            replace: "dict_replace",
            version: "dict_version",
        },
    )
    .map_err(|e| e.message)
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- encoding resolution + transcode (the cp1252/UTF-8 path) ---
    // This is the bug class behind the web app's mojibake guard + the
    // "applying a fix also normalises encoding" promise; it had zero coverage.

    #[test]
    fn resolve_encoding_maps_the_offered_labels() {
        assert_eq!(resolve_encoding(None).unwrap().name(), "UTF-8");
        assert_eq!(resolve_encoding(Some("")).unwrap().name(), "UTF-8");
        assert_eq!(resolve_encoding(Some("utf-8")).unwrap().name(), "UTF-8");
        for label in ["windows-1252", "CP1252", "latin1", "ISO-8859-1"] {
            assert_eq!(
                resolve_encoding(Some(label)).unwrap().name(),
                "windows-1252",
                "{label}"
            );
        }
    }

    /// An unknown label is an ERROR here, as it always was on Python.
    ///
    /// This assertion used to say the opposite — "an unknown label falls back to
    /// UTF-8 (lossy), not an error" — which is how the bug survived: it was not an
    /// oversight, it was *codified*. But the fallback is not lossy, it is silent:
    /// `C3 A9` decodes cleanly as `é` in UTF-8 and `Ã©` in cp1252, so a caller who
    /// typo'd their label got the wrong text and no error at all. `apply_fixes` would
    /// then rewrite the file from that mis-decode.
    #[test]
    fn an_unknown_encoding_label_is_an_error_not_a_fallback() {
        assert!(resolve_encoding(Some("not-a-charset")).is_err());
        assert!(resolve_encoding(Some("cp1252x")).is_err());
        // ...and the label is named, so the caller can see their typo.
        assert!(
            resolve_encoding(Some("cp1252x"))
                .unwrap_err()
                .contains("cp1252x")
        );
    }

    #[test]
    fn classify_collapses_notfound_and_io_to_io() {
        // The deliberate, allowlisted divergence from the producer: with no
        // filesystem `NotFound`/`Io` are unreachable, so both collapse to "io"
        // (the producer's `kind()` returns "not_found" for `NotFound`). Everything
        // else delegates verbatim. Pins the divergence in-crate.
        assert_eq!(classify(&ValidatorError::NotFound("x".into())).0, "io");
        assert_eq!(
            classify(&ValidatorError::Io {
                path: "x".into(),
                source: std::io::Error::other("x"),
            })
            .0,
            "io"
        );
        assert_eq!(classify(&ValidatorError::NotAgs4("x".into())).0, "not_ags4");
    }

    #[test]
    fn resolve_dict_override_accepts_every_bundled_edition() {
        use laterite_ags4_validator::DictVersion;
        for ed in DictVersion::ALL {
            assert!(
                resolve_dict_override(Some(ed.as_str())).is_ok(),
                "bundled edition {} must resolve",
                ed.as_str()
            );
        }
        assert!(resolve_dict_override(Some("auto")).unwrap().is_none());
        assert!(resolve_dict_override(None).unwrap().is_none());
        // A bogus label errors, and the message lists EVERY bundled edition —
        // proving it derives from `DictVersion::ALL`, not a stale hand-list.
        let err = resolve_dict_override(Some("9.9")).unwrap_err();
        for ed in DictVersion::ALL {
            assert!(
                err.contains(ed.as_str()),
                "message must list {}",
                ed.as_str()
            );
        }
    }

    #[test]
    fn byte_0xe9_is_replacement_under_utf8_but_e_acute_under_cp1252() {
        let data = [b'a', 0xE9, b'b']; // 0xE9 = 'é' in cp1252, invalid UTF-8
        let (utf8, _, had_errors) = resolve_encoding(Some("utf-8")).unwrap().decode(&data);
        assert!(had_errors, "0xE9 is not valid UTF-8");
        assert!(
            utf8.contains('\u{FFFD}'),
            "lossy decode inserts U+FFFD: {utf8:?}"
        );
        let (cp, _, had) = resolve_encoding(Some("windows-1252"))
            .unwrap()
            .decode(&data);
        assert!(!had);
        assert_eq!(cp, "aéb");
    }

    #[test]
    fn build_custom_dict_is_a_no_op_without_bytes() {
        let none = build_custom_dict(None, false, None, encoding_rs::UTF_8).expect("no dict");
        assert!(
            none.is_none(),
            "no bytes means no overlay, not an empty one"
        );
    }
}
