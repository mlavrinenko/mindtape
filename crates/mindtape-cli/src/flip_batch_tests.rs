//! Unit tests for the post-write revalidation's positive branches: a reloaded
//! problem the landed batch owns.
//!
//! The batch's happy paths live in `flip_tests.rs` and `link_cmd_tests.rs`;
//! here are the two decisions `revalidate` can only make against a project
//! built for each — a landed file that failed to reload, and a landed task
//! that now violates a fatal rule. Both are constructed directly because
//! neither can be reached through `run_with`: the pre-write check refuses each
//! before the file lands (`crate::pre_write`), leaving this net as the
//! backstop for a reload that disagrees with it.
#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use super::{Landed, revalidate};
use crate::test_util::make_project_with_lib;
use crate::util::Project;

const PRELUDE: &str = include_str!("../../../lib/prelude.typ");

const LEGEND: &str = "[[tasks.status]]\nvalue = \"wip\"\n\
     [[tasks.status]]\nvalue = \"done\"\nterminal = true\n";

/// A graph where `wip` → `done` only, with `blocked` declared but unreachable,
/// so a task authored `blocked` reloads with a fatal violation.
const UNREACHABLE_LEGEND: &str = "[[tasks.status]]\nvalue = \"wip\"\nto = [\"done\"]\n\
     [[tasks.status]]\nvalue = \"done\"\nterminal = true\n\
     [[tasks.status]]\nvalue = \"blocked\"\n";

const BLOCKED: &str = "#import \"@local/mindtape:0.2.0\": *\n\
#show: task.with(\n  title: \"Blocked\",\n  status: blocked(2026, 6, 1),\n)\n\n== Summary\n\nBody.\n";

/// A file that fails to evaluate, so the reload reports it as a load error.
const BROKEN: &str = "#import \"@local/mindtape:0.2.0\": *\n#panic(\"broken\")\n";

fn project(dir: &Path, files: &[(&str, &str)], config: &str) -> Project {
    make_project_with_lib(dir, PRELUDE, files, config)
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
        order: ids.iter().map(|id| (*id).to_owned()).collect(),
    }
}

#[test]
fn a_landed_file_that_fails_to_reload_is_reported() {
    let dir = tempfile::tempdir().unwrap();
    let project = project(dir.path(), &[("broken.typ", BROKEN)], LEGEND);

    let mut problems = Vec::new();
    revalidate(
        &project,
        &landed(&["broken"], &["tasks/broken.typ"]),
        &mut problems,
    );

    assert_eq!(problems.len(), 1, "{problems:?}");
    assert!(
        problems[0].contains("failed to load after flip"),
        "{problems:?}"
    );
    assert!(problems[0].contains("broken.typ"), "{problems:?}");
}

#[test]
fn a_landed_task_with_a_fatal_violation_is_reported() {
    let dir = tempfile::tempdir().unwrap();
    let project = project(dir.path(), &[("blocked.typ", BLOCKED)], UNREACHABLE_LEGEND);

    let mut problems = Vec::new();
    revalidate(&project, &landed(&["blocked"], &[]), &mut problems);

    assert_eq!(problems.len(), 1, "{problems:?}");
    assert!(problems[0].contains("blocked"), "{problems:?}");
}
