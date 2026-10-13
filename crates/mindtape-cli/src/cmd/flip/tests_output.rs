//! `flip` tests that need their own process: the command's stdout/stderr
//! verbs, the shadowed-verb warning, and `run`'s cwd-based project discovery.
//!
//! `std::io::set_output_capture` is nightly-only, so asserting on what the
//! command prints means reading a real child's streams; `run` additionally
//! needs `set_current_dir`, which is process-global. Each test below re-execs
//! the test binary for its own name, with `_MT_FLIP_CHILD` marking the child
//! branch. Fixtures come from `cmd/flip/tests.rs`.
#![allow(clippy::unwrap_used)]

use super::tests::{
    ALPHA, BROKEN_RELOAD_CONFIG, GAMMA, LEGEND_CONFIG, REACHABLE_CONFIG, STATELESS, flip_args,
    read, setup,
};
use super::{run, run_with};

const FLIP_CHILD_ENV: &str = "_MT_FLIP_CHILD";

/// `mt flip --list` in `project`, for the child half of a `--list` test.
fn list_verbs(project: &crate::util::Project) {
    let mut args = flip_args("done", &[]);
    args.status = None;
    args.list = true;
    run_with(project, &args).unwrap();
}

/// `run` is the thin wrapper that discovers the project from the cwd and hands
/// off to `run_with`. Asserting only `is_err()` on an unknown status would pass
/// for any failure at all, so this drives a successful flip and checks the file
/// on disk actually changed — which only holds if the delegation happened.
///
/// `set_current_dir` is process-global, hence the child: mutating the cwd of
/// the shared test process would race every sibling test that resolves paths.
#[test]
fn run_delegates_to_run_with() {
    if mindtape::test_support::rerun_isolated(FLIP_CHILD_ENV).is_some() {
        return;
    }
    let (dir, _) = setup(&[("alpha.typ", ALPHA)], LEGEND_CONFIG);
    std::env::set_current_dir(dir.path()).unwrap();
    run(&flip_args("done", &["alpha"])).unwrap();
    assert!(
        read(dir.path(), "alpha.typ").contains("status: done("),
        "run() must reach run_with and rewrite the file"
    );
}

/// A legend value colliding with a built-in subcommand name (`add`) can never
/// dispatch as a shortcut verb, because the built-in wins. The flip still
/// succeeds; the warning is the whole observable effect, and it goes to stderr.
#[test]
fn shadowed_verb_warns_on_stderr() {
    if let Some(err) = mindtape::test_support::rerun_isolated_stderr(FLIP_CHILD_ENV) {
        assert!(
            err.contains("shadows") && err.contains("add"),
            "expected a shadowed-verb warning naming `add`: {err}"
        );
        return;
    }
    let shadow_cfg =
        "[[tasks.status]]\nvalue = \"add\"\n[[tasks.status]]\nvalue = \"done\"\nterminal = true\n";
    let (_dir, project) = setup(&[("a.typ", ALPHA)], shadow_cfg);
    run_with(&project, &flip_args("done", &["a"])).unwrap();
}

#[test]
fn restamp_prints_re_stamped() {
    if let Some(out) = mindtape::test_support::rerun_isolated_stdout(FLIP_CHILD_ENV) {
        assert!(
            out.contains("re-stamped"),
            "expected 're-stamped' for same-status flip: {out}"
        );
        return;
    }
    let (_dir, project) = setup(&[("a.typ", ALPHA)], LEGEND_CONFIG);
    run_with(&project, &flip_args("wip", &["a"])).unwrap();
}

#[test]
fn flip_prints_flipped_not_re_stamped() {
    if let Some(out) = mindtape::test_support::rerun_isolated_stdout(FLIP_CHILD_ENV) {
        assert!(
            out.contains("flipped"),
            "expected 'flipped' for diff-status flip: {out}"
        );
        assert!(
            !out.contains("re-stamped"),
            "unexpected 're-stamped' for diff-status flip: {out}"
        );
        return;
    }
    let (_dir, project) = setup(&[("a.typ", ALPHA)], LEGEND_CONFIG);
    run_with(&project, &flip_args("done", &["a"])).unwrap();
}

#[test]
fn terminal_flip_prints_recur_notice() {
    if let Some(out) = mindtape::test_support::rerun_isolated_stdout(FLIP_CHILD_ENV) {
        assert!(
            out.contains("recurring"),
            "expected recurring notice: {out}"
        );
        return;
    }
    let (_dir, project) = setup(&[("g.typ", GAMMA)], LEGEND_CONFIG);
    run_with(&project, &flip_args("done", &["g"])).unwrap();
}

/// A legend whose entries carry glyphs renders them in the `--list` output —
/// the `Some(glyph)` arm of `list_verbs`' entry loop, which the floor-free
/// `LEGEND_CONFIG` never reaches.
#[test]
fn list_prints_glyph_verbs() {
    if let Some(out) = mindtape::test_support::rerun_isolated_stdout(FLIP_CHILD_ENV) {
        assert!(
            out.contains("🔨 wip") && out.contains("✅ done"),
            "expected glyph-prefixed legend verbs in --list output: {out}"
        );
        return;
    }
    let glyph_cfg = "[[tasks.status]]\nvalue = \"wip\"\nglyph = \"🔨\"\n\
        [[tasks.status]]\nvalue = \"done\"\nglyph = \"✅\"\nterminal = true\n";
    let (_dir, project) = setup(&[], glyph_cfg);
    list_verbs(&project);
}

/// An explicitly emptied legend (`status = []`, which unconstrains the floored
/// section) hits `list_verbs`' no-legend branch: the hint, not a verb listing.
#[test]
fn list_empty_legend_prints_hint() {
    if let Some(out) = mindtape::test_support::rerun_isolated_stdout(FLIP_CHILD_ENV) {
        assert!(
            out.contains("no status legend configured")
                && out.contains("shortcut verbs are unavailable"),
            "expected the empty-legend hint in --list output: {out}"
        );
        return;
    }
    let empty_cfg = "[tasks]\nstatus = []\n";
    let (_dir, project) = setup(&[], empty_cfg);
    list_verbs(&project);
}

#[test]
fn list_prints_verbs() {
    if let Some(out) = mindtape::test_support::rerun_isolated_stdout(FLIP_CHILD_ENV) {
        assert!(
            out.contains("wip"),
            "expected legend verbs in --list output: {out}"
        );
        return;
    }
    let (_dir, project) = setup(&[], LEGEND_CONFIG);
    list_verbs(&project);
}

/// The revalidation violation itself is printed, not returned: `run_with`'s
/// error carries only the problem count, so the message naming the field and
/// the reason is only observable on stderr.
#[test]
fn revalidate_prints_the_unreachable_violation() {
    if let Some(err) = mindtape::test_support::rerun_isolated_stderr(FLIP_CHILD_ENV) {
        assert!(
            err.contains("s.typ:") && err.contains(": status: `blocked` is unreachable"),
            "expected the reachability violation prefixed as an error: {err}"
        );
        return;
    }
    let (_dir, project) = setup(&[("s.typ", STATELESS)], REACHABLE_CONFIG);
    let err = run_with(&project, &flip_args("blocked", &["s"])).unwrap_err();
    crate::diag::emit(&err);
}

/// A rewrite that would not evaluate is refused *before* the write, so the
/// report says the source did not load rather than that the file did — the
/// difference being that there is no file. The message is only observable on a
/// child's stderr. With one target, `run_with` returns that target's refusal
/// rather than a summary, and the child prints it as `main` would.
#[test]
fn refuses_a_rewrite_that_would_not_load() {
    if let Some(err) = mindtape::test_support::rerun_isolated_stderr(FLIP_CHILD_ENV) {
        assert!(
            err.contains("error: tasks/alpha.typ:3: did not load:"),
            "expected the pre-write refusal prefixed as an error: {err}"
        );
        assert!(
            !err.contains("rejected"),
            "the refusal is the problem, with no summary of its own: {err}"
        );
        return;
    }
    let (dir, project) = setup(&[("alpha.typ", ALPHA)], BROKEN_RELOAD_CONFIG);
    let err = run_with(&project, &flip_args("broken\"done", &["alpha"])).unwrap_err();
    crate::diag::emit(&err);
    assert_eq!(
        std::fs::read_to_string(dir.path().join("tasks/alpha.typ")).unwrap(),
        ALPHA,
        "a refused flip must leave the file byte-identical"
    );
}
