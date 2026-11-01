//! End-to-end proof that the embedded `mindtape:tasks` pack, extended alone
//! (no local rules layered over it), enforces the task practice it ships:
//! a well-formed body passes every header/content rule the pack declares, an
//! empty body — the task's own Problem step — is refused, and the AS-IS,
//! decision, statefulness and example rules follow the task's properties.
//! Split out of `config/packs_tests.rs`, which was already at the linecop
//! cap, per `CONTRIBUTING.md`'s `just fix-check`/ejectest convention.
#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use crate::body::content_gate::compile_content_rules;
use crate::body::gate::compile_rules;
use crate::body::{BodyRules, check_body_rules};
use crate::config::Config;
use crate::config::layers::load_config_layered_from;
use crate::model::Task;
use crate::test_support::{bare_task, home_and_project};
use crate::validate::Violation;

/// Build the typed [`Config`] a project whose ONLY `.mindtape/config.toml`
/// content is `extends` would load — against the real
/// [`crate::config::pack_registry::EMBEDDED_PACKS`], not a fixture registry,
/// so this proves the pack actually shipped, not just that the merge
/// mechanism works.
fn config_extending(extends: &str) -> (tempfile::TempDir, Config) {
    let (tmp, home, project) = home_and_project();
    std::fs::write(project.join(".mindtape").join("config.toml"), extends).unwrap();

    let (config, _sources, _origins) =
        load_config_layered_from(&project, Some(&home), None).unwrap();
    (tmp, config)
}

fn config_extending_the_real_pack() -> (tempfile::TempDir, Config) {
    config_extending("extends = [\"mindtape:tasks\"]\n")
}

/// The pack with its optional rule groups switched on.
fn config_with(statefulness: bool, examples: bool) -> (tempfile::TempDir, Config) {
    config_extending(&format!(
        "extends = [{{ pack = \"mindtape:tasks\", with = {{ statefulness = {statefulness}, examples = {examples} }} }}]\n"
    ))
}

/// A task of `kind` carrying `tags`, for the rules that read either.
fn task(kind: &str, tags: &[&str]) -> Task {
    let mut task = bare_task("ship-it");
    task.kind = Some(kind.to_owned());
    task.tags = tags.iter().map(|tag| (*tag).to_owned()).collect();
    task
}

/// Every header/content violation `check_body_rules` reports against
/// `source`, under `config`'s compiled `[[body.headers.rules]]` and
/// `[[body.content.rules]]`.
fn violations_for_task(config: &Config, task: &Task, source: &str) -> Vec<Violation> {
    let headers = compile_rules(&config.tasks.body.headers.rules).unwrap();
    let content = compile_content_rules(&config.tasks.body.content.rules).unwrap();
    let rules = BodyRules {
        headers: &headers,
        content: &content,
    };
    check_body_rules(task, source, rules, config)
}

fn violations_for(config: &Config, source: &str) -> Vec<Violation> {
    violations_for_task(config, &bare_task("ship-it"), source)
}

/// Every `[[tasks.rules]]` breach for `task`, run the way `mt check` runs them.
fn task_rule_violations(config: &Config, task: &Task) -> Vec<Violation> {
    let mut out = Vec::new();
    crate::rules::check_all(std::slice::from_ref(task), config, &mut out);
    out
}

/// The rule names a body breaches.
fn breached(violations: &[Violation]) -> Vec<&str> {
    violations
        .iter()
        .map(|violation| violation.field.as_str())
        .collect()
}

fn assert_breaches(violations: &[Violation], rule: &str) {
    let names = breached(violations);
    assert!(names.contains(&rule), "wanted {rule}: {names:?}");
}

fn assert_body_breaches(config: &Config, source: &str, rule: &str) {
    assert_breaches(&violations_for(config, source), rule);
}

fn assert_task_breaches(config: &Config, kind: &str, source: &str, rule: &str) {
    assert_breaches(&violations_for_task(config, &task(kind, &[]), source), rule);
}

/// A body carrying every section the pack's practice asks for, in the order
/// it asks for it, each with its markers — the shape `mindtape:tasks`
/// exists to require.
const WELL_FORMED: &str = "\
== Problem

+ Run the command.
+ 🔴 It does the wrong thing.

== TO-BE

- 🟢 It does the right thing instead.

== Changes

- `src/thing.rs`: fix the wrong thing.

== Out of scope

- Everything not touched by the fix above.
";

/// The same body under the heading the transition accepts beside `Problem`.
fn as_is() -> String {
    WELL_FORMED.replace("== Problem", "== AS-IS")
}

/// `as_is()` with `heading` and `body` spliced in before Out of scope.
fn with_section(heading: &str, body: &str) -> String {
    as_is().replace(
        "== Out of scope",
        &format!("== {heading}\n\n{body}\n\n== Out of scope"),
    )
}

#[test]
fn a_well_formed_body_passes_under_the_pack_alone() {
    let (_tmp, config) = config_extending_the_real_pack();

    for source in [WELL_FORMED.to_owned(), as_is()] {
        let violations = violations_for(&config, &source);
        assert!(violations.is_empty(), "{violations:?}");
    }
}

#[test]
fn an_empty_body_is_refused_under_the_pack_alone() {
    let (_tmp, config) = config_extending_the_real_pack();

    let violations = violations_for(&config, "");
    assert!(
        !violations.is_empty(),
        "an empty body must breach the pack's own section rules"
    );
}

#[test]
fn as_is_takes_the_same_content_rules_as_problem() {
    let (_tmp, config) = config_extending_the_real_pack();
    let prose = as_is().replace(
        "+ Run the command.\n+ 🔴 It does the wrong thing.",
        "The command does the wrong thing.",
    );

    let violations = violations_for(&config, &prose);
    assert_breaches(&violations, "body.content.rules.as-is-is-numbered-steps");
    assert_breaches(&violations, "body.content.rules.as-is-marks-the-defect");
}

#[test]
fn as_is_is_unique_and_says_something() {
    let (_tmp, config) = config_extending_the_real_pack();
    let twice = with_section("AS-IS", "+ Again.\n+ 🔴 Again.");
    let empty = as_is().replace("+ Run the command.\n+ 🔴 It does the wrong thing.", "");

    assert_body_breaches(&config, &twice, "body.headers.rules.one-as-is-section");
    assert_body_breaches(&config, &empty, "body.content.rules.as-is-non-empty");
}

#[test]
fn as_is_keeps_the_section_order_and_word_cap() {
    let (_tmp, config) = config_extending_the_real_pack();
    let out_of_order = as_is().replace(
        "== AS-IS\n\n+ Run the command.\n+ 🔴 It does the wrong thing.\n\n== TO-BE\n\n- 🟢 It does the right thing instead.",
        "== TO-BE\n\n- 🟢 It does the right thing instead.\n\n== AS-IS\n\n+ Run the command.\n+ 🔴 It does the wrong thing.",
    );
    let padded = as_is().replace(
        "+ 🔴 It does the wrong thing.",
        &format!("+ 🔴 It does the wrong thing. {}", "word ".repeat(130)),
    );

    assert_body_breaches(
        &config,
        &out_of_order,
        "body.headers.rules.to-be-after-as-is",
    );
    assert_body_breaches(&config, &padded, "body.content.rules.as-is-word-cap");
}

#[test]
fn a_decision_carries_alternatives() {
    let (_tmp, config) = config_extending_the_real_pack();
    let source = with_section("Alternatives", "- Another design, which lost.");

    let violations = violations_for_task(&config, &task("decision", &[]), &source);
    assert!(violations.is_empty(), "{violations:?}");
}

#[test]
fn a_decision_without_alternatives_is_refused() {
    let (_tmp, config) = config_extending_the_real_pack();

    assert_task_breaches(
        &config,
        "decision",
        &as_is(),
        "body.headers.rules.alternatives-for-feat",
    );
}

#[test]
fn only_a_feat_or_decision_carries_alternatives() {
    let (_tmp, config) = config_extending_the_real_pack();
    let source = with_section("Alternatives", "- Another design, which lost.");

    assert_task_breaches(
        &config,
        "refactor",
        &source,
        "body.headers.rules.alternatives-only-for-feat",
    );
}

#[test]
fn the_rollback_policy_follows_statefulness() {
    let (_tmp, config) = config_extending_the_real_pack();
    let rollback = with_section("Rollback", "- Revert the commit.");
    let cases = [
        (
            "stateful",
            &as_is(),
            Some("body.headers.rules.rollback-for-stateful"),
        ),
        (
            "stateless",
            &rollback,
            Some("body.headers.rules.rollback-only-for-stateful"),
        ),
        ("stateful", &rollback, None),
        ("stateless", &as_is(), None),
    ];

    for (tag, source, expected) in cases {
        let violations = violations_for_task(&config, &task("dev", &[tag]), source);
        match expected {
            Some(rule) => assert_breaches(&violations, rule),
            None => assert!(violations.is_empty(), "{violations:?}"),
        }
    }
}

#[test]
fn the_statefulness_tag_rule_is_off_by_default() {
    let (_tmp, config) = config_extending_the_real_pack();

    let violations = task_rule_violations(&config, &task("dev", &[]));
    let names = breached(&violations);
    assert!(!names.contains(&"rules.statefulness-tag"), "{names:?}");
}

#[test]
fn turning_statefulness_on_requires_a_tag() {
    let (_tmp, config) = config_with(true, false);

    let untagged = task_rule_violations(&config, &task("dev", &[]));
    let tagged = task_rule_violations(&config, &task("dev", &["stateless"]));
    let decision = task_rule_violations(&config, &task("decision", &[]));
    assert_breaches(&untagged, "rules.statefulness-tag");
    assert!(!breached(&tagged).contains(&"rules.statefulness-tag"));
    assert!(!breached(&decision).contains(&"rules.statefulness-tag"));
}

#[test]
fn the_example_rules_are_off_by_default() {
    let (_tmp, config) = config_extending_the_real_pack();

    let violations = violations_for(&config, &as_is());
    let names = breached(&violations);
    assert!(!names.contains(&"body.content.rules.as-is-has-a-repro"));
    assert!(!names.contains(&"body.content.rules.to-be-has-an-example"));
}

#[test]
fn turning_examples_on_requires_a_fence_in_as_is_and_to_be() {
    let (_tmp, config) = config_with(false, true);

    let violations = violations_for(&config, &as_is());
    assert_breaches(&violations, "body.content.rules.as-is-has-a-repro");
    assert_breaches(&violations, "body.content.rules.to-be-has-an-example");

    let fenced = as_is().replace(
        "+ 🔴 It does the wrong thing.",
        "+ 🔴 It does the wrong thing.\n\n```sh\nmt thing\n```",
    );
    let fenced = fenced.replace(
        "- 🟢 It does the right thing instead.",
        "- 🟢 It does the right thing instead.\n\n```sh\nmt thing --right\n```",
    );
    let violations = violations_for(&config, &fenced);
    assert!(violations.is_empty(), "{violations:?}");
}

#[test]
fn no_repro_and_no_ui_ux_waive_the_example_rules() {
    let (_tmp, config) = config_with(false, true);

    let violations =
        violations_for_task(&config, &task("dev", &["no-repro", "no-ui-ux"]), &as_is());
    assert!(violations.is_empty(), "{violations:?}");
}
