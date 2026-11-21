//! End-to-end tests for `mt templates`: the listing, its JSON shape, and the
//! two spellings it replaced.
#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use mindtape::test_support::write_file;

use crate::support;
use support::{json, run_text, scaffold_project, stdout};

/// `mt templates` lists every template `mt add` accepts, one line each: the
/// name, the `kind` it sets, its slot count, and `default` on the one `mt add`
/// picks. A template dropped into the configured directory appears without a
/// config change.
#[test]
fn lists_each_template_with_its_kind_and_slots() {
    let dir = scaffold_project("[tasks]\nroots = [\"tasks\"]\n");
    write_file(
        dir.path(),
        ".mindtape/templates/dev.typ",
        "#show: task.with(title: slot(\"title\"), kind: \"dev\")\n",
    );
    let text = stdout(dir.path(), &["templates"]);
    assert_eq!(
        text,
        "dev   kind: dev  1 slot\ntask  kind: any  1 slot   default\n"
    );
}

/// The `kind` column reads `any` for a template whose `kind` is a slot, and
/// the JSON wraps the same rows in `templates` under `$schema`.
#[test]
fn json_wraps_the_rows_in_templates() {
    let dir = scaffold_project("[tasks]\nroots = [\"tasks\"]\n");
    write_file(
        dir.path(),
        ".mindtape/templates/kind-slot.typ",
        "#show: task.with(kind: slot(\"kind\", example: \"test\"))\n",
    );
    let value = json(dir.path(), &["templates", "--json"]);
    support::assert_schema(&value, "templates");
    assert_eq!(
        value["templates"],
        serde_json::json!([
            {"name": "kind-slot", "kind": "any", "slots": 1, "default": false},
            {"name": "task", "kind": "any", "slots": 1, "default": true},
        ])
    );
}

/// The listing has one spelling. Neither the retired `mt add --list` flag nor
/// the retired `mt explain templates` subcommand parses.
#[test]
fn the_retired_spellings_are_gone() {
    let dir = scaffold_project("[tasks]\nroots = [\"tasks\"]\n");
    for args in [&["add", "--list"][..], &["explain", "templates"]] {
        let (ok, text) = run_text(dir.path(), args);
        assert!(!ok, "`mt {args:?}` must not parse: {text}");
    }
}
