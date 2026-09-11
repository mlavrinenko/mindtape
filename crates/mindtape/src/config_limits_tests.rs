#![allow(clippy::unwrap_used)]

use super::{Severity, check_limits};
use crate::config::Config;
use crate::model::{Status, Task};
use crate::parse_config;
use crate::test_support::bare_task;

/// A task carrying a status and (optionally) a `kind`; everything else empty.
fn task(id: &str, status: &str, kind: Option<&str>) -> Task {
    let mut built = bare_task(id);
    built.status = Some(Status {
        name: status.to_owned(),
        date: None,
        note: None,
    });
    built.kind = kind.map(str::to_owned);
    built
}

fn config(toml: &str) -> Config {
    parse_config(toml).unwrap()
}

/// One `wip` cap of 1, the default (warn) severity.
const WIP_MAX_ONE: &str = "[[tasks.limits]]\nfilter = 'status == \"wip\"'\nmax = 1\n";

/// Ids of the tasks reported by the single breach `check_limits` must produce.
fn sole_breach_ids(tasks: &[Task], config: &Config) -> Vec<String> {
    let breaches = check_limits(tasks, config).unwrap();
    assert_eq!(
        breaches.len(),
        1,
        "expected exactly one breach: {breaches:?}"
    );
    breaches.first().unwrap().tasks.clone()
}

#[test]
fn no_limits_configured_is_silent() {
    let tasks = vec![task("a", "wip", None), task("b", "wip", None)];
    assert!(check_limits(&tasks, &Config::default()).unwrap().is_empty());
}

#[test]
fn under_the_cap_is_silent() {
    let tasks = vec![task("a", "wip", None), task("b", "done", None)];
    assert!(
        check_limits(&tasks, &config(WIP_MAX_ONE))
            .unwrap()
            .is_empty()
    );
}

#[test]
fn at_the_cap_is_silent() {
    let tasks = vec![task("a", "wip", None), task("b", "wip", None)];
    let two = "[[tasks.limits]]\nfilter = 'status == \"wip\"'\nmax = 2\n";
    assert!(check_limits(&tasks, &config(two)).unwrap().is_empty());
    // Exactly one match against a cap of one is likewise no breach.
    let one = vec![task("a", "wip", None)];
    assert!(check_limits(&one, &config(WIP_MAX_ONE)).unwrap().is_empty());
}

#[test]
fn over_the_cap_reports_count_cap_and_tasks() {
    let tasks = vec![
        task("a", "wip", None),
        task("b", "wip", None),
        task("c", "done", None),
        task("d", "wip", None),
    ];
    let breaches = check_limits(&tasks, &config(WIP_MAX_ONE)).unwrap();
    let breach = breaches.first().unwrap();
    assert_eq!(breaches.len(), 1);
    assert_eq!(breach.index, 0);
    assert_eq!(breach.count(), 3);
    assert_eq!(breach.max, 1);
    assert_eq!(breach.tasks, ["a", "b", "d"]);
    assert_eq!(breach.severity, Severity::Warn);
    let rendered = breach.to_string();
    assert!(rendered.contains("3 task(s) match"), "{rendered}");
    assert!(rendered.contains("status == \"wip\""), "{rendered}");
    assert!(rendered.contains("max 1"), "{rendered}");
    assert!(rendered.contains(super::DEFAULT_ADVICE), "{rendered}");
}

#[test]
fn a_filter_matching_nothing_is_silent() {
    let tasks = vec![task("a", "wip", Some("bug")), task("b", "wip", Some("bug"))];
    let toml = "[[tasks.limits]]\nfilter = 'kind == \"epic\"'\nmax = 0\n";
    assert!(check_limits(&tasks, &config(toml)).unwrap().is_empty());
}

#[test]
fn two_limits_report_independently() {
    let tasks = vec![
        task("a", "wip", Some("umbrella")),
        task("b", "wip", Some("umbrella")),
        task("c", "wip", None),
        task("d", "wip", Some("bug")),
    ];
    let toml = "\
[[tasks.limits]]
filter = 'status == \"wip\" && kind == \"umbrella\"'
max = 1

[[tasks.limits]]
filter = 'status == \"wip\" && kind != \"umbrella\"'
max = 1
severity = \"error\"
";
    let breaches = check_limits(&tasks, &config(toml)).unwrap();
    assert_eq!(breaches.len(), 2);
    let umbrellas = breaches.first().unwrap();
    let leaves = breaches.get(1).unwrap();
    assert_eq!(umbrellas.index, 0);
    assert_eq!(umbrellas.tasks, ["a", "b"]);
    assert_eq!(umbrellas.severity, Severity::Warn);
    assert_eq!(leaves.index, 1);
    assert_eq!(leaves.tasks, ["c", "d"]);
    assert_eq!(leaves.severity, Severity::Error);
}

/// The rule the umbrella pair rests on: a task with no `kind` at all counts
/// against a `kind != "..."` limit (see `compare` in `filter.rs`).
#[test]
fn a_task_with_no_kind_counts_against_a_kind_inequality() {
    let tasks = vec![
        task("kindless-one", "wip", None),
        task("kindless-two", "wip", None),
        task("coordinator", "wip", Some("umbrella")),
    ];
    let toml = "[[tasks.limits]]\nfilter = 'status == \"wip\" && kind != \"umbrella\"'\nmax = 1\n";
    assert_eq!(
        sole_breach_ids(&tasks, &config(toml)),
        ["kindless-one", "kindless-two"]
    );
}

/// No status name is hardcoded: a project whose in-flight state is `doing` caps
/// it with the same primitive.
#[test]
fn an_in_flight_status_need_not_be_named_wip() {
    let tasks = vec![
        task("a", "doing", None),
        task("b", "doing", None),
        task("c", "todo", None),
    ];
    let toml = "\
[[tasks.status]]
value = \"todo\"
[[tasks.status]]
value = \"doing\"
[[tasks.status]]
value = \"shipped\"
terminal = true

[[tasks.limits]]
filter = 'status == \"doing\"'
max = 1
";
    assert_eq!(sole_breach_ids(&tasks, &config(toml)), ["a", "b"]);
}

#[test]
fn a_configured_message_replaces_the_default_advice() {
    let tasks = vec![task("a", "wip", None), task("b", "wip", None)];
    let toml = "\
[[tasks.limits]]
filter = 'status == \"wip\"'
max = 1
message = \"close the open leaf before opening another\"
severity = \"error\"
";
    let breaches = check_limits(&tasks, &config(toml)).unwrap();
    let breach = breaches.first().unwrap();
    assert_eq!(breach.message, "close the open leaf before opening another");
    assert!(breach.severity.is_error());
    assert_eq!(breach.severity.label(), "error");
    assert!(!Severity::Warn.is_error());
    assert_eq!(Severity::Warn.label(), "warning");
    assert!(breach.to_string().contains(&breach.message));
}

#[test]
fn an_unparsable_filter_names_the_offending_limit() {
    let tasks = vec![task("a", "wip", None)];
    let toml = "\
[[tasks.limits]]
filter = 'status == \"wip\"'
max = 9

[[tasks.limits]]
filter = 'nonesuch == \"x\"'
max = 1
";
    let err = check_limits(&tasks, &config(toml)).unwrap_err();
    let msg = err.to_string();
    assert_eq!(err.index, 1);
    assert!(msg.contains("tasks.limits[1]"), "{msg}");
    assert!(msg.contains("nonesuch"), "{msg}");
}

/// A malformed limit filter is one config error at load, the same treatment
/// `required` and `[views]` get — not a surprise at the end of a `mt check`.
#[test]
fn a_malformed_limit_filter_fails_at_config_load() {
    let err = Config::default().validate().err();
    assert!(err.is_none(), "the empty config must validate: {err:?}");

    let unknown_field = config("[[tasks.limits]]\nfilter = 'nonesuch == \"x\"'\nmax = 1\n");
    let msg = unknown_field.validate().unwrap_err();
    assert!(msg.contains("tasks.limits[0].filter"), "{msg}");
    assert!(msg.contains("nonesuch"), "{msg}");

    let syntax = config("[[tasks.limits]]\nfilter = 'status == '\nmax = 1\n");
    let msg = syntax.validate().unwrap_err();
    assert!(msg.contains("tasks.limits[0].filter"), "{msg}");

    let unknown_view = config("[[tasks.limits]]\nfilter = '@nosuchview'\nmax = 1\n");
    let msg = unknown_view.validate().unwrap_err();
    assert!(msg.contains("tasks.limits[0].filter"), "{msg}");

    let good = config(WIP_MAX_ONE);
    assert!(good.validate().is_ok(), "a valid limit must not be flagged");
}

/// `max` is required rather than defaulted: a missing cap must not silently
/// read as zero (which would forbid the matched set outright).
#[test]
fn a_limit_without_max_fails_to_parse() {
    let err = parse_config("[[tasks.limits]]\nfilter = 'status == \"wip\"'\n").unwrap_err();
    assert!(err.to_string().contains("max"), "{err}");
}

#[test]
fn an_unknown_limit_key_is_rejected() {
    let err =
        parse_config("[[tasks.limits]]\nfilter = ''\nmax = 1\nview = \"stuck\"\n").unwrap_err();
    assert!(err.to_string().contains("view"), "{err}");
}
