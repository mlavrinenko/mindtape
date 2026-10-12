//! End-to-end proof that `mt ls --json | mt import` preserves `recur` —
//! `truth/tasks/carry-recur-through-the-canonical-task-json-round-trip.typ`.
//! `examples/recurring` is both the fixture and the dogfood: `mt ls --json`
//! reads it directly (a pure command; the export needs no scratch copy), and
//! the export is piped into a scratch project sharing its `[tasks.recur]`
//! policy and status legend. The fixture's one `rrule(...)` task needs
//! `[tasks.recur] rrule = true`, which the scratch config carries too.
#![allow(clippy::unwrap_used, clippy::disallowed_methods)]

use std::path::{Path, PathBuf};

use crate::support;

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
        .success();

    let reimported = support::mt(dir.path())
        .args(["ls", "", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let reimported = String::from_utf8(reimported).unwrap();

    let before = recur_pairs(&exported);
    let after = recur_pairs(&reimported);
    assert!(
        before
            .iter()
            .any(|(title, _)| title == "Send the cost report"),
        "the export lacks the fixture's rrule(...) task:\n{exported}"
    );
    assert_eq!(
        before, after,
        "recur did not survive the round trip:\n--- exported ---\n{exported}\n--- reimported ---\n{reimported}"
    );
}
