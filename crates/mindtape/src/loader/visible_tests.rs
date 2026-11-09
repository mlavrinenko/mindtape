//! The destination probe, the per-root and project-wide listings, and the
//! filesystem fallback.
#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use std::path::{Path, PathBuf};
use std::process::Command;

use super::{
    NotVisible, project_is_worktree, root_task_sources, visible_task_sources, visible_typ_files,
    would_be_visible,
};
use crate::config::Config;
use crate::parse_config;
use crate::test_support::{tempdir, write_file};

/// Run `git <args>` in `dir` with the developer's Git configuration out of
/// reach, so only the fixture's own `.gitignore` decides visibility.
fn git(dir: &Path, args: &[&str]) {
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

fn git_repo() -> tempfile::TempDir {
    let dir = tempdir();
    git(dir.path(), &["init", "-q"]);
    dir
}

fn config(text: &str) -> Config {
    parse_config(text).unwrap()
}

fn abs(root: &Path, rel: &str) -> PathBuf {
    root.join(rel)
}

#[test]
fn a_new_typ_file_under_a_root_would_be_visible() {
    let dir = git_repo();
    std::fs::create_dir_all(dir.path().join("tasks")).unwrap();

    assert_eq!(
        would_be_visible(
            dir.path(),
            &Config::default(),
            &abs(dir.path(), "tasks/new.typ")
        ),
        Ok(())
    );
}

#[test]
fn an_ignored_new_file_is_refused_while_a_tracked_one_is_allowed() {
    let dir = git_repo();
    write_file(dir.path(), ".gitignore", "target/\n");
    write_file(dir.path(), "target/tracked.typ", "");
    git(dir.path(), &["add", ".gitignore"]);
    git(dir.path(), &["add", "-f", "target/tracked.typ"]);
    git(dir.path(), &["commit", "-q", "-m", "seed"]);
    let config = config("[tasks]\nroots = [\".\"]\n");

    assert_eq!(
        would_be_visible(dir.path(), &config, &abs(dir.path(), "target/new.typ")),
        Err(NotVisible::Ignored)
    );
    assert_eq!(
        would_be_visible(dir.path(), &config, &abs(dir.path(), "target/tracked.typ")),
        Ok(())
    );
}

#[test]
fn reserved_and_template_destinations_are_refused() {
    let dir = tempdir();
    let config = config("[tasks]\nroots = [\".\"]\n[templates]\ndirs = [\"tmpl\"]\n");

    for rel in [".mindtape/templates/new.typ", "tmpl/new.typ"] {
        assert_eq!(
            would_be_visible(dir.path(), &config, &abs(dir.path(), rel)),
            Err(NotVisible::Reserved),
            "{rel}"
        );
    }
}

#[test]
fn dot_dot_resolves_before_the_policy_answers() {
    let dir = tempdir();
    std::fs::create_dir_all(dir.path().join("tasks")).unwrap();

    assert_eq!(
        would_be_visible(
            dir.path(),
            &Config::default(),
            &abs(dir.path(), "tasks/../tasks/new.typ")
        ),
        Ok(())
    );
    assert_eq!(
        would_be_visible(
            dir.path(),
            &Config::default(),
            &abs(dir.path(), "tasks/../elsewhere/new.typ")
        ),
        Err(NotVisible::OutsideRoots)
    );
}

#[test]
fn a_non_typ_or_directory_destination_is_refused() {
    let dir = tempdir();
    std::fs::create_dir_all(dir.path().join("tasks/box.typ")).unwrap();

    assert_eq!(
        would_be_visible(
            dir.path(),
            &Config::default(),
            &abs(dir.path(), "tasks/notes.md")
        ),
        Err(NotVisible::NotTyp)
    );
    assert_eq!(
        would_be_visible(
            dir.path(),
            &Config::default(),
            &abs(dir.path(), "tasks/box.typ")
        ),
        Err(NotVisible::NotAFile)
    );
}

#[cfg(unix)]
#[test]
fn a_symlink_destination_is_refused() {
    let dir = tempdir();
    write_file(dir.path(), "tasks/real.typ", "");
    std::os::unix::fs::symlink("real.typ", dir.path().join("tasks/link.typ")).unwrap();

    assert_eq!(
        would_be_visible(
            dir.path(),
            &Config::default(),
            &abs(dir.path(), "tasks/link.typ")
        ),
        Err(NotVisible::Symlink)
    );
}

#[test]
fn a_tracked_deleted_destination_is_visible() {
    let dir = git_repo();
    write_file(dir.path(), ".gitignore", "target/\n");
    write_file(dir.path(), "target/tracked.typ", "");
    git(dir.path(), &["add", ".gitignore"]);
    git(dir.path(), &["add", "-f", "target/tracked.typ"]);
    git(dir.path(), &["commit", "-q", "-m", "seed"]);
    std::fs::remove_file(dir.path().join("target/tracked.typ")).unwrap();
    let config = config("[tasks]\nroots = [\".\"]\n");

    assert_eq!(
        would_be_visible(dir.path(), &config, &abs(dir.path(), "target/tracked.typ")),
        Ok(()),
        "a write there recreates an index-tracked file Git still lists"
    );
}

/// A whitelist rule can leave a root's own name matching a broad ignore
/// pattern while its contents stay visible. The root is then not an ignored
/// root, and its untracked files must not be hidden behind a warning.
#[test]
fn a_whitelisted_root_is_not_an_ignored_root() {
    let dir = git_repo();
    write_file(dir.path(), ".gitignore", "*\n!tasks/\n!tasks/*.typ\n");
    write_file(dir.path(), "tasks/tracked.typ", "");
    git(
        dir.path(),
        &["add", "-f", ".gitignore", "tasks/tracked.typ"],
    );
    git(dir.path(), &["commit", "-q", "-m", "seed"]);
    let config = config("[tasks]\nroots = [\"tasks\"]\n");

    let files = visible_task_sources(dir.path(), &config).unwrap();
    assert_eq!(files, [abs(dir.path(), "tasks/tracked.typ")]);
    let outcome = crate::load_project(dir.path(), &config);
    assert!(outcome.warnings.is_empty(), "{:?}", outcome.warnings);
}

/// An absolute pathspec through a symlink makes Git reject the query, so the
/// probe runs from the nearest existing parent instead and still answers.
#[cfg(unix)]
#[test]
fn an_ignored_destination_through_a_symlink_is_refused() {
    let dir = git_repo();
    write_file(dir.path(), ".gitignore", "target/\n");
    std::fs::create_dir_all(dir.path().join("target/sub")).unwrap();
    std::os::unix::fs::symlink(dir.path().join("target"), dir.path().join("link")).unwrap();
    let config = config("[tasks]\nroots = [\".\"]\n");

    assert_eq!(
        would_be_visible(dir.path(), &config, &abs(dir.path(), "link/sub/new.typ")),
        Err(NotVisible::Ignored)
    );
}

#[test]
fn a_missing_ignored_root_still_refuses_a_new_destination() {
    let dir = git_repo();
    write_file(dir.path(), ".gitignore", "target/\n");
    git(dir.path(), &["add", ".gitignore"]);
    git(dir.path(), &["commit", "-q", "-m", "seed"]);
    let config = config("[tasks]\nroots = [\"target\"]\n");

    assert_eq!(
        would_be_visible(dir.path(), &config, &abs(dir.path(), "target/new.typ")),
        Err(NotVisible::Ignored)
    );
    let outcome = crate::load_project(dir.path(), &config);
    assert_eq!(outcome.warnings.len(), 1, "{:?}", outcome.warnings);
}

#[test]
fn a_non_git_project_accepts_a_new_destination_under_a_root() {
    let dir = tempdir();
    let config = config("[tasks]\nroots = [\".\"]\n");

    assert_eq!(
        would_be_visible(dir.path(), &config, &abs(dir.path(), "target/new.typ")),
        Ok(())
    );
}

#[test]
fn root_task_sources_lists_only_that_root() {
    let dir = git_repo();
    write_file(dir.path(), "tasks/.gitignore", "hidden.typ\n");
    write_file(dir.path(), "tasks/tracked.typ", "");
    write_file(dir.path(), "tasks/hidden.typ", "");
    write_file(dir.path(), "tasks/new.typ", "");
    write_file(dir.path(), "extra/other.typ", "");
    git(
        dir.path(),
        &[
            "add",
            "tasks/.gitignore",
            "tasks/tracked.typ",
            "extra/other.typ",
        ],
    );
    git(dir.path(), &["add", "-f", "tasks/hidden.typ"]);
    git(dir.path(), &["commit", "-q", "-m", "seed"]);
    let config = config("[tasks]\nroots = [\"tasks\", \"extra\"]\n");

    let files = root_task_sources(dir.path(), &config, &config.tasks.roots[0]).unwrap();
    assert_eq!(
        files,
        [
            abs(dir.path(), "tasks/hidden.typ"),
            abs(dir.path(), "tasks/new.typ"),
            abs(dir.path(), "tasks/tracked.typ"),
        ]
    );
}

#[cfg(unix)]
#[test]
fn visible_task_sources_keeps_a_non_utf8_path() {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;

    let dir = git_repo();
    let rel = Path::new("tasks").join(OsStr::from_bytes(b"caf\xff.typ"));
    write_file(dir.path(), &rel, "");

    let files = visible_task_sources(dir.path(), &Config::default()).unwrap();
    assert_eq!(files, [dir.path().join(rel)]);
}

#[test]
fn visible_task_sources_unions_overlapping_roots_once() {
    let dir = tempdir();
    write_file(dir.path(), "tasks/a.typ", "");
    write_file(dir.path(), "tasks/extra/b.typ", "");
    let config = config("[tasks]\nroots = [\"tasks\", \"tasks/extra\"]\n");

    let files = visible_task_sources(dir.path(), &config).unwrap();
    assert_eq!(
        files,
        [
            abs(dir.path(), "tasks/a.typ"),
            abs(dir.path(), "tasks/extra/b.typ"),
        ]
    );
}

#[test]
fn project_is_worktree_tells_git_from_the_fallback_walk() {
    let dir = git_repo();
    assert!(project_is_worktree(dir.path()));

    let plain = tempdir();
    assert!(!project_is_worktree(plain.path()));
}

#[test]
fn visible_typ_files_follows_git_and_skips_reserved_metadata() {
    let dir = git_repo();
    write_file(dir.path(), ".gitignore", "target/\n");
    write_file(dir.path(), "lib/x.typ", "");
    write_file(dir.path(), "docs/a.typ", "");
    write_file(dir.path(), ".mindtape/templates/task.typ", "");
    write_file(dir.path(), "target/tracked.typ", "");
    write_file(dir.path(), "gone.typ", "");
    git(
        dir.path(),
        &["add", ".gitignore", "lib", "docs", ".mindtape", "gone.typ"],
    );
    git(dir.path(), &["add", "-f", "target/tracked.typ"]);
    git(dir.path(), &["commit", "-q", "-m", "seed"]);
    std::fs::remove_file(dir.path().join("gone.typ")).unwrap();
    write_file(dir.path(), "target/ignored.typ", "");
    write_file(dir.path(), "notes/new.typ", "");

    let files = visible_typ_files(dir.path()).unwrap();
    assert_eq!(
        files,
        [
            abs(dir.path(), "docs/a.typ"),
            abs(dir.path(), "lib/x.typ"),
            abs(dir.path(), "notes/new.typ"),
            abs(dir.path(), "target/tracked.typ"),
        ]
    );
}

#[test]
fn visible_typ_files_strips_a_nested_projects_prefix() {
    let outer = git_repo();
    let project = outer.path().join("nested/project");
    write_file(&project, "lib/x.typ", "");
    write_file(outer.path(), "elsewhere/y.typ", "");

    let files = visible_typ_files(&project).unwrap();
    assert_eq!(files, [abs(&project, "lib/x.typ")]);
}

#[cfg(unix)]
#[test]
fn the_non_git_walk_skips_reserved_dirs_and_directory_symlinks() {
    let dir = tempdir();
    write_file(dir.path(), "lib/x.typ", "");
    write_file(dir.path(), "target/gen.typ", "");
    write_file(dir.path(), ".mindtape/templates/task.typ", "");
    write_file(dir.path(), "elsewhere/y.typ", "");
    std::os::unix::fs::symlink(dir.path().join("elsewhere"), dir.path().join("linked")).unwrap();

    let files = visible_typ_files(dir.path()).unwrap();
    assert_eq!(
        files,
        [
            abs(dir.path(), "elsewhere/y.typ"),
            abs(dir.path(), "lib/x.typ"),
            abs(dir.path(), "target/gen.typ"),
        ]
    );
}

#[test]
fn a_git_failure_other_than_a_fallback_is_reported_for_each_listing() {
    let dir = tempdir();
    write_file(dir.path(), ".git", "not a git directory\n");
    write_file(dir.path(), "tasks/a.typ", "");
    let config = Config::default();

    let error = root_task_sources(dir.path(), &config, &config.tasks.roots[0]).unwrap_err();
    assert_eq!(error.file, Path::new("tasks"));

    let error = visible_task_sources(dir.path(), &config).unwrap_err();
    assert_eq!(error.file, Path::new("tasks"));

    let error = visible_typ_files(dir.path()).unwrap_err();
    assert!(error.message.contains("project root"), "{}", error.message);
}

#[test]
fn a_refusal_explains_itself() {
    assert!(NotVisible::Ignored.to_string().contains("ignored by git"));
    assert!(NotVisible::Reserved.to_string().contains("reserved"));
}
