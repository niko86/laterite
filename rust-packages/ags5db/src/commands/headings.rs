//! `ags5db headings <db> <group>` — schema dump for one group.
//!
//! Mirrors Python `_cmd_headings`. Reads `_spec_headings` for the named
//! group; one record per heading. Unknown group → exit 4 with fuzzy hint.

use crate::Ctx;
use crate::output::{Rows, render_rows};
use ags5db::conn::open_readonly;
use ags5db::db::{headings_for, resolve_db_and_group};
use clap::Args;
use serde_json::{Map, Value};
use std::path::PathBuf;

#[derive(Args, Debug)]
pub struct Cmd {
    /// Path to .ags5db file
    pub db: PathBuf,

    /// Group code (e.g. LOCA)
    pub group: String,
}

pub fn run(args: Cmd, ctx: Ctx) -> anyhow::Result<()> {
    let (_db, code) = resolve_db_and_group(&args.db, &args.group)?;
    let conn = open_readonly(&args.db)?;
    let headings = headings_for(&conn, &code)?;

    let columns = vec![
        "name".to_string(),
        "status".to_string(),
        "ags_type".to_string(),
        "canonical_type".to_string(),
        "unit".to_string(),
        "hint".to_string(),
    ];
    let mut records: Vec<Map<String, Value>> = Vec::with_capacity(headings.len());
    for h in headings {
        let mut rec = Map::new();
        rec.insert("name".into(), Value::from(h.name));
        rec.insert("status".into(), Value::from(h.status));
        rec.insert("ags_type".into(), Value::from(h.ags_type));
        rec.insert("canonical_type".into(), Value::from(h.canonical_type));
        rec.insert("unit".into(), Value::from(h.unit));
        rec.insert("hint".into(), Value::from(h.hint));
        records.push(rec);
    }
    render_rows(&Rows { columns, records }, ctx.mode, None)?;
    Ok(())
}
