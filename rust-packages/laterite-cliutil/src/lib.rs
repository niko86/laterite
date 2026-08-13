//! Shared CLI presentation for the workspace's Rust binaries.
//!
//! This is the de-duplication of code that was previously copied
//! verbatim between `lat` and `ags4-corpus-qa` (and is mirrored
//! again in another workspace CLI, which stays on its own copy for now —
//! it's binary-only with no lib target). The behaviour here is
//! deliberately byte-identical to those copies so all the CLIs in the
//! toolkit "look and feel like one tool": the same `indicatif`
//! spinner (100 ms steady tick, live on a TTY / one static line piped
//! / silent under `--quiet`), the same `comfy-table` `UTF8_FULL` grid
//! (bold-cyan header, alternating dim rows), the same
//! `NO_COLOR`/TTY colour gate, and the same rich-style coloured-JSON
//! token palette.
//!
//! The gogcli → discrawl → cli-printing-press lineage these CLIs
//! follow: results to **stdout** in the resolved mode, progress and
//! diagnostics to **stderr**, and **ndjson automatically when piped**
//! ([`OutputMode::auto`]) so a scripted/agent caller needs no flag.

use std::io::{self, IsTerminal, Write};
use std::path::{Path, PathBuf};
use std::time::Duration;

use comfy_table::{Attribute, Cell, Color, ContentArrangement, Table, presets::UTF8_FULL};
use serde_json::Value;

/// The `Ctx` + `Report` + `emit` + `Plan` report-document scaffold
/// (the CLI output contract), lifted here from
/// `ags4-corpus-qa/src/output.rs` so every CLI shares one copy.
pub mod report;

// --- output mode ------------------------------------------------------

/// How a command's result document is rendered to **stdout**.
///
/// `table` is the human form (the styled `comfy-table` summary); `json`
/// is indented (pretty, coloured on a TTY); `ndjson` is the same
/// document on a single line terminated by `\n` — stream/`jq` friendly
/// and the auto-default when stdout is piped.
///
/// Deliberately only three modes (no csv/tsv): these CLIs emit nested
/// *report documents*, not flat row tables, so csv/tsv would be a
/// dishonest column-flattening. Honest scoping over fake parity.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
#[cfg_attr(feature = "clap", clap(rename_all = "kebab-case"))]
pub enum OutputMode {
    Table,
    Json,
    Ndjson,
}

impl OutputMode {
    /// `table` in a TTY, `ndjson` when piped — agent-friendly with no
    /// flag (the auto-JSON convention shared with cli-printing-press).
    #[must_use]
    pub fn auto() -> Self {
        if io::stdout().is_terminal() {
            Self::Table
        } else {
            Self::Ndjson
        }
    }
}

// --- colour gate ------------------------------------------------------

/// Off when `no_color` is set, the `NO_COLOR` env var is present, or
/// stdout isn't a TTY — the convention every Unix tool uses. `lat`
/// and its sibling CLIs use the env+TTY form; the explicit `no_color`
/// argument lets a CLI with a `--no-color` flag fold it in.
#[must_use]
pub fn colour_enabled(no_color: bool) -> bool {
    !no_color && std::env::var_os("NO_COLOR").is_none() && io::stdout().is_terminal()
}

// --- styled table -----------------------------------------------------

/// A `UTF8_FULL` grid with the toolkit's house style: bold-cyan
/// header, alternating dim data rows, dynamic column arrangement.
/// `use_color` off → no ANSI (for files / piped / `NO_COLOR`).
/// `headers` then `rows` of equal width.
#[must_use]
pub fn styled_table(headers: &[&str], rows: Vec<Vec<String>>, use_color: bool) -> Table {
    let mut t = Table::new();
    t.load_style(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic);
    t.set_header(
        headers
            .iter()
            .map(|h| {
                let mut c = Cell::new(*h).add_attribute(Attribute::Bold);
                if use_color {
                    c = c.fg(Color::Cyan);
                }
                c
            })
            .collect::<Vec<_>>(),
    );
    for (i, r) in rows.into_iter().enumerate() {
        let dim = use_color && i % 2 == 1;
        t.add_row(
            r.into_iter()
                .map(|v| {
                    let mut c = Cell::new(v);
                    if dim {
                        c = c.add_attribute(Attribute::Dim);
                    }
                    c
                })
                .collect::<Vec<_>>(),
        );
    }
    t
}

// --- progress spinner / bar -------------------------------------------

/// Animated single-line spinner on **stderr**. Live `indicatif` bar on
/// a TTY (100 ms steady tick); one static line when piped/CI; a silent
/// no-op under `--quiet`. Drop to clear the line.
pub struct Spinner {
    inner: Kind,
}

enum Kind {
    Live(indicatif::ProgressBar),
    Static,
    Quiet,
}

impl Spinner {
    #[must_use]
    pub fn start(msg: &str, quiet: bool) -> Self {
        if quiet {
            return Self { inner: Kind::Quiet };
        }
        if !io::stderr().is_terminal() {
            eprintln!("{msg}");
            return Self {
                inner: Kind::Static,
            };
        }
        let pb = indicatif::ProgressBar::new_spinner();
        pb.set_message(msg.to_string());
        pb.enable_steady_tick(Duration::from_millis(100));
        Self {
            inner: Kind::Live(pb),
        }
    }

    /// Update the live message (no-op when static/quiet).
    pub fn set(&self, msg: &str) {
        if let Kind::Live(pb) = &self.inner {
            pb.set_message(msg.to_string());
        }
    }
}

impl Drop for Spinner {
    fn drop(&mut self) {
        if let Kind::Live(pb) = &self.inner {
            pb.finish_and_clear();
        }
    }
}

/// A determinate bar for batch passes (live on a stderr TTY, hidden
/// otherwise — i.e. piped/CI/`--quiet`). `len` items.
#[must_use]
pub fn progress_bar(len: u64, quiet: bool) -> indicatif::ProgressBar {
    if quiet || !io::stderr().is_terminal() {
        return indicatif::ProgressBar::hidden();
    }
    let pb = indicatif::ProgressBar::new(len);
    pb.set_style(
        indicatif::ProgressStyle::with_template("{spinner} {pos}/{len} [{bar:30}] {msg}")
            .unwrap_or_else(|_| indicatif::ProgressStyle::default_bar())
            .progress_chars("=> "),
    );
    pb
}

// --- multi-line progress (one line per worker) ------------------------

/// A header line + N fixed worker lines on **stderr** (e.g. a
/// parallel directory walk: each worker shows the nested folder it's
/// descending). Same Live/Static/Quiet + TTY/`--quiet` discipline as
/// [`Spinner`] — scaled to many lines via `indicatif::MultiProgress`.
/// Drop clears the whole area so the following result table starts
/// on a clean row.
pub struct MultiLine {
    inner: MultiKind,
}

enum MultiKind {
    /// Real TTY: a steady-ticking header + N child spinners.
    Live {
        // `mp` owns the draw target; kept so `suspend`/`clear` work.
        mp: indicatif::MultiProgress,
        header: indicatif::ProgressBar,
        lines: Vec<indicatif::ProgressBar>,
    },
    /// Piped/CI: the header was printed once, no ANSI thereafter.
    Static,
    /// `--quiet`: total no-op.
    Quiet,
}

impl MultiLine {
    /// `lines` worker lines under `header`. Gating mirrors
    /// [`Spinner::start`] exactly.
    #[must_use]
    pub fn start(header: &str, lines: usize, quiet: bool) -> Self {
        if quiet {
            return Self {
                inner: MultiKind::Quiet,
            };
        }
        // Non-TTY (or degenerate 0 lines): one static line, no ANSI —
        // piped/agent output must stay clean.
        if lines == 0 || !io::stderr().is_terminal() {
            eprintln!("{header}");
            return Self {
                inner: MultiKind::Static,
            };
        }
        let mp =
            indicatif::MultiProgress::with_draw_target(indicatif::ProgressDrawTarget::stderr());
        // Header first → it renders on top of the worker lines.
        let h = mp.add(indicatif::ProgressBar::new_spinner());
        h.set_message(header.to_string());
        // ONLY the header steady-ticks: keeps the area animating even
        // when a worker blocks on a slow stat, without N timers
        // contending with the rayon update feed.
        h.enable_steady_tick(Duration::from_millis(100));
        let lines = (0..lines)
            .map(|_| {
                let pb = mp.add(indicatif::ProgressBar::new_spinner());
                pb.set_message("idle");
                pb
            })
            .collect();
        Self {
            inner: MultiKind::Live {
                mp,
                header: h,
                lines,
            },
        }
    }

    /// Set worker line `idx`'s message (no-op when static/quiet, or
    /// `idx` out of range — display must never panic the caller).
    pub fn set_line(&self, idx: usize, msg: &str) {
        if let MultiKind::Live { lines, .. } = &self.inner {
            if let Some(pb) = lines.get(idx) {
                pb.set_message(msg.to_string());
            }
        }
    }

    /// Update the header (no-op when static/quiet — Static already
    /// printed its one line; don't spam piped logs).
    pub fn set_header(&self, msg: &str) {
        if let MultiKind::Live { header, .. } = &self.inner {
            header.set_message(msg.to_string());
        }
    }

    /// Run `f` with the live area cleared (then redrawn) so a raw
    /// `eprintln!` (e.g. a `skip:` diagnostic) can't smear the
    /// multi-line region. Pass-through when static/quiet.
    pub fn suspend<R>(&self, f: impl FnOnce() -> R) -> R {
        match &self.inner {
            MultiKind::Live { mp, .. } => mp.suspend(f),
            _ => f(),
        }
    }
}

impl Drop for MultiLine {
    fn drop(&mut self) {
        if let MultiKind::Live { mp, header, lines } = &self.inner {
            for pb in lines {
                pb.finish_and_clear();
            }
            header.finish_and_clear();
            let _ = mp.clear();
        }
    }
}

/// Terminal width (columns) of the **stderr** TTY, or `None` when
/// stderr isn't a terminal (piped/CI) or the size is unknown. Used to
/// fit a live progress label to the real width instead of a fixed
/// cap. `console` is an indicatif transitive dep — no new surface.
#[must_use]
pub fn term_cols() -> Option<usize> {
    // console::Term::size_checked() → Some((rows, cols)).
    console::Term::stderr()
        .size_checked()
        .map(|(_, cols)| cols as usize)
}

// --- `--readme` self-documentation ------------------------------------

/// If `--readme` is anywhere in argv, print the embedded Markdown to
/// **stdout** and exit 0. Call before arg parsing so a missing
/// subcommand can't pre-empt it. stdout (not stderr) so
/// `tool --readme | less` / agents work. The `binary` is version-
/// locked to its doc via the caller's `include_str!`.
pub fn print_readme_if_requested(md: &str) {
    if readme_arg_present(std::env::args().skip(1)) {
        print!("{md}");
        std::process::exit(0);
    }
}

/// The pure arg-scan (testable without exiting the process).
fn readme_arg_present(args: impl Iterator<Item = String>) -> bool {
    args.take_while(|a| a != "--").any(|a| a == "--readme")
}

// --- atomic file write ------------------------------------------------

/// Atomic file write: create parent dirs, write a sibling temp file,
/// then rename over the target so a reader never sees a partial file.
pub fn write_atomic(path: &Path, bytes: &[u8]) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)?;
        }
    }
    // Sibling temp file (same dir → rename is atomic), then swap.
    let mut tmp = path.as_os_str().to_owned();
    tmp.push(format!(".tmp.{}", std::process::id()));
    let tmp = PathBuf::from(tmp);
    std::fs::write(&tmp, bytes)?;
    std::fs::rename(&tmp, path)
}

// --- coloured JSON ----------------------------------------------------

// Rich-style JSON token palette — identical across every CLI in the
// workspace so their JSON output is visually the same.
const C_RESET: &str = "\x1b[0m";
const C_KEY: &str = "\x1b[1;36m"; // bold cyan
const C_STRING: &str = "\x1b[32m"; // green
const C_NUMBER: &str = "\x1b[33m"; // yellow
const C_LITERAL: &str = "\x1b[35m"; // magenta — bools + null
const C_STRUCT: &str = "\x1b[2m"; // dim — { } [ ] , :

fn write_scalar<W: Write>(out: &mut W, v: &Value) -> io::Result<()> {
    match v {
        Value::Null => write!(out, "{C_LITERAL}null{C_RESET}"),
        Value::Bool(b) => write!(out, "{C_LITERAL}{b}{C_RESET}"),
        Value::Number(n) => write!(out, "{C_NUMBER}{n}{C_RESET}"),
        Value::String(s) => {
            let esc = serde_json::to_string(s).unwrap_or_else(|_| "\"\"".into());
            write!(out, "{C_STRING}{esc}{C_RESET}")
        }
        _ => Ok(()),
    }
}

fn write_coloured_pretty<W: Write>(out: &mut W, v: &Value, depth: usize) -> io::Result<()> {
    let pad = |d: usize| "  ".repeat(d);
    match v {
        Value::Array(a) if a.is_empty() => write!(out, "{C_STRUCT}[]{C_RESET}")?,
        Value::Object(m) if m.is_empty() => write!(out, "{C_STRUCT}{{}}{C_RESET}")?,
        Value::Array(a) => {
            writeln!(out, "{C_STRUCT}[{C_RESET}")?;
            for (i, item) in a.iter().enumerate() {
                write!(out, "{}", pad(depth + 1))?;
                write_coloured_pretty(out, item, depth + 1)?;
                if i + 1 < a.len() {
                    writeln!(out, "{C_STRUCT},{C_RESET}")?;
                } else {
                    writeln!(out)?;
                }
            }
            write!(out, "{}{C_STRUCT}]{C_RESET}", pad(depth))?;
        }
        Value::Object(m) => {
            writeln!(out, "{C_STRUCT}{{{C_RESET}")?;
            for (i, (k, val)) in m.iter().enumerate() {
                let k_esc = serde_json::to_string(k).unwrap_or_else(|_| "\"\"".into());
                write!(
                    out,
                    "{}{C_KEY}{k_esc}{C_RESET}{C_STRUCT}:{C_RESET} ",
                    pad(depth + 1)
                )?;
                write_coloured_pretty(out, val, depth + 1)?;
                if i + 1 < m.len() {
                    writeln!(out, "{C_STRUCT},{C_RESET}")?;
                } else {
                    writeln!(out)?;
                }
            }
            write!(out, "{}{C_STRUCT}}}{C_RESET}", pad(depth))?;
        }
        scalar => write_scalar(out, scalar)?,
    }
    Ok(())
}

/// Write `v` as indented JSON + a trailing newline. `coloured` → the
/// rich token palette (for a TTY); otherwise plain `to_writer_pretty`.
pub fn write_json_pretty<W: Write>(out: &mut W, v: &Value, coloured: bool) -> io::Result<()> {
    if coloured {
        write_coloured_pretty(out, v, 0)?;
    } else {
        serde_json::to_writer_pretty(&mut *out, v)?;
    }
    out.write_all(b"\n")
}

/// Write `v` as a single-line JSON document + a trailing newline
/// (NDJSON-style: one document per line). Never coloured.
pub fn write_ndjson<W: Write>(out: &mut W, v: &Value) -> io::Result<()> {
    serde_json::to_writer(&mut *out, v)?;
    out.write_all(b"\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn output_mode_auto_is_one_of_the_modes() {
        // Can't force a TTY in the test harness; just assert it's a
        // valid variant (piped under `cargo test` ⇒ Ndjson).
        assert!(matches!(
            OutputMode::auto(),
            OutputMode::Table | OutputMode::Json | OutputMode::Ndjson
        ));
    }

    /// The grid `lat` prints, pinned glyph-for-glyph.
    ///
    /// The old assertion here only checked that the CELL TEXT appeared, which
    /// every border style in comfy-table satisfies — so it could not tell
    /// `UTF8_FULL` from `ASCII_MARKDOWN`, and the comfy-table 8 migration
    /// (`load_preset` -> `load_style`, the preset becoming a `TableStyle`
    /// rather than a glyph string) would have gone green whatever it did to the
    /// output. This is the whole visible surface of `lat`'s table mode; a
    /// silent restyle is a real regression even though nothing fails to compile.
    #[test]
    fn styled_table_is_the_utf8_full_grid() {
        let t = styled_table(
            &["A", "Bee"],
            vec![vec!["1".into(), "22".into()], vec!["3".into(), "4".into()]],
            false,
        );
        assert_eq!(
            t.to_string(),
            concat!(
                "┌───┬─────┐\n",
                "│ A ┆ Bee │\n",
                "╞═══╪═════╡\n",
                "│ 1 ┆ 22  │\n",
                "├╌╌╌┼╌╌╌╌╌┤\n",
                "│ 3 ┆ 4   │\n",
                "└───┴─────┘",
            )
        );
    }

    #[test]
    fn ndjson_is_single_line_and_round_trips() {
        let v = json!({"a": 1, "b": ["x", "y"]});
        let mut buf = Vec::new();
        write_ndjson(&mut buf, &v).unwrap();
        let s = String::from_utf8(buf).unwrap();
        assert_eq!(s.matches('\n').count(), 1, "exactly one trailing newline");
        assert!(s.ends_with('\n'));
        let back: Value = serde_json::from_str(s.trim()).unwrap();
        assert_eq!(back, v);
    }

    #[test]
    fn pretty_plain_round_trips_coloured_has_ansi() {
        let v = json!({"k": "v", "n": 2, "ok": true, "nil": null});
        let mut plain = Vec::new();
        write_json_pretty(&mut plain, &v, false).unwrap();
        let ps = String::from_utf8(plain).unwrap();
        assert!(!ps.contains('\x1b'), "plain must have no ANSI");
        let back: Value = serde_json::from_str(ps.trim()).unwrap();
        assert_eq!(back, v);

        let mut col = Vec::new();
        write_json_pretty(&mut col, &v, true).unwrap();
        let cs = String::from_utf8(col).unwrap();
        assert!(cs.contains('\x1b'), "coloured must contain ANSI escapes");
        assert!(cs.ends_with('\n'));
    }

    #[test]
    fn write_atomic_creates_parents_and_overwrites() {
        let base = std::env::temp_dir().join(format!("agscliutil_t_{}", std::process::id()));
        let target = base.join("nested/deep/report.json");
        let _ = std::fs::remove_dir_all(&base);

        write_atomic(&target, b"first").unwrap(); // parents don't exist yet
        assert_eq!(std::fs::read(&target).unwrap(), b"first");
        write_atomic(&target, b"second").unwrap(); // overwrite in place
        assert_eq!(std::fs::read(&target).unwrap(), b"second");
        // No leftover temp sibling.
        let leftover = std::fs::read_dir(target.parent().unwrap())
            .unwrap()
            .filter_map(Result::ok)
            .any(|e| e.file_name().to_string_lossy().contains(".tmp."));
        assert!(!leftover, "temp file not cleaned up");

        let _ = std::fs::remove_dir_all(&base);
    }

    fn kind(ml: &MultiLine) -> &'static str {
        match &ml.inner {
            MultiKind::Live { .. } => "live",
            MultiKind::Static => "static",
            MultiKind::Quiet => "quiet",
        }
    }

    #[test]
    fn multiline_gating_is_quiet_and_nontty_safe() {
        assert_eq!(kind(&MultiLine::start("h", 4, true)), "quiet");
        // stderr is piped under `cargo test` → Static (same caveat as
        // output_mode_auto: the harness can't fake a TTY).
        assert_eq!(kind(&MultiLine::start("h", 4, false)), "static");
        assert_eq!(kind(&MultiLine::start("h", 0, false)), "static");
        // Mutators on a non-Live instance must never panic.
        let ml = MultiLine::start("h", 4, true);
        ml.set_line(99, "x");
        ml.set_header("y");
        assert_eq!(ml.suspend(|| 7), 7);
    }

    #[test]
    fn term_cols_is_safe_off_a_tty() {
        // Piped under `cargo test` → None; must never panic, and any
        // Some value is a sane positive width.
        match term_cols() {
            None => {}
            Some(c) => assert!(c > 0 && c < 100_000),
        }
    }

    #[test]
    fn readme_arg_present_detects_flag_before_double_dash() {
        let v = |xs: &[&str]| readme_arg_present(xs.iter().map(std::string::ToString::to_string));
        assert!(v(&["--readme"]));
        assert!(v(&["validate", "--quiet", "--readme"]));
        assert!(!v(&["validate", "--quiet"]));
        assert!(!v(&[]));
        // After `--`, args are passthrough — don't treat as the flag.
        assert!(!v(&["run", "--", "--readme"]));
    }

    fn strip_ansi(s: &str) -> String {
        [C_RESET, C_KEY, C_STRING, C_NUMBER, C_LITERAL, C_STRUCT]
            .iter()
            .fold(s.to_string(), |acc, c| acc.replace(c, ""))
    }

    /// The coloured pretty-printer must preserve structure exactly and paint each
    /// token its own colour. Stripping the palette back out must yield valid,
    /// correctly-indented JSON that round-trips — which pins the comma logic (a
    /// wrong separator is invalid JSON), the inline empty-collection rendering
    /// (`[]`/`{}`, not a multi-line pair), and the two-space-per-depth indentation.
    /// The raw output pins each scalar arm to its own palette entry (an arm deleted
    /// to the `_ => Ok(())` catch-all writes nothing, and its colour disappears).
    #[test]
    fn coloured_pretty_preserves_structure_and_paints_each_scalar() {
        let v = json!({
            "nil": null,
            "flag": true,
            "count": 2,
            "name": "hi",
            "empty_arr": [],
            "empty_obj": {},
            "nested": {"s": "hi"},
            "nums": [1, 2],
            // a non-scalar array element, so the item-recursion depth is observable
            "arr_of_obj": [{"deep": 1}],
        });
        let mut buf = Vec::new();
        write_coloured_pretty(&mut buf, &v, 0).unwrap();
        let cs = String::from_utf8(buf).unwrap();

        // each scalar carries its own palette entry
        assert!(
            cs.contains(&format!("{C_LITERAL}null{C_RESET}")),
            "null colour"
        );
        assert!(
            cs.contains(&format!("{C_LITERAL}true{C_RESET}")),
            "bool colour"
        );
        assert!(
            cs.contains(&format!("{C_NUMBER}2{C_RESET}")),
            "number colour"
        );
        assert!(
            cs.contains(&format!("{C_STRING}\"hi\"{C_RESET}")),
            "string colour"
        );

        // strip the palette → must be valid, structure-preserving JSON
        let plain = strip_ansi(&cs);
        let back: Value = serde_json::from_str(&plain)
            .unwrap_or_else(|e| panic!("stripped output is not valid JSON ({e}):\n{plain}"));
        assert_eq!(back, v, "coloured pretty changed the data");

        // empty collections render inline, not as a multi-line pair
        assert!(
            plain.contains("\"empty_arr\": []"),
            "empty array not inline:\n{plain}"
        );
        assert!(
            plain.contains("\"empty_obj\": {}"),
            "empty object not inline:\n{plain}"
        );

        // two spaces per depth: a nested object key and an array element both sit at
        // depth 2 (four leading spaces)
        assert!(
            plain.contains("\n    \"s\": "),
            "nested key mis-indented:\n{plain}"
        );
        assert!(
            plain.contains("\n    1"),
            "array element mis-indented:\n{plain}"
        );
        // a non-scalar array element carries the item-recursion depth: its key sits
        // one level deeper again (six spaces)
        assert!(
            plain.contains("\n      \"deep\": "),
            "array-of-object element mis-indented:\n{plain}"
        );
    }

    /// The colour gate is off whenever `no_color` is set — and, in a piped test
    /// harness, off regardless (stdout is not a TTY). Asserting it stays off for
    /// both argument values pins the `!no_color && … && is_terminal()` conjunction:
    /// a return flipped to `true`, or an `&&` loosened to `||`, would let the
    /// NO_COLOR-unset term turn colour on where the real gate keeps it off.
    /// (The dropped-`!` and always-`false` mutants are TTY-masked here — both sides
    /// are `false` off a TTY — and are recorded as harness residuals.)
    #[test]
    fn colour_gate_stays_off_off_a_tty() {
        assert!(
            !colour_enabled(true),
            "explicit no_color must force colour off"
        );
        assert!(
            !colour_enabled(false),
            "no TTY under cargo test → colour off"
        );
    }

    /// A determinate bar is hidden off a stderr TTY (piped/CI) and under `--quiet`.
    /// `is_hidden()` cannot see the gate — indicatif auto-hides a stderr bar off a
    /// TTY, so even a broken gate's live bar reads hidden — but a hidden bar carries
    /// no length while `ProgressBar::new(len)` does, so `length()` distinguishes
    /// them. `cargo test` pipes stderr, so both cases must have `None`; a flipped
    /// `&&` or dropped `!` would build a `Some(len)` bar.
    #[test]
    fn progress_bar_is_hidden_off_a_tty_and_under_quiet() {
        assert_eq!(progress_bar(10, false).length(), None);
        assert_eq!(progress_bar(10, true).length(), None);
    }

    fn spinner_kind(s: &Spinner) -> &'static str {
        match &s.inner {
            Kind::Live(_) => "live",
            Kind::Static => "static",
            Kind::Quiet => "quiet",
        }
    }

    /// The spinner gate: `--quiet` → a silent no-op; off a stderr TTY → one static
    /// line, never a live ticker. `cargo test` pipes stderr, so the non-quiet case
    /// must be Static — a dropped `!` on the TTY check would build a Live spinner.
    #[test]
    fn spinner_gating_is_quiet_and_nontty_safe() {
        assert_eq!(spinner_kind(&Spinner::start("m", true)), "quiet");
        assert_eq!(spinner_kind(&Spinner::start("m", false)), "static");
        // mutators on a non-Live spinner must not panic
        Spinner::start("m", true).set("x");
    }
}
