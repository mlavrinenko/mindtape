//! Shared test infrastructure for integration tests.
//!
//! Provides helpers for creating temporary Typst projects with the
//! `@mindtape` package layout needed by the eval and store pipelines.

#![allow(clippy::unwrap_used, clippy::missing_panics_doc, dead_code)]

use std::path::PathBuf;

use typst::foundations::Datetime;

/// Standard prelude content matching `lib/prelude.typ`.
pub const PRELUDE_CONTENT: &str = r#"
#let due(year, month, day) = metadata(("due", datetime(year: year, month: month, day: day)))
#let start(year, month, day) = metadata(("start", datetime(year: year, month: month, day: day)))
#let id(uuid) = metadata(("id", uuid))
#let tag(name) = metadata(("tag", name))
#let rank(n) = metadata(("rank", n))
#let high = rank(100)
#let medium = rank(50)
#let low = rank(10)
"#;

pub const TYPST_TOML_CONTENT: &str =
    "[package]\nname = \"mindtape\"\nversion = \"0.1.0\"\nentrypoint = \"prelude.typ\"\n";

/// Create a temporary project directory with the `@mindtape` package layout:
/// - `Cargo.toml` (project root marker)
/// - `lib/prelude.typ` (mindtape functions)
/// - `lib/typst.toml` (package manifest)
///
/// Returns the project root path (kept alive — caller must hold the `PathBuf`).
pub fn setup_typst_project() -> PathBuf {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.keep();
    std::fs::write(root.join("Cargo.toml"), "").unwrap();
    let lib_dir = root.join("lib");
    std::fs::create_dir(&lib_dir).unwrap();
    std::fs::write(lib_dir.join("prelude.typ"), PRELUDE_CONTENT).unwrap();
    std::fs::write(lib_dir.join("typst.toml"), TYPST_TOML_CONTENT).unwrap();
    root
}

/// Shorthand for creating a `Datetime` from year/month/day.
pub fn ymd(year: i32, month: u8, day: u8) -> Datetime {
    Datetime::from_ymd(year, month, day).unwrap()
}
