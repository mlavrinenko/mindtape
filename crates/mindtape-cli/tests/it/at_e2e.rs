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

/// The registry's set mark, read back from `mt _cli --json`, is the whole
/// rule: a marked read command runs on `@both`, and every other read command
/// refuses it with the one-project refusal before it runs.
#[test]
fn a_set_runs_exactly_the_commands_the_registry_marks() {
    let (_alpha, _beta, cfg) = setup();
    for (path, node) in crate::read_no_write_e2e::read_leaves() {
        let on_set = node
            .get("runs_on_set")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or_else(|| panic!("`{path}` carries no runs_on_set mark"));
        let mut args = vec!["@both"];
        args.extend(path.split(' '));
        args.extend_from_slice(crate::read_no_write_e2e::extra_args(&path));
        let out = run(cfg.path(), &args);
        let stderr = String::from_utf8(out.stderr).unwrap();
        let refused = stderr.contains("runs on one project, and @both holds 2 projects");
        if on_set {
            assert!(out.status.success() && !refused, "{args:?}: {stderr}");
        } else {
            assert_eq!(out.status.code(), Some(1), "{args:?}: {stderr}");
            assert!(refused, "{args:?}: {stderr}");
        }
    }
}

/// Two projects named `blog`, under `projects/` and `oss/` of one root, both in
/// the `@both` set. Returns the root, which owns both, and the config home.
fn two_blogs() -> (tempfile::TempDir, tempfile::TempDir) {
    let root = tempfile::tempdir().unwrap();
    for (parent, title) in [("projects", "Mine"), ("oss", "Theirs")] {
        let project = support::project_with_tasks(&[("post.typ", title)]).keep();
        std::fs::create_dir_all(root.path().join(parent)).unwrap();
        std::fs::rename(project, root.path().join(parent).join("blog")).unwrap();
    }
    let cfg = tempfile::tempdir().unwrap();
    let mine = root.path().join("projects/blog");
    let theirs = root.path().join("oss/blog");
    support::write_at_toml(cfg.path(), &[("both", &[&mine, &theirs])]);
    (root, cfg)
}

#[test]
fn a_name_two_directories_share_is_refused_listing_both_and_how_to_pick() {
    let (root, cfg) = two_blogs();
    let (code, _, stderr) = support::parts_of(&run(cfg.path(), &["@blog", "ls"]));
    assert_eq!(code, Some(2), "{stderr}");
    // The path at.toml names, in the separator `mt` shows people.
    let shown = |rel: &str| mindtape::display_path(root.path().join(rel));
    assert_eq!(
        stderr,
        format!(
            "error: name matches several directories: '@blog'\n\
             hint: pick one by its path suffix, as in mt @projects/blog ls\n\
             note: candidate: @projects/blog → {}\n\
             note: candidate: @oss/blog → {}\n",
            shown("projects/blog"),
            shown("oss/blog")
        )
    );
}

#[test]
fn the_suffix_name_mt_at_prints_selects_that_project() {
    let (_root, cfg) = two_blogs();
    let (ok, text) = support::text_of(&run(cfg.path(), &["at", "both"]));
    assert!(ok && text.contains("oss/blog "), "{text}");
    let (ok, text) = support::text_of(&run(cfg.path(), &["@oss/blog", "ls", ""]));
    assert!(
        ok && text.contains("Theirs") && !text.contains("Mine"),
        "{text}"
    );
}
