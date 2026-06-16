//! `lat-db groups <db> [--nonempty]` — every group with row count + parent.
//!
//! Thin CLI shim over `laterite_ags5_db::introspect::list_groups` (lib-ified in
//! F2a-2e). The data work lives in the lib; this file owns clap args
//! + row rendering.

use crate::Ctx;
use crate::output::{Rows, render_rows};
use clap::Args;
use laterite_ags5_db::introspect;
use serde_json::{Map, Value};
use std::path::PathBuf;

#[derive(Args, Debug)]
pub struct Cmd {
    /// Path to .ags5db file
    pub db: PathBuf,

    /// Show only groups with rows > 0
    #[arg(long)]
    pub nonempty: bool,
}

pub fn run(args: Cmd, ctx: Ctx) -> anyhow::Result<()> {
    let groups = introspect::list_groups(&args.db, args.nonempty)?;

    let columns = vec![
        "code".to_string(),
        "rows".to_string(),
        "parent".to_string(),
        "contents".to_string(),
    ];
    let mut records: Vec<Map<String, Value>> = Vec::with_capacity(groups.len());
    for g in groups {
        let mut rec = Map::new();
        rec.insert("code".into(), Value::from(g.code));
        rec.insert("rows".into(), Value::from(g.rows));
        rec.insert("parent".into(), Value::from(g.parent));
        rec.insert("contents".into(), Value::from(g.contents));
        records.push(rec);
    }

    render_rows(&Rows { columns, records }, ctx.mode, None)?;
    Ok(())
}
