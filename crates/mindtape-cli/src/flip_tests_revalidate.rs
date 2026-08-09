//! The post-flip revalidation cluster: after a batch, `run_with` reloads the
//! project and reports violations or load errors that touch a flipped file.
//! These tests pin that reload-and-report path — the unreachable-status catch,
//! the load-error report on the touched file, and the tolerance for pre-existing
//! breakage on untouched files and tasks — split out of `flip_tests.rs` along
//! the same seam the `flip_tests_output.rs` sibling draws for the stderr half
//! of these reports. Shared fixtures come from `super::tests`.
#![allow(clippy::unwrap_used)]

use super::run_with;
use super::tests::{
    ALPHA, BETA, BROKEN_RELOAD_CONFIG, LEGEND_CONFIG, REACHABLE_CONFIG, STATELESS, flip_args, read,
    setup,
};

/// A file that fails to evaluate (an unclosed content block), used as a
/// pre-existing broken task that no flip touches.
const BROKEN_FILE: &str = "#import \"@local/mindtape:0.2.0\": *\n\
#show: task.with(\n  title: \"Broken\",\n  status: wip(2026, 6, 1),\n)\n\n== Summary\n\nBody [\n";

/// The post-flip revalidation reloads the project and reports violations on the
/// touched task. The returned error only carries the count; the violation text
/// itself goes to stderr, so `flip_tests_output.rs` asserts that half.
#[test]
fn revalidate_catches_unreachable_status() {
    let dir = tempfile::tempdir().unwrap();
    let project = setup(dir.path(), &[("s.typ", STATELESS)], REACHABLE_CONFIG);
    let err = run_with(&project, &flip_args("blocked", &["s"]))
        .unwrap_err()
        .to_string();
    assert!(
        err.contains("1 task(s) had problems"),
        "expected the revalidation failure count: {err}"
    );
}

/// A flip whose rewrite breaks the file (the status value renders into a Typst
/// syntax error) surfaces through the post-flip reload: `revalidate` appends
/// the load error against the touched file to the problem list. The returned
/// error only carries the count; the message itself goes to stderr (see
/// `flip_tests_output.rs`).
#[test]
fn revalidate_reports_load_error_on_touched_file() {
    let dir = tempfile::tempdir().unwrap();
    let project = setup(dir.path(), &[("alpha.typ", ALPHA)], BROKEN_RELOAD_CONFIG);
    let err = run_with(&project, &flip_args("broken\"done", &["alpha"]))
        .unwrap_err()
        .to_string();
    assert!(
        err.contains("1 task(s) had problems"),
        "expected the load-failure count: {err}"
    );
}

/// A pre-existing broken file elsewhere in the project is not the flip's
/// doing, so `revalidate` must not report its load error: only errors on the
/// touched file are appended to the problem list, and the batch stays Ok.
#[test]
fn revalidate_ignores_load_errors_on_untouched_files() {
    let dir = tempfile::tempdir().unwrap();
    let project = setup(
        dir.path(),
        &[("alpha.typ", ALPHA), ("broken.typ", BROKEN_FILE)],
        LEGEND_CONFIG,
    );
    run_with(&project, &flip_args("done", &["alpha"])).unwrap();
    assert!(read(dir.path(), "alpha.typ").contains("status: done("));
}

/// A task whose authored status is unreachable under the declared graph is a
/// pre-existing condition: when that task is not part of the batch, its
/// revalidation violation must not fail the flip of an unrelated task.
#[test]
fn revalidate_ignores_violations_on_untouched_tasks() {
    let dir = tempfile::tempdir().unwrap();
    // BETA's authored `blocked` is unreachable under REACHABLE_CONFIG; only
    // alpha is flipped, so the violation must stay out of the problem list.
    let project = setup(
        dir.path(),
        &[("alpha.typ", ALPHA), ("beta.typ", BETA)],
        REACHABLE_CONFIG,
    );
    run_with(&project, &flip_args("done", &["alpha"])).unwrap();
    assert!(read(dir.path(), "alpha.typ").contains("status: done("));
}
