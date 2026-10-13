//! `[typst]` config: Typst evaluation settings and project-local packages.

use serde::Deserialize;

/// `[typst]` — Typst evaluation settings.
#[derive(Debug, Clone, Default, Deserialize, PartialEq, schemars::JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct TypstConfig {
    /// `@local/<name>` packages to load from a directory in the project, so a
    /// task can import a package that is not installed globally. `mt`
    /// registers `@local/mindtape` itself when `<root>/lib` holds the
    /// `mindtape` package; list any other package here.
    pub local_packages: Vec<LocalPackage>,
}

/// `[[typst.local-packages]]` — one `@local/<name>` package loaded from a project directory.
#[derive(Debug, Clone, Default, Deserialize, PartialEq, schemars::JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct LocalPackage {
    /// The `@local/<name>` package name.
    pub name: String,
    /// Directory holding the package's `typst.toml`, relative to the project root.
    pub path: String,
}
