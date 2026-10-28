//! Running a config command: the one executor behind an `[[on.flip]]` `run`
//! action and a `[[tasks.waits]]` checker.
//!
//! A `run` is a [`ConfigCommand`]: a string goes through the platform
//! [`SHELL`], and an array of strings is a program and its arguments, spawned
//! with no shell at all.
//!
//! The trust boundary is a `Justfile`'s or a git hook's: the command comes from
//! the project's own `.mindtape/config.toml`, and it runs with the caller's full
//! authority. Nothing here is sandboxed, and nothing pretends to be. A config
//! you would not run is a config you should not have checked out.

use std::ffi::OsStr;
use std::fmt;
use std::path::Path;
use std::process::Command;

use serde::Deserialize;

/// The shell a string command goes through, as `(program, command-flag)`.
///
/// A string command is a shell line, so there has to be a shell, and there is
/// no portable one. A project used on more than one platform writes its
/// command as an array instead ([`ConfigCommand::Argv`]), which needs none.
#[cfg(not(windows))]
pub const SHELL: (&str, &str) = ("sh", "-c");
/// The shell a string command goes through, as `(program, command-flag)`.
#[cfg(windows)]
pub const SHELL: (&str, &str) = ("cmd", "/C");

/// A command a config asks `mt` to run: a `run` key's value.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, schemars::JsonSchema)]
#[serde(untagged)]
pub enum ConfigCommand {
    /// A command line for the platform shell: `sh -c`, or `cmd /C` on Windows.
    Line(String),
    /// A program and its arguments, run directly with no shell, the same on
    /// every platform. The program is found on `PATH`.
    Argv(Vec<String>),
}

impl Default for ConfigCommand {
    fn default() -> Self {
        Self::Line(String::new())
    }
}

impl ConfigCommand {
    /// The command with `replace` applied to the line, or to each argument
    /// alone, so a filled value never splits or joins an argument.
    #[must_use]
    pub fn map(&self, replace: impl Fn(&str) -> String) -> Self {
        match self {
            Self::Line(line) => Self::Line(replace(line)),
            Self::Argv(argv) => Self::Argv(argv.iter().map(|arg| replace(arg)).collect()),
        }
    }

    /// Whether there is nothing to run: a blank line, or an array whose
    /// program is missing or blank.
    #[must_use]
    pub fn is_blank(&self) -> bool {
        match self {
            Self::Line(line) => line.trim().is_empty(),
            Self::Argv(argv) => argv.first().is_none_or(|program| program.trim().is_empty()),
        }
    }
}

/// A line shows as written. An array shows in TOML's array syntax, each
/// argument a quoted string, as `["cargo", "test"]`.
impl fmt::Display for ConfigCommand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Line(line) => f.write_str(line),
            Self::Argv(argv) => {
                let quoted: Vec<String> = argv
                    .iter()
                    .map(|arg| serde_json::Value::from(arg.as_str()).to_string())
                    .collect();
                write!(f, "[{}]", quoted.join(", "))
            }
        }
    }
}

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

/// Run `command` from `dir`, with `env` added to the caller's environment,
/// capturing both streams. A line goes through [`SHELL`], and an array runs
/// its program directly.
///
/// # Errors
/// Returns the spawn error when the shell or the program cannot start, and
/// [`std::io::ErrorKind::InvalidInput`] for an array with no program. A
/// command that runs and fails is not an error here: its [`ShellOutput`] says
/// how it ended.
pub fn run(
    command: &ConfigCommand,
    dir: &Path,
    env: &[(&str, &OsStr)],
) -> std::io::Result<ShellOutput> {
    let mut process = match command {
        ConfigCommand::Line(line) => {
            let (shell, flag) = SHELL;
            let mut process = Command::new(shell);
            process.arg(flag).arg(line);
            process
        }
        ConfigCommand::Argv(argv) => {
            let Some((program, rest)) = argv.split_first() else {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "the command array names no program",
                ));
            };
            let mut process = Command::new(program);
            process.args(rest);
            process
        }
    };
    let output = process
        .current_dir(dir)
        .envs(env.iter().copied())
        .output()?;
    Ok(ShellOutput {
        code: output.status.code(),
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    })
}

#[cfg(test)]
#[path = "shell_tests.rs"]
mod tests;
