//! Shared fixture for the `[[on.add]]` end-to-end suites: the template, the
//! base config, the Typst helpers a derive imports, and the project builder.
#![allow(dead_code, clippy::unwrap_used, clippy::disallowed_methods)]

use std::path::Path;

use crate::support;

pub(super) const TEMPLATE: &str = mindtape::typst_task!(
    r#"(
  title: slot("title"),
  tags: slot("tags", kind: "list", default: ("base",)),
)
"#
);

pub(super) const BASE: &str = r#"
[tasks]
roots = ["tasks"]

[[tasks.status]]
value = "proposed"

[[tasks.status]]
value = "wip"

[[tasks.status]]
value = "done"
terminal = true

[templates]
dirs = [".mindtape/templates"]
default = "task"
"#;

/// The Typst helpers the derive tests call, written at the project root.
const HELPER: &str = r#"
#let classify-tags(draft) = ("derived", "from-title")

#let classify-kind(draft) = "bug"

#let classify-dir(draft) = "tasks/" + draft.at("kind")

#let bad-tags(draft) = 7
"#;

/// A project with `config` plus [`BASE`] and [`TEMPLATE`] as its default, and
/// [`HELPER`] at the root for a `derive` to import.
pub(super) fn project(extra: &str) -> tempfile::TempDir {
    let dir = support::scaffold_project(&format!("{BASE}{extra}"));
    std::fs::create_dir_all(dir.path().join(".mindtape/templates")).unwrap();
    std::fs::write(dir.path().join(".mindtape/templates/task.typ"), TEMPLATE).unwrap();
    std::fs::write(dir.path().join("add.typ"), HELPER).unwrap();
    dir
}

/// Run `mt add` in `dir` with `args`, returning stdout, stderr and success.
pub(super) fn add(dir: &Path, args: &[&str]) -> (String, String, bool) {
    let output = support::mt(dir).args(args).output().unwrap();
    (
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
        output.status.success(),
    )
}

pub(super) fn read(dir: &Path, rel: &str) -> String {
    std::fs::read_to_string(dir.join(rel)).unwrap()
}

/// A hook that fills `kind`, `due` and the destination from `kind`.
pub(super) const FILL_AND_DIR: &str = r#"
[[on.add]]
name = "fixes"
when = 'template == "task"'
actions = [{ set = { kind = "bug", due = "2026-12-01" }, dir = "tasks/{kind}" }]

[[on.add]]
name = "regression"
when = 'kind == "bug"'
actions = [{ append = { tags = ["regression"] } }]
"#;

/// A hook whose only action derives both a field and the destination, with the
/// function names `HELPER` exports.
pub(super) const DERIVE_DIR: &str = r#"
[[on.add]]
name = "classify"
when = 'template == "task"'
actions = [
  { set = { kind = "bug" } },
  { derive = { typst = "add.typ", function = "classify-tags", field = "tags" } },
  { dir = { derive = { typst = "add.typ", function = "classify-dir" } } },
]

[[on.add]]
name = "regression"
when = 'kind == "bug"'
actions = [{ append = { tags = ["regression"] } }]
"#;
