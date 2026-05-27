//! File introspection — `info` (file summary + per-group row counts),
//! `list_groups` (groups in this file), and a thin wrapper around the
//! existing `db::headings_for` (group schema).
//!
//! Stage F2a-2e: extracted from `commands/{info,groups}.rs` so the
//! data work is reusable from both the CLI binary and `laterite-py`.

use std::path::Path;

use crate::conn::open_readonly;
use crate::db::{HeadingRow, classify_catalog_error, display_native, headings_for};
use ags5_core::error::CliError;

/// One group's row count + parent + contents description.
#[derive(Debug, Clone)]
pub struct GroupRow {
    pub code: String,
    pub rows: i64,
    pub parent: String,
    pub contents: String,
}

/// File summary: file path + size + format/library versions + per-group row counts.
#[derive(Debug, Clone)]
pub struct InfoSummary {
    pub file: String,
    pub size_mb: f64,
    pub format_version: Option<String>,
    pub library_version: Option<String>,
    pub groups: Vec<GroupRow>,
}

impl InfoSummary {
    pub fn n_groups(&self) -> usize {
        self.groups.len()
    }
    pub fn n_nonempty(&self) -> usize {
        self.groups.iter().filter(|g| g.rows > 0).count()
    }
}

/// File-level summary mirroring `ags5db info`.
pub fn info(db: &Path) -> Result<InfoSummary, CliError> {
    if !db.exists() {
        return Err(CliError::FileNotFound(db.display().to_string()));
    }
    let conn = open_readonly(db)?;
    let size_bytes = std::fs::metadata(db)
        .map_err(|e| CliError::Schema(format!("metadata: {}", e)))?
        .len();
    let size_mb = (size_bytes as f64 / 1_000_000.0 * 100.0).round() / 100.0;

    // Tolerate missing _spec_meta (pre-6.5 files).
    let (format_version, library_version) = conn
        .query_row(
            "SELECT format_version, library_version FROM _spec_meta",
            [],
            |row| {
                Ok((
                    row.get::<_, Option<String>>(0)?,
                    row.get::<_, Option<String>>(1)?,
                ))
            },
        )
        .unwrap_or((None, None));

    let groups = read_groups(&conn, /*nonempty=*/ false)?;
    Ok(InfoSummary {
        file: display_native(db),
        size_mb,
        format_version,
        library_version,
        groups,
    })
}

/// Every registered group in the file with row counts. `nonempty`
/// filters to groups that actually carry rows.
pub fn list_groups(db: &Path, nonempty: bool) -> Result<Vec<GroupRow>, CliError> {
    if !db.exists() {
        return Err(CliError::FileNotFound(db.display().to_string()));
    }
    let conn = open_readonly(db)?;
    read_groups(&conn, nonempty)
}

/// Schema dump for one group. Thin wrapper around `db::headings_for`
/// that opens the connection itself — the CLI command already calls
/// `headings_for` directly, but the laterite Python API needs a
/// no-connection-needed entry point.
///
/// Validates the group exists in `_spec_groups` before reading
/// headings; an unknown code returns `CliError::UnknownGroup` rather
/// than silently yielding an empty list.
pub fn headings(db: &Path, code: &str) -> Result<Vec<HeadingRow>, CliError> {
    if !db.exists() {
        return Err(CliError::FileNotFound(db.display().to_string()));
    }
    let conn = open_readonly(db)?;
    let code_upper = code.to_uppercase();
    let known: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM _spec_groups WHERE code = ?",
            [&code_upper],
            |row| row.get(0),
        )
        .map_err(|e| classify_catalog_error(e, "_spec_groups"))?;
    if known == 0 {
        return Err(CliError::UnknownGroup {
            code: code_upper,
            hints: Vec::new(),
        });
    }
    headings_for(&conn, &code_upper)
}

// --- F2a-2f: spec-tables inspector ----------------------------------

/// One group's self-describing metadata (from `_spec_groups`).
/// `index_parent` uses the Option<Option<T>> trick: outer `None` means
/// the column doesn't exist (pre-6.5.2 file); `Some(None)` means it
/// exists but is NULL; `Some(Some(v))` is the populated value.
#[derive(Debug, Clone)]
pub struct GroupBlock {
    pub code: String,
    pub contents: String,
    pub parent: String,
    pub is_high_volume: bool,
    pub index_parent: Option<Option<String>>,
}

/// One heading's self-describing metadata (from `_spec_headings`).
/// `indexed` follows the same Option<Option<bool>> convention.
#[derive(Debug, Clone)]
pub struct HeadingDetail {
    pub name: String,
    pub status: String,
    pub ags_type: String,
    pub canonical_type: String,
    pub unit: String,
    pub display_hint: String,
    pub indexed: Option<Option<bool>>,
}

/// Result of [`inspect`]: file-level meta + optional per-group block.
#[derive(Debug, Clone)]
pub struct InspectReport {
    pub format_version: String,
    pub library_version: String,
    pub written_at: String,
    pub note: String,
    pub n_groups: i64,
    pub n_headings: i64,
    pub group: Option<GroupBlock>,
    pub headings: Option<Vec<HeadingDetail>>,
}

/// Dump the file's `_spec_*` self-describing tables. With
/// `group=None`, returns scalar meta + counts; with `group=Some(code)`,
/// also fills in the group block + its headings.
pub fn inspect(db: &Path, group: Option<&str>) -> Result<InspectReport, CliError> {
    if !db.exists() {
        return Err(CliError::FileNotFound(db.display().to_string()));
    }
    let conn = open_readonly(db)?;

    // _spec_meta: classify missing as catalog error (exit 2).
    let meta = conn
        .query_row(
            "SELECT format_version, library_version,
                    CAST(written_at AS VARCHAR), note
               FROM _spec_meta",
            [],
            |row| {
                Ok((
                    row.get::<_, Option<String>>(0)?,
                    row.get::<_, Option<String>>(1)?,
                    row.get::<_, Option<String>>(2)?,
                    row.get::<_, Option<String>>(3)?,
                ))
            },
        )
        .map_err(|e| classify_catalog_error(e, "_spec_meta"))?;

    let n_groups: i64 = conn
        .query_row("SELECT COUNT(*) FROM _spec_groups", [], |row| row.get(0))
        .map_err(|e| CliError::Schema(e.to_string()))?;
    let n_headings: i64 = conn
        .query_row("SELECT COUNT(*) FROM _spec_headings", [], |row| row.get(0))
        .map_err(|e| CliError::Schema(e.to_string()))?;

    let mut report = InspectReport {
        format_version: meta.0.unwrap_or_default(),
        library_version: meta.1.unwrap_or_else(|| "(unset)".into()),
        written_at: meta.2.unwrap_or_default(),
        note: meta.3.unwrap_or_default(),
        n_groups,
        n_headings,
        group: None,
        headings: None,
    };

    if let Some(code) = group {
        let code_upper = code.to_uppercase();
        // Validate the group exists — return UnknownGroup (exit 4)
        // rather than silently emitting an empty record.
        let known: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM _spec_groups WHERE code = ?",
                [&code_upper],
                |row| row.get(0),
            )
            .map_err(|e| classify_catalog_error(e, "_spec_groups"))?;
        if known == 0 {
            return Err(CliError::UnknownGroup {
                code: code_upper,
                hints: Vec::new(),
            });
        }

        let has_index_parent = column_exists(&conn, "_spec_groups", "index_parent")?;
        let has_indexed = column_exists(&conn, "_spec_headings", "indexed")?;

        let g_sql = if has_index_parent {
            "SELECT contents, parent, is_high_volume, index_parent
               FROM _spec_groups WHERE code = ?"
        } else {
            "SELECT contents, parent, is_high_volume FROM _spec_groups WHERE code = ?"
        };
        let g_block = conn
            .query_row(g_sql, [&code_upper], |row| {
                let contents: Option<String> = row.get(0)?;
                let parent: Option<String> = row.get(1)?;
                let is_high_volume: bool = row.get(2)?;
                let index_parent: Option<Option<String>> = if has_index_parent {
                    Some(row.get::<_, Option<String>>(3)?)
                } else {
                    None
                };
                Ok(GroupBlock {
                    code: code_upper.clone(),
                    contents: contents.unwrap_or_default(),
                    parent: parent.unwrap_or_else(|| "(root)".into()),
                    is_high_volume,
                    index_parent,
                })
            })
            .map_err(|e| CliError::Schema(e.to_string()))?;

        let h_sql = if has_indexed {
            "SELECT name, status, ags_type, canonical_type, unit, display_hint, indexed
               FROM _spec_headings WHERE group_code = ? ORDER BY name"
        } else {
            "SELECT name, status, ags_type, canonical_type, unit, display_hint
               FROM _spec_headings WHERE group_code = ? ORDER BY name"
        };
        let mut stmt = conn
            .prepare(h_sql)
            .map_err(|e| CliError::Schema(e.to_string()))?;
        let headings: Vec<HeadingDetail> = stmt
            .query_map([&code_upper], |row| {
                Ok(HeadingDetail {
                    name: row.get(0)?,
                    status: row.get(1)?,
                    ags_type: row.get(2)?,
                    canonical_type: row.get(3)?,
                    unit: row.get::<_, Option<String>>(4)?.unwrap_or_default(),
                    display_hint: row.get::<_, Option<String>>(5)?.unwrap_or_default(),
                    indexed: if has_indexed {
                        Some(row.get::<_, Option<bool>>(6)?)
                    } else {
                        None
                    },
                })
            })
            .map_err(|e| CliError::Schema(e.to_string()))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| CliError::Schema(e.to_string()))?;

        report.group = Some(g_block);
        report.headings = Some(headings);
    }

    Ok(report)
}

fn column_exists(conn: &duckdb::Connection, table: &str, column: &str) -> Result<bool, CliError> {
    let n: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM information_schema.columns
              WHERE table_name = ? AND column_name = ?",
            [table, column],
            |row| row.get(0),
        )
        .map_err(|e| CliError::Schema(e.to_string()))?;
    Ok(n > 0)
}

// --- group listing helpers (used by `info` + `list_groups`) ---------

fn read_groups(conn: &duckdb::Connection, nonempty: bool) -> Result<Vec<GroupRow>, CliError> {
    let mut stmt = conn
        .prepare(
            "SELECT g.code,
                    COALESCE(t.estimated_size, 0) AS rows,
                    COALESCE(g.parent, '') AS parent,
                    COALESCE(g.contents, '') AS contents
               FROM _spec_groups g
               LEFT JOIN duckdb_tables() t
                 ON t.table_name = 'g_' || lower(g.code)
              ORDER BY rows DESC, g.code",
        )
        .map_err(|e| classify_catalog_error(e, "_spec_groups"))?;
    let mut out: Vec<GroupRow> = Vec::new();
    let mut rs = stmt
        .query([])
        .map_err(|e| CliError::Schema(e.to_string()))?;
    while let Some(row) = rs.next().map_err(|e| CliError::Schema(e.to_string()))? {
        let code: String = row.get(0).map_err(|e| CliError::Schema(e.to_string()))?;
        let rows: i64 = row.get(1).map_err(|e| CliError::Schema(e.to_string()))?;
        if nonempty && rows == 0 {
            continue;
        }
        let parent: String = row.get(2).map_err(|e| CliError::Schema(e.to_string()))?;
        let contents: String = row.get(3).map_err(|e| CliError::Schema(e.to_string()))?;
        out.push(GroupRow {
            code,
            rows,
            parent,
            contents,
        });
    }
    Ok(out)
}
