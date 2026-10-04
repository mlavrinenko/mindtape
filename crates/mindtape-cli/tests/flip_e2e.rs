//! End-to-end tests for the `mt <status>` shortcut verbs, driven through the
//! `mt` binary.
#![allow(clippy::unwrap_used, clippy::disallowed_methods)]

mod support;

const LEGEND: &str = "[tasks]\nroots = [\"tasks\"]\n\
     [[tasks.status]]\nvalue = \"wip\"\n\
     [[tasks.status]]\nvalue = \"done\"\nterminal = true\n";

/// `mt done --help` is a help request, not a failure: it exits 0, prints to
/// stdout, names the verb and shows none of the internal parser's doc comment.
#[test]
fn shortcut_verb_help_prints_help_and_exits_zero() {
    let dir = support::scaffold_project(LEGEND);
    let output = support::mt(dir.path())
        .args(["done", "--help"])
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        output.status.success(),
        "stdout: {stdout}\nstderr: {stderr}"
    );
    assert!(stderr.is_empty(), "help goes to stdout: {stderr}");
    assert!(stdout.contains("Usage: mt done"), "{stdout}");
    assert!(!stdout.contains("Error:"), "{stdout}");
    assert!(!stdout.contains("Parser wrapper"), "{stdout}");
    assert!(!stdout.contains("mindtape-cli"), "{stdout}");
}

/// A genuine parse error on a shortcut verb still fails, and names the verb.
#[test]
fn shortcut_verb_bad_flag_still_fails() {
    let dir = support::scaffold_project(LEGEND);
    let output = support::mt(dir.path())
        .args(["done", "--no-such-flag"])
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("--no-such-flag"), "{stderr}");
    assert!(!stderr.contains("mindtape-cli"), "{stderr}");
}
