//! Unit tests for the destination guard: the resolution that runs before the
//! policy answers, and the outside-root case it leaves to the warning.
#![allow(clippy::unwrap_used)]

use super::{Mode, check, resolve};

#[test]
fn resolve_appends_missing_components_to_the_nearest_existing_ancestor() {
    let dir = mindtape::test_support::tempdir();
    std::fs::create_dir_all(dir.path().join("tasks")).unwrap();

    let resolved = resolve(&dir.path().join("tasks/fresh/new.typ"));

    assert_eq!(
        resolved,
        mindtape::canonical(dir.path())
            .unwrap()
            .join("tasks/fresh/new.typ")
    );
}

#[cfg(unix)]
#[test]
fn resolve_follows_a_symlink_before_folding_dot_dot() {
    let dir = mindtape::test_support::tempdir();
    std::fs::create_dir_all(dir.path().join("target/sub")).unwrap();
    std::os::unix::fs::symlink(dir.path().join("target/sub"), dir.path().join("link")).unwrap();

    let resolved = resolve(&dir.path().join("link/../new.typ"));

    assert_eq!(
        resolved,
        mindtape::canonical(dir.path())
            .unwrap()
            .join("target/new.typ")
    );
}

#[test]
fn a_destination_outside_every_root_is_not_refused() {
    let (_dir, project) = crate::test_util::make_project(&[], mindtape::Config::default());
    let outside = mindtape::test_support::tempdir();

    assert!(check(&project, &outside.path().join("new.typ"), Mode::Add).is_ok());
}

/// The final component is never followed, so a destination that is itself a
/// symlink is refused rather than answered for its target.
#[cfg(unix)]
#[test]
fn a_destination_that_is_a_symlink_is_refused() {
    let (dir, mut project) = crate::test_util::make_project(&[], mindtape::Config::default());
    project.root = mindtape::canonical(dir.path()).unwrap();
    let link = project.root.join("tasks/link.typ");
    std::fs::write(project.root.join("tasks/real.typ"), "").unwrap();
    std::os::unix::fs::symlink(project.root.join("tasks/real.typ"), &link).unwrap();

    let error = check(&project, &link, Mode::Add).unwrap_err();
    assert!(
        error.to_string().contains("symbolic link"),
        "the refusal names the link: {error}"
    );
}
