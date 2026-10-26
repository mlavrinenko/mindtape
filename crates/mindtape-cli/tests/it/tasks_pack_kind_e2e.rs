//! The shipped `mindtape:tasks` pack declares the `kind` field and the
//! comment policy its own rules read, so a project that extends it alone
//! needs no `[tasks.fields.kind]` or `[tasks.comments]` of its own.
#![allow(clippy::unwrap_used)]

use mindtape::test_support::write_file;

use crate::support;

/// `mt check` over one open task whose front matter carries `extra` and whose
/// source ends with `tail`, under the pack alone.
fn check(extra: &str, tail: &str) -> (bool, String) {
    let dir =
        support::scaffold_project("extends = [\"mindtape:tasks\"]\n[tasks]\nroots = [\"tasks\"]\n");
    write_file(
        dir.path(),
        "tasks/board.typ",
        format!(
            "#import \"@local/mindtape:0.2.0\": *\n\n\
             #show: task.with(\n  title: \"Cover the board\",\n  status: wip(2026, 6, 1),\n  \
             {extra}\n)\n\n\
             == Problem\n\n+ Run `mt board`.\n+ 🔴 It lists nothing.\n\n\
             == TO-BE\n\n- 🟢 It lists the tasks.\n\n\
             == Changes\n\n- `src/board.rs`: list the tasks.\n\n\
             == Out of scope\n\n- The other views.\n{tail}"
        ),
    );
    support::run_text(dir.path(), &["check"])
}

/// `mt check` must refuse the task and print `needle`.
fn refused(extra: &str, tail: &str, needle: &str) {
    let (ok, text) = check(extra, tail);
    assert!(!ok && text.contains(needle), "{text}");
}

#[test]
fn a_kind_the_pack_lists_passes() {
    let (ok, text) = check("kind: \"refactor\",", "");
    assert!(ok, "{text}");
}

#[test]
fn a_kind_the_pack_does_not_list_is_refused() {
    refused("kind: \"chore\",", "", "`chore` is not an allowed value");
}

#[test]
fn an_open_task_without_a_kind_is_refused() {
    refused("", "", "kind");
}

#[test]
fn a_comment_in_a_task_is_refused() {
    refused("kind: \"refactor\",", "\n// a note to self\n", "comment");
}
