#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use mindtape::eval::{eval_file, eval_file_full, Task};
use mindtape::world::MindTapeWorld;
use typst::foundations::Datetime;

/// Create a temp project dir with a `.typ` file, evaluate it, return tasks.
fn eval_typ(source: &str) -> Result<Vec<Task>, String> {
    let dir = tempfile::tempdir().unwrap();
    // Create a Cargo.toml marker so find_project_root stops here
    std::fs::write(dir.path().join("Cargo.toml"), "").unwrap();
    let file = dir.path().join("test.typ");
    std::fs::write(&file, source).unwrap();
    let world = MindTapeWorld::new(&file).map_err(|e| e.clone())?;
    eval_file(&world)
}

/// Create a temp project with a prelude and a `.typ` file that imports it.
fn eval_typ_with_prelude(prelude: &str, source: &str) -> Result<Vec<Task>, String> {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("Cargo.toml"), "").unwrap();
    let lib_dir = dir.path().join("lib");
    std::fs::create_dir(&lib_dir).unwrap();
    std::fs::write(lib_dir.join("prelude.typ"), prelude).unwrap();
    let file = dir.path().join("test.typ");
    std::fs::write(&file, source).unwrap();
    let world = MindTapeWorld::new(&file).map_err(|e| e.clone())?;
    eval_file(&world)
}

/// Create a temp project with `@mindtape` package (lib/typst.toml + lib/prelude.typ)
/// and a `.typ` file that can use `#import "@mindtape/mindtape:0.1.0": ...`.
fn eval_typ_with_package(prelude: &str, source: &str) -> Result<Vec<Task>, String> {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("Cargo.toml"), "").unwrap();
    let lib_dir = dir.path().join("lib");
    std::fs::create_dir(&lib_dir).unwrap();
    std::fs::write(lib_dir.join("prelude.typ"), prelude).unwrap();
    std::fs::write(
        lib_dir.join("typst.toml"),
        "[package]\nname = \"mindtape\"\nversion = \"0.1.0\"\nentrypoint = \"prelude.typ\"\n",
    )
    .unwrap();
    let file = dir.path().join("test.typ");
    std::fs::write(&file, source).unwrap();
    let world = MindTapeWorld::new(&file).map_err(|e| e.clone())?;
    eval_file(&world)
}

/// Create a temp project dir with a `.typ` file, evaluate it, return full result.
fn eval_typ_full(source: &str) -> Result<mindtape::eval::EvalResult, String> {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("Cargo.toml"), "").unwrap();
    let file = dir.path().join("test.typ");
    std::fs::write(&file, source).unwrap();
    let world = MindTapeWorld::new(&file).map_err(|e| e.clone())?;
    eval_file_full(&world)
}

fn ymd(year: i32, month: u8, day: u8) -> Datetime {
    Datetime::from_ymd(year, month, day).unwrap()
}

#[test]
fn eval_empty_file() {
    let tasks = eval_typ("").unwrap();
    assert!(tasks.is_empty());
}

#[test]
fn eval_heading_only() {
    let tasks = eval_typ("= Just a heading").unwrap();
    assert!(tasks.is_empty());
}

#[test]
fn eval_unchecked_task() {
    let tasks = eval_typ("- [ ] Buy milk").unwrap();
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].title, "Buy milk");
    assert!(!tasks[0].done);
    assert_eq!(tasks[0].due, None);
}

#[test]
fn eval_checked_task() {
    let tasks = eval_typ("- [x] Done thing").unwrap();
    assert_eq!(tasks.len(), 1);
    assert!(tasks[0].done);
    assert_eq!(tasks[0].title, "Done thing");
}

#[test]
fn eval_plain_list_item_not_a_task() {
    let tasks = eval_typ("- Not a task").unwrap();
    assert!(tasks.is_empty());
}

#[test]
fn eval_multiple_tasks() {
    let tasks = eval_typ(
        "- [ ] First\n- [x] Second\n- [ ] Third",
    )
    .unwrap();
    assert_eq!(tasks.len(), 3);
    assert_eq!(tasks[0].title, "First");
    assert_eq!(tasks[1].title, "Second");
    assert_eq!(tasks[2].title, "Third");
}

#[test]
fn eval_task_with_inline_metadata_due() {
    let tasks = eval_typ(
        r#"- [ ] Deadline task #metadata(("due", datetime(year: 2026, month: 5, day: 1)))"#,
    )
    .unwrap();
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].title, "Deadline task");
    assert_eq!(tasks[0].due, Some(ymd(2026, 5, 1)));
}

#[test]
fn eval_mixed_content() {
    let source = r#"
= Project Title

Some paragraph text.

- [ ] A real task
- Not a task
- [x] Done task

#let note = "just a binding"
"#;
    let tasks = eval_typ(source).unwrap();
    assert_eq!(tasks.len(), 2);
    assert_eq!(tasks[0].title, "A real task");
    assert_eq!(tasks[1].title, "Done task");
}

#[test]
fn eval_task_with_prelude_import() {
    let prelude = r#"#let due(date) = metadata(("due", date))"#;
    let source = r#"#import "lib/prelude.typ": due

- [ ] Learn piano #due(datetime(year: 2026, month: 4, day: 1))
"#;
    let tasks = eval_typ_with_prelude(prelude, source).unwrap();
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].title, "Learn piano");
    assert_eq!(tasks[0].due, Some(ymd(2026, 4, 1)));
}

// --- tag extraction ---

#[test]
fn eval_task_with_single_tag() {
    let tasks = eval_typ(
        r#"- [ ] Urgent thing #metadata(("tag", "urgent"))"#,
    )
    .unwrap();
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].tags, vec!["urgent"]);
}

#[test]
fn eval_task_with_multiple_tags() {
    let tasks = eval_typ(
        r#"- [ ] Multi-tagged #metadata(("tag", "work")) #metadata(("tag", "urgent"))"#,
    )
    .unwrap();
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].tags, vec!["work", "urgent"]);
}

#[test]
fn eval_task_without_tags_has_empty_vec() {
    let tasks = eval_typ("- [ ] Plain task").unwrap();
    assert_eq!(tasks.len(), 1);
    assert!(tasks[0].tags.is_empty());
}

#[test]
fn eval_task_with_due_and_tags() {
    let tasks = eval_typ(
        r#"- [ ] Both #metadata(("due", datetime(year: 2026, month: 3, day: 1))) #metadata(("tag", "fun"))"#,
    )
    .unwrap();
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].due, Some(ymd(2026, 3, 1)));
    assert_eq!(tasks[0].tags, vec!["fun"]);
}

#[test]
fn eval_tag_via_prelude() {
    let prelude = r#"#let tag(name) = metadata(("tag", name))"#;
    let source = r#"#import "lib/prelude.typ": tag

- [ ] Tagged task #tag("hobby")
"#;
    let tasks = eval_typ_with_prelude(prelude, source).unwrap();
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].tags, vec!["hobby"]);
}

// --- @mindtape package imports ---

#[test]
fn eval_package_import_due() {
    let prelude = r#"
#let due(date) = metadata(("due", date))
#let id(uuid) = metadata(("id", uuid))
#let tag(name) = metadata(("tag", name))
"#;
    let source = r#"#import "@mindtape/mindtape:0.1.0": due

- [ ] Package task #due(datetime(year: 2026, month: 7, day: 4))
"#;
    let tasks = eval_typ_with_package(prelude, source).unwrap();
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].title, "Package task");
    assert_eq!(tasks[0].due, Some(ymd(2026, 7, 4)));
}

#[test]
fn eval_package_import_tag() {
    let prelude = r#"
#let due(date) = metadata(("due", date))
#let id(uuid) = metadata(("id", uuid))
#let tag(name) = metadata(("tag", name))
"#;
    let source = r#"#import "@mindtape/mindtape:0.1.0": tag

- [ ] Package tagged #tag("priority")
"#;
    let tasks = eval_typ_with_package(prelude, source).unwrap();
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].tags, vec!["priority"]);
}

#[test]
fn eval_package_import_all_functions() {
    let prelude = r#"
#let due(date) = metadata(("due", date))
#let id(uuid) = metadata(("id", uuid))
#let tag(name) = metadata(("tag", name))
"#;
    let source = r#"#import "@mindtape/mindtape:0.1.0": due, tag

- [ ] Full featured #due(datetime(year: 2026, month: 12, day: 25)) #tag("holiday") #tag("fun")
"#;
    let tasks = eval_typ_with_package(prelude, source).unwrap();
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].title, "Full featured");
    assert_eq!(tasks[0].due, Some(ymd(2026, 12, 25)));
    assert_eq!(tasks[0].tags, vec!["holiday", "fun"]);
}

// --- eval_file_full: title extraction ---

#[test]
fn eval_full_extracts_title_from_heading() {
    let result = eval_typ_full("= My Project\n\n- [ ] A task").unwrap();
    assert_eq!(result.title, Some("My Project".to_string()));
}

#[test]
fn eval_full_no_heading_returns_none_title() {
    let result = eval_typ_full("- [ ] Just a task").unwrap();
    assert_eq!(result.title, None);
}

#[test]
fn eval_full_uses_first_heading_only() {
    let result = eval_typ_full("= First\n\n= Second\n\n- [ ] task").unwrap();
    assert_eq!(result.title, Some("First".to_string()));
}

// --- eval_file_full: bindings extraction ---

#[test]
fn eval_full_extracts_string_binding() {
    let result = eval_typ_full(r#"#let note = "hello world""#).unwrap();
    assert_eq!(result.bindings.len(), 1);
    assert_eq!(result.bindings[0].0, "note");
    assert_eq!(result.bindings[0].1, "string");
    assert_eq!(result.bindings[0].2, "\"hello world\"");
}

#[test]
fn eval_full_extracts_int_binding() {
    let result = eval_typ_full("#let count = 42").unwrap();
    let binding = result.bindings.iter().find(|b| b.0 == "count").unwrap();
    assert_eq!(binding.1, "int");
    assert_eq!(binding.2, "42");
}

#[test]
fn eval_full_extracts_bool_binding() {
    let result = eval_typ_full("#let active = true").unwrap();
    let binding = result.bindings.iter().find(|b| b.0 == "active").unwrap();
    assert_eq!(binding.1, "bool");
    assert_eq!(binding.2, "true");
}

#[test]
fn eval_full_skips_function_bindings() {
    let result = eval_typ_full(r#"
#let greet(name) = "hello " + name
#let note = "data"
"#).unwrap();
    // Only `note` should appear, not `greet`
    assert_eq!(result.bindings.len(), 1);
    assert_eq!(result.bindings[0].0, "note");
}

// --- eval_file_full: position tracking ---

#[test]
fn eval_full_assigns_positions() {
    let result = eval_typ_full("- [ ] First\n- [x] Second\n- [ ] Third").unwrap();
    assert_eq!(result.tasks.len(), 3);
    assert_eq!(result.tasks[0].position, 0);
    assert_eq!(result.tasks[1].position, 1);
    assert_eq!(result.tasks[2].position, 2);
}

#[test]
fn eval_full_positions_skip_non_tasks() {
    let result = eval_typ_full("- [ ] Task A\n- Not a task\n- [ ] Task B").unwrap();
    assert_eq!(result.tasks.len(), 2);
    assert_eq!(result.tasks[0].position, 0);
    assert_eq!(result.tasks[1].position, 1);
}

// --- eval_file_full: additional binding types ---

#[test]
fn eval_full_extracts_float_binding() {
    let result = eval_typ_full("#let ratio = 3.14").unwrap();
    let binding = result.bindings.iter().find(|b| b.0 == "ratio").unwrap();
    assert_eq!(binding.1, "float");
    assert_eq!(binding.2, "3.14");
}

#[test]
fn eval_full_extracts_datetime_binding() {
    let result = eval_typ_full("#let deadline = datetime(year: 2026, month: 6, day: 15)").unwrap();
    let binding = result.bindings.iter().find(|b| b.0 == "deadline").unwrap();
    assert_eq!(binding.1, "date");
    assert_eq!(binding.2, "\"2026-06-15\"");
}

#[test]
fn eval_full_extracts_none_binding() {
    let result = eval_typ_full("#let maybe = none").unwrap();
    let binding = result.bindings.iter().find(|b| b.0 == "maybe").unwrap();
    assert_eq!(binding.1, "none");
    assert_eq!(binding.2, "null");
}

#[test]
fn eval_full_skips_array_binding() {
    // Arrays are not a supported binding type — should be skipped
    let result = eval_typ_full(r#"#let items = ("a", "b", "c")"#).unwrap();
    assert!(!result.bindings.iter().any(|b| b.0 == "items"));
}

#[test]
fn eval_checked_task_uppercase_x() {
    let tasks = eval_typ("- [X] Done with uppercase").unwrap();
    assert_eq!(tasks.len(), 1);
    assert!(tasks[0].done);
    assert_eq!(tasks[0].title, "Done with uppercase");
}
