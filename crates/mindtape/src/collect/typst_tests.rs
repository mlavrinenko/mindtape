#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::disallowed_methods
)]

use crate::config::Config;
use crate::config::layers::load_config_layered_from;
use crate::model::Task;
use crate::test_support::{home_and_project, write_file};
use crate::validate::{ViolationKind, validate};

const LIB: &str = r#"
#let title(task) = task.title
#let field(value) = value
#let paths(task) = ("src/a.rs", "src/b.rs")
#let summary(task) = (count: 2, files: ("src/a.rs",))
#let wrong(task) = 3
#let boom(task) = panic("boom")
"#;

fn project(config: &str) -> (tempfile::TempDir, Config) {
    let (tmp, home, project) = home_and_project();
    write_file(&project, "lib.typ", LIB);
    write_file(&project, ".mindtape/config.toml", config);
    let config = load_config_layered_from(&project, Some(&home), None)
        .unwrap()
        .0;
    (tmp, config)
}

fn task() -> Task {
    Task {
        id: "t".to_owned(),
        title: "The title".to_owned(),
        body: "== Changes\n\n- `src/a.rs`\n".to_owned(),
        ..Task::default()
    }
}

fn strings(task: &Task, name: &str) -> Vec<String> {
    task.derived[name]
        .as_array()
        .unwrap()
        .iter()
        .map(|value| value.as_str().unwrap().to_owned())
        .collect()
}

#[test]
fn a_typst_collector_reads_the_whole_task_or_one_field() {
    let (_tmp, config) = project(
        r#"
[tasks]
roots = ["tasks"]

[[tasks.collectors]]
name = "title"
query = { typst = "lib.typ", function = "title" }
returns = "string"

[[tasks.collectors]]
name = "body"
query = { typst = "lib.typ", function = "field", field = "body" }
returns = "string"
"#,
    );
    let mut tasks = vec![task()];
    super::annotate(&mut tasks, &config);
    assert!(
        tasks[0].collector_problems.is_empty(),
        "{:?}",
        tasks[0].collector_problems
    );
    assert_eq!(
        tasks[0].derived["title"],
        typst_harvest::HVal::Str("The title".to_owned())
    );
    assert_eq!(
        tasks[0].derived["body"],
        typst_harvest::HVal::Str("== Changes\n\n- `src/a.rs`\n".to_owned())
    );
}

#[test]
fn a_typst_collector_checks_a_record_and_a_list() {
    let (_tmp, config) = project(
        r#"
[tasks]
roots = ["tasks"]

[[tasks.collectors]]
name = "files"
query = { typst = "lib.typ", function = "paths" }
returns = ["path"]

[[tasks.collectors]]
name = "summary"
query = { typst = "lib.typ", function = "summary" }
returns = { count = "int", files = ["path"] }
"#,
    );
    let mut tasks = vec![task()];
    super::annotate(&mut tasks, &config);
    assert_eq!(strings(&tasks[0], "files"), ["src/a.rs", "src/b.rs"]);
    assert!(tasks[0].derived["summary"].get("count").is_some());
}

#[test]
fn a_wrong_typed_value_is_a_problem_and_a_collector_violation() {
    let (_tmp, config) = project(
        r#"
[tasks]
roots = ["tasks"]

[[tasks.collectors]]
name = "wrong"
query = { typst = "lib.typ", function = "wrong" }
returns = "string"
"#,
    );
    let mut tasks = vec![task()];
    super::annotate(&mut tasks, &config);
    assert!(!tasks[0].derived.contains_key("wrong"));
    let problem = tasks[0].collector_problems[0].text.clone();
    assert!(problem.contains("derived.wrong"), "{problem}");
    assert!(problem.contains("lib.typ"), "{problem}");
    let violations = validate(&tasks, &config);
    let violation = violations
        .iter()
        .find(|violation| violation.kind == ViolationKind::Collector)
        .unwrap();
    assert_eq!(violation.field, "derived.wrong");
    assert!(violation.is_fatal());
}

#[test]
fn a_panic_is_a_problem_naming_the_module_and_function() {
    let (_tmp, config) = project(
        r#"
[tasks]
roots = ["tasks"]

[[tasks.collectors]]
name = "boom"
query = { typst = "lib.typ", function = "boom" }
returns = "string"
"#,
    );
    let mut tasks = vec![task()];
    super::annotate(&mut tasks, &config);
    let problem = tasks[0].collector_problems[0].text.clone();
    assert!(problem.contains("boom in lib.typ"), "{problem}");
    assert!(problem.contains("derived.boom"), "{problem}");
}

#[test]
fn a_missing_function_is_refused_at_load() {
    let (_tmp, home, project) = home_and_project();
    write_file(&project, "lib.typ", LIB);
    write_file(
        &project,
        ".mindtape/config.toml",
        r#"
[[tasks.collectors]]
name = "missing"
query = { typst = "lib.typ", function = "nope" }
returns = "string"
"#,
    );
    let err = load_config_layered_from(&project, Some(&home), None).unwrap_err();
    assert!(err.to_string().contains("does not export `nope`"), "{err}");
    assert!(err.to_string().contains("tasks.collectors[0]"), "{err}");
}

#[test]
fn a_pack_collector_resolves_its_module_from_the_pack_directory() {
    let (_tmp, home, project) = home_and_project();
    write_file(
        &project,
        "packs/team/pack.toml",
        r#"
[[tasks.collectors]]
name = "title"
query = { typst = "lib.typ", function = "title" }
returns = "string"
"#,
    );
    write_file(
        &project,
        "packs/team/lib.typ",
        "#let title(task) = task.title\n",
    );
    write_file(
        &project,
        ".mindtape/config.toml",
        "extends = [\"./packs/team/pack.toml\"]\n",
    );
    let config = load_config_layered_from(&project, Some(&home), None)
        .unwrap()
        .0;
    assert!(config.tasks.collectors[0].root.is_some());
    let mut tasks = vec![task()];
    super::annotate(&mut tasks, &config);
    assert_eq!(
        tasks[0].derived["title"],
        typst_harvest::HVal::Str("The title".to_owned())
    );
}

#[test]
fn a_collector_outside_the_project_root_is_refused_at_load() {
    let (_tmp, home, project) = home_and_project();
    write_file(&project, "lib.typ", LIB);
    write_file(
        &project,
        ".mindtape/config.toml",
        r#"
[[tasks.collectors]]
name = "missing"
query = { typst = "nope.typ", function = "title" }
returns = "string"
"#,
    );
    let err = load_config_layered_from(&project, Some(&home), None).unwrap_err();
    assert!(err.to_string().contains("does not exist"), "{err}");
}
