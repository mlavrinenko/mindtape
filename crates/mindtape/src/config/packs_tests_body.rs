//! End-to-end proof that the embedded `mindtape:tasks` pack, extended alone
//! (no local rules layered over it), enforces the task practice it ships:
//! a well-formed body passes every header/content rule the pack declares, an
//! empty body — the task's own AS-IS step — is refused, and the AS-IS,
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

/// The well-formed body with `from` replaced by `to` breaches `rule`.
fn assert_changes_breach(from: &str, to: &str, rule: &str) {
    let (_tmp, config) = config_extending_the_real_pack();
    let source = as_is().replace(from, to);
    let violations = violations_for(&config, &source);
    assert_breaches(&violations, rule);
}

fn assert_task_breaches(config: &Config, kind: &str, source: &str, rule: &str) {
    assert_breaches(&violations_for_task(config, &task(kind, &[]), source), rule);
}

/// A body carrying every section the pack's practice asks for, in the order
/// it asks for it, each with its markers — the shape `mindtape:tasks`
/// exists to require.
const WELL_FORMED: &str = "\
== AS-IS

+ Run the command.
+ 🔴 It does the wrong thing.

== TO-BE

- 🟢 It does the right thing instead.

== Changes

- `src/thing.rs`: fix the wrong thing.

== Out of scope

- Everything not touched by the fix above.
";

/// The well-formed body, owned, so a test can splice into it.
fn as_is() -> String {
    WELL_FORMED.to_owned()
}

/// `as_is()` with `heading` and `body` spliced in before Out of scope.
fn with_section(heading: &str, body: &str) -> String {
    as_is().replace(
        "== Out of scope",
        &format!("== {heading}\n\n{body}\n\n== Out of scope"),
    )
}

/// `as_is()` with `text` inserted before the `heading` section.
fn before(heading: &str, text: &str) -> String {
    as_is().replace(&format!("== {heading}"), &format!("{text}\n\n== {heading}"))
}

/// The shape a docs task needs beside the shared sections.
fn docs_body() -> String {
    before(
        "Changes",
        "== Source of truth\n\n- `src/thing.rs`: the code.",
    )
}

/// The shape a feat or decision task needs beside the shared sections.
fn design_body() -> String {
    with_section("Alternatives", "- Another design, which lost.")
}

/// The shape a fix task needs beside the shared sections.
fn fix_body() -> String {
    before("TO-BE", "== Environment\n\n- Linux, `mt` from `main`.")
}

/// `body` with a fenced block after `anchor`.
fn with_fence(body: &str, anchor: &str, command: &str) -> String {
    body.replace(anchor, &format!("{anchor}\n\n```sh\n{command}\n```"))
}

fn with_as_is_fence(body: &str) -> String {
    with_fence(body, "+ 🔴 It does the wrong thing.", "mt thing")
}

fn with_to_be_fence(body: &str) -> String {
    with_fence(
        body,
        "- 🟢 It does the right thing instead.",
        "mt thing --right",
    )
}

fn fenced(body: &str) -> String {
    with_to_be_fence(&with_as_is_fence(body))
}

#[test]
fn a_well_formed_body_passes_under_the_pack_alone() {
    let (_tmp, config) = config_extending_the_real_pack();

    let violations = violations_for(&config, WELL_FORMED);
    assert!(violations.is_empty(), "{violations:?}");
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
fn changes_bullets_must_start_with_one_backticked_path() {
    assert_changes_breach(
        "- `src/thing.rs`: fix the wrong thing.",
        "- `src/thing.rs`: fix the wrong thing.\n- and cover it with a test.",
        "body.content.rules.changes-bullets-start-with-a-path",
    );
}

#[test]
fn changes_must_not_be_empty() {
    assert_changes_breach(
        "- `src/thing.rs`: fix the wrong thing.",
        "",
        "body.content.rules.changes-non-empty",
    );
}

#[test]
fn as_is_takes_the_content_rules() {
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
fn a_problem_heading_is_refused() {
    let (_tmp, config) = config_extending_the_real_pack();
    let source = WELL_FORMED.replace("== AS-IS", "== Problem");

    let violations = violations_for(&config, &source);
    assert_breaches(&violations, "body.headers.rules.section-allowlist");
    let refusal = violations
        .iter()
        .find(|violation| violation.field == "body.headers.rules.section-allowlist")
        .unwrap();
    assert!(
        refusal.message.contains("AS-IS"),
        "the refusal must name AS-IS: {}",
        refusal.message
    );
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
        "body.headers.rules.alternatives-for-design",
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
        "body.headers.rules.alternatives-only-for-design",
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

    let violations = violations_for_task(&config, &task("fix", &[]), &fix_body());
    let names = breached(&violations);
    assert!(!names.contains(&"body.content.rules.as-is-has-a-repro"));
    assert!(!names.contains(&"body.content.rules.to-be-has-an-example"));
}

#[test]
fn examples_on_only_require_the_fences_their_kind_carries() {
    let (_tmp, config) = config_with(false, true);

    for (kind, source) in [
        ("refactor", as_is()),
        ("docs", docs_body()),
        ("decision", design_body()),
    ] {
        let violations = violations_for_task(&config, &task(kind, &[]), &source);
        assert!(violations.is_empty(), "{kind}: {violations:?}");
    }

    let fix = violations_for_task(&config, &task("fix", &[]), &fix_body());
    assert_breaches(&fix, "body.content.rules.as-is-has-a-repro");
    let feat = violations_for_task(&config, &task("feat", &[]), &design_body());
    assert_breaches(&feat, "body.content.rules.to-be-has-an-example");
    assert!(!breached(&feat).contains(&"body.content.rules.as-is-has-a-repro"));
    let dev = violations_for_task(&config, &task("dev", &[]), &as_is());
    assert_breaches(&dev, "body.content.rules.to-be-has-an-example");
    assert!(!breached(&dev).contains(&"body.content.rules.as-is-has-a-repro"));
}

#[test]
fn a_fix_needs_both_fences_and_a_feat_or_dev_needs_the_to_be_one() {
    let (_tmp, config) = config_with(false, true);

    assert_task_breaches(
        &config,
        "fix",
        &with_as_is_fence(&fix_body()),
        "body.content.rules.to-be-has-an-example",
    );

    let fix = violations_for_task(&config, &task("fix", &[]), &fenced(&fix_body()));
    assert!(fix.is_empty(), "{fix:?}");

    for (kind, source) in [
        ("feat", with_to_be_fence(&design_body())),
        ("dev", with_to_be_fence(&as_is())),
    ] {
        let violations = violations_for_task(&config, &task(kind, &[]), &source);
        assert!(violations.is_empty(), "{kind}: {violations:?}");
    }
}

#[test]
fn the_example_waivers_follow_the_kinds_they_apply_to() {
    let (_tmp, config) = config_with(false, true);

    let waived = with_to_be_fence(&fix_body());
    let violations = violations_for_task(&config, &task("fix", &["no-repro"]), &waived);
    assert!(violations.is_empty(), "{violations:?}");

    let violations = violations_for_task(&config, &task("fix", &["no-repro"]), &fix_body());
    assert_breaches(&violations, "body.content.rules.to-be-has-an-example");

    for (kind, source) in [("feat", design_body()), ("dev", as_is())] {
        let violations = violations_for_task(&config, &task(kind, &["no-ui-ux"]), &source);
        assert!(violations.is_empty(), "{kind}: {violations:?}");
    }

    let violations = violations_for_task(&config, &task("fix", &["no-ui-ux"]), &fix_body());
    assert_breaches(&violations, "body.content.rules.as-is-has-a-repro");
}
