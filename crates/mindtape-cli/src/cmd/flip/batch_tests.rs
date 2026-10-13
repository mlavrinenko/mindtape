//! Unit tests for the post-write revalidation's positive branches: a reloaded
//! problem the landed batch owns.
//!
//! The batch's happy paths live in `cmd/flip/tests.rs` and `cmd/link/tests.rs`;
//! here are the two decisions `revalidate` can only make against a project
//! built for each — a landed file that failed to reload, and a landed task
//! that now violates a fatal rule. Both are constructed directly because
//! neither can be reached through `run_with`: the pre-write check refuses each
//! before the file lands (`crate::pre_write`), leaving this net as the
//! backstop for a reload that disagrees with it.
#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use std::collections::HashSet;
use std::path::PathBuf;

use super::{Landed, revalidate};
use crate::test_util::make_project_with_lib;
use crate::util::Project;

use mindtape::test_support::PRELUDE;

const LEGEND: &str = "[[tasks.status]]\nvalue = \"wip\"\n\
     [[tasks.status]]\nvalue = \"done\"\nterminal = true\n";

/// A graph where `wip` → `done` only, with `blocked` declared but unreachable,
/// so a task authored `blocked` reloads with a fatal violation.
const UNREACHABLE_LEGEND: &str = "[[tasks.status]]\nvalue = \"wip\"\nto = [\"done\"]\n\
     [[tasks.status]]\nvalue = \"done\"\nterminal = true\n\
     [[tasks.status]]\nvalue = \"blocked\"\n";

const BLOCKED: &str = mindtape::typst_task!(
    "\
#show: task.with(\n  title: \"Blocked\",\n  status: blocked(2026, 6, 1),\n)\n\n== Summary\n\nBody.\n"
);

/// A file that fails to evaluate, so the reload reports it as a load error.
const BROKEN: &str = mindtape::typst_task!("#panic(\"broken\")\n");

fn project(files: &[(&str, &str)], config: &str) -> (tempfile::TempDir, Project) {
    make_project_with_lib(PRELUDE, files, config)
}

/// A batch that landed `ids` at `files`. Built directly rather than through
/// [`Landed::record`], which needs a loaded `Task` — a file that failed to
/// reload yields none.
fn landed(ids: &[&str], files: &[&str]) -> Landed {
    Landed {
        ids: ids
            .iter()
            .map(|id| (*id).to_owned())
            .collect::<HashSet<_>>(),
        files: files.iter().map(PathBuf::from).collect(),
    }
}

#[test]
fn a_landed_file_that_fails_to_reload_is_reported() {
    let (_dir, project) = project(&[("broken.typ", BROKEN)], LEGEND);

    let mut problems = Vec::new();
    revalidate(
        &project,
        &landed(&["broken"], &["tasks/broken.typ"]),
        &mut problems,
    );

    assert_eq!(problems.len(), 1, "{problems:?}");
    let text = crate::diag::plain(&problems[0]);
    assert!(text.starts_with("error: tasks/broken.typ"), "{text}");
    assert!(
        text.contains("note: it was written, and no longer loads"),
        "{text}"
    );
}

#[test]
fn a_landed_task_with_a_fatal_violation_is_reported() {
    let (_dir, project) = project(&[("blocked.typ", BLOCKED)], UNREACHABLE_LEGEND);

    let mut problems = Vec::new();
    revalidate(&project, &landed(&["blocked"], &[]), &mut problems);

    assert_eq!(problems.len(), 1, "{problems:?}");
    assert!(
        crate::diag::plain(&problems[0]).contains("blocked"),
        "{problems:?}"
    );
}

/// Whether the batch wrote nothing decides whether the reload is paid for at
/// all: an empty batch is empty, and any recorded task makes it not.
#[test]
fn a_batch_that_recorded_nothing_is_empty() {
    assert!(Landed::default().is_empty());
    assert!(!landed(&["one"], &["tasks/one.typ"]).is_empty());
}
