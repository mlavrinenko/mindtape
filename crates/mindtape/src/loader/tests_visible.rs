#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use std::path::Path;
use std::process::Command;

use super::load_project;
use super::tests::{TASK_BODY, load_ok, one_error, setup};
use crate::config::Config;
use crate::parse_config;
use crate::test_support::{project_with_file, tempdir, write_file};

/// Run `git <args>` in `dir` with the developer's Git configuration out of
/// reach, asserting success. A test about ignore semantics must read only the
/// fixture's own `.gitignore`, never a global `core.excludesFile`.
pub(super) fn git(dir: &Path, args: &[&str]) {
    let output = Command::new("git")
        .args([
            "-c",
            "user.name=t",
            "-c",
            "user.email=t@t",
            "-c",
            "commit.gpgsign=false",
        ])
        .args(args)
        .current_dir(dir)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", dir.join("no-global-git-config"))
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git {args:?} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

/// A fresh tempdir that is an empty Git working tree.
pub(super) fn git_repo() -> tempfile::TempDir {
    let dir = tempdir();
    git(dir.path(), &["init", "-q"]);
    dir
}

/// The ids `load_project` reports, in its id order.
fn ids(outcome: &super::LoadOutcome) -> Vec<&str> {
    outcome.tasks.iter().map(|task| task.id.as_str()).collect()
}

/// Git lists tracked files and untracked files it does not ignore. A tracked
/// file under an ignore rule stays a task, and an untracked ignored one does
/// not.
#[test]
fn git_lists_tracked_and_untracked_sources_but_not_untracked_ignored_ones() {
    let dir = git_repo();
    write_file(dir.path(), "tasks/.gitignore", "ignored/\n");
    write_file(dir.path(), "tasks/visible.typ", TASK_BODY);
    write_file(dir.path(), "tasks/ignored/tracked.typ", TASK_BODY);
    git(dir.path(), &["add", "-A"]);
    git(dir.path(), &["add", "-f", "tasks/ignored/tracked.typ"]);
    git(dir.path(), &["commit", "-q", "-m", "seed"]);
    write_file(dir.path(), "tasks/new.typ", TASK_BODY);
    write_file(dir.path(), "tasks/ignored/untracked.typ", TASK_BODY);

    let outcome = load_ok(dir.path(), &Config::default());
    assert_eq!(ids(&outcome), ["new", "tracked", "visible"]);
}

/// A tracked file deleted from the working tree is not a task. The policy's
/// regular-file kind filters the listing, so the missing file cannot reach the
/// harvest. Commit states still list it, and that path stays untouched.
#[test]
fn git_does_not_load_a_tracked_file_deleted_from_the_working_tree() {
    let dir = git_repo();
    write_file(dir.path(), "tasks/gone.typ", TASK_BODY);
    git(dir.path(), &["add", "-A"]);
    git(dir.path(), &["commit", "-q", "-m", "seed"]);
    std::fs::remove_file(dir.path().join("tasks/gone.typ")).unwrap();

    let outcome = load_ok(dir.path(), &Config::default());
    assert!(outcome.tasks.is_empty(), "{:?}", outcome.tasks);
}

/// An ignored root is not walked: its tracked files stay visible, one warning
/// says why, and a root that is not ignored earns no warning.
#[test]
fn an_ignored_root_lists_only_tracked_files_and_warns_once() {
    let dir = git_repo();
    write_file(dir.path(), ".gitignore", "target/\n");
    write_file(dir.path(), "target/tracked.typ", TASK_BODY);
    git(dir.path(), &["add", ".gitignore"]);
    git(dir.path(), &["add", "-f", "target/tracked.typ"]);
    git(dir.path(), &["commit", "-q", "-m", "seed"]);
    write_file(dir.path(), "target/untracked.typ", TASK_BODY);
    write_file(dir.path(), "tasks/visible.typ", TASK_BODY);
    let config = parse_config("[tasks]\nroots = [\"target\", \"tasks\"]\n").unwrap();

    let outcome = load_ok(dir.path(), &config);
    assert_eq!(ids(&outcome), ["tracked", "visible"]);
    assert_eq!(outcome.warnings.len(), 1, "{:?}", outcome.warnings);
    let warning = outcome.warnings.first().unwrap();
    assert_eq!(warning.file, Path::new("target"));
    assert!(
        warning
            .message
            .contains("only its tracked files are visible"),
        "{}",
        warning.message
    );
}

/// Git's listing still includes the reserved files the policy deliberately
/// skips, so a misfiled task's warning survives the change from a walk to Git.
/// A template beside it earns none.
#[test]
fn git_discovery_keeps_the_misfiled_task_warning() {
    let dir = git_repo();
    let config = parse_config("[tasks]\nroots = [\".\"]\n").unwrap();
    write_file(
        dir.path(),
        ".mindtape/task.typ",
        "#show: task.with(title: \"Misfiled\")\n",
    );
    write_file(
        dir.path(),
        ".mindtape/templates/task.typ",
        "#show: task.with(title: \"Template\")\n",
    );

    let outcome = load_ok(dir.path(), &config);
    assert!(outcome.tasks.is_empty(), "{:?}", outcome.tasks);
    assert_eq!(outcome.warnings.len(), 1, "{:?}", outcome.warnings);
    assert_eq!(
        outcome.warnings.first().unwrap().file,
        Path::new(".mindtape/task.typ")
    );
}

/// Git prints paths from the repository root. A project nested in a larger
/// repository must strip its own prefix before making them project-relative.
#[test]
fn a_project_nested_in_a_larger_repository_keeps_project_relative_paths() {
    let outer = git_repo();
    let project = outer.path().join("nested/project");
    std::fs::create_dir_all(project.join("tasks")).unwrap();
    write_file(&project, "tasks/a.typ", TASK_BODY);

    let outcome = load_ok(&project, &Config::default());
    assert_eq!(ids(&outcome), ["a"]);
    assert_eq!(
        outcome.tasks.first().unwrap().file,
        Path::new("tasks/a.typ")
    );
}

/// Only a non-worktree root or a missing `git` falls back. A Git failure that
/// is neither is a load error that specifies the root, not a silent empty scan.
#[test]
fn another_git_failure_is_a_load_error_naming_the_root() {
    let dir = project_with_file(setup(), ".git", "not a git directory\n");
    write_file(dir.path(), "tasks/a.typ", TASK_BODY);

    let outcome = load_project(dir.path(), &Config::default());
    let error = one_error(&outcome);
    assert_eq!(error.file, Path::new("tasks"));
    assert!(error.message.contains("tasks"), "{}", error.message);
}

/// A project outside Git falls back to the policy walk, which never follows a
/// directory symlink (a cycle or a shared tree would make the scan unbounded).
#[cfg(unix)]
#[test]
fn a_non_git_project_walks_without_following_directory_symlinks() {
    let dir = project_with_file(setup(), "tasks/a.typ", TASK_BODY);
    write_file(dir.path(), "elsewhere/b.typ", TASK_BODY);
    std::os::unix::fs::symlink(
        dir.path().join("elsewhere"),
        dir.path().join("tasks/linked"),
    )
    .unwrap();

    let outcome = load_ok(dir.path(), &Config::default());
    assert_eq!(ids(&outcome), ["a"]);
}

/// `git ls-files -z` paths are bytes. A non-UTF-8 name must survive decoding
/// instead of being silently dropped or mangled. Typst refuses to evaluate such
/// a path, so the error is where the loader's preserved path shows up.
#[cfg(unix)]
#[test]
fn a_non_utf8_git_path_is_preserved_in_the_load_error() {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;

    let dir = git_repo();
    let rel = Path::new("tasks").join(OsStr::from_bytes(b"caf\xff.typ"));
    write_file(dir.path(), &rel, TASK_BODY);

    let outcome = load_project(dir.path(), &Config::default());
    let error = one_error(&outcome);
    assert_eq!(error.file, rel);
}

/// A root of `.` excludes `docs`, and a narrower root still loads a task
/// there. The narrower root's tag wins.
#[test]
fn a_root_excludes_docs_and_a_narrower_root_loads_it() {
    let dir = setup();
    let config = parse_config(
        "[[tasks.roots]]\npath = \".\"\nname = \"home\"\nexclude = [\"docs/**\"]\n\n\
         [[tasks.roots]]\npath = \"docs/tasks\"\nname = \"docs\"\nderive-tags = true\n",
    )
    .unwrap();
    write_file(dir.path(), "docs/example.typ", TASK_BODY);
    write_file(dir.path(), "docs/tasks/real.typ", TASK_BODY);
    write_file(dir.path(), "tasks/other.typ", TASK_BODY);

    let outcome = load_ok(dir.path(), &config);
    assert_eq!(outcome.tasks.len(), 2, "{:?}", outcome.tasks);
    let real = &outcome.tasks[1];
    assert_eq!(real.id, "real");
    assert_eq!(real.derived_tags, ["docs"]);
}
