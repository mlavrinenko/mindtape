#![allow(clippy::unwrap_used)]

use super::{
    ALWAYS, ActionKind, FlipAction, NEVER, copy_in_string, desugar_into, into_targets,
    matching_flip_hooks, validate_flip_hooks,
};
use crate::config::{Config, parse_config};
use crate::filter::Filter;
use crate::model::{Status, Task};
use crate::ready::Readiness;
use crate::test_support::bare_task;

/// A task at `status`, for scoping a `when` against the status being left.
fn task_at(id: &str, status: &str) -> Task {
    let mut task = bare_task(id);
    task.status = Some(Status {
        name: status.to_owned(),
        date: Some("2026-07-22".to_owned()),
        note: None,
    });
    task
}

fn config_with(hooks: &str) -> Config {
    parse_config(hooks).unwrap()
}

/// The two atoms `into(...)` rewrites to must really be constant, since every
/// hook's scoping rests on them. `has(id)` holds for any task because `Task::id`
/// is a plain `String` that is never absent; `miss(id)` is its complement.
/// If the filter language ever makes `id` optional, this fails first.
#[test]
fn into_atoms_are_constant() {
    let config = Config::default();
    for task in [Task::default(), bare_task("x"), task_at("y", "wip")] {
        assert!(
            Filter::parse(ALWAYS)
                .unwrap()
                .matches(&task, &config)
                .unwrap(),
            "ALWAYS must match every task"
        );
        assert!(
            !Filter::parse(NEVER)
                .unwrap()
                .matches(&task, &config)
                .unwrap(),
            "NEVER must match no task"
        );
    }
}

#[test]
fn desugar_into_rewrites_on_target_match() {
    assert_eq!(desugar_into(r#"into("done")"#, "done"), ALWAYS);
    assert_eq!(desugar_into(r#"into("done")"#, "wip"), NEVER);
}

#[test]
fn desugar_into_accepts_whitespace_in_the_call() {
    assert_eq!(desugar_into(r#"into ( "done" )"#, "done"), ALWAYS);
}

#[test]
fn desugar_into_keeps_the_rest_of_the_expression() {
    let out = desugar_into(r#"into("done") && has_tag("api")"#, "done");
    assert_eq!(out, format!("{ALWAYS} && has_tag(\"api\")"));
}

/// An `into(` inside a string literal is data, not a call.
#[test]
fn desugar_into_skips_string_literals() {
    let expr = r#"contains("into(\"done\")", title)"#;
    assert_eq!(desugar_into(expr, "done"), expr);
    assert!(into_targets(expr).is_empty());
}

/// A backslash inside a literal consumes the character after it, so the quote
/// it escapes does not close the string and a following `into(...)` stays data.
#[test]
fn copy_in_string_consumes_the_escaped_character() {
    let mut out = String::new();
    let mut in_string = true;
    let consumed = copy_in_string("\\\"rest", '\\', &mut out, &mut in_string);
    assert_eq!(consumed, 2, "backslash and its escaped quote are one unit");
    assert_eq!(out, "\\\"");
    assert!(in_string, "an escaped quote must not close the literal");
}

/// An unescaped `"` clears `in_string`, so the scanner resumes looking for
/// `into(...)` calls after the literal.
#[test]
fn copy_in_string_closes_on_the_unescaped_quote() {
    let mut out = String::new();
    let mut in_string = true;
    let consumed = copy_in_string("\"rest", '"', &mut out, &mut in_string);
    assert_eq!(consumed, 1);
    assert_eq!(out, "\"");
    assert!(!in_string, "an unescaped quote must close the literal");
}

/// `into` is matched as a whole identifier only.
#[test]
fn desugar_into_leaves_longer_identifiers_alone() {
    for expr in [r#"zinto("done")"#, r#"into_view("done")"#] {
        assert_eq!(desugar_into(expr, "done"), expr);
    }
}

/// A malformed call passes through, so the filter parser reports it by name
/// rather than the hook silently never firing.
#[test]
fn desugar_into_passes_malformed_calls_through() {
    let expr = "into(done)";
    assert_eq!(desugar_into(expr, "done"), expr);
    assert!(
        Filter::parse(expr)
            .unwrap()
            .matches(&Task::default(), &Config::default())
            .is_err()
    );
}

#[test]
fn into_targets_collects_every_name_in_order() {
    let expr = r#"into("done") || into("cancelled")"#;
    assert_eq!(into_targets(expr), vec!["done", "cancelled"]);
}

// --- action kinds ------------------------------------------------------

fn action(toml: &str) -> FlipAction {
    toml::from_str(toml).unwrap()
}

#[test]
fn action_kinds_resolve() {
    assert_eq!(
        action(r#"show = "hi""#).kind().unwrap(),
        ActionKind::Show("hi")
    );
    assert_eq!(
        action(
            r#"list = "@wip"
title = "In flight""#
        )
        .kind()
        .unwrap(),
        ActionKind::List {
            filter: "@wip",
            title: Some("In flight"),
        }
    );
    assert_eq!(
        action(
            r#"run = "just check"
gate = true"#
        )
        .kind()
        .unwrap(),
        ActionKind::Run {
            command: Some("just check"),
            gate: true,
            message: None,
        }
    );
}

/// `run` is optional on a gating action: `gate = true` with no `run` still
/// resolves to `Run`, just with no command to execute — the flip refuses
/// unconditionally whenever the hook's `when` matches.
#[test]
fn a_gate_with_no_run_resolves_as_a_runless_gate() {
    assert_eq!(
        action(
            r#"gate = true
message = "re-read the scope""#
        )
        .kind()
        .unwrap(),
        ActionKind::Run {
            command: None,
            gate: true,
            message: Some("re-read the scope"),
        }
    );
}

/// `message` is optional too, same as a `run` gate — the runner falls back to
/// its default advice.
#[test]
fn a_runless_gate_needs_no_message() {
    assert_eq!(
        action("gate = true").kind().unwrap(),
        ActionKind::Run {
            command: None,
            gate: true,
            message: None,
        }
    );
}

#[test]
fn action_without_a_kind_is_rejected() {
    let err = action(r#"title = "x""#).kind().unwrap_err();
    assert!(err.contains("declares no kind"), "{err}");
}

/// `gate = false` alone (the default) is the same "nothing to do" shape as no
/// fields at all: no command to run, and no unconditional refusal either.
#[test]
fn a_bare_gate_false_action_is_rejected() {
    let err = action("gate = false").kind().unwrap_err();
    assert!(err.contains("declares no kind"), "{err}");
}

#[test]
fn action_with_two_kinds_is_rejected() {
    let err = action(
        r#"show = "a"
run = "b""#,
    )
    .kind()
    .unwrap_err();
    assert!(err.contains("more than one"), "{err}");
}

#[test]
fn modifiers_belonging_to_another_kind_are_rejected() {
    for toml in [
        "show = \"a\"\ngate = true",
        "show = \"a\"\ntitle = \"t\"",
        "show = \"a\"\nmessage = \"m\"",
        "list = \"@wip\"\ngate = true",
        "run = \"true\"\ntitle = \"t\"",
        "gate = true\ntitle = \"t\"",
    ] {
        let err = action(toml).kind().unwrap_err();
        assert!(err.contains("does not apply"), "{toml}: {err}");
    }
}

// --- selection ---------------------------------------------------------

const TWO_HOOKS: &str = r#"
[[on.flip]]
when = 'into("done")'
actions = [{ run = "true", gate = true }]

[[on.flip]]
when = 'into("wip") && status == "accepted"'
actions = [{ show = "starting" }]
"#;

#[test]
fn hooks_fire_only_on_the_matching_transition() {
    let config = config_with(TWO_HOOKS);
    let task = task_at("t", "accepted");
    let ready = Readiness::empty();

    let into_done = matching_flip_hooks(&config, &task, "done", &ready).unwrap();
    assert_eq!(into_done.len(), 1);
    assert_eq!(into_done.first().unwrap().when, r#"into("done")"#);

    let into_wip = matching_flip_hooks(&config, &task, "wip", &ready).unwrap();
    assert_eq!(into_wip.len(), 1);
    assert!(into_wip.first().unwrap().when.contains("accepted"));

    let into_cancelled = matching_flip_hooks(&config, &task, "cancelled", &ready).unwrap();
    assert!(into_cancelled.is_empty());
}

/// The task half of `when` reads the status being left, not the one arriving.
#[test]
fn when_reads_the_status_being_left() {
    let config = config_with(TWO_HOOKS);
    let ready = Readiness::empty();
    let wrong_source = task_at("t", "proposed");
    assert!(
        matching_flip_hooks(&config, &wrong_source, "wip", &ready)
            .unwrap()
            .is_empty()
    );
}

#[test]
fn empty_when_fires_on_every_flip() {
    let config = config_with("[[on.flip]]\nwhen = ''\nactions = [{ show = \"x\" }]\n");
    let fired = matching_flip_hooks(&config, &task_at("t", "wip"), "done", &Readiness::empty());
    assert_eq!(fired.unwrap().len(), 1);
}

#[test]
fn an_unevaluable_when_names_its_index() {
    let mut config = config_with("[[on.flip]]\nwhen = ''\nactions = [{ show = \"x\" }]\n");
    config
        .on
        .flip
        .first_mut()
        .unwrap()
        .when
        .push_str("no_such_field == 1");
    let err = matching_flip_hooks(&config, &task_at("t", "wip"), "done", &Readiness::empty())
        .unwrap_err();
    assert_eq!(err.index, 0);
    assert!(err.to_string().contains("on.flip[0].when"), "{err}");
}

#[test]
fn no_hooks_means_nothing_fires() {
    let config = Config::default();
    let fired = matching_flip_hooks(&config, &task_at("t", "wip"), "done", &Readiness::empty());
    assert!(fired.unwrap().is_empty());
}

// --- load-time validation ----------------------------------------------

/// Every problem `validate_flip_hooks` reports, each from a config that is
/// otherwise well-formed.
#[test]
fn validation_reports_each_problem() {
    let cases = [
        (
            "[[on.flip]]\nwhen = 'into(\"shipped\")'\nactions = [{ show = \"x\" }]\n",
            "names no configured status",
        ),
        (
            "[[on.flip]]\nwhen = 'no_such_field == 1'\nactions = [{ show = \"x\" }]\n",
            "on.flip[0].when",
        ),
        ("[[on.flip]]\nwhen = ''\nactions = []\n", "no actions"),
        (
            "[[on.flip]]\nwhen = ''\nactions = [{ list = \"@ghost\" }]\n",
            "on.flip[0].actions[0].list",
        ),
        (
            "[[on.flip]]\nwhen = ''\nactions = [{ title = \"x\" }]\n",
            "on.flip[0].actions[0]",
        ),
    ];
    for (toml, expected) in cases {
        // parse_config, not a bare deserialize: the shipped legend has to be in
        // place for the `into(...)` name check to have anything to check against.
        let problems = validate_flip_hooks(&config_with(toml)).join("; ");
        assert!(problems.contains(expected), "{toml} -> {problems}");
    }
}

/// A valid hook against the shipped legend passes both the module check and the
/// whole-config check `mt check` runs.
#[test]
fn a_valid_hook_passes_validation() {
    let config = config_with(
        "[views.wip]\nfilter = 'status == \"wip\"'\n\
         [[on.flip]]\nwhen = 'into(\"done\") && !has_tag(\"chore\")'\n\
         actions = [{ list = \"@wip\", title = \"T\" }, { run = \"true\", gate = true }]\n",
    );
    assert!(validate_flip_hooks(&config).is_empty());
    assert!(config.validate().is_ok());
}

/// A run-less gate — `gate = true` and `message`, no `run` — is a valid hook:
/// the config the task's Scope shows as the motivating example.
#[test]
fn a_runless_gate_passes_validation() {
    let config = config_with(
        "[[on.flip]]\nwhen = 'into(\"wip\") && stale(30)'\n\
         actions = [{ gate = true, message = \"re-read the scope\" }]\n",
    );
    assert!(validate_flip_hooks(&config).is_empty());
    assert!(config.validate().is_ok());
}

/// An empty legend unconstrains `into(...)` rather than rejecting every name.
#[test]
fn an_empty_legend_accepts_any_into_name() {
    let config = config_with(
        "[tasks]\nstatus = []\n[[on.flip]]\nwhen = 'into(\"whatever\")'\n\
         actions = [{ show = \"x\" }]\n",
    );
    assert!(validate_flip_hooks(&config).is_empty());
}

/// A malformed hook is a load error, so no command ever reaches the run path.
#[test]
fn a_malformed_hook_fails_config_validation() {
    let config =
        config_with("[[on.flip]]\nwhen = 'into(\"nope\")'\nactions = [{ show = \"x\" }]\n");
    assert!(config.validate().is_err());
}
