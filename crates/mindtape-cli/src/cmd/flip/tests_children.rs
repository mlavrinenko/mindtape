//! A status that is both `unblocked` and `terminal` needs finished children:
//! `mt flip` refuses to enter it while a child is unfinished, `--force`
//! overrides.
#![allow(clippy::unwrap_used)]

use super::run_with;
use super::tests::{flip_args, read, setup};

const CONFIG: &str = "[[tasks.status]]\nvalue = \"proposed\"\n\
     [[tasks.status]]\nvalue = \"wip\"\nunblocked = true\n\
     [[tasks.status]]\nvalue = \"done\"\nterminal = true\nunblocked = true\n\
     [[tasks.status]]\nvalue = \"cancelled\"\nterminal = true\n";

const EPIC: &str = "#import \"@local/mindtape:0.2.0\": *\n\
#show: task.with(\n  title: \"Epic\",\n  status: proposed(2026, 6, 1),\n)\n";

const CHILD: &str = "#import \"@local/mindtape:0.2.0\": *\n\
#show: task.with(\n  title: \"Child\",\n  status: proposed(2026, 6, 1),\n  \
links: parent(\"epic.typ\"),\n)\n";

fn project() -> (tempfile::TempDir, crate::util::Project) {
    let (dir, project) = setup(&[("epic.typ", EPIC), ("child.typ", CHILD)], CONFIG);
    assert_eq!(project.outcome.tasks.len(), 2, "fixture must load");
    (dir, project)
}

#[test]
fn refusal_names_the_status_and_every_unfinished_child() {
    let (_dir, project) = project();
    let world = project.outcome.tasks.clone();
    let epic = world.iter().find(|task| task.id == "epic").unwrap();
    let legend = &project.config.tasks.status;
    let err = super::check_children(&world, epic, "done", &project.config).unwrap_err();
    assert_eq!(
        crate::diag::plain(&err),
        "error: status requires finished children: 'done'\n\
         hint: or pass --force to override\n\
         note: unfinished: child (proposed)\n"
    );
    assert!(legend.requires_finished_children("done"));
    for status in ["wip", "cancelled"] {
        assert!(super::check_children(&world, epic, status, &project.config).is_ok());
    }
}

#[test]
fn flip_refuses_closing_a_parent_over_an_unfinished_child() {
    let (dir, project) = project();
    assert!(run_with(&project, &flip_args("done", &["epic"])).is_err());
    assert!(read(dir.path(), "epic.typ").contains("status: proposed("));
}

#[test]
fn a_parent_starts_and_cancels_over_open_children() {
    let (dir, project) = project();
    run_with(&project, &flip_args("wip", &["epic"])).unwrap();
    assert!(read(dir.path(), "epic.typ").contains("status: wip("));
    run_with(&project, &flip_args("cancelled", &["epic"])).unwrap();
    assert!(read(dir.path(), "epic.typ").contains("status: cancelled("));
}

#[test]
fn force_overrides_the_refusal() {
    let (dir, project) = project();
    let mut args = flip_args("done", &["epic"]);
    args.force = true;
    run_with(&project, &args).unwrap();
    assert!(read(dir.path(), "epic.typ").contains("status: done("));
}

#[test]
fn closing_the_children_first_in_the_batch_lets_the_parent_close() {
    let (dir, project) = project();
    run_with(&project, &flip_args("done", &["child", "epic"])).unwrap();
    assert!(read(dir.path(), "epic.typ").contains("status: done("));
}
