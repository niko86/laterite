//! Shared helpers for the command modules — encoding/edition resolution and the
//! sibling-path conventions, lifted verbatim from the pre-rework `main.rs`.

use std::path::{Path, PathBuf};
use std::process::exit;

use laterite_ags4_validator::{CheckOptions, DictVersion, overlay};

use crate::cli::DictArgs;

/// Map an `--encoding <name>` label to an `encoding_rs` encoding.
///
/// Straight to the parse leaf — this used to be a PRIVATE table with a wider label
/// set than the leaf's (it alone accepted `latin9` / `latin-9`), so
/// `lat --encoding latin-9` worked on the binary while the Python library rejected
/// the same label. Those two aliases were promoted into the leaf; there is now one
/// table, and one label means one thing on every surface.
pub fn resolve_encoding(label: &str) -> Option<&'static encoding_rs::Encoding> {
    laterite_ags4_parse::resolve_encoding(Some(label))
}

/// Fold the shared `--dict-version` / `--dict` / `--encoding` flags onto a base
/// `CheckOptions`, exiting 5 with the pre-rework message on a bad value.
pub fn apply_dict_args(mut opts: CheckOptions, d: &DictArgs) -> CheckOptions {
    if let Some(v) = d.dict_version.as_deref() {
        opts.dict_version = match v {
            "auto" => None,
            // Ask the GENERATED `from_edition`, not a hand-written match. The error
            // message below was already generated (`editions_joined`) while the arms
            // above it were not — so a new edition in ags_dictionary.json would have
            // produced a CLI that rejects `4.3` with a message advertising `4.3`.
            other => {
                if let Some(dv) = DictVersion::from_edition(other) {
                    Some(dv)
                } else {
                    eprintln!(
                        "error: --dict-version expects auto|{}, got {other:?}",
                        laterite_ags4_validator::editions_joined("|")
                    );
                    exit(5);
                }
            }
        };
    }
    if let Some(label) = d.encoding.as_deref() {
        if let Some(enc) = resolve_encoding(label) {
            opts.encoding = enc;
        } else {
            // Name the labels AGS4 files actually turn up in, then say what the
            // real rule is. The accepted set is every WHATWG label (via
            // `Encoding::for_label`) plus the leaf's extra aliases, which is far
            // too long to list and would rot the moment it was written down.
            eprintln!(
                "error: --encoding {label:?} not recognised \
                 (common: utf-8 / cp1252 / latin1 / iso-8859-1 / latin-9; \
                 any WHATWG encoding label is accepted)"
            );
            exit(5);
        }
    }
    if let Some(p) = d.dict.as_ref() {
        // A forced base and "no base" cannot both hold.
        if d.dict_replace && opts.dict_version.is_some() {
            eprintln!(
                "error: --dict-replace cannot be combined with --dict-version \
                 (a forced base contradicts a full replacement)"
            );
            exit(5);
        }
        let bytes = match std::fs::read(p) {
            Ok(b) => b,
            Err(e) => {
                eprintln!("error: cannot read --dict {}: {e}", p.display());
                exit(5);
            }
        };
        // With `--dict`, `--dict-version` (already folded into `opts.dict_version`
        // above) selects the OVERLAY BASE rather than a bundled edition;
        // `--dict-replace` drops the base entirely; otherwise the base is detected
        // structurally from the dictionary itself (#568 §2).
        let base = if d.dict_replace {
            overlay::BaseSpec::Replace
        } else if let Some(v) = opts.dict_version {
            overlay::BaseSpec::Force(v)
        } else {
            overlay::BaseSpec::Auto
        };
        // Advisory label for the cert — the basename, never the path (#568 §4).
        let name = p
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("custom-dict")
            .to_string();
        match overlay::parse_dict(
            &bytes,
            overlay::DictFormat::Auto,
            opts.encoding,
            base,
            &name,
        ) {
            Ok(cd) => opts.custom_dict = Some(cd),
            Err(e) => {
                eprintln!("error: bad --dict {name}: {e}");
                exit(5);
            }
        }
    }
    opts
}

/// `<source>.ags.idx` — append `.idx` to the full filename, so
/// `delivery.ags` → `delivery.ags.idx` (matching Python `.certify()`).
pub fn default_index_path(path: &Path) -> PathBuf {
    let mut s = path.as_os_str().to_os_string();
    s.push(".idx");
    PathBuf::from(s)
}

/// `delivery.ags` → `delivery.fixed.ags` (insert `.fixed` before the extension);
/// an extension-less `foo` → `foo.fixed`.
pub fn sibling_fixed_path(path: &Path) -> PathBuf {
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("out");
    let fname = match path.extension().and_then(|s| s.to_str()) {
        Some(ext) => format!("{stem}.fixed.{ext}"),
        None => format!("{stem}.fixed"),
    };
    path.with_file_name(fname)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every edition the DICTIONARY defines must be accepted by `--dict-version`.
    ///
    /// This is the assertion a hand-written match cannot survive. The old code listed
    /// the editions by hand while deriving its *rejection message* from
    /// `DictVersion::ALL` — so bundling a new edition would have produced a CLI that
    /// rejects `4.3` and, in the same breath, tells you `4.3` is one of the values it
    /// expects. Nothing failed, because nothing compared the two.
    ///
    /// It passes trivially today. It is written for the day `ALL` grows: if anyone
    /// reintroduces a hand-list, this goes red the moment the dictionary moves past it.
    #[test]
    fn every_bundled_edition_is_accepted() {
        for dv in DictVersion::ALL {
            let args = DictArgs {
                dict_version: Some(dv.as_str().to_string()),
                dict: None,
                dict_replace: false,
                encoding: None,
            };
            let opts = apply_dict_args(CheckOptions::default(), &args);
            assert_eq!(
                opts.dict_version,
                Some(*dv),
                "--dict-version {} was not accepted, but the dictionary bundles it",
                dv.as_str()
            );
        }
    }

    /// `auto` means "decide from `TRAN_AGS`", i.e. force nothing.
    #[test]
    fn auto_forces_no_edition() {
        let args = DictArgs {
            dict_version: Some("auto".to_string()),
            dict: None,
            dict_replace: false,
            encoding: None,
        };
        assert_eq!(
            apply_dict_args(CheckOptions::default(), &args).dict_version,
            None
        );
    }

    #[test]
    fn default_index_path_appends_ags_idx() {
        // `delivery.ags` → `delivery.ags.idx` (append, matching Python .certify()).
        assert_eq!(
            default_index_path(Path::new("/data/delivery.ags")),
            PathBuf::from("/data/delivery.ags.idx")
        );
    }
}
