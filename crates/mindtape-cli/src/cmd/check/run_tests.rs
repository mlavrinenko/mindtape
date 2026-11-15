//! Unit tests for the `[commits] run` command: each way the configured
//! command can end, and the lines it prints.

#![allow(clippy::unwrap_used)]

use mindtape::shell::{ConfigCommand, ShellOutput};

use super::{check, hits};
use crate::test_util::make_project;

/// A project whose `[commits] run` is `run`.
fn project(run: Option<ConfigCommand>) -> (tempfile::TempDir, crate::util::Project) {
    let mut config = mindtape::parse_config("").unwrap();
    config.commits.run = run;
    make_project(&[], config)
}

/// How many problems `run` reports over a message that passes every other
/// check.
fn problems(run: Option<ConfigCommand>) -> usize {
    let (_dir, project) = project(run);
    check(&project, "feat: x\n", None).unwrap()
}

#[cfg(unix)]
fn sh(script: &str) -> ConfigCommand {
    ConfigCommand::Argv(vec!["sh".to_owned(), "-c".to_owned(), script.to_owned()])
}

#[test]
fn no_command_passes() {
    assert_eq!(problems(None), 0);
}

#[test]
fn a_blank_command_passes() {
    assert_eq!(problems(Some(ConfigCommand::Line("  ".to_owned()))), 0);
}

#[cfg(unix)]
#[test]
fn a_command_is_read_by_its_exit_code_and_output() {
    assert_eq!(problems(Some(sh("cat >/dev/null"))), 0);
    assert_eq!(problems(Some(sh("printf 'one\\n\\ntwo\\n'; exit 1"))), 2);
    assert_eq!(problems(Some(sh("exit 3"))), 1);
}

#[cfg(unix)]
#[test]
fn a_command_that_cannot_start_is_an_error() {
    let (_dir, project) = project(Some(ConfigCommand::Argv(vec![
        "/no/such/program".to_owned(),
    ])));
    assert!(check(&project, "feat: x\n", None).is_err());
}

#[test]
fn hits_reads_stdout_then_stderr_and_drops_blank_lines() {
    let output = ShellOutput {
        code: Some(1),
        stdout: "one\n\ntwo \n".to_owned(),
        stderr: "\nthree\n".to_owned(),
    };
    assert_eq!(hits(&output), ["one", "two", "three"]);
}
