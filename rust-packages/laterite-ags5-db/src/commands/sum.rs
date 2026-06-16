//! `lat-db sum <db> <group> <field> [--where ...]` — SUM(field) on a group.
//!
//! Mirrors Python `_cmd_sum`. Field must be a numeric heading; returns a
//! scalar float (0.0 on an empty SUM). The data work lives in
//! `laterite_ags5_db::query::sum`; this is the clap + scalar-render shim.

use crate::Ctx;
use crate::output::render_scalar;
use clap::Args;
use serde_json::Value;
use std::path::PathBuf;

#[derive(Args, Debug)]
pub struct Cmd {
    /// Path to .ags5db file
    pub db: PathBuf,

    /// Group code (e.g. SAMP)
    pub group: String,

    /// Heading to sum (must be numeric)
    pub field: String,

    /// 'field<op>value' filter; repeatable (ANDed)
    #[arg(long)]
    pub r#where: Vec<String>,
}

pub fn run(args: Cmd, ctx: Ctx) -> anyhow::Result<()> {
    let total = laterite_ags5_db::query::sum(&args.db, &args.group, &args.field, &args.r#where)?;
    render_scalar(&Value::from(total), ctx.mode)?;
    Ok(())
}
