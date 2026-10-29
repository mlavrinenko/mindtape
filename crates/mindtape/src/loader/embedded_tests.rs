#![allow(clippy::unwrap_used)]

use std::path::Path;

use typst_harvest::{HVal, HarvestWorld, harvest};

use super::{PRELUDE, PRELUDE_VERSION, package_provider};
use crate::loader::first_task_marker;

/// The title of the one task `main` declares, evaluated with `@local`
/// packages read under `package_root` and the embedded prelude behind it.
fn title_under(package_root: &Path, main: &str) -> Option<String> {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("t.typ"), main).unwrap();
    let world = HarvestWorld::with_root(&dir.path().join("t.typ"), dir.path())
        .unwrap()
        .with_file_provider(package_provider(Some(package_root.to_path_buf())));
    let harvest = harvest(&world).unwrap();
    first_task_marker(&harvest.markers)
        .and_then(|marker| marker.get("fields"))
        .and_then(|fields| fields.get("title"))
        .and_then(HVal::as_str)
        .map(str::to_owned)
}

fn task_file() -> String {
    format!(
        "#import \"@local/mindtape:{PRELUDE_VERSION}\": *\n\
         #show: task.with(title: \"Embedded\")\n"
    )
}

#[test]
fn a_task_loads_where_no_package_directory_holds_the_prelude() {
    let empty = tempfile::tempdir().unwrap();
    assert_eq!(
        title_under(empty.path(), &task_file()),
        Some("Embedded".to_owned())
    );
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

/// A published crate cannot reach `lib/`, so it carries a copy. Edit
/// `lib/prelude.typ`, then copy it to `prelude.typ` beside this file.
#[test]
fn the_embedded_prelude_is_the_in_tree_prelude() {
    const IN_TREE: &str = include_str!("../../../../lib/prelude.typ");
    assert!(
        PRELUDE == IN_TREE,
        "crates/mindtape/src/loader/prelude.typ differs from lib/prelude.typ; \
         run `cp lib/prelude.typ crates/mindtape/src/loader/prelude.typ`"
    );
}
