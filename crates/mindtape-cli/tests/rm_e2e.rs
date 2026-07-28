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

mod support;

/// A tasks root plus a custom `blocks` edge kind, so the survivor's
/// `edge("blocks", …)` resolves and `mt check` accepts it.
const CONFIG: &str = "[tasks]\nroots = [\"tasks\"]\n\n[[tasks.links.kinds]]\nname = \"blocks\"\n";

/// The retired task: terminal, so `mt rm` sweeps it without a target override.
const GONE: &str = "#import \"@local/mindtape:0.2.0\": *\n\n\
#show: task.with(\n  \
title: \"Gone\",\n  \
status: done(2026, 6, 1),\n\
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

fn git(dir: &Path, args: &[&str]) {
    let status = Command::new("git")
        .current_dir(dir)
        .args(args)
        .status()
        .unwrap();
    assert!(status.success(), "git {args:?} failed");
}

/// Scaffold a committed-clean git repo with the config, in-tree lib, and tasks.
fn scaffold(dir: &Path) {
    std::fs::create_dir_all(dir.join(".mindtape")).unwrap();
    std::fs::write(dir.join(".mindtape/config.toml"), CONFIG).unwrap();
    support::install_lib(dir);
    std::fs::write(dir.join("tasks/gone.typ"), GONE).unwrap();
    std::fs::write(dir.join("tasks/live.typ"), LIVE).unwrap();
    git(dir, &["init", "-q"]);
    git(dir, &["config", "user.email", "t@example.com"]);
    git(dir, &["config", "user.name", "t"]);
    git(dir, &["add", "-A"]);
    git(
        dir,
        &["-c", "commit.gpgsign=false", "commit", "-q", "-m", "seed"],
    );
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
