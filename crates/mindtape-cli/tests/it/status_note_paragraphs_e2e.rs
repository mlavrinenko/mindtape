//! A multi-paragraph, bulleted status note must survive the `mt flip -m`
//! write and the `mt ls --format json` read as readable prose, not one
//! run-on word with the paragraph break and the list markers silently
//! dropped (`truth/tasks/fix-the-status-note-flattening-on-read.typ`).
#![allow(clippy::unwrap_used, clippy::disallowed_methods)]

use crate::support;

const LEGEND: &str = "[tasks]\nroots = [\"tasks\"]\n\
     [[tasks.status]]\nvalue = \"wip\"\n\
     [[tasks.status]]\nvalue = \"done\"\nterminal = true\n";

const TASK: &str = "#import \"@local/mindtape:0.2.0\": *\n\n\
     #show: task.with(\n  title: \"Alpha\",\n  status: wip(2026, 6, 1),\n)\n\n\
     == Summary\n\nBody.\n";

const NOTE: &str = "First line of the outcome.\n\n\
     Second paragraph explaining the decision.\n\n\
     - a bullet\n- another";

#[test]
fn multi_paragraph_bulleted_note_survives_flip_then_ls_json() {
    let dir = support::scaffold_project(LEGEND);
    std::fs::write(dir.path().join("tasks/alpha.typ"), TASK).unwrap();

    support::run_ok(dir.path(), &["flip", "done", "alpha", "-m", NOTE]);

    let note = support::status_note(dir.path());

    assert!(
        note.contains("outcome. Second paragraph"),
        "paragraph break dropped no separator: {note:?}"
    );
    assert!(
        note.contains("a bullet another"),
        "list items welded together: {note:?}"
    );
}
