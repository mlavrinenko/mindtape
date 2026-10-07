//! `mt ls --interactive` — a terminal picker over the `ls` list.
//!
//! Renders the same table `ls` prints, then holds the terminal in a raw-mode key
//! loop on an alternate screen. Typing a `slug#` stem hash highlights the tasks
//! whose hash starts with it (reverse-video); Enter opens the first highlighted
//! task in the configured editor (`go`), Ctrl+Enter opens every listed task,
//! Space re-evaluates the project, and Esc / Ctrl+C leaves. Every letter is a
//! hash digit, so no letter quits. See `truth/tasks/ls-interactive-mode.typ`.
//!
//! Terminal entry/exit (raw mode, alternate screen, the [`tui::Tui`] RAII
//! guard) lives in the [`tui`] submodule; this module owns the picker's
//! state machine — key classification, the hash-buffer selection, and the
//! draw/refresh loop built on top of it.
//!
//! # Files
//!
//! - `tui.rs` — terminal lifecycle: entering and leaving raw mode.

use std::io::{self, IsTerminal, Write};
use std::path::PathBuf;

use anyhow::{Result, bail};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use crossterm::terminal::{Clear, ClearType};
use crossterm::{cursor, execute};

use anstyle::{Effects, Style};
use mindtape::{ConditionEnv, Config, Readiness, StemHashes, Task};

use crate::cmd::ls::{LsArgs, resolve_and_select};
use crate::render::RenderCtx;
use crate::render::color::{Painter, is_color_active};
use crate::render::output::{OutputFormat, Truncation};
use crate::render::table::render_highlighted;
use crate::{cmd::go, util};

mod tui;
use tui::{Tui, suspend};

/// The live key hints shown in the footer.
const HINTS: &str =
    "slug# select · ⌫ del · Del clear · ⏎ open · ^⏎ open all · Space refresh · Esc quit";

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
    guard_with(
        args,
        io::stdin().is_terminal() && io::stdout().is_terminal(),
    )
}

/// [`guard`] with the terminal answer supplied rather than read from the
/// process, so both arms are reachable in a test.
///
/// Reading `is_terminal()` inside the check made the outcome depend on how the
/// suite was invoked: piped (CI, `just check`, which runs the arm under
/// `parallel`) it refused and the refusal test passed, but run straight from a
/// terminal `cargo test` inherits the tty, the guard allowed, and that same test
/// failed. A test whose verdict flips with the caller's stdout is not a test.
fn guard_with(args: &LsArgs, tty: bool) -> Result<()> {
    if !matches!(args.flags.format, OutputFormat::Table) {
        bail!("--interactive is table-only; drop --format json/csv/path/path0");
    }
    if !tty {
        bail!("--interactive needs a TTY on stdin and stdout (not available when piped)");
    }
    Ok(())
}

/// The loop's mutable state: the resolved view plus the typed hash buffer.
struct State {
    config: Config,
    root: PathBuf,
    tasks: Vec<Task>,
    readiness: Readiness,
    /// The environment a column's conditional visibility resolves against,
    /// rebuilt with the rest of the listing on every refresh.
    env: ConditionEnv,
    /// The stem hashes of every loaded task; the `slug#` column and the
    /// selection both read them.
    hashes: StemHashes,
    /// What `--limit` cut from the listed rows; recomputed on every refresh,
    /// since a flip made in the editor can move a task in or out of the view.
    truncated: Option<Truncation>,
    /// The typed stem hash prefix; empty means nothing is selected.
    buffer: String,
}

impl State {
    fn from_project(project: util::Project, args: &LsArgs) -> Result<Self> {
        let root = project.root.clone();
        let listing = resolve_and_select(args, project)?;
        Ok(Self {
            config: listing.config,
            root,
            tasks: listing.tasks,
            readiness: listing.readiness,
            env: listing.env,
            hashes: listing.hashes,
            truncated: listing.truncated,
            buffer: String::new(),
        })
    }

    /// What the table renderer reads, borrowed from this state.
    fn render_ctx(&self) -> RenderCtx<'_> {
        RenderCtx {
            config: &self.config,
            readiness: &self.readiness,
            env: &self.env,
            hashes: &self.hashes,
        }
    }

    /// Re-open the project, recompute the view, and clear the selection.
    fn refresh(&mut self, args: &LsArgs) -> Result<()> {
        *self = Self::from_project(util::open()?, args)?;
        Ok(())
    }

    /// A per-row mask: row `i` is highlighted when its stem hash starts with
    /// the buffer. An empty buffer highlights nothing.
    fn highlight_mask(&self) -> Vec<bool> {
        (0..self.tasks.len()).map(|i| self.matches(i)).collect()
    }

    /// Index of the first (topmost) highlighted row.
    fn first_highlighted(&self) -> Option<usize> {
        (0..self.tasks.len()).find(|&i| self.matches(i))
    }

    fn matches(&self, index: usize) -> bool {
        !self.buffer.is_empty()
            && self
                .tasks
                .get(index)
                .is_some_and(|task| self.hashes.matches(&task.id, &self.buffer))
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
#[derive(Debug, Clone, PartialEq, Eq)]
enum Action {
    /// Stay; redraw from the current state.
    Continue,
    /// Open the tasks at these indices in the editor (empty means no-op).
    Open(Vec<usize>),
    /// Re-open the project and recompute the view.
    Refresh,
    /// Leave interactive mode.
    Quit,
}

/// The intent a key carries, decoded independently of any IO so it stays
/// testable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Intent {
    /// A stem hash digit, already read the way `mt` reads a typed hash.
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
        KeyCode::Esc => Intent::Quit,
        KeyCode::Char(' ') => Intent::Refresh,
        KeyCode::Char(c) if c.is_ascii_alphanumeric() => Intent::Digit(c.to_ascii_lowercase()),
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
        match apply(state, classify(key)) {
            Action::Continue => {}
            Action::Open(indices) => handle_open(state, tui, &indices, args)?,
            Action::Refresh => state.refresh(args)?,
            Action::Quit => return Ok(()),
        }
    }
}

/// Open the tasks at `indices`, then reload the view if anything opened.
fn handle_open(state: &mut State, tui: &Tui, indices: &[usize], args: &LsArgs) -> Result<()> {
    if open(state, tui, indices)? {
        state.refresh(args)?;
    }
    Ok(())
}

/// Apply an intent: mutate the buffer, or resolve which tasks the open
/// intents target. Pure in the `Tui` — the caller does the actual opening.
fn apply(state: &mut State, intent: Intent) -> Action {
    match intent {
        Intent::Digit(c) => {
            state.buffer.push(c);
            Action::Continue
        }
        Intent::DelLast => {
            state.buffer.pop();
            Action::Continue
        }
        Intent::Clear => {
            state.buffer.clear();
            Action::Continue
        }
        Intent::OpenFirst => Action::Open(collect_first(state)),
        Intent::OpenAll => Action::Open((0..state.tasks.len()).collect()),
        Intent::Refresh => Action::Refresh,
        Intent::Quit => Action::Quit,
        Intent::Ignore => Action::Continue,
    }
}

fn collect_first(state: &State) -> Vec<usize> {
    state.first_highlighted().into_iter().collect()
}

/// Open the tasks at `indices` in the editor, suspending the TUI around the
/// spawn so a terminal editor gets a clean screen. No indices is a no-op
/// that reports `false`.
fn open(state: &State, tui: &Tui, indices: &[usize]) -> Result<bool> {
    let paths = state.paths(indices);
    if paths.is_empty() {
        return Ok(false);
    }
    let template = go::resolve_editor(&state.config)?;
    suspend(tui, || {
        for path in &paths {
            go::spawn_editor(&template, path)?;
        }
        Ok(())
    })?;
    Ok(true)
}

/// Redraw the frame: the highlighted table plus the footer, cleared each time.
fn draw(state: &State) -> Result<()> {
    let mask = state.highlight_mask();
    let table = render_highlighted(&state.tasks, state.render_ctx(), &mask)?;
    let body = format!("{table}\n{}\n", footer(&state.buffer, state.truncated));
    // Raw mode does not translate `\n`; move every line back to column 0.
    let body = body.replace('\n', "\r\n");
    let mut out = io::stdout();
    execute!(out, Clear(ClearType::All), cursor::MoveTo(0, 0))?;
    out.write_all(body.as_bytes())?;
    out.flush()?;
    Ok(())
}

/// The footer: a dim hint line, preceded by the live `slug#` buffer when typing
/// and by the `truncated:` line when `--limit` hid rows. The picker can only
/// open what it lists, so a caller who capped the listing has to be told the
/// task they are looking for may be one of the rows `-n` cut.
fn footer(buffer: &str, truncated: Option<Truncation>) -> String {
    let painter = Painter::new(is_color_active());
    let dim = Style::new().effects(Effects::DIMMED);
    let mut lines: Vec<String> = Vec::new();
    if let Some(cut) = truncated {
        lines.push(painter.paint(&cut.footer(), dim));
    }
    if !buffer.is_empty() {
        lines.push(format!("slug#: {buffer}"));
    }
    lines.push(painter.paint(HINTS, dim));
    lines.join("\n")
}

#[cfg(test)]
mod tests;
