#![allow(clippy::unwrap_used)]

use std::path::Path;

use super::RuleSites;
use crate::config::layers::{LayerSource, load_config_layered_from};
use crate::config::packs::{PackOrigin, RuleOrigin, RuleOriginEntry};
use crate::rules::check_all;
use crate::test_support::bare_task;
use crate::validate::severity::Severity;
use crate::validate::{Location, Violation};

/// A rule git cannot answer, so every config that declares it skips it.
fn skipping_rule(name: &str) -> String {
    format!("[[tasks.rules]]\nname = \"{name}\"\nassert = 'changed(\"worktree\") <= 3'\n")
}

/// The one report `check_all` raises for a project at `root` whose rules are
/// all skipped: the project is no git repository.
fn skip_report(root: &Path, xdg: Option<&Path>) -> Violation {
    let (mut config, _, _) = load_config_layered_from(root, None, xdg).unwrap();
    config.git.set_root(root);
    let mut task = bare_task("a");
    task.status = Some(crate::model::Status {
        name: "wip".to_owned(),
        date: None,
        note: None,
    });
    let mut out = Vec::new();
    check_all(&[task], &config, &mut out);
    assert_eq!(out.len(), 1, "{out:?}");
    out.remove(0)
}

fn project(config: &str) -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join(".mindtape")).unwrap();
    std::fs::write(dir.path().join(".mindtape/config.toml"), config).unwrap();
    dir
}

#[test]
fn a_skipped_project_rule_is_located_at_its_header() {
    let dir = project(&format!(
        "[tasks]\nroots = [\"tasks\"]\n\n{}",
        skipping_rule("src-budget")
    ));
    let skip = skip_report(dir.path(), None);
    assert_eq!(skip.severity, Severity::Warn);
    assert_eq!(skip.location, Location::at(".mindtape/config.toml", 4));
}

#[test]
fn a_skipped_path_pack_rule_is_located_in_the_pack_file() {
    let dir = project("extends = [\"./pack.toml\"]\n");
    let pack = format!("# a pack\n\n{}", skipping_rule("src-budget"));
    std::fs::write(dir.path().join("pack.toml"), pack).unwrap();
    let skip = skip_report(dir.path(), None);
    assert_eq!(skip.location, Location::at("pack.toml", 3));
}

#[test]
fn a_skipped_global_layer_rule_is_located_in_that_layer() {
    let dir = project("[tasks]\nroots = [\"tasks\"]\n");
    let xdg = tempfile::tempdir().unwrap();
    let defaults = xdg.path().join("mindtape/defaults.toml");
    std::fs::create_dir_all(defaults.parent().unwrap()).unwrap();
    std::fs::write(&defaults, skipping_rule("src-budget")).unwrap();
    let skip = skip_report(dir.path(), Some(xdg.path()));
    assert_eq!(skip.location, Location::at(defaults, 1));
}

#[test]
fn a_rule_the_project_overrides_by_name_is_located_where_it_won() {
    let dir = project(&format!(
        "extends = [\"./pack.toml\"]\n\n{}",
        skipping_rule("src-budget")
    ));
    std::fs::write(dir.path().join("pack.toml"), skipping_rule("src-budget")).unwrap();
    let skip = skip_report(dir.path(), None);
    assert_eq!(skip.location, Location::at(".mindtape/config.toml", 3));
}

#[test]
fn a_built_in_pack_rule_is_located_by_pack_name_and_line() {
    let entry = RuleOriginEntry {
        label: "tasks.rules.imperative-title".to_owned(),
        origin: RuleOrigin::Pack(PackOrigin::Embedded("mindtape-tasks".to_owned())),
        line: Some(17),
    };
    let sites = RuleSites::from_origins(&[entry], Path::new("/project"));
    let site = sites.get("rules.imperative-title").unwrap();
    assert_eq!(site.to_string(), "pack mindtape-tasks:17");
    assert_eq!(site.file(), None);
}

#[test]
fn an_entry_without_a_line_or_a_file_has_no_site() {
    let inline = RuleOriginEntry {
        label: "tasks.rules.inline".to_owned(),
        origin: RuleOrigin::Layer(LayerSource::Project(
            "/project/.mindtape/config.toml".into(),
        )),
        line: None,
    };
    let floor = RuleOriginEntry {
        label: "tasks.rules.floor".to_owned(),
        origin: RuleOrigin::Layer(LayerSource::Embedded),
        line: Some(2),
    };
    let sites = RuleSites::from_origins(&[inline, floor], Path::new("/project"));
    assert_eq!(sites.get("rules.inline"), None);
    assert_eq!(sites.get("rules.floor"), None);
}
