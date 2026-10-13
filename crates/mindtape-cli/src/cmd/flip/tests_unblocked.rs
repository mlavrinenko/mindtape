//! `unblocked = true` on a legend entry: `mt flip` refuses to enter it while
//! the task waits on an unfinished blocking-kind target, `--force` overrides.
#![allow(clippy::unwrap_used)]

use mindtape::Blocker;

use super::run_with;
use super::tests::{flip_args, read, setup};

const CONFIG: &str = "[[tasks.status]]\nvalue = \"proposed\"\n\
     [[tasks.status]]\nvalue = \"wip\"\nunblocked = true\n\
     [[tasks.status]]\nvalue = \"done\"\nterminal = true\nunblocked = true\n\
     [[tasks.status]]\nvalue = \"cancelled\"\nterminal = true\n";

const DEP: &str =
    mindtape::typst_task!("(\n  title: \"Dep\",\n  status: proposed(2026, 6, 1),\n)\n");

const WAITER: &str = mindtape::typst_task!(
    "(\n  title: \"Waiter\",\n  status: proposed(2026, 6, 1),\n  \
links: depends-on(\"dep.typ\"),\n)\n"
);

fn project() -> (tempfile::TempDir, crate::util::Project) {
    let (dir, project) = setup(&[("dep.typ", DEP), ("waiter.typ", WAITER)], CONFIG);
    assert_eq!(project.outcome.tasks.len(), 2, "fixture must load");
    (dir, project)
}

#[test]
fn refusal_names_the_status_and_every_blocker() {
    let blockers = [
        Blocker {
            stem: "write-atomic".to_owned(),
            status: Some("proposed".to_owned()),
            kind: "depends-on".to_owned(),
        },
        Blocker {
            stem: "link-cycle".to_owned(),
            status: Some("wip".to_owned()),
            kind: "depends-on".to_owned(),
        },
    ];
    let legend = mindtape::parse_config(CONFIG).unwrap().tasks.status;
    let err = super::check_unblocked(&legend, "done", &blockers).unwrap_err();
    assert_eq!(
        crate::diag::plain(&err),
        "error: status requires an unblocked task: 'done'\n\
         hint: or pass --force to override\n\
         note: waits on: write-atomic (proposed), link-cycle (wip)\n"
    );
    assert!(super::check_unblocked(&legend, "cancelled", &blockers).is_ok());
    assert!(super::check_unblocked(&legend, "done", &[]).is_ok());
}

#[test]
fn flip_refuses_entering_an_unblocked_status_while_blocked() {
    let (dir, project) = project();
    for status in ["wip", "done"] {
        assert!(run_with(&project, &flip_args(status, &["waiter"])).is_err());
        assert!(read(dir.path(), "waiter.typ").contains("status: proposed("));
    }
}

#[test]
fn flip_allows_a_status_without_the_flag_while_blocked() {
    let (dir, project) = project();
    run_with(&project, &flip_args("cancelled", &["waiter"])).unwrap();
    assert!(read(dir.path(), "waiter.typ").contains("status: cancelled("));
}

#[test]
fn force_overrides_the_refusal() {
    let (dir, project) = project();
    let mut args = flip_args("done", &["waiter"]);
    args.force = true;
    run_with(&project, &args).unwrap();
    assert!(read(dir.path(), "waiter.typ").contains("status: done("));
}

#[test]
fn an_earlier_flip_in_the_batch_unblocks_a_later_target() {
    let (dir, project) = project();
    run_with(&project, &flip_args("done", &["dep", "waiter"])).unwrap();
    assert!(read(dir.path(), "waiter.typ").contains("status: done("));
}
