//! The `mindtape:flow` and `mindtape:ice` packs, loaded through the real
//! registry: the status legend with its edges, the `stale-wip` gate and its
//! `days` parameter, the ICE priority over the floor's scale, and a project's
//! own legend replacing the pack's whole. How a flip moves through the graph
//! is the CLI tests' business (`flow_pack_e2e.rs`).
#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use super::tests_budget::load;
use crate::config::Config;
use crate::config::layers::load_config_layered_from;
use crate::config::status::LegendEntry;
use crate::test_support::home_and_project;

fn legend(config: &Config) -> &[LegendEntry] {
    &config.tasks.status.entries
}

fn entry<'a>(config: &'a Config, value: &str) -> &'a LegendEntry {
    legend(config)
        .iter()
        .find(|entry| entry.value == value)
        .unwrap()
}

fn values(config: &Config) -> Vec<&str> {
    legend(config)
        .iter()
        .map(|entry| entry.value.as_str())
        .collect()
}

#[test]
fn flow_restates_the_six_statuses_with_their_edges() {
    let config = load("extends = [\"mindtape:flow\"]\n");
    assert_eq!(
        values(&config),
        ["proposed", "accepted", "wip", "done", "cancelled", "failed"]
    );
    let edges: [(&str, &[&str]); 6] = [
        ("proposed", &["accepted", "wip", "cancelled"]),
        ("accepted", &["wip", "cancelled"]),
        ("wip", &["done", "failed", "accepted"]),
        ("done", &["wip", "cancelled", "failed"]),
        ("cancelled", &[]),
        ("failed", &["wip"]),
    ];
    for (value, to) in edges {
        assert_eq!(entry(&config, value).to, to, "{value}");
    }
    for (value, unblocked) in [("wip", true), ("done", true), ("accepted", false)] {
        assert_eq!(entry(&config, value).unblocked, unblocked, "{value}");
    }
    assert_eq!(entry(&config, "wip").glyph.as_deref(), Some("🔨"));
}

#[test]
fn flow_ships_the_stale_gate_at_thirty_days() {
    let config = load("extends = [\"mindtape:flow\"]\n");
    let hook = &config.on.flip[0];
    assert_eq!(hook.name.as_deref(), Some("stale-wip"));
    assert_eq!(hook.when, r#"into("wip") && stale(30)"#);
    assert!(hook.actions[0].gate);
    assert!(hook.actions[0].run.is_none());
}

#[test]
fn flow_takes_the_stale_days_as_a_parameter() {
    let config = load("extends = [{ pack = \"mindtape:flow\", with = { days = 7 } }]\n");
    assert_eq!(config.on.flip[0].when, r#"into("wip") && stale(7)"#);
}

#[test]
fn a_projects_own_legend_replaces_the_flow_legend_whole() {
    let config = load(
        "extends = [\"mindtape:flow\"]\n\n[[tasks.status]]\nvalue = \"open\"\n\n\
         [[tasks.status]]\nvalue = \"wip\"\n\n\
         [[tasks.status]]\nvalue = \"closed\"\nterminal = true\n",
    );
    assert_eq!(values(&config), ["open", "wip", "closed"]);
    assert!(entry(&config, "open").to.is_empty());
}

#[test]
fn a_legend_without_wip_leaves_the_stale_gate_naming_a_status_it_lacks() {
    let own = "extends = [\"mindtape:flow\"]\n\n[[tasks.status]]\nvalue = \"open\"\n";
    let (_tmp, home, project) = home_and_project();
    std::fs::write(project.join(".mindtape").join("config.toml"), own).unwrap();
    let err = load_config_layered_from(&project, Some(&home), None).unwrap_err();
    assert!(err.to_string().contains("unknown status: 'wip'"), "{err}");
    let disabled = format!("disable = [\"stale-wip\"]\n{own}");
    assert!(load(&disabled).on.flip.is_empty());
}

#[test]
fn ice_sets_the_framework_over_the_floors_scale_and_bands() {
    let config = load("extends = [\"mindtape:ice\"]\n");
    let priority = &config.tasks.priority;
    assert_eq!(priority.framework.as_deref(), Some("ice"));
    assert!(!priority.allow_manual);
    assert_eq!(priority.field.example, Some(3.0));
    assert_eq!((priority.field.min, priority.field.max), (0.0, 9.0));
    let labels: Vec<_> = priority
        .field
        .bands
        .iter()
        .map(|band| band.label.as_str())
        .collect();
    assert_eq!(labels, ["low", "normal", "high"]);
}

#[test]
fn the_packs_adopt_beside_the_tasks_pack() {
    let config = load("extends = [\"mindtape:tasks\", \"mindtape:flow\", \"mindtape:ice\"]\n");
    assert_eq!(config.on.flip.len(), 1);
    assert_eq!(config.tasks.priority.framework.as_deref(), Some("ice"));
}
