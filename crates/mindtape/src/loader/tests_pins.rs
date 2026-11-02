//! Version-aware `[[typst.local-packages]]` pins: a pin serves only the
//! version its manifest declares, and a commit state's own package files win.
#![allow(clippy::unwrap_used)]

use std::path::{Path, PathBuf};

use typst_harvest::{HarvestWorld, harvest};

use super::tests::{PRELUDE_STUB, first_task_title, lib_config, setup, write_pkg};
use crate::test_support::{tempdir, write_file};

/// The title of the one task `tasks/t.typ` under `root` loads as, with the
/// project's pins resolved against `packages` instead of the machine's package
/// root.
fn title_under(
    root: &Path,
    overrides: &[(String, PathBuf)],
    packages: Option<PathBuf>,
) -> Option<String> {
    let world = HarvestWorld::with_root(&root.join("tasks/t.typ"), root).unwrap();
    let world = super::pins::with_packages_at(world, overrides, packages);
    let harvested = harvest(&world).unwrap();
    first_task_title(&harvested.markers)
}

/// A task importing the prelude at `version`, through a stub that reads its
/// title from the call.
fn task_importing(version: &str) -> String {
    format!("#import \"@local/mindtape:{version}\": *\n#show: task.with(title: \"Hi\")\n")
}

/// A prelude stub whose marker always says `Installed`, so a test can tell the
/// installed package from the pin.
const INSTALLED_STUB: &str = "#let task(body, ..meta) = [#metadata((marker: \"mindtape.task\", fields: (title: \"Installed\"), extras: (:))) <mindtape>]\n";

/// A project pinning `lib` at `pin` (`None` leaves its manifest out) and an
/// installed 0.2.0 whose entrypoint is `installed`, with a task importing
/// 0.2.0.
fn mixed_version(
    pin: Option<(&str, &str)>,
    installed: &str,
) -> (tempfile::TempDir, tempfile::TempDir, Vec<(String, PathBuf)>) {
    let dir = setup();
    match pin {
        Some((version, entry)) => write_pkg(dir.path(), "lib", "mindtape", version, entry),
        None => std::fs::create_dir_all(dir.path().join("lib")).unwrap(),
    }
    let packages = tempdir();
    write_pkg(
        packages.path(),
        "local/mindtape/0.2.0",
        "mindtape",
        "0.2.0",
        installed,
    );
    write_file(dir.path(), "tasks/t.typ", task_importing("0.2.0"));
    let overrides = super::package_overrides(dir.path(), &lib_config());
    (dir, packages, overrides)
}

/// The title a task importing 0.2.0 loads as under [`mixed_version`].
fn title_with_pin(pin_version: &str, installed: &str) -> Option<String> {
    let (dir, packages, overrides) = mixed_version(Some((pin_version, PRELUDE_STUB)), installed);
    title_under(dir.path(), &overrides, Some(packages.path().to_path_buf()))
}

#[test]
fn a_pin_at_another_version_falls_through_to_the_installed_package() {
    // The project pins `lib` at 0.3.0, but the task file imports 0.2.0 — the
    // mixed-version state a commit range across a prelude bump overlays. The
    // pin must not capture the import: the installed 0.2.0 serves it.
    assert_eq!(title_with_pin("0.3.0", PRELUDE_STUB), Some("Hi".to_owned()));
}

#[test]
fn a_pin_at_the_imports_version_still_wins_over_the_installed_package() {
    // Both packages declare 0.2.0, so the pin is authoritative: the installed
    // copy's fixed title must not replace the pinned stub's.
    assert_eq!(
        title_with_pin("0.2.0", INSTALLED_STUB),
        Some("Hi".to_owned())
    );
}

#[test]
fn a_pin_with_no_manifest_still_names_its_own_path() {
    // A pin whose manifest cannot be read still serves every version, so Typst
    // reports the pin's own `typst.toml` rather than the installed package's.
    let (dir, packages, overrides) = mixed_version(None, PRELUDE_STUB);
    let world = HarvestWorld::with_root(&dir.path().join("tasks/t.typ"), dir.path()).unwrap();
    let world =
        super::pins::with_packages_at(world, &overrides, Some(packages.path().to_path_buf()));
    let err = harvest(&world).unwrap_err().to_string();
    let manifest = crate::display_path("lib/typst.toml");
    assert!(
        err.contains(&manifest),
        "the pin's manifest is named: {err}"
    );
}

#[test]
fn a_pin_serves_a_task_without_a_package_root() {
    // The provider has no package root of its own; the pin's files still serve
    // the task. The no-platform-root dependency path is covered end to end by
    // the subproject range test.
    let dir = setup();
    write_pkg(dir.path(), "lib", "mindtape", "0.3.0", PRELUDE_STUB);
    write_file(dir.path(), "tasks/t.typ", task_importing("0.3.0"));
    let overrides = super::package_overrides(dir.path(), &lib_config());
    assert_eq!(
        title_under(dir.path(), &overrides, None),
        Some("Hi".to_owned())
    );
}
