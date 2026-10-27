#![allow(clippy::unwrap_used, clippy::indexing_slicing)]
//! `ask_with`: which plan's lines the prompt shows, and what each answer does.

use std::cell::RefCell;

use super::super::{Removal, RetirePlan, ask_with};
use super::{delete_plan, ids, task};
use mindtape::test_support::tempdir_with_files;

/// A delete plan for one task, on disk under a temp root.
fn delete_removal() -> (tempfile::TempDir, Removal) {
    let dir = tempdir_with_files(&[("gone.typ", "gone\n")]);
    let tasks = vec![task("gone", "gone.typ", Some("done"))];
    let plan = delete_plan(&tasks, &ids(&["gone"]), dir.path());
    (dir, Removal::Delete(plan))
}

fn retire_removal() -> Removal {
    Removal::Retire(RetirePlan {
        status: "done".to_owned(),
        rename: None,
        rewrites: vec![],
        link_rewrites: vec![],
        deletions: vec![],
    })
}

/// Run `ask_with` on a terminal, answering `answer`; return the result and the
/// lines the prompt showed.
fn ask(removal: &Removal, answer: bool) -> (crate::diag::Result<()>, Vec<String>) {
    let shown = RefCell::new(Vec::new());
    let result = ask_with(removal, true, |lines| {
        shown.borrow_mut().extend_from_slice(lines);
        Ok(answer)
    });
    (result, shown.into_inner())
}

#[test]
fn a_delete_plan_shows_the_delete_prompt_and_a_yes_goes_on() {
    let (_dir, removal) = delete_removal();
    let (result, shown) = ask(&removal, true);
    assert!(result.is_ok());
    assert_eq!(
        shown,
        ["rm will delete 1 task:", "  - gone.typ"].map(str::to_owned)
    );
}

#[test]
fn a_retire_plan_shows_the_retire_prompt() {
    let (result, shown) = ask(&retire_removal(), true);
    assert!(result.is_ok());
    assert_eq!(shown, ["rm will retire 0 tasks to done:".to_owned()]);
}

#[test]
fn a_no_aborts_the_run() {
    let (_dir, removal) = delete_removal();
    let (result, _) = ask(&removal, false);
    assert_eq!(
        crate::diag::plain(&result.unwrap_err()),
        "error: aborted by user\n"
    );
}

#[test]
fn without_a_terminal_nothing_is_asked() {
    let asked = RefCell::new(false);
    let result = ask_with(&retire_removal(), false, |_| {
        *asked.borrow_mut() = true;
        Ok(true)
    });
    assert!(result.is_err());
    assert!(!asked.into_inner());
}
