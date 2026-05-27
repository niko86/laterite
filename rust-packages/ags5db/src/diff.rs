//! Cross-file diff for `.ags5db` files — per-group added / removed /
//! modified row counts plus sample KEY tuples.
//!
//! Stage F2a-2d: extracted from `commands/diff.rs` so the algorithm
//! lives in the lib (callable from both the CLI binary and
//! `laterite-py`). The CLI command keeps the clap args + table-mode
//! rendering + exit-code mapping; the data work lives here.
//!
//! Algorithm (identical to the Python `ags5_db.diff` and the old inline
//! CLI version):
//!
//!   For each group code present in either file's `_spec_groups`:
//!     - read every row's KEY tuple (own KEYs + inherited KEYs from
//!       ancestors)
//!     - compute `hash(non_key_cols)` server-side via DuckDB
//!     - build `{key_tuple_str -> fingerprint}` for both files
//!     - set-diff: added (in B not A), removed (in A not B),
//!                 modified (same KEY, different fingerprint).
//!
//! Identity is the AGS KEY tuple, not the UUID surrogate — UUIDs are
//! random per write so the same conceptual row has different UUIDs
//! across files. The KEY tuple is the only cross-file-stable identifier.

use std::collections::{BTreeSet, HashMap, HashSet};
use std::path::Path;

use serde_json::Value;

use crate::conn::open_readonly;
use crate::db::{classify_catalog_error, value_to_json};
use ags5_core::error::CliError;

/// Whole-file diff result.
#[derive(Debug, Clone)]
pub struct DiffResult {
    pub changed_groups: Vec<GroupDiff>,
    pub groups_only_in_a: Vec<String>,
    pub groups_only_in_b: Vec<String>,
}

impl DiffResult {
    /// True if at least one group has any added/removed/modified rows
    /// **or** appears in only one file. Mirrors the binary's exit-1
    /// semantics.
    pub fn has_changes(&self) -> bool {
        !self.changed_groups.is_empty()
            || !self.groups_only_in_a.is_empty()
            || !self.groups_only_in_b.is_empty()
    }
}

/// One group's diff state, with sample KEY tuples for human-readable
/// renders.
#[derive(Debug, Clone)]
pub struct GroupDiff {
    pub code: String,
    pub added: usize,
    pub removed: usize,
    pub modified: usize,
    pub unchanged: usize,
    /// Sample KEY tuples (raw JSON values, ready for any caller-side
    /// formatting). Each `Vec<Value>` is the per-KEY-column values of
    /// one row. Capped at the caller's `samples` arg.
    pub sample_added: Vec<Vec<Value>>,
    pub sample_removed: Vec<Vec<Value>>,
    pub sample_modified: Vec<Vec<Value>>,
}

/// Diff two `.ags5db` files. `samples` caps the number of sample KEY
/// tuples per change category per group (set 0 to suppress).
pub fn diff_dbs(a: &Path, b: &Path, samples: usize) -> Result<DiffResult, CliError> {
    if !a.exists() {
        return Err(CliError::FileNotFound(a.display().to_string()));
    }
    if !b.exists() {
        return Err(CliError::FileNotFound(b.display().to_string()));
    }
    let a_conn = open_readonly(a)?;
    let b_conn = open_readonly(b)?;

    let a_codes = group_codes(&a_conn)?;
    let b_codes = group_codes(&b_conn)?;
    let codes: BTreeSet<String> = a_codes.iter().chain(b_codes.iter()).cloned().collect();

    let a_views: HashSet<String> = view_names(&a_conn)?;
    let b_views: HashSet<String> = view_names(&b_conn)?;

    // Parent-chain KEY lookup is per-file because two files may have
    // drifted parents for a code; cache per-file-per-code.
    let mut a_key_cache: HashMap<String, Vec<String>> = HashMap::new();
    let mut b_key_cache: HashMap<String, Vec<String>> = HashMap::new();

    let mut by_group: Vec<GroupDiff> = Vec::new();
    let mut only_in_a: Vec<String> = Vec::new();
    let mut only_in_b: Vec<String> = Vec::new();

    for code in &codes {
        let view = format!("v_{}", code.to_lowercase());
        let a_present = a_views.contains(&view);
        let b_present = b_views.contains(&view);

        if !a_present && !b_present {
            continue;
        }
        if !a_present {
            only_in_b.push(code.clone());
            continue;
        }
        if !b_present {
            only_in_a.push(code.clone());
            continue;
        }

        let a_keys = resolve_key_columns(&a_conn, code, &mut a_key_cache)?;
        let b_keys = resolve_key_columns(&b_conn, code, &mut b_key_cache)?;
        // Intersection — a column missing on one side can't be shared
        // identity. In practice the two should match.
        let key_set_a: HashSet<&String> = a_keys.iter().collect();
        let key_names: Vec<String> = b_keys
            .iter()
            .filter(|k| key_set_a.contains(*k))
            .cloned()
            .collect();
        if key_names.is_empty() {
            // No shared KEYs — can't compute comparable identity. Skip
            // rather than emit a misleading 0/0 record.
            continue;
        }

        let a_fp = fingerprint_rows(&a_conn, &view, &key_names)?;
        let b_fp = fingerprint_rows(&b_conn, &view, &key_names)?;

        let a_key_set: HashSet<&String> = a_fp.keys().collect();
        let b_key_set: HashSet<&String> = b_fp.keys().collect();
        let added: BTreeSet<&String> = b_key_set.difference(&a_key_set).copied().collect();
        let removed: BTreeSet<&String> = a_key_set.difference(&b_key_set).copied().collect();
        let common: BTreeSet<&String> = a_key_set.intersection(&b_key_set).copied().collect();
        let modified: BTreeSet<&String> = common
            .iter()
            .filter(|k| a_fp[**k].fp != b_fp[**k].fp)
            .copied()
            .collect();
        let unchanged = common.len() - modified.len();

        if added.is_empty() && removed.is_empty() && modified.is_empty() {
            continue;
        }

        let sample_added = added
            .iter()
            .take(samples)
            .map(|k| b_fp[*k].key_values.clone())
            .collect();
        let sample_removed = removed
            .iter()
            .take(samples)
            .map(|k| a_fp[*k].key_values.clone())
            .collect();
        let sample_modified = modified
            .iter()
            .take(samples)
            .map(|k| a_fp[*k].key_values.clone())
            .collect();

        by_group.push(GroupDiff {
            code: code.clone(),
            added: added.len(),
            removed: removed.len(),
            modified: modified.len(),
            unchanged,
            sample_added,
            sample_removed,
            sample_modified,
        });
    }

    // Sort changed groups by total-changes desc, then code asc.
    by_group.sort_by(|a, b| {
        let ta = a.added + a.removed + a.modified;
        let tb = b.added + b.removed + b.modified;
        tb.cmp(&ta).then_with(|| a.code.cmp(&b.code))
    });
    only_in_a.sort();
    only_in_b.sort();

    Ok(DiffResult {
        changed_groups: by_group,
        groups_only_in_a: only_in_a,
        groups_only_in_b: only_in_b,
    })
}

/// One row's fingerprint: the KEY column values (preserved for sample
/// rendering) plus the DuckDB-computed `hash(non_key_cols)` integer.
/// DuckDB's `hash()` returns UBIGINT (u64); real values regularly
/// exceed i64::MAX so the raw u64 type is load-bearing.
struct Fp {
    key_values: Vec<Value>,
    fp: u64,
}

fn group_codes(conn: &duckdb::Connection) -> Result<Vec<String>, CliError> {
    let mut stmt = conn
        .prepare("SELECT code FROM _spec_groups ORDER BY code")
        .map_err(|e| classify_catalog_error(e, "_spec_groups"))?;
    let rows = stmt
        .query_map([], |row| row.get::<_, String>(0))
        .map_err(|e| CliError::Schema(e.to_string()))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| CliError::Schema(e.to_string()))?;
    Ok(rows)
}

fn view_names(conn: &duckdb::Connection) -> Result<HashSet<String>, CliError> {
    let mut stmt = conn
        .prepare("SELECT table_name FROM information_schema.tables WHERE table_name LIKE 'v_%'")
        .map_err(|e| CliError::Schema(e.to_string()))?;
    let rows = stmt
        .query_map([], |row| row.get::<_, String>(0))
        .map_err(|e| CliError::Schema(e.to_string()))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| CliError::Schema(e.to_string()))?;
    Ok(rows.into_iter().collect())
}

/// Walk the parent chain in `_spec_groups`, accumulating KEY heading
/// names (lower-cased to match the view's column form).
fn resolve_key_columns<'a>(
    conn: &duckdb::Connection,
    code: &str,
    cache: &'a mut HashMap<String, Vec<String>>,
) -> Result<&'a Vec<String>, CliError> {
    if !cache.contains_key(code) {
        let mut seen: HashSet<String> = HashSet::new();
        let mut chain: Vec<String> = Vec::new();
        let mut stack: Vec<String> = Vec::new();
        let mut current = code.to_string();
        loop {
            let parent: Option<String> = conn
                .query_row(
                    "SELECT parent FROM _spec_groups WHERE code = ?",
                    [&current],
                    |row| row.get::<_, Option<String>>(0),
                )
                .unwrap_or_default();
            stack.push(current.clone());
            match parent {
                Some(p) if !p.is_empty() => current = p,
                _ => break,
            }
        }
        stack.reverse();
        for c in stack {
            let mut stmt = conn
                .prepare(
                    "SELECT name FROM _spec_headings
                      WHERE group_code = ? AND status = 'KEY' ORDER BY name",
                )
                .map_err(|e| classify_catalog_error(e, "_spec_headings"))?;
            let names: Vec<String> = stmt
                .query_map([&c], |row| row.get(0))
                .map_err(|e| CliError::Schema(e.to_string()))?
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| CliError::Schema(e.to_string()))?;
            for n in names {
                let lower = n.to_lowercase();
                if seen.insert(lower.clone()) {
                    chain.push(lower);
                }
            }
        }
        cache.insert(code.to_string(), chain);
    }
    Ok(cache.get(code).unwrap())
}

fn fingerprint_rows(
    conn: &duckdb::Connection,
    view: &str,
    key_names: &[String],
) -> Result<HashMap<String, Fp>, CliError> {
    let mut col_stmt = conn
        .prepare(
            "SELECT column_name FROM information_schema.columns
              WHERE table_name = ? ORDER BY ordinal_position",
        )
        .map_err(|e| CliError::Schema(e.to_string()))?;
    let view_cols: Vec<String> = col_stmt
        .query_map([view], |row| row.get(0))
        .map_err(|e| CliError::Schema(e.to_string()))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| CliError::Schema(e.to_string()))?;
    if view_cols.is_empty() {
        return Ok(HashMap::new());
    }

    // Child views only expose KEYs that JOIN-bind into the child's
    // identity, which can be a strict subset of the full ancestry.
    let view_cols_lower: HashSet<String> = view_cols.iter().map(|c| c.to_lowercase()).collect();
    let resolved_keys: Vec<String> = key_names
        .iter()
        .filter(|k| view_cols_lower.contains(k.as_str()))
        .cloned()
        .collect();
    if resolved_keys.is_empty() {
        return Ok(HashMap::new());
    }

    let key_set: HashSet<&str> = resolved_keys.iter().map(String::as_str).collect();
    let skip: HashSet<&str> = ["id", "parent_id", "_content_hash"]
        .iter()
        .copied()
        .collect();
    let non_key_cols: Vec<&String> = view_cols
        .iter()
        .filter(|c| !key_set.contains(c.as_str()) && !skip.contains(c.as_str()))
        .collect();

    let quoted_keys = resolved_keys
        .iter()
        .map(|k| format!("\"{}\"", k))
        .collect::<Vec<_>>()
        .join(", ");
    let fp_expr = if non_key_cols.is_empty() {
        "CAST(0 AS UBIGINT)".to_string()
    } else {
        let cols = non_key_cols
            .iter()
            .map(|c| format!("\"{}\"", c))
            .collect::<Vec<_>>()
            .join(", ");
        format!("hash({})", cols)
    };

    let sql = format!("SELECT {}, {} AS _fp FROM {}", quoted_keys, fp_expr, view);
    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| CliError::Sql(e.to_string()))?;
    let n_keys = resolved_keys.len();
    let mut rows_iter = stmt.query([]).map_err(|e| CliError::Sql(e.to_string()))?;

    let mut out: HashMap<String, Fp> = HashMap::new();
    while let Some(row) = rows_iter.next().map_err(|e| CliError::Sql(e.to_string()))? {
        let mut key_values: Vec<Value> = Vec::with_capacity(n_keys);
        for i in 0..n_keys {
            let v: duckdb::types::Value = row.get(i).map_err(|e| CliError::Sql(e.to_string()))?;
            key_values.push(value_to_json(v));
        }
        // Skip rows whose KEY tuple is entirely NULL (orphan/incomplete).
        if key_values.iter().all(Value::is_null) {
            continue;
        }
        let fp: u64 = row.get(n_keys).map_err(|e| CliError::Sql(e.to_string()))?;
        let key_str =
            serde_json::to_string(&key_values).map_err(|e| CliError::Sql(e.to_string()))?;
        out.insert(key_str, Fp { key_values, fp });
    }
    Ok(out)
}
