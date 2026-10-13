//! Unit test for `land`'s destination-creation guard: the destination is
//! created when the first write actually needs it, so a batch whose records
//! all fail leaves not even an empty directory behind.
#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use mindtape::ImportReport;
use mindtape::model::Status;
use mindtape::test_support::bare_task;
use mindtape::test_support::tempdir;

use super::{StagedTask, land};
use crate::cmd::import::tests::setup_dest;

/// A clean record must create its destination directory before writing into
/// it — the import destination need not pre-exist.
#[test]
fn land_creates_a_destination_that_did_not_exist_yet() {
    let dir = tempdir();
    let project = setup_dest(dir.path());

    let mut task = bare_task("solo");
    task.title = "Solo Task".to_owned();
    task.status = Some(Status {
        name: "wip".to_owned(),
        date: Some("2026-06-01".to_owned()),
        note: None,
    });
    let source = mindtape::emit_task(&task);

    let dest = dir.path().join("tasks/fresh");
    assert!(!dest.exists(), "the destination must not pre-exist");

    let staged = vec![StagedTask {
        label: "solo".to_owned(),
        dest: dest.clone(),
        stem: "solo".to_owned(),
        source,
        report: ImportReport::default(),
    }];
    let outcomes = land(&project, staged).unwrap();

    assert!(outcomes[0].filed, "{:?}", outcomes[0].problems);
    assert!(dest.join("solo.typ").exists(), "the destination is created");
}
