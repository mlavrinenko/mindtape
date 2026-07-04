//! Shared scaffolding for CLI end-to-end tests (`tests/*_e2e.rs`).
//!
//! Not a test target itself: cargo only auto-discovers `.rs` files directly
//! under `tests/`, so this module only compiles into a binary that `mod
//! support;`s it.
#![allow(clippy::unwrap_used)]

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
