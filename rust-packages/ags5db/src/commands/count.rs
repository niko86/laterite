//! `ags5db count <db> <group> [--where ...]` — COUNT(*) on a group's view.
//!
//! Mirrors Python `_cmd_count`. Returns a scalar integer. The data work
//! (predicate validation + parameterised SELECT) lives in
//! `ags5db::query::count`; this is the clap + scalar-render shim.

use crate::Ctx;
use crate::output::render_scalar;
use clap::Args;
use serde_json::Value;
use std::path::PathBuf;

#[derive(Args, Debug)]
pub struct Cmd {
    /// Path to .ags5db file
    pub db: PathBuf,

    /// Group code (e.g. LOCA)
    pub group: String,

    /// 'field<op>value' filter; repeatable (ANDed)
    #[arg(long)]
    pub r#where: Vec<String>,
}

pub fn run(args: Cmd, ctx: Ctx) -> anyhow::Result<()> {
    let n = ags5db::query::count(&args.db, &args.group, &args.r#where)?;
    render_scalar(&Value::from(n), ctx.mode)?;
    Ok(())
}
