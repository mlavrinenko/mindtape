#![allow(clippy::unwrap_used)]

use mindtape::test_support::tempdir_with_dirs;
use mindtape::test_support::write_file;
use std::path::PathBuf;

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
use mindtape::test_support::bare_task;
use mindtape::{Config, Readiness, Task};

use super::{Action, Intent, State, apply, classify, collect_first, footer, guard_with, run};
use crate::cmd::ls::{LsArgs, LsFlags};
use crate::render::output::{OutputFormat, Truncation};
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
        hashes: util::stem_hashes(&tasks, &Config::default()),
        tasks,
        readiness: Readiness::empty(),
        env: mindtape::ConditionEnv::default(),
        truncated: None,
        buffer: buffer.to_owned(),
    }
}

/// The stem hash row `index` of `state(n, _)` prints.
fn hash_at(n: usize, index: usize) -> String {
    let state = state(n, "");
    let task = state.tasks.get(index).unwrap();
    state.hashes.hash_of(&task.id).unwrap().to_owned()
}

/// The rows whose stem hash starts with `prefix`, in listing order.
fn rows_starting(n: usize, prefix: &str) -> Vec<usize> {
    (0..n)
        .filter(|&i| hash_at(n, i).starts_with(prefix))
        .collect()
}

#[test]
fn classify_digits_and_verbs() {
    assert_eq!(
        classify(key(KeyCode::Char('3'), KeyModifiers::NONE)),
        Intent::Digit('3')
    );
    // Every letter is a hash digit, `q` included; Esc quits.
    assert_eq!(
        classify(key(KeyCode::Char('q'), KeyModifiers::NONE)),
        Intent::Digit('q')
    );
    assert_eq!(
        classify(key(KeyCode::Esc, KeyModifiers::NONE)),
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
fn classify_lowercases_a_typed_digit_and_leaves_reading_it_to_the_library() {
    for (typed, read) in [('K', 'k'), ('I', 'i'), ('l', 'l'), ('O', 'o')] {
        assert_eq!(
            classify(key(KeyCode::Char(typed), KeyModifiers::NONE)),
            Intent::Digit(read)
        );
    }
}

#[test]
fn classify_unknown_is_ignored() {
    assert_eq!(
        classify(key(KeyCode::Char('-'), KeyModifiers::NONE)),
        Intent::Ignore
    );
    assert_eq!(
        classify(key(KeyCode::Up, KeyModifiers::NONE)),
        Intent::Ignore
    );
}

#[test]
fn apply_digit_appends_to_buffer() {
    let mut state = state(3, "");
    assert_eq!(apply(&mut state, Intent::Digit('2')), Action::Continue);
    assert_eq!(state.buffer, "2");
}

#[test]
fn apply_del_last_pops_buffer() {
    let mut state = state(3, "12");
    assert_eq!(apply(&mut state, Intent::DelLast), Action::Continue);
    assert_eq!(state.buffer, "1");
}

#[test]
fn apply_clear_empties_buffer() {
    let mut state = state(3, "12");
    assert_eq!(apply(&mut state, Intent::Clear), Action::Continue);
    assert_eq!(state.buffer, "");
}

#[test]
fn apply_refresh_and_quit_are_passthrough() {
    assert_eq!(apply(&mut state(3, ""), Intent::Refresh), Action::Refresh);
    assert_eq!(apply(&mut state(3, ""), Intent::Quit), Action::Quit);
}

#[test]
fn apply_ignore_keeps_buffer() {
    let mut state = state(3, "1");
    assert_eq!(apply(&mut state, Intent::Ignore), Action::Continue);
    assert_eq!(state.buffer, "1");
}

#[test]
fn apply_open_first_targets_the_first_highlighted() {
    assert_eq!(
        apply(&mut state(10, &hash_at(10, 2)), Intent::OpenFirst),
        Action::Open(vec![2])
    );
}

#[test]
fn apply_open_first_without_a_match_opens_nothing() {
    assert_eq!(
        apply(&mut state(3, "u"), Intent::OpenFirst),
        Action::Open(vec![])
    );
}

#[test]
fn apply_open_all_targets_every_task() {
    assert_eq!(
        apply(&mut state(3, ""), Intent::OpenAll),
        Action::Open(vec![0, 1, 2])
    );
}

#[test]
fn empty_buffer_highlights_nothing() {
    let state = state(5, "");
    assert_eq!(state.highlight_mask(), vec![false; 5]);
    assert_eq!(state.first_highlighted(), None);
}

#[test]
fn prefix_selects_by_stem_hash() {
    // The first digit of the last row's hash; with 100 rows of 32 possible
    // digits, rows of other hashes share it too.
    let prefix = hash_at(100, 99).get(..1).unwrap().to_owned();
    let state = state(100, &prefix);
    let mask = state.highlight_mask();
    let on: Vec<usize> = mask
        .iter()
        .enumerate()
        .filter_map(|(i, &on)| on.then_some(i))
        .collect();
    assert_eq!(on, rows_starting(100, &prefix));
    assert!(on.len() > 1, "the fixture must share a first digit: {on:?}");
    assert_eq!(state.first_highlighted(), on.first().copied());
}

#[test]
fn a_whole_hash_narrows_to_its_row() {
    let state = state(100, &hash_at(100, 99));
    assert_eq!(state.first_highlighted(), Some(99));
    assert_eq!(state.highlight_mask().iter().filter(|&&on| on).count(), 1);
}

#[test]
fn nonmatching_buffer_highlights_nothing() {
    // `u` is no hash digit.
    let state = state(5, "u");
    assert_eq!(state.first_highlighted(), None);
}

#[test]
fn footer_shows_buffer_when_typing() {
    assert!(footer("1a", None).contains("slug#: 1a"));
    // Empty buffer drops the status line, keeping only the hints.
    assert!(!footer("", None).contains("slug#:"));
}

#[test]
fn footer_reports_a_truncated_listing() {
    // The picker can only open what it lists, so `-n` hiding rows has to
    // reach the footer — otherwise a caller who capped the view reads an
    // absent task as a missing one.
    let cut = Truncation::new(Some(3), 0, 40).unwrap();
    let text = footer("", Some(cut));
    assert!(text.contains("37 more not shown"), "got: {text}");
    // The hints keep the last line: the truncation note is prepended, never
    // printed instead of the keys the caller needs.
    assert!(text.lines().next_back().unwrap().contains("Esc quit"));
}

#[test]
fn footer_stays_silent_on_a_complete_listing() {
    assert!(!footer("", None).contains("not shown"));
}

fn ls_args(format: OutputFormat) -> LsArgs {
    LsArgs {
        filter: None,
        flags: LsFlags {
            format,
            json: false,
            limit: None,
            skip: None,
            sort: vec![],
            columns: None,
            interactive: true,
        },
    }
}

// --- guard / run ----------------------------------------------------------

/// The format check runs before the TTY check, so a non-table format is
/// refused even on a real terminal.
#[test]
fn guard_rejects_non_table() {
    assert!(guard_with(&ls_args(OutputFormat::Json), true).is_err());
}

/// Table format with no TTY refuses, and names the terminal rather than the format.
#[test]
fn guard_table_fails_on_tty_not_format() {
    let err = guard_with(&ls_args(OutputFormat::Table), false).unwrap_err();
    crate::diag::assert_plain(
        &err,
        "error: --interactive needs a terminal on stdin and stdout\n\
         hint: drop --interactive when piping\n",
    );
}

/// Table format with a TTY is the one combination interactive can serve.
#[test]
fn guard_allows_table_on_a_tty() {
    assert!(guard_with(&ls_args(OutputFormat::Table), true).is_ok());
}

#[test]
fn from_project_honours_a_views_columns() {
    // The picker renders the same table `ls` prints, so a `@view`'s columns
    // have to reach it. They did not: the state kept the raw project config
    // and dropped the one the resolved listing folds them into, so `mt ls
    // @cols -i` drew columns `mt ls @cols` would not.
    let config =
        mindtape::parse_config("[views.cols]\nfilter = ''\ncolumns = ['status', 'title']\n")
            .unwrap();
    let (_dir, project) =
        crate::test_util::make_project(&[("x.typ", mindtape::test_support::TASK_SRC)], config);
    let args = LsArgs {
        filter: Some("@cols".to_owned()),
        ..ls_args(OutputFormat::Table)
    };
    let state = State::from_project(project, &args).unwrap();
    assert_eq!(state.config.ls.columns_order, vec!["status", "title"]);
}

#[test]
fn from_project_carries_the_truncation_into_the_state() {
    let (_dir, project) = crate::test_util::make_project(
        &[
            ("a.typ", mindtape::test_support::TASK_SRC),
            ("b.typ", mindtape::test_support::TASK_SRC),
        ],
        Config::default(),
    );
    let mut args = ls_args(OutputFormat::Table);
    args.flags.limit = Some(1);
    let state = State::from_project(project, &args).unwrap();
    assert_eq!(state.tasks.len(), 1);
    assert_eq!(state.truncated.map(|cut| cut.total), Some(2));
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
fn classify_plain_c_no_ctrl_is_a_hash_digit() {
    assert_eq!(
        classify(key(KeyCode::Char('c'), KeyModifiers::NONE)),
        Intent::Digit('c')
    );
}

// --- collect_first --------------------------------------------------------

#[test]
fn collect_first_match_returns_correct_index() {
    assert_eq!(collect_first(&state(10, &hash_at(10, 2))), vec![2]);
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
/// Typst-evaluated Mindtape's own `truth/tasks/` backlog, and asserted only
/// that the result was non-empty. That passed for the wrong reason (any
/// backlog anywhere) and broke the moment the suite ran from another directory.
#[test]
fn refresh_loads_tasks_from_project() {
    if mindtape::test_support::rerun_isolated(CHILD).is_some() {
        return;
    }
    let dir = tempdir_with_dirs(&[".mindtape", "tasks"]);
    write_file(
        dir.path(),
        ".mindtape/config.toml",
        "[tasks]\nroots = [\"tasks\"]\n",
    );
    write_file(dir.path(), "tasks/refreshed.typ", TASK_METADATA);
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

#[test]
fn a_typed_o_or_i_narrows_as_the_zero_or_one_the_resolver_reads() {
    let mut state = state(100, "");
    let hash = hash_at(100, 99);
    let typed: String = hash
        .chars()
        .map(|c| match c {
            '0' => 'o',
            '1' => 'i',
            other => other,
        })
        .collect();
    for c in typed.chars() {
        apply(&mut state, Intent::Digit(c));
    }
    assert_eq!(state.first_highlighted(), Some(99), "{typed}");
}
