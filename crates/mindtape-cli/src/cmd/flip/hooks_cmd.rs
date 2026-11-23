//! Executing a `[[on.flip]]` `run` action's command.
//!
//! The command runs through [`mindtape::shell`], the executor `[[tasks.waits]]`
//! checkers share, from the project root with the transition in its
//! environment. That module states the trust boundary. The environment also
//! carries the flips in progress ([`ENV_CHAIN`]), so a nested `mt flip` can
//! refuse one that would loop.
//!
//! A pre-write command runs for one transition ([`RunCtx`]). An after-the-write
//! command runs once for the batch that landed ([`BatchCtx`]), so it sees the
//! status and the landed tasks instead of one task.

use std::path::Path;

use mindtape::shell::ConfigCommand;

use crate::cmd::flip::batch::Landed;
use crate::cmd::flip::chain::{Chain, ENV_CHAIN};
use crate::diag::{Diag, Result};

/// Absolute path of the task the flip targets.
pub(crate) const ENV_FILE: &str = "MINDTAPE_TASK_FILE";
/// Identity of the task the flip targets (its file stem, unless `id:` overrides).
pub(crate) const ENV_STEM: &str = "MINDTAPE_TASK_STEM";
/// The status the flip stamps, taken from the transition's `into(...)`.
pub(crate) const ENV_STATUS: &str = "MINDTAPE_TASK_STATUS";
/// The tasks an after-the-write command runs for, one id per line, in the order
/// they landed. A pre-write command sets none.
pub(crate) const ENV_TASKS: &str = "MINDTAPE_FLIP_TASKS";

/// The transition a command runs for, as the command sees it.
pub(crate) struct RunCtx<'a> {
    /// Project root, and the command's working directory.
    pub root: &'a Path,
    /// Absolute path of the task file.
    pub path: &'a Path,
    /// The task's identity.
    pub stem: &'a str,
    /// The status the flip stamps.
    pub target: &'a str,
    /// The flips in progress above this one.
    pub chain: &'a Chain,
}

/// The batch an after-the-write command runs for, as the command sees it.
pub(crate) struct BatchCtx<'a> {
    /// Project root, and the command's working directory.
    pub root: &'a Path,
    /// The status the batch stamps.
    pub target: &'a str,
    /// The flips in progress above this one.
    pub chain: &'a Chain,
    /// The tasks the batch wrote.
    pub landed: &'a Landed,
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
    /// The run's outcome for a message: an exit code, or a signal.
    pub fn verdict(&self) -> String {
        self.code.map_or_else(
            || "terminated by signal".to_owned(),
            |c| format!("exit {c}"),
        )
    }
}

/// Run `command`, a shell line or an array, from the project root, with
/// the transition in the environment ([`ENV_FILE`], [`ENV_STEM`],
/// [`ENV_STATUS`]) and the chain with this flip added ([`ENV_CHAIN`]).
///
/// The run captures output rather than inheriting it. A gate's evidence prints
/// under the refusal that rests on it, and a scripted flip stays clean when the
/// command succeeds.
///
/// # Errors
/// Returns an error when the shell itself fails to spawn. A command that
/// runs and fails is not an error here: it is a [`CommandReport`] the caller
/// decides about.
pub(crate) fn run_command(command: &ConfigCommand, ctx: &RunCtx<'_>) -> Result<CommandReport> {
    let chain = ctx.chain.passed_down(ctx.path, ctx.stem, ctx.target);
    let env = [
        (ENV_FILE, ctx.path.as_os_str()),
        (ENV_STEM, ctx.stem.as_ref()),
        (ENV_STATUS, ctx.target.as_ref()),
        (ENV_CHAIN, chain.as_os_str()),
    ];
    run_with(command, ctx.root, &env)
}

/// Run `command` once for the batch that landed, from the project root, with
/// the status ([`ENV_STATUS`]), the landed tasks ([`ENV_TASKS`]) and the chain
/// with every landed flip added ([`ENV_CHAIN`]) in the environment. It sees the
/// batch, so [`ENV_FILE`] and [`ENV_STEM`] are unset.
///
/// # Errors
/// Returns an error when the shell itself fails to spawn. A command that runs
/// and fails is not an error here: it is a [`CommandReport`] the caller decides
/// about.
pub(crate) fn run_batch_command(
    command: &ConfigCommand,
    ctx: &BatchCtx<'_>,
) -> Result<CommandReport> {
    let chain = ctx.chain.passed_down_batch(ctx.landed.tasks(), ctx.target);
    let tasks = ctx.landed.ids().join("\n");
    let env = [
        (ENV_STATUS, ctx.target.as_ref()),
        (ENV_TASKS, tasks.as_ref()),
        (ENV_CHAIN, chain.as_os_str()),
    ];
    run_with(command, ctx.root, &env)
}

/// Run `command` from `root` with `env`, mapping a spawn failure to the
/// command's `Diag`.
fn run_with(
    command: &ConfigCommand,
    root: &Path,
    env: &[(&str, &std::ffi::OsStr)],
) -> Result<CommandReport> {
    let output = mindtape::shell::run(command, root, env).map_err(|err| {
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
