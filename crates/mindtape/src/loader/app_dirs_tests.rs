//! Tests for the app-directory env seam.
#![allow(clippy::unwrap_used)]

use std::ffi::OsString;
use std::path::{Path, PathBuf};

use super::{config_home, data_home, resolve};

/// A stand-in for `dirs::config_dir()`, so the fall-through case is asserted
/// without depending on the machine running the test.
#[allow(clippy::unnecessary_wraps)] // must match `dirs::config_dir`'s signature
fn machine_dir() -> Option<PathBuf> {
    Some(PathBuf::from("/machine/config"))
}

/// A set, non-empty override replaces the platform directory entirely — the
/// point of the seam, since on Windows the platform lookup ignores every
/// environment variable.
#[test]
fn a_set_override_wins_over_the_platform_directory() {
    let resolved = resolve(Some(OsString::from("/tmp/pinned")), machine_dir);

    assert_eq!(resolved.as_deref(), Some(Path::new("/tmp/pinned")));
}

/// An empty override is an explicit "no such directory", not a fall-through to
/// the machine's own — that is what makes it usable to isolate a test.
#[test]
fn an_empty_override_disables_the_directory() {
    let resolved = resolve(Some(OsString::new()), machine_dir);

    assert_eq!(resolved, None);
}

/// Unset falls through to the platform resolver.
#[test]
fn an_unset_override_falls_through() {
    let resolved = resolve(None, machine_dir);

    assert_eq!(resolved.as_deref(), Some(Path::new("/machine/config")));
}

/// A resolver with no directory at all stays `None` rather than panicking.
#[test]
fn an_unset_override_with_no_platform_directory_is_none() {
    let resolved = resolve(None, || None);

    assert_eq!(resolved, None);
}

/// Both public entry points read their own variable. Asserted only as "does not
/// panic and agrees with the seam" — the env is process-global, so a test that
/// set the variable would race every other test in this binary.
#[test]
fn the_entry_points_resolve_without_panicking() {
    assert_eq!(
        config_home(),
        resolve(std::env::var_os(super::CONFIG_HOME_ENV), dirs::config_dir)
    );
    assert_eq!(
        data_home(),
        resolve(std::env::var_os(super::DATA_HOME_ENV), dirs::data_local_dir)
    );
}
