//! The host bindings' shared option normalisation — the seam nobody owned.
//!
//! Every surface binding (PyO3, napi, wasm, the facade, the CLI) takes the
//! same caller-facing knobs — an edition label, a write mode, a custom
//! dictionary, a staged file write — and until #923 each normalised them in
//! its own dialect. Four copies drifted three ways before anything noticed:
//! two `unwrap_or(V4_1_1)` survived the generated-fallback sweep, "auto" grew
//! two semantics, and one surface's "staged" write lost its exclusive create.
//! No gate compared the copies, and the two biggest hosts are excluded from
//! `cargo test --workspace`, so the logic was untestable exactly where it was
//! duplicated.
//!
//! This module is the one copy, in a crate the workspace test run reaches.
//! A binding shrinks to marshal → call → map [`OptError`] into its own error
//! type. What stays per-surface is *data*, not logic: the flag spellings a
//! surface's user actually typed ([`DictFlags`]), so an error still names the
//! knob as that surface spells it.

use std::path::Path;

use laterite_ags4_validator::overlay::{self, BaseSpec, CustomDict, DictFormat};
use laterite_ags4_validator::{DictVersion, dict, editions_joined};

use crate::emit::EmitMode;

/// A refused option, host-agnostically.
///
/// `code`/`kind` follow the `lat` exit-code contract the py and node surfaces
/// already speak (5 = `bad_dict` / bad argument); `message` is the caller-facing
/// text. Each host maps this into its own error type at the boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OptError {
    pub code: i32,
    pub kind: &'static str,
    pub message: String,
}

impl OptError {
    fn bad(kind: &'static str, message: String) -> OptError {
        OptError {
            code: 5,
            kind,
            message,
        }
    }
}

impl std::fmt::Display for OptError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for OptError {}

/// Parse an edition label, keeping `auto` deferrable.
///
/// `None` means the caller did not choose — absent, empty, or the literal
/// `auto` — and the decision belongs to whatever resolves next (a check door
/// defers to the file's own `TRAN_AGS`; an emit door falls back — see
/// [`edition_or_fallback`]). Collapsing here is what forked the surfaces:
/// a deferring host and a collapsing host cannot share a parser that has
/// already decided.
///
/// Both the accepted set and the rejection message come from the dictionary
/// (`from_edition` + `editions_joined` are generated from `ags_dictionary.json`),
/// so a new edition reaches every surface's parser and every surface's error
/// text in the same commit.
pub fn edition(label: Option<&str>) -> Result<Option<DictVersion>, OptError> {
    match label.map(str::trim) {
        None | Some("" | "auto") => Ok(None),
        Some(other) => DictVersion::from_edition(other).map(Some).ok_or_else(|| {
            OptError::bad(
                "bad_args",
                format!(
                    "unknown edition {other:?}; expected auto|{}",
                    editions_joined("|")
                ),
            )
        }),
    }
}

/// [`edition`] for the doors with nothing to defer to: `auto` becomes the
/// dictionary's generated fallback — never a hand-written version literal,
/// which is the exact hard-coding this function retired (twice-missed by the
/// 2026-07-14 sweep, at the two node emit doors).
pub fn edition_or_fallback(label: Option<&str>) -> Result<DictVersion, OptError> {
    Ok(edition(label)?.unwrap_or(dict::FALLBACK))
}

/// Parse a write-mode label. Absent/empty means [`EmitMode::AutoFix`] — the
/// "just give me valid AGS4" default every surface documents.
pub fn write_mode(label: Option<&str>) -> Result<EmitMode, OptError> {
    match label.map(str::trim).map(str::to_ascii_lowercase).as_deref() {
        None | Some("" | "autofix") => Ok(EmitMode::AutoFix),
        Some("report") => Ok(EmitMode::Report),
        Some("strict") => Ok(EmitMode::Strict),
        Some(other) => Err(OptError::bad(
            "bad_args",
            format!("unknown mode {other:?}; expected autofix|report|strict"),
        )),
    }
}

/// How a surface spells the custom-dictionary knobs, for error text only.
///
/// The logic below is one copy; what a user TYPED is not — `--dict-replace`
/// on the CLI is `dictReplace` in Node and `dict_replace` in Python — and an
/// error that names a knob the caller cannot find is worse than four copies.
/// This is deliberately data, so the spelling difference cannot grow logic.
#[derive(Debug, Clone, Copy)]
pub struct DictFlags {
    /// The dictionary-source knob (`--dict`, `dict`, `dict_bytes`, …).
    pub source: &'static str,
    /// The full-replacement knob.
    pub replace: &'static str,
    /// The forced-base-edition knob.
    pub version: &'static str,
}

/// Build the runtime custom-dictionary overlay from a path or bytes.
///
/// One ladder for all four surfaces: the source arms (a path the host reads,
/// or bytes the caller already holds — the wasm sandbox has no path arm and
/// passes `None`), the `replace`/`force`/`auto` base selection, and the
/// contradiction refusal (a forced base and "no base" cannot both hold). The
/// advisory name the cert records is the path's basename or a neutral label —
/// never a full filesystem path (laterite-dev#568 §4).
///
/// `enc` is the caller's already-resolved source encoding — the same one it
/// hands `CheckOptions` — so the label is resolved once per call.
pub fn custom_dict(
    dict_path: Option<&Path>,
    dict_bytes: Option<&[u8]>,
    dict_replace: bool,
    over: Option<DictVersion>,
    enc: &'static encoding_rs::Encoding,
    flags: DictFlags,
) -> Result<Option<CustomDict>, OptError> {
    let (bytes, name): (Vec<u8>, String) = if let Some(p) = dict_path {
        let b = std::fs::read(p).map_err(|e| {
            OptError::bad(
                "bad_dict",
                format!("cannot read {} {}: {e}", flags.source, p.display()),
            )
        })?;
        let name = p
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("custom-dict")
            .to_string();
        (b, name)
    } else if let Some(b) = dict_bytes {
        (b.to_vec(), "custom-dict".to_string())
    } else {
        return Ok(None);
    };
    if dict_replace && over.is_some() {
        return Err(OptError::bad(
            "bad_dict",
            format!(
                "{} cannot be combined with {} (a forced base contradicts a full replacement)",
                flags.replace, flags.version
            ),
        ));
    }
    let base = if dict_replace {
        BaseSpec::Replace
    } else if let Some(v) = over {
        BaseSpec::Force(v)
    } else {
        BaseSpec::Auto
    };
    overlay::parse_dict(&bytes, DictFormat::Auto, enc, base, &name)
        .map(Some)
        .map_err(|e| OptError::bad("bad_dict", format!("bad {} {name}: {e}", flags.source)))
}

/// Write `bytes` to `dest` via a temporary file in the destination's own
/// directory + rename — atomic on one filesystem, so `dest` never holds a
/// partial write. The build doors' `out=` contract on every surface
/// (`std::fs::rename` replaces an existing file on Unix and Windows alike;
/// `create_new` makes a name collision an error rather than a silent
/// overwrite of whatever was squatting on it).
pub fn staged_write(dest: &Path, bytes: &[u8]) -> Result<(), OptError> {
    let io_err = |e: std::io::Error| OptError {
        code: 3,
        kind: "io",
        message: format!("cannot write {}: {e}", dest.display()),
    };
    let dir = staging_dir(dest);
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.subsec_nanos());
    let tmp = dir.join(format!(
        ".laterite-build-{}-{nanos}.tmp",
        std::process::id()
    ));
    std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&tmp)
        .and_then(|mut f| std::io::Write::write_all(&mut f, bytes))
        .and_then(|()| std::fs::rename(&tmp, dest))
        .map_err(|e| {
            // Best-effort cleanup: the temp file is ours alone (create_new),
            // and leaving it behind litters the caller's output directory.
            let _ = std::fs::remove_file(&tmp);
            io_err(e)
        })
}

/// The directory the staging file goes in: the DESTINATION's own, never the
/// system temp dir — rename is only atomic within one filesystem, and that
/// atomicity is the door's whole promise. A bare filename has no parent (or an
/// empty one), and both mean the current directory.
///
/// Its own function because the property is invisible to the integration
/// tests: on one filesystem a mis-chosen directory still passes every
/// end-to-end assertion, so the choice is pinned here, where a unit test can
/// see it.
fn staging_dir(dest: &Path) -> &Path {
    match dest.parent() {
        Some(p) if !p.as_os_str().is_empty() => p,
        _ => Path::new("."),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auto_empty_and_absent_all_defer() {
        for label in [None, Some(""), Some("auto"), Some("  auto  ")] {
            assert_eq!(edition(label).unwrap(), None, "{label:?}");
        }
    }

    #[test]
    fn a_real_label_resolves_and_a_bad_one_names_the_set() {
        assert_eq!(
            edition(Some("4.1.1")).unwrap(),
            Some(DictVersion::from_edition("4.1.1").unwrap())
        );
        let err = edition(Some("4.9")).unwrap_err();
        assert!(err.message.contains("unknown edition"), "{}", err.message);
        assert!(err.message.contains("\"4.9\""), "{}", err.message);
        // The set AND the word `auto` are in the message — the caller is told
        // every spelling that would have worked.
        assert!(err.message.contains("auto|"), "{}", err.message);
        assert!(err.message.contains("4.1.1"), "{}", err.message);
    }

    /// The defect class this module exists for: the fallback is the
    /// dictionary's, by construction — there is no version literal here to
    /// go stale when the fallback moves.
    #[test]
    fn the_fallback_is_the_dictionarys_own() {
        assert_eq!(edition_or_fallback(None).unwrap(), dict::FALLBACK);
        assert_eq!(edition_or_fallback(Some("auto")).unwrap(), dict::FALLBACK);
    }

    #[test]
    fn write_mode_accepts_the_three_and_defaults_to_autofix() {
        assert_eq!(write_mode(None).unwrap(), EmitMode::AutoFix);
        assert_eq!(write_mode(Some("")).unwrap(), EmitMode::AutoFix);
        assert_eq!(write_mode(Some("AutoFix")).unwrap(), EmitMode::AutoFix);
        assert_eq!(write_mode(Some("report")).unwrap(), EmitMode::Report);
        assert_eq!(write_mode(Some(" STRICT ")).unwrap(), EmitMode::Strict);
        let err = write_mode(Some("nope")).unwrap_err();
        assert!(err.message.contains("unknown mode"), "{}", err.message);
        assert!(
            err.message.contains("autofix|report|strict"),
            "{}",
            err.message
        );
    }

    const FLAGS: DictFlags = DictFlags {
        source: "--dict",
        replace: "--dict-replace",
        version: "--dict-version",
    };

    #[test]
    fn no_source_is_no_dict() {
        let got = custom_dict(None, None, false, None, encoding_rs::UTF_8, FLAGS).unwrap();
        assert!(got.is_none());
    }

    /// The contradiction refusal names the knobs as THIS surface spells them —
    /// the one per-surface difference, and it is data, not logic.
    #[test]
    fn replace_and_forced_base_contradict_in_the_surfaces_spelling() {
        let err = custom_dict(
            None,
            Some(b"anything"),
            true,
            Some(dict::FALLBACK),
            encoding_rs::UTF_8,
            FLAGS,
        )
        .unwrap_err();
        assert_eq!(err.kind, "bad_dict");
        assert!(err.message.contains("--dict-replace"), "{}", err.message);
        assert!(err.message.contains("--dict-version"), "{}", err.message);
    }

    /// The CLI prints errors through `{e}` — Display IS the message, not a
    /// derived or truncated view of it.
    #[test]
    fn opt_error_displays_its_message() {
        let err = edition(Some("9.9")).unwrap_err();
        assert_eq!(err.to_string(), err.message);
    }

    /// The refusal is the CONJUNCTION: `replace` with no forced base must
    /// reach the ladder's Replace arm, not trip the contradiction message.
    #[test]
    fn replace_alone_is_legal_not_a_contradiction() {
        let err = custom_dict(
            None,
            Some(b"\x00not a dictionary"),
            true,
            None,
            encoding_rs::UTF_8,
            FLAGS,
        )
        .unwrap_err();
        assert!(err.message.contains("bad --dict"), "{}", err.message);
        assert!(
            !err.message.contains("cannot be combined"),
            "{}",
            err.message
        );
    }

    #[test]
    fn an_unreadable_path_and_garbage_bytes_each_name_the_source_flag() {
        let err = custom_dict(
            Some(Path::new("/nonexistent/dict.json")),
            None,
            false,
            None,
            encoding_rs::UTF_8,
            FLAGS,
        )
        .unwrap_err();
        assert!(
            err.message.contains("cannot read --dict"),
            "{}",
            err.message
        );

        let err = custom_dict(
            None,
            Some(b"\x00not a dictionary"),
            false,
            None,
            encoding_rs::UTF_8,
            FLAGS,
        )
        .unwrap_err();
        assert!(err.message.contains("bad --dict"), "{}", err.message);
        // In-memory bytes get the neutral advisory label, never a path.
        assert!(err.message.contains("custom-dict"), "{}", err.message);
    }

    fn scratch(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("laterite-hostopts-{name}"));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn staged_write_replaces_and_leaves_no_litter() {
        let dir = scratch("staged");
        let dest = dir.join("out.ags");
        std::fs::write(&dest, b"stale").unwrap();
        staged_write(&dest, b"fresh").unwrap();
        assert_eq!(std::fs::read(&dest).unwrap(), b"fresh");
        assert_eq!(
            std::fs::read_dir(&dir).unwrap().count(),
            1,
            "only the destination may remain"
        );
    }

    #[test]
    fn a_failed_staging_reports_the_destination_not_the_temp() {
        let err = staged_write(Path::new("/nonexistent/dir/out.ags"), b"x").unwrap_err();
        assert_eq!(err.kind, "io");
        assert!(err.message.contains("out.ags"), "{}", err.message);
    }

    /// See `staging_dir` — the same-filesystem property cannot fail an
    /// integration test on a single-filesystem machine, so the choice itself
    /// is the thing asserted.
    #[test]
    fn the_staging_dir_is_the_destinations_own() {
        assert_eq!(
            staging_dir(Path::new("/a/b/out.ags")),
            Path::new("/a/b"),
            "staging anywhere else forfeits rename atomicity"
        );
        assert_eq!(staging_dir(Path::new("out.ags")), Path::new("."));
        assert_eq!(staging_dir(Path::new("./out.ags")), Path::new("."));
    }
}
