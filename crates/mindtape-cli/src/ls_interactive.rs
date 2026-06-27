//! `mindtape ls --interactive` — a terminal picker over the `ls` list.
//!
//! Renders the same table `ls` prints, then holds the terminal in a raw-mode key
//! loop on an alternate screen. Typing the `#`-column ordinal highlights a task
//! (reverse-video); Enter opens the first highlighted task in the configured
//! editor (`go`), Ctrl+Enter opens every listed task, Space re-evaluates the
//! project, and `q` / Ctrl+C leaves. See `truth/tasks/ls-interactive-mode.typ`.

use std::io::{self, IsTerminal, Write};
use std::path::PathBuf;

use anyhow::{Result, bail};
use crossterm::event::{
    self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, KeyboardEnhancementFlags,
    PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags,
};
use crossterm::terminal::{
    Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode,
    enable_raw_mode, supports_keyboard_enhancement,
};
use crossterm::{cursor, execute};

use anstyle::{Effects, Style};
use mindtape::{Config, Readiness, Task};

use crate::ls::{LsArgs, select_view};
use crate::output::OutputFormat;
use crate::render::color::{Painter, is_color_active};
use crate::render::table::render_highlighted;
use crate::{go, util};

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

// --- Terminal lifecycle ------------------------------------------------------

/// RAII guard for the alternate-screen + raw-mode terminal state. Restores the
/// terminal on drop; [`suspend`] temporarily restores it around an editor spawn.
struct Tui {
    enhanced: bool,
}

impl Tui {
    fn enter() -> Result<Self> {
        let enhanced = matches!(supports_keyboard_enhancement(), Ok(true));
        enter_screen(enhanced)?;
        Ok(Self { enhanced })
    }
}

impl Drop for Tui {
    fn drop(&mut self) {
        let _ = leave_screen(self.enhanced);
    }
}

fn enter_screen(enhanced: bool) -> Result<()> {
    enable_raw_mode()?;
    let mut out = io::stdout();
    execute!(out, EnterAlternateScreen, cursor::Hide)?;
    if enhanced {
        execute!(
            out,
            PushKeyboardEnhancementFlags(KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES)
        )?;
    }
    Ok(())
}

fn leave_screen(enhanced: bool) -> Result<()> {
    let mut out = io::stdout();
    if enhanced {
        let _ = execute!(out, PopKeyboardEnhancementFlags);
    }
    execute!(out, cursor::Show, LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}

/// Leave the TUI, run `f` on the normal screen (the editor), then re-enter.
fn suspend<F: FnOnce() -> Result<()>>(tui: &Tui, f: F) -> Result<()> {
    leave_screen(tui.enhanced)?;
    let result = f();
    enter_screen(tui.enhanced)?;
    result
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
    use mindtape::test_support::bare_task;
    use mindtape::{Config, Readiness, Task};

    use super::{Intent, State, classify, footer};

    fn key(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
        KeyEvent {
            code,
            modifiers,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }
    }

    fn state(n: usize, buffer: &str) -> State {
        let tasks: Vec<Task> = (0..n).map(|i| bare_task(&format!("t{i}"))).collect();
        State {
            config: Config::default(),
            root: ".".into(),
            tasks,
            readiness: Readiness::empty(),
            buffer: buffer.to_owned(),
        }
    }

    #[test]
    fn classify_digits_and_verbs() {
        assert_eq!(
            classify(key(KeyCode::Char('3'), KeyModifiers::NONE)),
            Intent::Digit('3')
        );
        assert_eq!(
            classify(key(KeyCode::Char('q'), KeyModifiers::NONE)),
            Intent::Quit
        );
        assert_eq!(
            classify(key(KeyCode::Char(' '), KeyModifiers::NONE)),
            Intent::Refresh
        );
        assert_eq!(
            classify(key(KeyCode::Backspace, KeyModifiers::NONE)),
            Intent::DelLast
        );
        assert_eq!(
            classify(key(KeyCode::Delete, KeyModifiers::NONE)),
            Intent::Clear
        );
        assert_eq!(
            classify(key(KeyCode::Enter, KeyModifiers::NONE)),
            Intent::OpenFirst
        );
    }

    #[test]
    fn classify_ctrl_modifiers() {
        assert_eq!(
            classify(key(KeyCode::Char('c'), KeyModifiers::CONTROL)),
            Intent::Quit
        );
        assert_eq!(
            classify(key(KeyCode::Enter, KeyModifiers::CONTROL)),
            Intent::OpenAll
        );
        assert_eq!(
            classify(key(KeyCode::Backspace, KeyModifiers::CONTROL)),
            Intent::Clear
        );
    }

    #[test]
    fn classify_unknown_is_ignored() {
        assert_eq!(
            classify(key(KeyCode::Char('x'), KeyModifiers::NONE)),
            Intent::Ignore
        );
        assert_eq!(
            classify(key(KeyCode::Up, KeyModifiers::NONE)),
            Intent::Ignore
        );
    }

    #[test]
    fn empty_buffer_highlights_nothing() {
        let state = state(5, "");
        assert_eq!(state.highlight_mask(), vec![false; 5]);
        assert_eq!(state.first_highlighted(), None);
    }

    #[test]
    fn prefix_selects_by_ordinal() {
        // 12 tasks, buffer "1" → ordinals 1, 10, 11, 12 (indices 0, 9, 10, 11).
        let state = state(12, "1");
        let mask = state.highlight_mask();
        let on: Vec<usize> = mask
            .iter()
            .enumerate()
            .filter_map(|(i, &on)| on.then_some(i))
            .collect();
        assert_eq!(on, vec![0, 9, 10, 11]);
        // First highlighted is the topmost (ordinal 1, index 0).
        assert_eq!(state.first_highlighted(), Some(0));
    }

    #[test]
    fn two_digit_prefix_narrows() {
        let state = state(12, "12");
        assert_eq!(state.first_highlighted(), Some(11));
        assert_eq!(state.highlight_mask().iter().filter(|&&on| on).count(), 1);
    }

    #[test]
    fn nonmatching_buffer_highlights_nothing() {
        let state = state(5, "9");
        assert_eq!(state.first_highlighted(), None);
    }

    #[test]
    fn footer_shows_buffer_when_typing() {
        assert!(footer("12").contains("#12"));
        // Empty buffer drops the status line, keeping only the hints.
        assert!(!footer("").contains('#'));
    }
}
