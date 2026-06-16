//! CLI-dep-free read-side query API over a `.ags5db` — the counterpart
//! to `convert.rs`, and the lib surface `laterite-py` binds for the
//! Python query API (PR-B3 of
//!
//! Each fn opens the DB read-only, resolves + validates the group and
//! any `--where` predicates against the file's `_spec_*` tables (so a
//! typo is a typed error *before* it reaches SQL), builds a
//! **parameterised** SELECT (values bound via `?` — no injection
//! surface), and returns a structured result. The bin's `commands/*`
//! wrap these with clap arg-parsing + scalar/table rendering (max-cols
//! capping, canonical-type sub-headers, stderr hints — all presentation,
//! all bin-only); `laterite-py` wraps them for the Python API.

use std::collections::HashSet;
use std::path::Path;

use duckdb::Connection;
use serde_json::{Map, Value};

use crate::conn::open_readonly;
use crate::db::{JsonParam, Rows, fetch_rows, headings_for, resolve_db_and_group};
use crate::predicate::{Predicate, parse_many};
use laterite_core::error::CliError;

// ---------------------------------------------------------------------
// shared predicate → SQL helpers (moved from commands/count.rs so sum /
// peek and the lib query fns share one implementation)
// ---------------------------------------------------------------------

/// Predicate fields must exist in the group's `_spec_headings`. Mirrors
/// Python's `_check_field` — caught before we hit the DB so the error is
/// classified as exit 5 ("--where parse / unknown field") rather than
/// SQL-error exit 8.
pub fn validate_predicate_fields(
    conn: &Connection,
    group_code: &str,
    preds: &[Predicate],
) -> Result<(), CliError> {
    if preds.is_empty() {
        return Ok(());
    }
    let headings = headings_for(conn, group_code)?;
    let valid: HashSet<String> = headings
        .iter()
        .flat_map(|h| [h.name.to_lowercase(), h.name.to_uppercase()])
        .collect();
    for p in preds {
        let f_lo = p.field.to_lowercase();
        let f_up = p.field.to_uppercase();
        if !valid.contains(&f_lo) && !valid.contains(&f_up) {
            return Err(CliError::Predicate {
                arg: format!("{}{}{}", p.field, p.op.as_sql(), value_repr(&p.value)),
                reason: format!("unknown {} field: {:?}", group_code, p.field),
            });
        }
    }
    Ok(())
}

/// Build the `WHERE` body + bound params. The view exposes lower-case
/// column names; values go out as `?` placeholders.
pub fn build_where(preds: &[Predicate]) -> (String, Vec<Value>) {
    if preds.is_empty() {
        return (String::new(), Vec::new());
    }
    let mut parts: Vec<String> = Vec::new();
    let mut params: Vec<Value> = Vec::new();
    for p in preds {
        parts.push(format!("{} {} ?", p.field.to_lowercase(), p.op.as_sql()));
        params.push(p.value.clone());
    }
    (parts.join(" AND "), params)
}

fn value_repr(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        other => other.to_string(),
    }
}

// ---------------------------------------------------------------------
// query API
// ---------------------------------------------------------------------

/// `COUNT(*)` on a group's view, optional ANDed `--where` predicates.
pub fn count(db: &Path, group: &str, where_args: &[String]) -> Result<i64, CliError> {
    let (_db, code) = resolve_db_and_group(db, group)?;
    let predicates = parse_many(where_args)?;
    let conn = open_readonly(db)?;
    validate_predicate_fields(&conn, &code, &predicates)?;

    let (where_sql, params) = build_where(&predicates);
    let view = format!("v_{}", code.to_lowercase());
    let sql = if where_sql.is_empty() {
        format!("SELECT COUNT(*) FROM {}", view)
    } else {
        format!("SELECT COUNT(*) FROM {} WHERE {}", view, where_sql)
    };
    let wrapped: Vec<JsonParam> = params.iter().map(JsonParam).collect();
    conn.query_row(&sql, duckdb::params_from_iter(wrapped.iter()), |row| {
        row.get(0)
    })
    .map_err(|e| CliError::Sql(e.to_string()))
}

/// `SUM(field)` (cast to DOUBLE) on a group, optional `--where`. The
/// field must be a numeric heading (canonical type decimal/integer);
/// an empty SUM yields `0.0` (python-ags4 parity).
pub fn sum(db: &Path, group: &str, field: &str, where_args: &[String]) -> Result<f64, CliError> {
    let (_db, code) = resolve_db_and_group(db, group)?;
    let predicates = parse_many(where_args)?;
    let conn = open_readonly(db)?;
    validate_predicate_fields(&conn, &code, &predicates)?;

    // Validate the sum target exists and is numeric — DuckDB would
    // happily SUM a text column to NULL; we surface it as a typed
    // predicate error (exit 5) so an agent has a reason to retry.
    let headings = headings_for(&conn, &code)?;
    let h = headings
        .iter()
        .find(|h| h.name.eq_ignore_ascii_case(field))
        .ok_or_else(|| CliError::Predicate {
            arg: field.to_string(),
            reason: format!("unknown {} field", code),
        })?;
    let ct = h.canonical_type.to_lowercase();
    if ct != "decimal" && ct != "integer" {
        return Err(CliError::Predicate {
            arg: field.to_string(),
            reason: format!("field {:?} is {}, not numeric", h.name, h.canonical_type),
        });
    }

    let (where_sql, params) = build_where(&predicates);
    let view = format!("v_{}", code.to_lowercase());
    let f = field.to_lowercase();
    let sql = if where_sql.is_empty() {
        format!("SELECT CAST(SUM({}) AS DOUBLE) FROM {}", f, view)
    } else {
        format!(
            "SELECT CAST(SUM({}) AS DOUBLE) FROM {} WHERE {}",
            f, view, where_sql
        )
    };
    let wrapped: Vec<JsonParam> = params.iter().map(JsonParam).collect();
    let total: Option<f64> = conn
        .query_row(&sql, duckdb::params_from_iter(wrapped.iter()), |row| {
            row.get(0)
        })
        .map_err(|e| CliError::Sql(e.to_string()))?;
    Ok(total.unwrap_or(0.0))
}

/// Run a read-only SELECT, returning `Rows`. `limit` (when > 0) is
/// appended unless the statement already names a LIMIT; `explain` runs
/// the plan instead. The "auto-applied LIMIT" stderr hint is the bin's
/// concern, computed there from the same inputs.
pub fn sql(db: &Path, statement: &str, limit: usize, explain: bool) -> Result<Rows, CliError> {
    if !db.exists() {
        return Err(CliError::FileNotFound(db.display().to_string()));
    }
    let conn = open_readonly(db)?;
    let final_stmt = build_sql(statement, limit, explain);
    fetch_rows(&conn, &final_stmt, &[])
}

/// The final statement `sql` runs, given the raw input + flags. Exposed
/// so the bin can decide whether it auto-limited (for its hint) without
/// re-deriving the rule.
pub fn build_sql(statement: &str, limit: usize, explain: bool) -> String {
    if explain {
        return format!("EXPLAIN {}", statement);
    }
    if limit > 0 && !statement.to_uppercase().contains("LIMIT") {
        return format!("{} LIMIT {}", statement, limit);
    }
    statement.to_string()
}

/// Browse rows of a group's view: pick `fields` (None = all headings),
/// AND `--where`, `limit`/`offset`. `drop_null_cols` removes columns
/// that are NULL across every returned row (a data transform — the
/// bin's TABLE-mode auto-drop + max-cols cap + type sub-headers stay in
/// `commands/peek.rs`).
#[allow(clippy::too_many_arguments)]
pub fn peek(
    db: &Path,
    group: &str,
    fields: Option<&str>,
    where_args: &[String],
    limit: usize,
    offset: usize,
    drop_null_cols: bool,
) -> Result<Rows, CliError> {
    let (_db, code) = resolve_db_and_group(db, group)?;
    let predicates = parse_many(where_args)?;
    let conn = open_readonly(db)?;
    validate_predicate_fields(&conn, &code, &predicates)?;

    let cols: Vec<String> = match fields {
        Some(s) => s
            .split(',')
            .map(|f| f.trim().to_string())
            .filter(|f| !f.is_empty())
            .collect(),
        None => headings_for(&conn, &code)?
            .iter()
            .map(|h| h.name.to_lowercase())
            .collect(),
    };
    if cols.is_empty() {
        return Err(CliError::Predicate {
            arg: fields.unwrap_or_default().to_string(),
            reason: "no field names parsed".into(),
        });
    }
    validate_field_names(&conn, &code, &cols)?;

    let (where_sql, params) = build_where(&predicates);
    let col_list = cols
        .iter()
        .map(|f| f.to_lowercase())
        .collect::<Vec<_>>()
        .join(", ");
    let view = format!("v_{}", code.to_lowercase());
    let mut stmt = format!("SELECT {} FROM {}", col_list, view);
    if !where_sql.is_empty() {
        stmt.push_str(&format!(" WHERE {}", where_sql));
    }
    stmt.push_str(&format!(" LIMIT {}", limit));
    if offset > 0 {
        stmt.push_str(&format!(" OFFSET {}", offset));
    }

    let mut rows = fetch_rows(&conn, &stmt, &params)?;
    if drop_null_cols && !rows.records.is_empty() {
        let nulls = all_null_columns(&rows);
        if !nulls.is_empty() {
            drop_columns(&mut rows, &nulls);
        }
    }
    Ok(rows)
}

/// `--fields` names must exist in the group's headings.
pub fn validate_field_names(
    conn: &Connection,
    code: &str,
    fields: &[String],
) -> Result<(), CliError> {
    let hs = headings_for(conn, code)?;
    let valid: HashSet<String> = hs.iter().map(|h| h.name.to_lowercase()).collect();
    for f in fields {
        if !valid.contains(&f.to_lowercase()) {
            return Err(CliError::Predicate {
                arg: f.clone(),
                reason: format!("unknown {} field", code),
            });
        }
    }
    Ok(())
}

/// Columns that are NULL across every returned row.
pub fn all_null_columns(rows: &Rows) -> Vec<String> {
    rows.columns
        .iter()
        .filter(|c| {
            rows.records
                .iter()
                .all(|r| r.get(*c).is_none_or(Value::is_null))
        })
        .cloned()
        .collect()
}

/// Drop the named columns from both the column list and every record.
pub fn drop_columns(rows: &mut Rows, drop: &[String]) {
    let drop_set: HashSet<&str> = drop.iter().map(String::as_str).collect();
    rows.columns.retain(|c| !drop_set.contains(c.as_str()));
    for rec in &mut rows.records {
        retain_keys(rec, &drop_set);
    }
}

fn retain_keys(rec: &mut Map<String, Value>, drop: &HashSet<&str>) {
    let to_remove: Vec<String> = rec
        .keys()
        .filter(|k| drop.contains(k.as_str()))
        .cloned()
        .collect();
    for k in to_remove {
        rec.remove(&k);
    }
}

// ---------------------------------------------------------------------
// .ags5db structural + data-correctness validator — Rust-backed replacement
// for `laterite_ags5x.validation.validate_ags5db` (Stage F2a-2).
//
// Two checks per row:
//   abbr_unknown — every PA-typed value must appear in the file's own
//                  v_abbr set. Each heading carries its own allowed set
//                  (keyed by ABBR_HDNG).
//   dt_invalid   — every non-empty DT-typed value must parse via the
//                  same `parse_value` the AGS4 ingest uses, so the
//                  validator never disagrees with the ingest path.
// ---------------------------------------------------------------------

use laterite_core::ags_types::{CanonicalType, canonical_type, parse_value};
use laterite_core::registry::registry;

/// One validation finding: the layer surfaces the location, the
/// code, and a human-readable message. Same shape Python's
/// `ValidationError` once returned.
#[derive(Debug, Clone)]
pub struct ValidationFinding {
    pub severity: String,
    pub code: String,
    pub where_: String,
    pub message: String,
}

/// Validate a `.ags5db` file's spec-correctness. ``check_abbr`` and
/// ``check_dt`` independently enable the two layers; either off skips
/// that pass.
pub fn validate_db(
    db: &Path,
    check_abbr: bool,
    check_dt: bool,
) -> Result<Vec<ValidationFinding>, CliError> {
    if !db.exists() {
        return Err(CliError::FileNotFound(db.display().to_string()));
    }
    let conn = open_readonly(db)?;
    let reg = registry();
    let mut findings: Vec<ValidationFinding> = Vec::new();

    // Step 1: build the ABBR allow-list per heading. The file might
    // not have an ABBR group at all (e.g. a delivery that doesn't use
    // any PA-typed headings) — that's fine; we just skip ABBR checks.
    let mut abbr_index: std::collections::HashMap<String, HashSet<String>> =
        std::collections::HashMap::new();
    if check_abbr && table_present(&conn, "v_abbr")? {
        let mut stmt = conn
            .prepare("SELECT abbr_hdng, abbr_code FROM v_abbr")
            .map_err(|e| CliError::Sql(e.to_string()))?;
        let mut rows = stmt.query([]).map_err(|e| CliError::Sql(e.to_string()))?;
        while let Some(row) = rows.next().map_err(|e| CliError::Sql(e.to_string()))? {
            let hdng: Option<String> = row.get(0).ok();
            let code: Option<String> = row.get(1).ok();
            if let (Some(h), Some(c)) = (hdng, code) {
                abbr_index.entry(h).or_default().insert(c);
            }
        }
    }

    // Step 2: per registered group with PA/DT headings, sweep rows.
    for g in reg.iter() {
        let pa_headings: Vec<&laterite_core::registry::Heading> = g
            .headings
            .iter()
            .filter(|h| h.ags_type.eq_ignore_ascii_case("PA"))
            .collect();
        let dt_headings: Vec<&laterite_core::registry::Heading> = g
            .headings
            .iter()
            .filter(|h| {
                canonical_type(&h.ags_type)
                    .map(|c| c == CanonicalType::Datetime)
                    .unwrap_or(false)
            })
            .collect();

        let do_abbr = check_abbr && !pa_headings.is_empty() && !abbr_index.is_empty();
        let do_dt = check_dt && !dt_headings.is_empty();
        if !do_abbr && !do_dt {
            continue;
        }

        // Skip silently if the group's view isn't in this file (e.g. an
        // older delivery that predates the group, or just an unused
        // group). The python validator skips on CatalogException; we
        // probe first.
        let view = g.view();
        if !table_present(&conn, &view)? {
            continue;
        }

        let mut cols: Vec<&str> = Vec::new();
        if do_abbr {
            cols.extend(pa_headings.iter().map(|h| h.name.as_str()));
        }
        if do_dt {
            cols.extend(dt_headings.iter().map(|h| h.name.as_str()));
        }
        let select = cols
            .iter()
            .map(|c| format!("\"{}\"", c.to_lowercase()))
            .collect::<Vec<_>>()
            .join(", ");
        let sql = format!("SELECT {} FROM {}", select, view);

        let mut stmt = conn
            .prepare(&sql)
            .map_err(|e| CliError::Sql(e.to_string()))?;
        let mut row_iter = stmt.query([]).map_err(|e| CliError::Sql(e.to_string()))?;

        let mut row_idx: usize = 0;
        while let Some(row) = row_iter.next().map_err(|e| CliError::Sql(e.to_string()))? {
            let mut col_idx: usize = 0;
            if do_abbr {
                for h in &pa_headings {
                    let val: Option<String> = row.get(col_idx).ok();
                    col_idx += 1;
                    if let Some(v) = val.as_deref() {
                        if v.is_empty() {
                            continue;
                        }
                        let allowed = abbr_index.get(&h.name);
                        if allowed.is_none() || allowed.unwrap().contains(v) {
                            continue;
                        }
                        findings.push(ValidationFinding {
                            severity: "error".into(),
                            code: "abbr_unknown".into(),
                            where_: format!("{}[row {}].{}", g.table(), row_idx, h.name),
                            message: format!("value {:?} not in ABBR set for {}", v, h.name),
                        });
                    }
                }
            }
            if do_dt {
                for h in &dt_headings {
                    // DT values may already be typed datetimes in the
                    // DB (if the writer normalised them), in which case
                    // `row.get::<String>` returns Err. Treat that as
                    // valid — the DT column is already strongly typed,
                    // so no parse-failure can hide there.
                    let as_str: Option<String> = row.get(col_idx).ok();
                    col_idx += 1;
                    let Some(s) = as_str.as_deref() else { continue };
                    let trimmed = s.trim();
                    if trimmed.is_empty() {
                        continue;
                    }
                    if !parse_value(Some(trimmed), "DT").is_null() {
                        continue;
                    }
                    findings.push(ValidationFinding {
                        severity: "error".into(),
                        code: "dt_invalid".into(),
                        where_: format!("{}[row {}].{}", g.table(), row_idx, h.name),
                        message: format!("value {:?} is not a parseable AGS DT", trimmed),
                    });
                }
            }
            row_idx += 1;
        }
    }

    Ok(findings)
}

fn table_present(conn: &Connection, name: &str) -> Result<bool, CliError> {
    let n: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM information_schema.tables \
             WHERE table_name = ? AND table_schema = 'main'",
            [name],
            |row| row.get(0),
        )
        .map_err(|e| CliError::Sql(e.to_string()))?;
    Ok(n > 0)
}

// ---------------------------------------------------------------------
// blob table query — Rust-backed replacement for `ags5_db.blobs.list_blobs`
// (Stage F2a-2). Returns blob metadata only; the `data` BLOB column is
// excluded so callers don't accidentally materialise large payloads in
// memory. To fetch a single blob's bytes, use a follow-up `sql` query
// keyed by `id`.
// ---------------------------------------------------------------------

/// List blob rows with optional `parent_code` (e.g. "FILE") + `kind`
/// (e.g. "attachment") filters. Returns `{id, parent_code, parent_id,
/// kind, mime_type, filename, sha256, byte_length}` per row.
pub fn list_blobs(
    db: &Path,
    parent_code: Option<&str>,
    kind: Option<&str>,
) -> Result<Rows, CliError> {
    if !db.exists() {
        return Err(CliError::FileNotFound(db.display().to_string()));
    }
    let conn = open_readonly(db)?;

    let mut conditions: Vec<&str> = Vec::new();
    let mut params: Vec<Value> = Vec::new();

    let parent_table_val: String;
    if let Some(code) = parent_code {
        parent_table_val = format!("g_{}", code.to_lowercase());
        conditions.push("parent_table = ?");
        params.push(Value::from(parent_table_val.clone()));
    }
    if let Some(k) = kind {
        conditions.push("kind = ?");
        params.push(Value::from(k.to_string()));
    }

    let where_clause = if conditions.is_empty() {
        String::new()
    } else {
        format!(" WHERE {}", conditions.join(" AND "))
    };

    // Aliased columns: surface `parent_code` (just `parent_table` with
    // the `g_` prefix stripped + uppercased) so callers don't have to
    // unswizzle it client-side. `byte_length` is `octet_length(data)`
    // so we never pull the bytes themselves into the result set.
    let sql = format!(
        "SELECT \
           id, \
           UPPER(SUBSTRING(parent_table, 3)) AS parent_code, \
           parent_id, \
           kind, \
           mime_type, \
           filename, \
           sha256, \
           octet_length(data) AS byte_length \
         FROM blob{} ORDER BY id",
        where_clause
    );

    fetch_rows(&conn, &sql, &params)
}
