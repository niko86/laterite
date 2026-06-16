//! FILE_FSET binary round-trip — slurp on ingest, unspool on export.
//!
//! AGS4.1 lets a transfer file reference adjacent binary attachments via
//! the FILE group: each row defines a logical file set under
//! `FILE_FSET`, with `FILE_NAME` pointing at a path relative to the .ags
//! file (or an explicit attachments directory). Other groups reference
//! attachments via headings ending `_FSET` (`LOCA_FILE_FSET` etc.).
//!
//! We slurp every referenced file at ingest time into the `blob` table
//! (`kind='attachment'`, parent row = the FILE row's UUID), and write
//! them back to disk alongside the output `.ags` on export. The blob
//! table already exists in the DDL — we just populate / drain it.
//!
//! The FILE group isn't in `ags5_dictionary.json`; AGS4 ingest registers
//! it as a passthrough (parent=LOCA, all headings type=X). That's fine
//! for us — we look up rows through the `v_file` view, which the DDL
//! builder generates for any registered group, passthrough included.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use duckdb::{AccessMode, Config, Connection};
use sha2::{Digest, Sha256};

use laterite_core::error::CliError;

/// Outcome of a slurp / unspool pass — surfaced to the CLI so the
/// command can report `files_attached: N` / `files_written: N`.
#[derive(Debug, Default)]
pub struct AttachmentStats {
    pub files_processed: usize,
    pub bytes_total: u64,
    pub warnings: Vec<String>,
}

/// Walk FILE group rows in `db_path`, resolve each `FILE_NAME` relative
/// to `attachments_dir` (falling back to the .ags file's parent), slurp
/// the bytes into the `blob` table.
///
/// Idempotent: a blob row is skipped if one with the same `sha256` and
/// `parent_id` already exists. That makes `ags4-to-db --append` safe to
/// run twice without doubling up attachments.
pub fn slurp_attachments(
    db_path: &Path,
    attachments_dir: &Path,
) -> Result<AttachmentStats, CliError> {
    let conn = Connection::open(db_path)
        .map_err(|e| CliError::Schema(format!("open db for attachments: {}", e)))?;
    let mut stats = AttachmentStats::default();

    // FILE group is registered as passthrough at AGS4-ingest time. If
    // this file didn't have one, `v_file` won't exist — that's fine,
    // we just have no attachments to slurp.
    let file_view_exists: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM information_schema.tables WHERE table_name = 'v_file'",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);
    if file_view_exists == 0 {
        return Ok(stats);
    }

    // Read (UUID, file_name) for every FILE row. The view's column names
    // are lowercase. Most FILE-group flavours include `file_name`; if
    // they don't we have nothing to attach.
    let cols: Vec<String> = {
        let mut s = conn
            .prepare("SELECT * FROM v_file LIMIT 0")
            .map_err(|e| CliError::Schema(format!("describe v_file: {}", e)))?;
        let r = s.query([]).map_err(|e| CliError::Schema(e.to_string()))?;
        let stmt_ref = r
            .as_ref()
            .ok_or_else(|| CliError::Schema("v_file statement detached".into()))?;
        (0..stmt_ref.column_count())
            .map(|i| {
                stmt_ref
                    .column_name(i)
                    .map(String::from)
                    .unwrap_or_default()
            })
            .collect()
    };
    if !cols.iter().any(|c| c == "file_name") {
        return Ok(stats);
    }

    let mut stmt = conn
        .prepare("SELECT id, file_name FROM v_file WHERE file_name IS NOT NULL")
        .map_err(|e| CliError::Schema(format!("query v_file: {}", e)))?;
    let mut rows_iter = stmt
        .query([])
        .map_err(|e| CliError::Schema(format!("query v_file: {}", e)))?;

    // Pre-load existing (parent_id, sha256) pairs so idempotent slurps
    // skip already-present blobs in O(1) instead of doing per-file
    // queries.
    let already: HashMap<(String, String), bool> = {
        let mut s = conn
            .prepare(
                "SELECT parent_id, sha256 FROM blob
                  WHERE kind = 'attachment' AND parent_table = 'g_file'",
            )
            .map_err(|e| CliError::Schema(format!("preload blob: {}", e)))?;
        let mut r = s.query([]).map_err(|e| CliError::Schema(e.to_string()))?;
        let mut out: HashMap<(String, String), bool> = HashMap::new();
        while let Some(row) = r.next().map_err(|e| CliError::Schema(e.to_string()))? {
            let pid: Option<String> = row.get(0).ok();
            let sha: Option<String> = row.get(1).ok();
            if let (Some(p), Some(s)) = (pid, sha) {
                out.insert((p, s), true);
            }
        }
        out
    };

    let mut to_insert: Vec<(String, String, String, Vec<u8>, String)> = Vec::new();

    while let Some(row) = rows_iter
        .next()
        .map_err(|e| CliError::Schema(format!("scan v_file: {}", e)))?
    {
        let parent_id: String = row
            .get(0)
            .map_err(|e| CliError::Schema(format!("v_file.id: {}", e)))?;
        let file_name: String = row
            .get(1)
            .map_err(|e| CliError::Schema(format!("v_file.file_name: {}", e)))?;

        let resolved = resolve_attachment_path(attachments_dir, &file_name);
        let bytes = match fs::read(&resolved) {
            Ok(b) => b,
            Err(_) => {
                stats.warnings.push(format!(
                    "missing attachment: {} (looked in {})",
                    file_name,
                    attachments_dir.display(),
                ));
                continue;
            }
        };

        let sha = sha256_hex(&bytes);
        if already.contains_key(&(parent_id.clone(), sha.clone())) {
            continue; // idempotent skip
        }
        let mime = guess_mime(&file_name).to_string();
        let basename = Path::new(&file_name)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or(&file_name)
            .to_string();
        stats.bytes_total += bytes.len() as u64;
        stats.files_processed += 1;
        to_insert.push((parent_id, sha, mime, bytes, basename));
    }
    drop(rows_iter);
    drop(stmt);

    if to_insert.is_empty() {
        return Ok(stats);
    }

    // Bulk insert via prepared statement — the blob table's PK uses a
    // sequence default, so we only supply the user-facing columns.
    let mut ins = conn
        .prepare(
            "INSERT INTO blob (parent_table, parent_id, kind, mime_type, filename, sha256, data)
             VALUES ('g_file', ?, 'attachment', ?, ?, ?, ?)",
        )
        .map_err(|e| CliError::Schema(format!("prepare blob INSERT: {}", e)))?;
    for (parent_id, sha, mime, bytes, basename) in &to_insert {
        ins.execute(duckdb::params![
            parent_id,
            mime,
            basename,
            sha,
            bytes.as_slice()
        ])
        .map_err(|e| CliError::Schema(format!("INSERT blob: {}", e)))?;
    }

    Ok(stats)
}

/// Drain attachment blobs out of `db_path`, reconstructing the AGS4
/// Rule 20 sidecar tree **`<out_dir>/FILE/<FILE_FSET>/<FILE_NAME>`**.
///
/// `blob` stores only a basename and no FSET, so the FSET and the
/// original (possibly sub-pathed) `FILE_NAME` are recovered by joining
/// each blob back to its FILE-group row via `blob.parent_id =
/// v_file.id`. A blob with no resolvable FILE row, or a FILE flavour
/// without `FILE_FSET`, falls back to a flat `<out_dir>/<basename>`
/// write **+ a warning** — nothing is silently dropped. A same-name
/// target with a *different* sha256 is skipped (+ warning): never
/// clobber what looks like a different revision.
pub fn unspool_attachments(db_path: &Path, out_dir: &Path) -> Result<AttachmentStats, CliError> {
    // Read-only — the unspool path never writes back to the DB. Sharing
    // the file with the export's read-only handle would clash anyway on
    // Windows, where DuckDB's file lock isn't shared between processes
    // even within the same binary.
    let cfg = Config::default()
        .access_mode(AccessMode::ReadOnly)
        .map_err(|e| CliError::Schema(format!("config: {}", e)))?;
    let conn = Connection::open_with_flags(db_path, cfg)
        .map_err(|e| CliError::Schema(format!("open db for unspool: {}", e)))?;
    let mut stats = AttachmentStats::default();

    let blob_present: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM information_schema.tables WHERE table_name = 'blob'",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);
    if blob_present == 0 {
        return Ok(stats);
    }

    // Can we recover FILE_FSET / FILE_NAME for the tree? Only if a
    // `v_file` view with those columns exists (passthrough FILE
    // flavours may lack either). Mirrors slurp's defensive probe.
    let vfile_exists: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM information_schema.tables WHERE table_name = 'v_file'",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);
    let (has_fset, has_name) = if vfile_exists == 1 {
        let cols: Vec<String> = {
            let mut s = conn
                .prepare("SELECT * FROM v_file LIMIT 0")
                .map_err(|e| CliError::Schema(format!("describe v_file: {}", e)))?;
            let r = s.query([]).map_err(|e| CliError::Schema(e.to_string()))?;
            let sr = r
                .as_ref()
                .ok_or_else(|| CliError::Schema("v_file statement detached".into()))?;
            (0..sr.column_count())
                .map(|i| sr.column_name(i).map(String::from).unwrap_or_default())
                .collect()
        };
        (
            cols.iter().any(|c| c == "file_fset"),
            cols.iter().any(|c| c == "file_name"),
        )
    } else {
        (false, false)
    };

    // Recover FSET (+ original FILE_NAME) per blob via the FILE row.
    // LEFT JOIN so an orphan blob still comes through (→ flat fallback).
    let sql = if has_fset {
        format!(
            "SELECT b.filename, b.sha256, b.data, f.file_fset, {} \
             FROM blob b LEFT JOIN v_file f ON b.parent_id = f.id \
             WHERE b.kind = 'attachment' AND b.filename IS NOT NULL",
            if has_name { "f.file_name" } else { "NULL" }
        )
    } else {
        "SELECT b.filename, b.sha256, b.data, NULL, NULL FROM blob b \
         WHERE b.kind = 'attachment' AND b.filename IS NOT NULL"
            .to_string()
    };
    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| CliError::Schema(format!("query blob: {}", e)))?;
    let mut rows_iter = stmt
        .query([])
        .map_err(|e| CliError::Schema(format!("query blob: {}", e)))?;

    fs::create_dir_all(out_dir).map_err(|e| {
        CliError::Schema(format!(
            "create attachments dir {}: {}",
            out_dir.display(),
            e
        ))
    })?;

    while let Some(row) = rows_iter
        .next()
        .map_err(|e| CliError::Schema(format!("read blob: {}", e)))?
    {
        let basename: String = row
            .get(0)
            .map_err(|e| CliError::Schema(format!("blob.filename: {}", e)))?;
        let expected_sha: Option<String> = row.get(1).ok();
        let data: Vec<u8> = row
            .get(2)
            .map_err(|e| CliError::Schema(format!("blob.data: {}", e)))?;
        let fset: Option<String> = row.get::<_, Option<String>>(3).ok().flatten();
        let fname: Option<String> = row.get::<_, Option<String>>(4).ok().flatten();

        // FILE/<fset>/<file_name>. Fall back to a flat basename write
        // (+ warning) when the FSET can't be recovered — never drop.
        let rel: PathBuf = match fset.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
            Some(fs_tok) => {
                let leaf = fname
                    .as_deref()
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .unwrap_or(&basename);
                let mut p = PathBuf::from("FILE");
                p.push(path_token(fs_tok));
                for seg in leaf
                    .trim_start_matches("./")
                    .trim_start_matches(".\\")
                    .split(['/', '\\'])
                {
                    if !seg.is_empty() && seg != "." {
                        p.push(seg);
                    }
                }
                p
            }
            None => {
                stats.warnings.push(format!(
                    "{basename}: no FILE_FSET resolvable (orphan blob?) — wrote flat",
                ));
                PathBuf::from(&basename)
            }
        };
        let target = out_dir.join(&rel);
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| CliError::Schema(format!("create {}: {}", parent.display(), e)))?;
        }
        if target.exists() {
            let existing = fs::read(&target).unwrap_or_default();
            let existing_sha = sha256_hex(&existing);
            let same = expected_sha
                .as_deref()
                .map(|s| s.eq_ignore_ascii_case(&existing_sha))
                .unwrap_or(false);
            if !same {
                stats.warnings.push(format!(
                    "skipped {}: target exists with different content (would clobber)",
                    target.display(),
                ));
                continue;
            }
            // Same content already on disk — count as processed, don't rewrite.
            stats.files_processed += 1;
            continue;
        }
        fs::write(&target, &data).map_err(|e| {
            CliError::Schema(format!("write attachment {}: {}", target.display(), e))
        })?;
        stats.bytes_total += data.len() as u64;
        stats.files_processed += 1;
    }
    Ok(stats)
}

/// Collapse a (defensively slash-laden) FSET token to one safe path
/// segment — FSET is normally a simple token like `FS1`.
fn path_token(s: &str) -> String {
    s.split(['/', '\\'])
        .filter(|x| !x.is_empty() && *x != ".")
        .collect::<Vec<_>>()
        .join("_")
}

/// Resolve `FILE_NAME` against `attachments_dir`. AGS4 doesn't constrain
/// slash style — assume Windows backslashes can appear on either OS.
/// Strip a leading `./` and translate any backslash to the host separator
/// so a Linux machine can still find files referenced with `attachments\foo.pdf`.
fn resolve_attachment_path(attachments_dir: &Path, file_name: &str) -> PathBuf {
    let normalised = file_name
        .trim_start_matches("./")
        .trim_start_matches(".\\")
        .replace('\\', std::path::MAIN_SEPARATOR_STR);
    attachments_dir.join(normalised)
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(bytes);
    hex::encode(h.finalize())
}

/// Best-effort mime type by extension. The blob table stores this as a
/// hint — nothing consumes it programmatically yet, but it's surfaced
/// in tooling that lists attachments.
fn guess_mime(file_name: &str) -> &'static str {
    let lower = file_name.to_ascii_lowercase();
    let ext = lower.rsplit('.').next().unwrap_or("");
    match ext {
        "pdf" => "application/pdf",
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "gif" => "image/gif",
        "txt" | "log" => "text/plain",
        "csv" => "text/csv",
        "xml" => "application/xml",
        "json" => "application/json",
        "zip" => "application/zip",
        "xls" | "xlsx" => "application/vnd.ms-excel",
        "doc" | "docx" => "application/msword",
        _ => "application/octet-stream",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn guess_mime_known_extensions() {
        assert_eq!(guess_mime("a.pdf"), "application/pdf");
        assert_eq!(guess_mime("a.PDF"), "application/pdf");
        assert_eq!(guess_mime("a.txt"), "text/plain");
        assert_eq!(guess_mime("noext"), "application/octet-stream");
    }

    #[test]
    fn resolve_strips_dot_slash_and_normalises_backslashes() {
        let base = Path::new("/tmp/attach");
        let p = resolve_attachment_path(base, "./sub\\file.pdf");
        let expected = base.join(format!("sub{}file.pdf", std::path::MAIN_SEPARATOR_STR));
        assert_eq!(p, expected);
    }
}
