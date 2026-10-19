//! Where `mt` keeps its per-user state: the global config directory (the XDG
//! `defaults.toml` layer, the JSON Schema, presets, `at.toml`), the local data
//! directory (the `allowed/` records) and the package root the
//! `@local/mindtape` Typst package is installed under.
//!
//! The two app directories resolve through [`dirs`] by default, but `dirs`
//! reads the environment only on Unix. On Windows it goes through the
//! `SHGetKnownFolderPath` known-folder API, which no environment variable
//! overrides, so a test that points `XDG_CONFIG_HOME` at a tempdir isolates
//! nothing there and a test that asserts a write landed under an injected
//! `XDG_DATA_HOME` fails outright.
//!
//! Hence one env seam that works on every platform: `$MINDTAPE_CONFIG_HOME` and
//! `$MINDTAPE_DATA_HOME` replace the resolved directory wholesale. Set (and
//! non-empty) wins over `dirs`; set-but-empty means "this app directory does
//! not exist", which drops the global config layer rather than falling back to
//! the machine's own: an explicit disable, never a silent fall-through.
//! Unset resolves as before.
//!
//! The package root is not an app directory of `mt`'s own. Unset, it is the
//! one Typst and tinymist read `@local` packages from, which typst-world
//! resolves, so `mt install` writes where every Typst tool looks.

use std::ffi::OsString;
use std::path::PathBuf;

/// Environment override for the global config directory (the one holding
/// `mindtape/defaults.toml`).
pub const CONFIG_HOME_ENV: &str = "MINDTAPE_CONFIG_HOME";

/// Environment override for the local data directory, and with it the
/// package root under its `typst/packages/`.
pub const DATA_HOME_ENV: &str = "MINDTAPE_DATA_HOME";

/// The directory `mindtape/defaults.toml`, `config.schema.json`, `presets/` and
/// `at.toml` live under: `$MINDTAPE_CONFIG_HOME` when set, else
/// `dirs::config_dir()`. `None` means no such directory applies.
#[must_use]
pub fn config_home() -> Option<PathBuf> {
    resolve(std::env::var_os(CONFIG_HOME_ENV), dirs::config_dir)
}

/// The directory the `mindtape/allowed/` records live under:
/// `$MINDTAPE_DATA_HOME` when set, else `dirs::data_local_dir()`. `None` means
/// no such directory applies.
#[must_use]
pub fn data_home() -> Option<PathBuf> {
    resolve(std::env::var_os(DATA_HOME_ENV), dirs::data_local_dir)
}

/// The directory holding `local/<name>/<version>/`, which `mt install` writes
/// the `@local/mindtape` package under and every unoverridden `@local` import
/// resolves from: `$MINDTAPE_DATA_HOME/typst/packages` when that is set, else
/// [`typst_world::platform_package_root`], the directory Typst and tinymist
/// read (`$TYPST_PACKAGE_PATH`, else `dirs::data_dir()` joined with
/// `typst/packages`).
///
/// `None` resolves no global package at all, for a write and a read alike:
/// `$MINDTAPE_DATA_HOME` set but empty, or no Typst package directory.
#[must_use]
pub fn package_root() -> Option<PathBuf> {
    package_root_from(
        std::env::var_os(DATA_HOME_ENV),
        typst_world::platform_package_root,
    )
}

/// [`package_root`] with its inputs passed in: the data home's env value and
/// Typst's own package root for when it is unset.
fn package_root_from(
    data_home: Option<OsString>,
    platform: fn() -> Option<PathBuf>,
) -> Option<PathBuf> {
    match data_home {
        Some(value) => resolve(Some(value), || None).map(|dir| dir.join("typst").join("packages")),
        None => platform(),
    }
}

/// The shared override rule: a non-empty env value wins, an empty one means
/// "none", an absent one falls through to `fallback`. Pure in its env input so
/// both cases are testable without mutating the process environment.
fn resolve(value: Option<OsString>, fallback: fn() -> Option<PathBuf>) -> Option<PathBuf> {
    match value {
        Some(value) if value.is_empty() => None,
        Some(value) => Some(PathBuf::from(value)),
        None => fallback(),
    }
}

#[cfg(test)]
#[path = "app_dirs_tests.rs"]
mod tests;
