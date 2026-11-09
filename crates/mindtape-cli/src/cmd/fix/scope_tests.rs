#![allow(clippy::unwrap_used, clippy::disallowed_methods)]

use std::path::{Path, PathBuf};

use mindtape::test_support::{git_as_tester, git_init, tempdir, write_file};
use mindtape::{Config, RootSpec};

use super::collect;

fn write_typ(root: &Path, rel: &str) {
    write_file(root, rel, "#import \"@local/mindtape:0.4.0\": *\n");
}

fn shown(files: &[PathBuf], root: &Path) -> Vec<String> {
    files
        .iter()
        .map(|path| {
            path.strip_prefix(root)
                .unwrap_or(path)
                .to_string_lossy()
                .replace('\\', "/")
        })
        .collect()
}

fn config(roots: &[&str], templates: &[&str]) -> Config {
    let mut config = Config::default();
    config.tasks.roots = roots
        .iter()
        .map(|root| RootSpec::Short((*root).to_owned()))
        .collect();
    config.templates.dirs = templates.iter().map(|dir| (*dir).to_owned()).collect();
    config
}

/// An empty Git repository, so the listing takes its Git mode.
fn git_project() -> tempfile::TempDir {
    let dir = tempdir();
    git_init(dir.path());
    dir
}

/// Stage everything and commit it with the throwaway test identity.
fn commit(dir: &Path) {
    git_as_tester(dir, &["add", "-A"]);
    git_as_tester(dir, &["commit", "-q", "-m", "seed"]);
}

#[test]
fn default_scope_is_task_sources_and_project_template_dirs() {
    let dir = tempdir();
    let root = dir.path();
    for rel in ["tasks/a.typ", "tasks/nested/b.typ", "templates/c.typ"] {
        write_typ(root, rel);
    }
    write_typ(root, "elsewhere/d.typ");
    let outside = tempdir();
    write_typ(outside.path(), "e.typ");
    let outside_dir = outside.path().to_string_lossy().into_owned();

    let files = collect(
        root,
        &config(&["tasks"], &["templates", &outside_dir]),
        false,
    )
    .unwrap();
    assert_eq!(
        shown(&files, root),
        ["tasks/a.typ", "tasks/nested/b.typ", "templates/c.typ"]
    );
}

#[test]
fn all_lists_every_project_typ_and_still_adds_templates() {
    let dir = tempdir();
    let root = dir.path();
    for rel in [
        "tasks/a.typ",
        "lib/x.typ",
        "docs/guide.typ",
        "target/d.typ",
        ".mindtape/cache/e.typ",
        ".mindtape/templates/f.typ",
    ] {
        write_typ(root, rel);
    }

    let files = collect(root, &config(&["tasks"], &[".mindtape/templates"]), true).unwrap();
    assert_eq!(
        shown(&files, root),
        [
            ".mindtape/templates/f.typ",
            "docs/guide.typ",
            "lib/x.typ",
            "tasks/a.typ"
        ]
    );
}

#[test]
fn a_tracked_ignored_task_source_is_in_both_scopes() {
    let dir = git_project();
    let root = dir.path();
    write_file(root, ".gitignore", "tasks/ignored.typ\n");
    write_typ(root, "tasks/ignored.typ");
    write_typ(root, "tasks/visible.typ");
    git_as_tester(root, &["add", "-f", "tasks/ignored.typ"]);
    commit(root);

    let config = config(&["tasks"], &[]);
    let default = collect(root, &config, false).unwrap();
    assert_eq!(
        shown(&default, root),
        ["tasks/ignored.typ", "tasks/visible.typ"]
    );

    let all = collect(root, &config, true).unwrap();
    assert_eq!(
        shown(&all, root),
        ["tasks/ignored.typ", "tasks/visible.typ"]
    );
}

#[test]
fn an_untracked_ignored_file_is_in_neither_scope() {
    let dir = git_project();
    let root = dir.path();
    write_file(root, ".gitignore", "target/\n");
    write_typ(root, "tasks/real.typ");
    write_typ(root, "target/ignored.typ");
    commit(root);

    let config = config(&["."], &[]);
    let default = collect(root, &config, false).unwrap();
    assert_eq!(shown(&default, root), ["tasks/real.typ"]);

    let all = collect(root, &config, true).unwrap();
    assert_eq!(shown(&all, root), ["tasks/real.typ"]);
}

#[test]
fn a_tracked_deleted_file_is_in_neither_scope() {
    let dir = git_project();
    let root = dir.path();
    write_typ(root, "tasks/gone.typ");
    write_typ(root, "tasks/real.typ");
    commit(root);
    std::fs::remove_file(root.join("tasks/gone.typ")).unwrap();

    let config = config(&["tasks"], &[]);
    let default = collect(root, &config, false).unwrap();
    assert_eq!(shown(&default, root), ["tasks/real.typ"]);

    let all = collect(root, &config, true).unwrap();
    assert_eq!(shown(&all, root), ["tasks/real.typ"]);
}

#[test]
fn a_non_git_all_skips_target_and_scans_configured_templates() {
    let dir = tempdir();
    let root = dir.path();
    for rel in [
        "tasks/real.typ",
        "target/generated.typ",
        ".mindtape/templates/task.typ",
        "notes.typ",
    ] {
        write_typ(root, rel);
    }
    let config = config(&["tasks"], &[".mindtape/templates"]);

    let default = collect(root, &config, false).unwrap();
    assert_eq!(
        shown(&default, root),
        [".mindtape/templates/task.typ", "tasks/real.typ"]
    );

    let all = collect(root, &config, true).unwrap();
    assert_eq!(
        shown(&all, root),
        [
            ".mindtape/templates/task.typ",
            "notes.typ",
            "tasks/real.typ"
        ]
    );
}

#[cfg(unix)]
#[test]
fn template_walk_skips_reserved_dirs_and_collects_symlinked_files() {
    use std::os::unix::fs::symlink;

    let dir = tempdir();
    let root = dir.path();
    for rel in [
        "templates/a.typ",
        "templates/sub/b.typ",
        "templates/.git/c.typ",
        "templates/target/d.typ",
        "templates/.mindtape/cache/e.typ",
    ] {
        write_typ(root, rel);
    }
    let outside = tempdir();
    write_typ(outside.path(), "shared.typ");
    symlink(
        outside.path().join("shared.typ"),
        root.join("templates/linked.typ"),
    )
    .unwrap();
    symlink(outside.path(), root.join("templates/linked-dir")).unwrap();

    let files = collect(root, &config(&["tasks"], &["templates"]), false).unwrap();
    assert_eq!(
        shown(&files, root),
        [
            "templates/a.typ",
            "templates/linked.typ",
            "templates/sub/b.typ"
        ]
    );
}

#[test]
fn listing_diag_keeps_the_error_place_and_hints() {
    let error = mindtape::LoadError {
        file: PathBuf::from("tasks"),
        line: Some(3),
        column: Some(4),
        message: "can't list".to_owned(),
        hints: vec!["check git".to_owned()],
        ..Default::default()
    };
    let text = crate::diag::plain(&super::listing_diag(error));
    assert!(text.contains("tasks:3:4"), "{text}");
    assert!(text.contains("can't list"), "{text}");
    assert!(text.contains("check git"), "{text}");
}

#[cfg(unix)]
#[test]
fn a_symlinked_directory_is_never_followed_and_a_symlinked_file_only_appears_in_all() {
    use std::os::unix::fs::symlink;

    let dir = tempdir();
    let root = dir.path();
    write_typ(root, "tasks/real.typ");
    let outside = tempdir();
    write_typ(outside.path(), "shared.typ");
    symlink(outside.path(), root.join("tasks/linked-dir")).unwrap();
    symlink(
        outside.path().join("shared.typ"),
        root.join("tasks/linked.typ"),
    )
    .unwrap();

    let config = config(&["tasks"], &[]);
    let default = collect(root, &config, false).unwrap();
    assert_eq!(shown(&default, root), ["tasks/real.typ"]);

    let all = collect(root, &config, true).unwrap();
    assert_eq!(shown(&all, root), ["tasks/linked.typ", "tasks/real.typ"]);
}
