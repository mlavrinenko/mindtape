//! Executing a `[[on.flip]]` `run` action's command.
//!
//! The trust boundary is a `Justfile`'s or a git hook's: the command comes from
//! the project's own `.mindtape/config.toml`, and it runs through the platform
//! shell ([`SHELL`]) with the caller's full authority, in the project root.
//! Nothing here is sandboxed, and nothing pretends to be — a config you would
//! not run is a config you should not have checked out.
//!
//! Only `mt flip` reaches this module. No read command does.

use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result};

/// The shell a `run` command goes through, as `(program, command-flag)`.
///
/// A `run` command is a shell line, not an argv, so there has to be a shell —
/// and there is no portable one. `.mindtape/config.toml` is a checked-in file
/// whose commands are written for the platform the project builds on, exactly
/// like the `Justfile` recipes or git hooks they stand in for; a project that
/// wants both writes a command both shells accept, or invokes a script.
#[cfg(not(windows))]
pub(crate) const SHELL: (&str, &str) = ("sh", "-c");
#[cfg(windows)]
pub(crate) const SHELL: (&str, &str) = ("cmd", "/C");

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

/// Run `command` through the platform [`SHELL`], from the project root, with
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
pub(crate) fn run_command(command: &str, ctx: &RunCtx<'_>) -> Result<CommandReport> {
    let (shell, flag) = SHELL;
    let output = Command::new(shell)
        .arg(flag)
        .arg(command)
        .current_dir(ctx.root)
        .env(ENV_FILE, ctx.path)
        .env(ENV_STEM, ctx.stem)
        .env(ENV_STATUS, ctx.target)
        .output()
        .with_context(|| format!("run on.flip command {command:?}"))?;
    let mut text = String::from_utf8_lossy(&output.stdout).into_owned();
    text.push_str(&String::from_utf8_lossy(&output.stderr));
    Ok(CommandReport {
        success: output.status.success(),
        code: output.status.code(),
        output: text,
    })
}

#[cfg(test)]
#[path = "flip_hooks_cmd_tests.rs"]
mod tests;
