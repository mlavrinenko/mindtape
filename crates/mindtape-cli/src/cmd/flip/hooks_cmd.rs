//! Executing a `[[on.flip]]` `run` action's command.
//!
//! The command runs through [`mindtape::shell`], the executor `[[tasks.waits]]`
//! checkers share, from the project root with the transition in its
//! environment. That module states the trust boundary.

use std::path::Path;

use mindtape::shell::ConfigCommand;

use crate::diag::{Diag, Result};

/// Absolute path of the task being flipped.
pub(crate) const ENV_FILE: &str = "MINDTAPE_TASK_FILE";
/// Identity of the task being flipped (its file stem, unless `id:` overrides).
pub(crate) const ENV_STEM: &str = "MINDTAPE_TASK_STEM";
/// The status being stamped — the `into(...)` of the transition.
pub(crate) const ENV_STATUS: &str = "MINDTAPE_TASK_STATUS";

/// The transition a command runs for, as the command sees it.
pub(crate) struct RunCtx<'a> {
    /// Project root; the command's working directory.
    pub root: &'a Path,
    /// Absolute path of the task file.
    pub path: &'a Path,
    /// The task's identity.
    pub stem: &'a str,
    /// The status being stamped.
    pub target: &'a str,
}

/// What one command run reported.
pub(crate) struct CommandReport {
    /// Whether it exited zero.
    pub success: bool,
    /// Exit code, or `None` when a signal ended it.
    pub code: Option<i32>,
    /// Combined stdout and stderr, in that order.
    pub output: String,
}

impl CommandReport {
    /// How the run ended, for a message: an exit code, or a signal.
    pub fn verdict(&self) -> String {
        self.code.map_or_else(
            || "terminated by signal".to_owned(),
            |c| format!("exit {c}"),
        )
    }
}

/// Run `command`, a shell line or an array, from the project root, with
/// the transition in the environment ([`ENV_FILE`], [`ENV_STEM`],
/// [`ENV_STATUS`]).
///
/// Output is captured rather than inherited so a gate's evidence can be printed
/// under the refusal that rests on it, and so a scripted flip stays clean when
/// the command succeeds.
///
/// # Errors
/// Returns an error when the shell itself cannot be spawned. A command that
/// runs and fails is not an error here — it is a [`CommandReport`] the caller
/// decides about.
pub(crate) fn run_command(command: &ConfigCommand, ctx: &RunCtx<'_>) -> Result<CommandReport> {
    let env = [
        (ENV_FILE, ctx.path.as_os_str()),
        (ENV_STEM, ctx.stem.as_ref()),
        (ENV_STATUS, ctx.target.as_ref()),
    ];
    let output = mindtape::shell::run(command, ctx.root, &env).map_err(|err| {
        Diag::failed("can't run on.flip command")
            .value(command.to_string())
            .io(&err)
    })?;
    Ok(CommandReport {
        success: output.code == Some(0),
        code: output.code,
        output: output.stdout + &output.stderr,
    })
}

#[cfg(test)]
#[path = "hooks_cmd_tests.rs"]
mod tests;
