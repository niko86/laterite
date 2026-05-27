//! Self-describing `_spec_*` tables — port of `ags5_db._spec_tables`.
//!
//! Three tables populated at write time so a downstream consumer can
//! introspect the file without the Python (or Rust) registry:
//!
//!   _spec_meta       one-row table: format version, library version,
//!                    written-at, note
//!   _spec_groups     one row per registered group code
//!   _spec_headings   one row per heading in every group
//!
//! The Python version uses Polars → Arrow → DuckDB for bulk insert. In
//! Rust we use parameterised INSERTs in a single transaction — the spec
//! tables are at most a few thousand rows, fast enough without an
//! Arrow round-trip.

use chrono::Utc;
use duckdb::Connection;

use ags5_core::ags_types::{canonical_type, display_hint};
use ags5_core::error::CliError;
use ags5_core::registry::Registry;

const FORMAT_VERSION: &str = "6.5.0";

const DDL: &str = "
CREATE TABLE IF NOT EXISTS _spec_meta (
    format_version VARCHAR NOT NULL,
    library_version VARCHAR,
    written_at TIMESTAMP NOT NULL DEFAULT current_timestamp,
    note VARCHAR
);

CREATE TABLE IF NOT EXISTS _spec_groups (
    code VARCHAR PRIMARY KEY,
    contents VARCHAR NOT NULL,
    parent VARCHAR,
    is_high_volume BOOLEAN NOT NULL DEFAULT FALSE,
    index_parent BOOLEAN
);

CREATE TABLE IF NOT EXISTS _spec_headings (
    group_code VARCHAR NOT NULL,
    name VARCHAR NOT NULL,
    status VARCHAR NOT NULL,
    ags_type VARCHAR NOT NULL,
    canonical_type VARCHAR NOT NULL,
    unit VARCHAR,
    description VARCHAR,
    display_hint VARCHAR,
    indexed BOOLEAN,
    PRIMARY KEY (group_code, name)
);
";

pub fn ensure_spec_tables(conn: &Connection) -> Result<(), CliError> {
    conn.execute_batch(DDL)
        .map_err(|e| CliError::Schema(format!("create _spec_*: {}", e)))?;
    Ok(())
}

/// Write the in-process registry into the spec tables. Truncates first
/// so the spec reflects exactly the registry at write time.
///
/// `library_version` is optional (callers may pass `None` — the column is
/// nullable on disk). `note` is a free-form string surfaced by `inspect`.
pub fn write_spec(
    conn: &Connection,
    reg: &Registry,
    library_version: Option<&str>,
    note: Option<&str>,
) -> Result<(), CliError> {
    ensure_spec_tables(conn)?;

    conn.execute("DELETE FROM _spec_meta", [])
        .map_err(|e| CliError::Schema(format!("clear _spec_meta: {}", e)))?;
    conn.execute("DELETE FROM _spec_headings", [])
        .map_err(|e| CliError::Schema(format!("clear _spec_headings: {}", e)))?;
    conn.execute("DELETE FROM _spec_groups", [])
        .map_err(|e| CliError::Schema(format!("clear _spec_groups: {}", e)))?;

    // _spec_meta: one row. written_at stays in UTC to match Python's
    // `datetime.now(tz=datetime.UTC)`. DuckDB stores as TIMESTAMP with
    // microsecond precision; an explicit ISO string is the safest bind.
    let now = Utc::now().format("%Y-%m-%d %H:%M:%S%.6f").to_string();
    conn.execute(
        "INSERT INTO _spec_meta (format_version, library_version, written_at, note)
         VALUES (?, ?, CAST(? AS TIMESTAMP), ?)",
        duckdb::params![FORMAT_VERSION, library_version, now, note],
    )
    .map_err(|e| CliError::Schema(format!("insert _spec_meta: {}", e)))?;

    // _spec_groups: one row per group, batched via Appender for speed.
    let mut app = conn
        .appender("_spec_groups")
        .map_err(|e| CliError::Schema(format!("open _spec_groups appender: {}", e)))?;
    for g in reg.iter() {
        app.append_row(duckdb::params![
            g.code,
            g.contents,
            g.parent.as_deref(),
            g.is_high_volume,
            g.index_parent,
        ])
        .map_err(|e| CliError::Schema(format!("append _spec_groups: {}", e)))?;
    }
    app.flush()
        .map_err(|e| CliError::Schema(format!("flush _spec_groups: {}", e)))?;
    drop(app);

    // _spec_headings: same pattern, one row per heading per group.
    let mut app = conn
        .appender("_spec_headings")
        .map_err(|e| CliError::Schema(format!("open _spec_headings appender: {}", e)))?;
    for g in reg.iter() {
        for h in &g.headings {
            let ct = canonical_type(&h.ags_type)
                .map(|c| c.as_str().to_string())
                .unwrap_or_else(|| "string".to_string());
            let hint = display_hint(&h.ags_type);
            app.append_row(duckdb::params![
                g.code,
                h.name,
                h.status,
                h.ags_type,
                ct,
                h.unit.as_deref(),
                h.description.as_str(),
                hint.as_deref(),
                h.indexed,
            ])
            .map_err(|e| CliError::Schema(format!("append _spec_headings: {}", e)))?;
        }
    }
    app.flush()
        .map_err(|e| CliError::Schema(format!("flush _spec_headings: {}", e)))?;

    Ok(())
}
