//! Throwaway projects the e2e tests start from: a scaffolded config, task
//! files, templates and git repositories, re-exported by [`super`].
#![allow(clippy::unwrap_used, clippy::disallowed_methods, dead_code)]

use super::{PRELUDE, run_ok};
use mindtape::test_support::tempdir;
use mindtape::test_support::write_file;
use std::path::Path;

/// A template whose slots span both sides of the escaping decision: a `string`
/// title and an `int` priority render as literals and must stay verbatim, while
/// the `text` slots land in markup and must be escaped. `why` sits inside a link
/// note, the one markup position the harvest reads back as a value, so a test can
/// assert what a filled `text` slot actually means rather than only how it looks.
const SLOT_TEMPLATE: &str = mindtape::typst_task!(
    r#"(
  title: slot("title", prompt: "Task title", example: "Fix the login redirect"),
  priority: slot("weight", kind: "int", default: 5),
  links: related("anchor.typ")[#slot("why", kind: "text", example: "same session store")],
)

== Summary

#slot("summary", kind: "text", default: "- TODO")
"#
);

/// A throwaway project scaffolded with `source` as its default template
/// (`.mindtape/templates/task.typ`) — the shared prologue
/// [`project_with_slot_template`] and [`project_with_raw_slot_template`]
/// both build on before adding their own fixtures, and the one every test
/// about a template's *shape* (a single-line `task.with(…)`, a template that
/// cannot evaluate) reaches for directly.
pub fn project_with_template(source: &str) -> tempfile::TempDir {
    project_with_template_and_config("[tasks]\nroots = [\"tasks\"]\n", source)
}

/// [`project_with_template`], but with `config` written to
/// `.mindtape/config.toml` in place of the bare `[tasks] roots = ["tasks"]`
/// default — for a test whose default template must clear a rule the
/// project's own config declares (`[[tasks.body.headers.rules]]`,
/// `[[tasks.body.content.rules]]`).
pub fn project_with_template_and_config(config: &str, source: &str) -> tempfile::TempDir {
    let dir = tempdir();
    install_lib(dir.path());
    std::fs::create_dir_all(dir.path().join(".mindtape/templates")).unwrap();
    write_file(dir.path(), ".mindtape/config.toml", config);
    write_file(dir.path(), ".mindtape/templates/task.typ", source);
    dir
}

/// A throwaway project whose default template is [`SLOT_TEMPLATE`], with the
/// `anchor` task the template's `related` edge points at already filed.
pub fn project_with_slot_template() -> tempfile::TempDir {
    let dir = project_with_template(SLOT_TEMPLATE);
    write_file(
        dir.path(),
        "tasks/anchor.typ",
        mindtape::typst_task!("(title: \"anchor\", status: proposed(2026, 7, 22))\n"),
    );
    dir
}

/// A template with one `raw`-kind slot, the unescaped counterpart to
/// [`SLOT_TEMPLATE`]'s `text` slots — proves the escaping decision is a
/// per-slot choice the template author makes, not a per-command one.
const TYPST_SLOT_TEMPLATE: &str = mindtape::typst_task!(
    r#"(
  title: slot("title", prompt: "Task title", example: "Fix the login redirect"),
)

== Summary

#slot("summary", kind: "raw", default: "- `TODO`")
"#
);

/// A throwaway project whose default template is [`TYPST_SLOT_TEMPLATE`].
pub fn project_with_raw_slot_template() -> tempfile::TempDir {
    project_with_template(TYPST_SLOT_TEMPLATE)
}

/// A scaffolded project for `preset`, with the Typst package installed.
pub fn init_preset(preset: &str) -> tempfile::TempDir {
    let dir = tempdir();
    install_lib(dir.path());
    run_ok(dir.path(), &["init", "--preset", preset]);
    dir
}

/// [`scaffold_project`] with one task file per `(name, source)` pair dropped
/// into `tasks/`.
pub fn scaffold_config(config: &str, files: &[(&str, &str)]) -> tempfile::TempDir {
    let dir = scaffold_project(config);
    for (name, contents) in files {
        std::fs::write(dir.path().join("tasks").join(name), contents).unwrap();
    }
    dir
}

/// Run `git` in `dir` and assert it succeeded.
pub fn git(dir: &Path, args: &[&str]) {
    let status = std::process::Command::new("git")
        .current_dir(dir)
        .args(args)
        .status()
        .unwrap();
    assert!(status.success(), "git {args:?} failed");
}

/// Turn `dir` into a committed-clean git repository: init, a local identity so
/// the commit does not depend on the developer's own, and one seed commit of
/// everything present. `commit.gpgsign=false` keeps a signing setup out of it.
pub fn git_commit_all(dir: &Path) {
    git(dir, &["init", "-q"]);
    git(dir, &["config", "user.email", "t@example.com"]);
    git(dir, &["config", "user.name", "t"]);
    git(dir, &["add", "-A"]);
    git(
        dir,
        &["-c", "commit.gpgsign=false", "commit", "-q", "-m", "seed"],
    );
}

/// Install the in-tree `@local/mindtape` lib override and an empty `tasks/`
/// root under `dir`, so a task filed there can actually be evaluated.
pub fn install_lib(dir: &Path) {
    mindtape::test_support::write_lib_package(dir, PRELUDE);
    std::fs::create_dir_all(dir.join("tasks")).unwrap();
}

/// A temp project that can evaluate `@local/mindtape` via the
/// in-tree lib override the loader honours, with the barebones `[tasks]
/// roots = ["tasks"]` config the `add` e2e batteries build on before
/// overriding it with a scenario-specific one.
pub fn scaffold() -> tempfile::TempDir {
    scaffold_project(TASKS_ROOT_CONFIG)
}

/// A throwaway project directory: the in-tree `@local/mindtape` package
/// override plus an empty `tasks/` root ([`install_lib`]), and `config` written
/// to `.mindtape/config.toml`. The shared prologue for every e2e test that
/// needs a real project with a config of its own.
pub fn scaffold_project(config: &str) -> tempfile::TempDir {
    let dir = tempdir();
    install_lib(dir.path());
    std::fs::create_dir_all(dir.path().join(".mindtape")).unwrap();
    write_file(dir.path(), ".mindtape/config.toml", config);
    dir
}

/// `[tasks] roots = ["tasks"]` — the minimal config both [`project_with_tasks`]
/// and [`project_with_task`] scaffold onto via [`scaffold_project`].
const TASKS_ROOT_CONFIG: &str = "[tasks]\nroots = [\"tasks\"]\n";

/// A minimal real task source through the actual prelude: `title`, `wip`
/// since 2026-06-01, priority 3 — the plumbing [`project_with_tasks`] shares
/// across every fixture that needs several distinguishable real tasks.
fn task_source(title: &str) -> String {
    format!(
        "#import \"@local/mindtape:0.2.0\": *\n\n\
         #show: task.with(\n  title: \"{title}\",\n  status: wip(2026, 6, 1),\n  priority: 3,\n)\n"
    )
}

/// A throwaway project — [`scaffold_project`] with [`TASKS_ROOT_CONFIG`],
/// plus one real task per `(file, title)` pair, written under `tasks/`.
/// Unlike [`project_with_task`]'s single raw-metadata fixture, this goes
/// through the actual prelude and supports several tasks, for a test that
/// needs to tell them apart (`mt find`/`mt ls --format path` fixtures).
pub fn project_with_tasks(tasks: &[(&str, &str)]) -> tempfile::TempDir {
    let dir = scaffold_project(TASKS_ROOT_CONFIG);
    for (file, title) in tasks {
        std::fs::write(dir.path().join("tasks").join(file), task_source(title)).unwrap();
    }
    dir
}

/// A throwaway project — [`scaffold_project`] with [`TASKS_ROOT_CONFIG`] and
/// one `wip` task titled `title` — for tests that only need something to list.
pub fn project_with_task(title: &str) -> tempfile::TempDir {
    let dir = scaffold_project(TASKS_ROOT_CONFIG);
    write_file(
        dir.path(),
        "tasks/test.typ",
        format!(
            r#"#metadata((
  marker: "mindtape.task",
  fields: (
    title: "{title}",
    status: (name: "wip", date: datetime(year: 2026, month: 6, day: 1), note: none),
    priority: 5,
  ),
  extras: (:),
))
"#
        ),
    );
    dir
}

/// A committed-clean repository holding one `wip` task, `parser`, with
/// `rules` appended to its config — where every `mt check --commit-msg` test
/// starts.
pub fn commit_repo(rules: &str) -> tempfile::TempDir {
    let dir = project_with_tasks(&[("parser.typ", "Write the parser")]);
    let config = dir.path().join(".mindtape/config.toml");
    let base = std::fs::read_to_string(&config).unwrap();
    std::fs::write(&config, format!("{base}\n{rules}")).unwrap();
    git_commit_all(dir.path());
    dir
}
