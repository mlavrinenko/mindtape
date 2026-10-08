#![allow(clippy::unwrap_used)]

use super::{TargetValidator, Validators, accepts, validate_link_kinds};
use crate::config::{Config, parse_config};
use crate::graph::ready::Readiness;
use crate::link::{LinkKind, LinksConfig};
use crate::model::Task;
use crate::test_support::bare_task;

/// A custom kind with `validate` set and everything else at its default.
fn kind(name: &str, validate: TargetValidator) -> LinkKind {
    LinkKind {
        name: name.to_owned(),
        validate,
        ..LinkKind::default()
    }
}

/// A links config holding one custom kind.
fn one(kind_spec: LinkKind) -> LinksConfig {
    LinksConfig {
        kinds: vec![kind_spec],
    }
}

/// Whether `validator` accepts `value`, with no sibling pattern. Every bundled
/// validator except `regex` goes through this.
fn takes(validator: TargetValidator, value: &str) -> bool {
    accepts(validator, value, None)
}

/// Assert one validator's accept/reject pair through the live config path
/// (`Validators::problem`), not just the raw dispatch.
fn assert_pair(validator: TargetValidator, good: &str, bad: &str) {
    let config = one(kind("artifact", validator));
    let validators = Validators::build(&config);
    assert_eq!(
        validators.problem("artifact", good),
        None,
        "`{}` should accept `{good}`",
        validator.name()
    );
    let problem = validators
        .problem("artifact", bad)
        .unwrap_or_else(|| panic!("`{}` should reject `{bad}`", validator.name()));
    assert!(problem.contains("artifact"), "names the kind: {problem}");
    assert!(
        problem.contains(validator.name()),
        "names the validator: {problem}"
    );
    assert!(problem.contains(bad), "names the value: {problem}");
}

#[test]
fn url_validator_accepts_and_rejects() {
    assert_pair(
        TargetValidator::Url,
        "https://example.com/pr/7?x=1#h",
        "nope",
    );
    assert!(takes(TargetValidator::Url, "mailto:dev@example.com"));
    // No scheme, a space, and a non-alphabetic scheme start: all rejected.
    assert!(!takes(TargetValidator::Url, "example.com/pr/7"));
    assert!(!takes(TargetValidator::Url, "https://exa mple.com"));
    assert!(!takes(TargetValidator::Url, "1http://example.com"));
    assert!(!takes(TargetValidator::Url, "https:"));
}

#[test]
fn git_sha_validator_accepts_and_rejects() {
    assert_pair(TargetValidator::GitSha, "0123abc", "0123ab");
    // Abbreviated SHA-1 (7) through full SHA-256 (64) inclusive.
    assert!(takes(TargetValidator::GitSha, &"a".repeat(40)));
    assert!(takes(TargetValidator::GitSha, &"a".repeat(64)));
    assert!(!takes(TargetValidator::GitSha, &"a".repeat(65)));
    assert!(!takes(TargetValidator::GitSha, "0123abg"));
}

#[test]
fn uuid_validator_accepts_and_rejects() {
    assert_pair(
        TargetValidator::Uuid,
        "8f14e45f-ea6d-4b1e-9f6e-2c9f8f2a1b3c",
        "8f14e45f-not-a-uuid",
    );
}

#[test]
fn any_validator_accepts_anything_but_emptiness() {
    assert_pair(TargetValidator::Any, "RFC-1149 // whatever", "");
}

#[test]
fn regex_validator_uses_the_sibling_pattern() {
    let config = one(LinkKind {
        regex: Some("^JIRA-[0-9]+$".to_owned()),
        ..kind("ticket", TargetValidator::Regex)
    });
    let validators = Validators::build(&config);
    assert_eq!(validators.problem("ticket", "JIRA-42"), None);
    let problem = validators.problem("ticket", "JIRA-x").unwrap();
    assert!(problem.contains("regex"), "names the validator: {problem}");
    assert_eq!(
        problem,
        "'ticket' edge target 'JIRA-x' is not a valid 'regex'"
    );
}

/// `validate = "regex"` with no sibling `regex` key is a CONFIG error, not a
/// silently permissive kind: caught at load, before any task is read.
#[test]
fn regex_without_pattern_is_a_config_error() {
    let err = parse_config("[[tasks.links.kinds]]\nname = \"ticket\"\nvalidate = \"regex\"\n")
        .expect("parses")
        .validate()
        .expect_err("regex without a pattern must fail validation");
    assert!(err.contains("tasks.links.kinds.ticket"), "{err}");
    assert!(err.contains("regex"), "{err}");
}

#[test]
fn uncompilable_regex_is_a_config_error() {
    let config = Config {
        tasks: crate::config::TasksConfig {
            links: one(LinkKind {
                regex: Some("(unclosed".to_owned()),
                ..kind("ticket", TargetValidator::Regex)
            }),
            ..crate::config::TasksConfig::default()
        },
        ..Config::default()
    };
    let problems = validate_link_kinds(&config);
    assert_eq!(problems.len(), 1, "{problems:?}");
    assert!(problems.first().unwrap().contains("invalid pattern"));
}

/// A `regex` key on any other validator is dead config — flagged rather than
/// silently ignored.
#[test]
fn regex_key_on_another_validator_is_a_config_error() {
    let config = Config {
        tasks: crate::config::TasksConfig {
            links: one(LinkKind {
                regex: Some("^x$".to_owned()),
                ..kind("commit", TargetValidator::GitSha)
            }),
            ..crate::config::TasksConfig::default()
        },
        ..Config::default()
    };
    let problems = validate_link_kinds(&config);
    assert_eq!(problems.len(), 1, "{problems:?}");
    assert!(problems.first().unwrap().contains("git-sha"));
}

#[test]
fn well_formed_kinds_have_no_config_problems() {
    let config = parse_config(
        "[[tasks.links.kinds]]\nname = \"commit\"\nvalidate = \"git-sha\"\n\
         [[tasks.links.kinds]]\nname = \"ticket\"\nvalidate = \"regex\"\nregex = \"^T-[0-9]+$\"\n",
    )
    .unwrap();
    assert!(validate_link_kinds(&config).is_empty());
    assert!(config.validate().is_ok());
}

/// A `local-task-path` kind carries no shape check of its own — resolution
/// against the loaded task set is its check, so `Validators` never holds it.
#[test]
fn local_task_path_kinds_are_not_shape_checked() {
    let config = one(kind("child-of", TargetValidator::LocalTaskPath));
    let validators = Validators::build(&config);
    assert_eq!(validators.problem("child-of", "anything at all"), None);
    assert_eq!(validators.problem("depends-on", "b.typ"), None);
    assert!(takes(TargetValidator::LocalTaskPath, "b.typ"));
}

/// A pattern that failed to compile rejects every target rather than waving
/// them through — the failure stays loud even if config validation is skipped.
#[test]
fn broken_pattern_rejects_every_target() {
    let config = one(LinkKind {
        regex: Some("(unclosed".to_owned()),
        ..kind("ticket", TargetValidator::Regex)
    });
    let validators = Validators::build(&config);
    assert!(validators.problem("ticket", "anything").is_some());
}

/// `mt ls` never follows a non-task target: a kind pointing outside the backlog
/// cannot block, so the linking task still reads ready.
#[test]
fn readiness_never_follows_an_external_target() {
    let config = parse_config(
        "[[tasks.links.kinds]]\nname = \"discharged-by\"\nvalidate = \"url\"\nblocking = true\n",
    )
    .unwrap();
    let mut task: Task = bare_task("a");
    task.links = vec![crate::link::Link {
        task: "https://example.com/pr/7".to_owned(),
        kind: "discharged-by".to_owned(),
        note: None,
    }];
    let readiness = Readiness::compute(std::slice::from_ref(&task), &config);
    assert!(readiness.is_ready(&task), "external edge must not block");
}

#[test]
fn validator_names_round_trip_their_config_spelling() {
    for (validator, spelling) in [
        (TargetValidator::LocalTaskPath, "local-task-path"),
        (TargetValidator::Url, "url"),
        (TargetValidator::GitSha, "git-sha"),
        (TargetValidator::Uuid, "uuid"),
        (TargetValidator::Regex, "regex"),
        (TargetValidator::Any, "any"),
    ] {
        assert_eq!(validator.name(), spelling);
        let toml = format!("[[tasks.links.kinds]]\nname = \"k\"\nvalidate = \"{spelling}\"\n");
        let config = parse_config(&toml).unwrap();
        assert_eq!(
            config.tasks.links.kinds.first().unwrap().validate,
            validator
        );
    }
}
