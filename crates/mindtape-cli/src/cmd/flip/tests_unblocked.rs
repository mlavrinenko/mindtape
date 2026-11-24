//! `unblocked = true` on a legend entry: `mt flip` refuses to enter it while
//! the task waits on an unfinished blocking-kind target, `--force` overrides.
#![allow(clippy::unwrap_used)]

use mindtape::Blocker;

use super::run_with;
use super::tests::{
    UNBLOCKED_CONFIG, assert_batch_flipped, assert_flipped, assert_forced, flip_args, loaded, read,
};

const DEP: &str =
    mindtape::typst_task!("(\n  title: \"Dep\",\n  status: proposed(2026, 6, 1),\n)\n");

const WAITER: &str = mindtape::typst_task!(
    "(\n  title: \"Waiter\",\n  status: proposed(2026, 6, 1),\n  \
links: depends-on(\"dep.typ\"),\n)\n"
);

/// The dependency and the task waiting on it, under [`UNBLOCKED_CONFIG`].
const FILES: &[(&str, &str)] = &[("dep.typ", DEP), ("waiter.typ", WAITER)];

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
    let legend = mindtape::parse_config(UNBLOCKED_CONFIG)
        .unwrap()
        .tasks
        .status;
    let err = super::check_unblocked(&legend, "done", &blockers).unwrap_err();
    crate::diag::assert_plain(
        &err,
        "error: status requires an unblocked task: 'done'\n\
         hint: finish what it waits on first, or pass --force to skip this check\n\
         note: waits on: write-atomic (proposed), link-cycle (wip)\n",
    );
    assert!(super::check_unblocked(&legend, "cancelled", &blockers).is_ok());
    assert!(super::check_unblocked(&legend, "done", &[]).is_ok());
}

#[test]
fn flip_refuses_entering_an_unblocked_status_while_blocked() {
    let (dir, project) = loaded(FILES, UNBLOCKED_CONFIG);
    for status in ["wip", "done"] {
        assert!(run_with(&project, &flip_args(status, &["waiter"])).is_err());
        assert!(read(dir.path(), "waiter.typ").contains("status: proposed("));
    }
}

#[test]
fn flip_allows_a_status_without_the_flag_while_blocked() {
    assert_flipped(FILES, UNBLOCKED_CONFIG, "cancelled", "waiter");
}

#[test]
fn force_overrides_the_refusal() {
    assert_forced(FILES, UNBLOCKED_CONFIG, "done", "waiter");
}

#[test]
fn an_earlier_flip_in_the_batch_unblocks_a_later_target() {
    assert_batch_flipped(
        FILES,
        UNBLOCKED_CONFIG,
        "done",
        &["dep", "waiter"],
        "waiter",
    );
}
