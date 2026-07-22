//! Shared fixtures for CLI command tests.

#![allow(clippy::unwrap_used)]

use std::path::Path;
use std::process::{Command, Output};

use mindtape::{Config, load_project, parse_config};

use crate::util::Project;

/// Re-exec this test binary for the single test named `test_name`, with
/// `sentinel` set in the child's environment.
///
/// Process-global state (a `OnceLock`, the `log` global logger) can only be
/// asserted "unset before init" in a fresh process: sharing the test binary's
/// process would race on init order and poison the state for every other test.
/// Callers branch on `std::env::var(sentinel)` — parent side spawns, child side
/// runs the real assertions. `RUST_LOG` is cleared so the child's logging is
/// independent of the developer's environment.
pub fn run_self_test(test_name: &str, sentinel: &str) -> Output {
    Command::new(std::env::current_exe().unwrap())
        .env(sentinel, "1")
        .env_remove("RUST_LOG")
        .args([test_name, "--nocapture"])
        .output()
        .unwrap()
}

/// Assert a [`run_self_test`] child exited successfully, surfacing its stderr.
pub fn assert_child_ok(out: &Output) {
    assert!(
        out.status.success(),
        "child failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

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

/// [`make_project`], but first writes an in-tree `@local/mindtape` package
/// override (`prelude`) and a `.mindtape/config.toml` from `config_text`, then
/// parses `config_text` to load `files`. Shared by command tests (`flip`,
/// `recur`, ...) that need a real Typst package resolvable on disk, e.g. to
/// exercise commands that revalidate by reloading config from disk.
pub fn make_project_with_lib(
    dir: &Path,
    prelude: &str,
    files: &[(&str, &str)],
    config_text: &str,
) -> Project {
    mindtape::test_support::write_lib_package(dir, prelude);
    std::fs::create_dir_all(dir.join(".mindtape")).unwrap();
    std::fs::write(dir.join(".mindtape/config.toml"), config_text).unwrap();
    let config = parse_config(config_text).unwrap();
    make_project(dir, files, config)
}
