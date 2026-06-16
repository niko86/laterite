//! `lat-db agent-context <db> [--top N]` — one-call warm-up.
//!
//! Mirrors Python `_cmd_agent_context`. Returns a composite document with
//! file metadata + top-N populated groups + a sample row per group +
//! recipe suggestions. Designed for "agent just landed on an unfamiliar
//! .ags5db" so it can skip the orient/narrow phase.

use std::collections::HashSet;
use std::path::PathBuf;

use clap::Args;
use serde::Serialize;
use serde_json::{Map, Value};

use crate::Ctx;
use crate::output::render_record;
use crate::progress;
use laterite_ags5_db::conn::open_readonly;
use laterite_ags5_db::db::{display_native, fetch_rows, headings_for};
use laterite_ags5_db::error::CliError;

#[derive(Args, Debug)]
pub struct Cmd {
    /// Path to .ags5db file
    pub db: PathBuf,

    /// Number of populated groups to include (default 10)
    #[arg(long, default_value_t = 10)]
    pub top: usize,
}

#[derive(Serialize)]
struct AgentContext {
    file: String,
    size_mb: f64,
    format_version: Option<String>,
    library_version: Option<String>,
    written_at: Option<String>,
    n_groups_populated: usize,
    n_groups_shown: usize,
    populated_groups: Vec<PopulatedGroup>,
    recipe_suggestions: Vec<RecipeSuggestion>,
}

#[derive(Serialize)]
struct PopulatedGroup {
    code: String,
    rows: i64,
    parent: Option<String>,
    contents: Option<String>,
    key_fields: Vec<String>,
    useful_fields: Vec<String>,
    sample_row: Map<String, Value>,
}

#[derive(Serialize)]
struct RecipeSuggestion {
    recipe: String,
    when: String,
}

pub fn run(args: Cmd, ctx: Ctx) -> anyhow::Result<()> {
    if !args.db.exists() {
        return Err(CliError::FileNotFound(args.db.display().to_string()).into());
    }
    let conn = open_readonly(&args.db)?;
    let size_bytes = std::fs::metadata(&args.db)
        .map_err(|e| CliError::Schema(format!("metadata: {}", e)))?
        .len();
    let size_mb = (size_bytes as f64 / 1_000_000.0 * 100.0).round() / 100.0;

    let (format_version, library_version, written_at) = conn
        .query_row(
            // CAST timestamp to VARCHAR — DuckDB's Rust binding can't pull a
            // TIMESTAMP straight into Option<String>, and the previous
            // `unwrap_or((None,None,None))` was silently swallowing the
            // type-mismatch and returning nulls for format/library version too.
            "SELECT format_version, library_version,
                    CAST(written_at AS VARCHAR)
               FROM _spec_meta",
            [],
            |row| {
                Ok((
                    row.get::<_, Option<String>>(0)?,
                    row.get::<_, Option<String>>(1)?,
                    row.get::<_, Option<String>>(2)?,
                ))
            },
        )
        .unwrap_or((None, None, None));

    // Load every populated group (rows > 0), ordered by row count desc.
    // Re-use the same SQL shape as `info` so the row count source agrees.
    let mut stmt = conn
        .prepare(
            "SELECT g.code,
                    COALESCE(t.estimated_size, 0) AS rows,
                    g.parent,
                    g.contents
               FROM _spec_groups g
               LEFT JOIN duckdb_tables() t
                 ON t.table_name = 'g_' || lower(g.code)
              WHERE COALESCE(t.estimated_size, 0) > 0
              ORDER BY rows DESC, g.code",
        )
        .map_err(|e| CliError::Schema(e.to_string()))?;

    struct Summary {
        code: String,
        rows: i64,
        parent: Option<String>,
        contents: Option<String>,
    }
    let summaries: Vec<Summary> = stmt
        .query_map([], |row| {
            Ok(Summary {
                code: row.get(0)?,
                rows: row.get(1)?,
                parent: row.get(2)?,
                contents: row.get(3)?,
            })
        })
        .map_err(|e| CliError::Schema(e.to_string()))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| CliError::Schema(e.to_string()))?;

    let n_groups_populated = summaries.len();
    let top: Vec<&Summary> = summaries.iter().take(args.top).collect();

    progress(
        &format!("warming up: probing {} populated group(s)...", top.len()),
        ctx.quiet,
    );

    let mut populated: Vec<PopulatedGroup> = Vec::with_capacity(top.len());
    for s in &top {
        let headings = headings_for(&conn, &s.code)?;
        let key_fields: Vec<String> = headings
            .iter()
            .filter(|h| h.status.eq_ignore_ascii_case("KEY"))
            .map(|h| h.name.to_lowercase())
            .collect();

        // Probe one row to populate useful_fields + sample_row. Drop columns
        // that are NULL in the probe so the agent sees what's actually
        // populated (not the registry superset). The Python path uses
        // `query_readings(..., fields=g.headings.py_names)` which yields
        // only the AGS heading columns — `id`/`parent_id`/`_content_hash`
        // are surrogate-key bookkeeping that the writer adds and aren't
        // useful to a downstream consumer, so exclude them here to match.
        let view = format!("v_{}", s.code.to_lowercase());
        let probe = fetch_rows(&conn, &format!("SELECT * FROM {} LIMIT 1", view), &[])?;
        let surrogate: &[&str] = &["id", "parent_id", "_content_hash"];
        let is_surrogate = |c: &str| surrogate.contains(&c);

        let mut sample_row: Map<String, Value> = Map::new();
        let mut useful_fields: Vec<String> = Vec::new();
        if let Some(row) = probe.records.first() {
            for col in &probe.columns {
                if is_surrogate(col) {
                    continue;
                }
                let v = row.get(col).cloned().unwrap_or(Value::Null);
                if !v.is_null() {
                    sample_row.insert(col.clone(), v);
                }
            }
            for col in &probe.columns {
                if useful_fields.len() >= 5 {
                    break;
                }
                if is_surrogate(col) {
                    continue;
                }
                let v = row.get(col).cloned().unwrap_or(Value::Null);
                if !v.is_null() && !key_fields.contains(col) {
                    useful_fields.push(col.clone());
                }
            }
        }

        populated.push(PopulatedGroup {
            code: s.code.clone(),
            rows: s.rows,
            parent: s.parent.clone(),
            contents: s.contents.clone(),
            key_fields,
            useful_fields,
            sample_row,
        });
    }

    let all_codes: HashSet<String> = summaries.iter().map(|s| s.code.clone()).collect();
    let recipe_suggestions = suggest_recipes(&all_codes);

    let composite = AgentContext {
        file: display_native(&args.db),
        size_mb,
        format_version,
        library_version,
        written_at,
        n_groups_populated,
        n_groups_shown: populated.len(),
        populated_groups: populated,
        recipe_suggestions,
    };

    if ctx.mode == crate::output::OutputMode::Table {
        render_table(&composite)?;
        return Ok(());
    }
    render_record(&composite, ctx.mode)?;
    Ok(())
}

fn render_table(c: &AgentContext) -> anyhow::Result<()> {
    use crate::output::{OutputMode, Rows, render_rows};
    use serde_json::{Map, Value};

    println!("file                {}", c.file);
    println!("size                {} MB", c.size_mb);
    println!(
        "format_version      {}",
        c.format_version.as_deref().unwrap_or("(unset)"),
    );
    println!(
        "library_version     {}",
        c.library_version.as_deref().unwrap_or("(unset)"),
    );
    println!(
        "written_at          {}",
        c.written_at.as_deref().unwrap_or("(unset)"),
    );
    println!(
        "populated groups    {} (showing {})",
        c.n_groups_populated, c.n_groups_shown,
    );
    println!();

    // Populated groups table: code, rows, parent, contents.
    let columns = vec![
        "code".into(),
        "rows".into(),
        "parent".into(),
        "contents".into(),
    ];
    let mut records: Vec<Map<String, Value>> = Vec::new();
    for g in &c.populated_groups {
        let mut rec = Map::new();
        rec.insert("code".into(), Value::from(g.code.clone()));
        rec.insert("rows".into(), Value::from(g.rows));
        rec.insert(
            "parent".into(),
            Value::from(g.parent.clone().unwrap_or_default()),
        );
        rec.insert(
            "contents".into(),
            Value::from(g.contents.clone().unwrap_or_default()),
        );
        records.push(rec);
    }
    render_rows(&Rows { columns, records }, OutputMode::Table, None)?;

    if !c.recipe_suggestions.is_empty() {
        println!();
        println!("recipe suggestions:");
        for s in &c.recipe_suggestions {
            println!("  {:<24}  {}", s.recipe, s.when);
        }
    }
    Ok(())
}

fn suggest_recipes(codes: &HashSet<String>) -> Vec<RecipeSuggestion> {
    let mut out: Vec<RecipeSuggestion> = Vec::new();
    let has = |c: &str| codes.contains(c);
    if has("GEOL") && (has("LLPL") || has("TREG") || has("GCHM") || has("LNMC")) {
        out.push(RecipeSuggestion {
            recipe: "depth-band-join".into(),
            when: "to relate per-depth test results to geology layers".into(),
        });
    }
    if has("SCPT") || has("MOND") {
        out.push(RecipeSuggestion {
            recipe: "depth-bin-aggregate".into(),
            when: "to summarise high-volume readings by depth bins".into(),
        });
        out.push(RecipeSuggestion {
            recipe: "high-volume-downsample".into(),
            when: "to subset SCPT/MOND rows for plotting".into(),
        });
    }
    if has("LOCA") {
        out.push(RecipeSuggestion {
            recipe: "cross-borehole-compare".into(),
            when: "for site-level stats grouped by loca_id / loca_type".into(),
        });
    }
    if has("TREG") && has("TREL") {
        out.push(RecipeSuggestion {
            recipe: "parent-chain-drill".into(),
            when: "to pull triaxial readings filtered by test stage type".into(),
        });
    }
    if has("ABBR") {
        out.push(RecipeSuggestion {
            recipe: "abbr-lookup".into(),
            when: "to decode AGS abbreviation codes (e.g. GEOL_GEOL='ALV')".into(),
        });
    }
    out
}
