//! Shared fixtures for CLI command tests.

#![allow(clippy::unwrap_used)]

use std::path::Path;

use mindtape::test_support::tempdir;
use mindtape::{Config, load_project, parse_config};
use tempfile::TempDir;

use crate::util::Project;

/// [`make_project_in`] a fresh tempdir, which the caller keeps alive.
pub fn make_project(files: &[(&str, &str)], config: Config) -> (TempDir, Project) {
    let dir = tempdir();
    let project = make_project_in(dir.path(), files, config);
    (dir, project)
}

/// Create a project rooted at `dir` with `files` (`name`, `body`) written under
/// `tasks/`, then load it with `config`.
pub fn make_project_in(dir: &Path, files: &[(&str, &str)], config: Config) -> Project {
    std::fs::create_dir_all(dir.join(".mindtape")).unwrap();
    std::fs::create_dir_all(dir.join("tasks")).unwrap();
    for (name, body) in files {
        std::fs::write(dir.join("tasks").join(name), body).unwrap();
    }
    let outcome = load_project(dir, &config);
    Project {
        root: dir.to_path_buf(),
        config,
        outcome,
    }
}

/// [`make_project_with_lib_in`] a fresh tempdir, which the caller keeps alive.
pub fn make_project_with_lib(
    prelude: &str,
    files: &[(&str, &str)],
    config_text: &str,
) -> (TempDir, Project) {
    let dir = tempdir();
    let project = make_project_with_lib_in(dir.path(), prelude, files, config_text);
    (dir, project)
}

/// [`make_project_in`], but first writes an in-tree `@local/mindtape` package
/// override (`prelude`) and a `.mindtape/config.toml` from `config_text`, then
/// parses `config_text` to load `files`. Shared by command tests (`flip`,
/// `recur`, ...) that need a real Typst package resolvable on disk, e.g. to
/// exercise commands that revalidate by reloading config from disk.
pub fn make_project_with_lib_in(
    dir: &Path,
    prelude: &str,
    files: &[(&str, &str)],
    config_text: &str,
) -> Project {
    mindtape::test_support::write_lib_package(dir, prelude);
    std::fs::create_dir_all(dir.join(".mindtape")).unwrap();
    std::fs::write(dir.join(".mindtape/config.toml"), config_text).unwrap();
    let config = parse_config(config_text).unwrap();
    make_project_in(dir, files, config)
}
