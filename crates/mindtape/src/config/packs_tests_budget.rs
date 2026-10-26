//! The `mindtape:wip-caps` and `mindtape:budget` packs, loaded through the
//! real registry: their rules with each parameter's default and with a `with`
//! value, the estimate unit and path set `budget` sets, and the project's
//! own keys that merge over them.
#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use crate::config::Config;
use crate::config::layers::load_config_layered_from;
use crate::test_support::home_and_project;

/// The config a project holding `own` as its `.mindtape/config.toml` loads.
fn load(own: &str) -> Config {
    let (_tmp, home, project) = home_and_project();
    std::fs::write(project.join(".mindtape").join("config.toml"), own).unwrap();
    load_config_layered_from(&project, Some(&home), None)
        .unwrap()
        .0
}

/// The `(when, assert)` of the rule `name`, with a missing one as `""`.
fn rule(config: &Config, name: &str) -> (String, String) {
    let rule = config
        .tasks
        .rules
        .iter()
        .find(|rule| rule.name.as_deref() == Some(name))
        .unwrap_or_else(|| panic!("no rule {name}"));
    (
        rule.when.clone().unwrap_or_default(),
        rule.assertion.clone().unwrap_or_default(),
    )
}

#[test]
fn wip_caps_ships_both_caps_at_one() {
    let config = load("extends = [\"mindtape:wip-caps\"]\n");
    assert_eq!(
        rule(&config, "milestone-wip-cap"),
        (
            r#"status == "wip" && kind == "milestone""#.to_owned(),
            r#"count(status == "wip" && kind == "milestone") <= 1"#.to_owned()
        )
    );
    assert_eq!(
        rule(&config, "leaf-wip-cap"),
        (
            r#"status == "wip" && kind != "milestone""#.to_owned(),
            r#"count(status == "wip" && kind != "milestone") <= 1"#.to_owned()
        )
    );
}

#[test]
fn wip_caps_takes_each_cap_as_a_parameter() {
    let config = load(
        "extends = [{ pack = \"mindtape:wip-caps\", with = { milestones = 2, leaves = 3 } }]\n",
    );
    assert!(rule(&config, "milestone-wip-cap").1.ends_with("<= 2"));
    assert!(rule(&config, "leaf-wip-cap").1.ends_with("<= 3"));
}

#[test]
fn budget_ships_both_rules_with_their_defaults() {
    let config = load("extends = [\"mindtape:budget\"]\n");
    assert_eq!(
        rule(&config, "judgment-stays-small"),
        (
            r#"!terminal && kind != "milestone" && difficulty >= 3"#.to_owned(),
            "estimate.src-files <= 5".to_owned()
        )
    );
    assert_eq!(
        rule(&config, "src-budget"),
        (
            r#"status == "wip" && has(estimate.src-files)"#.to_owned(),
            r#"changed("staged+worktree", "@source") <= estimate.src-files"#.to_owned()
        )
    );
}

#[test]
fn budget_takes_the_cap_and_the_difficulty_floor_as_parameters() {
    let config = load(
        "extends = [{ pack = \"mindtape:budget\", with = { max-files = 8, min-difficulty = 4 } }]\n",
    );
    let (when, assertion) = rule(&config, "judgment-stays-small");
    assert!(when.ends_with("difficulty >= 4"), "{when}");
    assert_eq!(assertion, "estimate.src-files <= 8");
}

#[test]
fn budget_sizes_in_source_files_over_every_path() {
    let config = load("extends = [\"mindtape:budget\"]\n");
    assert_eq!(config.tasks.estimate.units, ["src-files"]);
    assert_eq!(config.dicts["source"].values, ["**"]);
    assert_eq!(config.tasks.estimate.queries["src-files"], "@source");
}

#[test]
fn a_projects_queries_and_source_set_merge_over_the_packs() {
    let config = load(
        "extends = [\"mindtape:budget\"]\n\n\
         [tasks.estimate]\nunits = [\"src-files\", \"hunks\"]\n\n\
         [tasks.estimate.queries]\nhunks = \"@source\"\n\n\
         [dicts.source]\nvalues = [\"!docs/**\"]\n",
    );
    assert_eq!(config.tasks.estimate.units, ["src-files", "hunks"]);
    assert_eq!(config.tasks.estimate.queries["src-files"], "@source");
    assert_eq!(config.tasks.estimate.queries["hunks"], "@source");
    assert_eq!(config.dicts["source"].values, ["!docs/**"]);
}

#[test]
fn both_packs_adopt_together_without_a_clash() {
    let config = load(
        "extends = [\"mindtape:tasks\", \"mindtape:wip-caps\", \
         { pack = \"mindtape:budget\", with = { max-files = 8 } }]\n",
    );
    for name in [
        "milestone-wip-cap",
        "leaf-wip-cap",
        "judgment-stays-small",
        "src-budget",
    ] {
        rule(&config, name);
    }
}
