//! Tests for the app-directory env seam.
#![allow(clippy::unwrap_used)]

use std::ffi::OsString;
use std::path::{Path, PathBuf};

use super::{config_home, data_home, package_root, package_root_from, resolve};

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

/// A stand-in for `typst_world::platform_package_root()`, the directory Typst
/// and tinymist read `@local` packages from on the machine running the test.
#[allow(clippy::unnecessary_wraps)] // must match `platform_package_root`'s signature
fn typst_root() -> Option<PathBuf> {
    Some(PathBuf::from("/machine/typst/packages"))
}

/// A set data home holds the package root, so `mt install` and the loader
/// both move with it.
#[test]
fn a_set_data_home_holds_the_package_root() {
    let root = package_root_from(Some(OsString::from("/tmp/pinned")), typst_root);

    assert_eq!(
        root,
        Some(Path::new("/tmp/pinned").join("typst").join("packages"))
    );
}

/// An empty data home resolves no global package, for a read and a write.
#[test]
fn an_empty_data_home_names_no_package_root() {
    assert_eq!(package_root_from(Some(OsString::new()), typst_root), None);
}

/// Unset, the package root is Typst's own, not one under `data_home()`:
/// on Windows that is `%APPDATA%` where `data_home()` is `%LOCALAPPDATA%`.
#[test]
fn an_unset_data_home_uses_typsts_package_root() {
    assert_eq!(package_root_from(None, typst_root), typst_root());
}

/// The entry point takes the fallback from typst-world, the reader every
/// world uses, so the install and the read can't name two directories.
#[test]
fn the_package_root_falls_back_to_typst_worlds() {
    assert_eq!(
        package_root(),
        package_root_from(
            std::env::var_os(super::DATA_HOME_ENV),
            typst_world::platform_package_root
        )
    );
}
