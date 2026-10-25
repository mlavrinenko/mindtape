//! `load_one`: one task file under the project's config. Shares
//! `loader/tests.rs`'s fixtures.
#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use std::path::Path;

use super::tests::{TASK_BODY, setup};
use super::{load_one, load_project};
use crate::config::Config;
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
