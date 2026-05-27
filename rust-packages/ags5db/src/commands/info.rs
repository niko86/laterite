//! `ags5db info <db>` — file-level summary.
//!
//! Thin CLI shim over `ags5db::introspect::info` (lib-ified in F2a-2e).
//! Mirrors the Python `_cmd_info` output: file metadata + per-group
//! row counts. The NDJSON shape is byte-for-byte stable so e2e parity
//! tests against the Python reference still pass.

use crate::Ctx;
use crate::output::{OutputMode, Rows, render_record, render_rows};
use ags5db::introspect;
use clap::Args;
use serde::Serialize;
use serde_json::{Map, Value};
use std::path::PathBuf;

#[derive(Args, Debug)]
pub struct Cmd {
    /// Path to .ags5db file
    pub db: PathBuf,
}

#[derive(Serialize)]
struct GroupRowOut {
    code: String,
    rows: i64,
    parent: Option<String>,
}

#[derive(Serialize)]
struct InfoPayload {
    file: String,
    size_mb: f64,
    format_version: Option<String>,
    library_version: Option<String>,
    n_groups: usize,
    n_nonempty: usize,
    groups: Vec<GroupRowOut>,
}

pub fn run(args: Cmd, ctx: Ctx) -> anyhow::Result<()> {
    let summary = introspect::info(&args.db)?;

    let n_groups = summary.n_groups();
    let n_nonempty = summary.n_nonempty();

    let groups_out: Vec<GroupRowOut> = summary
        .groups
        .into_iter()
        .map(|g| GroupRowOut {
            code: g.code,
            rows: g.rows,
            // The NDJSON shape uses `Option<String>` (None when no
            // parent); introspect returns "" for ergonomics, convert back.
            parent: if g.parent.is_empty() {
                None
            } else {
                Some(g.parent)
            },
        })
        .collect();

    let payload = InfoPayload {
        file: summary.file,
        size_mb: summary.size_mb,
        format_version: summary.format_version,
        library_version: summary.library_version,
        n_groups,
        n_nonempty,
        groups: groups_out,
    };

    if ctx.mode == OutputMode::Table {
        println!("file            {}", payload.file);
        println!("size            {} MB", payload.size_mb);
        println!(
            "format_version  {}",
            payload.format_version.as_deref().unwrap_or("(pre-6.5)"),
        );
        println!(
            "library_version {}",
            payload.library_version.as_deref().unwrap_or("(unset)"),
        );
        println!(
            "groups          {} ({} non-empty)",
            payload.n_groups, payload.n_nonempty,
        );
        println!();

        let columns = vec!["code".to_string(), "rows".to_string(), "parent".to_string()];
        let mut records: Vec<Map<String, Value>> = Vec::new();
        for g in &payload.groups {
            if g.rows == 0 {
                continue;
            }
            let mut rec = Map::new();
            rec.insert("code".into(), Value::from(g.code.clone()));
            rec.insert("rows".into(), Value::from(g.rows));
            rec.insert(
                "parent".into(),
                Value::from(g.parent.clone().unwrap_or_default()),
            );
            records.push(rec);
        }
        render_rows(&Rows { columns, records }, OutputMode::Table, None)?;
        return Ok(());
    }

    render_record(&payload, ctx.mode)?;
    Ok(())
}
