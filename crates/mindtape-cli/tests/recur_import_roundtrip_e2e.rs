//! End-to-end proof that `mt ls --json | mt import` preserves `recur` —
//! `truth/tasks/carry-recur-through-the-canonical-task-json-round-trip.typ`.
//! `examples/recurring` is both the fixture and the dogfood: `mt ls --json`
//! reads it directly (a pure command; the export needs no scratch copy), and
//! the export is piped into a scratch project sharing its `[tasks.recur]`
//! policy and status legend.
//!
//! The fixture's one `rrule(...)` task needs both `[tasks.recur] rrule =
//! true` (carried into the scratch config below, matching the fixture) AND
//! the `rrule` cargo feature to import cleanly — `just check-rrule` is the
//! only recipe that builds with it (see `Justfile`), so a default build
//! skips writing that one record (`check_policy`'s cargo-feature gate, an
//! existing, orthogonal limitation this task does not touch) and the
//! assertion below excludes it rather than asserting it too.
//!
//! The fixture's two same-titled "Post standup notes" tasks are excluded
//! outright: `mt import` re-stems from `title`, not the exported `id`/`file`,
//! so the `recurred-from` link on one of the pair — authored against the
//! ORIGINAL stem (`standup-notes-2026-09-14.typ`) — cannot resolve against
//! the freshly-disambiguated one (`post-standup-notes.typ`), and that record
//! is skipped. Pre-existing and orthogonal to `recur` (any same-titled linked
//! pair hits it); not this task's gap to close.
#![allow(clippy::unwrap_used, clippy::disallowed_methods)]

use std::path::{Path, PathBuf};

mod support;

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap()
}

/// A scratch project sharing `examples/recurring`'s status legend and
/// `[tasks.recur]` policy — not its `[[typst.local-packages]]` override,
/// which `support::install_lib` substitutes with the in-tree prelude
/// directly regardless of project location.
fn scaffold(dir: &Path) {
    std::fs::create_dir_all(dir.join(".mindtape")).unwrap();
    std::fs::write(
        dir.join(".mindtape/config.toml"),
        "[tasks]\nroots = [\"tasks\"]\n\
         [tasks.recur]\nrrule = true\n\
         [[tasks.status]]\nvalue = \"todo\"\n\
         [[tasks.status]]\nvalue = \"wip\"\n\
         [[tasks.status]]\nvalue = \"done\"\nterminal = true\n\
         [[tasks.status]]\nvalue = \"cancelled\"\nterminal = true\n",
    )
    .unwrap();
    support::install_lib(dir);
}

/// `(title, recur)` pairs from `mt ls --json`'s envelope (`{"tasks": [...],
/// ...}` — `mt ls` always has rule/view context, so it always envelope-wraps,
/// unlike `mt eval --json`'s bare array), sorted by title so two duplicate
/// titles (the fixture's pair of "Post standup notes" instances) compare as a
/// multiset rather than colliding in a map.
fn recur_pairs(json: &str) -> Vec<(String, serde_json::Value)> {
    let envelope: serde_json::Value = serde_json::from_str(json).unwrap();
    let mut pairs: Vec<(String, serde_json::Value)> = envelope
        .get("tasks")
        .and_then(serde_json::Value::as_array)
        .expect("mt ls --json always envelopes a `tasks` array")
        .iter()
        .map(|record| {
            let title = record
                .get("title")
                .and_then(serde_json::Value::as_str)
                .unwrap()
                .to_owned();
            let recur = record
                .get("recur")
                .cloned()
                .unwrap_or(serde_json::Value::Null);
            (title, recur)
        })
        .collect();
    pairs.sort_by(|left, right| left.0.cmp(&right.0));
    pairs
}

#[test]
fn recur_survives_ls_json_piped_into_import() {
    let fixture = repo_root().join("examples/recurring");
    // An explicit (empty) filter overrides the project's default `!terminal`
    // rule, so the export carries every task — including the terminal
    // "Post standup notes" instance the other one's `recurred-from` link
    // targets, which a filtered export would drop and leave unresolved.
    let exported = support::mt(&fixture)
        .args(["ls", "", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let exported = String::from_utf8(exported).unwrap();

    let dir = tempfile::tempdir().unwrap();
    scaffold(dir.path());
    support::mt(dir.path())
        .arg("import")
        .write_stdin(exported.clone())
        .assert()
        .code(predicates::ord::le(1));

    let reimported = support::mt(dir.path())
        .args(["ls", "", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let reimported = String::from_utf8(reimported).unwrap();

    let excluded = |title: &str| {
        title == "Post standup notes"
            || (title == "Send the cost report" && !cfg!(feature = "rrule"))
    };
    let mut before = recur_pairs(&exported);
    before.retain(|(title, _)| !excluded(title));
    let mut after = recur_pairs(&reimported);
    after.retain(|(title, _)| !excluded(title));
    assert!(
        !before.is_empty(),
        "nothing left to compare after exclusions"
    );
    assert_eq!(
        before, after,
        "recur did not survive the round trip:\n--- exported ---\n{exported}\n--- reimported ---\n{reimported}"
    );
}
