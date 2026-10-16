//! Unit tests for the board: parsing `[board]` and its per-set override,
//! evaluating one project's row, and hiding and sorting rows.

#![allow(clippy::unwrap_used)]

use super::{Board, Cell, TopTask};
use crate::at::{AtConfig, AtError};
use crate::config::Config;
use crate::model::Task;
use crate::test_support::{dep_config, dep_task};

/// A config with `wip` and `done`, plus a `wip` view.
fn with_view() -> Config {
    dep_config("[views.wip]\nfilter = 'status == \"wip\"'\n")
}

/// Two open tasks, one blocked on the other, and one done.
fn tasks() -> Vec<Task> {
    vec![
        dep_task("a", "wip", &[]),
        dep_task("b", "wip", &["a.typ"]),
        dep_task("c", "done", &[]),
    ]
}

fn board(toml: &str) -> Board {
    AtConfig::parse(toml).unwrap().board("any").clone()
}

fn counts(cells: &[Cell]) -> Vec<Option<usize>> {
    cells
        .iter()
        .map(|cell| match cell {
            Cell::Count(n) => Some(*n),
            _ => None,
        })
        .collect()
}

fn failure(toml: &str) -> String {
    match AtConfig::parse(toml) {
        Err(err @ AtError::Board { .. }) => err.to_string(),
        other => panic!("expected a board error, got {other:?}"),
    }
}

#[test]
fn filter_cells_count_the_matching_tasks() {
    let board = board(
        "[[board.columns]]\nname = \"all\"\nfilter = \"\"\n\
         [[board.columns]]\nname = \"ready\"\nfilter = \"ready\"\n\
         [[board.columns]]\nname = \"blocked\"\nfilter = \"blocked\"\n",
    );
    let cells = board.row(&tasks(), &with_view());
    assert_eq!(counts(&cells), [Some(3), Some(1), Some(1)]);
}

#[test]
fn a_view_cell_uses_the_projects_view() {
    let board =
        board("[[board.columns]]\nname = \"wip\"\nfilter = \"@wip\"\nfallback = \"terminal\"\n");
    let cells = board.row(&tasks(), &with_view());
    assert_eq!(counts(&cells), [Some(2)]);
}

#[test]
fn a_view_cell_falls_back_when_the_project_lacks_the_view() {
    let board =
        board("[[board.columns]]\nname = \"wip\"\nfilter = \"@wip\"\nfallback = \"terminal\"\n");
    let cells = board.row(&tasks(), &dep_config(""));
    assert_eq!(counts(&cells), [Some(1)]);
}

#[test]
fn a_view_cell_without_a_fallback_fails_naming_the_column() {
    let board = board("[[board.columns]]\nname = \"wip\"\nfilter = \"@wip\"\n");
    let cells = board.row(&tasks(), &dep_config(""));
    let [Cell::Failed(message)] = cells.as_slice() else {
        panic!("expected one failed cell, got {cells:?}");
    };
    assert!(message.contains("wip"), "{message}");
    assert!(message.contains("unknown view"), "{message}");
}

#[test]
fn a_status_the_legend_lacks_fails_even_with_no_tasks() {
    let board = board("[[board.columns]]\nname = \"doing\"\nfilter = 'status == \"doing\"'\n");
    let cells = board.row(&[], &dep_config(""));
    let [Cell::Failed(message)] = cells.as_slice() else {
        panic!("expected one failed cell, got {cells:?}");
    };
    assert!(message.starts_with("column 'doing': "), "{message}");
}

#[test]
fn a_top_cell_is_the_first_task_of_the_listing() {
    let board = board("[[board.columns]]\nname = \"next\"\ntop = \"!terminal\"\n");
    let cells = board.row(&tasks(), &with_view());
    let [Cell::Top(Some(top))] = cells.as_slice() else {
        panic!("expected a top task, got {cells:?}");
    };
    // The ready task outranks the blocked one in the default order.
    assert_eq!(top.id, "a");
    assert_eq!(top.title, "a");
}

#[test]
fn a_top_cell_over_no_match_is_empty() {
    let board = board("[[board.columns]]\nname = \"next\"\ntop = \"ready\"\n");
    let cells = board.row(&[dep_task("c", "done", &[])], &with_view());
    assert_eq!(cells, [Cell::Top(None)]);
}

#[test]
fn hide_reads_cells_by_column_name() {
    let board = board(
        "[board]\nhide = \"open == 0 && next == \\\"\\\"\"\n\
         [[board.columns]]\nname = \"open\"\nfilter = \"!terminal\"\n\
         [[board.columns]]\nname = \"next\"\ntop = \"ready\"\n",
    );
    assert!(board.hides(&[Cell::Count(0), Cell::Top(None)]));
    assert!(!board.hides(&[Cell::Count(2), Cell::Top(None)]));
}

#[test]
fn a_failed_cell_never_hides_its_row() {
    let board =
        board("[board]\nhide = \"open == 0\"\n[[board.columns]]\nname = \"open\"\nfilter = \"\"\n");
    assert!(!board.hides(&[Cell::Failed("column 'open': boom".to_owned())]));
}

#[test]
fn an_empty_hide_hides_nothing() {
    let board = board("[board]\nhide = \"\"\n");
    assert!(!board.hides(&[Cell::Count(0), Cell::Count(0), Cell::Count(0)]));
}

#[test]
fn sort_orders_rows_by_column_and_keeps_failures_last() {
    let board = board(
        "[board]\nsort = [\"ready:desc\", \"open\"]\n\
         [[board.columns]]\nname = \"open\"\nfilter = \"!terminal\"\n\
         [[board.columns]]\nname = \"ready\"\nfilter = \"ready\"\n",
    );
    let failed = Cell::Failed("column 'ready': boom".to_owned());
    let mut rows = vec![
        ("broken", vec![Cell::Count(1), failed]),
        ("few", vec![Cell::Count(1), Cell::Count(1)]),
        ("many", vec![Cell::Count(9), Cell::Count(4)]),
        ("tied", vec![Cell::Count(0), Cell::Count(1)]),
    ];
    board.sort_rows(&mut rows, |row| &row.1);
    let order: Vec<&str> = rows.iter().map(|row| row.0).collect();
    assert_eq!(order, ["many", "tied", "few", "broken"]);
}

#[test]
fn sort_orders_top_cells_by_title() {
    let board =
        board("[board]\nsort = [\"next\"]\n[[board.columns]]\nname = \"next\"\ntop = \"\"\n");
    let top = |title: &str| {
        Cell::Top(Some(TopTask {
            id: title.to_owned(),
            title: title.to_owned(),
            file: format!("{title}.typ").into(),
        }))
    };
    let mut rows = vec![
        ("b", vec![top("b")]),
        ("none", vec![Cell::Top(None)]),
        ("a", vec![top("a")]),
    ];
    board.sort_rows(&mut rows, |row| &row.1);
    let order: Vec<&str> = rows.iter().map(|row| row.0).collect();
    assert_eq!(order, ["none", "a", "b"]);
}

#[test]
fn a_sets_board_overrides_the_global_one_key_by_key() {
    let at = AtConfig::parse(
        "[board]\nsort = [\"open\"]\nhide = \"\"\n\
         [at.home]\ninclude = [\"~/projects/*\"]\n\
         [at.home.board]\nsort = [\"ready:desc\"]\n",
    )
    .unwrap();
    assert_eq!(at.board("home").sort(), ["ready:desc"]);
    assert_eq!(at.board("home").hide(), None);
    assert_eq!(at.board("other").sort(), ["open"]);
    assert_eq!(at.board("home").columns(), at.board("other").columns());
}

#[test]
fn the_shipped_board_names_no_status() {
    let shipped = board("");
    let names: Vec<&str> = shipped.columns().iter().map(|c| c.name.as_str()).collect();
    assert_eq!(names, ["open", "ready", "blocked", "next"]);
    for column in shipped.columns() {
        let exprs = [&column.filter, &column.top, &column.fallback];
        assert!(
            exprs
                .iter()
                .all(|expr| !expr.as_deref().unwrap_or("").contains("status"))
        );
    }
    // It evaluates under a legend that declares neither `wip` nor `done`.
    let config = crate::config::parse_config("[[tasks.status]]\nvalue = \"todo\"\n").unwrap();
    let cells = shipped.row(&[dep_task("x", "todo", &[])], &config);
    assert!(
        cells.iter().all(|cell| !matches!(cell, Cell::Failed(_))),
        "{cells:?}"
    );
}

#[test]
fn a_column_needs_exactly_one_of_filter_and_top() {
    let both = failure("[[board.columns]]\nname = \"x\"\nfilter = \"\"\ntop = \"\"\n");
    assert!(both.contains("[board]"), "{both}");
    assert!(failure("[[board.columns]]\nname = \"x\"\n").contains("filter"));
}

#[test]
fn column_names_are_unique() {
    let twice = "[[board.columns]]\nname = \"x\"\nfilter = \"\"\n";
    assert!(failure(&format!("{twice}{twice}")).contains("'x'"));
}

#[test]
fn a_cell_filter_that_does_not_parse_is_refused() {
    let message = failure("[[board.columns]]\nname = \"x\"\nfilter = \"((\"\n");
    assert!(message.contains("'x'"), "{message}");
}

#[test]
fn sort_and_hide_name_only_columns() {
    let sort = failure("[board]\nsort = [\"nope:desc\"]\n");
    assert!(sort.contains("nope"), "{sort}");
    let direction = failure("[board]\nsort = [\"open:up\"]\n");
    assert!(direction.contains("up"), "{direction}");
    let hide = failure("[board]\nhide = \"nope > 1\"\n");
    assert!(hide.contains("nope"), "{hide}");
}

#[test]
fn a_bad_set_board_names_its_set() {
    let message = failure("[at.home]\ninclude = [\"~/a\"]\n[at.home.board]\nsort = [\"nope\"]\n");
    assert!(message.contains("[at.home.board]"), "{message}");
}
