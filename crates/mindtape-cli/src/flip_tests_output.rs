//! `flip` tests that need their own process: the command's stdout/stderr
//! verbs, the shadowed-verb warning, and `run`'s cwd-based project discovery.
//!
//! `std::io::set_output_capture` is nightly-only, so asserting on what the
//! command prints means reading a real child's streams; `run` additionally
//! needs `set_current_dir`, which is process-global. Each test below re-execs
//! the test binary for its own name, with `_MT_FLIP_CHILD` marking the child
//! branch. Fixtures come from `flip_tests.rs`.
#![allow(clippy::unwrap_used)]

use super::tests::{
    ALPHA, BROKEN_RELOAD_CONFIG, GAMMA, LEGEND_CONFIG, REACHABLE_CONFIG, STATELESS, flip_args,
    read, setup,
};
use super::{run, run_with};

const FLIP_CHILD_ENV: &str = "_MT_FLIP_CHILD";

fn child_output(name: &str) -> std::process::Output {
    std::process::Command::new(std::env::current_exe().unwrap())
        .env(FLIP_CHILD_ENV, "1")
        .args([name, "--nocapture"])
        .output()
        .unwrap()
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
    if std::env::var(FLIP_CHILD_ENV).is_err() {
        let o = child_output("run_delegates_to_run_with");
        assert!(
            o.status.success(),
            "child: {}",
            String::from_utf8_lossy(&o.stderr)
        );
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    setup(dir.path(), &[("alpha.typ", ALPHA)], LEGEND_CONFIG);
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
    if std::env::var(FLIP_CHILD_ENV).is_err() {
        let o = child_output("shadowed_verb_warns_on_stderr");
        assert!(
            o.status.success(),
            "child: {}",
            String::from_utf8_lossy(&o.stderr)
        );
        let err = String::from_utf8_lossy(&o.stderr);
        assert!(
            err.contains("shadows") && err.contains("add"),
            "expected a shadowed-verb warning naming `add`: {err}"
        );
        return;
    }
    let shadow_cfg =
        "[[tasks.status]]\nvalue = \"add\"\n[[tasks.status]]\nvalue = \"done\"\nterminal = true\n";
    let dir = tempfile::tempdir().unwrap();
    let project = setup(dir.path(), &[("a.typ", ALPHA)], shadow_cfg);
    run_with(&project, &flip_args("done", &["a"])).unwrap();
}

#[test]
fn restamp_prints_re_stamped() {
    if std::env::var(FLIP_CHILD_ENV).is_err() {
        let o = child_output("restamp_prints_re_stamped");
        assert!(
            o.status.success(),
            "child: {}",
            String::from_utf8_lossy(&o.stderr)
        );
        let out = String::from_utf8_lossy(&o.stdout);
        assert!(
            out.contains("re-stamped"),
            "expected 're-stamped' for same-status flip: {out}"
        );
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let project = setup(dir.path(), &[("a.typ", ALPHA)], LEGEND_CONFIG);
    run_with(&project, &flip_args("wip", &["a"])).unwrap();
}

#[test]
fn flip_prints_flipped_not_re_stamped() {
    if std::env::var(FLIP_CHILD_ENV).is_err() {
        let o = child_output("flip_prints_flipped_not_re_stamped");
        assert!(
            o.status.success(),
            "child: {}",
            String::from_utf8_lossy(&o.stderr)
        );
        let out = String::from_utf8_lossy(&o.stdout);
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
    let dir = tempfile::tempdir().unwrap();
    let project = setup(dir.path(), &[("a.typ", ALPHA)], LEGEND_CONFIG);
    run_with(&project, &flip_args("done", &["a"])).unwrap();
}

#[test]
fn terminal_flip_prints_recur_notice() {
    if std::env::var(FLIP_CHILD_ENV).is_err() {
        let o = child_output("terminal_flip_prints_recur_notice");
        assert!(
            o.status.success(),
            "child: {}",
            String::from_utf8_lossy(&o.stderr)
        );
        let out = String::from_utf8_lossy(&o.stdout);
        assert!(
            out.contains("recurring"),
            "expected recurring notice: {out}"
        );
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let project = setup(dir.path(), &[("g.typ", GAMMA)], LEGEND_CONFIG);
    run_with(&project, &flip_args("done", &["g"])).unwrap();
}

/// A legend whose entries carry glyphs renders them in the `--list` output —
/// the `Some(glyph)` arm of `list_verbs`' entry loop, which the floor-free
/// `LEGEND_CONFIG` never reaches.
#[test]
fn list_prints_glyph_verbs() {
    if std::env::var(FLIP_CHILD_ENV).is_err() {
        let o = child_output("list_prints_glyph_verbs");
        assert!(
            o.status.success(),
            "child: {}",
            String::from_utf8_lossy(&o.stderr)
        );
        let out = String::from_utf8_lossy(&o.stdout);
        assert!(
            out.contains("🔨 wip") && out.contains("✅ done"),
            "expected glyph-prefixed legend verbs in --list output: {out}"
        );
        return;
    }
    let glyph_cfg = "[[tasks.status]]\nvalue = \"wip\"\nglyph = \"🔨\"\n\
        [[tasks.status]]\nvalue = \"done\"\nglyph = \"✅\"\nterminal = true\n";
    let dir = tempfile::tempdir().unwrap();
    let project = setup(dir.path(), &[], glyph_cfg);
    let mut args = flip_args("done", &[]);
    args.status = None;
    args.list = true;
    run_with(&project, &args).unwrap();
}

/// An explicitly emptied legend (`status = []`, which unconstrains the floored
/// section) hits `list_verbs`' no-legend branch: the hint, not a verb listing.
#[test]
fn list_empty_legend_prints_hint() {
    if std::env::var(FLIP_CHILD_ENV).is_err() {
        let o = child_output("list_empty_legend_prints_hint");
        assert!(
            o.status.success(),
            "child: {}",
            String::from_utf8_lossy(&o.stderr)
        );
        let out = String::from_utf8_lossy(&o.stdout);
        assert!(
            out.contains("no status legend configured")
                && out.contains("shortcut verbs are unavailable"),
            "expected the empty-legend hint in --list output: {out}"
        );
        return;
    }
    let empty_cfg = "[tasks]\nstatus = []\n";
    let dir = tempfile::tempdir().unwrap();
    let project = setup(dir.path(), &[], empty_cfg);
    let mut args = flip_args("done", &[]);
    args.status = None;
    args.list = true;
    run_with(&project, &args).unwrap();
}

#[test]
fn list_prints_verbs() {
    if std::env::var(FLIP_CHILD_ENV).is_err() {
        let o = child_output("list_prints_verbs");
        assert!(
            o.status.success(),
            "child: {}",
            String::from_utf8_lossy(&o.stderr)
        );
        let out = String::from_utf8_lossy(&o.stdout);
        assert!(
            out.contains("wip"),
            "expected legend verbs in --list output: {out}"
        );
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let project = setup(dir.path(), &[], LEGEND_CONFIG);
    let mut args = flip_args("done", &[]);
    args.status = None;
    args.list = true;
    run_with(&project, &args).unwrap();
}

/// The revalidation violation itself is printed, not returned: `run_with`'s
/// error carries only the problem count, so the message naming the field and
/// the reason is only observable on stderr.
#[test]
fn revalidate_prints_the_unreachable_violation() {
    if std::env::var(FLIP_CHILD_ENV).is_err() {
        let o = child_output("revalidate_prints_the_unreachable_violation");
        let err = String::from_utf8_lossy(&o.stderr);
        assert!(
            err.contains("s: status: `blocked` is unreachable"),
            "expected the reachability violation naming task and field: {err}"
        );
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let project = setup(dir.path(), &[("s.typ", STATELESS)], REACHABLE_CONFIG);
    assert!(run_with(&project, &flip_args("blocked", &["s"])).is_err());
}

/// A rewrite that would not evaluate is refused *before* the write, so the
/// report says the source did not load rather than that the file did — the
/// difference being that there is no file. The message is only observable on a
/// child's stderr: `run_with`'s error carries the problem count alone.
#[test]
fn refuses_a_rewrite_that_would_not_load() {
    if std::env::var(FLIP_CHILD_ENV).is_err() {
        let o = child_output("refuses_a_rewrite_that_would_not_load");
        let err = String::from_utf8_lossy(&o.stderr);
        assert!(
            err.contains("alpha: did not load:"),
            "expected the pre-write refusal naming the task: {err}"
        );
        assert!(
            err.contains("nothing was written"),
            "the refusal must say the file was left alone: {err}"
        );
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let project = setup(dir.path(), &[("alpha.typ", ALPHA)], BROKEN_RELOAD_CONFIG);
    assert!(run_with(&project, &flip_args("broken\"done", &["alpha"])).is_err());
    assert_eq!(
        std::fs::read_to_string(dir.path().join("tasks/alpha.typ")).unwrap(),
        ALPHA,
        "a refused flip must leave the file byte-identical"
    );
}
