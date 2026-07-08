//! Certificate mint + consume — the `.ags.idx` logic shared by `lat certify`
//! (mint) and `lat validate --index` (consume). Co-located so the mint→consume
//! round-trip tests reach both. The CLI stamps the shared `ENGINE_IDENTITY`
//! (PR 1a), so its certs interop with Python/Node/DuckDB.

use std::path::{Path, PathBuf};

use laterite_ags4_core::index::{ENGINE_IDENTITY, Sidecar, ValidationStamp};
use laterite_ags4_parse::parse_bytes;
use laterite_ags4_validator::dict::FALLBACK;
use laterite_ags4_validator::{CheckOptions, ValidatorError, resolve_dict_version, tran_ags_of};
use laterite_cliutil::write_atomic;

use crate::commands::common::default_index_path;

/// Outcome of a `--index` certificate consume: skip the engine (the cert vouches
/// for a clean validation) or re-validate, carrying the human reason.
pub enum CertOutcome {
    Skip(ValidationStamp),
    Revalidate(String),
}

/// Mint the `.ags.idx` certificate for an error-clean `path`: re-index the bytes,
/// stamp the shared engine identity + edition + advisory counts, and write it
/// (to `out`, else `<path>.ags.idx`).
pub fn mint_index(
    path: &Path,
    opts: &CheckOptions,
    warnings: u32,
    fyi: u32,
    out: Option<&Path>,
) -> Result<PathBuf, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("read {}: {e}", path.display()))?;
    let pf = parse_bytes(&bytes, opts.encoding).map_err(|e| ValidatorError::from(e).to_string())?;
    let dv = resolve_dict_version(opts.dict_version, tran_ags_of(&pf).as_deref())
        .map(|(dv, _)| dv)
        .unwrap_or(FALLBACK);
    let stamp = ValidationStamp {
        validator: ENGINE_IDENTITY.to_string(),
        // The ENGINE version (comparable across surfaces), not this CLI's crate.
        validator_version: laterite_ags4_validator::VERSION.to_string(),
        compat: None,
        check_files: opts.check_files,
        edition_forced: opts.dict_version.is_some(),
        checked_at: chrono::Utc::now().to_rfc3339(),
        warnings,
        fyi,
    };
    let sidecar =
        Sidecar::assemble(&bytes, dv.as_str().to_string(), stamp).map_err(|e| e.to_string())?;
    let dest = out
        .map(PathBuf::from)
        .unwrap_or_else(|| default_index_path(path));
    let json = sidecar.to_json().map_err(|e| e.to_string())?;
    write_atomic(&dest, &json).map_err(|e| format!("write {}: {e}", dest.display()))?;
    Ok(dest)
}

/// Decide whether a `.ags.idx` certificate can stand in for a fresh validation:
/// trusted only when byte-fresh, minted by THIS engine identity
/// (`ENGINE_IDENTITY` + the validator VERSION), and its profile covers the
/// request. Any miss ⇒ re-validate with the reason.
pub fn try_certified_skip(path: &Path, opts: &CheckOptions, cert_path: &Path) -> CertOutcome {
    let bytes = match std::fs::read(path) {
        Ok(b) => b,
        Err(e) => return CertOutcome::Revalidate(format!("cannot read the source file: {e}")),
    };
    let cert_bytes = match std::fs::read(cert_path) {
        Ok(b) => b,
        Err(e) => {
            return CertOutcome::Revalidate(format!(
                "cannot read certificate {}: {e}",
                cert_path.display()
            ));
        }
    };
    let sidecar = match Sidecar::from_json(&cert_bytes) {
        Ok(s) => s,
        Err(e) => {
            return CertOutcome::Revalidate(format!(
                "{} is not a valid .ags.idx certificate: {e}",
                cert_path.display()
            ));
        }
    };
    if !sidecar.is_fresh_for(&bytes) {
        return CertOutcome::Revalidate(
            "certificate is stale — the file changed since it was minted".to_string(),
        );
    }
    if !sidecar.checker_matches(ENGINE_IDENTITY, laterite_ags4_validator::VERSION, None) {
        return CertOutcome::Revalidate(format!(
            "certificate was minted by {} {} (compat {:?}), not {} {}",
            sidecar.validation.validator,
            sidecar.validation.validator_version,
            sidecar.validation.compat.as_deref(),
            ENGINE_IDENTITY,
            laterite_ags4_validator::VERSION
        ));
    }
    let forced_edition = opts.dict_version.map(|dv| dv.as_str());
    if !sidecar.profile_covers(opts.check_files, forced_edition) {
        return CertOutcome::Revalidate(
            "certificate's validation profile does not cover this request \
             (--check-files / --dict-version)"
                .to_string(),
        );
    }
    CertOutcome::Skip(sidecar.validation.clone())
}

/// Print the provenance note for a trusted `--index` skip.
pub fn report_certified_skip(stamp: &ValidationStamp, include_warnings: bool, include_fyi: bool) {
    let mut advisory = Vec::new();
    if include_warnings {
        advisory.push(format!("{} warning(s)", stamp.warnings));
    }
    if include_fyi {
        advisory.push(format!("{} fyi", stamp.fyi));
    }
    let recorded = if advisory.is_empty() {
        String::new()
    } else {
        let hint = if (include_warnings && stamp.warnings > 0) || (include_fyi && stamp.fyi > 0) {
            " — run without --index to list them"
        } else {
            ""
        };
        format!("; cert recorded {}{hint}", advisory.join(", "))
    };
    eprintln!(
        "note: certified clean by {} at {} — rule engine skipped{recorded}",
        stamp.validator, stamp.checked_at
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use laterite_ags4_validator::{check_file, findings::Severity};

    /// The shared hand-authored ERROR-clean AGS4 fixture (CRLF, enforced by
    /// `.gitattributes`). Referenced, not copied — one source.
    const CLEAN_AGS: &str =
        include_str!("../../../laterite-ags4-validator/tests/fixtures/clean_minimal.ags");

    fn errors(found: &laterite_ags4_validator::Findings) -> u32 {
        found
            .values()
            .flatten()
            .filter(|f| f.severity == Severity::Error)
            .count() as u32
    }

    #[test]
    fn mint_index_writes_a_certificate_with_the_shared_identity() {
        let dir = std::env::temp_dir();
        let src = dir.join(format!("lat_mint_{}.ags", std::process::id()));
        std::fs::write(&src, CLEAN_AGS).unwrap();
        let opts = CheckOptions {
            include_warnings: true,
            ..CheckOptions::default()
        };
        // Sanity: the fixture really is error-clean (else the mint would be a lie).
        assert_eq!(errors(&check_file(&src, &opts).unwrap()), 0);

        let dest = mint_index(&src, &opts, 0, 0, None).unwrap();
        assert_eq!(dest, default_index_path(&src));
        let sidecar = Sidecar::from_json(&std::fs::read(&dest).unwrap()).unwrap();
        // PR 1a: the CLI now stamps the shared identity, not "lat-check".
        assert_eq!(sidecar.validation.validator, ENGINE_IDENTITY);
        assert_eq!(sidecar.file.edition, "4.2");
        for g in ["PROJ", "TRAN", "UNIT", "TYPE"] {
            assert!(sidecar.groups.contains_key(g), "missing group {g}");
        }
        assert!(sidecar.is_fresh_for(CLEAN_AGS.as_bytes()));

        let _ = std::fs::remove_file(&src);
        let _ = std::fs::remove_file(&dest);
    }

    #[test]
    fn certified_skip_trusts_a_fresh_own_engine_cert() {
        let dir = std::env::temp_dir();
        let src = dir.join(format!("lat_skip_ok_{}.ags", std::process::id()));
        std::fs::write(&src, CLEAN_AGS).unwrap();
        let opts = CheckOptions {
            include_warnings: true,
            ..CheckOptions::default()
        };
        let cert = mint_index(&src, &opts, 0, 0, None).unwrap();

        match try_certified_skip(&src, &opts, &cert) {
            CertOutcome::Skip(stamp) => {
                assert_eq!(stamp.validator, ENGINE_IDENTITY);
                assert_eq!(stamp.warnings, 0);
            }
            CertOutcome::Revalidate(why) => panic!("expected skip, got revalidate: {why}"),
        }
        let _ = std::fs::remove_file(&src);
        let _ = std::fs::remove_file(&cert);
    }

    #[test]
    fn certified_skip_declines_a_stale_cert() {
        let dir = std::env::temp_dir();
        let src = dir.join(format!("lat_skip_stale_{}.ags", std::process::id()));
        std::fs::write(&src, CLEAN_AGS).unwrap();
        let opts = CheckOptions {
            include_warnings: true,
            ..CheckOptions::default()
        };
        let cert = mint_index(&src, &opts, 0, 0, None).unwrap();
        // Mutate the source AFTER minting → the cert no longer matches its bytes.
        std::fs::write(&src, format!("{CLEAN_AGS}\r\n\"GROUP\",\"EXTRA\"\r\n")).unwrap();
        assert!(
            matches!(try_certified_skip(&src, &opts, &cert), CertOutcome::Revalidate(w) if w.contains("stale")),
            "a changed file must decline the skip"
        );
        let _ = std::fs::remove_file(&src);
        let _ = std::fs::remove_file(&cert);
    }

    #[test]
    fn certified_skip_declines_when_profile_insufficient() {
        let dir = std::env::temp_dir();
        let src = dir.join(format!("lat_skip_profile_{}.ags", std::process::id()));
        std::fs::write(&src, CLEAN_AGS).unwrap();
        // Mint WITHOUT the on-disk file check…
        let mint_opts = CheckOptions {
            include_warnings: true,
            ..CheckOptions::default()
        };
        let cert = mint_index(&src, &mint_opts, 0, 0, None).unwrap();
        // …then ask for it: a weaker cert can't cover a --check-files request.
        let want = CheckOptions {
            include_warnings: true,
            check_files: true,
            ..CheckOptions::default()
        };
        assert!(
            matches!(try_certified_skip(&src, &want, &cert), CertOutcome::Revalidate(w) if w.contains("profile")),
            "a cert minted without --check-files must not cover a --check-files request"
        );
        let _ = std::fs::remove_file(&src);
        let _ = std::fs::remove_file(&cert);
    }

    #[test]
    fn certified_skip_declines_a_missing_or_bogus_cert() {
        let dir = std::env::temp_dir();
        let src = dir.join(format!("lat_skip_bogus_{}.ags", std::process::id()));
        std::fs::write(&src, CLEAN_AGS).unwrap();
        let opts = CheckOptions::default();
        let missing = dir.join(format!("lat_skip_missing_{}.idx", std::process::id()));
        assert!(matches!(
            try_certified_skip(&src, &opts, &missing),
            CertOutcome::Revalidate(_)
        ));
        let junk = dir.join(format!("lat_skip_junk_{}.idx", std::process::id()));
        std::fs::write(&junk, b"not json").unwrap();
        assert!(
            matches!(try_certified_skip(&src, &opts, &junk), CertOutcome::Revalidate(w) if w.contains("valid .ags.idx")),
            "a non-certificate file must decline the skip"
        );
        let _ = std::fs::remove_file(&src);
        let _ = std::fs::remove_file(&junk);
    }
}
