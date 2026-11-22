//! A marker-bearing task through the commit door. `mt check --commits <range>`
//! loads each state's tasks through the same candidate path `mt add` uses
//! ([`mindtape::harvest_candidate`]), so a body marker has to survive that
//! evaluation too
//! (`truth/tasks/2026-11/carry-body-markers-through-the-candidate-door.typ`).
//! The task-scope rule here reads the task in both states of the commit, which
//! only answers when each state's marker-bearing file evaluated cleanly.
//!
//! Its own binary rather than more of `commit_range_e2e.rs`, which was near
//! the linecop cap.
#![allow(clippy::unwrap_used, clippy::disallowed_methods)]

use mindtape::test_support::write_file;
use std::path::Path;

use crate::commit_range_e2e::check_range;
use crate::support;

/// The metadata the task body emits, the marker a `field = "markers"` check
/// reads off the working tree.
const MARKER: &str = "#metadata((goal: \"implementable-tasks\", verdict: \"helps\")) <goals>\n";

/// A rule that answers only when the task loaded in both states of the commit:
/// the parent it changes and the tree it leaves.
const BOTH_STATES: &str = "[[commits.rules]]\n\
     name = \"parser-in-both-states\"\n\
     scope = \"task\"\n\
     assert = 'staged(status == \"wip\") && head(status == \"wip\")'\n\
     message = \"load the parser in both commit states\"\n\
     severity = \"error\"\n";

/// Rewrite `tasks/parser.typ` with a body marker after its `task.with` call.
fn write_marked_parser(dir: &Path) {
    write_file(
        dir,
        "tasks/parser.typ",
        format!(
            "#import \"@local/mindtape:{version}\": *\n\n\
             #show: task.with(\n  title: \"Write the parser\",\n  status: wip(2026, 6, 1),\n  \
             priority: 3,\n)\n\n\
             {MARKER}",
            version = mindtape::PRELUDE_VERSION
        ),
    );
}

/// A marker-bearing body loads in both states of a commit, and the range check
/// reports neither a load error nor a warning for it.
#[test]
fn a_range_check_loads_a_marker_bearing_task_in_each_state() {
    let repo = support::commit_repo(BOTH_STATES);
    let dir = repo.path();
    write_marked_parser(dir);
    support::git(dir, &["add", "tasks/parser.typ"]);
    support::commit(dir, "feat: mark the parser body\n\nRefs: parser");

    let (ok, text) = check_range(dir, "HEAD~1..HEAD");

    assert!(ok, "{text}");
    assert!(!text.contains("warning:"), "{text}");
}
