//! The `mindtape:tweaks` and `mindtape:commits` packs, loaded through the
//! real registry: each alone over the floor, beside `mindtape:budget`, and the
//! rules and path sets they ship. How the rules judge a commit is the CLI
//! tests' business (`commit_packs_e2e.rs`).
#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use super::tests_budget::load;
use crate::config::Config;
use crate::config::commits::CommitRuleSpec;

fn commit_rule<'a>(config: &'a Config, name: &str) -> &'a CommitRuleSpec {
    config
        .commits
        .rules
        .iter()
        .find(|rule| rule.name.as_deref() == Some(name))
        .unwrap_or_else(|| panic!("no commit rule {name}"))
}

fn has_task_rule(config: &Config, name: &str) -> bool {
    config
        .tasks
        .rules
        .iter()
        .any(|rule| rule.name.as_deref() == Some(name))
}

#[test]
fn tweaks_ships_the_task_rule_and_the_commit_rule() {
    let config = load("extends = [\"mindtape:tweaks\"]\n");
    assert!(has_task_rule(&config, "tweak-lands-done"));
    let rule = commit_rule(&config, "tweak-stays-one-file");
    assert_eq!(rule.assertion, r#"count(files, "@source") <= 1"#);
    assert_eq!(config.dicts["source"].values, ["**"]);
}

#[test]
fn tweaks_and_budget_agree_on_the_source_set() {
    let config = load("extends = [\"mindtape:tweaks\", \"mindtape:budget\"]\n");
    assert_eq!(config.dicts["source"].values, ["**"]);
    assert!(has_task_rule(&config, "judgment-stays-small"));
    commit_rule(&config, "tweak-stays-one-file");
}

#[test]
fn commits_ships_both_commit_rules() {
    let config = load("extends = [\"mindtape:commits\"]\n");
    let changelog = commit_rule(&config, "changelog-with-product-change");
    let when = changelog.when.as_deref().unwrap();
    assert!(when.contains(r#"count(files, "@product") >= 1"#), "{when}");
    assert!(when.contains("fixup|squash|amend"), "{when}");
    assert!(when.contains("Changelog: none"), "{when}");
    assert_eq!(changelog.assertion, r#"matches(files, "^CHANGELOG[.]md$")"#);
    let refs = commit_rule(&config, "refs-task-is-started");
    assert_eq!(refs.assertion, r#"staged(status == "wip" || terminal)"#);
    assert_eq!(config.dicts["product"].values, ["**"]);
}

#[test]
fn a_projects_product_set_replaces_the_packs() {
    let config = load(
        "extends = [\"mindtape:commits\"]\n\n[dicts.product]\nvalues = [\"crates/**\", \"!**/*_tests.rs\"]\n",
    );
    assert_eq!(
        config.dicts["product"].values,
        ["crates/**", "!**/*_tests.rs"]
    );
}

#[test]
fn every_flagship_pack_adopts_together() {
    let config = load(
        "extends = [\"mindtape:tasks\", \"mindtape:wip-caps\", \"mindtape:budget\", \
         \"mindtape:tweaks\", \"mindtape:commits\"]\n",
    );
    commit_rule(&config, "tweak-stays-one-file");
    commit_rule(&config, "refs-task-is-started");
    commit_rule(&config, "flip-with-work");
}
