//! Shared scaffolding for CLI end-to-end tests (`tests/*_e2e.rs`).
//!
//! Not a test target itself: cargo only auto-discovers `.rs` files directly
//! under `tests/`, so this module only compiles into a binary that `mod
//! support;`s it — and each such binary sees the helpers it does not call as
//! dead code, hence the module-wide allow.
#![allow(clippy::unwrap_used, dead_code)]

use std::path::Path;

const PRELUDE: &str = include_str!("../../../../lib/prelude.typ");

/// Every character that opens Typst syntax, in one line of otherwise ordinary
/// prose: emphasis (`*`, `_`), math (`$`), code (`#`), content-block delimiters
/// (`[`, `]`), a label (`<a>`) and a reference (`@at`). Text carrying these must
/// survive a write unchanged — unescaped, `create-*star*` fails to load and
/// `<a>`/`@at` silently evaluate away.
pub const MARKUP_HAZARDS: &str = "create-*star* _u_ $x$ #h [b] <a> >g @at";

/// A template whose slots span both sides of the escaping decision: a `string`
/// title and an `int` priority render as literals and must stay verbatim, while
/// the `text` slots land in markup and must be escaped. `why` sits inside a link
/// note, the one markup position the harvest reads back as a value, so a test can
/// assert what a filled `text` slot actually means rather than only how it looks.
const SLOT_TEMPLATE: &str = r#"#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: slot("title", prompt: "Task title"),
  priority: slot("weight", kind: "int", default: 5),
  links: related("anchor.typ")[#slot("why", kind: "text")],
)

== Summary

#slot("summary", kind: "text", default: "- TODO")
"#;

/// A throwaway project whose default template is [`SLOT_TEMPLATE`], with the
/// `anchor` task the template's `related` edge points at already filed.
pub fn project_with_slot_template() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    install_lib(dir.path());
    std::fs::create_dir_all(dir.path().join(".mindtape/templates")).unwrap();
    std::fs::write(
        dir.path().join(".mindtape/config.toml"),
        "[tasks]\nroots = [\"tasks\"]\n",
    )
    .unwrap();
    std::fs::write(
        dir.path().join(".mindtape/templates/task.typ"),
        SLOT_TEMPLATE,
    )
    .unwrap();
    std::fs::write(
        dir.path().join("tasks/anchor.typ"),
        "#import \"@local/mindtape:0.2.0\": *\n\
         #show: task.with(title: \"anchor\", status: proposed(2026, 7, 22))\n",
    )
    .unwrap();
    dir
}

/// The canonical-JSON row `mt ls --json` reports for task `id` in `dir`. Reading
/// a value back through the harvest is the only way to tell text that survived a
/// write from text Typst quietly interpreted on the way in.
pub fn task_json(dir: &Path, id: &str) -> serde_json::Value {
    let output = assert_cmd::Command::cargo_bin("mt")
        .unwrap()
        .current_dir(dir)
        .args(["ls", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let parsed: serde_json::Value = serde_json::from_slice(&output).unwrap();
    parsed
        .get("tasks")
        .and_then(serde_json::Value::as_array)
        .unwrap()
        .iter()
        .find(|task| task.get("id").and_then(serde_json::Value::as_str) == Some(id))
        .unwrap_or_else(|| panic!("no task {id:?} in {parsed}"))
        .clone()
}

/// The value at RFC-6901 `pointer` inside a [`task_json`] row (`/links/0/note`).
/// Spelled as a lookup rather than `row["links"][0]`, which the workspace's
/// `indexing_slicing` lint denies.
pub fn at<'a>(row: &'a serde_json::Value, pointer: &str) -> &'a serde_json::Value {
    row.pointer(pointer)
        .unwrap_or_else(|| panic!("no {pointer} in {row}"))
}

/// Install the in-tree `@local/mindtape` lib override and an empty `tasks/`
/// root under `dir`, so a task filed there can actually be evaluated.
pub fn install_lib(dir: &Path) {
    std::fs::create_dir_all(dir.join("lib")).unwrap();
    std::fs::write(dir.join("lib/prelude.typ"), PRELUDE).unwrap();
    std::fs::write(
        dir.join("lib/typst.toml"),
        "[package]\nname = \"mindtape\"\nversion = \"0.2.0\"\nentrypoint = \"prelude.typ\"\n",
    )
    .unwrap();
    std::fs::create_dir_all(dir.join("tasks")).unwrap();
}

/// A throwaway project — lib override, `[tasks] roots = ["tasks"]`, and one
/// `wip` task titled `title` — for tests that only need something to list.
pub fn project_with_task(title: &str) -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    install_lib(dir.path());
    std::fs::create_dir_all(dir.path().join(".mindtape")).unwrap();
    std::fs::write(
        dir.path().join(".mindtape/config.toml"),
        "[tasks]\nroots = [\"tasks\"]\n",
    )
    .unwrap();
    std::fs::write(
        dir.path().join("tasks/test.typ"),
        format!(
            r#"#metadata((
  marker: "mindtape.task",
  fields: (
    title: "{title}",
    status: (name: "wip", date: datetime(year: 2026, month: 6, day: 1), note: none),
    priority: 5,
  ),
  extras: (:),
))
"#
        ),
    )
    .unwrap();
    dir
}
