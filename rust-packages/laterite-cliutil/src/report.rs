//! Report dispatch — the gogcli/ags5db output contract, in crate form.
//!
//! Each subcommand builds a *report document* and hands it to [`emit`]:
//! it goes to **stdout** in the resolved [`OutputMode`] (`table` = the
//! styled human summary; `json`/`ndjson` = the serialized doc, `ndjson`
//! automatically when piped). The live progress spinner/bar stays on
//! **stderr** (the rest of this crate), and one-line location/
//! diagnostic hints go to stderr via [`note`]. `--compact` swaps in a
//! trimmed value (drops the heavy per-file arrays) for token-lean
//! machine consumption.
//!
//! `Ctx` is resolved once in `main` and threaded through every
//! `run()` — the same pattern `ags5db` uses, so command modules never
//! re-parse the CLI. Lifted verbatim from
//! (behaviour byte-identical — the `without_keys` test moved with it).

use std::io::{self, Write};

use serde::Serialize;
use serde_json::Value;

use crate::{OutputMode, colour_enabled, write_json_pretty, write_ndjson};

/// Cross-command context: output mode + the global side-effect flags.
/// `Copy` so it threads cheaply into rayon closures.
#[derive(Debug, Clone, Copy)]
pub struct Ctx {
    pub mode: OutputMode,
    pub quiet: bool,
    pub dry_run: bool,
    pub no_input: bool,
    pub compact: bool,
    pub no_color: bool,
}

impl Ctx {
    /// Colour is on only for `table`/`json` on a TTY without
    /// `--no-color`/`NO_COLOR`. `ndjson` is never coloured (it's the
    /// machine stream).
    pub fn colour(&self) -> bool {
        self.mode != OutputMode::Ndjson && colour_enabled(self.no_color)
    }
}

/// A subcommand's result document. JSON/NDJSON come free from
/// `Serialize`; `table` is the bespoke human rendering. `--compact`
/// reports override [`compact_value`](Report::compact_value) to drop
/// heavy arrays.
pub trait Report: Serialize {
    /// The `table`-mode stdout payload (styled summary, possibly with
    /// a per-item list unless `ctx.compact`). Writes to `w` (stdout).
    fn render_table(&self, w: &mut dyn Write, ctx: &Ctx) -> io::Result<()>;

    /// Full serialized document (json/ndjson, no `--compact`).
    fn full_value(&self) -> Value {
        serde_json::to_value(self).unwrap_or(Value::Null)
    }

    /// `--compact` projection. Default = the full document; reports
    /// with large per-file arrays override to drop them.
    fn compact_value(&self) -> Value {
        self.full_value()
    }
}

/// Render `r` to stdout in `ctx.mode`. Stdout is locked once here so a
/// rayon-parallel pass can't interleave into the final document.
pub fn emit<R: Report>(r: &R, ctx: &Ctx) -> io::Result<()> {
    let mut out = io::stdout().lock();
    match ctx.mode {
        OutputMode::Table => r.render_table(&mut out, ctx),
        OutputMode::Json => {
            let v = if ctx.compact {
                r.compact_value()
            } else {
                r.full_value()
            };
            write_json_pretty(&mut out, &v, ctx.colour())
        }
        OutputMode::Ndjson => {
            let v = if ctx.compact {
                r.compact_value()
            } else {
                r.full_value()
            };
            write_ndjson(&mut out, &v)
        }
    }
}

/// A `--dry-run` preview document, shared by every subcommand. It
/// mutates nothing; `action` is the stage that *would* have run,
/// `summary` is the human one-liner, and `detail` carries the
/// machine-friendly numbers (file count, bytes, size buckets, …).
#[derive(Debug, Serialize)]
pub struct Plan {
    /// Always `true` — lets a json/ndjson consumer branch on it.
    pub dry_run: bool,
    /// `"crawl"` | `"validate"` | `"parity"` | … .
    pub action: String,
    /// Human one-liner (the `table`-mode headline).
    pub summary: String,
    #[serde(flatten)]
    pub detail: serde_json::Map<String, Value>,
}

impl Plan {
    pub fn new(action: &str, summary: impl Into<String>) -> Self {
        Self {
            dry_run: true,
            action: action.to_string(),
            summary: summary.into(),
            detail: serde_json::Map::new(),
        }
    }
    /// Add a machine-readable detail field (chainable).
    pub fn with(mut self, key: &str, value: impl Into<Value>) -> Self {
        self.detail.insert(key.to_string(), value.into());
        self
    }
}

impl Report for Plan {
    fn render_table(&self, w: &mut dyn Write, _ctx: &Ctx) -> io::Result<()> {
        writeln!(w, "DRY RUN ({}) — {}", self.action, self.summary)?;
        for (k, v) in &self.detail {
            // Strings unquoted; everything else compact JSON.
            match v {
                Value::String(s) => writeln!(w, "  {k}: {s}")?,
                other => writeln!(w, "  {k}: {other}")?,
            }
        }
        writeln!(w, "  (nothing written — mutate-nothing preview)")
    }
}

/// A one-line hint to **stderr**: result locations (`manifest → …`),
/// skips, degradations. Kept off stdout so the result document stays
/// pipe-clean; not gated by `--quiet` (that's only the progress
/// animation) since a locator is useful even in scripts.
pub fn note(msg: impl AsRef<str>) {
    eprintln!("{}", msg.as_ref());
}

/// `serde_json::Value` with the named keys removed (top-level object
/// only) — the building block for `--compact` projections that drop a
/// heavy per-file array while keeping schema/summary/counts.
pub fn without_keys(mut v: Value, keys: &[&str]) -> Value {
    if let Some(obj) = v.as_object_mut() {
        for k in keys {
            obj.remove(*k);
        }
    }
    v
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn without_keys_drops_only_named_top_level_keys() {
        let v = json!({"schema": 1, "summary": {"a": 1}, "files": [1, 2, 3]});
        let c = without_keys(v, &["files"]);
        assert!(c.get("files").is_none());
        assert_eq!(c["schema"], 1);
        assert_eq!(c["summary"]["a"], 1);
    }
}
