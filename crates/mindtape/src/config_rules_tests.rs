#![allow(clippy::unwrap_used)]

use super::{RuleSpec, check_all, labels, validate_rules};
use crate::config::Config;
use crate::model::{Status, Task};
use crate::parse_config;
use crate::severity::Severity;
use crate::test_support::bare_task;
use crate::validate::Violation;

/// A task with a title, a status, and a marker line — the three things a rule
/// and its report read.
fn task(id: &str, title: &str, status: &str) -> Task {
    let mut built = bare_task(id);
    built.title = title.to_owned();
    built.status = Some(Status {
        name: status.to_owned(),
        date: None,
        note: None,
    });
    built.marker_line = 3;
    built
}

fn config(toml: &str) -> Config {
    parse_config(toml).unwrap()
}

/// Every breach `check_all` raises over `tasks`.
fn breaches(tasks: &[Task], config: &Config) -> Vec<Violation> {
    let mut out = Vec::new();
    check_all(tasks, config, &mut out);
    out
}

/// The one breach `check_all` must raise, or a panic naming what it got.
fn sole_breach(tasks: &[Task], config: &Config) -> Violation {
    let found = breaches(tasks, config);
    assert_eq!(found.len(), 1, "expected exactly one breach: {found:?}");
    found.into_iter().next().unwrap()
}

/// The `imperative-title` rule at `error`, over the built-in dev-verb dict.
const IMPERATIVE: &str = "\
[[tasks.rules]]
name = \"imperative-title\"
assert = 'in_dict(\"mindtape-dev-verbs\", word(title, 1))'
message = \"start the title with an imperative verb\"
severity = \"error\"
";

#[test]
fn no_rules_configured_is_silent() {
    let tasks = vec![task("a", "Loader refactoring", "wip")];
    assert!(breaches(&tasks, &Config::default()).is_empty());
}

#[test]
fn a_satisfied_assert_raises_nothing() {
    let tasks = vec![task("a", "Refactor the loader", "wip")];
    assert!(breaches(&tasks, &config(IMPERATIVE)).is_empty());
}

#[test]
fn a_breach_carries_the_label_message_location_and_severity() {
    let tasks = vec![task("a", "Loader refactoring", "wip")];
    let breach = sole_breach(&tasks, &config(IMPERATIVE));
    assert_eq!(breach.task_id, "a");
    assert_eq!(breach.field, "rules.imperative-title");
    assert_eq!(breach.message, "start the title with an imperative verb");
    assert_eq!(breach.severity, Some(Severity::Error));
    assert!(breach.is_fatal());
    let location = breach.location;
    assert_eq!(location.file, std::path::Path::new("tasks/a.typ"));
    assert_eq!(location.line, 3);
}

#[test]
fn a_warn_breach_is_not_fatal() {
    let toml = "[[tasks.rules]]\nname = \"n\"\nassert = 'kind == \"bug\"'\n";
    let breach = sole_breach(&[task("a", "T", "wip")], &config(toml));
    assert_eq!(breach.severity, Some(Severity::Warn));
    assert!(!breach.is_fatal(), "warn prints, it does not fail");
}

/// Every check `MindTape` makes on its own stays fatal and label-less: severity
/// is the one thing only a config-declared rule gets to choose.
#[test]
fn a_built_in_violation_carries_no_severity() {
    let breach = crate::validate::violation(&task("a", "T", "wip"), "tags", "nope".to_owned());
    assert_eq!(breach.severity, None);
    assert!(breach.is_fatal());
}

#[test]
fn when_gates_which_tasks_are_asked() {
    let toml = format!("{IMPERATIVE}when = 'status != \"wip\"'\n");
    let tasks = vec![task("a", "Loader refactoring", "wip")];
    assert!(
        breaches(&tasks, &config(&toml)).is_empty(),
        "a task the gate excludes is never asked the assert"
    );
    let tasks = vec![task("a", "Loader refactoring", "done")];
    assert_eq!(sole_breach(&tasks, &config(&toml)).task_id, "a");
}

#[test]
fn an_absent_message_names_the_assert_expression() {
    let toml = "[[tasks.rules]]\nname = \"n\"\nassert = 'kind == \"bug\"'\n";
    let breach = sole_breach(&[task("a", "T", "wip")], &config(toml));
    assert_eq!(breach.message, "must satisfy `kind == \"bug\"`");
}

#[test]
fn an_unnamed_rule_is_labelled_by_its_index() {
    let toml = "[[tasks.rules]]\nassert = 'kind == \"bug\"'\n\n[[tasks.rules]]\nassert = 'kind == \"bug\"'\n";
    let found = breaches(&[task("a", "T", "wip")], &config(toml));
    let labels: Vec<&str> = found.iter().map(|b| b.field.as_str()).collect();
    assert_eq!(labels, ["rules[0]", "rules[1]"]);
}

/// `label` is the reporting contract, so it is pinned directly too: a mutant
/// swapping the two arms would still produce two distinct labels above.
#[test]
fn label_prefers_the_name_over_the_index() {
    let named = RuleSpec {
        name: Some("imperative-title".to_owned()),
        assertion: Some("true".to_owned()),
        check: None,
        when: None,
        message: None,
        severity: Severity::Warn,
    };
    assert_eq!(named.label(7), "rules.imperative-title");
    let anonymous = RuleSpec {
        name: None,
        ..named
    };
    assert_eq!(anonymous.label(7), "rules[7]");
}

/// Two entries in `[[tasks.rules]]` sharing a `name` — WITHIN one file — is a
/// config error naming both, the same treatment
/// `crate::body_headers::validate_header_rules` /
/// `crate::body_content::validate_content_rules` already give their own
/// arrays. Cross-FILE replacement by name (a nearer layer or an extended
/// pack overriding a farther one's same-named rule) stays legal — that
/// merge already collapses to one entry before `validate` ever sees this
/// array, so this check only ever sees the same-file case it is for; see
/// `crate::config_packs` for the merge and its own coverage of the
/// cross-file case.
#[test]
fn rejects_duplicate_rule_names_within_one_array() {
    let toml = "[[tasks.rules]]\nname = \"dup\"\nassert = 'kind == \"bug\"'\n\n\
                [[tasks.rules]]\nname = \"dup\"\nassert = 'kind == \"chore\"'\n";
    let err = config(toml).validate().unwrap_err();
    assert!(
        err.contains("tasks.rules[1]") && err.contains("dup"),
        "{err}"
    );
}

/// An absent `name` never collides — only two rules that both opted into a
/// name can clash, matching `duplicate_rule_names`'s own reading.
#[test]
fn two_unnamed_rules_never_collide() {
    let toml = "[[tasks.rules]]\nassert = 'kind == \"bug\"'\n\n\
                [[tasks.rules]]\nassert = 'kind == \"bug\"'\n";
    config(toml).validate().unwrap();
}

#[test]
fn a_task_with_no_source_reports_at_line_one() {
    let mut subject = task("a", "T", "wip");
    subject.marker_line = 0;
    let toml = "[[tasks.rules]]\nname = \"n\"\nassert = 'kind == \"bug\"'\n";
    assert_eq!(sole_breach(&[subject], &config(toml)).location.line, 1);
}

#[test]
fn rules_run_task_by_task_in_config_order() {
    let toml = "[[tasks.rules]]\nname = \"one\"\nassert = 'kind == \"bug\"'\n\n\
                [[tasks.rules]]\nname = \"two\"\nassert = 'kind == \"bug\"'\n";
    let tasks = vec![task("a", "T", "wip"), task("b", "T", "wip")];
    let found = breaches(&tasks, &config(toml));
    let seen: Vec<String> = found
        .iter()
        .map(|b| format!("{}/{}", b.task_id, b.field))
        .collect();
    assert_eq!(
        seen,
        ["a/rules.one", "a/rules.two", "b/rules.one", "b/rules.two"]
    );
}

/// `ready` / `blocked` need readiness over the whole set, so a rule can assert
/// something about what is startable — not only about a field.
#[test]
fn a_rule_can_read_readiness() {
    let toml = "[[tasks.rules]]\nname = \"no-blocked-wip\"\nassert = '!blocked'\n";
    let mut blocked = task("a", "T", "wip");
    blocked.links = vec![crate::link::Link {
        kind: "depends-on".to_owned(),
        task: "b.typ".to_owned(),
        note: None,
    }];
    let tasks = vec![blocked, task("b", "T", "wip")];
    assert_eq!(sole_breach(&tasks, &config(toml)).task_id, "a");
}

#[test]
fn an_unusable_assert_is_one_load_problem_naming_its_path() {
    let toml = "[[tasks.rules]]\nname = \"n\"\nassert = 'nosuchfield == \"x\"'\n";
    let problems = validate_rules(&config(toml));
    assert_eq!(problems.len(), 1, "{problems:?}");
    let first = problems.first().unwrap();
    assert!(first.starts_with("tasks.rules[0].assert: "), "{first}");
    assert!(first.contains("nosuchfield"), "{first}");
}

#[test]
fn an_unusable_when_is_reported_at_its_own_path() {
    let toml = format!("{IMPERATIVE}when = 'in_dict(\"ghost\", title)'\n");
    let problems = validate_rules(&config(&toml));
    assert_eq!(problems.len(), 1, "{problems:?}");
    assert!(
        problems
            .first()
            .unwrap()
            .starts_with("tasks.rules[0].when: "),
        "{problems:?}"
    );
}

#[test]
fn a_syntax_error_is_reported_at_load() {
    let toml = "[[tasks.rules]]\nassert = '(status == \"wip\"'\n";
    assert_eq!(validate_rules(&config(toml)).len(), 1);
}

/// Both `assert` and `check` on one rule is the first half of the shape
/// contract `validate_rules` enforces at load.
#[test]
fn a_rule_with_both_assert_and_check_is_one_load_problem() {
    let toml = "[[tasks.rules]]\nassert = 'kind == \"bug\"'\ncheck = { typst = \"check.typ\", function = \"f\" }\n";
    let problems = validate_rules(&config(toml));
    assert_eq!(problems.len(), 1, "{problems:?}");
    let first = problems.first().unwrap();
    assert!(first.starts_with("tasks.rules[0]: "), "{first}");
    assert!(first.contains("not both"), "{first}");
}

/// Neither `assert` nor `check` is the other half: a rule that constrains
/// nothing is refused rather than silently dropped.
#[test]
fn a_rule_with_neither_assert_nor_check_is_one_load_problem() {
    let toml = "[[tasks.rules]]\nname = \"empty\"\n";
    let problems = validate_rules(&config(toml));
    assert_eq!(problems.len(), 1, "{problems:?}");
    let first = problems.first().unwrap();
    assert!(first.starts_with("tasks.rules[0]: "), "{first}");
    assert!(first.contains("is required"), "{first}");
}

/// A rule whose assert cannot be parsed constrains nothing at evaluation time
/// — `Config::validate` already refused the config, so reaching here means a
/// hand-built one, and a broken rule may not invent breaches.
#[test]
fn an_unparsable_rule_raises_no_breach() {
    let toml = "[[tasks.rules]]\nassert = '(status == \"wip\"'\n";
    assert!(breaches(&[task("a", "T", "wip")], &config(toml)).is_empty());
}

/// A `when` that cannot be parsed drops the rule rather than leaving an
/// assertion running ungated over the whole backlog.
#[test]
fn an_unparsable_when_drops_the_whole_rule() {
    let toml = "[[tasks.rules]]\nassert = 'kind == \"bug\"'\nwhen = '(status == \"wip\"'\n";
    assert!(breaches(&[task("a", "T", "wip")], &config(toml)).is_empty());
}

#[test]
fn the_whole_config_refuses_a_malformed_rule() {
    let err = config("[[tasks.rules]]\nassert = 'nosuchfield == 1'\n")
        .validate()
        .unwrap_err();
    assert!(err.contains("tasks.rules[0].assert"), "{err}");
}

/// An estimate unit outside `[tasks.estimate] units` fails at load, like an
/// unknown field.
#[test]
fn the_whole_config_refuses_an_undeclared_estimate_unit() {
    let toml = "[tasks.estimate]\nunits = [\"files\"]\n\n\
                [[tasks.rules]]\nassert = 'estimate.src-files <= 5'\n";
    let err = config(toml).validate().unwrap_err();
    assert!(
        err.contains("tasks.rules[0].assert") && err.contains("estimate.src-files"),
        "{err}"
    );
    let toml = toml.replace("\"files\"", "\"src-files\"");
    config(&toml).validate().unwrap();
}

/// A rule reading `changed()` over a root git cannot read: skipped once, as a
/// warning, whatever its own severity and however many tasks it covers.
#[test]
fn a_rule_reading_changed_outside_git_is_skipped_once_never_an_error() {
    let dir = tempfile::tempdir().unwrap();
    let mut config = config(
        "\
[[tasks.rules]]
name = \"src-budget\"
assert = 'changed(\"staged+worktree\", \"src/**\") <= 3'
severity = \"error\"
",
    );
    config.git.set_root(dir.path());
    let tasks = vec![task("a", "Refactor it", "wip"), task("b", "Fix it", "wip")];
    let skip = sole_breach(&tasks, &config);
    assert_eq!(skip.field, "rules.src-budget");
    assert_eq!(skip.severity, Some(Severity::Warn));
    assert!(skip.message.starts_with("skipped: "), "{}", skip.message);
    assert!(
        skip.message.contains("not a git repository"),
        "{}",
        skip.message
    );
}

/// The same holds when `changed()` sits in the rule's `when`.
#[test]
fn a_when_reading_changed_outside_git_skips_the_rule_too() {
    let dir = tempfile::tempdir().unwrap();
    let mut config = config(
        "\
[[tasks.rules]]
name = \"busy\"
when = 'changed(\"worktree\") > 0'
assert = 'has(title)'
",
    );
    config.git.set_root(dir.path());
    let skip = sole_breach(&[task("a", "Refactor it", "wip")], &config);
    assert!(skip.message.starts_with("skipped: "), "{}", skip.message);
}

/// `labels` names a rule by its `name` when it has one, else by its position —
/// the same fallback [`RuleSpec::label`] uses, since
/// [`crate::candidate::elsewhere_violations`] matches a violation's `field`
/// against exactly these strings.
#[test]
fn labels_names_by_name_then_by_index() {
    let config = config(
        "\
[[tasks.rules]]
name = \"named\"
assert = 'has(title)'

[[tasks.rules]]
assert = 'has(title)'
",
    );
    let names = labels(&config);
    assert!(names.contains("rules.named"), "{names:?}");
    assert!(names.contains("rules[1]"), "{names:?}");
}
