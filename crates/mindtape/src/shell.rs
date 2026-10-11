//! Running a config command through the platform shell: the one executor
//! behind an `[[on.flip]]` `run` action and a `[[tasks.waits]]` checker.
//!
//! The trust boundary is a `Justfile`'s or a git hook's: the command comes from
//! the project's own `.mindtape/config.toml`, and it runs through [`SHELL`]
//! with the caller's full authority. Nothing here is sandboxed, and nothing
//! pretends to be. A config you would not run is a config you should not have
//! checked out.

use std::ffi::OsStr;
use std::path::Path;
use std::process::Command;

/// The shell a config command goes through, as `(program, command-flag)`.
///
/// A config command is a shell line, not an argv, so there has to be a shell,
/// and there is no portable one. `.mindtape/config.toml` is a checked-in file
/// whose commands are written for the platform the project builds on, exactly
/// like the `Justfile` recipes or git hooks they stand in for. A project that
/// wants both writes a command both shells accept, or invokes a script.
#[cfg(not(windows))]
pub const SHELL: (&str, &str) = ("sh", "-c");
/// The shell a config command goes through, as `(program, command-flag)`.
#[cfg(windows)]
pub const SHELL: (&str, &str) = ("cmd", "/C");

/// What one command run left behind.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShellOutput {
    /// Exit code, or `None` when a signal ended it.
    pub code: Option<i32>,
    /// Captured standard output, lossily decoded.
    pub stdout: String,
    /// Captured standard error, lossily decoded.
    pub stderr: String,
}

/// Run `command` through [`SHELL`] from `dir`, with `env` added to the
/// caller's environment, capturing both streams.
///
/// # Errors
/// Returns the spawn error when the shell itself cannot start. A command that
/// runs and fails is not an error here: its [`ShellOutput`] says how it ended.
pub fn run(command: &str, dir: &Path, env: &[(&str, &OsStr)]) -> std::io::Result<ShellOutput> {
    let (shell, flag) = SHELL;
    let output = Command::new(shell)
        .arg(flag)
        .arg(command)
        .current_dir(dir)
        .envs(env.iter().copied())
        .output()?;
    Ok(ShellOutput {
        code: output.status.code(),
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    })
}
