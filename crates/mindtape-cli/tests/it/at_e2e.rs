//! `mt @name` and `mt at`, end to end: a project is `-w <path>`, a set is held
//! and refused by a command that runs on one project.
#![allow(clippy::unwrap_used, clippy::disallowed_methods)]

use std::path::Path;

use crate::support;

/// Two projects, `alpha` and `beta`, and a config home whose `at.toml` names
/// `@alpha` a project and `@both` a set of the two.
pub(crate) fn setup() -> (tempfile::TempDir, tempfile::TempDir, tempfile::TempDir) {
    let alpha = support::project_with_tasks(&[("one.typ", "One")]);
    let beta = support::project_with_tasks(&[("two.typ", "Two")]);
    let cfg = tempfile::tempdir().unwrap();
    support::write_at_toml(
        cfg.path(),
        &[
            ("alpha", &[alpha.path()]),
            ("both", &[alpha.path(), beta.path()]),
        ],
    );
    (alpha, beta, cfg)
}

pub(crate) fn run(cfg: &Path, args: &[&str]) -> std::process::Output {
    let here = tempfile::tempdir().unwrap();
    support::mt(here.path())
        .env(mindtape::CONFIG_HOME_ENV, cfg)
        .args(args)
        .output()
        .unwrap()
}

#[test]
fn a_project_name_runs_the_command_there() {
    let (_alpha, _beta, cfg) = setup();
    let (ok, text) = support::text_of(&run(cfg.path(), &["@alpha", "ls", ""]));
    assert!(ok && text.contains("One"), "{text}");
}

#[test]
fn a_set_name_is_refused_by_a_one_project_command() {
    let (_alpha, _beta, cfg) = setup();
    let out = run(cfg.path(), &["@both", "eval", "one"]);
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert_eq!(out.status.code(), Some(1), "{stderr}");
    assert!(
        stderr.starts_with("error: `eval` runs on one project, and @both holds 2 projects\n"),
        "{stderr}"
    );
    assert!(
        stderr.contains("hint: name one project, as in mt @"),
        "{stderr}"
    );
}

#[test]
fn mt_at_lists_the_entries_and_the_members_of_one() {
    let (_alpha, _beta, cfg) = setup();
    let (ok, text) = support::text_of(&run(cfg.path(), &["at"]));
    assert!(
        ok && text.contains("@alpha → ") && text.contains("@both → "),
        "{text}"
    );
    let (ok, text) = support::text_of(&run(cfg.path(), &["at", "both"]));
    assert!(ok && text.starts_with("@both holds 2 projects\n"), "{text}");
}

#[test]
fn a_set_name_is_refused_by_a_command_that_never_opens_a_project() {
    let (_alpha, _beta, cfg) = setup();
    for args in [["@both", "init"], ["@both", "install"], ["@both", "done"]] {
        let here = tempfile::tempdir().unwrap();
        let out = support::mt(here.path())
            .env(mindtape::CONFIG_HOME_ENV, cfg.path())
            .args(args)
            .output()
            .unwrap();
        let stderr = String::from_utf8(out.stderr).unwrap();
        assert_eq!(out.status.code(), Some(1), "{args:?}: {stderr}");
        assert!(
            stderr.contains(&format!(
                "`{}` runs on one project, and @both holds 2 projects",
                args[1]
            )),
            "{args:?}: {stderr}"
        );
        let written = std::fs::read_dir(here.path()).unwrap().count();
        assert_eq!(written, 0, "{args:?} wrote into the working directory");
    }
}
