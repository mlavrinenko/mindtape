//! `mt templates`'s kind reading and text rendering.
#![allow(clippy::unwrap_used)]

use mindtape::test_support::{tempdir, write_file};

use super::{Template, collect, kind_of, render, template_json};

fn template(name: &str, kind: &str, slots: usize, default: bool) -> Template {
    Template {
        name: name.to_owned(),
        kind: kind.to_owned(),
        slots,
        default,
    }
}

/// A literal `kind:` is the kind the template sets. A `kind:` slot, an absent
/// `kind:` and a file with no `task.with(…)` call all read `any`.
#[test]
fn a_literal_kind_is_the_kind_and_everything_else_reads_any() {
    for (source, want) in [
        (
            "#show: task.with(title: slot(\"title\"), kind: \"dev\")\n",
            "dev",
        ),
        (
            "#show: task.with(kind: slot(\"kind\", example: \"test\"))\n",
            "any",
        ),
        ("#show: task.with(title: \"x\")\n", "any"),
        ("not a task file", "any"),
    ] {
        assert_eq!(kind_of(source), want, "{source}");
    }
}

/// A row gives the template, the `kind` it sets and its slot count, and one
/// slot reads singular.
#[test]
fn a_row_names_the_template_its_kind_and_its_slots() {
    for (row, want) in [
        (
            template("dev", "dev", 12, false),
            "dev  kind: dev  12 slots\n",
        ),
        (
            template("task", "any", 1, false),
            "task  kind: any  1 slot\n",
        ),
    ] {
        assert_eq!(render(&[row]), want);
    }
}

#[test]
fn the_default_template_carries_the_marker_after_the_widest_row() {
    let text = render(&[
        template("decision", "decision", 8, false),
        template("task", "any", 12, true),
    ]);
    assert_eq!(
        text,
        "decision  kind: decision  8 slots\ntask      kind: any       12 slots   default\n"
    );
}

#[test]
fn no_templates_says_so() {
    assert_eq!(render(&[]), "no templates\n");
}

#[test]
fn json_carries_the_same_row() {
    let value = serde_json::to_value(template_json(&template("task", "any", 12, true))).unwrap();
    assert_eq!(
        value,
        serde_json::json!({"name": "task", "kind": "any", "slots": 12, "default": true})
    );
}

#[test]
fn collect_reads_the_builtin_template_and_the_configured_directory() {
    let dir = tempdir();
    write_file(
        dir.path(),
        ".mindtape/templates/dev.typ",
        "#show: task.with(title: slot(\"title\"), kind: \"dev\")\n",
    );
    let config = mindtape::config::parse_config("[templates]\ndefault = \"dev\"\n").unwrap();

    let listed = collect(&config, dir.path()).unwrap();
    assert_eq!(
        listed,
        [
            template("dev", "dev", 1, true),
            template("task", "any", 1, false)
        ]
    );
}
