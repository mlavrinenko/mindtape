#![allow(clippy::unwrap_used)]

use std::path::{Path, PathBuf};

use mindtape::{Config, RootSpec};

use super::collect;

fn write_typ(root: &Path, rel: &str) {
    let path = root.join(rel);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, "#import \"@local/mindtape:0.4.0\": *\n").unwrap();
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

#[test]
fn default_scope_is_task_roots_and_project_template_dirs() {
    let dir = mindtape::test_support::tempdir();
    let root = dir.path();
    for rel in ["tasks/a.typ", "tasks/nested/b.typ", "templates/c.typ"] {
        write_typ(root, rel);
    }
    write_typ(root, "elsewhere/d.typ");
    let outside = mindtape::test_support::tempdir();
    write_typ(outside.path(), "e.typ");
    let outside_dir = outside.path().to_string_lossy().into_owned();

    let files = collect(
        root,
        &config(&["tasks"], &["templates", &outside_dir]),
        false,
    );
    assert_eq!(
        shown(&files, root),
        ["tasks/a.typ", "tasks/nested/b.typ", "templates/c.typ"]
    );
}

#[test]
fn all_walks_the_project_and_skips_the_generated_trees() {
    let dir = mindtape::test_support::tempdir();
    let root = dir.path();
    for rel in [
        "tasks/a.typ",
        "elsewhere/b.typ",
        ".git/c.typ",
        "target/d.typ",
        ".mindtape/cache/e.typ",
        ".mindtape/templates/f.typ",
    ] {
        write_typ(root, rel);
    }

    let files = collect(root, &config(&["tasks"], &[]), true);
    assert_eq!(
        shown(&files, root),
        [
            ".mindtape/templates/f.typ",
            "elsewhere/b.typ",
            "tasks/a.typ"
        ]
    );
}

#[cfg(unix)]
#[test]
fn a_symlinked_directory_is_skipped_and_a_symlinked_file_is_collected() {
    use std::os::unix::fs::symlink;

    let dir = mindtape::test_support::tempdir();
    let root = dir.path();
    write_typ(root, "tasks/real.typ");
    let outside = mindtape::test_support::tempdir();
    write_typ(outside.path(), "shared.typ");
    symlink(outside.path(), root.join("tasks/linked-dir")).unwrap();
    symlink(
        outside.path().join("shared.typ"),
        root.join("tasks/linked.typ"),
    )
    .unwrap();

    let files = collect(root, &config(&["tasks"], &[]), false);
    assert_eq!(shown(&files, root), ["tasks/linked.typ", "tasks/real.typ"]);
}
