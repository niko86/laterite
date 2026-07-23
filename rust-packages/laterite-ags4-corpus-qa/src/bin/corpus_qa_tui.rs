//! `crawl --pick` — interactive multi-select of harvested candidates.
//!
//! Compiled in ONLY with `--features tui` (mirrors the validator's
//! `--tui` discipline: default build links no ratatui; `--pick`
//! without the feature is a clean "rebuild with --features tui"
//! error). Structure forked from
//! `laterite-ags4-validator/src/bin/ags4_check_tui.rs`: same
//! setup/restore/panic-hook, blocking event loop, Table + `TableState`
//! + Scrollbar + `/` filter — with a `[x]` multi-select column.

use std::io::{self, Stdout};
use std::path::PathBuf;

use ratatui::Terminal;
use ratatui::backend::{Backend, CrosstermBackend};
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::crossterm::execute;
use ratatui::crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::layout::{Constraint, Layout, Margin};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::text::Line;
use ratatui::widgets::{
    Block, Borders, Cell, Paragraph, Row, Scrollbar, ScrollbarOrientation, ScrollbarState, Table,
    TableState,
};

struct FileRow {
    path: PathBuf,
    label: String,
    size: String,
    sel: bool,
}

struct App {
    rows: Vec<FileRow>,
    visible: Vec<usize>,
    state: TableState,
    filter: String,
    filtering: bool,
    page: usize,
    quit: bool,
    confirmed: bool,
}

fn human(n: u64) -> String {
    const U: [&str; 4] = ["B", "K", "M", "G"];
    let mut v = n as f64;
    let mut i = 0;
    while v >= 1024.0 && i < 3 {
        v /= 1024.0;
        i += 1;
    }
    if i == 0 {
        format!("{n}{}", U[0])
    } else {
        format!("{v:.1}{}", U[i])
    }
}

impl App {
    fn new(files: &[PathBuf]) -> Self {
        let rows: Vec<FileRow> = files
            .iter()
            .map(|p| FileRow {
                size: human(std::fs::metadata(p).map_or(0, |m| m.len())),
                label: p.to_string_lossy().into_owned(),
                path: p.clone(),
                sel: false,
            })
            .collect();
        let mut a = App {
            visible: (0..rows.len()).collect(),
            rows,
            state: TableState::default(),
            filter: String::new(),
            filtering: false,
            page: 10,
            quit: false,
            confirmed: false,
        };
        if !a.visible.is_empty() {
            a.state.select(Some(0));
        }
        a
    }

    fn refilter(&mut self) {
        let q = self.filter.to_ascii_lowercase();
        self.visible = self
            .rows
            .iter()
            .enumerate()
            .filter(|(_, r)| q.is_empty() || r.label.to_ascii_lowercase().contains(&q))
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

    fn move_by(&mut self, d: isize) {
        if self.visible.is_empty() {
            return;
        }
        let cur = self.state.selected().unwrap_or(0) as isize;
        let last = self.visible.len() as isize - 1;
        self.state
            .select(Some(cur.saturating_add(d).clamp(0, last) as usize));
    }

    fn toggle_current(&mut self) {
        if let Some(v) = self.state.selected() {
            if let Some(&i) = self.visible.get(v) {
                self.rows[i].sel = !self.rows[i].sel;
            }
        }
    }

    fn toggle_all_visible(&mut self) {
        let any_unset = self.visible.iter().any(|&i| !self.rows[i].sel);
        for &i in &self.visible {
            self.rows[i].sel = any_unset;
        }
    }

    fn selected_count(&self) -> usize {
        self.rows.iter().filter(|r| r.sel).count()
    }
}

/// Show the picker; return the chosen paths (empty = cancelled, the
/// caller then aborts the crawl with a message). Terminal is always
/// restored, even on panic.
pub fn pick(files: &[PathBuf]) -> io::Result<Vec<PathBuf>> {
    if files.is_empty() {
        return Ok(Vec::new());
    }
    install_panic_hook();
    let mut term = setup()?;
    let res = event_loop(&mut term, App::new(files));
    restore(&mut term)?;
    res
}

fn setup() -> io::Result<Terminal<CrosstermBackend<Stdout>>> {
    enable_raw_mode()?;
    let mut out = io::stdout();
    execute!(out, EnterAlternateScreen)?;
    Terminal::new(CrosstermBackend::new(out))
}

// ratatui 0.30 gave `Backend` an associated `Error`; pin it to `io::Error`
// (what CrosstermBackend produces) so `?` keeps unifying with io::Result here.
fn restore<B: Backend<Error = io::Error> + io::Write>(t: &mut Terminal<B>) -> io::Result<()> {
    disable_raw_mode()?;
    execute!(t.backend_mut(), LeaveAlternateScreen)?;
    t.show_cursor()?;
    Ok(())
}

fn install_panic_hook() {
    let prev = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
        prev(info);
    }));
}

fn event_loop<B: Backend<Error = io::Error>>(
    term: &mut Terminal<B>,
    mut app: App,
) -> io::Result<Vec<PathBuf>> {
    loop {
        term.draw(|f| ui(f, &mut app))?;
        if let Event::Key(k) = event::read()? {
            if k.kind != KeyEventKind::Press {
                continue;
            }
            let ctrl = k.modifiers.contains(KeyModifiers::CONTROL);
            if ctrl && k.code == KeyCode::Char('c') {
                app.quit = true;
            } else if app.filtering {
                match k.code {
                    KeyCode::Esc => {
                        app.filtering = false;
                        app.filter.clear();
                        app.refilter();
                    }
                    KeyCode::Enter => app.filtering = false,
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
                        app.confirmed = true;
                        app.quit = true;
                    }
                    KeyCode::Char(' ') => app.toggle_current(),
                    KeyCode::Char('a') => app.toggle_all_visible(),
                    KeyCode::Char('/') => app.filtering = true,
                    KeyCode::Down | KeyCode::Char('j') => app.move_by(1),
                    KeyCode::Up | KeyCode::Char('k') => app.move_by(-1),
                    KeyCode::PageDown => app.move_by(app.page as isize),
                    KeyCode::PageUp => app.move_by(-(app.page as isize)),
                    KeyCode::Char('g') => app.state.select(Some(0)),
                    KeyCode::Char('G') if !app.visible.is_empty() => {
                        app.state.select(Some(app.visible.len() - 1));
                    }
                    _ => {}
                }
            }
        }
        if app.quit {
            return Ok(if app.confirmed {
                app.rows
                    .into_iter()
                    .filter(|r| r.sel)
                    .map(|r| r.path)
                    .collect()
            } else {
                Vec::new()
            });
        }
    }
}

fn ui(f: &mut ratatui::Frame, app: &mut App) {
    let chunks = Layout::vertical([Constraint::Min(3), Constraint::Length(1)]).split(f.area());
    app.page = (chunks[0].height as usize).saturating_sub(3).max(1);

    let header = Row::new(["", "Size", "Source"]).style(
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    );
    let body = app.visible.iter().map(|&i| {
        let r = &app.rows[i];
        Row::new(vec![
            Cell::from(if r.sel { "[x]" } else { "[ ]" }),
            Cell::from(r.size.clone()),
            Cell::from(r.label.clone()),
        ])
    });
    let title = format!(
        " select files — {} of {} selected ",
        app.selected_count(),
        app.rows.len()
    );
    let table = Table::new(
        body,
        [
            Constraint::Length(3),
            Constraint::Length(8),
            Constraint::Min(10),
        ],
    )
    .header(header)
    .block(Block::default().borders(Borders::ALL).title(title))
    .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED))
    .highlight_symbol("▌");
    f.render_stateful_widget(table, chunks[0], &mut app.state);

    let mut sb = ScrollbarState::new(app.visible.len()).position(app.state.selected().unwrap_or(0));
    f.render_stateful_widget(
        Scrollbar::new(ScrollbarOrientation::VerticalRight),
        chunks[0].inner(Margin {
            vertical: 1,
            horizontal: 0,
        }),
        &mut sb,
    );

    let status = if app.filtering {
        Line::from(format!("/{}_", app.filter)).fg(Color::Yellow)
    } else {
        Line::from("Space toggle · a all · / filter · Enter confirm · q/Esc cancel")
            .fg(Color::DarkGray)
    };
    f.render_widget(Paragraph::new(status), chunks[1]);
}
