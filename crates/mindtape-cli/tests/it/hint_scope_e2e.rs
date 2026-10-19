//! A hint's `mt` command carries the `@name` or `-w <dir>` the run was given,
//! so it runs as printed from where the user stood.
#![allow(clippy::unwrap_used, clippy::disallowed_methods)]

use std::path::Path;

use crate::support;

/// A project whose one task misspells its status helper, so `mt check` hints
/// that `mt flip --list` lists the statuses.
fn misspelled() -> tempfile::TempDir {
    let project = support::project_with_tasks(&[]);
    std::fs::write(
        project.path().join("tasks/bad.typ"),
        mindtape::typst_task!("(\n  title: \"Bad\",\n  status: finishd(2026, 10, 19),\n)\n"),
    )
    .unwrap();
    project
}

/// The stderr of `mt <selector…> check` in `dir` with `env` set, which must
/// exit 1.
fn check_fails(dir: &Path, selector: &[&str], env: &[(&str, &Path)]) -> String {
    let mut command = support::mt(dir);
    command
        .envs(env.iter().copied())
        .args(selector)
        .arg("check");
    let (code, _, stderr) = support::parts_of(&command.output().unwrap());
    assert_eq!(code, Some(1), "{stderr}");
    stderr
}

fn hint(command: &str) -> String {
    format!("hint: run {command} to see the statuses\n")
}

/// `stderr` hints `command`, which lists the statuses when run as printed in
/// `dir` with `env` set.
fn runs_as_printed(stderr: &str, command: &str, dir: &Path, env: &[(&str, &Path)]) {
    assert!(stderr.contains(&hint(command)), "{stderr}");
    let words: Vec<&str> = command.split(' ').skip(1).collect();
    let mut run = support::mt(dir);
    run.envs(env.iter().copied()).args(&words);
    let (ok, text) = support::text_of(&run.output().unwrap());
    assert!(ok && text.contains("done"), "{command}: {text}");
}

/// A config home whose `at.toml` names `project` `@blog`.
fn naming_blog(project: &Path) -> tempfile::TempDir {
    let cfg = tempfile::tempdir().unwrap();
    support::write_at_toml(cfg.path(), &[("blog", &[project])]);
    cfg
}

#[test]
fn a_named_projects_hint_runs_under_its_name() {
    let project = misspelled();
    let cfg = naming_blog(project.path());
    let here = tempfile::tempdir().unwrap();
    let env = [(mindtape::CONFIG_HOME_ENV, cfg.path())];
    let stderr = check_fails(here.path(), &["@blog"], &env);
    runs_as_printed(&stderr, "mt @blog flip --list", here.path(), &env);
}

#[test]
fn a_worktree_flags_hint_carries_the_flag_as_typed() {
    let parent = tempfile::tempdir().unwrap();
    let project = misspelled();
    std::fs::rename(project.path(), parent.path().join("blog")).unwrap();
    let stderr = check_fails(parent.path(), &["-w", "blog"], &[]);
    runs_as_printed(&stderr, "mt -w blog flip --list", parent.path(), &[]);
}

#[test]
fn a_run_that_found_its_project_keeps_its_hints_bare() {
    let project = misspelled();
    let inside = check_fails(project.path(), &[], &[]);
    let here = tempfile::tempdir().unwrap();
    let by_env = check_fails(here.path(), &[], &[("MINDTAPE_WORKTREE", project.path())]);
    for stderr in [inside, by_env] {
        assert!(stderr.contains(&hint("mt flip --list")), "{stderr}");
    }
}

#[test]
fn a_global_commands_hint_stays_bare_under_a_worktree_flag() {
    let parent = tempfile::tempdir().unwrap();
    std::fs::create_dir(parent.path().join("new")).unwrap();
    let out = support::mt(parent.path())
        .args(["-w", "new", "init", "--preset", "nope"])
        .output()
        .unwrap();
    let (code, _, stderr) = support::parts_of(&out);
    assert_eq!(code, Some(2), "{stderr}");
    assert!(
        stderr.contains("hint: run mt preset ls to see the presets\n"),
        "{stderr}"
    );
}
