//! Shared task fixtures for unit tests.
//!
//! Exposed behind the `test-support` feature so dependent crates (the CLI) can
//! reuse these builders in their own unit tests; within this crate they are
//! available to `#[cfg(test)]` code directly.

use std::collections::BTreeMap;
use std::path::Path;
use std::process::Command;

use crate::config::Config;
use crate::link::Link;
use crate::model::{Status, Task};
use crate::numeric::{InputValue, NumValue};

/// A task with the given id and every other field at its empty/`None` default.
///
/// `file` is `tasks/<id>.typ` and `title` mirrors the id. Tests override only
/// the fields they exercise.
#[must_use]
pub fn bare_task(id: &str) -> Task {
    Task {
        id: id.to_owned(),
        file: format!("tasks/{id}.typ").into(),
        title: id.to_owned(),
        status: None,
        priority: None,
        difficulty: None,
        kind: None,
        assignee: None,
        estimate: BTreeMap::new(),
        links: Vec::new(),
        tags: Vec::new(),
        derived_tags: Vec::new(),
        depth: 0,
        topo: 0,
        blocks: 0,
        due: None,
        start: None,
        reviewed: None,
        recur: None,
        extras: BTreeMap::new(),
        shape_errors: Vec::new(),
        body: String::new(),
    }
}

/// An `ice` framework priority with the given inputs — the shared fixture for
/// framework-priority scenarios (schema serialization, validation).
#[must_use]
pub fn ice_priority(impact: f64, confidence: f64, ease: f64) -> NumValue {
    NumValue::Framework {
        name: "ice".to_owned(),
        inputs: BTreeMap::from([
            ("impact".to_owned(), InputValue::Num(impact)),
            ("confidence".to_owned(), InputValue::Num(confidence)),
            ("ease".to_owned(), InputValue::Num(ease)),
        ]),
    }
}

/// Write an in-tree `lib/` mindtape package override under `dir` (its
/// `typst.toml` plus `prelude.typ` from `prelude`), so `@local/mindtape:0.2.0`
/// resolves in a temp project without a global install. Callers pass the real
/// prelude via `include_str!("../../../lib/prelude.typ")`. Shared by the emitter
/// round-trip tests and the CLI flip tests.
///
/// # Panics
/// Panics if writing the package files under `dir` fails — a test-only helper,
/// so an IO failure against a fresh tempdir is a test-harness bug, not a case
/// to recover from.
#[allow(clippy::unwrap_used)]
pub fn write_lib_package(dir: &Path, prelude: &str) {
    let lib = dir.join("lib");
    std::fs::create_dir_all(&lib).unwrap();
    std::fs::write(
        lib.join("typst.toml"),
        "[package]\nname = \"mindtape\"\nversion = \"0.2.0\"\nentrypoint = \"prelude.typ\"\n",
    )
    .unwrap();
    std::fs::write(lib.join("prelude.typ"), prelude).unwrap();
}

/// Write the barebones project scaffold under `dir`: a `.mindtape/config.toml`
/// declaring a single `tasks` root, plus the empty root itself. The prologue
/// every test that needs a real project on disk repeats, and deliberately
/// nothing more — a scenario-specific config overwrites the file, and the lib
/// override ([`write_lib_package`]) is a separate decision, since a test about
/// package resolution wants a project WITHOUT one.
///
/// # Panics
/// Panics if writing under `dir` fails — a test-only helper, so an IO failure
/// against a fresh tempdir is a test-harness bug, not a case to recover from.
#[allow(clippy::unwrap_used)]
pub fn write_bare_project(dir: &Path) {
    std::fs::create_dir_all(dir.join(".mindtape")).unwrap();
    std::fs::write(
        dir.join(".mindtape/config.toml"),
        "[tasks]\nroots = [\"tasks\"]\n",
    )
    .unwrap();
    std::fs::create_dir_all(dir.join("tasks")).unwrap();
}

/// `git init --quiet` in `dir`, asserting success. Shared by every test that
/// drives a real git working copy — the estimate-drift check (`mt check`)
/// reads one, and both the in-crate unit tests and the CLI's e2e tests need
/// the exact same three-line setup to get one.
///
/// # Panics
/// Panics if `git init` fails — a test-harness bug against a fresh tempdir,
/// not a case to recover from.
#[allow(clippy::unwrap_used)]
pub fn git_init(dir: &Path) {
    assert!(
        Command::new("git")
            .args(["init", "--quiet"])
            .current_dir(dir)
            .status()
            .unwrap()
            .success(),
        "git init must succeed in a fresh tempdir"
    );
}

/// A `links:` edge of `kind` to `target`, with no note — the common shape test
/// fixtures need for readiness/cycle/resolution scenarios.
#[must_use]
pub fn link_edge(target: &str, kind: &str) -> Link {
    Link {
        task: target.to_owned(),
        kind: kind.to_owned(),
        note: None,
    }
}

/// A `depends-on` edge to `target` — the most common edge kind in fixtures.
#[must_use]
pub fn dep_edge(target: &str) -> Link {
    link_edge(target, "depends-on")
}

/// A `links:` edge of `kind` to `target` carrying `note` (e.g. the `related`
/// escape hatch, which requires one).
#[must_use]
pub fn noted_edge(target: &str, kind: &str, note: &str) -> Link {
    Link {
        note: Some(note.to_owned()),
        ..link_edge(target, kind)
    }
}

/// A canonical `mindtape.task` Typst metadata source: one `wip` task with a
/// title and priority. Loader and CLI command tests write this to disk.
pub const TASK_SRC: &str = "#metadata((
  marker: \"mindtape.task\",
  fields: (
    title: \"Task X\",
    status: (name: \"wip\", date: datetime(year: 2026, month: 6, day: 1), note: none),
    priority: 8,
  ),
  extras: (:),
))
";

/// The minimal legend any dependency-graph fixture needs: an open `wip` and a
/// terminal `done`, with `extra` appended for a project that also declares,
/// say, a custom blocking link kind. Shared by `ready_tests.rs` and
/// `dep_rank_tests.rs` — the one-hop and transitive readers of the same edges,
/// which must be exercised against the same project shape to be comparable.
///
/// # Panics
/// Panics if `extra` does not parse — a test-only helper, so a malformed
/// fixture is a test bug rather than a case to recover from.
#[must_use]
#[allow(clippy::unwrap_used)]
pub fn dep_config(extra: &str) -> Config {
    crate::config::parse_config(&format!(
        "[[tasks.status]]\nvalue = \"wip\"\n\
         [[tasks.status]]\nvalue = \"done\"\nterminal = true\n{extra}"
    ))
    .unwrap()
}

/// A task with `status` stamped and one `depends-on` edge per entry in `deps`
/// (each a raw target as authored, e.g. `"other.typ"`) — the fixture shape the
/// readiness and graph-rank tests are written in.
#[must_use]
pub fn dep_task(id: &str, status: &str, deps: &[&str]) -> Task {
    let mut task = bare_task(id);
    task.status = Some(Status {
        name: status.to_owned(),
        date: None,
        note: None,
    });
    task.links = deps.iter().map(|dep| dep_edge(dep)).collect();
    task
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The helper's contract: `@local/mindtape` resolves from the written
    /// directory, and the prelude lands verbatim. The manifest is asserted by
    /// the fields a Typst package resolver actually reads, not byte-for-byte —
    /// a test helper's exact formatting is not a behaviour worth pinning.
    #[test]
    fn write_lib_package_writes_a_resolvable_package() {
        let dir = tempfile::tempdir().expect("tempdir creation must succeed");
        let prelude = "#let task(body, ..meta) = body\n";
        write_lib_package(dir.path(), prelude);

        let manifest = std::fs::read_to_string(dir.path().join("lib").join("typst.toml"))
            .expect("typst.toml must be readable");
        for field in [
            "name = \"mindtape\"",
            "entrypoint = \"prelude.typ\"",
            "[package]",
        ] {
            assert!(manifest.contains(field), "manifest must declare {field}");
        }

        let written = std::fs::read_to_string(dir.path().join("lib").join("prelude.typ"))
            .expect("prelude.typ must be readable");
        assert_eq!(written, prelude, "the prelude must land verbatim");
    }
}
