//! End-to-end tests for `mt rules`
//! (`truth/tasks/list-the-active-rules-in-one-line-each-with-mt-rules.typ`):
//! one line per rule the merged config applies, after packs, `disable` and
//! same-name overrides, and `--for` narrowing it to the command that enforces
//! it.
#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use mindtape::test_support::write_file;

use crate::support;

use support::{mt, run_text, scaffold_project};

/// A pack holding a task rule the project overrides, one it disables, a body
/// rule and a commit rule.
const PACK: &str = r#"
[[tasks.rules]]
name = "verb-title"
assert = 'has(title)'
message = "start the title with a verb"

[[tasks.rules]]
name = "no-todo"
assert = 'has(title)'
message = "drop the TODO"

[[tasks.body.headers.rules]]
name = "one-problem"
action = "unique"
path = ["Problem"]
message = "keep one Problem section"

[[commits.rules]]
name = "names-a-task"
assert = 'has(refs)'
message = "name the task in a Refs: footer"
"#;

/// The project: it extends [`PACK`], overrides `verb-title` with its own
/// message and severity, disables `no-todo`, adds an unnamed rule with no
/// message, and gates `done` with a flip action that carries a message.
const CONFIG: &str = r#"extends = ["./pack.toml"]
disable = ["no-todo"]

[tasks]
roots = ["tasks"]

[[tasks.rules]]
name = "verb-title"
when = '!terminal'
assert = 'has(title)'
message = "open the title with an imperative verb"
severity = "error"

[[tasks.rules]]
assert = 'has(title)'

[[on.flip]]
when = 'into("done")'
actions = [{ gate = true, message = "run the tests, then flip again" }]
"#;

fn project() -> tempfile::TempDir {
    let dir = scaffold_project(CONFIG);
    write_file(dir.path(), "pack.toml", PACK);
    dir
}

fn stdout(dir: &std::path::Path, args: &[&str]) -> String {
    let out = mt(dir).args(args).output().unwrap();
    assert!(out.status.success(), "{out:?}");
    String::from_utf8(out.stdout).unwrap()
}

/// The task and body rules, padded to the widest name of the full list.
const TASK_LINES: &str = "\
error  tasks.rules.verb-title                open the title with an imperative verb
warn   tasks.rules[1]                        must satisfy `has(title)`
error  tasks.body.headers.rules.one-problem  keep one Problem section
";

const COMMIT_LINE: &str =
    "warn   commits.rules.names-a-task            name the task in a Refs: footer\n";

const FLIP_LINE: &str =
    "error  on.flip[0].actions[0]                 run the tests, then flip again\n";

#[test]
fn lists_each_rule_after_the_pack_override_and_disable() {
    let dir = project();

    assert_eq!(
        stdout(dir.path(), &["rules"]),
        format!("{TASK_LINES}{COMMIT_LINE}{FLIP_LINE}")
    );
}

/// Each array rule is named as `mt config show --origin` names it, in its
/// order, so a reader can look a listed rule up there.
#[test]
fn names_each_rule_as_config_show_origin_does() {
    let dir = project();
    let origin = stdout(dir.path(), &["config", "show", "--origin"]);
    let labels: Vec<&str> = origin
        .lines()
        .filter_map(|line| line.strip_prefix("rule "))
        .filter_map(|line| line.split(": ").next())
        .collect();

    let listed = stdout(dir.path(), &["rules"]);
    let names: Vec<&str> = listed
        .lines()
        .filter_map(|line| line.split_whitespace().nth(1))
        .filter(|name| !name.starts_with("on.flip"))
        .collect();

    assert_eq!(names, labels);
}

/// `mt add` checks the task and body rules, `mt flip` those plus its flip
/// actions, and a commit only the commit rules.
#[test]
fn for_keeps_the_rules_that_moment_checks() {
    let dir = project();
    let cases = [
        ("add", TASK_LINES.to_owned()),
        ("flip", format!("{TASK_LINES}{FLIP_LINE}")),
        (
            "commit",
            "warn   commits.rules.names-a-task  name the task in a Refs: footer\n".to_owned(),
        ),
    ];

    for (moment, expected) in cases {
        assert_eq!(
            stdout(dir.path(), &["rules", "--for", moment]),
            expected,
            "--for {moment}"
        );
    }
}

#[test]
fn json_carries_the_section_when_and_assert() {
    let dir = project();

    let value: serde_json::Value =
        serde_json::from_str(&stdout(dir.path(), &["rules", "--json"])).unwrap();

    assert_eq!(value["schema_version"], mindtape::SCHEMA_VERSION);
    let rules = value["rules"].as_array().unwrap();
    assert_eq!(rules.len(), 5, "{value}");
    assert_eq!(
        rules[0],
        serde_json::json!({
            "section": "tasks.rules",
            "name": "tasks.rules.verb-title",
            "severity": "error",
            "message": "open the title with an imperative verb",
            "when": "!terminal",
            "assert": "has(title)",
            "check": null,
        })
    );
    assert_eq!(
        rules[4],
        serde_json::json!({
            "section": "on.flip",
            "name": "on.flip[0].actions[0]",
            "severity": "error",
            "message": "run the tests, then flip again",
            "when": "into(\"done\")",
            "assert": null,
            "check": null,
        })
    );
}

#[test]
fn json_for_narrows_like_the_plain_list() {
    let dir = project();

    let value: serde_json::Value =
        serde_json::from_str(&stdout(dir.path(), &["rules", "--for", "commit", "--json"])).unwrap();

    assert_eq!(value["rules"].as_array().unwrap().len(), 1, "{value}");
    assert_eq!(value["rules"][0]["name"], "commits.rules.names-a-task");
}

#[test]
fn says_so_when_no_rule_applies() {
    let dir = scaffold_project("[tasks]\nroots = [\"tasks\"]\n");

    assert_eq!(stdout(dir.path(), &["rules"]), "no rules apply\n");
}

#[test]
fn refuses_an_unknown_moment() {
    let dir = project();

    let (ok, text) = run_text(dir.path(), &["rules", "--for", "push"]);

    assert!(!ok, "{text}");
    assert!(
        text.contains("possible values: add, commit, flip"),
        "{text}"
    );
}
