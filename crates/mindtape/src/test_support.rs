//! Shared task fixtures for unit tests.
//!
//! Exposed behind the `test-support` feature so dependent crates (the CLI) can
//! reuse these builders in their own unit tests; within this crate they are
//! available to `#[cfg(test)]` code directly.
//!
//! Built for the `test-support` feature alone (no `cfg(test)`), so the
//! crate-root `cfg_attr(test, ...)` exemption from `disallowed-methods`
//! (`clippy.toml`) does not reach it; it is test fixture code all the same,
//! hence its own module-wide allow.
#![allow(clippy::disallowed_methods)]

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// A task file's source: the `@local/mindtape` import line, then `$body`, a
/// string literal. It expands to one literal, so a `const` can hold it, and
/// a test names the package version only here.
#[macro_export]
macro_rules! typst_task {
    ($body:literal) => {
        concat!("#import \"@local/mindtape:0.2.0\": *\n", $body)
    };
}
use std::process::Command;

use crate::config::Config;
use crate::link::Link;
use crate::model::numeric::{InputValue, NumValue};
use crate::model::{Status, Task};

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
        waits_on: Vec::new(),
        answers: BTreeMap::new(),
        tags: Vec::new(),
        derived_tags: Vec::new(),
        depth: 0,
        topo: 0,
        blocks: 0,
        children: 0,
        children_done: 0,
        due: None,
        start: None,
        reviewed: None,
        recur: None,
        extras: BTreeMap::new(),
        shape_errors: Vec::new(),
        body: String::new(),
        spans: crate::model::task_spans::TaskCallSpans::default(),
    }
}

/// The config an unconfigured project gets: the embedded floor and nothing
/// over it.
///
/// [`Config::default`] is not that — it bypasses `defaults.toml` entirely, so
/// it carries no `[ls.columns]` entries and therefore no column that renders.
/// A test asserting on what `mt ls` shows wants the floor; a test pinning one
/// key in isolation still wants `Config::default()`.
///
/// # Panics
/// Panics only if the compiled-in floor stops parsing, which every other
/// `parse_config` caller would fail on first.
#[must_use]
#[allow(clippy::expect_used)]
pub fn floor_config() -> Config {
    crate::parse_config("").expect("the embedded floor must parse")
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

/// The in-tree `lib/prelude.typ`, read once here so no test spells its own
/// path to it.
pub const PRELUDE: &str = include_str!("../../../lib/prelude.typ");

/// Write an in-tree `lib/` mindtape package override under `dir` (its
/// `typst.toml` plus `prelude.typ` from `prelude`), so `@local/mindtape:0.2.0`
/// resolves in a temp project without a global install. Callers pass the real
/// prelude, [`PRELUDE`]. Shared by the emitter
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

/// A fresh tempdir, removed when the returned guard drops. The setup line
/// nearly every test that touches the disk opens with.
///
/// # Panics
/// Panics if the OS refuses a temp directory, a test-harness bug.
#[allow(clippy::unwrap_used)]
#[must_use]
pub fn tempdir() -> tempfile::TempDir {
    tempfile::tempdir().unwrap()
}

/// A fresh [`tempdir`] with each of `dirs` (relative paths) created in it.
///
/// # Panics
/// Panics if creating a directory fails, a test-harness bug.
#[allow(clippy::unwrap_used)]
#[must_use]
pub fn tempdir_with_dirs(dirs: &[&str]) -> tempfile::TempDir {
    let dir = tempdir();
    for rel in dirs {
        std::fs::create_dir_all(dir.path().join(rel)).unwrap();
    }
    dir
}

/// A fresh [`tempdir`] holding `.mindtape/config.toml` with `config` and
/// nothing else: no lib, no task root.
///
/// # Panics
/// Panics if writing the file fails, a test-harness bug.
#[allow(clippy::unwrap_used)]
#[must_use]
pub fn tempdir_with_config(config: &str) -> tempfile::TempDir {
    let dir = tempdir_with_dirs(&[".mindtape"]);
    std::fs::write(dir.path().join(".mindtape/config.toml"), config).unwrap();
    dir
}

/// A fresh tempdir with an (empty) `home/project` tree already created,
/// `project` carrying an empty `.mindtape/` (no `config.toml`/`defaults.toml`
/// of its own — every caller writes the layer text its scenario needs). Keep
/// the returned `TempDir` alive for as long as the paths are used. Shared by
/// `config/layers_tests.rs` and `config/packs_tests.rs`, which both build a
/// `home`/ancestor/project chain to drive `discover_layers`/`load_config_layered`
/// against.
///
/// # Panics
/// Panics if creating either directory fails — a test-harness bug against a
/// fresh tempdir, not a case to recover from.
///
#[allow(clippy::unwrap_used)]
#[must_use]
pub fn home_and_project() -> (tempfile::TempDir, PathBuf, PathBuf) {
    let tmp = tempdir();
    let home = tmp.path().join("home");
    let project = home.join("project");
    std::fs::create_dir_all(project.join(".mindtape")).unwrap();
    (tmp, home, project)
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
/// drives a real git working copy — `changed()` in the filter language
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

/// Run `git <args>` in `root` as a throwaway identity with signing off,
/// asserting success — the committing half of [`git_init`] for tests that
/// build a history.
///
/// # Panics
/// Panics if git fails.
#[allow(clippy::unwrap_used)]
pub fn git_as_tester(root: &Path, args: &[&str]) {
    let ok = Command::new("git")
        .args(["-c", "user.name=t", "-c", "user.email=t@t"])
        .args(["-c", "commit.gpgsign=false"])
        .args(args)
        .current_dir(root)
        .status()
        .unwrap()
        .success();
    assert!(ok, "git {args:?} must succeed");
}

/// A `links:` edge of `kind` to `target`, with no note — the common shape test
/// fixtures need for readiness/cycle/resolution scenarios.
/// A [`bare_task`] carrying `links`.
#[must_use]
pub fn linked_task(id: &str, links: Vec<Link>) -> Task {
    Task {
        links,
        ..bare_task(id)
    }
}

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
/// say, a custom blocking link kind. Shared by `graph/ready_tests.rs` and
/// `graph/dep_rank_tests.rs` — the one-hop and transitive readers of the same edges,
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

/// Each violation as `(task_id, field, message)` — what a check test asserts.
#[must_use]
pub fn violation_triples(violations: &[crate::validate::Violation]) -> Vec<(&str, &str, &str)> {
    violations
        .iter()
        .map(|v| (v.task_id.as_str(), v.field.as_str(), v.message.as_str()))
        .collect()
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
        let dir = tempdir();
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
