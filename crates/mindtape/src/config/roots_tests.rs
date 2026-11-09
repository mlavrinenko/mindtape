#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use super::validate_task_sources;
use crate::config::parse_config;
use crate::test_support::{home_and_project, tempdir};

/// The error [`validate_task_sources`] refuses `config` with, against a fresh
/// project root.
fn refused(config: &str) -> String {
    let config = parse_config(config).unwrap();
    let dir = tempdir();
    validate_task_sources(&config, dir.path())
        .unwrap_err()
        .to_string()
}

/// Assert [`validate_task_sources`] accepts `config` against a fresh project
/// root.
fn accepts(config: &str) {
    let config = parse_config(config).unwrap();
    let dir = tempdir();
    if let Err(err) = validate_task_sources(&config, dir.path()) {
        panic!("expected the config to pass, got {err}");
    }
}

#[test]
fn a_root_under_mindtape_is_reserved() {
    let message = refused("[tasks]\nroots = [\".mindtape/templates\"]\n");
    assert!(
        message.contains(
            "tasks.roots: `.mindtape/templates` is reserved for project metadata and templates"
        ),
        "{message}"
    );
}

#[test]
fn a_root_under_git_is_reserved() {
    let message = refused("[tasks]\nroots = [\".git/tasks\"]\n");
    assert!(
        message.contains("tasks.roots: `.git/tasks` is inside `.git`, reserved for git metadata"),
        "{message}"
    );
}

#[test]
fn reserved_components_are_matched_in_any_case() {
    let message = refused("[tasks]\nroots = [\".MINDtape/tasks\"]\n");
    assert!(
        message.contains("reserved for project metadata and templates"),
        "{message}"
    );
}

#[test]
fn a_root_inside_a_template_dir_is_reserved() {
    let message = refused(
        "[tasks]\nroots = [\"templates/tasks\"]\n\n\
         [templates]\ndirs = [\"templates\"]\n",
    );
    assert!(
        message.contains("templates.dirs: `templates` is at or above task root `templates/tasks`"),
        "{message}"
    );
}

#[test]
fn a_template_dir_equal_to_the_project_root_is_reserved() {
    let message = refused("[tasks]\nroots = [\"tasks\"]\n\n[templates]\ndirs = [\".\"]\n");
    assert!(
        message.contains("templates.dirs: `.` is at or above the project root"),
        "{message}"
    );
}

#[test]
fn a_template_dir_ancestor_of_the_project_root_is_reserved() {
    let message = refused("[tasks]\nroots = [\"tasks\"]\n\n[templates]\ndirs = [\"..\"]\n");
    assert!(
        message.contains("templates.dirs: `..` is at or above the project root"),
        "{message}"
    );
}

#[test]
fn a_template_dir_ancestor_of_a_task_root_is_reserved() {
    let message = refused(
        "[tasks]\nroots = [\"tasks/sub\"]\n\n\
         [templates]\ndirs = [\"tasks\"]\n",
    );
    assert!(
        message.contains("templates.dirs: `tasks` is at or above task root `tasks/sub`"),
        "{message}"
    );
}

#[test]
fn a_root_of_dot_that_contains_template_dirs_is_valid() {
    accepts("[tasks]\nroots = [\".\"]\n\n[templates]\ndirs = [\".mindtape/templates\"]\n");
}

#[test]
fn a_template_dir_inside_a_root_is_valid() {
    accepts("[tasks]\nroots = [\"tasks\"]\n\n[templates]\ndirs = [\"tasks/templates\"]\n");
}

/// A commit state's config goes through `build_config` alone, so a reserved
/// root it declares is read as written rather than refused at load.
#[test]
fn a_historical_state_config_skips_the_reserved_root_check() {
    let (_tmp, home, project) = home_and_project();
    let text = "[tasks]\nroots = [\".mindtape/templates\"]\n";

    let config =
        crate::config::layers::load_config_with_project_in(&project, text, Some(&home), None, None)
            .unwrap();
    assert_eq!(config.tasks.roots[0].path(), ".mindtape/templates");
}

#[test]
fn roots_are_compared_in_their_normalized_form() {
    accepts(
        "[tasks]\nroots = [\"./tasks/../tasks\"]\n\n[templates]\ndirs = [\".mindtape/templates\"]\n",
    );
    let message = refused("[tasks]\nroots = [\"./tasks/../.git\"]\n");
    assert!(
        message.contains("tasks.roots: `./tasks/../.git`"),
        "{message}"
    );
}
