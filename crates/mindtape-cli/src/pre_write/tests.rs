//! The pre-write seam's own tests: what it accepts, what it refuses, and the
//! two properties the writers on top of it depend on — that a batch is one
//! question, and that a refusal touches nothing.
#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use mindtape::test_support::tempdir;
use std::path::Path;

use mindtape::parse_config;

use super::{Pending, absorb, check, ensure, verify_one};
use crate::test_util::make_project;
use crate::util::Project;

/// A task marker with the given status and `depends-on` edges, written as a
/// bare `#metadata(...)` so no `@local/mindtape` package has to be installed.
fn marker(status: &str, deps: &[&str]) -> String {
    let links: String = deps
        .iter()
        .map(|dep| format!("(task: {dep:?}, kind: \"depends-on\", note: none),"))
        .collect();
    format!(
        "#metadata((\n  marker: \"mindtape.task\",\n  fields: (\n    title: \"T\",\n    \
         status: (name: {status:?}, date: none, note: none),\n    links: ({links}),\n  ),\n  \
         extras: (:),\n))\n"
    )
}

/// A project under `dir` with `files` already on disk, loaded under the floor
/// config — whose status legend is non-empty, so a bogus status name is
/// actually out of bounds.
fn setup(files: &[(&str, &str)]) -> (tempfile::TempDir, Project) {
    make_project(files, parse_config("").unwrap())
}

/// The whole point, in one assertion: a rewrite that would not validate is
/// refused, and the file it would have replaced is byte-identical afterwards.
#[test]
fn refuses_a_rewrite_without_touching_the_file() {
    let good = marker("wip", &[]);
    let (dir, project) = setup(&[("a.typ", good.as_str())]);
    let file = dir.path().join("tasks/a.typ");
    let bad = marker("bogus", &[]);

    let err = verify_one(&project, &project.outcome.tasks, &file, &bad).unwrap_err();

    let text = crate::diag::plain(&err);
    assert!(text.starts_with("error: tasks/a.typ:"), "{text}");
    assert!(!text.contains("rejected"), "no summary of its own: {text}");
    assert_eq!(std::fs::read_to_string(&file).unwrap(), good);
}

/// An accepted rewrite hands back the task it will load as, so a batch landing
/// file by file can fold it into the world the next entry is checked against.
#[test]
fn accepts_a_rewrite_and_returns_its_task() {
    let (dir, project) = setup(&[("a.typ", marker("wip", &[]).as_str())]);
    let file = dir.path().join("tasks/a.typ");

    let landed = verify_one(
        &project,
        &project.outcome.tasks,
        &file,
        &marker("done", &[]),
    )
    .unwrap()
    .unwrap();

    assert_eq!(landed.id, "a");
    assert_eq!(landed.file, Path::new("tasks/a.typ"));
}

/// A batch is one question, not N: `a` depends on `b`, and `b` exists only as
/// the other candidate in the same batch. Checked together the edge resolves;
/// this is the property `mt link --apply`, `mt import` and `mt recur --advance`
/// each rely on.
#[test]
fn resolves_an_edge_between_two_candidates_in_one_batch() {
    let (dir, project) = setup(&[]);
    let a_file = dir.path().join("tasks/a.typ");
    let b_file = dir.path().join("tasks/b.typ");
    let a_source = marker("wip", &["b.typ"]);
    let b_source = marker("wip", &[]);

    let batch = [
        Pending {
            file: &a_file,
            source: &a_source,
        },
        Pending {
            file: &b_file,
            source: &b_source,
        },
    ];
    let together = check(&project, &project.outcome.tasks, &batch).unwrap();
    assert!(
        together[0].problems().is_empty(),
        "{:?}",
        together[0].problems()
    );

    // The same candidate alone cannot see `b`, which is exactly why the batch
    // is not a loop over this call.
    let alone = check(&project, &project.outcome.tasks, &batch[..1]).unwrap();
    assert!(!alone[0].problems().is_empty());
}

/// `ensure` is all-or-nothing: one rejected entry fails the call, and the
/// error counts the rejections rather than naming only the first.
#[test]
fn ensure_refuses_the_whole_batch_on_one_bad_entry() {
    let (dir, project) = setup(&[]);
    let good_file = dir.path().join("tasks/good.typ");
    let bad_file = dir.path().join("tasks/bad.typ");
    let good = marker("wip", &[]);
    let bad = marker("bogus", &[]);

    let err = ensure(
        &project,
        &[
            Pending {
                file: &good_file,
                source: &good,
            },
            Pending {
                file: &bad_file,
                source: &bad,
            },
        ],
    )
    .unwrap_err();

    let text = crate::diag::plain(&err);
    assert!(
        text.starts_with("error: <new task>:") && text.contains("`bogus`"),
        "{text}"
    );
    assert!(!text.contains("rejected"), "no summary of its own: {text}");
    assert!(!good_file.exists(), "ensure must not write anything");
}

/// A source that does not evaluate is reported as its own problem rather than
/// costing the batch its answer.
#[test]
fn reports_a_source_that_does_not_evaluate() {
    let (dir, project) = setup(&[]);
    let file = dir.path().join("tasks/a.typ");

    let verdicts = check(
        &project,
        &project.outcome.tasks,
        &[Pending {
            file: &file,
            source: "= just prose, no marker\n",
        }],
    )
    .unwrap();

    assert_eq!(verdicts.len(), 1);
    assert!(verdicts[0].task.is_none());
    assert!(verdicts[0].problems()[0].starts_with("did not load: "));
    assert!(verdicts[0].problems()[0].contains("no task marker"));
}

/// A load failure names the line it is at, in the file as it stands on disk, and
/// in a new task when there is no file yet.
#[test]
fn a_load_failure_names_its_line() {
    let good = marker("wip", &[]);
    let (dir, project) = setup(&[("a.typ", good.as_str())]);
    let broken = "= prose\n\n#metadata(\n";
    let on_disk = dir.path().join("tasks/a.typ");
    let absent = dir.path().join("tasks/new.typ");
    let pending = |file| Pending {
        file,
        source: broken,
    };

    let verdicts = check(
        &project,
        &project.outcome.tasks,
        &[pending(&on_disk), pending(&absent)],
    )
    .unwrap();

    let place = |at: usize| verdicts[at].problems()[0].clone();
    assert!(place(0).starts_with("tasks/a.typ:"), "{}", place(0));
    assert!(place(1).starts_with("<new task>:"), "{}", place(1));
    assert!(place(1).contains(": did not load:"), "{}", place(1));
}

/// A destination outside the project root has no project-relative path to
/// evaluate a candidate at. That is reported as *unchecked*, not as clean:
/// `mt add` reads the flag to fall back to its post-write verification.
#[test]
fn marks_a_destination_outside_the_root_unchecked() {
    let elsewhere = tempdir();
    let (_dir, project) = setup(&[]);
    let file = elsewhere.path().join("a.typ");
    let source = marker("bogus", &[]);

    let verdicts = check(
        &project,
        &project.outcome.tasks,
        &[Pending {
            file: &file,
            source: &source,
        }],
    )
    .unwrap();

    assert!(!verdicts[0].checked);
    assert!(verdicts[0].problems().is_empty());
}

/// A misconfigured header rule fails the whole call up front — the same
/// compile error `mt check` reports — rather than silently skipping the rule
/// or letting a candidate land unchecked against it.
#[test]
fn refuses_to_check_against_an_unparseable_header_rule() {
    let config = parse_config(
        "[[tasks.body.headers.rules]]\npath = [{ match = \"(\" }]\naction = \"require\"\n",
    )
    .unwrap();
    let (dir, project) = make_project(&[], config);
    let file = dir.path().join("tasks/a.typ");
    let source = marker("wip", &[]);

    let err = check(
        &project,
        &project.outcome.tasks,
        &[Pending {
            file: &file,
            source: &source,
        }],
    )
    .unwrap_err();

    assert!(
        err.to_string().contains("tasks.body.headers.rules"),
        "{err}"
    );
}

/// `absorb` replaces by path, so folding a flipped task in does not leave the
/// pre-flip copy beside it — which would read as a duplicate id to the very
/// next check.
#[test]
fn absorb_replaces_the_task_at_the_same_path() {
    let mut world = vec![mindtape::test_support::bare_task("a")];
    let mut updated = mindtape::test_support::bare_task("a");
    updated.title = "flipped".to_owned();

    absorb(&mut world, updated);

    assert_eq!(world.len(), 1);
    assert_eq!(world[0].title, "flipped");
}
