//! A directory pack: an `extends` path naming a directory reads its
//! `pack.toml`, and the pack's `templates/` joins `templates.dirs` after the
//! merge. The shared `layer` fixture stays in the parent test module and is
//! reached through `super::tests`.
#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use std::path::{Path, PathBuf};

use super::merge_layers_with_packs;
use super::tests::layer;
use crate::config::layers::{LayerSource, load_config_layered_from};
use crate::config::pack_registry::{PackOrigin, RuleOrigin};
use crate::config::packs_toml::read_array;
use crate::test_support::{tempdir_with_files, write_file};

const RULE: &str = "[[tasks.rules]]\nname = \"team-rule\"\nassert = 'kind == \"bug\"'\n";

/// A tempdir holding `files`, and its `project` folder, created.
fn tree(files: &[(&str, &str)]) -> (tempfile::TempDir, PathBuf) {
    let tmp = tempdir_with_files(files);
    let project = tmp.path().join("project");
    std::fs::create_dir_all(&project).unwrap();
    (tmp, project)
}

/// The merge of one project layer at `project` with `text` as its config.
fn merge_project(
    project: &Path,
    text: &str,
) -> Result<super::PackMerge, crate::config::ConfigError> {
    let source = LayerSource::Project(project.join(".mindtape").join("config.toml"));
    merge_layers_with_packs(&[layer(source, text)], project, None)
}

/// How many task rules `extends = [{entry}]` reads from `files` in a project.
fn rules_read(files: &[(&str, &str)], entry: &str) -> usize {
    let (_tmp, project) = tree(files);
    let merged = merge_project(&project, &format!("extends = [{entry}]\n")).unwrap();
    read_array(&merged.table, &["tasks", "rules"]).len()
}

#[test]
fn an_extends_path_naming_a_directory_reads_its_pack_toml() {
    let (_tmp, project) = tree(&[("project/team/pack.toml", RULE)]);

    let merged = merge_project(&project, "extends = [\"./team\"]\n").unwrap();

    assert_eq!(
        merged.rule_origins[0].origin,
        RuleOrigin::Pack(PackOrigin::Dir(project.join("team"))),
    );
}

#[test]
fn a_directory_pack_may_sit_outside_the_project() {
    let files = [("shared/team/pack.toml", RULE)];

    assert_eq!(rules_read(&files, "\"../shared/team/\""), 1);
}

#[test]
fn a_directory_with_no_pack_toml_is_a_missing_pack() {
    let (_tmp, project) = tree(&[]);
    std::fs::create_dir_all(project.join("team")).unwrap();

    let err = merge_project(&project, "extends = [\"./team\"]\n").unwrap_err();

    assert_eq!(
        err.to_string(),
        format!(
            "invalid config: extends: pack not found: '{}'",
            project.join("team").join("pack.toml").display()
        )
    );
}

#[test]
fn a_toml_file_beside_a_same_named_directory_is_ambiguous() {
    let (_tmp, project) = tree(&[
        ("project/team/pack.toml", RULE),
        ("project/team.toml", RULE),
    ]);

    let err = merge_project(&project, "extends = [\"./team\"]\n").unwrap_err();

    let rendered = err.to_string();
    assert!(rendered.contains("ambiguous"), "{rendered}");
    assert!(rendered.contains(&project.join("team").display().to_string()));
    assert!(rendered.contains(&project.join("team.toml").display().to_string()));
}

#[test]
fn naming_the_toml_file_beside_a_directory_is_not_ambiguous() {
    let files = [("project/team/pack.toml", ""), ("project/team.toml", RULE)];

    assert_eq!(rules_read(&files, "\"./team.toml\""), 1);
}

#[test]
fn a_directory_packs_templates_follow_the_projects_own_dirs() {
    let (_tmp, project) = tree(&[
        ("project/team/pack.toml", ""),
        ("project/team/templates/feat.typ", ""),
    ]);

    let merged = merge_project(
        &project,
        "extends = [\"./team\"]\n\n[templates]\ndirs = [\"mine\"]\n",
    )
    .unwrap();

    let team = project.join("team/templates").display().to_string();
    assert_eq!(
        merged.table["templates"]["dirs"].to_string(),
        format!("[\"mine\", {team:?}]")
    );
}

#[test]
fn only_a_directory_pack_with_a_templates_folder_adds_a_dir() {
    let (_tmp, project) = tree(&[
        ("project/bare/pack.toml", ""),
        ("project/team.toml", RULE),
        ("project/templates/feat.typ", ""),
    ]);

    let merged = merge_project(
        &project,
        "extends = [\"./bare\", \"./team.toml\", \"mindtape:tasks\"]\n",
    )
    .unwrap();

    assert!(
        merged.table.get("templates").is_none(),
        "{:?}",
        merged.table
    );
}

/// The template dirs a project under `home` loads to, `files` written under
/// `home` and `ancestor` and `own` the `extends` of `home`'s `defaults.toml`
/// and the project's `config.toml`.
fn load_dirs(files: &[(&str, &str)], ancestor: &str, own: &str) -> Vec<String> {
    let tmp = tempdir_with_files(files);
    let home = tmp.path().to_path_buf();
    let project = home.join("project");
    write_file(
        &home,
        ".mindtape/defaults.toml",
        format!("extends = [{ancestor}]\n"),
    );
    write_file(
        &project,
        ".mindtape/config.toml",
        format!("extends = [{own}]\n"),
    );
    let (config, _sources, _origins) =
        load_config_layered_from(&project, Some(&home), None).unwrap();
    config.templates.dirs
}

/// Two directory packs, `a` and `b`, each shipping `x.typ`, beside the
/// project.
const AB: [(&str, &str); 4] = [
    ("a/pack.toml", ""),
    ("a/templates/x.typ", ""),
    ("b/pack.toml", ""),
    ("b/templates/x.typ", ""),
];

/// The folder before `/templates` in each of `dirs`, the project's own
/// `.mindtape/templates` showing as `.mindtape`.
fn names(dirs: &[String]) -> Vec<&str> {
    (dirs.iter())
        .map(|dir| {
            let folder = dir.trim_end_matches("/templates");
            folder.rsplit('/').next().unwrap_or(folder)
        })
        .collect()
}

/// The template dirs of `AB` as folder names, for the layers' `extends`.
fn order(ancestor: &str, own: &str) -> Vec<String> {
    let dirs = load_dirs(&AB, ancestor, own);
    names(&dirs).into_iter().map(str::to_owned).collect()
}

#[test]
fn the_pack_listed_last_comes_first_and_the_later_layers_before_the_earlier() {
    assert_eq!(order("", "\"../a\", \"../b\""), [".mindtape", "b", "a"]);
    assert_eq!(order("\"./a\"", "\"../b\""), [".mindtape", "b", "a"]);
}

#[test]
fn a_directory_pack_two_layers_extend_adds_its_templates_once_at_its_latest_place() {
    assert_eq!(
        order("\"./a\", \"./b\"", "\"../a\""),
        [".mindtape", "a", "b"]
    );
}
