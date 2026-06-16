//! Output mode dispatch.
//!
//! Five modes (table/json/ndjson/csv/tsv) matching the Python `_cli_output.py`
//! contract. Auto-default: `table` if stdout is a TTY, `ndjson` when piped.
//! That makes the same CLI agent-friendly when scripted and human-friendly
//! at the terminal — no flag-juggling.
//!
//! Three shapes:
//!   * `render_record`  — one composite document (e.g. `info`).
//!   * `render_rows`    — a list of uniform records (e.g. `groups`, `peek`).
//!   * `render_scalar`  — a single value (e.g. `count`, `sum`).

use std::io::{self, IsTerminal, Write};

use anyhow::Result;
use comfy_table::{Attribute, Cell, Color, ContentArrangement, Table, presets::UTF8_FULL};
use serde::Serialize;
use serde_json::Value;

#[derive(Copy, Clone, Debug, PartialEq, Eq, clap::ValueEnum)]
#[clap(rename_all = "kebab-case")]
pub enum OutputMode {
    Table,
    Json,
    Ndjson,
    Csv,
    Tsv,
}

impl OutputMode {
    /// `table` in a TTY, `ndjson` when piped. Mirrors `_cli_output.default_mode`.
    pub fn auto() -> Self {
        if io::stdout().is_terminal() {
            Self::Table
        } else {
            Self::Ndjson
        }
    }
}

// --- composite record -------------------------------------------------------

/// Render a single composite record (e.g. `info`'s output) to stdout.
///
/// `table` and `json` produce indented forms; `ndjson` is a single-line
/// JSON object terminated by `\n` so a downstream `jq` or per-line parser
/// can consume one record at a time even when commands are concatenated.
pub fn render_record<T: Serialize>(value: &T, mode: OutputMode) -> Result<()> {
    let mut out = io::stdout().lock();
    let v: Value = serde_json::to_value(value)?;
    let use_color = colour_enabled();
    match mode {
        OutputMode::Table | OutputMode::Json => {
            // For composite records the closest analogue to rich.Table's
            // key/value layout is indented JSON. Single-record table mode
            // could grow into a real key/value table later if needed.
            if use_color {
                write_json_coloured(&mut out, &v, true)?;
            } else {
                serde_json::to_writer_pretty(&mut out, &v)?;
            }
            out.write_all(b"\n")?;
        }
        OutputMode::Ndjson | OutputMode::Csv | OutputMode::Tsv => {
            // Composite records (one document) don't map cleanly to CSV/TSV.
            // Fall through to JSON; commands that legitimately produce row-
            // shaped output use `render_rows`, which honours CSV/TSV.
            if use_color {
                write_json_coloured(&mut out, &v, false)?;
            } else {
                serde_json::to_writer(&mut out, &v)?;
            }
            out.write_all(b"\n")?;
        }
    }
    Ok(())
}

// --- row-shaped output ------------------------------------------------------

/// `Rows` is a lib-level data type (its canonical home is `db`); it is
/// re-exported here so the CLI renderers + command modules keep
/// importing it as `crate::output::Rows`.
pub use laterite_ags5_db::db::Rows;

/// Optional per-column metadata used by `peek` to attach a second-line
/// canonical type / unit label under each header in TABLE mode.
pub type TypeLabels = std::collections::HashMap<String, String>;

pub fn render_rows(rows: &Rows, mode: OutputMode, type_labels: Option<&TypeLabels>) -> Result<()> {
    let mut out = io::stdout().lock();
    let use_color = colour_enabled();
    match mode {
        OutputMode::Table => render_rows_table(rows, type_labels, &mut out)?,
        OutputMode::Ndjson => {
            for rec in &rows.records {
                let v = Value::Object(rec.clone());
                if use_color {
                    write_json_coloured(&mut out, &v, false)?;
                } else {
                    serde_json::to_writer(&mut out, rec)?;
                }
                out.write_all(b"\n")?;
            }
        }
        OutputMode::Json => {
            let v = Value::Array(
                rows.records
                    .iter()
                    .map(|r| Value::Object(r.clone()))
                    .collect(),
            );
            if use_color {
                write_json_coloured(&mut out, &v, true)?;
            } else {
                serde_json::to_writer_pretty(&mut out, &v)?;
            }
            out.write_all(b"\n")?;
        }
        OutputMode::Csv => write_separated(rows, b',', &mut out)?,
        OutputMode::Tsv => write_separated(rows, b'\t', &mut out)?,
    }
    Ok(())
}

fn render_rows_table(
    rows: &Rows,
    type_labels: Option<&TypeLabels>,
    out: &mut impl Write,
) -> Result<()> {
    if rows.records.is_empty() {
        writeln!(out, "(no rows)")?;
        return Ok(());
    }
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic);

    let use_color = colour_enabled();

    // Build header: column name + (optional) canonical-type/unit second line.
    // Style: bold cyan, matching Python's `header_style="bold cyan"`.
    let headers: Vec<Cell> = rows
        .columns
        .iter()
        .map(|col| {
            let label = type_labels.and_then(|m| m.get(col));
            let text = match label {
                Some(l) if !l.is_empty() => format!("{}\n{}", col, l),
                _ => col.clone(),
            };
            let mut cell = Cell::new(text).add_attribute(Attribute::Bold);
            if use_color {
                cell = cell.fg(Color::Cyan);
            }
            cell
        })
        .collect();
    table.set_header(headers);

    for (i, rec) in rows.records.iter().enumerate() {
        let dim_this_row = use_color && i % 2 == 1; // alternate "" / dim — matches
        // Python's row_styles=["", "dim"]
        let cells: Vec<Cell> = rows
            .columns
            .iter()
            .map(|c| {
                let mut cell = Cell::new(format_cell(rec.get(c).unwrap_or(&Value::Null)));
                if dim_this_row {
                    cell = cell.add_attribute(Attribute::Dim);
                }
                cell
            })
            .collect();
        table.add_row(cells);
    }
    writeln!(out, "{}", table)?;
    Ok(())
}

/// Decide whether to emit ANSI colour codes. Off when stdout isn't a TTY
/// or `NO_COLOR` is set in the environment. Mirrors the convention every
/// Unix tool uses; matches the gate the rest of the codebase uses for
/// `--no-color`.
fn colour_enabled() -> bool {
    if std::env::var_os("NO_COLOR").is_some() {
        return false;
    }
    io::stdout().is_terminal()
}

// --- live progress ---------------------------------------------------------

/// Animated single-line spinner for long-running commands. Backed by
/// `indicatif::ProgressBar::new_spinner`; ticks every 100 ms.
///
/// When stderr isn't a TTY (piped, redirected, CI) or `--quiet` is set,
/// returns a no-op shim that prints one-shot `progress()` lines instead.
/// Drop the handle to stop the spinner (clears the line on exit).
pub struct Spinner {
    inner: SpinnerKind,
}

enum SpinnerKind {
    Live(indicatif::ProgressBar),
    /// Non-TTY path: write one line per `set_message`, suppress final-line
    /// clear. The terminal user gets a "X..." log; the spinner user sees
    /// the same thing as before, no flicker.
    Static {
        quiet: bool,
    },
}

impl Spinner {
    /// Start a spinner with `initial_msg` on stderr. `quiet` suppresses
    /// all output (used for `--quiet` mode + the test suite).
    pub fn start(initial_msg: &str, quiet: bool) -> Self {
        let stderr_tty = io::stderr().is_terminal();
        if quiet {
            return Self {
                inner: SpinnerKind::Static { quiet: true },
            };
        }
        if !stderr_tty {
            // Non-TTY: emit a single line up front; subsequent
            // set_message calls also emit lines. No animation.
            eprintln!("{initial_msg}");
            return Self {
                inner: SpinnerKind::Static { quiet: false },
            };
        }
        let pb = indicatif::ProgressBar::new_spinner();
        pb.set_message(initial_msg.to_string());
        pb.enable_steady_tick(std::time::Duration::from_millis(100));
        Self {
            inner: SpinnerKind::Live(pb),
        }
    }

    /// Update the displayed message. On TTY, the spinner refreshes;
    /// otherwise a new line is printed (skipped when quiet).
    pub fn set_message(&self, msg: &str) {
        match &self.inner {
            SpinnerKind::Live(pb) => pb.set_message(msg.to_string()),
            SpinnerKind::Static { quiet: false } => eprintln!("{msg}"),
            SpinnerKind::Static { quiet: true } => {}
        }
    }
}

impl Drop for Spinner {
    fn drop(&mut self) {
        if let SpinnerKind::Live(pb) = &self.inner {
            pb.finish_and_clear();
        }
    }
}

// ANSI colour codes for the JSON syntax tokens. Picked to match the
// rich python library's `print_json` defaults: keys cyan-bold, strings
// green, numbers yellow/cyan, booleans + null magenta, structural chars
// dim. We render directly rather than via `nu_ansi_term` because the
// total surface is small enough that hand-rolling is clearer.
const C_RESET: &str = "\x1b[0m";
const C_KEY: &str = "\x1b[1;36m"; // bold cyan
const C_STRING: &str = "\x1b[32m"; // green
const C_NUMBER: &str = "\x1b[33m"; // yellow
const C_LITERAL: &str = "\x1b[35m"; // magenta — bools + null
const C_STRUCT: &str = "\x1b[2m"; // dim — { } [ ] , :

/// Write a JSON `Value` to `out` with ANSI colour codes per token.
/// `pretty=true` formats with 2-space indents like `to_writer_pretty`;
/// `false` emits a single compact line (matching `to_writer`).
fn write_json_coloured<W: Write>(out: &mut W, v: &Value, pretty: bool) -> Result<()> {
    if pretty {
        write_coloured_pretty(out, v, 0)?;
    } else {
        write_coloured_compact(out, v)?;
    }
    Ok(())
}

fn write_coloured_compact<W: Write>(out: &mut W, v: &Value) -> Result<()> {
    match v {
        Value::Null => write!(out, "{C_LITERAL}null{C_RESET}")?,
        Value::Bool(b) => write!(out, "{C_LITERAL}{b}{C_RESET}")?,
        Value::Number(n) => write!(out, "{C_NUMBER}{n}{C_RESET}")?,
        Value::String(s) => {
            let escaped = serde_json::to_string(s)?;
            write!(out, "{C_STRING}{escaped}{C_RESET}")?;
        }
        Value::Array(arr) => {
            write!(out, "{C_STRUCT}[{C_RESET}")?;
            for (i, item) in arr.iter().enumerate() {
                if i > 0 {
                    write!(out, "{C_STRUCT},{C_RESET}")?;
                }
                write_coloured_compact(out, item)?;
            }
            write!(out, "{C_STRUCT}]{C_RESET}")?;
        }
        Value::Object(map) => {
            write!(out, "{C_STRUCT}{{{C_RESET}")?;
            for (i, (k, v)) in map.iter().enumerate() {
                if i > 0 {
                    write!(out, "{C_STRUCT},{C_RESET}")?;
                }
                let k_escaped = serde_json::to_string(k)?;
                write!(out, "{C_KEY}{k_escaped}{C_RESET}{C_STRUCT}:{C_RESET}")?;
                write_coloured_compact(out, v)?;
            }
            write!(out, "{C_STRUCT}}}{C_RESET}")?;
        }
    }
    Ok(())
}

fn write_coloured_pretty<W: Write>(out: &mut W, v: &Value, depth: usize) -> Result<()> {
    let pad = |d: usize| "  ".repeat(d);
    match v {
        Value::Array(arr) if arr.is_empty() => write!(out, "{C_STRUCT}[]{C_RESET}")?,
        Value::Object(map) if map.is_empty() => write!(out, "{C_STRUCT}{{}}{C_RESET}")?,
        Value::Array(arr) => {
            writeln!(out, "{C_STRUCT}[{C_RESET}")?;
            for (i, item) in arr.iter().enumerate() {
                write!(out, "{}", pad(depth + 1))?;
                write_coloured_pretty(out, item, depth + 1)?;
                if i + 1 < arr.len() {
                    writeln!(out, "{C_STRUCT},{C_RESET}")?;
                } else {
                    writeln!(out)?;
                }
            }
            write!(out, "{}{C_STRUCT}]{C_RESET}", pad(depth))?;
        }
        Value::Object(map) => {
            writeln!(out, "{C_STRUCT}{{{C_RESET}")?;
            for (i, (k, v)) in map.iter().enumerate() {
                let k_escaped = serde_json::to_string(k)?;
                write!(
                    out,
                    "{}{C_KEY}{k_escaped}{C_RESET}{C_STRUCT}:{C_RESET} ",
                    pad(depth + 1),
                )?;
                write_coloured_pretty(out, v, depth + 1)?;
                if i + 1 < map.len() {
                    writeln!(out, "{C_STRUCT},{C_RESET}")?;
                } else {
                    writeln!(out)?;
                }
            }
            write!(out, "{}{C_STRUCT}}}{C_RESET}", pad(depth))?;
        }
        scalar => write_coloured_compact(out, scalar)?,
    }
    Ok(())
}

fn write_separated(rows: &Rows, delim: u8, out: &mut impl Write) -> Result<()> {
    let mut wtr = csv::WriterBuilder::new().delimiter(delim).from_writer(out);
    wtr.write_record(&rows.columns)?;
    for rec in &rows.records {
        let row: Vec<String> = rows
            .columns
            .iter()
            .map(|c| format_cell(rec.get(c).unwrap_or(&Value::Null)))
            .collect();
        wtr.write_record(&row)?;
    }
    wtr.flush()?;
    Ok(())
}

// --- scalar -----------------------------------------------------------------

/// Render a single value (e.g. `count`'s integer, `sum`'s float) to stdout.
///
/// Matches the Python `_render_scalar`: TABLE prints the bare value,
/// JSON/NDJSON emit `json.dumps(value)`, CSV/TSV emit the raw value with
/// no header (a single-cell record).
pub fn render_scalar(value: &Value, mode: OutputMode) -> Result<()> {
    let mut out = io::stdout().lock();
    match mode {
        OutputMode::Table => writeln!(out, "{}", format_cell(value))?,
        OutputMode::Json | OutputMode::Ndjson => {
            serde_json::to_writer(&mut out, value)?;
            out.write_all(b"\n")?;
        }
        OutputMode::Csv | OutputMode::Tsv => writeln!(out, "{}", format_cell(value))?,
    }
    Ok(())
}

// --- cell formatting --------------------------------------------------------

/// Format a JSON value as a display string for table/CSV cells.
/// Mirrors `_fmt_value` in the Python: NULL -> empty, floats via `{g}` (Polars
/// default), strings as-is, everything else via JSON debug.
pub fn format_cell(v: &Value) -> String {
    match v {
        Value::Null => String::new(),
        Value::Bool(b) => b.to_string(),
        Value::String(s) => s.clone(),
        Value::Number(n) => {
            if let Some(f) = n.as_f64() {
                if f.fract() == 0.0 && f.is_finite() && n.is_i64() {
                    // Integer-valued number: print without trailing zeros.
                    n.to_string()
                } else {
                    // Polars's default float repr uses {:g}; std doesn't expose
                    // exactly that, but the trim-trailing-zeros equivalent is
                    // close enough for human reading. Parity tests compare
                    // NDJSON (which keeps native floats), not table output.
                    let s = format!("{}", f);
                    s
                }
            } else {
                n.to_string()
            }
        }
        _ => v.to_string(),
    }
}
