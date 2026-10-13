//! `mt go`, `mt eval` and `@worktree` problems in the error
//! format: the value single-quoted and the next step on a `hint:` line.
#![allow(clippy::unwrap_used, clippy::disallowed_methods)]

use std::path::Path;

use crate::support;

/// `mt <args>` in `dir` with no editor in the environment.
fn run(dir: &Path, args: &[&str]) -> (Option<i32>, String) {
    let output = support::mt(dir)
        .env_remove("EDITOR")
        .args(args)
        .output()
        .unwrap();
    (
        output.status.code(),
        String::from_utf8(output.stderr).unwrap(),
    )
}

fn project() -> tempfile::TempDir {
    support::project_with_tasks(&[("alpha.typ", "Alpha")])
}

#[test]
fn go_without_an_editor_puts_the_fix_on_a_hint() {
    let (code, stderr) = run(project().path(), &["go", "alpha"]);

    assert_eq!(code, Some(1), "{stderr}");
    assert_eq!(
        stderr,
        "error: no editor configured\n\
         hint: set [go] editor in .mindtape/config.toml, or $VISUAL or $EDITOR\n"
    );
}

#[test]
fn eval_of_a_missing_path_quotes_it() {
    support::assert_stderr(
        project().path(),
        &["eval", "nope/missing.typ"],
        1,
        "error: no such file: 'nope/missing.typ'\n",
    );
}

#[test]
fn an_unknown_worktree_alias_is_quoted_with_a_hint() {
    let (code, stderr) = run(project().path(), &["@nope", "ls"]);

    assert_eq!(code, Some(2), "{stderr}");
    assert!(
        stderr.starts_with(
            "error: unknown worktree: '@nope'\nhint: add it to worktrees.toml\nnote: worktrees are configured in: '"
        ),
        "{stderr}"
    );
}
