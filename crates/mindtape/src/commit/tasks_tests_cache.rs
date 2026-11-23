//! State-loader cache and commit-state tests that need their own file to stay
//! under the linecap.
#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use super::tests::{
    MARKER, PIN_MANIFEST, PINNED_TASK, load, load_from, pin_config, revisions, sub_loader,
    sub_plan, titled_prelude,
};
use super::{StateLoader, StatePlan};
use crate::config::{Config, parse_config};
use crate::model::Task;
use crate::test_support::commit_files;
use crate::test_support::git_as_tester as git;
use crate::test_support::git_stdout;
use crate::test_support::git_tempdir;
use crate::test_support::write_file;
use typst_harvest::HVal;

/// A source that fails evaluation.
const BROKEN: &str = "#let x = (\n";

/// A project module whose value a task's body marker reports.
fn goal_module(value: &str) -> String {
    format!("#let goal = \"{value}\"\n")
}

/// A task that imports the project module and emits the value it read as a body
/// marker. Its own bytes are the same in every state.
const MODULE_TASK: &str = "#import \"/truth/goals.typ\": goal\n\n\
                           #metadata((marker: \"probe\", value: goal))\n\
                           #metadata((marker: \"mindtape.task\", \
                           fields: (title: \"Build\"), extras: (:)))\n";

/// The value the `probe` body marker carries, when the task emitted one.
fn probe(task: &Task) -> Option<&str> {
    task.markers
        .iter()
        .find(|marker| marker.value.get("marker").and_then(HVal::as_str) == Some("probe"))
        .and_then(|marker| marker.value.get("value"))
        .and_then(HVal::as_str)
}

/// The working-tree load of `root` under `config`.
fn working_base(root: &std::path::Path, config: &Config) -> Vec<Task> {
    crate::loader::load_project(root, config).tasks
}

/// A state loader bounded to `root`, over `base` and `config`.
fn state_loader<'a>(
    root: &'a std::path::Path,
    base: &'a [Task],
    config: &'a Config,
) -> StateLoader<'a> {
    StateLoader::with_layers(root, base, config, (Some(root.to_path_buf()), None, None))
}

/// A repository whose task imports a module every state sees, with the config it
/// declares and the commit of its first module.
fn module_repo() -> (tempfile::TempDir, Config, String) {
    let dir = git_tempdir();
    let root = dir.path();
    let text = "[tasks]\nroots = [\"tasks\"]\n";
    commit_files(
        root,
        &[
            (".mindtape/config.toml", text),
            ("truth/goals.typ", &goal_module("first")),
            ("tasks/a.typ", MODULE_TASK),
            ("tasks/notes.typ", "= Notes\n"),
        ],
        "one",
    );
    let one = git_stdout(root, &["rev-parse", "HEAD"]);
    let config = parse_config(text).unwrap();
    (dir, config, one)
}

#[test]
fn a_reused_state_reevaluates_a_task_whose_imported_module_changed() {
    let (dir, config, one) = module_repo();
    let root = dir.path();
    let mut loader = state_loader(root, &[], &config);
    let before = loader.load(&revisions(&one, &one)).unwrap();
    assert_eq!(probe(&before.staged.tasks[0]), Some("first"));
    assert_eq!(loader.evaluations(), 2, "the task and the file without one");

    commit_files(root, &[("truth/goals.typ", &goal_module("second"))], "two");
    let two = git_stdout(root, &["rev-parse", "HEAD"]);
    let after = loader.load(&revisions(&two, &two)).unwrap();
    assert_eq!(
        probe(&after.staged.tasks[0]),
        Some("second"),
        "the task replays the module it read, not the first state's"
    );
    assert_eq!(loader.evaluations(), 3, "the module alone forces it again");
    assert_eq!(
        before.staged.tasks[0].body, after.staged.tasks[0].body,
        "only the imported module changed"
    );
}

#[test]
fn a_seeded_task_is_not_reused_after_its_imported_module_changes() {
    let (dir, config, one) = module_repo();
    let root = dir.path();
    let base = working_base(root, &config);
    assert_eq!(probe(&base[0]), Some("first"), "the seed reads the module");

    // The working tree moves on while the state's own bytes stay put, so only
    // the recorded imports can tell the seed apart from the state.
    write_file(root, "truth/goals.typ", goal_module("second"));
    let mut loader = state_loader(root, &base, &config);
    let plan = revisions(&one, &one);
    let states = loader.load(&plan).unwrap();
    assert_eq!(probe(&states.staged.tasks[0]), Some("second"));
    assert_eq!(loader.evaluations(), 2, "the seed could not answer");
}

#[test]
fn a_seeded_task_keeps_the_state_root_choice_clean_or_dirty() {
    let cases = [
        (
            "[tasks]\nroots = [\"tasks\", \"tasks/sub\"]\n[tasks.derive-tags]\nenabled = true\n",
            vec!["tasks", "sub"],
        ),
        (
            "[tasks]\nroots = [\"./tasks\"]\n[tasks.derive-tags]\nenabled = true\n",
            vec!["tasks", "sub"],
        ),
        (
            "[tasks]\nroots = [\"a/../tasks\"]\n[tasks.derive-tags]\nenabled = true\n",
            vec!["tasks", "sub"],
        ),
    ];
    for (text, expected) in cases {
        let dir = git_tempdir();
        let root = dir.path();
        commit_files(
            root,
            &[
                (".mindtape/config.toml", text),
                ("tasks/sub/x.typ", MARKER),
                ("a/keep.txt", "x\n"),
            ],
            "seed",
        );
        let config = parse_config(text).unwrap();
        let plan = StatePlan::index(root).unwrap();
        let clean_base = working_base(root, &config);
        let clean = load_from(root, &clean_base, &config, &plan);

        // A working-tree edit that stays out of the index: the state bytes do
        // not change, so a seeded state must not depend on it.
        write_file(root, "tasks/sub/x.typ", format!("{MARKER}// dirty\n"));
        let dirty_base = working_base(root, &config);
        let dirty = load_from(root, &dirty_base, &config, &plan);

        assert_eq!(
            clean.staged.tasks[0].derived_tags, expected,
            "the first root wins: {text}"
        );
        assert_eq!(
            dirty.staged.tasks[0].derived_tags,
            clean.staged.tasks[0].derived_tags
        );
        assert_eq!(dirty.staged.tasks[0].depth, clean.staged.tasks[0].depth);
    }
}

#[test]
fn a_missing_state_config_keeps_ancestor_layers() {
    let dir = git_tempdir();
    let repo = dir.path();
    let root = repo.join("sub");
    commit_files(
        repo,
        &[
            (
                ".mindtape/defaults.toml",
                "[tasks]\nroots = [\"backlog\"]\n",
            ),
            ("sub/backlog/parser.typ", MARKER),
        ],
        "seed",
    );
    let config = parse_config("").unwrap();
    let mut plan = revisions("HEAD", "HEAD");
    plan.root_prefix = "sub/".to_owned();
    let mut loader =
        StateLoader::with_layers(&root, &[], &config, (Some(repo.to_path_buf()), None, None));
    let states = loader.load(&plan).unwrap();
    assert_eq!(states.staged.tasks[0].id, "parser");
}

#[test]
fn a_state_config_overrides_an_ancestor_root_and_pin() {
    let dir = git_tempdir();
    let repo = dir.path();
    let root = repo.join("sub");
    let old = titled_prelude("Ancestor pin");
    let new = titled_prelude("Project pin");
    commit_files(
        repo,
        &[
            (
                ".mindtape/defaults.toml",
                "[tasks]\nroots = [\"backlog\"]\n\n\
                 [[typst.local-packages]]\nname = \"mindtape\"\npath = \"old-lib\"\n",
            ),
            ("sub/old-lib/typst.toml", PIN_MANIFEST),
            ("sub/old-lib/prelude.typ", &old),
            ("sub/lib/typst.toml", PIN_MANIFEST),
            ("sub/lib/prelude.typ", &new),
            ("sub/.mindtape/config.toml", &pin_config("lib")),
            ("sub/tasks/x.typ", PINNED_TASK),
            ("sub/backlog/y.typ", MARKER),
        ],
        "seed",
    );
    let config = parse_config("").unwrap();
    let plan = sub_plan();
    let mut loader = sub_loader(&root, &config, repo);
    let states = loader.load(&plan).unwrap();
    assert_eq!(states.staged.tasks.len(), 1);
    assert_eq!(
        (
            states.staged.tasks[0].id.as_str(),
            states.staged.tasks[0].title.as_str()
        ),
        ("x", "Project pin")
    );
}

#[test]
fn a_state_config_reading_a_moved_dictionary_warns_and_falls_back() {
    let dir = git_tempdir();
    let root = dir.path();
    commit_files(
        root,
        &[
            (
                ".mindtape/config.toml",
                "[tasks]\nroots = [\"tasks\"]\n\n[dicts.words]\npath = \"words.txt\"\n",
            ),
            ("words.txt", "alpha\n"),
            ("tasks/a.typ", MARKER),
        ],
        "seed",
    );
    // The state tree still holds words.txt; the working tree does not.
    std::fs::remove_file(root.join("words.txt")).unwrap();
    let config = parse_config("[tasks]\nroots = [\"tasks\"]\n").unwrap();
    let plan = revisions("HEAD", "HEAD");
    let states = load(root, &config, &plan);
    assert_eq!(states.head.tasks[0].id, "a");
    let error = states
        .head
        .errors
        .first()
        .expect("the moved dictionary is reported");
    assert_eq!(
        error.file,
        std::path::PathBuf::from(".mindtape/config.toml")
    );
}

#[test]
fn the_state_file_cache_reuses_hits_and_never_stores_errors() {
    let dir = git_tempdir();
    let root = dir.path();
    let config_text = "[tasks]\nroots = [\"tasks\"]\n";
    commit_files(
        root,
        &[
            (".mindtape/config.toml", config_text),
            ("tasks/a.typ", MARKER),
            ("tasks/b.typ", MARKER),
            ("tasks/broken.typ", BROKEN),
        ],
        "one",
    );
    let config = parse_config(config_text).unwrap();
    let one = git_stdout(root, &["rev-parse", "HEAD"]);
    let mut loader =
        StateLoader::with_layers(root, &[], &config, (Some(root.to_path_buf()), None, None));
    loader.load(&revisions(&one, &one)).unwrap();
    assert_eq!(
        loader.evaluations(),
        3,
        "two tasks and the failed file are evaluated once"
    );

    commit_files(
        root,
        &[("tasks/b.typ", &format!("{MARKER}// new\n"))],
        "two",
    );
    let parent = git_stdout(root, &["rev-parse", "HEAD~1"]);
    let tip = git_stdout(root, &["rev-parse", "HEAD"]);
    loader.load(&revisions(&parent, &tip)).unwrap();
    assert_eq!(
        loader.evaluations(),
        5,
        "only the changed file and the failed file are evaluated again"
    );
}

#[test]
fn a_pin_the_working_tree_lacks_seeds_nothing() {
    let text = pin_config("gone-lib");
    let dir = git_tempdir();
    let root = dir.path();
    commit_files(
        root,
        &[(".mindtape/config.toml", &text), ("tasks/a.typ", MARKER)],
        "seed",
    );
    let config = parse_config(&text).unwrap();
    let states = load(root, &config, &revisions("HEAD", "HEAD"));
    assert_eq!(
        states.staged.tasks.len(),
        1,
        "the state still loads its own task"
    );
}

#[test]
fn a_seeded_task_the_working_tree_lost_is_not_seeded() {
    let (dir, config, one) = module_repo();
    let root = dir.path();
    let base = working_base(root, &config);
    std::fs::remove_file(root.join("tasks/a.typ")).unwrap();
    let mut loader = state_loader(root, &base, &config);
    let states = loader.load(&revisions(&one, &one)).unwrap();
    assert_eq!(
        states.staged.tasks.len(),
        1,
        "the state still loads the file the working tree lost"
    );
}

#[cfg(unix)]
#[test]
fn a_state_ignores_a_symlinked_task_file() {
    let config_text = "[tasks]\nroots = [\"tasks\"]\n";
    let dir = crate::test_support::symlink_repo(
        &[
            (".mindtape/config.toml", config_text),
            ("shared/real.typ", MARKER),
        ],
        &[("tasks/a.typ", "../shared/real.typ")],
    );
    let root = dir.path();

    let config = parse_config(config_text).unwrap();
    let plan = revisions("HEAD", "HEAD");
    let states = load(root, &config, &plan);
    assert!(
        states.head.tasks.is_empty(),
        "a 120000 entry is not a task source"
    );
    assert!(states.staged.tasks.is_empty());
    assert!(states.head.errors.is_empty(), "{:?}", states.head.errors);
}

#[cfg(unix)]
#[test]
fn the_same_oid_under_two_paths_loads_two_tasks() {
    let config_text = "[tasks]\nroots = [\"tasks\"]\n";
    let dir = git_tempdir();
    let root = dir.path();
    commit_files(
        root,
        &[
            (".mindtape/config.toml", config_text),
            ("tasks/a.typ", MARKER),
            ("tasks/b.typ", MARKER),
        ],
        "seed",
    );
    let config = parse_config(config_text).unwrap();
    let one = git_stdout(root, &["rev-parse", "HEAD"]);
    let states = load(root, &config, &revisions(&one, &one));
    let ids: Vec<&str> = states
        .staged
        .tasks
        .iter()
        .map(|task| task.id.as_str())
        .collect();
    let files: Vec<String> = states
        .staged
        .tasks
        .iter()
        .map(|task| crate::slash_path(&task.file))
        .collect();
    assert_eq!(ids, ["a", "b"]);
    assert_eq!(files, ["tasks/a.typ", "tasks/b.typ"]);
}

#[test]
fn a_state_rename_loads_the_new_path_only() {
    let config_text = "[tasks]\nroots = [\"tasks\"]\n";
    let dir = git_tempdir();
    let root = dir.path();
    commit_files(
        root,
        &[
            (".mindtape/config.toml", config_text),
            ("tasks/a.typ", MARKER),
        ],
        "one",
    );
    let first = git_stdout(root, &["rev-parse", "HEAD"]);
    git(root, &["mv", "tasks/a.typ", "tasks/b.typ"]);
    git(root, &["commit", "-q", "-m", "rename"]);
    let second = git_stdout(root, &["rev-parse", "HEAD"]);
    let config = parse_config(config_text).unwrap();
    let mut loader =
        StateLoader::with_layers(root, &[], &config, (Some(root.to_path_buf()), None, None));
    let states = loader.load(&revisions(&first, &second)).unwrap();
    assert_eq!(states.staged.tasks.len(), 1);
    assert_eq!(states.staged.tasks[0].id, "b");
}

#[test]
fn an_xdg_layer_supplies_a_state_root() {
    let dir = git_tempdir();
    let root = dir.path();
    let xdg = root.join("xdg");
    write_file(
        &xdg,
        "mindtape/defaults.toml",
        "[tasks]\nroots = [\"backlog\"]\n",
    );
    commit_files(root, &[("backlog/parser.typ", MARKER)], "seed");
    let config = parse_config("").unwrap();
    let plan = revisions("HEAD", "HEAD");
    let mut loader = StateLoader::with_layers(root, &[], &config, (None, Some(xdg), None));
    let states = loader.load(&plan).unwrap();
    assert_eq!(states.staged.tasks[0].id, "parser");
}

#[test]
fn a_state_config_resolves_a_pack_and_its_disable() {
    let dir = git_tempdir();
    let root = dir.path();
    let text = "extends = [\"mindtape:tasks\"]\ndisable = [\"allow-only-declared-sections\"]\n\
                [tasks]\nroots = [\"tasks\"]\n";
    commit_files(
        root,
        &[(".mindtape/config.toml", text), ("tasks/a.typ", MARKER)],
        "seed",
    );
    let config = parse_config("").unwrap();
    let plan = revisions("HEAD", "HEAD");
    let states = load(root, &config, &plan);
    assert_eq!(states.staged.tasks[0].id, "a");
    assert!(
        states.staged.errors.is_empty(),
        "{:?}",
        states.staged.errors
    );
}
