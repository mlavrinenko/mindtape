#![allow(clippy::unwrap_used)]

use crate::test_support::tempdir;
use std::collections::BTreeMap;
use std::path::Path;
use std::process::Command;

use crate::config::{Config, parse_config};
use crate::filter::{Filter, FilterError, expr_problem};
use crate::model::Task;
use crate::test_support::{bare_task, git_init};

/// A config whose git root is `root`.
fn config_at(root: &Path) -> Config {
    let mut config = parse_config("").unwrap();
    config.git.set_root(root);
    config
}

/// A task declaring `files` in its estimate.
fn task(files: f64) -> Task {
    Task {
        estimate: BTreeMap::from([("files".to_owned(), files)]),
        ..bare_task("t")
    }
}

fn eval(expr: &str, config: &Config) -> Result<bool, FilterError> {
    Filter::parse(expr).unwrap().matches(&task(2.0), config)
}

/// A repository with `src/lib.rs` staged and `src/lib_tests.rs` and
/// `README.md` loose in the working tree.
fn repo() -> tempfile::TempDir {
    let dir = tempdir();
    let root = dir.path();
    git_init(root);
    std::fs::create_dir_all(root.join("src")).unwrap();
    for file in ["src/lib.rs", "src/lib_tests.rs", "README.md"] {
        std::fs::write(root.join(file), "x\n").unwrap();
    }
    let ok = Command::new("git")
        .args(["add", "src/lib.rs"])
        .current_dir(root)
        .status()
        .unwrap()
        .success();
    assert!(ok);
    dir
}

#[test]
fn changed_counts_the_paths_in_each_named_state() {
    let dir = repo();
    let config = config_at(dir.path());
    assert!(eval(r#"changed("staged") == 1"#, &config).unwrap());
    assert!(eval(r#"changed("worktree") == 2"#, &config).unwrap());
    assert!(eval(r#"changed("staged+worktree") == 3"#, &config).unwrap());
}

#[test]
fn changed_counts_only_what_its_globs_admit() {
    let dir = repo();
    let config = config_at(dir.path());
    let expr = r#"changed("staged+worktree", "src/**", "!**/*_tests.rs") == 1"#;
    assert!(eval(expr, &config).unwrap());
    assert!(eval(r#"changed("worktree", "!README.md") == 1"#, &config).unwrap());
}

#[test]
fn changed_compares_against_a_field_on_either_side() {
    let dir = repo();
    let config = config_at(dir.path());
    assert!(eval(r#"changed("staged+worktree") > estimate.files"#, &config).unwrap());
    assert!(eval(r#"estimate.files >= changed("worktree")"#, &config).unwrap());
    assert!(!eval(r#"changed("worktree") < estimate.files"#, &config).unwrap());
}

#[test]
fn a_field_compared_to_an_absent_field_follows_the_missing_field_rule() {
    let config = parse_config("").unwrap();
    assert!(!eval("estimate.files == estimate.lines", &config).unwrap());
    assert!(eval("estimate.files != estimate.lines", &config).unwrap());
    assert!(eval("estimate.files == estimate.files", &config).unwrap());
}

#[test]
fn outside_a_repository_changed_fails_naming_the_reason() {
    let dir = tempdir();
    let err = eval(r#"changed("worktree") == 0"#, &config_at(dir.path())).unwrap_err();
    assert!(matches!(err, FilterError::Git { .. }), "{err:?}");
    assert!(err.to_string().contains("not a git repository"), "{err}");
}

#[test]
fn changed_is_a_count_not_a_predicate() {
    let dir = repo();
    let err = eval(r#"changed("worktree")"#, &config_at(dir.path())).unwrap_err();
    assert!(err.to_string().contains("not a condition"), "{err}");
}

#[test]
fn the_load_time_dry_run_checks_arguments_without_reading_git() {
    let dir = tempdir();
    let config = config_at(dir.path());
    let probe = Task::default();
    assert_eq!(
        expr_problem(
            r#"changed("staged", "src/**") <= estimate.files"#,
            &probe,
            &config
        ),
        None,
        "a well-formed call must pass the dry run outside any repository"
    );
    let state = expr_problem(r#"changed("committed") == 0"#, &probe, &config).unwrap();
    assert!(state.contains("committed"), "{state}");
    let glob = expr_problem(r#"changed("staged", "[oops") == 0"#, &probe, &config).unwrap();
    assert!(glob.contains("[oops"), "{glob}");
    let bare = expr_problem("changed() == 0", &probe, &config).unwrap();
    assert!(bare.contains("changed()"), "{bare}");
}
