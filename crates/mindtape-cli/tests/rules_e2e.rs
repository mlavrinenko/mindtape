//! End-to-end tests for `[[tasks.rules]]`
//! (`truth/tasks/validate-every-task-against-config-declared-rules.typ`): a
//! project-declared assertion in the filter language, evaluated inside the
//! validation pass every command shares.
//!
//! The seam is what these tests are about. A rule that only fired in
//! `mt check` would be a convention nobody keeps — so `mt add` is asserted
//! here too, refusing an error-severity breach *before* anything is written
//! and letting a warn-severity one through with its warning printed.
#![allow(clippy::unwrap_used)]

mod support;

use support::{mt, run_ok, scaffold_project};

/// A project whose config carries `rules` verbatim plus a single task titled
/// `title`, so a rule's verdict is a property of the title alone.
fn project(rules: &str, title: &str) -> tempfile::TempDir {
    let dir = scaffold_project(&format!("[tasks]\nroots = [\"tasks\"]\n\n{rules}"));
    std::fs::write(
        dir.path().join("tasks/subject.typ"),
        format!(
            "#import \"@local/mindtape:0.2.0\": *\n\n\
             #show: task.with(\n  title: \"{title}\",\n  status: wip(2026, 6, 1),\n)\n"
        ),
    )
    .unwrap();
    dir
}

/// A rule asserting the title opens with an imperative verb, at `severity`.
fn imperative_rule(severity: &str) -> String {
    format!(
        "[[tasks.rules]]\n\
         name = \"imperative-title\"\n\
         assert = 'in_dict(\"mindtape-dev-verbs\", word(title, 1))'\n\
         message = \"start the title with an imperative verb\"\n\
         severity = \"{severity}\"\n"
    )
}

/// `mt check`'s stdout plus stderr, and whether it exited zero.
fn check(dir: &std::path::Path) -> (bool, String) {
    let out = mt(dir).arg("check").output().unwrap();
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    (out.status.success(), text)
}

#[test]
fn an_assert_that_holds_leaves_check_green() {
    let dir = project(&imperative_rule("error"), "Refactor the loader");
    let (ok, text) = check(dir.path());
    assert!(ok, "a satisfied rule must not fail the command: {text}");
    assert!(!text.contains("imperative-title"), "got: {text}");
}

#[test]
fn a_breach_names_the_file_the_rule_and_its_message() {
    let dir = project(&imperative_rule("error"), "Loader refactoring");
    let (ok, text) = check(dir.path());
    assert!(
        !ok,
        "an error-severity breach must fail the command: {text}"
    );
    assert!(
        text.contains(
            "error: tasks/subject.typ:3: rules.imperative-title: \
             start the title with an imperative verb"
        ),
        "a breach reports at the task's marker line, prefixed by its \
         severity: {text}"
    );
}

#[test]
fn a_warn_breach_prints_and_passes() {
    let dir = project(&imperative_rule("warn"), "Loader refactoring");
    let (ok, text) = check(dir.path());
    assert!(ok, "warn is a report, not a failure: {text}");
    assert!(
        text.contains("warning: tasks/subject.typ:3: rules.imperative-title:"),
        "got: {text}"
    );
}

#[test]
fn severity_defaults_to_warn() {
    let rule = "[[tasks.rules]]\n\
                name = \"imperative-title\"\n\
                assert = 'in_dict(\"mindtape-dev-verbs\", word(title, 1))'\n";
    let dir = project(rule, "Loader refactoring");
    let (ok, text) = check(dir.path());
    assert!(ok, "an undeclared severity is `warn`: {text}");
    assert!(text.contains("warning: "), "got: {text}");
}

#[test]
fn a_rule_with_no_message_names_its_own_assert() {
    let rule = "[[tasks.rules]]\n\
                name = \"imperative-title\"\n\
                assert = 'matches(title, \"^Refactor\")'\n\
                severity = \"error\"\n";
    let dir = project(rule, "Loader refactoring");
    let (ok, text) = check(dir.path());
    assert!(!ok, "got: {text}");
    assert!(
        text.contains("rules.imperative-title: must satisfy `matches(title, \"^Refactor\")`"),
        "got: {text}"
    );
}

#[test]
fn an_unnamed_rule_is_identified_by_its_index() {
    let rule = "[[tasks.rules]]\n\
                assert = 'kind == \"bug\"'\n\
                severity = \"error\"\n";
    let dir = project(rule, "Refactor the loader");
    let (ok, text) = check(dir.path());
    assert!(!ok, "got: {text}");
    assert!(
        text.contains("rules[0]: must satisfy `kind == \"bug\"`"),
        "got: {text}"
    );
}

#[test]
fn when_excludes_the_offending_task() {
    let rule = format!("{}when = 'status != \"wip\"'\n", imperative_rule("error"));
    let dir = project(&rule, "Loader refactoring");
    let (ok, text) = check(dir.path());
    assert!(
        ok,
        "a task the `when` gate excludes is never asked the assert: {text}"
    );
}

#[test]
fn a_rule_reaches_a_project_dict() {
    let rules = "[dicts.verbs]\n\
                 values = [\"refactor\", \"ship\"]\n\n\
                 [[tasks.rules]]\n\
                 name = \"imperative-title\"\n\
                 assert = 'in_dict(\"verbs\", word(title, 1))'\n\
                 severity = \"error\"\n";
    let held = project(rules, "Refactor the loader");
    let (ok, text) = check(held.path());
    assert!(ok, "`fold` case matches a capitalised title word: {text}");

    let breached = project(rules, "Loader refactoring");
    let (failed, text) = check(breached.path());
    assert!(!failed, "got: {text}");
    assert!(text.contains("rules.imperative-title"), "got: {text}");
}

#[test]
fn a_malformed_assert_is_one_config_error_at_load() {
    let rule = "[[tasks.rules]]\n\
                name = \"broken\"\n\
                assert = 'nosuchfield == \"x\"'\n";
    let dir = project(rule, "Refactor the loader");
    let (ok, text) = check(dir.path());
    assert!(!ok, "got: {text}");
    assert!(
        text.contains("tasks.rules[0].assert") && text.contains("nosuchfield"),
        "a malformed rule fails at load, naming its own path: {text}"
    );
    assert_eq!(
        text.matches("nosuchfield").count(),
        1,
        "one config error, never one violation per artifact: {text}"
    );
}

#[test]
fn a_malformed_when_is_one_config_error_at_load() {
    let rule = format!("{}when = 'nosuchfield == \"x\"'\n", imperative_rule("warn"));
    let dir = project(&rule, "Refactor the loader");
    let (ok, text) = check(dir.path());
    assert!(!ok, "got: {text}");
    assert!(text.contains("tasks.rules[0].when"), "got: {text}");
}

#[test]
fn mt_add_refuses_a_write_that_breaches_an_error_rule() {
    let dir = project(&imperative_rule("error"), "Refactor the loader");
    let out = mt(dir.path())
        .args(["add", "--name", "leftover", "--set", "title=Loader rewrite"])
        .output()
        .unwrap();
    assert!(!out.status.success(), "an error rule must refuse the write");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("error: leftover: rules.imperative-title"),
        "a blocking problem reports as an error: {stderr}"
    );
    assert!(
        !stderr.contains("warning: leftover:"),
        "a refused write is not a warning: {stderr}"
    );
    assert!(
        !dir.path().join("tasks/leftover.typ").exists(),
        "a refused write must leave nothing behind to repair"
    );
}

#[test]
fn mt_add_lets_a_warn_rule_through() {
    let dir = project(&imperative_rule("warn"), "Refactor the loader");
    let out = mt(dir.path())
        .args(["add", "--name", "leftover", "--set", "title=Loader rewrite"])
        .output()
        .unwrap();
    assert!(out.status.success(), "a warn rule lets the write land");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("warning: leftover: rules.imperative-title"),
        "a non-blocking breach reports as a warning: {stderr}"
    );
    assert!(
        !stderr.contains("error: leftover:"),
        "a write that lands is not an error: {stderr}"
    );
    assert!(
        dir.path().join("tasks/leftover.typ").exists(),
        "a warn-severity breach reports and lets the write land"
    );
}

#[test]
fn mt_flip_refuses_a_write_that_breaches_an_error_rule() {
    // The rule only bites once the task is terminal, so the file is legal
    // until the flip that closes it — the gate is the write, not the history.
    let rule = "[[tasks.rules]]\n\
                name = \"closed-work-is-owned\"\n\
                assert = '!terminal || assignee == \"ada\"'\n\
                severity = \"error\"\n";
    let dir = project(rule, "Refactor the loader");
    run_ok(dir.path(), &["check"]);
    mt(dir.path())
        .args(["done", "subject"])
        .assert()
        .failure()
        .stderr(predicates::str::contains("rules.closed-work-is-owned"));
}

/// The mechanism above is proven against a synthetic project. These two check
/// the ADOPTION itself
/// (truth/tasks/enforce-an-imperative-verb-at-the-start-of-task-titles.typ):
/// that `imperative-title` actually landed in the two real configs the chain
/// was for — this repo's own backlog at `error`, and the `learn-piano`
/// example at `warn`, the severity a project adopting the convention
/// mid-backlog wants. Loaded straight off the committed files rather than
/// re-derived from a scaffold, so a config edit that drops the rule (or
/// loosens its severity) fails here even though the mechanism itself still
/// works.
fn find_imperative_title_rule(config: &mindtape::Config) -> &mindtape::RuleSpec {
    config
        .tasks
        .rules
        .iter()
        .find(|rule| rule.name.as_deref() == Some("imperative-title"))
        .expect("the imperative-title rule must be declared")
}

#[test]
fn this_repos_own_config_carries_the_rule_at_error() {
    let config = mindtape::parse_config(include_str!("../../../.mindtape/config.toml")).unwrap();
    assert_eq!(
        find_imperative_title_rule(&config).severity,
        mindtape::Severity::Error
    );
}

#[test]
fn the_learn_piano_example_carries_the_rule_at_warn() {
    let config = mindtape::parse_config(include_str!(
        "../../../examples/learn-piano/.mindtape/config.toml"
    ))
    .unwrap();
    assert_eq!(
        find_imperative_title_rule(&config).severity,
        mindtape::Severity::Warn
    );
}
