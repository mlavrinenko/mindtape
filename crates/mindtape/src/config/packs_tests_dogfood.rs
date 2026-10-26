//! `mindtape:tasks` declaring the `kind` field and the comment policy its own
//! rules read, and `mindtape:dogfood`: its four rules, its `section-allowlist`
//! replacing the tasks pack's by name, and the drift test that keeps the two
//! allowlists one `Dogfood` apart.
#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use super::tests_budget::load;
use crate::config::Config;
use crate::config::comments::CommentsPolicy;
use crate::config::pack_registry::EMBEDDED_PACKS;
use crate::config::tasks::FieldExample;

fn header_names(config: &Config) -> Vec<Option<&str>> {
    let rules = &config.tasks.body.headers.rules;
    rules.iter().map(|rule| rule.name.as_deref()).collect()
}

fn allowlist_index(config: &Config) -> usize {
    header_names(config)
        .iter()
        .position(|name| *name == Some("section-allowlist"))
        .unwrap()
}

/// The embedded pack `name`, parsed.
fn pack(name: &str) -> toml::Table {
    let (_, text) = EMBEDDED_PACKS
        .iter()
        .find(|(pack, _)| *pack == name)
        .unwrap();
    text.parse().unwrap()
}

/// The `section-allowlist` entry of the pack's `[[tasks.body.headers.rules]]`.
fn allowlist(name: &str) -> toml::Table {
    let pack = pack(name);
    let rules = pack["tasks"]["body"]["headers"]["rules"]
        .as_array()
        .unwrap();
    let rule = rules
        .iter()
        .find(|rule| rule["name"].as_str() == Some("section-allowlist"))
        .unwrap();
    rule.as_table().unwrap().clone()
}

#[test]
fn the_tasks_pack_declares_the_kind_its_rules_read() {
    let config = load("extends = [\"mindtape:tasks\"]\n");
    let kind = &config.tasks.fields["kind"];
    assert_eq!(
        kind.values,
        [
            "feat",
            "fix",
            "refactor",
            "docs",
            "test",
            "dev",
            "milestone"
        ]
    );
    assert_eq!(kind.example, Some(FieldExample::One("test".to_owned())));
    assert_eq!(config.tasks.comments.policy, CommentsPolicy::Forbid);
}

#[test]
fn the_dogfood_pack_ships_four_rules_and_the_allowlist() {
    let config = load("extends = [\"mindtape:dogfood\"]\n");
    let names = header_names(&config);
    for name in [
        "section-allowlist",
        "one-dogfood-section",
        "dogfood-after-out-of-scope",
    ] {
        assert!(names.contains(&Some(name)), "{names:?}");
    }
    let content = &config.tasks.body.content.rules;
    let content: Vec<_> = content
        .iter()
        .filter_map(|rule| rule.name.as_deref())
        .collect();
    assert_eq!(content, ["dogfood-non-empty", "dogfood-word-cap"]);
}

#[test]
fn listed_after_the_tasks_pack_the_dogfood_allowlist_replaces_it_in_place() {
    let tasks = load("extends = [\"mindtape:tasks\"]\n");
    let both = load("extends = [\"mindtape:tasks\", \"mindtape:dogfood\"]\n");
    assert_eq!(allowlist_index(&tasks), allowlist_index(&both));
    let rule = &both.tasks.body.headers.rules[allowlist_index(&both)];
    assert!(rule.message.as_deref().unwrap().contains("Dogfood"));
    assert_eq!(
        header_names(&both)
            .iter()
            .filter(|name| **name == Some("section-allowlist"))
            .count(),
        1
    );
}

#[test]
fn the_dogfood_allowlist_differs_from_the_tasks_one_by_dogfood_alone() {
    let (mut tasks, mut dogfood) = (allowlist("tasks"), allowlist("dogfood"));
    let allowed = |rule: &mut toml::Table| {
        let allowed = rule.remove("allowed").unwrap();
        allowed.as_array().unwrap()[0]["match"]
            .as_str()
            .unwrap()
            .to_owned()
    };
    let (tasks_allowed, dogfood_allowed) = (allowed(&mut tasks), allowed(&mut dogfood));
    assert_eq!(dogfood_allowed.replace("|Dogfood", ""), tasks_allowed);
    assert!(dogfood_allowed.ends_with("|Dogfood)$"));
    // Everything else but the message, which names Dogfood, is the same.
    tasks.remove("message");
    dogfood.remove("message");
    assert_eq!(tasks, dogfood);
}
