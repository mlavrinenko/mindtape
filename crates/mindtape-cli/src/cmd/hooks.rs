//! The presentation half of a hook action, shared by `mt flip`'s `[[on.flip]]`
//! runner and `mt rm`'s `[[on.rm]]` runner.
//!
//! A `show` prints text and a `list` prints a table. Both are presentation, so
//! both fall silent when the caller says so. A `run` is the caller's to
//! execute: [`present`] hands its fields back, and each runner judges the exit
//! code its own way, since `mt flip` has `--force` and `mt rm` does not.

use std::io::Write;

use mindtape::Readiness;
use mindtape::config::hooks::{ActionKind, FlipAction};
use mindtape::shell::ConfigCommand;

use crate::diag::{Diag, Kind, Result};
use crate::util::Project;

/// A `run` action's fields, handed back by [`present`] for the caller to
/// execute.
#[derive(Clone, Copy)]
pub(crate) struct RunAction<'a> {
    /// The command, or `None` for a run-less gate. `kind()` only ever pairs
    /// that with `gate: true`.
    pub(crate) command: Option<&'a ConfigCommand>,
    /// Whether a non-zero exit refuses the write.
    pub(crate) gate: bool,
    /// Explanation printed on a non-zero exit or an unconditional refusal.
    pub(crate) message: Option<&'a str>,
}

/// The project, the readiness a `list` filter reads, and whether presentation is
/// on at all.
pub(crate) struct Presenter<'a> {
    project: &'a Project,
    readiness: &'a Readiness,
    show: bool,
}

impl<'a> Presenter<'a> {
    /// Build the presenter for one command invocation.
    pub(crate) fn new(project: &'a Project, readiness: &'a Readiness, show: bool) -> Self {
        Self {
            project,
            readiness,
            show,
        }
    }
}

/// Run the presentation-only actions (`show` and `list`) and hand a `run` back
/// to the caller. `section` names the hook array in the error an action with no
/// single kind produces.
///
/// # Errors
/// Returns an error when the action doesn't declare a single kind, or the
/// `list` filter does not parse or evaluate.
pub(crate) fn present<'a>(
    action: &'a FlipAction,
    section: &str,
    presenter: &Presenter<'_>,
    out: &mut dyn Write,
) -> Result<Option<RunAction<'a>>> {
    match action
        .kind()
        .map_err(|message| Diag::verbatim(Kind::Failed, format!("{section} action: {message}")))?
    {
        ActionKind::Show(text) => {
            if presenter.show {
                writeln!(out, "{text}")?;
            }
            Ok(None)
        }
        ActionKind::List { filter, title } => {
            if presenter.show {
                crate::cmd::ls::print_matching(
                    presenter.project,
                    presenter.readiness,
                    filter,
                    title,
                    out,
                )?;
            }
            Ok(None)
        }
        ActionKind::Run {
            command,
            gate,
            message,
        } => Ok(Some(RunAction {
            command,
            gate,
            message,
        })),
    }
}
