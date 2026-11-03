#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use crate::test_support::commit_files;
use crate::test_support::git_as_tester as git;
use crate::test_support::git_stdout;
use crate::test_support::git_tempdir;
use crate::test_support::write_file;

use super::{CommitStates, StateLoader, StatePlan};
use crate::config::{Config, parse_config};

/// A raw task marker needing no prelude package.
const MARKER: &str =
    "#metadata((marker: \"mindtape.task\", fields: (title: \"Build\"), extras: (:)))\n";

#[test]
fn the_index_plan_reads_the_index_and_head_without_a_path_list() {
    let dir = git_tempdir();
    let root = dir.path();
    commit_files(root, &[("kept.typ", "old\n")], "seed");
    write_file(root, "kept.typ", "new\n");
    git(root, &["add", "kept.typ"]);

    let plan = StatePlan::index(root).unwrap();
    assert_eq!(
        (plan.head.rev.as_str(), plan.staged.rev.as_str()),
        ("HEAD", "")
    );
    assert!(plan.worktree);
    assert_eq!(plan.root_prefix, "");
    assert!(plan.head.differs.is_empty() && plan.staged.differs.is_empty());
}

#[test]
fn a_plan_before_the_first_commit_reads_the_empty_tree() {
    let dir = git_tempdir();
    let root = dir.path();
    write_file(root, "first.typ", "x\n");
    git(root, &["add", "first.typ"]);
    let plan = StatePlan::index(root).unwrap();
    assert_ne!(plan.head.rev, "HEAD");
    assert!(!plan.head.rev.is_empty(), "the empty tree has an id");
}

/// A prelude stub that ignores its arguments and always reports `title`.
fn titled_prelude(title: &str) -> String {
    format!(
        "#let task(body, ..meta) = [#metadata((marker: \"mindtape.task\", \
         fields: (title: \"{title}\"), extras: (:))) <mindtape>]\n"
    )
}

/// A task importing the pinned package at 0.3.0.
const PINNED_TASK: &str = "#import \"@local/mindtape:0.3.0\": *\n\n\
                           #show: task.with(title: \"Write the parser\")\n";

/// A plan with `head` and `staged` reading the named revisions, both with no
/// changed paths.
fn revisions(head: &str, staged: &str) -> StatePlan {
    StatePlan {
        root_prefix: String::new(),
        head: super::StateSource {
            rev: head.to_owned(),
            differs: Vec::new(),
        },
        staged: super::StateSource {
            rev: staged.to_owned(),
            differs: Vec::new(),
        },
        worktree: false,
    }
}

/// A plan whose project root sits in `sub/`.
fn sub_plan() -> StatePlan {
    let mut plan = revisions("HEAD", "HEAD");
    plan.root_prefix = "sub/".to_owned();
    plan
}

/// A loader whose config discovery ends at `repo`, for a subproject.
fn sub_loader<'a>(
    root: &'a std::path::Path,
    config: &'a Config,
    repo: &std::path::Path,
) -> StateLoader<'a> {
    StateLoader::with_layers(root, &[], config, (Some(repo.to_path_buf()), None, None))
}

#[test]
fn a_malformed_state_config_falls_back_and_is_reported() {
    let dir = git_tempdir();
    let root = dir.path();
    commit_files(
        root,
        &[(".mindtape/config.toml", "[typst"), ("tasks/a.typ", MARKER)],
        "broken config",
    );
    let config = parse_config("[tasks]\nroots = [\"tasks\"]\n").unwrap();
    let plan = revisions("HEAD", "HEAD");
    let states = load(root, &config, &plan);
    assert_eq!(
        states.head.tasks.len(),
        1,
        "the working roots still load it"
    );
    let error = states
        .head
        .errors
        .first()
        .expect("the broken config is reported");
    assert_eq!(
        error.file,
        std::path::PathBuf::from(".mindtape/config.toml")
    );
    assert!(
        error.message.contains("working tree's config"),
        "{}",
        error.message
    );
}

/// The `typst.toml` every stub package carries.
const PIN_MANIFEST: &str =
    "[package]\nname = \"mindtape\"\nversion = \"0.3.0\"\nentrypoint = \"prelude.typ\"\n";

/// A config with one `mindtape` pin at `path`.
fn pin_config(path: &str) -> String {
    format!(
        "[tasks]\nroots = [\"tasks\"]\n\n\
         [[typst.local-packages]]\nname = \"mindtape\"\npath = \"{path}\"\n"
    )
}

/// Load a plan over `config` with no working-tree tasks.
fn load(root: &std::path::Path, config: &Config, plan: &StatePlan) -> CommitStates {
    CommitStates::load(root, &[], config, plan).unwrap()
}

/// Assert the unchanged task resolved against the old pin in `head` and the
/// new one in `staged`.
fn assert_pin_titles(states: &CommitStates) {
    assert_eq!(states.head.tasks[0].title, "Old pin");
    assert_eq!(states.staged.tasks[0].title, "New pin");
}

/// A repository at the `old-lib` pin, its task importing the prelude.
fn old_pin_repo() -> tempfile::TempDir {
    let dir = git_tempdir();
    let root = dir.path();
    let old = titled_prelude("Old pin");
    commit_files(
        root,
        &[
            ("old-lib/typst.toml", PIN_MANIFEST),
            ("old-lib/prelude.typ", &old),
            (".mindtape/config.toml", &pin_config("old-lib")),
            ("tasks/parser.typ", PINNED_TASK),
        ],
        "old pin",
    );
    dir
}

#[test]
fn a_state_loads_its_unchanged_task_against_its_own_pin() {
    let dir = old_pin_repo();
    let root = dir.path();
    let new = titled_prelude("New pin");
    let moved = titled_prelude("Moved pin");
    commit_files(
        root,
        &[
            ("lib/typst.toml", PIN_MANIFEST),
            ("lib/prelude.typ", &new),
            // The old path stays behind at a third title, so `head` must read
            // the parent tree's bytes.
            ("old-lib/prelude.typ", &moved),
            (".mindtape/config.toml", &pin_config("lib")),
        ],
        "move pin",
    );

    let config = parse_config(&pin_config("lib")).unwrap();
    let plan = revisions("HEAD~1", "HEAD");
    let states = load(root, &config, &plan);
    assert_pin_titles(&states);
    assert!(states.head.errors.is_empty(), "{:?}", states.head.errors);
    assert!(
        states.staged.errors.is_empty(),
        "{:?}",
        states.staged.errors
    );
}

#[test]
fn an_index_state_loads_its_own_config_pin() {
    let dir = old_pin_repo();
    let root = dir.path();

    // The index moves the pin to `lib`, while the working tree stays on the
    // old one. The staged state must read the index's config and package.
    let new = titled_prelude("New pin");
    write_file(root, "lib/typst.toml", PIN_MANIFEST);
    write_file(root, "lib/prelude.typ", &new);
    write_file(root, ".mindtape/config.toml", pin_config("lib"));
    git(root, &["add", "-A"]);
    // The working tree keeps both the old config and different bytes at the
    // index's pin, so the staged state must read the index's package.
    write_file(root, ".mindtape/config.toml", pin_config("old-lib"));
    write_file(root, "lib/prelude.typ", titled_prelude("Working pin"));

    let config = parse_config(&pin_config("old-lib")).unwrap();
    let plan = StatePlan::index(root).unwrap();
    let states = load(root, &config, &plan);
    assert_pin_titles(&states);
    assert!(
        states.staged.errors.is_empty(),
        "{:?}",
        states.staged.errors
    );
}

/// A source that fails evaluation.
const BROKEN: &str = "#let x = (\n";

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
    ];
    for (text, expected) in cases {
        let dir = git_tempdir();
        let root = dir.path();
        commit_files(
            root,
            &[(".mindtape/config.toml", text), ("tasks/sub/x.typ", MARKER)],
            "seed",
        );
        let config = parse_config(text).unwrap();
        let plan = StatePlan::index(root).unwrap();
        let clean_base = crate::loader::load_project(root, &config).tasks;
        let clean = CommitStates::load(root, &clean_base, &config, &plan).unwrap();

        // A working-tree edit that stays out of the index: the state bytes do
        // not change, so a seeded state must not depend on it.
        write_file(root, "tasks/sub/x.typ", format!("{MARKER}// dirty\n"));
        let dirty_base = crate::loader::load_project(root, &config).tasks;
        let dirty = CommitStates::load(root, &dirty_base, &config, &plan).unwrap();

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
    let mut loader = StateLoader::new(root, &[], &config);
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

#[cfg(unix)]
#[test]
fn a_state_follows_a_symlinked_task_file() {
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
    assert_eq!(states.head.tasks[0].id, "a");
    assert_eq!(states.head.tasks[0].title, "Build");
    assert!(states.head.errors.is_empty(), "{:?}", states.head.errors);
}
