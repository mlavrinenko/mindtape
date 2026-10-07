//! Terminal lifecycle for the interactive picker: entering/leaving the
//! alternate screen and raw mode, and the RAII guard that restores the
//! terminal state on drop even when the event loop errors out.

use std::io;

use anyhow::Result;
use crossterm::event::{
    KeyboardEnhancementFlags, PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags,
};
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
    supports_keyboard_enhancement,
};
use crossterm::{cursor, execute};

/// RAII guard for the alternate-screen + raw-mode terminal state. Restores the
/// terminal on drop; [`suspend`] temporarily restores it around an editor spawn.
pub(super) struct Tui {
    enhanced: bool,
}

impl Tui {
    pub(super) fn enter() -> Result<Self> {
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
pub(super) fn suspend<F: FnOnce() -> Result<()>>(tui: &Tui, f: F) -> Result<()> {
    leave_screen(tui.enhanced)?;
    let result = f();
    enter_screen(tui.enhanced)?;
    result
}

// No in-process tests here on purpose: every function in this module drives the
// terminal, so an in-process test either hijacks the developer's own screen or
// asserts nothing on a machine without a TTY. They are pinned end-to-end under
// a pseudo-terminal instead — see `tests/interactive_e2e.rs`
// (`quitting_restores_the_terminal`, `opening_a_task_suspends_the_tui_and_runs_the_editor`).
