//! `TaskSource` and `TaskPath`: normalized path forms, filesystem kinds, and
//! which paths count as task sources.
#![allow(clippy::indexing_slicing, clippy::unwrap_used)]

use std::path::{Path, PathBuf};

use crate::parse_config;
use crate::task_source::{TaskPathKind, TaskSource};
use crate::test_support::{tempdir, write_file};

/// A project with `config` and a policy built against it. Keep the returned
/// tempdir alive: the policy's kinds are facts about the files under it.
fn project(config: &str) -> (tempfile::TempDir, TaskSource) {
    let dir = tempdir();
    std::fs::create_dir_all(dir.path().join(".mindtape")).unwrap();
    let config = parse_config(config).unwrap();
    let source = TaskSource::new(dir.path(), &config);
    (dir, source)
}

/// Whether `rel` is a source under `config`, with the file written.
fn is_source(config: &str, rel: &str) -> bool {
    let (dir, source) = project(config);
    write_file(dir.path(), rel, "");
    source.is_source(&source.task_path(Path::new(rel)))
}

/// The warning `rel` earns under `config`, with `body` written there.
fn warning(config: &str, rel: &str, body: &str) -> Option<crate::loader::LoadWarning> {
    let (dir, source) = project(config);
    write_file(dir.path(), rel, body);
    let path = source.task_path(Path::new(rel));
    let text = std::fs::read_to_string(path.abs()).unwrap();
    source.reserved_warning(&path, &text)
}

#[test]
fn a_relative_path_carries_normalized_project_relative_and_absolute_forms() {
    let (dir, source) = project("[tasks]\nroots = [\".\"]\n");
    let path = source.task_path(Path::new("./tasks/../tasks/build.typ"));

    assert_eq!(path.rel(), Path::new("tasks/build.typ"));
    assert_eq!(path.abs(), dir.path().join("tasks/build.typ"));
    assert_eq!(path.kind(), TaskPathKind::Missing);
}

#[test]
fn an_absolute_path_under_the_project_is_still_project_relative() {
    let (dir, source) = project("[tasks]\nroots = [\".\"]\n");
    let absolute = dir.path().join("tasks").join(".").join("build.typ");
    let path = source.task_path(&absolute);

    assert_eq!(path.rel(), Path::new("tasks/build.typ"));
    assert_eq!(path.abs(), dir.path().join("tasks/build.typ"));
}

#[test]
fn an_absolute_path_outside_the_project_keeps_a_dot_dot_form() {
    let (dir, source) = project("[tasks]\nroots = [\".\"]\n");
    let outside = dir.path().parent().unwrap().join("shared/tasks/build.typ");
    let path = source.task_path(&outside);

    assert_eq!(path.rel(), Path::new("../shared/tasks/build.typ"));
    assert_eq!(path.abs(), outside);
}

#[test]
fn a_regular_file_a_symlink_and_a_missing_path_have_distinct_kinds() {
    let (dir, source) = project("[tasks]\nroots = [\"tasks\"]\n");
    write_file(dir.path(), "tasks/build.typ", "");

    assert_eq!(
        source.task_path(Path::new("tasks/build.typ")).kind(),
        TaskPathKind::Regular
    );
    assert_eq!(
        source.task_path(Path::new("tasks/absent.typ")).kind(),
        TaskPathKind::Missing
    );

    #[cfg(unix)]
    {
        std::os::unix::fs::symlink("build.typ", dir.path().join("tasks/link.typ")).unwrap();
        assert_eq!(
            source.task_path(Path::new("tasks/link.typ")).kind(),
            TaskPathKind::Symlink
        );
    }
}

#[test]
fn a_regular_typ_file_under_a_root_is_a_source() {
    assert!(is_source(
        "[tasks]\nroots = [\"tasks\"]\n",
        "tasks/build.typ"
    ));
}

#[test]
fn a_non_typ_file_and_a_file_outside_every_root_are_not_sources() {
    for rel in ["tasks/notes.md", "elsewhere/build.typ"] {
        let (dir, source) = project("[tasks]\nroots = [\"tasks\"]\n");
        write_file(dir.path(), rel, "");
        let path = source.task_path(Path::new(rel));
        assert!(!source.is_source(&path), "{rel}");
        assert!(!source.is_reserved(&path), "{rel}");
    }
}

#[test]
fn a_missing_path_is_not_a_source() {
    let (_dir, source) = project("[tasks]\nroots = [\"tasks\"]\n");

    let path = source.task_path(Path::new("tasks/absent.typ"));
    assert!(!source.is_source(&path));
}

#[cfg(unix)]
#[test]
fn a_symlinked_typ_file_is_not_a_source() {
    let (dir, source) = project("[tasks]\nroots = [\"tasks\"]\n");
    write_file(dir.path(), "tasks/real.typ", "");
    std::os::unix::fs::symlink("real.typ", dir.path().join("tasks/link.typ")).unwrap();

    let path = source.task_path(Path::new("tasks/link.typ"));
    assert_eq!(path.kind(), TaskPathKind::Symlink);
    assert!(!source.is_source(&path));
}

#[test]
fn a_reserved_metadata_component_excludes_a_source_case_insensitively() {
    let (dir, source) = project("[tasks]\nroots = [\".\"]\n");
    for reserved in [".mindtape/hidden.typ", ".GIT/hooks/hidden.typ"] {
        write_file(dir.path(), reserved, "");
        let path = source.task_path(Path::new(reserved));
        assert!(!source.is_source(&path), "{reserved}");
        assert!(source.is_reserved(&path), "{reserved}");
        assert!(source.is_misfiled(&path), "{reserved}");
    }
}

#[test]
fn a_path_below_a_configured_template_dir_is_reserved() {
    let (dir, source) = project("[tasks]\nroots = [\".\"]\n[templates]\ndirs = [\"templates\"]\n");
    write_file(dir.path(), "templates/bug.typ", "");
    write_file(dir.path(), "tasks/build.typ", "");

    let template = source.task_path(Path::new("templates/bug.typ"));
    assert!(!source.is_source(&template));
    assert!(source.is_reserved(&template));
    assert!(!source.is_misfiled(&template));

    let task = source.task_path(Path::new("tasks/build.typ"));
    assert!(source.is_source(&task));
}

/// Containment is by path component, not by string prefix.
#[test]
fn a_sibling_with_a_longer_name_is_not_inside_the_dir() {
    let (dir, source) = project("[tasks]\nroots = [\".\"]\n[templates]\ndirs = [\"templates\"]\n");
    write_file(dir.path(), "tasks-extra/build.typ", "");
    write_file(dir.path(), "templates-extra/bug.typ", "");

    assert!(source.is_source(&source.task_path(Path::new("tasks-extra/build.typ"))));
    let sibling = source.task_path(Path::new("templates-extra/bug.typ"));
    assert!(source.is_source(&sibling));
    assert!(!source.is_reserved(&sibling));
}

#[test]
fn an_excluded_path_is_not_a_source() {
    let config = "[[tasks.roots]]\npath = \".\"\nexclude = [\"docs/**\"]\n";
    assert!(!is_source(config, "docs/example.typ"));
    assert!(is_source(config, "tasks/build.typ"));
}

/// An exclude narrows one root. A narrower root still admits the path, and
/// the file is a source through it.
#[test]
fn a_second_root_still_admits_an_excluded_path() {
    let config = "[[tasks.roots]]\npath = \".\"\nexclude = [\"docs/**\"]\n\n\
                  [[tasks.roots]]\npath = \"docs/tasks\"\n";
    assert!(is_source(config, "docs/tasks/build.typ"));
}

#[test]
fn a_root_the_policy_does_not_hold_admits_nothing() {
    let (dir, source) = project("[tasks]\nroots = [\"tasks\"]\n");
    let path = source.task_path(Path::new("tasks/build.typ"));

    assert!(!source.admits_in(&dir.path().join("elsewhere"), &path));
}

#[test]
fn an_exclude_glob_is_relative_to_its_root() {
    let config = "[[tasks.roots]]\npath = \"tasks\"\nexclude = [\"archive/**\"]\n";
    assert!(!is_source(config, "tasks/archive/old.typ"));
    assert!(is_source(config, "tasks/current.typ"));
}

#[test]
fn a_configured_root_below_the_project_is_the_only_source_scope() {
    let (dir, source) = project("[tasks]\nroots = [\"backlog/nested\"]\n");
    write_file(dir.path(), "backlog/nested/build.typ", "");
    write_file(dir.path(), "backlog/outside.typ", "");

    assert!(source.is_source(&source.task_path(Path::new("backlog/nested/build.typ"))));
    assert!(!source.is_source(&source.task_path(Path::new("backlog/outside.typ"))));
}

#[test]
fn a_misfiled_typ_with_a_task_call_warns_at_the_call_line() {
    let warning = warning(
        "[tasks]\nroots = [\".\"]\n",
        ".mindtape/task.typ",
        "#import \"@local/mindtape:0.4.0\": *\n\n#show: task.with(title: \"Misfiled\")\n",
    )
    .unwrap();
    assert_eq!(warning.file, Path::new(".mindtape/task.typ"));
    assert_eq!(warning.line, Some(3));
    assert!(warning.message.contains("task.with"), "{}", warning.message);
}

#[test]
fn a_misfiled_typ_with_no_task_call_does_not_warn() {
    assert!(
        warning(
            "[tasks]\nroots = [\".\"]\n",
            ".mindtape/notes.typ",
            "= Notes\n"
        )
        .is_none()
    );
}

/// A template counts as reserved, but its `task.with` call is what the
/// directory is for, so it earns no warning.
#[test]
fn a_template_with_a_task_call_does_not_warn() {
    assert!(
        warning(
            "[tasks]\nroots = [\".\"]\n",
            ".mindtape/templates/task.typ",
            "#show: task.with(title: \"Template\")\n",
        )
        .is_none()
    );
}

#[test]
fn a_regular_task_source_does_not_warn() {
    let (dir, source) = project("[tasks]\nroots = [\"tasks\"]\n");
    write_file(
        dir.path(),
        "tasks/build.typ",
        "#show: task.with(title: \"T\")\n",
    );
    let path = source.task_path(Path::new("tasks/build.typ"));
    let text = std::fs::read_to_string(path.abs()).unwrap();

    assert!(source.reserved_warning(&path, &text).is_none());
}

#[test]
fn the_policy_keeps_normalized_root_and_template_forms() {
    let dir = tempdir();
    let config = parse_config(
        "[[tasks.roots]]\npath = \"./tasks/../tasks/\"\n\
         [templates]\ndirs = [\"./tmpl/../tmpl\"]\n",
    )
    .unwrap();
    let source = TaskSource::new(dir.path(), &config);

    assert_eq!(source.root(), dir.path());
    assert_eq!(source.roots(), [dir.path().join("tasks")].as_slice());
    assert_eq!(source.templates(), [dir.path().join("tmpl")].as_slice());
    assert_eq!(config.tasks.roots[0].dir(), "tasks");
    assert_eq!(
        config.templates.normalized_dirs(),
        vec![PathBuf::from("tmpl")]
    );
}
