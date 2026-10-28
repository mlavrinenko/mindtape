//! A config error names the file it comes from: the layer, the ancestor
//! `defaults.toml` or the pack that holds the offending key.

#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use std::path::PathBuf;

use crate::config::ConfigError;
use crate::test_support::{home_and_project, write_file};

/// Load a project whose `.mindtape/config.toml` is `config`, beside `files`
/// (paths from the project root, `../` for the home directory), and return
/// the error.
fn failure(config: &str, files: &[(&str, &str)]) -> ConfigError {
    let (_tmp, home, project) = home_and_project();
    write_file(&project, ".mindtape/config.toml", config);
    for (path, text) in files {
        write_file(&project, path, text);
    }
    super::load_config_layered_from(&project, Some(&home), None).unwrap_err()
}

/// [`failure`] over a project that extends `team.toml`, which holds `pack`.
fn pack_failure(config: &str, pack: &str) -> ConfigError {
    let config = format!("extends = [\"./team.toml\"]\n{config}");
    failure(&config, &[("team.toml", pack)])
}

fn parse_in(err: ConfigError) -> (String, usize, String) {
    let ConfigError::ParseIn { origin, error } = err else {
        panic!("expected a parse error in a file, got {err}");
    };
    let shown = error.to_string();
    let line = shown
        .strip_prefix("TOML parse error at line ")
        .and_then(|rest| rest.split(',').next())
        .and_then(|line| line.parse().ok())
        .unwrap_or_else(|| panic!("no line in {shown}"));
    (origin, line, error.message().to_owned())
}

fn place(err: ConfigError) -> (PathBuf, usize, String) {
    let ConfigError::Invalid { message, place, .. } = err else {
        panic!("expected an invalid config, got {err}");
    };
    let place = place.unwrap_or_else(|| panic!("no place for {message}"));
    (place.file, place.line, message)
}

#[test]
fn an_unknown_key_in_a_path_pack_names_the_pack_and_its_line() {
    let (origin, line, message) = parse_in(pack_failure("", "[ls]\n\n[projet]\nx = 1\n"));
    assert_eq!((origin.as_str(), line), ("team.toml", 3));
    assert!(message.contains("unknown field `projet`"), "{message}");
}

#[test]
fn an_unknown_key_in_a_directory_pack_names_its_pack_file() {
    let err = failure(
        "extends = [\"./team\"]\n",
        &[("team/pack.toml", "[projet]\nx = 1\n")],
    );
    let (origin, line, _) = parse_in(err);
    assert_eq!(origin, crate::display_path("team/pack.toml"));
    assert_eq!(line, 1);
}

#[test]
fn an_unknown_key_in_an_ancestor_defaults_names_that_file() {
    let err = failure(
        "[ls]\n",
        &[("../.mindtape/defaults.toml", "\n[projet]\nx=1\n")],
    );
    let (origin, line, _) = parse_in(err);
    let defaults = crate::display_path(".mindtape/defaults.toml");
    assert!(origin.ends_with(&defaults), "{origin}");
    assert_eq!(line, 2);
}

#[test]
fn an_unknown_key_in_the_project_config_still_names_it() {
    let (origin, line, _) = parse_in(failure("[ls]\n[projet]\nx = 1\n", &[]));
    let config = crate::display_path(".mindtape/config.toml");
    assert_eq!((origin, line), (config, 2));
}

#[test]
fn a_wrong_typed_value_in_a_pack_names_the_pack() {
    let (origin, line, _) = parse_in(pack_failure("", "[views.board]\nsort = 3\n"));
    assert_eq!((origin.as_str(), line), ("team.toml", 2));
}

#[test]
fn a_bad_value_a_pack_sets_is_placed_in_the_pack_not_at_an_ancestor_key() {
    let mine = "\n[views.mine]\nfilter = \"has(title)\"\n";
    let (file, line, message) = place(pack_failure(mine, "[views.board]\nfilter = \"((\"\n"));
    assert_eq!((file, line), (PathBuf::from("team.toml"), 2));
    assert!(message.starts_with("views.board.filter: "), "{message}");
}

#[test]
fn a_bad_value_an_ancestor_defaults_sets_is_placed_in_that_file() {
    let defaults = "\n[views.board]\nfilter = \"((\"\n";
    let err = failure(
        "[views.mine]\nfilter = \"has(title)\"\n",
        &[("../.mindtape/defaults.toml", defaults)],
    );
    let (file, line, _) = place(err);
    assert!(file.ends_with(".mindtape/defaults.toml"), "{file:?}");
    assert_eq!(line, 3);
}

/// A rule of the project's own, which a pack rule's index must not reach.
const MINE: &str = "\n[[tasks.rules]]\nname = \"mine\"\nwhen = \"has(title)\"\nassert = \"ASSERT\"\nmessage = \"x\"\n";
const BOARD: &str = "[[tasks.rules]]\nname = \"board\"\nwhen = \"has(title)\"\nassert = \"ASSERT\"\nmessage = \"x\"\n";

#[test]
fn a_pack_rules_bad_expression_is_placed_in_the_pack_beside_the_projects_own_rules() {
    let err = pack_failure(
        &MINE.replace("ASSERT", "has(title)"),
        &BOARD.replace("ASSERT", "(("),
    );
    let (file, line, message) = place(err);
    assert_eq!((file, line), (PathBuf::from("team.toml"), 4));
    assert!(message.starts_with("tasks.rules[0].assert: "), "{message}");
}

#[test]
fn a_project_rules_bad_expression_is_placed_after_a_pack_rule() {
    let err = pack_failure(
        &MINE.replace("ASSERT", "(("),
        &BOARD.replace("ASSERT", "has(title)"),
    );
    let (file, line, message) = place(err);
    assert_eq!((file, line), (PathBuf::from(".mindtape/config.toml"), 6));
    assert!(message.starts_with("tasks.rules[1].assert: "), "{message}");
}

/// A project reached through a symlink still names its pack relative to the
/// root. The pack's path is canonical, so the root has to be too: Windows
/// has the same split between a short `RUNNER~1` name and the long one.
#[cfg(unix)]
#[test]
fn a_pack_under_a_symlinked_root_is_named_relative_to_it() {
    let (tmp, home, project) = home_and_project();
    let symlinked = tmp.path().join("link");
    std::os::unix::fs::symlink(&project, &symlinked).unwrap();
    write_file(
        &project,
        ".mindtape/config.toml",
        "extends = [\"./team.toml\"]\n",
    );
    write_file(&project, "team.toml", "[ls]\n\n[projet]\nx = 1\n");
    let err = super::load_config_layered_from(&symlinked, Some(&home), None).unwrap_err();
    let (origin, line, _) = parse_in(err);
    assert_eq!((origin.as_str(), line), ("team.toml", 3));
}
