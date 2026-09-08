#![allow(clippy::unwrap_used)]

use mindtape::config::{Legend, LegendEntry};
use mindtape::numeric::NumValue;
use mindtape::test_support::{bare_task, dep_edge};
use mindtape::{ColumnConfig, Config, Readiness, Status, Task, Visibility};

use super::super::color::parse_color;
use super::super::columns::due_cell;
use super::{render, render_highlighted};

fn with_column(key: &str, visibility: Visibility) -> Config {
    let mut config = Config::default();
    config.ls.columns.insert(
        key.to_owned(),
        ColumnConfig {
            visibility,
            ..ColumnConfig::default()
        },
    );
    config
}

fn task(id: &str, title: &str, status: &str) -> Task {
    let mut t = bare_task(id);
    t.title = title.to_owned();
    t.status = Some(Status {
        name: status.to_owned(),
        date: None,
        note: None,
    });
    t.priority = Some(NumValue::Num(5.0));
    t
}

fn task_with_due(id: &str, status: &str, due: &str) -> Task {
    let mut t = task(id, id, status);
    t.due = Some(due.to_owned());
    t
}

fn config_with_terminal(terminal_statuses: &[&str]) -> Config {
    let mut config = Config::default();
    config.tasks.status = Legend {
        entries: terminal_statuses
            .iter()
            .map(|name| LegendEntry {
                value: (*name).to_owned(),
                terminal: true,
                ..LegendEntry::default()
            })
            .collect(),
    };
    config
}

#[test]
fn one_line_per_task_plus_header() {
    let out = render(
        &[task("a", "A", "wip"), task("b", "B", "done")],
        &Config::default(),
        &Readiness::empty(),
    );
    assert_eq!(out.lines().count(), 3);
}

#[test]
fn highlight_empty_mask_equals_plain() {
    // No row flagged: the highlighted render is byte-identical to the plain
    // one (the `#` column is untouched).
    let tasks = [task("a", "A", "wip"), task("b", "B", "wip")];
    let config = Config::default();
    let plain = render(&tasks, &config, &Readiness::empty());
    let highlighted = render_highlighted(&tasks, &config, &Readiness::empty(), &[false, false]);
    assert_eq!(plain, highlighted);
}

#[test]
fn highlight_preserves_shape() {
    // Flagging a row keeps the header + one-line-per-task layout intact
    // (colour is off in tests, so the invert effect itself is not emitted).
    let tasks = [task("a", "A", "wip"), task("b", "B", "wip")];
    let out = render_highlighted(
        &tasks,
        &Config::default(),
        &Readiness::empty(),
        &[true, false],
    );
    assert_eq!(out.lines().count(), 3);
}

#[test]
fn no_box_drawing_chars() {
    let out = render(
        &[task("a", "A", "wip")],
        &Config::default(),
        &Readiness::empty(),
    );
    for ch in ['│', '┆', '─', '┌', '└', '╞'] {
        assert!(!out.contains(ch), "unexpected box char {ch:?}");
    }
}

#[test]
fn header_and_content_present() {
    let out = render(
        &[task("auth", "Rewrite auth", "wip")],
        &Config::default(),
        &Readiness::empty(),
    );
    assert!(out.contains("TITLE"));
    assert!(out.contains("Rewrite auth"));
    assert!(out.contains("wip"));
    assert!(out.lines().next().unwrap().contains('#'));
}

#[test]
fn readiness_column_appears_only_when_blocked() {
    // No blocked task: column stays hidden.
    let clean = render(
        &[task("a", "A", "wip")],
        &Config::default(),
        &Readiness::empty(),
    );
    assert!(!clean.contains("READY"));

    // `b` depends on the non-terminal `a`, so `b` is blocked and the column
    // appears with its label (glyphs are off in the test's non-TTY output).
    let mut blocked = task("b", "B", "wip");
    blocked.links = vec![dep_edge("a.typ")];
    let tasks = vec![task("a", "A", "wip"), blocked];
    let readiness = Readiness::compute(&tasks, &Config::default());
    let out = render(&tasks, &Config::default(), &readiness);
    assert!(out.contains("READY"));
    assert!(out.contains("blocked"));
}

#[test]
fn empty_columns_hidden_by_default() {
    // A task with status but no priority and no due date: PRIORITY and DUE
    // carry nothing, so `auto` drops them; STATUS and TITLE stay.
    let mut t = task("a", "A", "wip");
    t.priority = None;
    let out = render(&[t], &Config::default(), &Readiness::empty());
    let header = out.lines().next().unwrap();
    assert!(!header.contains("PRIORITY"), "empty PRIORITY must hide");
    assert!(!header.contains("DUE"), "empty DUE must hide");
    assert!(header.contains("STATUS"));
    assert!(header.contains("TITLE"));
}

#[test]
fn visibility_visible_forces_blank_column() {
    let mut t = task("a", "A", "wip");
    t.priority = None;
    let config = with_column("priority", Visibility::Visible);
    let out = render(&[t], &config, &Readiness::empty());
    assert!(
        out.lines().next().unwrap().contains("PRIORITY"),
        "visible must render even when blank"
    );
}

#[test]
fn visibility_hidden_drops_populated_column() {
    let config = with_column("status", Visibility::Hidden);
    let out = render(&[task("a", "A", "wip")], &config, &Readiness::empty());
    assert!(!out.contains("STATUS"), "hidden column must not render");
    assert!(
        !out.contains("wip"),
        "hidden column's cells must not render"
    );
}

#[test]
fn columns_order_selects_and_orders() {
    let mut config = Config::default();
    config.ls.columns_order = vec!["title".to_owned(), "status".to_owned()];
    let out = render(&[task("a", "Hello", "wip")], &config, &Readiness::empty());
    let header = out.lines().next().unwrap();
    let title_at = header.find("TITLE").unwrap();
    let status_at = header.find("STATUS").unwrap();
    assert!(title_at < status_at, "columns-order must reorder");
    assert!(!header.contains('#'), "unlisted handle column excluded");
    assert!(!header.contains("PRIORITY"), "unlisted column excluded");
}

#[test]
fn due_cell_overdue_non_terminal_is_red() {
    let task = task_with_due("t", "wip", "2020-01-01");
    let config = config_with_terminal(&["done"]);
    let cell = due_cell(&task, &config);
    assert!(cell.color.is_some(), "expected red color for overdue task");
}

#[test]
fn due_cell_overdue_terminal_is_plain() {
    let task = task_with_due("t", "done", "2020-01-01");
    let config = config_with_terminal(&["done"]);
    let cell = due_cell(&task, &config);
    assert!(cell.color.is_none(), "terminal task must not be accented");
}

#[test]
fn due_cell_future_due_is_plain() {
    let task = task_with_due("t", "wip", "2999-12-31");
    let config = config_with_terminal(&["done"]);
    let cell = due_cell(&task, &config);
    assert!(cell.color.is_none(), "future due must not be accented");
}

#[test]
fn due_cell_applies_date_format() {
    let task = task_with_due("t", "wip", "2026-06-27");
    let mut config = config_with_terminal(&["done"]);
    config.ls.date_format = Some("DD.MM.YYYY".to_owned());
    let cell = due_cell(&task, &config);
    assert_eq!(
        cell.text, "27.06.2026",
        "date format must rewrite displayed text"
    );
}

#[test]
fn due_cell_overdue_formatted_text_still_red() {
    // Formatting must not interfere with the overdue colour.
    let task = task_with_due("t", "wip", "2020-01-01");
    let mut config = config_with_terminal(&["done"]);
    config.ls.date_format = Some("DD.MM.YYYY".to_owned());
    let cell = due_cell(&task, &config);
    assert!(
        cell.color.is_some(),
        "overdue must still be red after formatting"
    );
    assert_eq!(cell.text, "01.01.2020");
}

#[test]
fn due_cell_overdue_color_override_changes_accent() {
    let task = task_with_due("t", "wip", "2020-01-01");
    let mut config = config_with_terminal(&["done"]);
    config.ls.overdue_color = Some("blue".to_owned());
    let cell = due_cell(&task, &config);
    assert_eq!(
        cell.color,
        parse_color("blue"),
        "overdue accent must honour [ls] overdue-color"
    );
}

#[test]
fn due_cell_overdue_color_unset_falls_back_to_red() {
    let task = task_with_due("t", "wip", "2020-01-01");
    let config = config_with_terminal(&["done"]);
    let cell = due_cell(&task, &config);
    assert_eq!(
        cell.color,
        parse_color("red"),
        "unset overdue-color must default to red"
    );
}

/// The `#` column is the one place a reader ever sees a handle, so a raised
/// `[handle] check-len` has to reach it — a table printing `1b` while the
/// resolver demands `1bq` would refuse every handle it displayed.
#[test]
fn handle_column_honours_a_configured_check_len() {
    let mut config = Config::default();
    config.handle.check_len = Some(2);
    let tasks = vec![task("alpha", "Alpha", "wip")];
    let readiness = Readiness::compute(&tasks, &config);

    let out = render(&tasks, &config, &readiness);
    let expected = format!("1{}", crate::handle::check_chars("alpha", 2));
    assert!(
        out.contains(&expected),
        "table should print the two-character handle {expected}: {out}"
    );
}
