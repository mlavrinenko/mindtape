//! How `mt` reports a problem. A line for stderr is built here as a [`Diag`]
//! and printed by [`render`], so a message can't leave the format. The modules
//! on the one list in `allowlist.rs` still print some lines of their own until
//! their fan-out task converts them.
//!
//! ```text
//! error: <what failed>: '<value>'[: <why>]
//! hint: <the next step>
//! note: <context, no action>
//! warning: <same shape as error; nothing was refused>
//! ```
//!
//! - The text is a `&'static str`, so a value can't be formatted into it. A
//!   value the user typed goes through [`Diag::value`] or [`Diag::context`],
//!   which single-quote it the way a shell takes it back: `'it'\''s'`.
//! - A field or rule name is bare: [`Diag::rule`].
//! - A problem tied to a file starts with its place: [`Diag::at`] renders
//!   `error: tasks/x.typ:2: rules.one-wip: <message>`.
//! - A cause chain joins into the one line with `: `. Rust's `Error:` and
//!   `Caused by:` never print, because `main` returns an `ExitCode`.
//! - At most two hints, each a [`Hint`]: a sentence, a command to run, a
//!   did-you-mean, or the names to choose from (`use one of: a, b`).
//!   Typst's own hints on an error ([`Diag::typst_hints`]) come on top of that,
//!   verbatim.
//! - Words that are not ours, such as Typst's message, are not a `&'static str`,
//!   so [`Diag::verbatim`] takes them whole, after the place:
//!   `error: tasks/x.typ:2:3: unexpected argument: titel`.
//! - A read or write that failed is [`Diag::cannot_read`] or
//!   [`Diag::cannot_write`]: `error: can't read: 'nope.typ': no such file or
//!   directory`. [`Diag::io`] adds the OS reason to any other message.
//! - The [`Kind`] decides the exit code: 2 when the command line was wrong
//!   ([`Diag::usage`]), 1 when the project refused the work or it failed
//!   ([`Diag::refused`], [`Diag::failed`]). No command picks a number.
//! - For a write command, `error:` means nothing was written for that task.
//!   A batch of several targets ends with one [`Diag::batch`] summary; a
//!   single target gets none.
//! - Colour goes on the prefix only, when stderr is a terminal or `--color`
//!   says so.
//!
//! # Files
//!
//! - `allowlist.rs` — the modules that still bypass the renderer, and the
//!   test that holds the source to the list.
//! - `hint.rs` — [`Hint`], [`Note`] and [`Command`], the lines under an error.
//! - `render.rs` — the renderer and the one writer to stderr.
//! - `summary.rs`, `summary_tests.rs` — the closing count of a command that found several
//!   problems, and the exit status of a command it ran.
//! - `io.rs` — the failed read or write and the OS's reason for it.
//! - `prompt.rs` — the confirmation question, the one line stderr gets that
//!   isn't a `Diag`: a prompt is not a diagnostic, so it has no prefix.
//! - `from_lib.rs` — library errors into a `Diag`, one exhaustive match each.
//! - `from_lib_tests.rs` — unit tests for the condition and cleanup conversions.
//! - `tests.rs`, `tests_fill.rs` — unit tests.

#[cfg(test)]
mod allowlist;
mod from_lib;
mod hint;
mod io;
mod prompt;
mod render;
mod summary;

use std::fmt;
use std::path::Path;

pub(crate) use from_lib::filter;
pub(crate) use hint::{Command, Hint, Note};
pub(crate) use prompt::confirm;
pub(crate) use render::{emit, emit_note, emit_text, line, report, stderr_is_terminal};
pub(crate) use summary::dedupe_notes;

/// What kind of problem a [`Diag`] reports, which decides the exit code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Kind {
    /// The command line was wrong: an unknown command, status, view or field,
    /// a filter that doesn't parse, missing targets. Exit 2, as clap's own.
    Usage,
    /// The project's rules refused the work. Exit 1.
    Refused,
    /// The work failed: a missing task, I/O, git, a broken config. Exit 1.
    Failed,
    /// Nothing was refused. Printed with [`emit`]; returned as an error it
    /// exits 1.
    Warning,
}

impl Kind {
    /// The process exit code for this kind.
    pub(crate) fn code(self) -> u8 {
        match self {
            Self::Usage => 2,
            Self::Refused | Self::Failed | Self::Warning => 1,
        }
    }

    /// The line prefix, without its colon.
    fn label(self) -> &'static str {
        match self {
            Self::Warning => "warning",
            Self::Usage | Self::Refused | Self::Failed => "error",
        }
    }
}

/// A batch summary's counts: `flipped 1 of 3 tasks; 2 failed`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Tally {
    done: usize,
    total: usize,
}

/// One problem, built with the methods below and printed by [`render`]. Boxed,
/// so a `Result<_, Diag>` stays a pointer wide.
#[derive(Debug, Clone, PartialEq, Eq)]
#[must_use]
pub(crate) struct Diag(Box<Parts>);

/// What a [`Diag`] holds, in the order [`render`] prints it.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Parts {
    kind: Kind,
    /// What an unconverted `anyhow` chain said above this message. Goes away
    /// with the bridge in `from_lib.rs`.
    legacy: Vec<String>,
    /// `path:line[:col]`, or a place with no file such as `pack <name>:3`.
    place: Option<String>,
    rule: Option<String>,
    what: &'static str,
    tally: Option<Tally>,
    value: Option<String>,
    context: Option<(&'static str, String)>,
    reason: Option<String>,
    hints: Vec<Hint>,
    notes: Vec<Note>,
}

impl Diag {
    fn new(kind: Kind, what: &'static str) -> Self {
        Self(Box::new(Parts {
            kind,
            legacy: Vec::new(),
            place: None,
            rule: None,
            what,
            tally: None,
            value: None,
            context: None,
            reason: None,
            hints: Vec::new(),
            notes: Vec::new(),
        }))
    }

    /// The command line was wrong. Exits 2.
    pub(crate) fn usage(what: &'static str) -> Self {
        Self::new(Kind::Usage, what)
    }

    /// The project's rules refused the work. Exits 1.
    pub(crate) fn refused(what: &'static str) -> Self {
        Self::new(Kind::Refused, what)
    }

    /// The work failed. Exits 1.
    pub(crate) fn failed(what: &'static str) -> Self {
        Self::new(Kind::Failed, what)
    }

    /// Something worth saying that refused nothing.
    pub(crate) fn warning(what: &'static str) -> Self {
        Self::new(Kind::Warning, what)
    }

    /// A problem of `kind` with no text of ours, whose whole message is
    /// `message` as another program worded it (Typst, a library), kept as is.
    /// Give it a place with [`Self::at`].
    pub(crate) fn verbatim(kind: Kind, message: impl Into<String>) -> Self {
        Self::new(kind, "").reason(message)
    }

    /// A problem of `kind` with `what` as its text.
    pub(crate) fn of(kind: Kind, what: &'static str) -> Self {
        Self::new(kind, what)
    }

    /// Typst's hints on the error, each a `hint:` line in Typst's words. They
    /// are not held to the two-hint limit, which is for hints we write.
    pub(crate) fn typst_hints(mut self, hints: impl IntoIterator<Item = String>) -> Self {
        self.0.hints.extend(hints.into_iter().map(Hint::typst));
        self
    }

    /// The last line of a batch with a failure: `verb` `done` of `total`
    /// tasks, and how many failed when some landed.
    pub(crate) fn batch(verb: &'static str, done: usize, total: usize) -> Self {
        let mut diag = Self::new(Kind::Failed, verb);
        diag.0.tally = Some(Tally { done, total });
        diag
    }

    /// A message from code not yet converted to this builder, kept word for
    /// word. Transitional: the fan-out tasks remove every caller.
    pub(crate) fn legacy(message: impl fmt::Display) -> Self {
        let mut diag = Self::new(Kind::Failed, "");
        diag.0.legacy = vec![message.to_string()];
        diag
    }

    /// The value that failed, as the user typed it. Rendered single-quoted.
    pub(crate) fn value(mut self, value: impl Into<String>) -> Self {
        self.0.value = Some(value.into());
        self
    }

    /// Where the value sits: `.context("in filter", expr)` renders
    /// `in filter '<expr>'` after the value.
    pub(crate) fn context(mut self, label: &'static str, value: impl Into<String>) -> Self {
        self.0.context = Some((label, value.into()));
        self
    }

    /// The file place the problem is tied to, as the user should read it.
    pub(crate) fn at(mut self, path: &Path, line: usize, col: Option<usize>) -> Self {
        self.0.place = Some(match col {
            Some(col) => format!("{}:{line}:{col}", path.display()),
            None => format!("{}:{line}", path.display()),
        });
        self
    }

    /// The file the problem is tied to, when no line of it is known.
    pub(crate) fn at_file(mut self, path: &Path) -> Self {
        self.0.place = Some(path.display().to_string());
        self
    }

    /// The field or rule the problem is about, rendered bare after the place.
    pub(crate) fn rule(mut self, name: impl Into<String>) -> Self {
        self.0.rule = Some(name.into());
        self
    }

    /// Why it failed: an error and its sources, joined with `: `.
    pub(crate) fn cause(self, err: &(dyn std::error::Error + 'static)) -> Self {
        let mut parts = vec![err.to_string()];
        let mut source = err.source();
        while let Some(next) = source {
            parts.push(next.to_string());
            source = next.source();
        }
        self.reason(parts.join(": "))
    }

    /// Why it failed, as a library or the OS words it. Private to this module,
    /// so a command can't pass free text: it passes an error to [`Self::cause`].
    fn reason(mut self, reason: impl Into<String>) -> Self {
        self.0.reason = Some(reason.into());
        self
    }

    /// The next step. At most two per message.
    ///
    /// # Panics
    /// In a debug build, on a third hint.
    pub(crate) fn hint(mut self, hint: Hint) -> Self {
        debug_assert!(self.0.hints.len() < 2, "a message takes at most two hints");
        self.0.hints.push(hint);
        self
    }

    /// Whether [`Self::hint`] can take another hint, for a caller that adds
    /// to a message it didn't build.
    pub(crate) fn has_hint_room(&self) -> bool {
        self.0.hints.len() < 2
    }

    /// Context the reader may need, with no action.
    pub(crate) fn note(mut self, note: Note) -> Self {
        self.0.notes.push(note);
        self
    }

    /// The kind, which decides the exit code.
    pub(crate) fn kind(&self) -> Kind {
        self.0.kind
    }
}

impl fmt::Display for Diag {
    /// The message line without its prefix, hints or notes, for code that
    /// still formats an error into a string of its own.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&render::message(self))
    }
}

impl std::error::Error for Diag {}

/// Every line of `diag` as stderr gets it, without colour, for a test that
/// pins the whole message.
#[cfg(test)]
pub(crate) fn plain(diag: &Diag) -> String {
    render::render(diag, false)
}

/// An `anyhow` error as stderr gets it, without colour.
#[cfg(test)]
pub(crate) fn plain_anyhow(err: anyhow::Error) -> String {
    plain(&Diag::from(err))
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod from_lib_tests;
#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests;

#[cfg(test)]
mod tests_fill;
