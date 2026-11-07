//! A listing that selects nothing names the way out: widen the filter to every
//! task, or read the project's views. The hint is table-only, so the machine
//! formats stay streams with no trailing prose.
#![allow(clippy::unwrap_used, clippy::disallowed_methods)]

use crate::support;

const HINT: &str =
    "\nhint: no task matches; run mt ls '' for every task, or mt explain views for a saved view\n";

/// One task, plus a view no filter below can match.
fn project() -> tempfile::TempDir {
    support::scaffold_config(
        "[tasks]\nroots = [\"tasks\"]\n\n[views.empty]\nfilter = 'title == \"zz\"'\n",
        &[("a.typ", mindtape::test_support::TASK_SRC)],
    )
}

/// Every way a table can select nothing from [`project`]: a filter, `find`,
/// and a saved view.
#[test]
fn a_table_that_selects_nothing_names_the_way_out() {
    let dir = project();
    for args in [
        &["ls", "title == \"zz\""][..],
        &["find", "zz"][..],
        &["ls", "@empty"][..],
    ] {
        let text = support::stdout(dir.path(), args);
        assert!(text.contains(HINT), "{args:?}: {text}");
    }
}

/// Rows, a skip and a limit are not an empty filter, and the machine formats
/// carry no prose at all.
#[test]
fn only_an_empty_selection_carries_the_hint() {
    let dir = project();
    for args in [
        &["ls", ""][..],
        &["find", "Task X"][..],
        &["ls", "", "--skip", "1"][..],
        &["ls", "", "-n", "0"][..],
    ] {
        let text = support::stdout(dir.path(), args);
        assert!(!text.contains("no task matches"), "{args:?}: {text}");
    }
    for format in ["json", "csv", "path", "path0"] {
        let text = support::stdout(dir.path(), &["ls", "title == \"zz\"", "--format", format]);
        assert!(!text.contains("hint:"), "{format}: {text}");
    }
}

#[test]
fn the_way_out_keeps_how_the_project_was_named() {
    let here = tempfile::tempdir().unwrap();
    let dir = project();
    let root = dir.path().to_str().unwrap();
    let text = support::stdout(here.path(), &["-w", root, "ls", "title == \"zz\""]);
    assert!(
        text.contains(&format!(
            "hint: no task matches; run mt -w {root} ls '' for every task"
        )),
        "{text}"
    );
}
