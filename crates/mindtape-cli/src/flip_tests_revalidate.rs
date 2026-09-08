//! What a flip refuses, and what it tolerates.
//!
//! Two halves that used to be one. A rewrite that would leave the flipped task
//! invalid is now caught as a *candidate*, before the write — so these tests
//! assert the file is byte-identical afterwards, which the old
//! write-then-reload order could not offer. What still runs after the batch is
//! the cross-task net: a problem a landed flip caused on some OTHER task, which
//! no per-candidate check can see. The tolerance tests pin that net's scope —
//! pre-existing breakage on untouched files and tasks is not the batch's doing.
//!
//! Split out of `flip_tests.rs` along the same seam the `flip_tests_output.rs`
//! sibling draws for the stderr half of these reports. Shared fixtures come
//! from `super::tests`.
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

/// A flip into a status the declared graph cannot reach is refused before the
/// write, and the file is left exactly as it was. The returned error only
/// carries the count; the violation text goes to stderr, so
/// `flip_tests_output.rs` asserts that half.
#[test]
fn refuses_a_flip_into_an_unreachable_status() {
    let dir = tempfile::tempdir().unwrap();
    let project = setup(dir.path(), &[("s.typ", STATELESS)], REACHABLE_CONFIG);
    let err = run_with(&project, &flip_args("blocked", &["s"]))
        .unwrap_err()
        .to_string();
    assert!(
        err.contains("1 task(s) had problems"),
        "expected the refusal count: {err}"
    );
    assert_eq!(
        read(dir.path(), "s.typ"),
        STATELESS,
        "a refused flip must leave the file byte-identical"
    );
}

/// A flip whose rewrite breaks the file (the status value renders into a Typst
/// syntax error) is caught as a candidate, so the broken source never reaches
/// disk. The returned error only carries the count; the message itself goes to
/// stderr (see `flip_tests_output.rs`).
#[test]
fn refuses_a_rewrite_that_would_not_load() {
    let dir = tempfile::tempdir().unwrap();
    let project = setup(dir.path(), &[("alpha.typ", ALPHA)], BROKEN_RELOAD_CONFIG);
    let err = run_with(&project, &flip_args("broken\"done", &["alpha"]))
        .unwrap_err()
        .to_string();
    assert!(
        err.contains("1 task(s) had problems"),
        "expected the refusal count: {err}"
    );
    assert_eq!(
        read(dir.path(), "alpha.typ"),
        ALPHA,
        "a refused flip must leave the file byte-identical"
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
