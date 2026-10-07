//! End-to-end proof that the embedded `mindtape-tasks` pack, extended alone
//! (no local rules layered over it), enforces the task practice it ships:
//! a well-formed six-section body passes every header/content rule the pack
//! declares, and an empty body — the task's own Problem step — is refused.
//! Split out of `config/packs_tests.rs`, which was already at the linecop
//! cap, per `CONTRIBUTING.md`'s `just fix-check`/ejectest convention.
#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use crate::body::content_gate::compile_content_rules;
use crate::body::gate::compile_rules;
use crate::body::{BodyRules, check_body_rules};
use crate::config::layers::load_config_layered_from;
use crate::test_support::{bare_task, home_and_project};

/// Build the typed [`crate::config::Config`] a project whose ONLY
/// `.mindtape/config.toml` content is `extends = ["mindtape-tasks"]` would
/// load — against the real [`super::EMBEDDED_PACKS`], not a fixture
/// registry, so this proves the pack actually shipped, not just that the
/// merge mechanism works.
fn config_extending_the_real_pack() -> (tempfile::TempDir, crate::config::Config) {
    let (tmp, home, project) = home_and_project();
    std::fs::write(
        project.join(".mindtape").join("config.toml"),
        "extends = [\"mindtape-tasks\"]\n",
    )
    .unwrap();

    let (config, _sources, _origins) =
        load_config_layered_from(&project, Some(&home), None).unwrap();
    (tmp, config)
}

/// Every header/content violation `check_body_rules` reports against
/// `source`, under `config`'s compiled `[[tasks.body.headers.rules]]` and
/// `[[tasks.body.content.rules]]`.
fn violations_for(config: &crate::config::Config, source: &str) -> Vec<crate::validate::Violation> {
    let headers = compile_rules(&config.tasks.body.headers.rules).unwrap();
    let content = compile_content_rules(&config.tasks.body.content.rules).unwrap();
    let rules = BodyRules {
        headers: &headers,
        content: &content,
    };
    check_body_rules(&bare_task("ship-it"), source, rules, config)
}

/// A body carrying every section the pack's practice asks for, in the order
/// it asks for it, each with its markers — the shape `mindtape-tasks`
/// exists to require — passes clean under the pack alone.
#[test]
fn a_well_formed_body_passes_under_the_pack_alone() {
    let (_tmp, config) = config_extending_the_real_pack();
    let source = "\
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

    let violations = violations_for(&config, source);
    assert!(violations.is_empty(), "{violations:?}");
}

/// The task's own Problem step: an empty body is refused under the pack
/// alone, with no local rule involved.
#[test]
fn an_empty_body_is_refused_under_the_pack_alone() {
    let (_tmp, config) = config_extending_the_real_pack();

    let violations = violations_for(&config, "");
    assert!(
        !violations.is_empty(),
        "an empty body must breach the pack's own section rules"
    );
}
