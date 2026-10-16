//! Where `mt` keeps its per-user state: the global config directory (the XDG
//! `defaults.toml` layer, the JSON Schema, presets, `at.toml`) and the
//! local data directory (the installed `@local/mindtape` Typst package).
//!
//! Both resolve through [`dirs`] by default — but `dirs` reads the environment
//! only on Unix. On Windows it goes through the `SHGetKnownFolderPath`
//! known-folder API, which no environment variable overrides, so a test that
//! points `XDG_CONFIG_HOME` at a tempdir isolates nothing there and a test that
//! asserts a write landed under an injected `XDG_DATA_HOME` fails outright.
//!
//! Hence one env seam that works on every platform: `$MINDTAPE_CONFIG_HOME` and
//! `$MINDTAPE_DATA_HOME` replace the resolved directory wholesale. Set (and
//! non-empty) wins over `dirs`; set-but-empty means "this app directory does
//! not exist", which drops the global config layer rather than falling back to
//! the machine's own — an explicit disable, never a silent fall-through.
//! Unset resolves as before.

use std::ffi::OsString;
use std::path::PathBuf;

/// Environment override for the global config directory (the one holding
/// `mindtape/defaults.toml`).
pub const CONFIG_HOME_ENV: &str = "MINDTAPE_CONFIG_HOME";

/// Environment override for the local data directory (the one holding
/// `typst/packages/local/mindtape/`).
pub const DATA_HOME_ENV: &str = "MINDTAPE_DATA_HOME";

/// The directory `mindtape/defaults.toml`, `config.schema.json`, `presets/` and
/// `at.toml` live under: `$MINDTAPE_CONFIG_HOME` when set, else
/// `dirs::config_dir()`. `None` means no such directory applies.
#[must_use]
pub fn config_home() -> Option<PathBuf> {
    resolve(std::env::var_os(CONFIG_HOME_ENV), dirs::config_dir)
}

/// The directory the `@local/mindtape` Typst package is installed under:
/// `$MINDTAPE_DATA_HOME` when set, else `dirs::data_local_dir()`. `None` means
/// no such directory applies.
#[must_use]
pub fn data_home() -> Option<PathBuf> {
    resolve(std::env::var_os(DATA_HOME_ENV), dirs::data_local_dir)
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
