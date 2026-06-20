//! Shared fixtures for CLI command tests.

#![allow(clippy::unwrap_used)]

use std::path::Path;

use mindtape::{Config, load_project};

use crate::util::Project;

/// Create a project rooted at `dir` with `files` (`name`, `body`) written under
/// `tasks/`, then load it with `config`.
pub fn make_project(dir: &Path, files: &[(&str, &str)], config: Config) -> Project {
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
