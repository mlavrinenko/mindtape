#![allow(clippy::unwrap_used)]
use std::path::{Path, PathBuf};

use mindtape::test_support::{TASK_SRC, bare_task};
use mindtape::{Config, LoadError, LoadOutcome, Task, load_project};

use super::{
    Project, hyperlinks_flag, init_hyperlinks_flag, init_worktree, pick_override,
    resolve_hyperlinks, resolve_root_with, resolve_task, warn_errors, worktree_override,
};
use crate::handle::{DEFAULT_CHECK_LEN, check_chars};
use crate::render::link::HyperlinkChoice;

#[test]
fn warn_errors_is_silent_when_no_errors() {
    warn_errors(&LoadOutcome::default());
}

#[test]
fn warn_errors_does_not_panic_with_errors() {
    let outcome = LoadOutcome {
        tasks: vec![],
        errors: vec![LoadError {
            file: "broken.typ".into(),
            message: "eval failed".to_owned(),
        }],
    };
    warn_errors(&outcome);
}

// Minimal task for unit tests — no file on disk needed for pure logic tests.
fn stub_task(id: &str, title: &str) -> Task {
    let mut t = bare_task(id);
    t.title = title.to_owned();
    t
}

fn make_project_with_tasks(dir: &Path, task_ids: &[&str]) -> Project {
    std::fs::create_dir_all(dir.join(".mindtape")).unwrap();
    std::fs::create_dir_all(dir.join("tasks")).unwrap();
    for id in task_ids {
        std::fs::write(dir.join("tasks").join(format!("{id}.typ")), TASK_SRC).unwrap();
    }
    let config = Config::default();
    let outcome = load_project(dir, &config);
    Project {
        root: dir.to_path_buf(),
        config,
        outcome,
    }
}

fn stub_project(tasks: Vec<Task>) -> Project {
    let outcome = LoadOutcome {
        tasks,
        errors: vec![],
    };
    Project {
        root: ".".into(),
        config: Config::default(),
        outcome,
    }
}

#[test]
fn resolve_by_slug_succeeds() {
    let dir = tempfile::tempdir().unwrap();
    let project = make_project_with_tasks(dir.path(), &["alpha"]);
    let (task, _) = resolve_task(&project, "alpha").unwrap();
    assert_eq!(task.id, "alpha");
}

#[test]
fn resolve_handle_correct_check_succeeds() {
    let task_a = stub_task("alpha", "Alpha Task");
    let task_b = stub_task("beta", "Beta Task");
    let project = stub_project(vec![task_a.clone(), task_b.clone()]);

    // Build the correct check char for ordinal 2 (beta).
    let check = check_chars("beta", DEFAULT_CHECK_LEN);
    let handle = format!("2{check}");
    let (resolved, _) = resolve_task(&project, &handle).unwrap();
    assert_eq!(resolved.id, "beta");
}

#[test]
fn resolve_handle_wrong_check_errors_with_current_handle() {
    let task_a = stub_task("alpha", "Alpha Task");
    let project = stub_project(vec![task_a]);

    // Compute correct check then use a wrong one.
    let correct = check_chars("alpha", DEFAULT_CHECK_LEN);
    // Pick a letter that is definitely wrong.
    let wrong_char = if correct == "a" { 'b' } else { 'a' };
    let bad_handle = format!("1{wrong_char}");

    let err = resolve_task(&project, &bad_handle).unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("check mismatch"),
        "error should mention mismatch: {msg}"
    );
    assert!(
        msg.contains("Alpha Task"),
        "error should name the title: {msg}"
    );
}

/// A project that raised `[handle] check-len` gets handles of that length, and
/// the resolver reads the same key the display does.
fn stub_project_with_check_len(tasks: Vec<Task>, len: usize) -> Project {
    let mut config = Config::default();
    config.handle.check_len = Some(len);
    Project {
        root: ".".into(),
        config,
        outcome: LoadOutcome {
            tasks,
            errors: vec![],
        },
    }
}

#[test]
fn resolve_handle_honours_a_configured_check_len() {
    let project = stub_project_with_check_len(
        vec![
            stub_task("alpha", "Alpha Task"),
            stub_task("beta", "Beta Task"),
        ],
        2,
    );

    let handle = format!("2{}", check_chars("beta", 2));
    let (resolved, _) = resolve_task(&project, &handle).unwrap();
    assert_eq!(resolved.id, "beta");
}

/// The whole point of raising the length: a one-character handle is a handle
/// that belongs to a different task, not a short spelling of this one. Matching
/// it as a prefix would hand back the collision rate the project paid to remove.
#[test]
fn resolve_handle_refuses_a_short_handle_rather_than_prefix_matching() {
    let project = stub_project_with_check_len(vec![stub_task("alpha", "Alpha Task")], 2);

    let full = check_chars("alpha", 2);
    let short = format!("1{}", &full[..1]);

    let err = resolve_task(&project, &short).unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("check character"),
        "error should name the length: {msg}"
    );
    assert!(
        msg.contains(&format!("1{full}")),
        "error should show the full-length handle: {msg}"
    );
    assert!(
        !msg.contains("check mismatch"),
        "a short handle is a different failure from a wrong check: {msg}"
    );
}

/// Too LONG is refused by the same arm, so a handle copied out of a project
/// with a longer setting cannot half-resolve here.
#[test]
fn resolve_handle_refuses_an_over_long_handle() {
    let project = stub_project(vec![stub_task("alpha", "Alpha Task")]);

    let err = resolve_task(&project, &format!("1{}", check_chars("alpha", 3))).unwrap_err();
    assert!(
        err.to_string().contains("check character"),
        "error should name the length: {err}"
    );
}

#[test]
fn pick_override_flag_wins_over_env() {
    let picked = pick_override(Some("/flag/path".into()), Some("/env/path".into()));
    assert_eq!(picked, Some(std::path::PathBuf::from("/flag/path")));
}

#[test]
fn pick_override_falls_back_to_env() {
    let picked = pick_override(None, Some("/env/path".into()));
    assert_eq!(picked, Some(std::path::PathBuf::from("/env/path")));
}

#[test]
fn pick_override_empty_env_is_unset() {
    assert_eq!(pick_override(None, Some(String::new().into())), None);
    assert_eq!(pick_override(None, None), None);
}

#[test]
fn resolve_hyperlinks_flag_wins_over_config() {
    // The spec's crux: `never` beats a config opt-in.
    let choice = resolve_hyperlinks(
        Some(HyperlinkChoice::Never),
        mindtape::HyperlinkChoice::Always,
    );
    assert_eq!(choice, HyperlinkChoice::Never);
}

#[test]
fn resolve_hyperlinks_falls_back_to_config_when_flag_absent() {
    let choice = resolve_hyperlinks(None, mindtape::HyperlinkChoice::Always);
    assert_eq!(choice, HyperlinkChoice::Always);
}

#[test]
fn resolve_hyperlinks_defaults_to_never_when_both_absent() {
    let choice = resolve_hyperlinks(None, mindtape::HyperlinkChoice::Never);
    assert_eq!(choice, HyperlinkChoice::Never);
}

#[test]
fn resolve_root_override_targets_named_worktree() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join(".mindtape")).unwrap();
    // cwd is elsewhere; the absolute override still selects the worktree.
    let cwd = tempfile::tempdir().unwrap();
    let root = resolve_root_with(cwd.path(), Some(dir.path())).unwrap();
    assert_eq!(root, dir.path().canonicalize().unwrap());
}

#[test]
fn resolve_root_override_relative_resolves_against_cwd() {
    let cwd = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(cwd.path().join("wt").join(".mindtape")).unwrap();
    let root = resolve_root_with(cwd.path(), Some(Path::new("wt"))).unwrap();
    assert_eq!(root, cwd.path().join("wt").canonicalize().unwrap());
}

#[test]
fn resolve_root_override_without_marker_errors() {
    let dir = tempfile::tempdir().unwrap();
    let err = resolve_root_with(dir.path(), Some(dir.path())).unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("no .mindtape/ directory"),
        "error should name the missing marker: {msg}"
    );
}

#[test]
fn resolve_root_no_override_walks_up() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join(".mindtape")).unwrap();
    let child = dir.path().join("a").join("b");
    std::fs::create_dir_all(&child).unwrap();
    let root = resolve_root_with(&child, None).unwrap();
    assert_eq!(root, dir.path());
}

#[test]
fn resolve_handle_out_of_range_errors() {
    let task_a = stub_task("alpha", "Alpha Task");
    let project = stub_project(vec![task_a]);
    // Ordinal 2 with total 1: index = 1, which is exactly total.
    // With < total (real): 1 < 1 = false → error includes "(default view has 1 tasks)"
    // With <= total: 1 <= 1 = true → passes filter → .get(1) = None → error WITHOUT total
    let err = resolve_task(&project, "2a").unwrap_err();
    assert!(
        err.to_string().contains("default view has 1 tasks"),
        "error should include the total task count: {err}"
    );
}

// --- Subprocess tests (process-global `OnceLock` isolation) -----------
//
// `init_worktree` sets a process-global `WORKTREE` exactly once, and
// `warn_errors` writes to the process's stderr. Both are only observable
// from a fresh process, so each test below re-execs the test binary for a
// single test name (`test_util::run_self_test`); the parent checks the
// child's exit status and, where relevant, its captured stderr.

const CHILD: &str = "_MT_UTIL_CHILD";

#[test]
fn init_worktree_sets_override_once() {
    if std::env::var(CHILD).is_err() {
        let out = crate::test_util::run_self_test("init_worktree_sets_override_once", CHILD);
        crate::test_util::assert_child_ok(&out);
        return;
    }
    assert_eq!(worktree_override(), None, "must be None before init");
    init_worktree(Some(PathBuf::from("/test/path")));
    assert_eq!(
        worktree_override(),
        Some(PathBuf::from("/test/path")),
        "must be set after init"
    );
}

#[test]
fn init_hyperlinks_flag_sets_override_once() {
    if std::env::var(CHILD).is_err() {
        let out = crate::test_util::run_self_test("init_hyperlinks_flag_sets_override_once", CHILD);
        crate::test_util::assert_child_ok(&out);
        return;
    }
    assert_eq!(hyperlinks_flag(), None, "must be None before init");
    init_hyperlinks_flag(Some(HyperlinkChoice::Always));
    assert_eq!(
        hyperlinks_flag(),
        Some(HyperlinkChoice::Always),
        "must be set after init"
    );
}

#[test]
fn warn_errors_prints_to_stderr() {
    if std::env::var(CHILD).is_err() {
        let out = crate::test_util::run_self_test("warn_errors_prints_to_stderr", CHILD);
        crate::test_util::assert_child_ok(&out);
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert!(
            stderr.contains("warning: broken.typ: eval failed"),
            "missing warning line: {stderr}"
        );
        return;
    }
    let outcome = LoadOutcome {
        tasks: vec![],
        errors: vec![LoadError {
            file: "broken.typ".into(),
            message: "eval failed".to_owned(),
        }],
    };
    warn_errors(&outcome);
}

#[test]
fn resolve_path_with_slash_is_path_not_slug() {
    let project = stub_project(vec![stub_task("alpha", "Alpha Task")]);
    // "a/b" contains / but not .typ → path resolution (real) vs slug (mutant ||→&&)
    let err = resolve_task(&project, "a/b").unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("resolve"),
        "path with / must be resolved as path, not slug: {msg}"
    );
}

#[test]
fn resolve_absolute_path_eq_not_neq() {
    let dir = tempfile::tempdir().unwrap();
    let project = make_project_with_tasks(dir.path(), &["alpha"]);
    let abs = dir.path().join("tasks/alpha.typ");
    std::fs::write(&abs, mindtape::test_support::TASK_SRC).unwrap();
    let abs = abs.canonicalize().unwrap();
    let (task, path) = resolve_task(&project, abs.to_str().unwrap()).unwrap();
    assert_eq!(task.id, "alpha");
    assert_eq!(path, abs);
}
