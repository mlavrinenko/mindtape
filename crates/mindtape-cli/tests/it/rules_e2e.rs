//! End-to-end tests for `[[tasks.rules]]`
//! (`truth/tasks/2026-09/validate-every-task-against-config-declared-rules.typ`):
//! a project-declared assertion in the filter language, evaluated inside the
//! validation pass every command shares.
//!
//! A rule that fired only in `mt check` would be a convention nobody keeps.
//! The tests assert `mt add` too.
#![allow(clippy::unwrap_used, clippy::disallowed_methods)]

use mindtape::test_support::write_file;
use std::path::{Path, PathBuf};

use crate::support;
use support::scaffold_project;

/// The four task paths every expected message in this file names.
const SHOWN: &[&str] = &[
    "tasks/subject.typ",
    "tasks/leftover.typ",
    "tasks/bad.typ",
    "tasks/good.typ",
];

/// The repo root, derived from this crate's manifest dir
/// (`crates/mindtape-cli` -> `../..`). A test finds its own
/// `.mindtape/config.toml` whatever directory `cargo test` runs from.
fn repo_root() -> PathBuf {
    mindtape::canonical(&Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")).unwrap()
}

/// A project whose config carries `rules` verbatim plus a single task titled
/// `title`, so a rule's verdict is a property of the title alone.
fn project(rules: &str, title: &str) -> tempfile::TempDir {
    let dir = scaffold_project(&format!("[tasks]\nroots = [\"tasks\"]\n\n{rules}"));
    write_file(
        dir.path(),
        "tasks/subject.typ",
        format!(
            "#import \"@local/mindtape:0.5.0\": *\n\n\
             #show: task.with(\n  title: \"{title}\",\n  status: wip(2026, 6, 1),\n)\n"
        ),
    );
    dir
}

/// A rule asserting the title opens with an imperative verb, at `severity`.
fn imperative_rule(severity: &str) -> String {
    format!(
        "[[tasks.rules]]\n\
         name = \"start-title-with-verb\"\n\
         assert = 'in_dict(\"mindtape-dev-verbs\", word(title, 1))'\n\
         message = \"start the title with an imperative verb\"\n\
         severity = \"{severity}\"\n"
    )
}

/// `mt check`'s stdout plus stderr, and whether it exited zero.
fn check(dir: &std::path::Path) -> (bool, String) {
    support::run_text(dir, &["check"])
}

#[test]
fn an_assert_that_holds_leaves_check_green() {
    let dir = project(&imperative_rule("error"), "Refactor the loader");
    let (ok, text) = check(dir.path());
    assert!(ok, "a satisfied rule must not fail the command: {text}");
    // The built-in template's own title may warn; the task's must not breach.
    assert!(
        !text.lines().any(
            |line| line.contains(&support::shown("tasks/subject.typ", SHOWN))
                && line.contains("start-title-with-verb")
        ),
        "got: {text}"
    );
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
        text.contains(&support::shown(
            "error: tasks/subject.typ:3: rules.start-title-with-verb: \
             start the title with an imperative verb",
            SHOWN
        )),
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
        text.contains(&support::shown(
            "warning: tasks/subject.typ:3: rules.start-title-with-verb:",
            SHOWN
        )),
        "got: {text}"
    );
}

/// The default severity is `error`, so a rule that leaves it out refuses the
/// write. A project writes `warn` to make the rule advisory.
#[test]
fn severity_defaults_to_error() {
    let dir = project(
        "[[tasks.rules]]\nassert = 'kind == \"bug\"'\n",
        "Refactor the loader",
    );
    support::assert_check_fails(dir.path(), "error: tasks/subject.typ:3: rules[0]");
}

#[test]
fn a_rule_with_no_message_names_its_own_assert() {
    let rule = "[[tasks.rules]]\n\
                name = \"start-title-with-verb\"\n\
                assert = 'matches(title, \"^Refactor\")'\n\
                severity = \"error\"\n";
    let dir = project(rule, "Loader refactoring");
    support::assert_check_fails(
        dir.path(),
        "rules.start-title-with-verb: must satisfy `matches(title, \"^Refactor\")`",
    );
}

#[test]
fn an_unnamed_rule_is_identified_by_its_index() {
    let rule = "[[tasks.rules]]\n\
                assert = 'kind == \"bug\"'\n\
                severity = \"error\"\n";
    let dir = project(rule, "Refactor the loader");
    support::assert_check_fails(dir.path(), "rules[0]: must satisfy `kind == \"bug\"`");
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

/// One `matches(dir, …)` rule accepts a month folder and refuses the root.
#[test]
fn a_matches_dir_rule_holds_a_task_to_its_month_folder() {
    let rule = "[[tasks.rules]]\n\
                name = \"file-tasks-by-month\"\n\
                assert = 'matches(dir, \"^tasks/[0-9]{4}-[0-9]{2}$\")'\n\
                message = \"file a task in a month folder\"\n\
                severity = \"error\"\n";
    let flat = project(rule, "Refactor the loader");
    let (ok, text) = check(flat.path());
    assert!(!ok, "a task at the root must be refused: {text}");
    assert!(
        text.contains(&support::shown(
            "tasks/subject.typ:3: rules.file-tasks-by-month",
            SHOWN
        )),
        "a breach names the file and the rule: {text}"
    );

    let nested = scaffold_project(&format!("[tasks]\nroots = [\"tasks\"]\n\n{rule}"));
    write_file(
        nested.path(),
        "tasks/2026-11/subject.typ",
        "#import \"@local/mindtape:0.5.0\": *\n\n#show: task.with(\n  title: \"Refactor the loader\",\n  status: wip(2026, 6, 1),\n)\n",
    );
    let (ok, text) = check(nested.path());
    assert!(ok, "a task in a month folder must pass: {text}");
}

#[test]
fn a_rule_reaches_a_project_dict() {
    let rules = "[dicts.verbs]\n\
                 values = [\"refactor\", \"ship\"]\n\n\
                 [[tasks.rules]]\n\
                 name = \"start-title-with-verb\"\n\
                 assert = 'in_dict(\"verbs\", word(title, 1))'\n\
                 severity = \"error\"\n";
    let held = project(rules, "Refactor the loader");
    let (ok, text) = check(held.path());
    assert!(ok, "`fold` case matches a capitalised title word: {text}");

    let breached = project(rules, "Loader refactoring");
    let (failed, text) = check(breached.path());
    assert!(!failed, "got: {text}");
    assert!(text.contains("rules.start-title-with-verb"), "got: {text}");
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
    support::assert_check_fails(dir.path(), "tasks.rules[0].when");
}

#[test]
fn mt_add_refuses_a_write_that_breaches_an_error_rule() {
    let dir = project(&imperative_rule("error"), "Refactor the loader");
    let stderr = support::failed(
        dir.path(),
        &["add", "--name", "leftover", "--set", "title=Loader rewrite"],
        "an error rule must refuse the write",
    );
    assert!(
        stderr.contains("error: <new task from template 'task'>:3: rules.start-title-with-verb"),
        "a blocking problem reports as an error: {stderr}"
    );
    assert!(
        !stderr.contains(&support::shown("warning: tasks/leftover.typ", SHOWN)),
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
    let out = support::run(
        dir.path(),
        &["add", "--name", "leftover", "--set", "title=Loader rewrite"],
    );
    assert!(out.status.success(), "a warn rule lets the write land");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("warning: <new task from template 'task'>:3: rules.start-title-with-verb"),
        "a non-blocking breach reports as a warning: {stderr}"
    );
    assert!(
        !stderr.contains(&support::shown("error: tasks/leftover.typ", SHOWN)),
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
    support::check_ok(dir.path());
    let stderr = support::failed(
        dir.path(),
        &["done", "subject"],
        "an error rule must refuse the flip",
    );
    assert!(
        stderr.contains(&support::shown(
            "error: tasks/subject.typ:3: rules.closed-work-is-owned",
            SHOWN
        )),
        "a blocking problem reports as an error: {stderr}"
    );
}

/// A warn rule says its piece and the flip still lands, and post-write
/// revalidation keeps that warning a warning.
#[test]
fn mt_flip_lets_a_warn_rule_through() {
    let dir = project(&imperative_rule("warn"), "Loader refactoring");
    let out = support::run(dir.path(), &["done", "subject"]);
    assert!(
        out.status.success(),
        "a warn rule must not fail the flip: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains(&support::shown(
            "warning: tasks/subject.typ:3: rules.start-title-with-verb",
            SHOWN
        )),
        "the warn breach reports as a warning: {stderr}"
    );
    assert!(
        !stderr.contains("error:"),
        "a warn-only flip prints no error: {stderr}"
    );
}

/// The tests above prove the mechanism on a synthetic project. This one checks
/// the adoption itself: `start-title-with-verb` reaches this repo's own backlog
/// at `error`, loading the committed config rather than a scaffold.
fn find_imperative_title_rule(config: &mindtape::Config) -> &mindtape::RuleSpec {
    config
        .tasks
        .rules
        .iter()
        .find(|rule| rule.name.as_deref() == Some("start-title-with-verb"))
        .expect("the start-title-with-verb rule must be declared")
}

/// This repo's own `.mindtape/config.toml` inherits `start-title-with-verb`
/// from the embedded `mindtape:tasks` pack through `extends`. `parse_config`
/// cannot prove that (it never resolves `extends`/`disable`), so this test
/// loads the full layered config against the real repo root instead.
#[test]
fn this_repos_own_config_carries_the_rule_at_error() {
    let config = mindtape::load_config(&repo_root()).unwrap();
    assert_eq!(
        find_imperative_title_rule(&config).severity,
        mindtape::Severity::Error
    );
}

/// `check = { typst, function, field }`
/// (`truth/tasks/2026-09/check-a-task-field-with-a-typst-function.typ`): a rule that
/// specifies a Typst function instead of a filter expression, for a structure
/// (an `x-<ns>` extras dict) the filter language has no way to reach.
///
/// A project whose config carries `rules` verbatim, a `check.typ` module
/// exporting `check-title`, and a single task titled `title`.
fn check_project(rules: &str, title: &str) -> tempfile::TempDir {
    let dir = scaffold_project(&format!("[tasks]\nroots = [\"tasks\"]\n\n{rules}"));
    write_file(
        dir.path(),
        "check.typ",
        "#let check-title(title) = if title == \"Loader refactoring\" {\n  \
             (\"title must not be exactly this\",)\n\
         } else {\n  ()\n}\n",
    );
    write_file(
        dir.path(),
        "tasks/subject.typ",
        format!(
            "#import \"@local/mindtape:0.5.0\": *\n\n\
             #show: task.with(\n  title: \"{title}\",\n  status: wip(2026, 6, 1),\n)\n"
        ),
    );
    dir
}

/// A `check` rule naming `check.typ`'s `check-title`, over the task's `title`
/// field, at `severity`.
fn check_title_rule(severity: &str) -> String {
    format!(
        "[[tasks.rules]]\n\
         name = \"no-loader-refactoring\"\n\
         check = {{ typst = \"check.typ\", function = \"check-title\", field = \"title\" }}\n\
         severity = \"{severity}\"\n"
    )
}

#[test]
fn a_check_rule_that_passes_leaves_check_green() {
    let dir = check_project(&check_title_rule("error"), "Refactor the loader");
    let (ok, text) = check(dir.path());
    assert!(ok, "a satisfied check must not fail the command: {text}");
}

#[test]
fn a_check_breach_names_the_file_the_rule_and_the_functions_own_string() {
    let dir = check_project(&check_title_rule("error"), "Loader refactoring");
    let (ok, text) = check(dir.path());
    assert!(
        !ok,
        "an error-severity check breach must fail the command: {text}"
    );
    assert!(
        text.contains(&support::shown(
            "error: tasks/subject.typ:3: rules.no-loader-refactoring: \
             title must not be exactly this",
            SHOWN
        )),
        "a check breach reports exactly like an assert breach: {text}"
    );
}

#[test]
fn mt_add_refuses_a_write_a_check_rule_rejects() {
    let dir = check_project(&check_title_rule("error"), "Refactor the loader");
    let out = support::run(
        dir.path(),
        &[
            "add",
            "--name",
            "leftover",
            "--set",
            "title=Loader refactoring",
        ],
    );
    assert!(
        !out.status.success(),
        "an error-severity check rule must refuse the write"
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("error: <new task from template 'task'>:3: rules.no-loader-refactoring"),
        "a blocking check problem reports as an error: {stderr}"
    );
    assert!(
        !dir.path().join("tasks/leftover.typ").exists(),
        "a refused write must leave nothing behind to repair"
    );
}

#[test]
fn an_unknown_check_function_is_one_config_error_at_load() {
    let rule = "[[tasks.rules]]\n\
                name = \"broken\"\n\
                check = { typst = \"check.typ\", function = \"no-such-function\" }\n";
    let dir = check_project(rule, "Refactor the loader");
    let (ok, text) = check(dir.path());
    assert!(!ok, "got: {text}");
    assert!(
        text.contains("tasks.rules[0].check") && text.contains("no-such-function"),
        "an unknown function fails at load, naming its own path: {text}"
    );
}

/// A pack file beside the project holds a word list, a rule reading it and a
/// `check` rule whose file imports a sibling. `mt check` resolves all three
/// from the pack's directory, and the project holds none of them.
#[test]
fn a_packs_check_file_and_dict_file_resolve_from_the_pack_directory() {
    let dir =
        scaffold_project("extends = [\"./team/pack.toml\"]\n\n[tasks]\nroots = [\"tasks\"]\n");
    let team = "[dicts.words]\npath = \"words.txt\"\n\n\
         [[tasks.rules]]\nname = \"known-verb\"\nseverity = \"error\"\n\
         assert = 'in_dict(\"words\", word(title, 1))'\n\n\
         [[tasks.rules]]\nname = \"not-banned\"\nseverity = \"error\"\n\
         check = { typst = \"check.typ\", function = \"check-title\", field = \"title\" }\n";
    write_file(dir.path(), "team/pack.toml", team);
    write_file(dir.path(), "team/words.txt", "fix\n");
    write_file(
        dir.path(),
        "team/helper.typ",
        "#let banned = \"Loader refactoring\"\n",
    );
    write_file(
        dir.path(),
        "team/check.typ",
        "#import \"helper.typ\": banned\n\
         #let check-title(title) = if title == banned { (\"title is banned\",) } else { () }\n",
    );
    for (stem, title) in [("bad", "Loader refactoring"), ("good", "Fix the loader")] {
        write_file(
            dir.path(),
            format!("tasks/{stem}.typ"),
            format!(
                "#import \"@local/mindtape:0.5.0\": *\n\n\
                 #show: task.with(\n  title: \"{title}\",\n  status: wip(2026, 6, 1),\n)\n"
            ),
        );
    }

    let (ok, text) = check(dir.path());

    assert!(!ok, "{text}");
    assert!(
        text.contains(&support::shown("tasks/bad.typ:3: rules.known-verb", SHOWN)),
        "{text}"
    );
    assert!(text.contains("rules.not-banned: title is banned"), "{text}");
    assert!(
        !text.contains(&support::shown("tasks/good.typ", SHOWN)),
        "{text}"
    );
}
