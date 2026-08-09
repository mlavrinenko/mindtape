//! End-to-end tests for `mt rm`'s retire strategy: flipping the removed task to
//! a terminal status in place, moving it into a `[retire] rename` archive
//! folder, and the confirmation-prompt and dry-run paths. Shares the `gone`/
//! `live` fixtures and the scaffolding in `rm_e2e.rs` through `use super::*`.
#![allow(clippy::unwrap_used)]

use super::*;

const RETIRE_CONFIG: &str = "[tasks]\nroots = [\"tasks\"]\n\n[[tasks.links.kinds]]\nname = \"blocks\"\n\n[rm]\nstrategy = \"retire\"\n";

const RETIRE_STATUS_CONFIG: &str = "[tasks]\nroots = [\"tasks\"]\n\n[[tasks.links.kinds]]\nname = \"blocks\"\n\n[rm]\nstrategy = \"retire\"\n[retire]\nstatus = \"cancelled\"\n";

const RETIRE_ARCHIVE_CONFIG: &str = "[tasks]\nroots = [\"tasks\"]\n\n[[tasks.links.kinds]]\nname = \"blocks\"\n\n[rm]\nstrategy = \"retire\"\n[retire]\nrename = \".archive\"\n";

/// Scaffold a retire-strategy project (the `gone`/`live` pair) under `config`
/// and run `mt rm --dry-run -y gone`, asserting success and the "strategy:
/// retire" preamble every retire dry-run test shares. Callers chain their own
/// further `.stdout(...)` checks onto the returned `Assert`.
fn retire_dry_run(dir: &Path, config: &str) -> assert_cmd::assert::Assert {
    scaffold_config(dir, config, &[("gone.typ", REMOVED), ("live.typ", LIVE)]);
    mt(dir)
        .args(["rm", "--dry-run", "-y", "gone"])
        .assert()
        .success()
        .stdout(predicates::str::contains("strategy: retire"))
}

#[test]
fn rm_retire_flips_to_first_terminal_status_without_git() {
    let dir = tempfile::tempdir().unwrap();
    retire_dry_run(dir.path(), RETIRE_CONFIG).stdout(predicates::str::contains(
        "would retire 1 task(s) (status → done)",
    ));
    assert!(dir.path().join("tasks/gone.typ").exists());

    // No git anywhere: the retire report's commit hint must not print either.
    mt(dir.path())
        .args(["rm", "-y", "gone"])
        .assert()
        .success()
        .stdout(predicates::str::contains("review the changes and commit").not());

    // The file stays, flipped to the legend's first terminal status.
    assert!(
        dir.path().join("tasks/gone.typ").exists(),
        "retired file must stay"
    );
    let retired = std::fs::read_to_string(dir.path().join("tasks/gone.typ")).unwrap();
    assert!(retired.contains("done("), "not flipped:\n{retired}");
    assert!(!retired.contains("wip("), "still wip:\n{retired}");
    // The inbound edge still resolves — the file stayed put.
    mt(dir.path()).arg("check").assert().success();
}

#[test]
fn rm_retire_prints_commit_hint_inside_git_repo() {
    // The retire strategy needs no git either, but when a repository does
    // exist the hint belongs — same emitter (`rm_gitless_report.rs`) as the
    // gitless case above, opposite repository state.
    let dir = tempfile::tempdir().unwrap();
    scaffold_config(
        dir.path(),
        RETIRE_CONFIG,
        &[("gone.typ", REMOVED), ("live.typ", LIVE)],
    );
    git_commit_all(dir.path());

    mt(dir.path())
        .args(["rm", "-y", "gone"])
        .assert()
        .success()
        .stdout(predicates::str::contains("review the changes and commit"));
}

#[test]
fn rm_retire_uses_configured_status_without_git() {
    let dir = tempfile::tempdir().unwrap();
    scaffold_config(
        dir.path(),
        RETIRE_STATUS_CONFIG,
        &[("gone.typ", REMOVED), ("live.typ", LIVE)],
    );

    mt(dir.path()).args(["rm", "-y", "gone"]).assert().success();

    let retired = std::fs::read_to_string(dir.path().join("tasks/gone.typ")).unwrap();
    assert!(retired.contains("cancelled("), "not cancelled:\n{retired}");
    mt(dir.path()).arg("check").assert().success();
}

#[test]
fn rm_retire_moves_the_file_into_the_archive_folder() {
    let dir = tempfile::tempdir().unwrap();
    scaffold_config(
        dir.path(),
        RETIRE_ARCHIVE_CONFIG,
        &[("gone.typ", REMOVED), ("live.typ", LIVE)],
    );

    mt(dir.path()).args(["rm", "-y", "gone"]).assert().success();

    assert!(!dir.path().join("tasks/gone.typ").exists());
    let archived = dir.path().join("tasks/.archive/gone.typ");
    assert!(archived.exists(), "not moved into .archive/");
    let contents = std::fs::read_to_string(&archived).unwrap();
    assert!(contents.contains("done("), "not flipped:\n{contents}");
    // The inbound edges retarget onto the new path — no dangling reference.
    let live = std::fs::read_to_string(dir.path().join("tasks/live.typ")).unwrap();
    assert!(
        live.contains(".archive/gone.typ"),
        "not retargeted:\n{live}"
    );
    assert!(
        !live.contains("\"gone.typ\""),
        "old target survived:\n{live}"
    );
    mt(dir.path()).arg("check").assert().success();
}

#[test]
fn rm_retire_prompt_lists_the_plan_and_an_empty_answer_aborts() {
    // Every other retire test passes `-y`, which skips `confirm_retire`
    // outright — so the prompt the retire path prints was never executed by the
    // suite while the delete path's was. Omitting `-y` reaches it: the empty
    // stdin reads as "no", the way the delete-path tests already rely on.
    let dir = tempfile::tempdir().unwrap();
    scaffold_config(
        dir.path(),
        RETIRE_CONFIG,
        &[("gone.typ", REMOVED), ("live.typ", LIVE)],
    );

    mt(dir.path())
        .args(["rm", "gone"])
        .assert()
        .failure()
        .stderr(predicates::str::contains("rm will retire 1 task(s) to done:"))
        .stderr(predicates::str::contains("gone.typ"))
        .stderr(predicates::str::contains("aborted by user"))
        // Retiring in place leaves the path alone, so no inbound edge moves and
        // the retarget line stays off. `…archive…` below is the opposite case.
        .stderr(predicates::str::contains("and retarget").not());

    // Declined means nothing was written: the task keeps its non-terminal status.
    let untouched = std::fs::read_to_string(dir.path().join("tasks/gone.typ")).unwrap();
    assert!(
        untouched.contains("wip("),
        "flipped despite abort:\n{untouched}"
    );
}

#[test]
fn rm_retire_prompt_names_the_archive_move_and_a_piped_yes_applies_it() {
    // The `rename` branch of the prompt, plus the accepting answer — the two
    // paths the aborting test above leaves unexecuted.
    let dir = tempfile::tempdir().unwrap();
    scaffold_config(
        dir.path(),
        RETIRE_ARCHIVE_CONFIG,
        &[("gone.typ", REMOVED), ("live.typ", LIVE)],
    );

    mt(dir.path())
        .args(["rm", "gone"])
        .write_stdin("y\n")
        .assert()
        .success()
        .stderr(predicates::str::contains("and move 1 task(s) into"))
        // Moving the file does retarget the three inbound edges of `live`.
        .stderr(predicates::str::contains(
            "and retarget 3 link(s) in 1 file(s)",
        ));

    assert!(dir.path().join("tasks/.archive/gone.typ").exists());
    mt(dir.path()).arg("check").assert().success();
}

#[test]
fn rm_retire_dry_run_previews_the_archive_move_and_writes_nothing() {
    // `report_retire_plan`'s `rename` branch (the "would move" preview, plus
    // the per-file `old → new` mapping) is reachable only when a dry-run
    // retire is combined with `[retire] rename` — every other retire test
    // above either has no rename configured or applies for real. Neither
    // `rm_retire_flips_to_first_terminal_status_without_git` (no rename) nor
    // `rm_retire_moves_the_file_into_the_archive_folder` (rename, but applied)
    // exercises it.
    let dir = tempfile::tempdir().unwrap();
    retire_dry_run(dir.path(), RETIRE_ARCHIVE_CONFIG)
        .stdout(predicates::str::contains(
            "would retire 1 task(s) (status → done):",
        ))
        .stdout(predicates::str::contains(
            "would move 1 task(s) into .archive/:",
        ))
        .stdout(predicates::str::contains(
            "tasks/gone.typ → tasks/.archive/gone.typ",
        ))
        .stdout(predicates::str::contains(
            "would rewrite 3 link(s) in 1 file(s):",
        ))
        .stdout(predicates::str::contains("dry run: nothing written"));

    // A preview writes nothing: the file stays put, non-terminal, and the
    // survivor's edges still point at the untouched original path.
    assert!(dir.path().join("tasks/gone.typ").exists());
    assert!(!dir.path().join("tasks/.archive/gone.typ").exists());
    let gone = std::fs::read_to_string(dir.path().join("tasks/gone.typ")).unwrap();
    assert!(gone.contains("wip("), "flipped despite dry-run:\n{gone}");
    let live = std::fs::read_to_string(dir.path().join("tasks/live.typ")).unwrap();
    assert!(
        live.contains("\"gone.typ\""),
        "retargeted despite dry-run:\n{live}"
    );
}
