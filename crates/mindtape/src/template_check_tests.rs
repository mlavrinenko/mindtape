#![allow(clippy::unwrap_used)]

use std::path::Path;

use super::*;

const TEMPLATE: &str = "#show: task.with(\n  title: slot(\"title\", example: \"T\"),\n  \
     kind: slot(\"kind\"),\n  tags: (\"a\",),\n)\n\n== Problem\n\n#slot(\"body\", kind: \"text\")\n";

fn breach_at(field: &str, line: usize) -> Violation {
    Violation {
        task_id: "t".to_owned(),
        field: field.to_owned(),
        message: "wrong".to_owned(),
        location: Location {
            file: "tasks/t.typ".into(),
            line,
        },
        severity: Severity::Error,
    }
}

fn sources(checked: &str) -> TemplateSources<'_> {
    TemplateSources {
        file: Path::new(".mindtape/templates/bug.typ"),
        authored: TEMPLATE,
        checked,
    }
}

#[test]
fn every_slot_with_neither_example_nor_default_is_a_breach_at_its_line() {
    let found = unfillable_slots(Path::new("t.typ"), TEMPLATE, TEMPLATE);
    let at: Vec<(usize, bool)> = (found.iter())
        .map(|v| {
            (
                v.location.line,
                v.message.contains("slot `kind`") || v.message.contains("slot `body`"),
            )
        })
        .collect();
    assert_eq!(at, vec![(3, true), (9, true)], "{found:?}");
}

#[test]
fn breaches_move_onto_the_template_s_argument_line() {
    let kept = template_breaches(sources(""), vec![breach_at("tags", 30)]);
    assert_eq!(
        kept.first().unwrap().location,
        Location {
            file: ".mindtape/templates/bug.typ".into(),
            line: 4,
        }
    );
}

#[test]
fn a_breach_off_any_argument_is_placed_by_its_line_text() {
    let checked = "#show: task.with(\n  status: todo(2026, 1, 1),\n)\n\n== Problem\n";
    let kept = template_breaches(sources(checked), vec![breach_at("status", 5)]);
    assert_eq!(kept.first().unwrap().location.line, 7);
    let kept = template_breaches(sources(checked), vec![breach_at("status", 2)]);
    assert_eq!(
        kept.first().unwrap().location.line,
        1,
        "falls back to task.with"
    );
}

#[test]
fn a_failure_is_placed_at_task_with() {
    let failure = template_failure(
        Path::new("t.typ"),
        "// x\n#show: task.with()\n",
        "boom".into(),
    );
    assert_eq!(failure.location.line, 2);
    assert_eq!(failure.field, "template");
}
