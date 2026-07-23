//! `manifest.json` — the crawl→validate hand-off.

use std::io::{self, Write};
use std::path::Path;

use anyhow::{Context, Result};
use laterite_cliutil::styled_table;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::output::{Ctx, Report, without_keys};

pub const SCHEMA: u32 = 1;

#[derive(Debug, Serialize, Deserialize)]
pub struct CrawlManifest {
    pub schema: u32,
    pub created: String, // RFC3339
    pub root: String,    // the crawled root (as given)
    pub selection: String,
    pub walk_skipped: u64, // unreadable dirs/files skipped during the walk
    pub files: Vec<ManifestEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestEntry {
    pub source: String, // original UNC/share path
    pub dest: String,   // relative to corpus-dir, e.g. harvested/ab..__x.ags
    pub size: u64,
    pub sha256: String, // content hash (whole file)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub copy_error: Option<String>,
}

impl CrawlManifest {
    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(p) = path.parent() {
            std::fs::create_dir_all(p).with_context(|| format!("create {}", p.display()))?;
        }
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json).with_context(|| format!("write {}", path.display()))?;
        Ok(())
    }

    pub fn load(path: &Path) -> Result<Self> {
        let s = std::fs::read_to_string(path)
            .with_context(|| format!("read manifest {}", path.display()))?;
        let m: CrawlManifest = serde_json::from_str(&s)
            .with_context(|| format!("parse manifest {}", path.display()))?;
        Ok(m)
    }
}

impl Report for CrawlManifest {
    fn render_table(&self, w: &mut dyn Write, ctx: &Ctx) -> io::Result<()> {
        let copy_failures = self.files.iter().filter(|e| e.copy_error.is_some()).count();
        writeln!(
            w,
            "{}",
            styled_table(
                &["Crawl", "Value"],
                vec![
                    vec!["root".into(), self.root.clone()],
                    vec!["selection".into(), self.selection.clone()],
                    vec!["files copied".into(), self.files.len().to_string()],
                    vec!["copy errors".into(), copy_failures.to_string()],
                    vec!["walk skipped".into(), self.walk_skipped.to_string()],
                ],
                ctx.colour(),
            )
        )
    }

    /// `--compact`: drop the per-file `files` array (it's the whole
    /// manifest body); keep `schema/root/selection/walk_skipped`.
    fn compact_value(&self) -> Value {
        without_keys(self.full_value(), &["files"])
    }
}
