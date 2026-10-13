//! End-to-end test for `mt rm` retargeting `links:` edges onto SWHID tombstones.
//!
//! Lives here (not an inline `#[path]` unit test) so it exercises the real
//! prelude edge helpers (`depends-on`, `related`, `edge`), a real git repo (the
//! tombstone pins the retired file's last-touch commit), and the compiled `mt`
//! binary end-to-end — then proves `mt check` stays green once the survivor's
//! inbound edges point at the tombstone. Mirrors `recur_e2e.rs`. The retire
//! strategy's half of the suite lives in `retire.rs`, pulled in below.
#![allow(clippy::unwrap_used)]

use std::path::Path;

use predicates::prelude::PredicateBooleanExt;

mod retire;
use crate::support;

use support::{git_commit_all, scaffold_config};

/// A tasks root plus a custom `blocks` edge kind, so the survivor's
/// `edge("blocks", …)` resolves and `mt check` accepts it.
const CONFIG: &str = "[tasks]\nroots = [\"tasks\"]\n\n[[tasks.links.kinds]]\nname = \"blocks\"\n";

const UNLINK_CONFIG: &str = "[tasks]\nroots = [\"tasks\"]\n\n[[tasks.links.kinds]]\nname = \"blocks\"\n\n[rm]\nstrategy = \"unlink\"\n";

const BATCH_CONFIG: &str = "[tasks]\nroots = [\"tasks\"]\n\n[[tasks.links.kinds]]\nname = \"blocks\"\n\n[rm]\ndefault-filter = \"status == \\\"wip\\\"\"\n";

/// The retired task: terminal, so `mt rm` sweeps it without a target override.
const GONE: &str = mindtape::typst_task!(
    "\n\
#show: task.with(\n  \
title: \"Gone\",\n  \
status: done(2026, 6, 1),\n\
)\n\n== Summary\n\nBody.\n"
);

/// A non-terminal removed task: `retire` must flip it, and the strategy must
/// kick in on its inbound edge even though the file itself is not terminal.
const REMOVED: &str = mindtape::typst_task!(
    "\n\
#show: task.with(\n  \
title: \"Gone\",\n  \
status: wip(2026, 6, 1),\n\
)\n\n== Summary\n\nBody.\n"
);

/// The survivor: three inbound edges to `gone`, one per kind the fix must
/// retarget — plain `depends-on`, the note-bearing `related` escape hatch, and a
/// custom kind via `edge(...)`. None uses `link(...)`.
const LIVE: &str = mindtape::typst_task!(
    "\n\
#show: task.with(\n  \
title: \"Live\",\n  \
status: wip(2026, 6, 1),\n  \
links: depends-on(\"gone.typ\") + related(\"gone.typ\")[why] + edge(\"blocks\", \"gone.typ\"),\n\
)\n\n== Summary\n\nBody.\n"
);

/// A survivor that never references `gone` — the unreferenced-removal case,
/// where deletion needs no tombstone and no git.
const UNRELATED: &str = mindtape::typst_task!(
    "\n\
#show: task.with(\n  \
title: \"Unrelated\",\n  \
status: wip(2026, 6, 1),\n\
)\n\n== Summary\n\nBody.\n"
);

/// Two independently-removed tasks (`gone-a`, `gone-b`), each with its own
/// survivor, for the multi-target `mt rm` batch below.
const GONE_A: &str = mindtape::typst_task!(
    "\n\
#show: task.with(\n  \
title: \"Gone A\",\n  \
status: done(2026, 6, 1),\n\
)\n\n== Summary\n\nBody.\n"
);

const GONE_B: &str = mindtape::typst_task!(
    "\n\
#show: task.with(\n  \
title: \"Gone B\",\n  \
status: done(2026, 6, 1),\n\
)\n\n== Summary\n\nBody.\n"
);

const LIVE_A: &str = mindtape::typst_task!(
    "\n\
#show: task.with(\n  \
title: \"Live A\",\n  \
status: wip(2026, 6, 1),\n  \
links: depends-on(\"gone-a.typ\"),\n\
)\n\n== Summary\n\nBody.\n"
);

const LIVE_B: &str = mindtape::typst_task!(
    "\n\
#show: task.with(\n  \
title: \"Live B\",\n  \
status: wip(2026, 6, 1),\n  \
links: depends-on(\"gone-b.typ\"),\n\
)\n\n== Summary\n\nBody.\n"
);

/// Scaffold the project files — config, in-tree lib, and the given task
/// sources — without initializing git.
fn scaffold_files(files: &[(&str, &str)]) -> tempfile::TempDir {
    scaffold_config(CONFIG, files)
}

/// Scaffold a committed-clean git repo with the config, in-tree lib, and the
/// referencing task pair.
fn scaffold() -> tempfile::TempDir {
    let dir = scaffold_files(&[("gone.typ", GONE), ("live.typ", LIVE)]);
    git_commit_all(dir.path());
    dir
}

fn mt(dir: &Path) -> assert_cmd::Command {
    support::mt(dir)
}

#[test]
fn rm_retargets_every_links_edge_and_check_stays_green() {
    let dir = scaffold();

    // Retire `gone`; `-y` skips the confirmation prompt.
    support::run_ok(dir.path(), &["rm", "-y", "gone"]);

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
    support::run_ok(dir.path(), &["check"]);
}

#[test]
fn rm_takes_two_refs_in_one_invocation_and_rewrites_each_inbound_link() {
    // TARGET is variadic: two refs in one call retire both, with one
    // confirmation prompt and one tombstone/rewrite pass covering the set —
    // not two separate `mt rm` invocations.
    let dir = scaffold_files(&[
        ("gone-a.typ", GONE_A),
        ("gone-b.typ", GONE_B),
        ("live-a.typ", LIVE_A),
        ("live-b.typ", LIVE_B),
    ]);
    git_commit_all(dir.path());

    mt(dir.path())
        .args(["rm", "-y", "gone-a", "gone-b"])
        .assert()
        .success()
        .stdout(predicates::str::contains("deleted 2 task(s)"))
        .stdout(predicates::str::contains("rewrote 2 link(s) in 2 file(s)"));

    // Both retired files are gone.
    assert!(!dir.path().join("tasks/gone-a.typ").exists());
    assert!(!dir.path().join("tasks/gone-b.typ").exists());

    // Each survivor's own inbound edge was rewritten onto its own tombstone.
    let live_a = std::fs::read_to_string(dir.path().join("tasks/live-a.typ")).unwrap();
    assert!(
        live_a.contains("swh:1:rev:") && live_a.contains("path=tasks/gone-a.typ"),
        "live-a not tombstoned:\n{live_a}"
    );
    assert!(
        !live_a.contains("\"gone-a.typ\""),
        "raw edge survived:\n{live_a}"
    );
    let live_b = std::fs::read_to_string(dir.path().join("tasks/live-b.typ")).unwrap();
    assert!(
        live_b.contains("swh:1:rev:") && live_b.contains("path=tasks/gone-b.typ"),
        "live-b not tombstoned:\n{live_b}"
    );
    assert!(
        !live_b.contains("\"gone-b.typ\""),
        "raw edge survived:\n{live_b}"
    );

    support::run_ok(dir.path(), &["check"]);
}

#[test]
fn rm_deletes_unreferenced_task_outside_git_repo() {
    // No repository anywhere: a task nothing references must be removable
    // anyway — no toplevel discovery, no tombstone, no committed-clean guard.
    let dir = scaffold_files(&[("gone.typ", GONE), ("unrelated.typ", UNRELATED)]);

    // --dry-run reports the plan without git and writes nothing.
    support::run_ok(dir.path(), &["rm", "--dry-run", "-y", "gone"]);
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
    support::run_ok(dir.path(), &["check"]);
}

#[test]
fn rm_deletes_unreferenced_task_inside_git_repo() {
    // The fast path also applies when git exists: nothing references `gone`,
    // so deletion still skips the repository entirely.
    let dir = scaffold_files(&[("gone.typ", GONE), ("unrelated.typ", UNRELATED)]);
    git_commit_all(dir.path());

    // A git repository does exist here, so the commit hint belongs.
    mt(dir.path())
        .args(["rm", "-y", "gone"])
        .assert()
        .success()
        .stdout(predicates::str::contains("review the changes and commit"));

    assert!(!dir.path().join("tasks/gone.typ").exists());
    assert!(dir.path().join("tasks/unrelated.typ").exists());
    support::run_ok(dir.path(), &["check"]);
}

#[test]
fn rm_defaults_to_reject_without_git_and_lists_surviving_referrers() {
    // No repository, no configured strategy: the default is `reject`, which
    // refuses and names every surviving task that points at `gone` — dry-run
    // included, since reject has no plan to preview.
    let dir = scaffold_files(&[("gone.typ", GONE), ("live.typ", LIVE)]);

    mt(dir.path())
        .args(["rm", "--dry-run", "-y", "gone"])
        .assert()
        .failure()
        .stderr(predicates::str::contains(
            "error: cannot remove tasks that other tasks link to\n",
        ))
        .stderr(predicates::str::contains(
            "note: link from: tasks/live.typ to gone\n",
        ));

    mt(dir.path())
        .args(["rm", "-y", "gone"])
        .assert()
        .failure()
        .stderr(predicates::str::contains(
            "error: cannot remove tasks that other tasks link to\n",
        ));

    assert!(dir.path().join("tasks/gone.typ").exists());
    assert!(dir.path().join("tasks/live.typ").exists());
}

#[test]
fn rm_unlink_strips_inbound_edges_and_deletes_without_git() {
    let dir = scaffold_config(UNLINK_CONFIG, &[("gone.typ", GONE), ("live.typ", LIVE)]);

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

    support::run_ok(dir.path(), &["rm", "-y", "gone"]);

    assert!(!dir.path().join("tasks/gone.typ").exists());
    let live = std::fs::read_to_string(dir.path().join("tasks/live.typ")).unwrap();
    assert!(!live.contains("gone.typ"), "edge survived:\n{live}");
    assert!(!live.contains("+"), "separator survived:\n{live}");
    assert!(!live.contains("swh:1:rev:"), "tombstone written:\n{live}");
    support::run_ok(dir.path(), &["check"]);
}

#[test]
fn rm_batch_removing_every_referrer_needs_no_strategy_without_git() {
    // The only referrer of `gone` is `live`, and both are being removed — the
    // batch must not reject or trim its own edges, so deletion is plain.
    let dir = scaffold_config(BATCH_CONFIG, &[("gone.typ", REMOVED), ("live.typ", LIVE)]);

    support::run_ok(dir.path(), &["rm", "-y"]);

    assert!(!dir.path().join("tasks/gone.typ").exists());
    assert!(!dir.path().join("tasks/live.typ").exists());
}
