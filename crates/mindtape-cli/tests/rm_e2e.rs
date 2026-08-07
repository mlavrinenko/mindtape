//! End-to-end test for `mt rm` retargeting `links:` edges onto SWHID tombstones.
//!
//! Lives here (not an inline `#[path]` unit test) so it exercises the real
//! prelude edge helpers (`depends-on`, `related`, `edge`), a real git repo (the
//! tombstone pins the retired file's last-touch commit), and the compiled `mt`
//! binary end-to-end — then proves `mt check` stays green once the survivor's
//! inbound edges point at the tombstone. Mirrors `recur_e2e.rs`.
#![allow(clippy::unwrap_used)]

use std::path::Path;
use std::process::Command;

use predicates::prelude::PredicateBooleanExt;

mod support;

/// A tasks root plus a custom `blocks` edge kind, so the survivor's
/// `edge("blocks", …)` resolves and `mt check` accepts it.
const CONFIG: &str = "[tasks]\nroots = [\"tasks\"]\n\n[[tasks.links.kinds]]\nname = \"blocks\"\n";

const UNLINK_CONFIG: &str = "[tasks]\nroots = [\"tasks\"]\n\n[[tasks.links.kinds]]\nname = \"blocks\"\n\n[rm]\nstrategy = \"unlink\"\n";

const RETIRE_CONFIG: &str = "[tasks]\nroots = [\"tasks\"]\n\n[[tasks.links.kinds]]\nname = \"blocks\"\n\n[rm]\nstrategy = \"retire\"\n";

const RETIRE_STATUS_CONFIG: &str = "[tasks]\nroots = [\"tasks\"]\n\n[[tasks.links.kinds]]\nname = \"blocks\"\n\n[rm]\nstrategy = \"retire\"\n[retire]\nstatus = \"cancelled\"\n";

const RETIRE_ARCHIVE_CONFIG: &str = "[tasks]\nroots = [\"tasks\"]\n\n[[tasks.links.kinds]]\nname = \"blocks\"\n\n[rm]\nstrategy = \"retire\"\n[retire]\nrename = \".archive\"\n";

const BATCH_CONFIG: &str = "[tasks]\nroots = [\"tasks\"]\n\n[[tasks.links.kinds]]\nname = \"blocks\"\n\n[rm]\ndefault-filter = \"status == \\\"wip\\\"\"\n";

/// The retired task: terminal, so `mt rm` sweeps it without a target override.
const GONE: &str = "#import \"@local/mindtape:0.2.0\": *\n\n\
#show: task.with(\n  \
title: \"Gone\",\n  \
status: done(2026, 6, 1),\n\
)\n\n== Summary\n\nBody.\n";

/// A non-terminal removed task: `retire` must flip it, and the strategy must
/// kick in on its inbound edge even though the file itself is not terminal.
const REMOVED: &str = "#import \"@local/mindtape:0.2.0\": *\n\n\
#show: task.with(\n  \
title: \"Gone\",\n  \
status: wip(2026, 6, 1),\n\
)\n\n== Summary\n\nBody.\n";

/// The survivor: three inbound edges to `gone`, one per kind the fix must
/// retarget — plain `depends-on`, the note-bearing `related` escape hatch, and a
/// custom kind via `edge(...)`. None uses `link(...)`.
const LIVE: &str = "#import \"@local/mindtape:0.2.0\": *\n\n\
#show: task.with(\n  \
title: \"Live\",\n  \
status: wip(2026, 6, 1),\n  \
links: depends-on(\"gone.typ\") + related(\"gone.typ\")[why] + edge(\"blocks\", \"gone.typ\"),\n\
)\n\n== Summary\n\nBody.\n";

/// A survivor that never references `gone` — the unreferenced-removal case,
/// where deletion needs no tombstone and no git.
const UNRELATED: &str = "#import \"@local/mindtape:0.2.0\": *\n\n\
#show: task.with(\n  \
title: \"Unrelated\",\n  \
status: wip(2026, 6, 1),\n\
)\n\n== Summary\n\nBody.\n";

fn git(dir: &Path, args: &[&str]) {
    let status = Command::new("git")
        .current_dir(dir)
        .args(args)
        .status()
        .unwrap();
    assert!(status.success(), "git {args:?} failed");
}

/// Scaffold the project files — config, in-tree lib, and the given task
/// sources — without initializing git.
fn scaffold_files(dir: &Path, files: &[(&str, &str)]) {
    scaffold_config(dir, CONFIG, files);
}

/// Scaffold the project files with an explicit config instead of [`CONFIG`].
fn scaffold_config(dir: &Path, config: &str, files: &[(&str, &str)]) {
    std::fs::create_dir_all(dir.join(".mindtape")).unwrap();
    std::fs::write(dir.join(".mindtape/config.toml"), config).unwrap();
    support::install_lib(dir);
    for (name, contents) in files {
        std::fs::write(dir.join("tasks").join(name), contents).unwrap();
    }
}

/// The git half of [`scaffold`]: init, identity, and a committed-clean seed.
fn git_commit_all(dir: &Path) {
    git(dir, &["init", "-q"]);
    git(dir, &["config", "user.email", "t@example.com"]);
    git(dir, &["config", "user.name", "t"]);
    git(dir, &["add", "-A"]);
    git(
        dir,
        &["-c", "commit.gpgsign=false", "commit", "-q", "-m", "seed"],
    );
}

/// Scaffold a committed-clean git repo with the config, in-tree lib, and the
/// referencing task pair.
fn scaffold(dir: &Path) {
    scaffold_files(dir, &[("gone.typ", GONE), ("live.typ", LIVE)]);
    git_commit_all(dir);
}

fn mt(dir: &Path) -> assert_cmd::Command {
    support::mt(dir)
}

#[test]
fn rm_retargets_every_links_edge_and_check_stays_green() {
    let dir = tempfile::tempdir().unwrap();
    scaffold(dir.path());

    // Retire `gone`; `-y` skips the confirmation prompt.
    mt(dir.path()).args(["rm", "-y", "gone"]).assert().success();

    // The retired file is gone; the survivor's edges now point at the tombstone.
    assert!(!dir.path().join("tasks/gone.typ").exists());
    let live = std::fs::read_to_string(dir.path().join("tasks/live.typ")).unwrap();
    assert!(
        live.contains("swh:1:rev:"),
        "no tombstone in survivor:\n{live}"
    );
    assert!(
        !live.contains("\"gone.typ\""),
        "a raw edge target survived the retarget:\n{live}"
    );
    // The `related` note and the custom kind name are untouched by the splice.
    assert!(live.contains("[why]"), "note dropped:\n{live}");
    assert!(live.contains("edge(\"blocks\", "), "kind dropped:\n{live}");

    // The dangling-edge regression: with every inbound edge tombstoned, no
    // survivor edge resolves to a missing task, so validation stays green.
    mt(dir.path()).arg("check").assert().success();
}

#[test]
fn rm_deletes_unreferenced_task_outside_git_repo() {
    // No repository anywhere: a task nothing references must be removable
    // anyway — no toplevel discovery, no tombstone, no committed-clean guard.
    let dir = tempfile::tempdir().unwrap();
    scaffold_files(
        dir.path(),
        &[("gone.typ", GONE), ("unrelated.typ", UNRELATED)],
    );

    // --dry-run reports the plan without git and writes nothing.
    mt(dir.path())
        .args(["rm", "--dry-run", "-y", "gone"])
        .assert()
        .success();
    assert!(dir.path().join("tasks/gone.typ").exists());

    // No git anywhere: the hint would point at a tool the user deliberately
    // does not have, so it must not print.
    mt(dir.path())
        .args(["rm", "-y", "gone"])
        .assert()
        .success()
        .stdout(predicates::str::contains("review the changes and commit").not());

    assert!(!dir.path().join("tasks/gone.typ").exists());
    // The survivor is untouched — no tombstone, no rewrite.
    let unrelated = std::fs::read_to_string(dir.path().join("tasks/unrelated.typ")).unwrap();
    assert!(!unrelated.contains("swh:1:rev:"), "survivor was rewritten");
    mt(dir.path()).arg("check").assert().success();
}

#[test]
fn rm_deletes_unreferenced_task_inside_git_repo() {
    // The fast path also applies when git exists: nothing references `gone`,
    // so deletion still skips the repository entirely.
    let dir = tempfile::tempdir().unwrap();
    scaffold_files(
        dir.path(),
        &[("gone.typ", GONE), ("unrelated.typ", UNRELATED)],
    );
    git_commit_all(dir.path());

    // A git repository does exist here, so the commit hint belongs.
    mt(dir.path())
        .args(["rm", "-y", "gone"])
        .assert()
        .success()
        .stdout(predicates::str::contains("review the changes and commit"));

    assert!(!dir.path().join("tasks/gone.typ").exists());
    assert!(dir.path().join("tasks/unrelated.typ").exists());
    mt(dir.path()).arg("check").assert().success();
}

#[test]
fn rm_defaults_to_reject_without_git_and_lists_surviving_referrers() {
    // No repository, no configured strategy: the default is `reject`, which
    // refuses and names every surviving task that points at `gone` — dry-run
    // included, since reject has no plan to preview.
    let dir = tempfile::tempdir().unwrap();
    scaffold_files(dir.path(), &[("gone.typ", GONE), ("live.typ", LIVE)]);

    mt(dir.path())
        .args(["rm", "--dry-run", "-y", "gone"])
        .assert()
        .failure()
        .stderr(predicates::str::contains(
            "cannot remove task(s): 1 reference(s) from surviving task(s)",
        ))
        .stderr(predicates::str::contains("live.typ  links to gone"));

    mt(dir.path())
        .args(["rm", "-y", "gone"])
        .assert()
        .failure()
        .stderr(predicates::str::contains(
            "cannot remove task(s): 1 reference(s) from surviving task(s)",
        ));

    assert!(dir.path().join("tasks/gone.typ").exists());
    assert!(dir.path().join("tasks/live.typ").exists());
}

#[test]
fn rm_unlink_strips_inbound_edges_and_deletes_without_git() {
    let dir = tempfile::tempdir().unwrap();
    scaffold_config(
        dir.path(),
        UNLINK_CONFIG,
        &[("gone.typ", GONE), ("live.typ", LIVE)],
    );

    // --dry-run reports the strategy and its planned edits without any git.
    mt(dir.path())
        .args(["rm", "--dry-run", "-y", "gone"])
        .assert()
        .success()
        .stdout(predicates::str::contains("strategy: unlink"))
        .stdout(predicates::str::contains("would delete 1 task(s)"))
        .stdout(predicates::str::contains(
            "would rewrite 3 link(s) in 1 file(s)",
        ));
    assert!(dir.path().join("tasks/gone.typ").exists());

    mt(dir.path()).args(["rm", "-y", "gone"]).assert().success();

    assert!(!dir.path().join("tasks/gone.typ").exists());
    let live = std::fs::read_to_string(dir.path().join("tasks/live.typ")).unwrap();
    assert!(!live.contains("gone.typ"), "edge survived:\n{live}");
    assert!(!live.contains("+"), "separator survived:\n{live}");
    assert!(!live.contains("swh:1:rev:"), "tombstone written:\n{live}");
    mt(dir.path()).arg("check").assert().success();
}

#[test]
fn rm_retire_flips_to_first_terminal_status_without_git() {
    let dir = tempfile::tempdir().unwrap();
    scaffold_config(
        dir.path(),
        RETIRE_CONFIG,
        &[("gone.typ", REMOVED), ("live.typ", LIVE)],
    );

    mt(dir.path())
        .args(["rm", "--dry-run", "-y", "gone"])
        .assert()
        .success()
        .stdout(predicates::str::contains("strategy: retire"))
        .stdout(predicates::str::contains(
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
fn rm_batch_removing_every_referrer_needs_no_strategy_without_git() {
    // The only referrer of `gone` is `live`, and both are being removed — the
    // batch must not reject or trim its own edges, so deletion is plain.
    let dir = tempfile::tempdir().unwrap();
    scaffold_config(
        dir.path(),
        BATCH_CONFIG,
        &[("gone.typ", REMOVED), ("live.typ", LIVE)],
    );

    mt(dir.path()).args(["rm", "-y"]).assert().success();

    assert!(!dir.path().join("tasks/gone.typ").exists());
    assert!(!dir.path().join("tasks/live.typ").exists());
}
