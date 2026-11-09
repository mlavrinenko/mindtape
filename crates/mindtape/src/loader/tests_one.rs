//! `load_one`: one task file under the project's config. Shares
//! `loader/tests.rs`'s fixtures.
#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use std::path::Path;

use super::tests::{TASK_BODY, setup};
use super::tests_visible::{git, git_repo};
use super::{load_one, load_project};
use crate::config::Config;
use crate::parse_config;
use crate::test_support::write_file;

#[test]
fn a_valid_task_file_gives_the_task_load_project_gives_it() {
    let dir = setup();
    write_file(dir.path(), "tasks/build.typ", TASK_BODY);
    write_file(dir.path(), "tasks/other.typ", TASK_BODY);
    let config = Config::default();

    let one = load_one(dir.path(), &config, Path::new("tasks/build.typ"))
        .unwrap()
        .unwrap();
    let mut whole = load_project(dir.path(), &config)
        .tasks
        .into_iter()
        .find(|task| task.id == "build")
        .unwrap();
    // The graph-derived fields are facts about the whole set.
    whole.topo = one.topo;
    whole.blocks = one.blocks;
    assert_eq!(one, whole);
    assert_eq!(one.title, "Build");

    let absolute = dir.path().join("tasks/build.typ");
    let again = load_one(dir.path(), &config, &absolute).unwrap().unwrap();
    assert_eq!(again, one, "an absolute path names the same file");
}

#[test]
fn a_file_with_no_task_call_gives_none() {
    let dir = setup();
    write_file(dir.path(), "tasks/notes.typ", "= Just notes\n");
    let got = load_one(dir.path(), &Config::default(), Path::new("tasks/notes.typ"));
    assert_eq!(got, Ok(None));
}

#[test]
fn a_load_failure_gives_the_error_load_project_lists() {
    let dir = setup();
    write_file(
        dir.path(),
        "tasks/broken.typ",
        "#import \"@local/nope:9.9.9\": *\n",
    );
    let config = Config::default();
    let err = load_one(dir.path(), &config, Path::new("tasks/broken.typ")).unwrap_err();
    assert_eq!(err, load_project(dir.path(), &config).errors[0]);
}

#[test]
fn a_path_outside_every_root_is_an_error_naming_it() {
    let dir = setup();
    let outside = Path::new("elsewhere/build.typ");
    write_file(dir.path(), outside.to_str().unwrap(), TASK_BODY);
    let err = load_one(dir.path(), &Config::default(), outside).unwrap_err();
    assert_eq!(err.file, outside);
    assert!(err.message.contains("task root"), "{}", err.message);
}

#[test]
fn a_template_under_a_root_dot_is_not_a_task_source() {
    let dir = setup();
    let config = parse_config("[tasks]\nroots = [\".\"]\n").unwrap();
    let template = Path::new(".mindtape/templates/task.typ");
    write_file(dir.path(), template.to_str().unwrap(), TASK_BODY);

    let err = load_one(dir.path(), &config, template).unwrap_err();
    assert_eq!(err.file, template);
    assert!(err.message.contains("task root"), "{}", err.message);
}

#[cfg(unix)]
#[test]
fn a_symlinked_typ_file_is_not_a_task_source() {
    let dir = setup();
    write_file(dir.path(), "tasks/real.typ", TASK_BODY);
    std::os::unix::fs::symlink("real.typ", dir.path().join("tasks/link.typ")).unwrap();

    let err = load_one(dir.path(), &Config::default(), Path::new("tasks/link.typ")).unwrap_err();
    assert_eq!(err.file, Path::new("tasks/link.typ"));
}

/// `load_one` and `load_project` agree about Git visibility: an untracked file
/// Git ignores is refused, while a tracked file under an ignore pattern loads.
#[test]
fn load_one_refuses_an_untracked_ignored_file() {
    let dir = git_repo();
    write_file(dir.path(), "tasks/.gitignore", "hidden.typ\n");
    write_file(dir.path(), "tasks/hidden.typ", TASK_BODY);
    git(dir.path(), &["add", "tasks/.gitignore"]);
    git(dir.path(), &["commit", "-q", "-m", "seed"]);

    let err = load_one(
        dir.path(),
        &Config::default(),
        Path::new("tasks/hidden.typ"),
    )
    .unwrap_err();
    assert_eq!(err.file, Path::new("tasks/hidden.typ"));
    assert!(err.message.contains("ignored"), "{}", err.message);
}

#[test]
fn load_one_returns_a_failed_root_listing() {
    let dir = setup();
    write_file(dir.path(), ".git", "not a git directory\n");
    write_file(dir.path(), "tasks/a.typ", TASK_BODY);

    let err = load_one(dir.path(), &Config::default(), Path::new("tasks/a.typ")).unwrap_err();
    assert_eq!(err.file, Path::new("tasks/a.typ"));
    assert!(err.message.contains("tasks"), "{}", err.message);
}

#[test]
fn load_one_loads_a_tracked_file_under_an_ignore_pattern() {
    let dir = git_repo();
    write_file(dir.path(), "tasks/.gitignore", "hidden.typ\n");
    write_file(dir.path(), "tasks/hidden.typ", TASK_BODY);
    git(dir.path(), &["add", "tasks/.gitignore"]);
    git(dir.path(), &["add", "-f", "tasks/hidden.typ"]);
    git(dir.path(), &["commit", "-q", "-m", "seed"]);

    let task = load_one(
        dir.path(),
        &Config::default(),
        Path::new("tasks/hidden.typ"),
    )
    .unwrap()
    .unwrap();
    assert_eq!(task.id, "hidden");
}
