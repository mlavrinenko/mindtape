#![allow(clippy::unwrap_used)]

use crate::test_support::tempdir;
use std::path::Path;

use crate::config::{Config, parse_config};
use crate::filter::{Filter, FilterError, expr_problem};
use crate::graph::ready::Readiness;
use crate::model::Task;
use crate::test_support::{bare_task, git_as_tester as git, git_init};

fn write(root: &Path, path: &str, text: &str) {
    let path = root.join(path);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, text).unwrap();
}

/// Commit everything in the tree with `message`.
fn commit(root: &Path, message: &str) {
    git(root, &["add", "-A"]);
    git(root, &["commit", "-q", "-m", message]);
}

/// A history exercising every way a trailer value resolves:
///
/// - `src/a.rs` is changed under a path-form trailer naming `old`, which a
///   later commit renames to `new`, and under a stem-form trailer naming
///   `kept`;
/// - `src/b.rs` is changed under a trailer naming `gone`, a task that was
///   deleted.
fn repo() -> tempfile::TempDir {
    let dir = tempdir();
    let root = dir.path();
    git_init(root);
    write(root, "tasks/old.typ", "old task\n");
    write(root, "tasks/gone.typ", "gone task\n");
    write(root, "src/a.rs", "one\n");
    commit(root, "feat: a\n\nRefs: tasks/old.typ");
    write(root, "src/a.rs", "two\n");
    commit(root, "fix: a\n\nRefs: kept");
    git(root, &["mv", "tasks/old.typ", "tasks/new.typ"]);
    commit(root, "task: rename old to new");
    write(root, "src/b.rs", "b\n");
    std::fs::remove_file(root.join("tasks/gone.typ")).unwrap();
    commit(root, "feat: b\n\nRefs: gone");
    dir
}

fn config_at(root: &Path) -> Config {
    let mut config = parse_config("").unwrap();
    config.git.set_root(root);
    config
}

fn tasks() -> Vec<Task> {
    ["new", "kept", "other"].map(bare_task).to_vec()
}

/// The stems of `tasks` that `expr` matches, over the loaded set.
fn matching(expr: &str, config: &Config) -> Result<Vec<String>, FilterError> {
    let set = tasks();
    let readiness = Readiness::compute(&set, config);
    let filter = Filter::parse(expr).unwrap();
    let mut out = Vec::new();
    for task in &set {
        if filter.matches_with(task, config, &readiness)? {
            out.push(task.id.clone());
        }
    }
    Ok(out)
}

#[test]
fn touches_matches_the_tasks_every_commit_changing_the_path_names() {
    let dir = repo();
    let config = config_at(dir.path());
    assert_eq!(
        matching(r#"touches("src/a.rs")"#, &config).unwrap(),
        ["new", "kept"],
        "a path-form trailer resolves by its stem, and `old` through its rename to `new`"
    );
    assert!(
        matching(r#"touches("src/none.rs")"#, &config)
            .unwrap()
            .is_empty()
    );
}

#[test]
fn a_retired_stem_matches_nothing_and_is_reported() {
    let dir = repo();
    let config = config_at(dir.path());
    assert!(
        matching(r#"touches("src/b.rs")"#, &config)
            .unwrap()
            .is_empty()
    );
    assert_eq!(config.git.retired_refs(), ["gone"]);
}

#[test]
fn outside_a_repository_touches_fails_naming_the_reason() {
    let dir = tempdir();
    let err = matching(r#"touches("src/a.rs")"#, &config_at(dir.path())).unwrap_err();
    assert!(matches!(err, FilterError::Git { .. }), "{err:?}");
    let text = err.to_string();
    assert!(text.contains("touches()"), "{text}");
    assert!(text.contains("not a git repository"), "{text}");
}

#[test]
fn the_dry_run_checks_touches_arguments_without_reading_git() {
    let dir = tempdir();
    let config = config_at(dir.path());
    let probe = Task::default();
    assert_eq!(
        expr_problem(r#"touches("src/a.rs")"#, &probe, &config),
        None
    );
    let bare = expr_problem("touches()", &probe, &config).unwrap();
    assert!(bare.contains("touches()"), "{bare}");
}

#[test]
fn the_index_is_cached_by_head() {
    let dir = repo();
    let root = dir.path();
    matching(r#"touches("src/a.rs")"#, &config_at(root)).unwrap();
    let cache = root.join(".mindtape/cache/commit-refs.json");
    let text = std::fs::read_to_string(&cache).unwrap();
    // Point a path at `other` in the cached index only: a hit reads it.
    let forged = text.replace("\"src/b.rs\"", "\"src/forged.rs\":[\"other\"],\"src/b.rs\"");
    assert_ne!(forged, text, "the cache must index src/b.rs");
    std::fs::write(&cache, forged).unwrap();
    assert_eq!(
        matching(r#"touches("src/forged.rs")"#, &config_at(root)).unwrap(),
        ["other"],
        "an unchanged HEAD must read the cached index"
    );
    write(root, "src/c.rs", "c\n");
    commit(root, "feat: c\n\nRefs: other");
    assert!(
        matching(r#"touches("src/forged.rs")"#, &config_at(root))
            .unwrap()
            .is_empty(),
        "a moved HEAD must rebuild the index"
    );
    assert_eq!(
        matching(r#"touches("src/c.rs")"#, &config_at(root)).unwrap(),
        ["other"]
    );
}
