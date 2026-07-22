#![allow(clippy::unwrap_used)]

use std::path::PathBuf;

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
use mindtape::test_support::bare_task;
use mindtape::{Config, Readiness, Task};

use super::{Intent, State, classify, collect_first, footer, guard, run};
use crate::ls::LsArgs;
use crate::output::OutputFormat;
use crate::util;

fn key(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
    KeyEvent {
        code,
        modifiers,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    }
}

fn state(n: usize, buffer: &str) -> State {
    let tasks: Vec<Task> = (0..n).map(|i| bare_task(&format!("t{i}"))).collect();
    State {
        config: Config::default(),
        root: ".".into(),
        tasks,
        readiness: Readiness::empty(),
        buffer: buffer.to_owned(),
    }
}

#[test]
fn classify_digits_and_verbs() {
    assert_eq!(
        classify(key(KeyCode::Char('3'), KeyModifiers::NONE)),
        Intent::Digit('3')
    );
    assert_eq!(
        classify(key(KeyCode::Char('q'), KeyModifiers::NONE)),
        Intent::Quit
    );
    assert_eq!(
        classify(key(KeyCode::Char(' '), KeyModifiers::NONE)),
        Intent::Refresh
    );
    assert_eq!(
        classify(key(KeyCode::Backspace, KeyModifiers::NONE)),
        Intent::DelLast
    );
    assert_eq!(
        classify(key(KeyCode::Delete, KeyModifiers::NONE)),
        Intent::Clear
    );
    assert_eq!(
        classify(key(KeyCode::Enter, KeyModifiers::NONE)),
        Intent::OpenFirst
    );
}

#[test]
fn classify_ctrl_modifiers() {
    assert_eq!(
        classify(key(KeyCode::Char('c'), KeyModifiers::CONTROL)),
        Intent::Quit
    );
    assert_eq!(
        classify(key(KeyCode::Enter, KeyModifiers::CONTROL)),
        Intent::OpenAll
    );
    assert_eq!(
        classify(key(KeyCode::Backspace, KeyModifiers::CONTROL)),
        Intent::Clear
    );
}

#[test]
fn classify_unknown_is_ignored() {
    assert_eq!(
        classify(key(KeyCode::Char('x'), KeyModifiers::NONE)),
        Intent::Ignore
    );
    assert_eq!(
        classify(key(KeyCode::Up, KeyModifiers::NONE)),
        Intent::Ignore
    );
}

#[test]
fn empty_buffer_highlights_nothing() {
    let state = state(5, "");
    assert_eq!(state.highlight_mask(), vec![false; 5]);
    assert_eq!(state.first_highlighted(), None);
}

#[test]
fn prefix_selects_by_ordinal() {
    // 12 tasks, buffer "1" → ordinals 1, 10, 11, 12 (indices 0, 9, 10, 11).
    let state = state(12, "1");
    let mask = state.highlight_mask();
    let on: Vec<usize> = mask
        .iter()
        .enumerate()
        .filter_map(|(i, &on)| on.then_some(i))
        .collect();
    assert_eq!(on, vec![0, 9, 10, 11]);
    // First highlighted is the topmost (ordinal 1, index 0).
    assert_eq!(state.first_highlighted(), Some(0));
}

#[test]
fn two_digit_prefix_narrows() {
    let state = state(12, "12");
    assert_eq!(state.first_highlighted(), Some(11));
    assert_eq!(state.highlight_mask().iter().filter(|&&on| on).count(), 1);
}

#[test]
fn nonmatching_buffer_highlights_nothing() {
    let state = state(5, "9");
    assert_eq!(state.first_highlighted(), None);
}

#[test]
fn footer_shows_buffer_when_typing() {
    assert!(footer("12").contains("#12"));
    // Empty buffer drops the status line, keeping only the hints.
    assert!(!footer("").contains('#'));
}

fn ls_args(format: OutputFormat) -> LsArgs {
    LsArgs {
        filter: None,
        format,
        json: false,
        limit: None,
        sort: vec![],
        columns: None,
        interactive: true,
    }
}

// --- guard / run ----------------------------------------------------------

#[test]
fn guard_rejects_non_table() {
    assert!(guard(&ls_args(OutputFormat::Json)).is_err());
}

#[test]
fn guard_table_fails_on_tty_not_format() {
    let err = guard(&ls_args(OutputFormat::Table)).unwrap_err();
    assert!(err.to_string().contains("TTY"), "got: {err}");
}

#[test]
fn run_fails_on_non_table() {
    let p = util::Project {
        root: "/nonexistent".into(),
        config: Config::default(),
        outcome: Default::default(),
    };
    assert!(run(&ls_args(OutputFormat::Json), p).is_err());
}

// --- classify ctrl --------------------------------------------------------

#[test]
fn classify_plain_c_no_ctrl_is_ignore() {
    assert_eq!(
        classify(key(KeyCode::Char('c'), KeyModifiers::NONE)),
        Intent::Ignore
    );
}

// --- collect_first --------------------------------------------------------

#[test]
fn collect_first_match_returns_correct_index() {
    assert_eq!(collect_first(&state(10, "3")), vec![2]);
}

// --- State::paths ---------------------------------------------------------

#[test]
fn paths_returns_absolute_task_paths() {
    assert_eq!(
        state(3, "").paths(&[0]),
        vec![PathBuf::from("./tasks/t0.typ")]
    );
}

// --- State::refresh -------------------------------------------------------

const CHILD: &str = "_MT_LS_INTERACTIVE_CHILD";

/// A task in the harvested-metadata form, so the fixture needs no prelude
/// package on disk and no Typst import resolution.
const TASK_METADATA: &str = r#"#metadata((
  marker: "mindtape.task",
  fields: (
    title: "Refreshed",
    status: (name: "wip", date: datetime(year: 2026, month: 6, day: 1), note: none),
    priority: 5,
  ),
  extras: (:),
))
"#;

/// `refresh` rediscovers the project from the current working directory, so it
/// can only be exercised against a fixture by changing the cwd — which is
/// process-global. It therefore runs in a child: the previous version of this
/// test resolved the root from wherever the suite happened to be running,
/// Typst-evaluated `MindTape`'s own `truth/tasks/` backlog, and asserted only
/// that the result was non-empty. That passed for the wrong reason (any
/// backlog anywhere) and broke the moment the suite ran from another directory.
#[test]
fn refresh_loads_tasks_from_project() {
    if std::env::var(CHILD).is_err() {
        let out = crate::test_util::run_self_test("refresh_loads_tasks_from_project", CHILD);
        crate::test_util::assert_child_ok(&out);
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join(".mindtape")).unwrap();
    std::fs::create_dir_all(dir.path().join("tasks")).unwrap();
    std::fs::write(
        dir.path().join(".mindtape/config.toml"),
        "[tasks]\nroots = [\"tasks\"]\n",
    )
    .unwrap();
    std::fs::write(dir.path().join("tasks/refreshed.typ"), TASK_METADATA).unwrap();
    std::env::set_current_dir(dir.path()).unwrap();

    let mut st = state(0, "5");
    st.refresh(&ls_args(OutputFormat::Table))
        .expect("refresh should load the fixture project");
    assert_eq!(
        st.tasks.first().map(|task| task.title.as_str()),
        Some("Refreshed"),
        "refresh should load exactly the fixture's one task"
    );
    assert_eq!(st.tasks.len(), 1, "and only that task");
    assert!(
        st.buffer.is_empty(),
        "refresh should clear the selection buffer"
    );
}
