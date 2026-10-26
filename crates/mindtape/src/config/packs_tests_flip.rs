//! `[[on.flip]]` merges by name across packs and layers like a rule array: an
//! ancestor's or a pack's gate fires beside the project's, a later entry with
//! the same `name` replaces an earlier one in place, `disable` drops one, and
//! an unnamed entry appends and can't be replaced.
#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use crate::config::Config;
use crate::config::layers::load_config_layered_from;
use crate::config::pack_registry::{RuleOrigin, RuleOriginEntry};
use crate::test_support::{home_and_project, write_file};

/// An `[[on.flip]]` gate named `name` (or unnamed when empty), firing on
/// `when` and refusing with `message`.
fn hook(name: &str, when: &str, message: &str) -> String {
    let name = if name.is_empty() {
        String::new()
    } else {
        format!("name = \"{name}\"\n")
    };
    format!(
        "[[on.flip]]\n{name}when = '{when}'\n\
         actions = [{{ gate = true, message = \"{message}\" }}]\n"
    )
}

/// The `[[on.flip]]` gates `named` as `(name, when, message)` triples.
fn hooks(named: &[(&str, &str, &str)]) -> String {
    named
        .iter()
        .map(|(name, when, message)| hook(name, when, message))
        .collect()
}

/// An ancestor's `stale-wip` gate saying `message`, then its `close` gate.
fn ancestor_hooks(message: &str) -> String {
    hooks(&[
        ("stale-wip", "into(\"wip\")", message),
        ("close", "into(\"done\")", "close gate"),
    ])
}

/// The merged config of a project with `ancestor` as its parent folder's
/// `defaults.toml` and `project` as its `config.toml`, with the origins.
fn load_with(
    ancestor: &str,
    project: &str,
) -> (
    Result<Config, crate::config::ConfigError>,
    Vec<RuleOriginEntry>,
) {
    let (_tmp, home, root) = home_and_project();
    write_file(&home, ".mindtape/defaults.toml", ancestor);
    write_file(&root, ".mindtape/config.toml", project);
    match load_config_layered_from(&root, Some(&home), None) {
        Ok((config, _, origins)) => (Ok(config), origins),
        Err(err) => (Err(err), Vec::new()),
    }
}

/// `(name, message)` of each merged flip hook's first action.
fn summary(config: &Config) -> Vec<(Option<String>, Option<String>)> {
    (config.on.flip.iter())
        .map(|hook| (hook.name.clone(), hook.actions[0].message.clone()))
        .collect()
}

fn named(name: &str, message: &str) -> (Option<String>, Option<String>) {
    (Some(name.to_owned()), Some(message.to_owned()))
}

#[test]
fn an_ancestors_gate_fires_beside_the_projects_own() {
    let (config, _) = load_with(
        &hook("stale-wip", "into(\"wip\")", "re-read the scope"),
        &hook("", "into(\"done\")", "run the checks"),
    );

    assert_eq!(
        summary(&config.unwrap()),
        [
            named("stale-wip", "re-read the scope"),
            (None, Some("run the checks".to_owned()))
        ]
    );
}

#[test]
fn a_later_entry_with_the_same_name_replaces_the_earlier_one_in_place() {
    let ancestor = ancestor_hooks("from the ancestor");

    let (config, _) = load_with(
        &ancestor,
        &hook("stale-wip", "into(\"wip\")", "from the project"),
    );

    assert_eq!(
        summary(&config.unwrap()),
        [
            named("stale-wip", "from the project"),
            named("close", "close gate")
        ]
    );
}

#[test]
fn disable_drops_a_named_flip_entry() {
    let ancestor = ancestor_hooks("re-read the scope");

    let (config, _) = load_with(&ancestor, "disable = [\"stale-wip\"]\n");

    assert_eq!(summary(&config.unwrap()), [named("close", "close gate")]);
}

#[test]
fn an_unnamed_entry_appends_and_is_never_replaced() {
    let same = hook("", "into(\"done\")", "same");

    let (config, _) = load_with(&same, &same);

    assert_eq!(config.unwrap().on.flip.len(), 2);
}

#[test]
fn two_entries_of_one_file_sharing_a_name_are_a_config_error() {
    let twice = hooks(&[
        ("close", "into(\"done\")", "a"),
        ("close", "into(\"cancelled\")", "b"),
    ]);

    let (config, _) = load_with("", &twice);

    let err = config.unwrap_err().to_string();
    assert!(err.contains("on.flip[1]: duplicate name 'close'"), "{err}");
}

#[test]
fn a_gate_is_labelled_by_its_name_or_its_position() {
    let (_, origins) = load_with(
        &hook("stale-wip", "into(\"wip\")", "a"),
        &hook("", "into(\"done\")", "b"),
    );

    let labels: Vec<&str> = origins
        .iter()
        .filter(|entry| entry.label.starts_with("on.flip"))
        .map(|entry| entry.label.as_str())
        .collect();
    assert_eq!(labels, ["on.flip.stale-wip", "on.flip[1]"]);
    assert!(matches!(
        origins.last().unwrap().origin,
        RuleOrigin::Layer(_)
    ));
}

#[test]
fn a_packs_gate_applies_beside_the_projects_and_the_project_replaces_it_by_name() {
    let (_tmp, home, root) = home_and_project();
    write_file(
        &home,
        "team.toml",
        hook("stale-wip", "into(\"wip\")", "from the pack"),
    );
    write_file(
        &root,
        ".mindtape/config.toml",
        format!(
            "extends = [\"../team.toml\"]\n{}",
            hooks(&[
                ("close", "into(\"done\")", "close gate"),
                ("stale-wip", "into(\"wip\")", "from the project"),
            ])
        ),
    );

    let config = load_config_layered_from(&root, Some(&home), None)
        .unwrap()
        .0;

    assert_eq!(
        summary(&config),
        [
            named("stale-wip", "from the project"),
            named("close", "close gate")
        ]
    );
}
