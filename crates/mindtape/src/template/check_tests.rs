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
        location: Location::at("tasks/t.typ", line),
        severity: Severity::Error,
        kind: crate::validate::violation::ViolationKind::Other,
        reads: None,
        unknown: None,
    }
}

fn sources(checked: &str) -> TemplateSources<'_> {
    TemplateSources {
        home: TemplateHome::File(Path::new(".mindtape/templates/bug.typ")),
        authored: TEMPLATE,
        checked,
    }
}

#[test]
fn every_slot_with_neither_example_nor_default_is_a_breach_at_its_line() {
    let found = unfillable_slots(
        TemplateHome::File(Path::new("t.typ")),
        TEMPLATE,
        TEMPLATE,
        &[],
    );
    let at: Vec<(usize, bool)> = (found.iter())
        .map(|v| {
            (
                v.location.line(),
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
        Location::at(".mindtape/templates/bug.typ", 4)
    );
}

#[test]
fn a_breach_off_any_argument_is_placed_by_its_line_text() {
    let checked = "#show: task.with(\n  status: todo(2026, 1, 1),\n)\n\n== Problem\n";
    let kept = template_breaches(sources(checked), vec![breach_at("status", 5)]);
    assert_eq!(kept.first().unwrap().location.line(), 7);
    let kept = template_breaches(sources(checked), vec![breach_at("status", 2)]);
    assert_eq!(
        kept.first().unwrap().location.line(),
        1,
        "falls back to task.with"
    );
}

#[test]
fn a_failure_is_placed_at_task_with() {
    let failure = template_failure(
        TemplateHome::File(Path::new("t.typ")),
        "// x\n#show: task.with()\n",
        "boom".into(),
    );
    assert_eq!(failure.location.line(), 2);
    assert_eq!(failure.field, "template");
}

/// A built-in template has no file: its breaches name it, not a path.
#[test]
fn a_built_in_template_breach_names_the_template_not_a_path() {
    let sources = TemplateSources {
        home: TemplateHome::Builtin("task"),
        ..sources("")
    };
    let kept = template_breaches(sources, vec![breach_at("tags", 30)]);
    assert_eq!(
        kept.first().unwrap().location,
        Location::Builtin {
            template: "task".to_owned(),
            line: 4,
        }
    );
}
