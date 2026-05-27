//! `ags4-check --tui` — interactive findings browser (experiment).
//!
//! Compiled in ONLY with `--features tui`. A pure, read-only consumer
//! of the `Findings` value `check_file` already produced: validate
//! once, then scroll/filter that in memory. No rule, parser, or
//! `Finding` change. The plain / `--json` / exit-code paths in
//! `ags4_check.rs` are untouched — see the plan's guardrails.
//!
//! Layout: a scrollable findings table on top, a stats panel beneath,
//! a status/help line at the bottom. Keys: ↑↓/j/k move, PgUp/PgDn (or
//! Ctrl-u/Ctrl-d) page, g/G top/bottom, `/` filter, q/Esc/Ctrl-c quit.
//!
//! crossterm is used via `ratatui::crossterm` so the backend/event
//! types are guaranteed version-matched to the linked ratatui.

use std::io::{self, Stdout};

use ags4_validator::{DictVersion, Findings, findings};
use ratatui::Terminal;
use ratatui::backend::{Backend, CrosstermBackend};
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::crossterm::execute;
use ratatui::crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::layout::{Constraint, Layout, Margin, Rect};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Block, Borders, Cell, Clear, Paragraph, Row, Scrollbar, ScrollbarOrientation, ScrollbarState,
    Table, TableState, Wrap,
};

/// Flattened finding (owned). The one-time clone of a few hundred short
/// strings is irrelevant next to validation; owning sidesteps borrow/
/// lifetime gymnastics through the event loop + `TableState`.
struct FRow {
    rule: String,
    line: String,
    group: String,
    desc: String,
}

/// Which column the `/` filter matches against. `All` keeps the
/// original behaviour (any column); the rest scope it (e.g. "filter by
/// rule"). `Tab` cycles through these.
#[derive(Clone, Copy, PartialEq)]
enum FilterCol {
    All,
    Rule,
    Line,
    Group,
    Desc,
}

impl FilterCol {
    fn next(self) -> Self {
        match self {
            FilterCol::All => FilterCol::Rule,
            FilterCol::Rule => FilterCol::Line,
            FilterCol::Line => FilterCol::Group,
            FilterCol::Group => FilterCol::Desc,
            FilterCol::Desc => FilterCol::All,
        }
    }
    fn label(self) -> &'static str {
        match self {
            FilterCol::All => "All",
            FilterCol::Rule => "Rule",
            FilterCol::Line => "Line",
            FilterCol::Group => "Group",
            FilterCol::Desc => "Description",
        }
    }
}

struct App {
    file: String,
    dict: &'static str,
    rows: Vec<FRow>,
    visible: Vec<usize>,
    state: TableState,
    filter: String,
    filtering: bool,
    filter_col: FilterCol,
    /// Detail popup for the selected finding (full, wrapped message —
    /// the table truncates it). `detail_scroll` scrolls long text.
    detail: bool,
    detail_scroll: u16,
    by_rule: Vec<(String, usize)>,
    total: usize,
    /// Data-area height from the last draw — drives PgUp/PgDn.
    page: usize,
    quit: bool,
}

impl App {
    fn new(found: &Findings, file: &str, dict: DictVersion) -> Self {
        let mut rows = Vec::new();
        for (rule, items) in found {
            let short = rule
                .strip_prefix("AGS Format Rule ")
                .unwrap_or(rule)
                .to_string();
            for f in items {
                rows.push(FRow {
                    rule: short.clone(),
                    line: f.line.map(|l| l.to_string()).unwrap_or_else(|| "-".into()),
                    group: f.group.clone(),
                    desc: f.desc.clone(),
                });
            }
        }
        let by_rule = findings::count_by_rule(found)
            .into_iter()
            .map(|(r, n)| {
                (
                    r.strip_prefix("AGS Format Rule ").unwrap_or(r).to_string(),
                    n,
                )
            })
            .collect();
        let mut app = App {
            file: file.to_string(),
            dict: dict.as_str(),
            visible: (0..rows.len()).collect(),
            rows,
            state: TableState::default(),
            filter: String::new(),
            filtering: false,
            filter_col: FilterCol::All,
            detail: false,
            detail_scroll: 0,
            by_rule,
            total: findings::count(found),
            page: 10,
            quit: false,
        };
        if !app.visible.is_empty() {
            app.state.select(Some(0));
        }
        app
    }

    /// Recompute `visible` after a filter edit; keep selection in range.
    /// The match is scoped to `filter_col` (e.g. "filter by rule");
    /// `All` matches any column.
    fn refilter(&mut self) {
        let q = self.filter.to_ascii_lowercase();
        let col = self.filter_col;
        self.visible = self
            .rows
            .iter()
            .enumerate()
            .filter(|(_, r)| {
                if q.is_empty() {
                    return true;
                }
                let hit = |s: &str| s.to_ascii_lowercase().contains(&q);
                match col {
                    FilterCol::All => hit(&r.rule) || hit(&r.line) || hit(&r.group) || hit(&r.desc),
                    FilterCol::Rule => hit(&r.rule),
                    FilterCol::Line => hit(&r.line),
                    FilterCol::Group => hit(&r.group),
                    FilterCol::Desc => hit(&r.desc),
                }
            })
            .map(|(i, _)| i)
            .collect();
        let sel = if self.visible.is_empty() {
            None
        } else {
            Some(
                self.state
                    .selected()
                    .unwrap_or(0)
                    .min(self.visible.len() - 1),
            )
        };
        self.state.select(sel);
    }

    /// The `FRow` under the current selection, if any.
    fn selected_frow(&self) -> Option<&FRow> {
        let vis = self.state.selected()?;
        self.visible.get(vis).map(|&i| &self.rows[i])
    }

    fn cycle_filter_col(&mut self) {
        self.filter_col = self.filter_col.next();
        self.refilter();
    }

    fn move_by(&mut self, delta: isize) {
        if self.visible.is_empty() {
            return;
        }
        let cur = self.state.selected().unwrap_or(0) as isize;
        let last = self.visible.len() as isize - 1;
        self.state
            .select(Some(cur.saturating_add(delta).clamp(0, last) as usize));
    }

    fn select_edge(&mut self, top: bool) {
        if self.visible.is_empty() {
            return;
        }
        self.state
            .select(Some(if top { 0 } else { self.visible.len() - 1 }));
    }
}

/// Entry point called from `ags4_check.rs` (only when `--tui` AND a
/// real terminal). Always restores the terminal — even on panic or a
/// loop error — so the user's shell is never left in raw mode.
pub fn run(found: &Findings, file: &str, dict: DictVersion) -> io::Result<()> {
    install_panic_hook();
    let mut terminal = setup()?;
    let res = event_loop(&mut terminal, App::new(found, file, dict));
    restore(&mut terminal)?;
    res
}

fn setup() -> io::Result<Terminal<CrosstermBackend<Stdout>>> {
    enable_raw_mode()?;
    let mut out = io::stdout();
    execute!(out, EnterAlternateScreen)?;
    Terminal::new(CrosstermBackend::new(out))
}

fn restore<B: Backend + std::io::Write>(t: &mut Terminal<B>) -> io::Result<()> {
    disable_raw_mode()?;
    execute!(t.backend_mut(), LeaveAlternateScreen)?;
    t.show_cursor()?;
    Ok(())
}

/// Restore the terminal *before* the default panic message prints,
/// otherwise a panic mid-draw leaves the shell with no echo. Installed
/// before `enable_raw_mode`.
fn install_panic_hook() {
    let prev = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
        prev(info);
    }));
}

fn event_loop<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, &mut app))?;
        // Blocking read — nothing animates, validation is already done,
        // so this is zero idle CPU.
        if let Event::Key(k) = event::read()? {
            if k.kind != KeyEventKind::Press {
                continue;
            }
            let ctrl = k.modifiers.contains(KeyModifiers::CONTROL);
            if ctrl && k.code == KeyCode::Char('c') {
                app.quit = true;
            } else if app.detail {
                // Detail popup: scroll the full message, back out with
                // Esc/Enter/q (q here closes the popup, not the app).
                match k.code {
                    KeyCode::Esc | KeyCode::Enter | KeyCode::Char('q') => {
                        app.detail = false;
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        app.detail_scroll = app.detail_scroll.saturating_add(1);
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        app.detail_scroll = app.detail_scroll.saturating_sub(1);
                    }
                    KeyCode::PageDown => {
                        app.detail_scroll = app.detail_scroll.saturating_add(10);
                    }
                    KeyCode::PageUp => {
                        app.detail_scroll = app.detail_scroll.saturating_sub(10);
                    }
                    _ => {}
                }
            } else if app.filtering {
                match k.code {
                    KeyCode::Esc => {
                        app.filtering = false;
                        app.filter.clear();
                        app.refilter();
                    }
                    KeyCode::Enter => app.filtering = false,
                    KeyCode::Tab => app.cycle_filter_col(),
                    KeyCode::Backspace => {
                        app.filter.pop();
                        app.refilter();
                    }
                    KeyCode::Char(c) => {
                        app.filter.push(c);
                        app.refilter();
                    }
                    _ => {}
                }
            } else {
                match k.code {
                    KeyCode::Char('q') | KeyCode::Esc => app.quit = true,
                    KeyCode::Enter => {
                        if app.selected_frow().is_some() {
                            app.detail = true;
                            app.detail_scroll = 0;
                        }
                    }
                    KeyCode::Tab => app.cycle_filter_col(),
                    KeyCode::Down | KeyCode::Char('j') => app.move_by(1),
                    KeyCode::Up | KeyCode::Char('k') => app.move_by(-1),
                    KeyCode::PageDown => app.move_by(app.page as isize),
                    KeyCode::PageUp => app.move_by(-(app.page as isize)),
                    KeyCode::Char('d') if ctrl => app.move_by(app.page as isize / 2),
                    KeyCode::Char('u') if ctrl => app.move_by(-(app.page as isize) / 2),
                    KeyCode::Char('g') => app.select_edge(true),
                    KeyCode::Char('G') => app.select_edge(false),
                    KeyCode::Char('/') => app.filtering = true,
                    _ => {}
                }
            }
        }
        if app.quit {
            return Ok(());
        }
    }
}

fn ui(f: &mut ratatui::Frame, app: &mut App) {
    let chunks = Layout::vertical([
        Constraint::Min(3),    // findings table (the scrollable data)
        Constraint::Length(6), // stats panel
        Constraint::Length(1), // status / filter line
    ])
    .split(f.area());

    // Track the visible data height for PgUp/PgDn (minus header + box).
    app.page = (chunks[0].height as usize).saturating_sub(3).max(1);

    // --- findings table -------------------------------------------------
    let header = Row::new(["Rule", "Line", "Group", "Description"]).style(
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    );
    let body = app.visible.iter().map(|&i| {
        let r = &app.rows[i];
        Row::new(vec![
            Cell::from(r.rule.clone()),
            Cell::from(r.line.clone()),
            Cell::from(r.group.clone()),
            Cell::from(r.desc.clone()),
        ])
    });
    let title = format!(
        " {} — {} finding(s){} ",
        app.file,
        app.total,
        if app.filter.is_empty() {
            String::new()
        } else {
            format!(", {} shown", app.visible.len())
        }
    );
    let table = Table::new(
        body,
        [
            Constraint::Length(6),
            Constraint::Length(6),
            Constraint::Length(8),
            Constraint::Min(10),
        ],
    )
    .header(header)
    .block(Block::default().borders(Borders::ALL).title(title))
    .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED))
    .highlight_symbol("▌");
    f.render_stateful_widget(table, chunks[0], &mut app.state);

    // Scrollbar on the table's right edge.
    let mut sb = ScrollbarState::new(app.visible.len()).position(app.state.selected().unwrap_or(0));
    f.render_stateful_widget(
        Scrollbar::new(ScrollbarOrientation::VerticalRight),
        chunks[0].inner(Margin {
            vertical: 1,
            horizontal: 0,
        }),
        &mut sb,
    );

    // --- stats panel ----------------------------------------------------
    let verdict = if app.total == 0 { "PASS" } else { "FAIL" };
    let per_rule = app
        .by_rule
        .iter()
        .map(|(r, n)| format!("R{r}:{n}"))
        .collect::<Vec<_>>()
        .join("  ");
    let stats = Paragraph::new(vec![
        Line::from(format!(
            "Total findings: {}    Verdict: {}    Dictionary: AGS {}",
            app.total, verdict, app.dict
        )),
        Line::from(format!("Per rule:  {per_rule}")),
    ])
    .wrap(Wrap { trim: true })
    .block(Block::default().borders(Borders::ALL).title(" Summary "));
    f.render_widget(stats, chunks[1]);

    // --- status / filter line ------------------------------------------
    let col = app.filter_col.label();
    let status = if app.filtering {
        Line::from(format!("/[{col}] {}_   (Tab: column)", app.filter)).fg(Color::Yellow)
    } else if app.filter.is_empty() {
        Line::from(format!(
            "↑↓ move · Enter detail · / filter · Tab column[{col}] · g/G · q quit"
        ))
        .fg(Color::DarkGray)
    } else {
        Line::from(format!(
            "filter[{col}]: {:?} · Enter detail · / edit · Tab column · q quit",
            app.filter
        ))
        .fg(Color::DarkGray)
    };
    f.render_widget(Paragraph::new(status), chunks[2]);

    // --- detail popup (Enter) ------------------------------------------
    // The table truncates wide messages; this shows the full finding,
    // wrapped and scrollable, over a cleared centred panel.
    if app.detail {
        if let Some(r) = app.selected_frow() {
            let label = |t| {
                Span::styled(
                    t,
                    Style::default()
                        .add_modifier(Modifier::BOLD)
                        .fg(Color::Cyan),
                )
            };
            let lines = vec![
                Line::from(vec![
                    label("Rule   "),
                    Span::raw(format!("AGS Format Rule {}", r.rule)),
                ]),
                Line::from(vec![label("Line   "), Span::raw(r.line.clone())]),
                Line::from(vec![label("Group  "), Span::raw(r.group.clone())]),
                Line::from(""),
                Line::from(r.desc.clone()),
            ];
            let area = centered_rect(80, 60, f.area());
            let popup = Paragraph::new(lines)
                .wrap(Wrap { trim: false })
                .scroll((app.detail_scroll, 0))
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(" Finding — Esc/Enter close · ↑↓ scroll "),
                );
            f.render_widget(Clear, area);
            f.render_widget(popup, area);
        }
    }
}

/// A centred rectangle `px`%×`py`% of `area` — the detail popup frame.
fn centered_rect(px: u16, py: u16, area: Rect) -> Rect {
    let vert = Layout::vertical([
        Constraint::Percentage((100 - py) / 2),
        Constraint::Percentage(py),
        Constraint::Percentage((100 - py) / 2),
    ])
    .split(area);
    Layout::horizontal([
        Constraint::Percentage((100 - px) / 2),
        Constraint::Percentage(px),
        Constraint::Percentage((100 - px) / 2),
    ])
    .split(vert[1])[1]
}
