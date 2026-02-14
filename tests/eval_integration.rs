use mindtape::eval::{eval_file, Task};
use mindtape::world::MindTapeWorld;
use typst::foundations::Datetime;

/// Create a temp project dir with a `.typ` file, evaluate it, return tasks.
fn eval_typ(source: &str) -> Result<Vec<Task>, String> {
    let dir = tempfile::tempdir().unwrap();
    // Create a Cargo.toml marker so find_project_root stops here
    std::fs::write(dir.path().join("Cargo.toml"), "").unwrap();
    let file = dir.path().join("test.typ");
    std::fs::write(&file, source).unwrap();
    let world = MindTapeWorld::new(&file).map_err(|e| e.to_string())?;
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
    let world = MindTapeWorld::new(&file).map_err(|e| e.to_string())?;
    eval_file(&world)
}

fn ymd(y: i32, m: u8, d: u8) -> Datetime {
    Datetime::from_ymd(y, m, d).unwrap()
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
