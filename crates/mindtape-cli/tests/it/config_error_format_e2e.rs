//! A config that fails to load is reported in the error format: the project
//! file and the line of the key at fault lead, a value is single-quoted, and
//! a near match is a `hint:` line.
#![allow(clippy::unwrap_used, clippy::disallowed_methods)]

use crate::support;

/// The stderr of `mt ls` over a project whose config is `config`.
fn ls_stderr(config: &str) -> String {
    let dir = support::scaffold_project(config);
    let output = support::mt(dir.path()).arg("ls").output().unwrap();
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    String::from_utf8(output.stderr).unwrap()
}

#[test]
fn a_bad_view_sort_is_reported_at_its_line() {
    let stderr =
        ls_stderr("[tasks]\nroots = [\"tasks\"]\n\n[views.broken]\nsort = [\"nofield\"]\n");

    assert_eq!(
        stderr,
        "error: .mindtape/config.toml:5: views.broken.sort: unknown sort field: 'nofield'\n"
    );
}

#[test]
fn an_unknown_pack_is_quoted_and_names_the_near_match() {
    let stderr = ls_stderr("extends = [\"mindtape:task\"]\n");

    assert!(
        stderr.starts_with(
            "error: .mindtape/config.toml:1: extends: unknown pack: 'mindtape:task'\n\
             hint: did you mean mindtape:tasks?\n\
             hint: use one of: "
        ),
        "{stderr}"
    );
}

#[test]
fn a_hook_title_on_a_run_action_says_which_kind_takes_it() {
    let stderr = ls_stderr(
        "[tasks]\nroots = [\"tasks\"]\n\n[[on.flip]]\nwhen = \"\"\n\
         actions = [{ run = \"true\", title = \"x\" }]\n",
    );

    assert_eq!(
        stderr,
        "error: .mindtape/config.toml:6: on.flip[0].actions[0]: 'title' applies only to list actions\n"
    );
}
