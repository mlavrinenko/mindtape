//! `mt ls` over a project with a task file that fails to load: it warns
//! naming the file, lists the rest, counts the broken file last in the line
//! under the table, and exits 0.
#![allow(clippy::unwrap_used, clippy::disallowed_methods)]

use crate::support;

/// A task whose `title` is misspelt, so the prelude refuses the call.
const MISSPELT: &str = "#import \"@local/mindtape:0.2.0\": *\n\n\
                        #show: task.with(\n  titel: \"Gamma\",\n  status: wip(2026, 6, 1),\n)\n";

/// The line under the table when one of the two tasks that loaded matches.
const FOOTER: &str = "\nyour filter · 1 of 2 tasks · 1 broken\n";

/// Two tasks that load and `gamma.typ`, which does not.
fn project() -> tempfile::TempDir {
    let dir = support::project_with_tasks(&[("alpha.typ", "Alpha"), ("beta.typ", "Beta")]);
    std::fs::write(dir.path().join("tasks/gamma.typ"), MISSPELT).unwrap();
    dir
}

#[test]
fn ls_warns_and_counts_the_broken_file_last() {
    let dir = project();
    let ls = ["ls", "contains(\"Alpha\")"];
    let (_, stderr) = support::run_stderr(dir.path(), &ls);
    assert!(stderr.starts_with("warning: tasks/gamma.typ:"), "{stderr}");
    let stdout = support::stdout(dir.path(), &ls);
    assert!(stdout.ends_with(FOOTER), "{stdout}");
}

#[test]
fn find_counts_the_broken_file_too() {
    let stdout = support::stdout(project().path(), &["find", "Beta"]);
    assert!(stdout.ends_with(FOOTER), "{stdout}");
}
