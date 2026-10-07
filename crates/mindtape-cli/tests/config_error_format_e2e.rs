//! A config that fails to load is reported in the error format: the project
//! file and the line of the key at fault lead, a value is single-quoted, and
//! a near match is a `hint:` line.
#![allow(clippy::unwrap_used, clippy::disallowed_methods)]

mod support;

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
