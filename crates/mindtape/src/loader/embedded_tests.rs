#![allow(clippy::unwrap_used)]

use std::path::Path;

use typst_harvest::{Harvest, HarvestError, HarvestWorld, harvest};

use super::{PRELUDE, PRELUDE_SNAPSHOTS, PRELUDE_SUPPORTED, PRELUDE_VERSION, prelude_supports};
use crate::loader::tests::first_task_title;
use crate::loader::unsupported_prelude_version;

/// The harvest of `main`, evaluated with `@local` packages read under
/// `package_root` and the embedded prelude behind them.
fn harvest_under(package_root: &Path, main: &str) -> Result<Harvest, HarvestError> {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("t.typ"), main).unwrap();
    let world = HarvestWorld::with_root(&dir.path().join("t.typ"), dir.path()).unwrap();
    let world = crate::loader::pins::with_packages_at(world, &[], Some(package_root.to_path_buf()));
    harvest(&world)
}

/// The title of the one task `main` declares, under [`harvest_under`].
fn title_under(package_root: &Path, main: &str) -> Option<String> {
    let harvest = harvest_under(package_root, main).unwrap();
    first_task_title(&harvest.markers)
}

fn task_file(version: &str) -> String {
    format!(
        "#import \"@local/mindtape:{version}\": *\n\
         #show: task.with(title: \"Embedded\")\n"
    )
}

/// Every listed version must evaluate through the copy in the binary, with no
/// package directory to hold it: that is what keeps an old pin loadable on a
/// fresh clone or a CI runner.
#[test]
fn every_supported_prelude_loads_where_no_package_directory_holds_it() {
    let empty = tempfile::tempdir().unwrap();
    for snapshot in PRELUDE_SNAPSHOTS {
        assert_eq!(
            title_under(empty.path(), &task_file(snapshot.version)),
            Some("Embedded".to_owned()),
            "version {} did not load",
            snapshot.version
        );
    }
}

#[test]
fn an_installed_prelude_wins_over_the_embedded_one() {
    let root = tempfile::tempdir().unwrap();
    let pkg = root.path().join("local/mindtape").join(PRELUDE_VERSION);
    std::fs::create_dir_all(&pkg).unwrap();
    std::fs::write(pkg.join("typst.toml"), super::prelude_manifest()).unwrap();
    let installed = PRELUDE.replacen("#let task(", "#let _installed = true\n#let task(", 1);
    assert_ne!(installed, PRELUDE, "the prelude no longer defines `task(`");
    std::fs::write(pkg.join("prelude.typ"), installed).unwrap();
    let main = format!(
        "#import \"@local/mindtape:{PRELUDE_VERSION}\": *\n\
         #show: task.with(title: if _installed {{ \"Installed\" }} else {{ \"\" }})\n"
    );
    assert_eq!(
        title_under(root.path(), &main),
        Some("Installed".to_owned())
    );
}

/// `PRELUDE_SUPPORTED` is the one list, the snapshots carry exactly those
/// versions, and the current version is the newest entry.
#[test]
fn the_supported_set_lists_every_snapshot_and_ends_at_the_current_version() {
    let snapshots: Vec<&str> = PRELUDE_SNAPSHOTS
        .iter()
        .map(|found| found.version)
        .collect();
    assert_eq!(snapshots, PRELUDE_SUPPORTED);
    assert_eq!(PRELUDE_SUPPORTED.last(), Some(&PRELUDE_VERSION));
    let highest = PRELUDE_SUPPORTED
        .iter()
        .map(|version| {
            version
                .parse::<typst::syntax::package::PackageVersion>()
                .unwrap()
        })
        .max()
        .unwrap();
    assert_eq!(highest.to_string(), PRELUDE_VERSION);
}

#[test]
fn the_supported_set_names_the_two_versions_this_release_reads() {
    assert_eq!(PRELUDE_SUPPORTED, ["0.2.0", "0.3.0"]);
    assert!(prelude_supports("0.2.0"));
    assert!(prelude_supports("0.3.0"));
    assert!(!prelude_supports("0.1.0"));
    assert!(!prelude_supports("99.99.99"));
    assert!(!prelude_supports("0.3"));
}

/// An unsupported version is one stable, typed refusal the CLI can fold,
/// naming the version and the set, not Typst's own not-found wording.
#[test]
fn an_unsupported_prelude_names_the_version_and_the_supported_set() {
    let empty = tempfile::tempdir().unwrap();
    let error = harvest_under(empty.path(), &task_file("0.1.0")).unwrap_err();
    let message = error.to_string();
    assert_eq!(
        unsupported_prelude_version(&message),
        Some("0.1.0"),
        "{message}"
    );
    assert!(message.contains("0.2.0, 0.3.0"), "{message}");
}

/// The gate runs before any package directory: a working `0.1.0` install
/// cannot make an unsupported version load.
#[test]
fn an_installed_unsupported_prelude_is_still_refused() {
    let root = tempfile::tempdir().unwrap();
    let pkg = root.path().join("local/mindtape/0.1.0");
    crate::test_support::write_package_version(&pkg, "0.1.0", PRELUDE);
    let error = harvest_under(root.path(), &task_file("0.1.0")).unwrap_err();
    assert_eq!(
        unsupported_prelude_version(&error.to_string()),
        Some("0.1.0"),
        "{error}"
    );
}

/// A published crate cannot reach `lib/`, so it carries a copy. Edit
/// `lib/prelude.typ`, then copy it to
/// `crates/mindtape/src/loader/preludes/<version>/prelude.typ` beside this
/// file.
#[test]
fn the_embedded_prelude_is_the_in_tree_prelude() {
    const IN_TREE: &str = include_str!("../../../../lib/prelude.typ");
    assert!(
        PRELUDE == IN_TREE,
        "crates/mindtape/src/loader/preludes/{PRELUDE_VERSION}/prelude.typ differs from \
         lib/prelude.typ; run `cp lib/prelude.typ \
         crates/mindtape/src/loader/preludes/{PRELUDE_VERSION}/prelude.typ`"
    );
}
