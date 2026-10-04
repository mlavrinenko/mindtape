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

/// A genuine parse error on a shortcut verb prints clap's usage error once and
/// exits 2, as `mt flip done --no-such-flag` does.
#[test]
fn shortcut_verb_bad_flag_is_a_usage_error_exiting_two() {
    let dir = support::scaffold_project(LEGEND);
    let run = |args: &[&str]| support::mt(dir.path()).args(args).output().unwrap();
    let shortcut = run(&["done", "--no-such-flag"]);
    let flip = run(&["flip", "done", "--no-such-flag"]);
    assert_eq!(shortcut.status.code(), Some(2));
    assert_eq!(flip.status.code(), Some(2));
    let stderr = String::from_utf8(shortcut.stderr).unwrap();
    assert!(stderr.contains("--no-such-flag"), "{stderr}");
    assert!(stderr.contains("mt done"), "{stderr}");
    assert!(!stderr.contains("Error: error:"), "{stderr}");
    assert!(!stderr.contains("mindtape-cli"), "{stderr}");
    assert_eq!(stderr.matches("unexpected argument").count(), 1, "{stderr}");
}
