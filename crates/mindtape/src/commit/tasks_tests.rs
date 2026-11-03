#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use crate::test_support::commit_files;
use crate::test_support::git_as_tester as git;
use crate::test_support::git_tempdir;
use crate::test_support::write_file;

use super::{CommitStates, StateLoader, StatePlan};
use crate::config::{Config, parse_config};
use crate::model::Task;

/// A raw task marker needing no prelude package.
pub(crate) const MARKER: &str =
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
pub(crate) fn titled_prelude(title: &str) -> String {
    format!(
        "#let task(body, ..meta) = [#metadata((marker: \"mindtape.task\", \
         fields: (title: \"{title}\"), extras: (:))) <mindtape>]\n"
    )
}

/// A task importing the pinned package at 0.3.0.
pub(crate) const PINNED_TASK: &str = "#import \"@local/mindtape:0.3.0\": *\n\n\
                           #show: task.with(title: \"Write the parser\")\n";

/// A plan with `head` and `staged` reading the named revisions, both with no
/// changed paths.
pub(crate) fn revisions(head: &str, staged: &str) -> StatePlan {
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
pub(crate) fn sub_plan() -> StatePlan {
    let mut plan = revisions("HEAD", "HEAD");
    plan.root_prefix = "sub/".to_owned();
    plan
}

/// A loader whose config discovery ends at `repo`, for a subproject.
pub(crate) fn sub_loader<'a>(
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
pub(crate) const PIN_MANIFEST: &str =
    "[package]\nname = \"mindtape\"\nversion = \"0.3.0\"\nentrypoint = \"prelude.typ\"\n";

/// A config with one `mindtape` pin at `path`.
pub(crate) fn pin_config(path: &str) -> String {
    format!(
        "[tasks]\nroots = [\"tasks\"]\n\n\
         [[typst.local-packages]]\nname = \"mindtape\"\npath = \"{path}\"\n"
    )
}

/// Load a plan over `config` with no working-tree tasks, bounded to `root` so
/// no developer config layer applies.
pub(crate) fn load(root: &std::path::Path, config: &Config, plan: &StatePlan) -> CommitStates {
    load_from(root, &[], config, plan)
}

/// [`load`] over a working-tree task list.
pub(crate) fn load_from(
    root: &std::path::Path,
    base: &[Task],
    config: &Config,
    plan: &StatePlan,
) -> CommitStates {
    let mut loader =
        StateLoader::with_layers(root, base, config, (Some(root.to_path_buf()), None, None));
    loader.load(plan).unwrap()
}

/// Assert the unchanged task resolved against the old pin in `head` and the
/// new one in `staged`.
pub(crate) fn assert_pin_titles(states: &CommitStates) {
    assert_eq!(states.head.tasks[0].title, "Old pin");
    assert_eq!(states.staged.tasks[0].title, "New pin");
}

/// A repository at the `old-lib` pin, its task importing the prelude.
pub(crate) fn old_pin_repo() -> tempfile::TempDir {
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
