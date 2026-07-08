//! Shared helpers for the command modules — encoding/edition resolution and the
//! sibling-path conventions, lifted verbatim from the pre-rework `main.rs`.

use std::path::{Path, PathBuf};
use std::process::exit;

use laterite_ags4_validator::{CheckOptions, DictVersion};

use crate::cli::DictArgs;

/// Map an `--encoding <name>` label to an `encoding_rs` encoding. The label set
/// is intentionally narrow: AGS4 prefers UTF-8 / ASCII; cp1252 and latin1 are
/// the legacy producers. Other WHATWG labels flow through `Encoding::for_label`.
pub fn resolve_encoding(label: &str) -> Option<&'static encoding_rs::Encoding> {
    let trimmed = label.trim().to_ascii_lowercase();
    let canonical = match trimmed.as_str() {
        "utf-8" | "utf8" => Some(encoding_rs::UTF_8),
        "cp1252" | "windows-1252" => Some(encoding_rs::WINDOWS_1252),
        // Latin-1 ≈ Windows-1252 except the 0x80-0x9F range; for AGS4 we treat
        // them as the same (cp1252 is the strict superset python-ags4 uses).
        "latin1" | "latin-1" | "iso-8859-1" => Some(encoding_rs::WINDOWS_1252),
        "iso-8859-15" | "latin9" | "latin-9" => Some(encoding_rs::ISO_8859_15),
        _ => None,
    };
    canonical.or_else(|| encoding_rs::Encoding::for_label(label.as_bytes()))
}

/// Fold the shared `--dict-version` / `--dict` / `--encoding` flags onto a base
/// `CheckOptions`, exiting 5 with the pre-rework message on a bad value.
pub fn apply_dict_args(mut opts: CheckOptions, d: &DictArgs) -> CheckOptions {
    if let Some(v) = d.dict_version.as_deref() {
        opts.dict_version = match v {
            "auto" => None,
            "4.0.3" => Some(DictVersion::V4_0_3),
            "4.0.4" => Some(DictVersion::V4_0_4),
            "4.1" => Some(DictVersion::V4_1),
            "4.1.1" => Some(DictVersion::V4_1_1),
            "4.2" => Some(DictVersion::V4_2),
            other => {
                eprintln!(
                    "error: --dict-version expects auto|{}, got {other:?}",
                    laterite_ags4_validator::editions_joined("|")
                );
                exit(5);
            }
        };
    }
    if let Some(label) = d.encoding.as_deref() {
        match resolve_encoding(label) {
            Some(enc) => opts.encoding = enc,
            None => {
                eprintln!(
                    "error: --encoding {label:?} not recognised \
                     (try utf-8 / cp1252 / latin1 / iso-8859-1)"
                );
                exit(5);
            }
        }
    }
    if let Some(p) = d.dict.as_ref() {
        opts.custom_dict = Some(p.clone());
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

    #[test]
    fn default_index_path_appends_ags_idx() {
        // `delivery.ags` → `delivery.ags.idx` (append, matching Python .certify()).
        assert_eq!(
            default_index_path(Path::new("/data/delivery.ags")),
            PathBuf::from("/data/delivery.ags.idx")
        );
    }
}
