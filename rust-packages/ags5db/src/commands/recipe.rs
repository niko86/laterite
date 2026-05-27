//! `ags5db recipe [name] [--group X]` — print a query template by name,
//! or list all when called without a name.
//!
//! No `<db>` arg — recipes are general patterns, not file-specific.
//! `--group` substitutes `<test_group>` / `<GROUP>` placeholders.

use crate::Ctx;
use crate::output::{OutputMode, Rows, render_record, render_rows};
use ags5db::error::CliError;
use ags5db::recipes::{Recipe, load_all, substitute_group};
use ags5db::suggest::suggest;
use clap::Args;
use serde::Serialize;
use serde_json::{Map, Value};

#[derive(Args, Debug)]
pub struct Cmd {
    /// Recipe slug, e.g. 'depth-band-join'. Omit to list all.
    pub name: Option<String>,

    /// Substitute <test_group>/<GROUP> placeholders in the printed body
    #[arg(long)]
    pub group: Option<String>,
}

#[derive(Serialize)]
struct RecipeListItem {
    name: String,
    shape: String,
}

#[derive(Serialize)]
struct RecipeDetail {
    name: String,
    shape: String,
    body: String,
}

pub fn run(args: Cmd, ctx: Ctx) -> anyhow::Result<()> {
    let recipes: Vec<Recipe> = load_all();

    if args.name.is_none() {
        // List mode — render as rows so table/csv outputs work cleanly.
        let columns = vec!["name".to_string(), "shape".to_string()];
        let mut records: Vec<Map<String, Value>> = Vec::with_capacity(recipes.len());
        for r in &recipes {
            let mut rec = Map::new();
            rec.insert("name".into(), Value::from(r.name.clone()));
            rec.insert("shape".into(), Value::from(r.shape.clone()));
            records.push(rec);
        }
        render_rows(&Rows { columns, records }, ctx.mode, None)?;
        return Ok(());
    }

    let target = args.name.unwrap();
    let lookup = target.to_lowercase();
    let recipe = match recipes.iter().find(|r| r.name == lookup) {
        Some(r) => r,
        None => {
            let names: Vec<String> = recipes.iter().map(|r| r.name.clone()).collect();
            let hints = suggest(&lookup, &names, 3, false);
            // Reuse exit-4: "unknown <thing>" is the same self-correction
            // signal an agent uses for unknown group codes.
            return Err(CliError::UnknownGroup {
                code: target,
                hints,
            }
            .into());
        }
    };

    let body = match &args.group {
        Some(g) => substitute_group(&recipe.body, g),
        None => recipe.body.clone(),
    };
    let detail = RecipeDetail {
        name: recipe.name.clone(),
        shape: recipe.shape.clone(),
        body,
    };

    if ctx.mode == OutputMode::Table {
        println!("name   {}", detail.name);
        println!("shape  {}", detail.shape);
        println!();
        println!("{}", detail.body);
        return Ok(());
    }
    render_record(&detail, ctx.mode)?;
    // RecipeListItem is unused in this path but defined for symmetry with
    // the Python; suppress the dead_code lint.
    let _ = std::marker::PhantomData::<RecipeListItem>;
    Ok(())
}
