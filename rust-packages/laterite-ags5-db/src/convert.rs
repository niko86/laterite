//! CLI-dep-free conversion API over the engine — the lib surface that
//! `laterite-py` calls (Stage B of
//!
//! Each function does the *data* work only — no clap, no spinner, no
//! output rendering — and returns structured stats. The `ags5db`
//! binary's `commands/*` wrap these with arg parsing, progress, and
//! `output` rendering. The conversion logic for `ags4_to_db` /
//! `db_to_ags4` migrates here from `commands/*` incrementally.
//! `.agsx` ↔ `.ags5db` conversion retired in Stage F2a.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

use duckdb::{AccessMode, Config, Connection};
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::db::value_to_json;
use crate::ddl::build_ddl;
use crate::spec_tables::write_spec;
use crate::uuid7;
use crate::writer::topological_order;
use laterite_core::ags_types::{
    CanonicalType, ags4_str, canonical_type, parse_value, truncate_dt_to_unit,
};
use laterite_core::ags4_codec::read_ags4;
use laterite_core::ags4_writer::{EmitGroup, write_ags4};
use laterite_core::error::CliError;
use laterite_core::registry::{GroupDescriptor, Heading, Registry, inherited_key_names, registry};

// ---------------------------------------------------------------------
// ags4-to-db conversion (moved from commands/ags4_to_db.rs, PR-B0a)
//
// `.agsx` ↔ `.ags5db` conversion was retired in Stage F2a; `.agsx` is
// now a Python-only inspection helper for AGS4 files. See
// ---------------------------------------------------------------------

/// Result of [`ags4_to_db`].
#[derive(Debug, Clone)]
pub struct ConvertStats {
    pub bytes: u64,
    pub mode: &'static str,
    pub attachments: u64,
    pub attachment_bytes: u64,
    pub warnings: Vec<String>,
}

/// Convert an AGS4 transfer file into a `.ags5db`. The data side of the
/// `ags4-to-db` command: parse + DDL + typed insert (`do_convert`),
/// slurp FILE_FSET attachments, then (unless `no_compact`) a CTAS
/// rewrite. `attachments_dir` resolves FILE references (defaults to the
/// `.ags`'s parent dir).
pub fn ags4_to_db(
    ags4_path: &Path,
    db_path: &Path,
    append: bool,
    no_compact: bool,
    attachments_dir: Option<&Path>,
) -> Result<ConvertStats, CliError> {
    if !append && db_path.exists() {
        fs::remove_file(db_path).map_err(|e| CliError::Schema(format!("remove dst: {}", e)))?;
    }
    do_convert(ags4_path, db_path, append)?;

    // Slurp attachments before compact: compact rewrites the whole DB,
    // so any blob inserts after it would be lost.
    let attach_dir = attachments_dir
        .map(Path::to_path_buf)
        .or_else(|| ags4_path.parent().map(Path::to_path_buf))
        .unwrap_or_else(|| std::path::PathBuf::from("."));
    let attach_stats = crate::attachments::slurp_attachments(db_path, &attach_dir)?;

    if !no_compact {
        compact_db(db_path)?;
    }

    let bytes = fs::metadata(db_path)
        .map_err(|e| CliError::Schema(format!("stat dst: {}", e)))?
        .len();
    Ok(ConvertStats {
        bytes,
        mode: if append { "append" } else { "create" },
        attachments: attach_stats.files_processed as u64,
        attachment_bytes: attach_stats.bytes_total,
        warnings: attach_stats.warnings,
    })
}

/// Post-write CTAS rewrite, mirroring Python `_compact`. Two reasons
/// the incremental Appender output isn't space-optimal:
///   1. DuckDB picks column compression per *segment*, and Appender
///      closes each batch as a small segment with sub-optimal choices.
///   2. DuckDB's storage doesn't reclaim deleted segments, so an
///      in-place rewrite would leave the original data around.
///
/// The fix is to write to a fresh sibling file via ATTACH + CTAS,
/// then atomically rename over the original. PRIMARY KEY constraints
/// aren't preserved across CTAS but UUID7 IDs are unique by
/// construction; we re-emit every explicit index and view so query
/// behaviour is unchanged.
pub fn compact_db(path: &std::path::Path) -> Result<(), CliError> {
    use duckdb::Connection;
    let tmp = path.with_extension("compact");
    if tmp.exists() {
        fs::remove_file(&tmp).ok();
    }

    {
        let conn = Connection::open(&tmp)
            .map_err(|e| CliError::Schema(format!("open compact tmp: {}", e)))?;
        let src_str = path.to_string_lossy();
        // Pick a unique alias for the ATTACH. DuckDB names a Connection's
        // implicit catalog after the file's basename, and `ATTACH ... AS X`
        // collides with any catalog already known to the process-wide
        // instance manager — including a destination file literally named
        // `src.ags5db` previously opened by another binding (e.g. Python's
        // duckdb wheel during the same process). Uniquifying with a UUID7
        // ensures the alias is collision-free regardless of dest filename
        // or co-resident DuckDB clients.
        let alias = format!("ags5db_compact_src_{}", uuid7::mint().simple());
        // ATTACH the original read-only under that alias. Backslashes in
        // Windows paths need to be escaped for the SQL string literal.
        let attach_sql = format!(
            "ATTACH '{}' AS {} (READ_ONLY)",
            src_str.replace('\\', "\\\\").replace('\'', "''"),
            alias,
        );
        conn.execute(&attach_sql, [])
            .map_err(|e| CliError::Schema(format!("ATTACH {}: {}", alias, e)))?;

        // Sequences first (some columns DEFAULT nextval('seq')).
        let mut stmt = conn
            .prepare(&format!(
                "SELECT sequence_name FROM duckdb_sequences()
                  WHERE database_name = '{}'",
                alias,
            ))
            .map_err(|e| CliError::Schema(format!("list seqs: {}", e)))?;
        let seqs: Vec<String> = stmt
            .query_map([], |r| r.get(0))
            .map_err(|e| CliError::Schema(e.to_string()))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| CliError::Schema(e.to_string()))?;
        for seq in seqs {
            conn.execute(&format!("CREATE SEQUENCE {}", seq), [])
                .map_err(|e| CliError::Schema(format!("CREATE SEQUENCE {}: {}", seq, e)))?;
        }

        // Tables next: CREATE TABLE t AS SELECT * FROM <alias>.t.
        let mut stmt = conn
            .prepare(&format!(
                "SELECT table_name FROM duckdb_tables()
                  WHERE database_name = '{}'",
                alias,
            ))
            .map_err(|e| CliError::Schema(format!("list tables: {}", e)))?;
        let tables: Vec<String> = stmt
            .query_map([], |r| r.get(0))
            .map_err(|e| CliError::Schema(e.to_string()))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| CliError::Schema(e.to_string()))?;
        for table in tables {
            let sql = format!(
                "CREATE TABLE {} AS SELECT * FROM {}.{}",
                table, alias, table
            );
            conn.execute(&sql, [])
                .map_err(|e| CliError::Schema(format!("CTAS {}: {}", table, e)))?;
        }

        // Then indexes — replay the original DDL captured by duckdb_indexes().
        let mut stmt = conn
            .prepare(&format!(
                "SELECT sql FROM duckdb_indexes()
                  WHERE database_name = '{}' AND sql IS NOT NULL",
                alias,
            ))
            .map_err(|e| CliError::Schema(format!("list indexes: {}", e)))?;
        let idx_sqls: Vec<String> = stmt
            .query_map([], |r| r.get(0))
            .map_err(|e| CliError::Schema(e.to_string()))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| CliError::Schema(e.to_string()))?;
        for idx_sql in idx_sqls {
            conn.execute(&idx_sql, [])
                .map_err(|e| CliError::Schema(format!("CREATE INDEX: {}", e)))?;
        }

        // Finally views.
        let mut stmt = conn
            .prepare(&format!(
                "SELECT sql FROM duckdb_views()
                  WHERE database_name = '{}' AND internal = false",
                alias,
            ))
            .map_err(|e| CliError::Schema(format!("list views: {}", e)))?;
        let view_sqls: Vec<String> = stmt
            .query_map([], |r| r.get(0))
            .map_err(|e| CliError::Schema(e.to_string()))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| CliError::Schema(e.to_string()))?;
        for view_sql in view_sqls {
            conn.execute(&view_sql, [])
                .map_err(|e| CliError::Schema(format!("CREATE VIEW: {}", e)))?;
        }

        conn.execute("CHECKPOINT", [])
            .map_err(|e| CliError::Schema(format!("CHECKPOINT: {}", e)))?;
    } // conn drops, closing both files

    // Atomic-ish replace: remove original, rename tmp.
    fs::remove_file(path).map_err(|e| CliError::Schema(format!("remove pre-compact: {}", e)))?;
    fs::rename(&tmp, path).map_err(|e| CliError::Schema(format!("rename compact tmp: {}", e)))?;
    Ok(())
}

/// Codec state carried across per-group inserts.
///
/// `lookup` maps `(parent_code, child_code, shared_key_tuple_str) →
/// parent_uuid`. The shared key tuple is the intersection of the parent's
/// KEY headings with the child's KEY headings, serialised to a canonical
/// string so it hashes (the actual tuple can't because of f64). When a
/// parent row gets inserted, it indexes itself for every descendant
/// code's shared-key shape.
///
/// `content_dedup[code] → {row_hash → uuid}` is the cross-file content-
/// hash dedup: identical rows (by all heading values, NOT just KEYs)
/// re-use the existing UUID without inserting again. Important even on
/// fresh writes when the AGS4 file itself has duplicate DATA rows
/// (which happens with ABBR lookup tables).
struct CodecCtx {
    lookup: HashMap<(String, String, String), String>,
    content_dedup: HashMap<String, HashMap<String, String>>,
}

impl CodecCtx {
    fn new() -> Self {
        Self {
            lookup: HashMap::new(),
            content_dedup: HashMap::new(),
        }
    }
}

/// Build `GroupDescriptor` entries for any groups in `parsed` that
/// aren't already in the static registry. Mirrors Python's
/// `_register_passthrough`:
///
///   * parent defaults to LOCA (round-trip works either way; the AGS
///     keys preserve the actual relationship in the data — the parent
///     hint only affects DB schema layout).
///   * status defaults to OTHER on every heading.
///   * AGS type per heading comes from the TYPE row of the AGS4 file;
///     unknown / empty values fall through to "X" (text).
///   * `is_high_volume` defaults to false — passthroughs never go down
///     the L-group CSV path.
fn build_passthrough_descriptors(
    reg: &Registry,
    parsed: &laterite_core::ags4_codec::ParsedAgs4,
) -> Vec<GroupDescriptor> {
    let mut out: Vec<GroupDescriptor> = Vec::new();
    for code in &parsed.order {
        if reg.get(code).is_some() {
            continue;
        }
        let group = match parsed.groups.get(code) {
            Some(g) => g,
            None => continue,
        };
        let headings: Vec<Heading> = group
            .headings
            .iter()
            .enumerate()
            .map(|(i, name)| {
                let ags_type = group
                    .types
                    .get(i)
                    .map(|s| s.as_str())
                    .filter(|s| !s.is_empty())
                    .unwrap_or("X")
                    .to_string();
                Heading {
                    name: name.clone(),
                    status: "OTHER".into(),
                    ags_type,
                    unit: group.units.get(i).cloned().filter(|s| !s.is_empty()),
                    description: String::new(),
                    indexed: None,
                }
            })
            .collect();
        out.push(GroupDescriptor {
            code: code.clone(),
            contents: format!("(passthrough) {}", code),
            parent: Some("LOCA".into()),
            headings,
            is_high_volume: false,
            index_parent: None,
        });
    }
    out
}

/// Reconstruct what `CodecCtx` would look like after inserting every
/// existing row in `conn` — used by `--append` so new rows resolve
/// parent UUIDs through existing parents and skip on content-hash dedup.
///
/// Mirrors Python `ags5_db._merge.load_ags4_codec_state`:
///   * `content_dedup[code][hash] = uuid` — preloaded from `_content_hash`
///   * `lookup[(parent_code, child_code, shared_str)] = uuid` — built by
///     formatting each parent row's KEY values back to AGS4-string form
///     via `ags4_str()` so on-disk values match new-row raw strings.
fn preload_codec_state(conn: &Connection, reg: &Registry) -> Result<CodecCtx, CliError> {
    use crate::db::value_to_json;
    use laterite_core::ags_types::ags4_str;

    let mut ctx = CodecCtx::new();

    // Pre-compute, for each parent code, the per-descendant shared-key
    // shape we'll need to index rows under. Mirrors Python's
    // `descendants_of` dict.
    let mut descendants_of: HashMap<String, Vec<(String, Vec<&Heading>)>> = HashMap::new();
    for g in reg.iter() {
        let mut own: Vec<(String, Vec<&Heading>)> = Vec::new();
        for cg in reg.iter() {
            if cg.parent.as_deref() != Some(&g.code) {
                continue;
            }
            let cg_key_names: HashSet<&str> = cg.key_headings().map(|h| h.name.as_str()).collect();
            let shared: Vec<&Heading> = g
                .key_headings()
                .filter(|h| cg_key_names.contains(h.name.as_str()))
                .collect();
            own.push((cg.code.clone(), shared));
        }
        descendants_of.insert(g.code.clone(), own);
    }

    for g in reg.iter() {
        let key_headings: Vec<&Heading> = g.key_headings().collect();
        let view = g.view();

        // Build the SELECT: id, _content_hash, [each KEY heading py_name].
        let mut cols: Vec<String> = vec!["id".into(), "_content_hash".into()];
        for h in &key_headings {
            cols.push(h.py_name());
        }
        let sql = format!("SELECT {} FROM {}", cols.join(", "), view);
        let mut stmt = match conn.prepare(&sql) {
            Ok(s) => s,
            Err(_) => continue, // view doesn't exist (no rows) — skip
        };
        let mut rows_iter = match stmt.query([]) {
            Ok(r) => r,
            Err(_) => continue,
        };

        let mut index: HashMap<String, String> = HashMap::new();
        let descendants = descendants_of.get(&g.code).cloned().unwrap_or_default();

        while let Some(row) = rows_iter
            .next()
            .map_err(|e| CliError::Schema(format!("preload {}: {}", view, e)))?
        {
            let row_uuid: Option<String> = row
                .get(0)
                .map_err(|e| CliError::Schema(format!("preload id: {}", e)))?;
            let row_uuid = match row_uuid {
                Some(s) => s,
                None => continue,
            };
            let content_hash: Option<String> = row
                .get(1)
                .map_err(|e| CliError::Schema(format!("preload hash: {}", e)))?;
            if let Some(h) = content_hash.filter(|s| !s.is_empty()) {
                index.insert(h, row_uuid.clone());
            }

            // Read each KEY value as a typed JSON Value, then format
            // back to its AGS4-string form via `ags4_str` so the lookup
            // key matches what new-row raw strings produce.
            let mut key_map: HashMap<String, String> = HashMap::with_capacity(key_headings.len());
            for (i, h) in key_headings.iter().enumerate() {
                let v: duckdb::types::Value = row.get(2 + i).map_err(|e| {
                    CliError::Schema(format!("preload {}.{}: {}", view, h.py_name(), e))
                })?;
                let json_v = value_to_json(v);
                key_map.insert(h.name.clone(), ags4_str(&json_v, &h.ags_type));
            }

            for (child_code, shared_headings) in &descendants {
                let parts: Vec<String> = shared_headings
                    .iter()
                    .map(|sh| key_map.get(&sh.name).cloned().unwrap_or_default())
                    .collect();
                let shared_tuple = parts.join("\n\0");
                ctx.lookup
                    .entry((g.code.clone(), child_code.clone(), shared_tuple))
                    .or_insert_with(|| row_uuid.clone());
            }
        }

        if !index.is_empty() {
            ctx.content_dedup.insert(g.code.clone(), index);
        }
    }

    Ok(ctx)
}

fn do_convert(
    ags4_path: &std::path::Path,
    db_path: &std::path::Path,
    append: bool,
) -> Result<(), CliError> {
    let parsed = read_ags4(ags4_path)?;

    // Detect unknown groups → build passthrough descriptors so they
    // can be ingested with all-string headings. Matches Python's
    // `_register_passthrough`: parent defaults to LOCA, every heading
    // is status=OTHER. The TYPE row from the AGS4 file gives us the
    // per-heading AGS code; unknown codes fall through to "X" (text).
    let static_reg = registry();
    let passthroughs = build_passthrough_descriptors(static_reg, &parsed);
    let session_reg: Registry;
    let reg: &Registry = if passthroughs.is_empty() {
        static_reg
    } else {
        session_reg = static_reg.extended_with(passthroughs.clone());
        &session_reg
    };

    // Convert parsed AGS4 → bucketed row maps then hand to the shared
    // writer. Extracting this lets the F2b-5 typed-graph write_db path
    // reuse exactly the same insertion machinery from a PROJ-walk
    // bucket source.
    let buckets: HashMap<String, Vec<HashMap<String, String>>> = parsed
        .order
        .iter()
        .filter_map(|code| {
            parsed
                .groups
                .get(code)
                .filter(|g| !g.rows.is_empty())
                .map(|g| (code.clone(), g.rows.clone()))
        })
        .collect();

    write_buckets(db_path, reg, &buckets, append)
}

/// Write a registry + bucketed rows into a `.ags5db`. Shared entry
/// point for the AGS4 converter (`do_convert`) and the typed-graph
/// writer (`laterite-py`'s F2b-5 `write_db`).
///
/// `buckets[code]` is a list of `HashMap<HEADING, value-string>`
/// rows; heading keys are UPPERCASE AGS4 names, values are AGS4-style
/// strings (caller is responsible for `Some(typed) → AGS4 string`
/// formatting). The function:
///
///   1. Opens / creates the DB and applies DDL from `reg`.
///   2. Optionally preloads dedup state from existing rows (`append=true`).
///   3. Walks groups in topological order, calling `insert_group_rows`
///      per group (UUID7 mint + content-hash dedup + parent-id
///      resolution + bulk appender insert).
///   4. Writes the `_spec_*` self-describing tables from `reg`.
pub fn write_buckets(
    db_path: &std::path::Path,
    reg: &Registry,
    buckets: &HashMap<String, Vec<HashMap<String, String>>>,
    append: bool,
) -> Result<(), CliError> {
    let conn =
        Connection::open(db_path).map_err(|e| CliError::Schema(format!("open dst: {}", e)))?;
    conn.execute_batch(&build_ddl(reg))
        .map_err(|e| CliError::Schema(format!("apply DDL: {}", e)))?;

    let mut ctx = if append {
        preload_codec_state(&conn, reg)?
    } else {
        CodecCtx::new()
    };

    // Topo order across the codes actually present, plus their ancestors
    // (the ancestor might not be in this source but its UUID may need
    // resolving for later groups — though in practice if PROJ is missing
    // we'll get an orphan parent_id, matching Python).
    let codes_present: HashSet<String> = buckets.keys().cloned().collect();
    let all_topo = topological_order(reg);
    let topo: Vec<String> = all_topo
        .into_iter()
        .filter(|c| codes_present.contains(c))
        .collect();

    for code in topo {
        if let Some(rows) = buckets.get(&code)
            && !rows.is_empty()
        {
            insert_group_rows(&conn, reg, &code, rows, &mut ctx)?;
        }
    }

    write_spec(&conn, reg, None, None)?;
    Ok(())
}

fn insert_group_rows(
    conn: &Connection,
    reg: &Registry,
    code: &str,
    rows: &[HashMap<String, String>],
    ctx: &mut CodecCtx,
) -> Result<(), CliError> {
    let g = reg
        .get(code)
        .ok_or_else(|| CliError::Schema(format!("unknown group: {}", code)))?;
    let inherited = inherited_key_names(reg, g);
    let own_keys: Vec<&Heading> = g
        .key_headings()
        .filter(|h| !inherited.contains(&h.name))
        .collect();
    let non_keys: Vec<&Heading> = g.non_key_headings().collect();

    // Descendant shared-keys: for each child group, the intersection of
    // its KEY headings with ours. We'll index every inserted row under
    // each descendant code's shared-key shape so the children find us.
    let descendants: Vec<&GroupDescriptor> = reg
        .iter()
        .filter(|cg| cg.parent.as_deref() == Some(code))
        .collect();
    let descendant_shared_keys: HashMap<String, Vec<&Heading>> = descendants
        .iter()
        .map(|cg| {
            let cg_key_names: HashSet<&str> = cg.key_headings().map(|h| h.name.as_str()).collect();
            let shared: Vec<&Heading> = g
                .key_headings()
                .filter(|h| cg_key_names.contains(h.name.as_str()))
                .collect();
            (cg.code.clone(), shared)
        })
        .collect();

    // Column order for the Appender — matches DDL.
    let mut cols: Vec<(String, String)> = Vec::new();
    cols.push(("id".into(), "ID".into()));
    cols.push(("parent_id".into(), "ID".into()));
    for h in &own_keys {
        cols.push((h.name.clone(), h.ags_type.clone()));
    }
    for h in &non_keys {
        cols.push((h.name.clone(), h.ags_type.clone()));
    }
    cols.push(("_content_hash".into(), "X".into()));

    let seen = ctx.content_dedup.entry(code.to_string()).or_default();

    let mut app = conn
        .appender(&g.table())
        .map_err(|e| CliError::Schema(format!("open {} appender: {}", g.table(), e)))?;

    for row in rows {
        // Parent_id via shared-keys intersection lookup.
        let parent_uuid = resolve_parent_uuid(reg, g, code, row, &ctx.lookup);

        // Content hash over ALL heading values, before type coercion —
        // matches Python: identical raw rows dedup, "0.0020" stays
        // distinct from "0.002" across deliveries.
        let mut hasher = Sha256::new();
        let mut keys: Vec<&String> = g.headings.iter().map(|h| &h.name).collect();
        keys.sort(); // deterministic order
        for k in &keys {
            let v = row.get(k.as_str()).map(String::as_str).unwrap_or("");
            hasher.update(k.as_bytes());
            hasher.update([0u8]);
            hasher.update(v.as_bytes());
            hasher.update([0u8]);
        }
        let content_hash = hex::encode(hasher.finalize());

        if let Some(existing_uuid) = seen.get(&content_hash) {
            // Re-index for descendants so they still find us, then skip.
            for (child_code, shared) in &descendant_shared_keys {
                let shared_tuple = encode_shared_tuple(row, shared);
                ctx.lookup
                    .entry((code.to_string(), child_code.clone(), shared_tuple))
                    .or_insert_with(|| existing_uuid.clone());
            }
            continue;
        }

        let row_uuid = uuid7::mint().to_string();
        seen.insert(content_hash.clone(), row_uuid.clone());

        // Append: id, parent_id, own keys, non-keys, _content_hash.
        let mut row_vals: Vec<duckdb::types::Value> = Vec::with_capacity(cols.len());
        row_vals.push(duckdb::types::Value::Text(row_uuid.clone()));
        row_vals.push(match parent_uuid {
            Some(p) => duckdb::types::Value::Text(p),
            None => duckdb::types::Value::Null,
        });
        for h in &own_keys {
            let raw = row.get(&h.name).map(String::as_str);
            let v = parse_value(raw, &h.ags_type);
            row_vals.push(json_to_duck(v, &h.ags_type));
        }
        for h in &non_keys {
            let raw = row.get(&h.name).map(String::as_str);
            let v = parse_value(raw, &h.ags_type);
            row_vals.push(json_to_duck(v, &h.ags_type));
        }
        row_vals.push(duckdb::types::Value::Text(content_hash.clone()));

        app.append_row(duckdb::appender_params_from_iter(row_vals.iter()))
            .map_err(|e| CliError::Schema(format!("append {} row: {}", g.table(), e)))?;

        // Index this row for every descendant shape.
        for (child_code, shared) in &descendant_shared_keys {
            let shared_tuple = encode_shared_tuple(row, shared);
            ctx.lookup
                .entry((code.to_string(), child_code.clone(), shared_tuple))
                .or_insert_with(|| row_uuid.clone());
        }
    }

    app.flush()
        .map_err(|e| CliError::Schema(format!("flush {} appender: {}", g.table(), e)))?;
    Ok(())
}

/// Look up the parent UUID for `row` using the shared-keys intersection.
///
/// The DuckDB-free base path mirrors this exactly: `laterite.ags4.read_typed`
/// (`_resolve_parent` / `_descendant_shared` in `ags4.py`) ports the same
/// intersection so it can build the typed tree without DuckDB. If you change
/// the algorithm here, the parity test
/// (`packages/laterite-ags5/tests/test_read_typed_parity.py`, which compares
/// the two paths across every reachable group) will fail until the port
/// follows.
fn resolve_parent_uuid(
    reg: &Registry,
    g: &GroupDescriptor,
    child_code: &str,
    row: &HashMap<String, String>,
    lookup: &HashMap<(String, String, String), String>,
) -> Option<String> {
    let parent_code = g.parent.as_deref()?;
    let parent_g = reg.get(parent_code)?;
    let g_key_names: HashSet<&str> = g.key_headings().map(|h| h.name.as_str()).collect();
    let shared: Vec<&Heading> = parent_g
        .key_headings()
        .filter(|h| g_key_names.contains(h.name.as_str()))
        .collect();
    let shared_tuple = encode_shared_tuple(row, &shared);
    lookup
        .get(&(
            parent_code.to_string(),
            child_code.to_string(),
            shared_tuple,
        ))
        .cloned()
}

/// Encode a tuple of (heading_name → row value) lookups into a canonical
/// string suitable for HashMap keys. Empty values are preserved as empty
/// strings so a partial-key row doesn't match a complete-key row.
fn encode_shared_tuple(row: &HashMap<String, String>, headings: &[&Heading]) -> String {
    let parts: Vec<String> = headings
        .iter()
        .map(|h| row.get(&h.name).cloned().unwrap_or_default())
        .collect();
    // Use a sentinel that can't appear inside an AGS4 KEY value (LF + NUL).
    parts.join("\n\0")
}

/// JSON → DuckDB Value, sized to the destination canonical type.
/// Duplicated from `writer.rs` so this module stays consumable on its own;
/// the two implementations track the same canonical-type table.
fn json_to_duck(v: Value, ags_type: &str) -> duckdb::types::Value {
    use duckdb::types::Value as D;
    if v.is_null() {
        return D::Null;
    }
    let ct = canonical_type(ags_type).unwrap_or(CanonicalType::String);
    match ct {
        CanonicalType::String | CanonicalType::Enum => match v {
            Value::String(s) => D::Text(s),
            other => D::Text(other.to_string()),
        },
        CanonicalType::Integer => v
            .as_i64()
            .map(D::BigInt)
            .or_else(|| v.as_f64().map(|f| D::BigInt(f as i64)))
            .unwrap_or(D::Null),
        CanonicalType::Decimal => v.as_f64().map(D::Double).unwrap_or(D::Null),
        CanonicalType::Datetime | CanonicalType::Date | CanonicalType::Time => match v {
            Value::String(s) => D::Text(s),
            _ => D::Null,
        },
        CanonicalType::Bool => match v {
            Value::Bool(b) => D::Boolean(b),
            _ => D::Null,
        },
    }
}

// ---------------------------------------------------------------------
// db-to-ags4 export (moved from commands/db_to_ags4.rs, PR-B0a)
// ---------------------------------------------------------------------

/// Result of [`db_to_ags4`] — the data-side export stats. Attachment
/// unspooling and the optional post-write validation stay in the bin's
/// `commands/db_to_ags4.rs` (they're orchestration, not data work, and
/// already have lib-accessible primitives: `crate::attachments` and the
/// `laterite_ags4_validator` crate).
#[derive(Debug, Clone)]
pub struct ExportStats {
    pub groups_emitted: usize,
    pub rows_emitted: usize,
    pub warnings: Vec<String>,
}

/// Export a `.ags5db` back to AGS4 plaintext. The data side of the
/// `db-to-ags4` command: read each group's `v_<code>` view in registry
/// order, format cells back to their AGS4 string form, and write the
/// flat GROUP/HEADING/UNIT/TYPE/DATA sections. Bails (UnsupportedFeature)
/// on any RL-typed heading (Rule 11 record links are unscoped) and
/// collects Rule 13/14/15/16 advisory warnings.
pub fn db_to_ags4(
    db_path: &std::path::Path,
    out_path: &std::path::Path,
) -> Result<ExportStats, CliError> {
    let cfg = Config::default()
        .access_mode(AccessMode::ReadOnly)
        .map_err(|e| CliError::Schema(format!("config: {}", e)))?;
    let conn = Connection::open_with_flags(db_path, cfg)
        .map_err(|e| CliError::Schema(format!("open src: {}", e)))?;
    let reg = registry();

    // Pre-flight: scan every heading present in this DB for Record Link
    // type. We check `_spec_headings` (the schema-at-write snapshot) so
    // passthrough groups registered at AGS4-ingest time are covered too
    // — they don't appear in the static registry. AGS4.1 Rule 11 demands
    // RL values be self-describing references; emitting them without
    // that resolver is misleading, so we bail loud rather than ship a
    // half-output file the user might mistake for valid.
    if let Ok(mut stmt) =
        conn.prepare("SELECT group_code, name FROM _spec_headings WHERE upper(ags_type) = 'RL'")
    {
        let rl_hits: Vec<(String, String)> = stmt
            .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))
            .map_err(|e| CliError::Schema(format!("scan _spec_headings: {}", e)))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| CliError::Schema(format!("scan _spec_headings: {}", e)))?;
        if let Some((gc, hn)) = rl_hits.first() {
            return Err(CliError::UnsupportedFeature(format!(
                "AGS4 Record Link headings (RL type) at {}.{}{}: \
                 record-link handling is unscoped, not emitting. \
                 Future work: ID + relationship resolver. See \
                 `laterite-ags4-validator` skill for spec context.",
                gc,
                hn,
                if rl_hits.len() > 1 {
                    format!(" (+{} more)", rl_hits.len() - 1)
                } else {
                    String::new()
                },
            )));
        }
    }

    // Collect rows per group, in registry order. Empty groups are
    // skipped (matches Python). We materialise everything before the
    // first write so any read-side error surfaces before the output
    // file is created — fail-loud rather than fail-with-partial-file.
    let mut emit_groups: Vec<OwnedEmitGroup> = Vec::new();
    let mut codes_present: HashSet<String> = HashSet::new();
    let mut proj_row_count: usize = 0;
    for g in reg.iter() {
        let view = g.view();
        // Skip if the view doesn't exist (group not in this DB).
        let present: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM information_schema.tables WHERE table_name = ?",
                [&view],
                |r| r.get(0),
            )
            .unwrap_or(0);
        if present == 0 {
            continue;
        }
        let rows = read_view_rows(&conn, g)?;
        if rows.is_empty() {
            continue;
        }
        codes_present.insert(g.code.clone());
        if g.code == "PROJ" {
            proj_row_count = rows.len();
        }
        emit_groups.push(OwnedEmitGroup {
            code: g.code.clone(),
            headings: g.headings.iter().map(|h| h.name.clone()).collect(),
            units: g
                .headings
                .iter()
                .map(|h| h.unit.clone().unwrap_or_default())
                .collect(),
            types: g.headings.iter().map(|h| h.ags_type.clone()).collect(),
            rows,
        });
    }

    // Build the warning list before opening the output — same fail-loud
    // principle as the RL check. None of these are blockers; they're
    // surfaced so the user knows what to expect when running the AGS4
    // validator on the result.
    let mut warnings: Vec<String> = Vec::new();
    if !codes_present.contains("TRAN") {
        warnings.push(
            "TRAN group missing (AGS4.1 Rule 14). Output won't be a fully \
             valid AGS4 file. Run `ags4_cli check` on the result if validity matters."
                .into(),
        );
    }
    if !codes_present.contains("PROJ") {
        warnings.push(
            "PROJ group missing (AGS4.1 Rule 13). Output won't be a fully \
             valid AGS4 file."
                .into(),
        );
    } else if proj_row_count != 1 {
        warnings.push(format!(
            "PROJ has {} rows; AGS4.1 Rule 13 requires exactly 1.",
            proj_row_count,
        ));
    }
    // UNIT + ABBR are commonly missing on partial or synthetic fixtures.
    // Only warn if they're referenced — i.e. some heading uses a unit
    // string or a PA/PT type that would normally be looked up.
    if !codes_present.contains("UNIT") {
        warnings.push(
            "UNIT group missing (AGS4.1 Rule 15). Any non-empty unit \
             strings won't have definitions to resolve against."
                .into(),
        );
    }
    if !codes_present.contains("ABBR") {
        warnings.push(
            "ABBR group missing (AGS4.1 Rule 16). PA / PT abbreviation \
             values won't have definitions to resolve against."
                .into(),
        );
    }

    let groups_emitted = emit_groups.len();
    let rows_emitted: usize = emit_groups.iter().map(|g| g.rows.len()).sum();

    // Write everything in one go. The AGS4 emitter doesn't need streaming
    // — even the 23 MB / 150k-row fixture is comfortable in RAM.
    let file =
        fs::File::create(out_path).map_err(|e| CliError::Schema(format!("create dst: {}", e)))?;
    let mut writer = std::io::BufWriter::new(file);

    let refs: Vec<EmitGroup<'_>> = emit_groups
        .iter()
        .map(|g| EmitGroup {
            code: &g.code,
            headings: g.headings.iter().map(String::as_str).collect(),
            units: g.units.iter().map(String::as_str).collect(),
            types: g.types.iter().map(String::as_str).collect(),
            rows: g.rows.clone(),
        })
        .collect();
    write_ags4(&mut writer, &refs)?;

    use std::io::Write;
    writer
        .flush()
        .map_err(|e| CliError::Schema(format!("flush dst: {}", e)))?;

    Ok(ExportStats {
        groups_emitted,
        rows_emitted,
        warnings,
    })
}

/// `EmitGroup` owns `&str`s, so build an owned mirror first then borrow.
struct OwnedEmitGroup {
    code: String,
    headings: Vec<String>,
    units: Vec<String>,
    types: Vec<String>,
    rows: Vec<Vec<String>>,
}

/// Read all rows from `v_<code>` and format each value back to its AGS4
/// string form. The view exposes one column per heading in lowercase
/// (`py_name`); we look them up by `h.py_name()` in registry order so
/// the output's column order matches the registry — and therefore the
/// `HEADING` row we emit. Missing columns (e.g. `id` / `parent_id`)
/// aren't part of the heading list so they fall out naturally.
fn read_view_rows(conn: &Connection, g: &GroupDescriptor) -> Result<Vec<Vec<String>>, CliError> {
    let view = g.view();
    let mut stmt = conn
        .prepare(&format!("SELECT * FROM {}", view))
        .map_err(|e| CliError::Schema(format!("prepare {}: {}", view, e)))?;
    let mut rows_iter = stmt
        .query([])
        .map_err(|e| CliError::Schema(format!("query {}: {}", view, e)))?;

    // Resolve column index per heading once. duckdb-rs only exposes
    // column names from the rows iter after a query has been run, so
    // the lookup is built here.
    let col_idx: std::collections::HashMap<String, usize> = {
        let stmt_ref = rows_iter
            .as_ref()
            .ok_or_else(|| CliError::Schema("statement detached after query".into()))?;
        let count = stmt_ref.column_count();
        (0..count)
            .map(|i| {
                let name = stmt_ref
                    .column_name(i)
                    .map(|s| s.to_string())
                    .unwrap_or_else(|_| format!("col_{}", i));
                (name, i)
            })
            .collect()
    };

    // Build per-heading (idx, ags_type, unit) tuples in registry order
    // so we skip missing columns gracefully. The unit comes along so
    // DT values can be truncated to match the precision declared by
    // the UNIT row (Rule 8 — see truncate_dt_to_unit). Empty unit means
    // no truncation.
    let plan: Vec<(Option<usize>, String, String)> = g
        .headings
        .iter()
        .map(|h| {
            (
                col_idx.get(&h.py_name()).copied(),
                h.ags_type.clone(),
                h.unit.clone().unwrap_or_default(),
            )
        })
        .collect();

    let mut out: Vec<Vec<String>> = Vec::new();
    while let Some(row) = rows_iter
        .next()
        .map_err(|e| CliError::Schema(format!("read {}: {}", view, e)))?
    {
        let mut cells: Vec<String> = Vec::with_capacity(plan.len());
        for (idx, ags_type, unit) in &plan {
            let s = match idx {
                Some(i) => {
                    let v: duckdb::types::Value = row
                        .get(*i)
                        .map_err(|e| CliError::Schema(format!("read {}.{}: {}", view, i, e)))?;
                    let json_v = value_to_json(v);
                    let raw = ags4_str(&json_v, ags_type);
                    // For DT columns, snap the value to the precision
                    // the UNIT row will declare — otherwise the
                    // validator flags every row with a Rule 8 mismatch
                    // (data has seconds but unit only carries minutes).
                    if ags_type.trim().eq_ignore_ascii_case("DT") && !unit.is_empty() {
                        truncate_dt_to_unit(&raw, unit)
                    } else {
                        raw
                    }
                }
                None => String::new(),
            };
            cells.push(s);
        }
        out.push(cells);
    }
    Ok(out)
}
