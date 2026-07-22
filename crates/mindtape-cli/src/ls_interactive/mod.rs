//! `mt ls --interactive` — a terminal picker over the `ls` list.
//!
//! Renders the same table `ls` prints, then holds the terminal in a raw-mode key
//! loop on an alternate screen. Typing the `#`-column ordinal highlights a task
//! (reverse-video); Enter opens the first highlighted task in the configured
//! editor (`go`), Ctrl+Enter opens every listed task, Space re-evaluates the
//! project, and `q` / Ctrl+C leaves. See `truth/tasks/ls-interactive-mode.typ`.
//!
//! Terminal entry/exit (raw mode, alternate screen, the [`tui::Tui`] RAII
//! guard) lives in the [`tui`] submodule; this module owns the picker's
//! state machine — key classification, the digit-buffer selection, and the
//! draw/refresh loop built on top of it.

use std::io::{self, IsTerminal, Write};
use std::path::PathBuf;

use anyhow::{Result, bail};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use crossterm::terminal::{Clear, ClearType};
use crossterm::{cursor, execute};

use anstyle::{Effects, Style};
use mindtape::{Config, Readiness, Task};

use crate::ls::{LsArgs, select_view};
use crate::output::OutputFormat;
use crate::render::color::{Painter, is_color_active};
use crate::render::table::render_highlighted;
use crate::{go, util};

mod tui;
use tui::{Tui, suspend};

/// The live key hints shown in the footer.
const HINTS: &str =
    "0-9 select · ⌫ del · Del clear · ⏎ open · ^⏎ open all · Space refresh · q quit";

/// Run the interactive picker over the view `args` selects from `project`.
///
/// # Errors
/// Returns an error when stdin/stdout is not a TTY, the format is not the table,
/// the view cannot be built, the terminal cannot enter raw mode, or the editor
/// fails to launch.
pub fn run(args: &LsArgs, project: util::Project) -> Result<()> {
    guard(args)?;
    let mut state = State::from_project(project, args)?;
    let tui = Tui::enter()?;
    let result = event_loop(&mut state, &tui, args);
    drop(tui); // restore the terminal before any error surfaces
    result
}

/// Reject the modes interactive cannot serve: a non-table format and a
/// non-interactive terminal (piped stdin or stdout).
fn guard(args: &LsArgs) -> Result<()> {
    if !matches!(args.format, OutputFormat::Table) {
        bail!("--interactive is table-only; drop --format json/csv");
    }
    if !io::stdin().is_terminal() || !io::stdout().is_terminal() {
        bail!("--interactive needs a TTY on stdin and stdout (not available when piped)");
    }
    Ok(())
}

/// The loop's mutable state: the resolved view plus the digit buffer.
struct State {
    config: Config,
    root: PathBuf,
    tasks: Vec<Task>,
    readiness: Readiness,
    /// Accumulated `#`-ordinal digits; empty means nothing is selected.
    buffer: String,
}

impl State {
    fn from_project(project: util::Project, args: &LsArgs) -> Result<Self> {
        let config = project.config.clone();
        let root = project.root.clone();
        let (tasks, readiness) = select_view(args, project)?;
        Ok(Self {
            config,
            root,
            tasks,
            readiness,
            buffer: String::new(),
        })
    }

    /// Re-open the project, recompute the view, and clear the selection.
    fn refresh(&mut self, args: &LsArgs) -> Result<()> {
        *self = Self::from_project(util::open()?, args)?;
        Ok(())
    }

    /// A per-row mask: row `i` is highlighted when its 1-based ordinal has the
    /// buffer as a prefix. An empty buffer highlights nothing.
    fn highlight_mask(&self) -> Vec<bool> {
        (0..self.tasks.len()).map(|i| self.matches(i)).collect()
    }

    /// Index of the first (lowest-ordinal, topmost) highlighted row.
    fn first_highlighted(&self) -> Option<usize> {
        (0..self.tasks.len()).find(|&i| self.matches(i))
    }

    fn matches(&self, index: usize) -> bool {
        !self.buffer.is_empty() && (index + 1).to_string().starts_with(&self.buffer)
    }

    /// Absolute paths for the given display indices, skipping any out of range.
    fn paths(&self, indices: &[usize]) -> Vec<PathBuf> {
        indices
            .iter()
            .filter_map(|&i| self.tasks.get(i))
            .map(|task| self.root.join(&task.file))
            .collect()
    }
}

/// What the loop does after one key.
enum Action {
    /// Stay; redraw from the current state.
    Continue,
    /// Re-open the project and recompute the view.
    Refresh,
    /// Leave interactive mode.
    Quit,
}

/// The intent a key carries, decoded independently of any IO so it stays
/// testable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Intent {
    Digit(char),
    DelLast,
    Clear,
    OpenFirst,
    OpenAll,
    Refresh,
    Quit,
    Ignore,
}

/// Map a key event to its [`Intent`].
///
/// `Ctrl+Enter` (open all) and `Ctrl+Backspace` (clear) only carry the modifier
/// on terminals that negotiated the keyboard-enhancement protocol; elsewhere they
/// fall back to plain Enter / Backspace, and `Del` is the reliable clear.
fn classify(key: KeyEvent) -> Intent {
    let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
    match key.code {
        KeyCode::Char('c') if ctrl => Intent::Quit,
        KeyCode::Char('q') => Intent::Quit,
        KeyCode::Char(' ') => Intent::Refresh,
        KeyCode::Char(c) if c.is_ascii_digit() => Intent::Digit(c),
        KeyCode::Backspace if ctrl => Intent::Clear,
        KeyCode::Backspace => Intent::DelLast,
        KeyCode::Delete => Intent::Clear,
        KeyCode::Enter if ctrl => Intent::OpenAll,
        KeyCode::Enter => Intent::OpenFirst,
        _ => Intent::Ignore,
    }
}

fn event_loop(state: &mut State, tui: &Tui, args: &LsArgs) -> Result<()> {
    loop {
        draw(state)?;
        let Event::Key(key) = event::read()? else {
            continue;
        };
        if key.kind != KeyEventKind::Press {
            continue;
        }
        match apply(state, tui, classify(key))? {
            Action::Continue => {}
            Action::Refresh => state.refresh(args)?,
            Action::Quit => return Ok(()),
        }
    }
}

/// Apply an intent: mutate the buffer, or open tasks / refresh.
fn apply(state: &mut State, tui: &Tui, intent: Intent) -> Result<Action> {
    match intent {
        Intent::Digit(c) => {
            state.buffer.push(c);
            Ok(Action::Continue)
        }
        Intent::DelLast => {
            state.buffer.pop();
            Ok(Action::Continue)
        }
        Intent::Clear => {
            state.buffer.clear();
            Ok(Action::Continue)
        }
        Intent::OpenFirst => open(state, tui, &collect_first(state)),
        Intent::OpenAll => open(state, tui, &(0..state.tasks.len()).collect::<Vec<_>>()),
        Intent::Refresh => Ok(Action::Refresh),
        Intent::Quit => Ok(Action::Quit),
        Intent::Ignore => Ok(Action::Continue),
    }
}

fn collect_first(state: &State) -> Vec<usize> {
    state.first_highlighted().into_iter().collect()
}

/// Open the tasks at `indices` in the editor, suspending the TUI around the
/// spawn so a terminal editor gets a clean screen. No indices is a no-op.
fn open(state: &State, tui: &Tui, indices: &[usize]) -> Result<Action> {
    let paths = state.paths(indices);
    if paths.is_empty() {
        return Ok(Action::Continue);
    }
    let template = go::resolve_editor(&state.config)?;
    suspend(tui, || {
        for path in &paths {
            go::spawn_editor(&template, path)?;
        }
        Ok(())
    })?;
    Ok(Action::Refresh)
}

/// Redraw the frame: the highlighted table plus the footer, cleared each time.
fn draw(state: &State) -> Result<()> {
    let mask = state.highlight_mask();
    let table = render_highlighted(&state.tasks, &state.config, &state.readiness, &mask);
    let body = format!("{table}\n{}\n", footer(&state.buffer));
    // Raw mode does not translate `\n`; move every line back to column 0.
    let body = body.replace('\n', "\r\n");
    let mut out = io::stdout();
    execute!(out, Clear(ClearType::All), cursor::MoveTo(0, 0))?;
    out.write_all(body.as_bytes())?;
    out.flush()?;
    Ok(())
}

/// The footer: a dim hint line, preceded by the live `#` buffer when typing.
fn footer(buffer: &str) -> String {
    let painter = Painter::new(is_color_active());
    let hints = painter.paint(HINTS, Style::new().effects(Effects::DIMMED));
    if buffer.is_empty() {
        hints
    } else {
        format!("#{buffer}\n{hints}")
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
