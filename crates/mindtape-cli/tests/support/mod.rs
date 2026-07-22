//! Shared scaffolding for CLI end-to-end tests (`tests/*_e2e.rs`).
//!
//! Not a test target itself: cargo only auto-discovers `.rs` files directly
//! under `tests/`, so this module only compiles into a binary that `mod
//! support;`s it — and each such binary sees the helpers it does not call as
//! dead code, hence the module-wide allow.
#![allow(clippy::unwrap_used, dead_code)]

use std::path::Path;

const PRELUDE: &str = include_str!("../../../../lib/prelude.typ");

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
