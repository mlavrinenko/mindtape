//! The body-marker half of the candidate seam. A candidate harvest supplies
//! both the `mindtape.task` marker and every other marker its body emits, so a
//! `check` rule over `field = "markers"` sees what a load of the written file
//! would. This module sits apart from `validate/candidate_tests.rs`, which was
//! near the linecop cap, along the marker-versus-shape seam. The shared
//! `marker`/`setup` helpers stay in the parent test module.
#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use std::path::Path;

use super::tests::{marker, setup};
use super::{BodyRules, Candidate, check_candidate, harvest_candidate};
use crate::test_support::{project_with_file, write_file};

/// A project whose one rule reads the body markers a task emits and refuses a
/// body with none. It is the rule `mt add` runs before it writes.
const MARKERS_RULE: &str = "\
[tasks]
roots = [\"tasks\"]

[[tasks.rules]]
name = \"goal-marker\"
when = \"!terminal\"
check = { typst = \"check.typ\", function = \"check-markers\", field = \"markers\" }
message = \"rate a goal in the body\"
severity = \"error\"
";

/// The rule calls a function that expects exactly one labelled marker.
const CHECK_MARKERS: &str = "\
#let check-markers(markers) = {
  if markers.len() != 1 { return (\"one marker expected: \" + repr(markers),) }
  if markers.at(0).label != \"goals\" { return (\"label: \" + repr(markers),) }
  ()
}
";

/// A task marker source with one labelled body marker after it.
fn with_goals(status: &str) -> String {
    format!(
        "{}\n#metadata((goal: \"implementable-tasks\", verdict: \"helps\")) <goals>\n",
        marker(status)
    )
}

/// One harvest gives the candidate both the marker that makes it a task and
/// the markers its body emits. A rule dressed as `check-goals` sees them before
/// the write, and a body without the marker is still refused. The door passed
/// an empty list before this, and refused an add the load accepted.
#[test]
fn a_candidate_sees_the_body_markers_its_source_emits() {
    let dir = project_with_file(setup(), ".mindtape/config.toml", MARKERS_RULE);
    write_file(dir.path(), "check.typ", CHECK_MARKERS);
    let config = crate::load_config(dir.path()).unwrap();
    let source = with_goals("wip");
    let candidate = Candidate {
        rel: Path::new("tasks/build.typ"),
        source: &source,
    };

    let task = harvest_candidate(dir.path(), &config, candidate).unwrap();
    assert_eq!(task.markers.len(), 1, "{:?}", task.markers);
    assert_eq!(task.markers[0].label.as_deref(), Some("goals"));

    let violations =
        check_candidate(dir.path(), &config, &[], candidate, BodyRules::default()).unwrap();
    assert!(violations.is_empty(), "{violations:?}");

    let bare = marker("wip");
    let refused = check_candidate(
        dir.path(),
        &config,
        &[],
        Candidate {
            rel: candidate.rel,
            source: &bare,
        },
        BodyRules::default(),
    )
    .unwrap();
    assert_eq!(refused.len(), 1, "{refused:?}");
    assert_eq!(refused[0].field, "rules.goal-marker");
}
