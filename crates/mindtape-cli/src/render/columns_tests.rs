#![allow(clippy::unwrap_used)]
use mindtape::config::Config;
use mindtape::numeric::NumValue;
use mindtape::test_support::bare_task;
use mindtape::{Readiness, today_iso};

use super::*;

fn rows<'a>(data: &[(&'a str, &'a Task)]) -> Vec<(String, &'a Task)> {
    data.iter().map(|(h, t)| (h.to_string(), *t)).collect()
}

#[test]
fn title_is_flex() {
    assert!(ColumnId::Title.is_flex());
}

#[test]
fn handle_is_not_flex() {
    assert!(!ColumnId::Handle.is_flex());
}

#[test]
fn status_is_not_flex() {
    assert!(!ColumnId::Status.is_flex());
}

#[test]
fn handle_always_significant() {
    assert!(ColumnId::Handle.auto_significant(
        &[],
        &Config::default(),
        &Readiness::default(),
        false
    ));
}

#[test]
fn title_always_significant() {
    assert!(ColumnId::Title.auto_significant(
        &[],
        &Config::default(),
        &Readiness::default(),
        false
    ));
}

#[test]
fn ready_not_significant_for_uncontested() {
    let task = bare_task("t");
    let config = Config::default();
    let readiness = Readiness::compute(std::slice::from_ref(&task), &config);
    assert!(
        !ColumnId::Ready.auto_significant(&rows(&[("1c", &task)]), &config, &readiness, false),
        "ready column must not be significant when no task is blocked"
    );
}

#[test]
fn priority_significant_when_set() {
    let mut task = bare_task("t");
    task.priority = Some(NumValue::Num(5.0));
    let config = Config::default();
    let readiness = Readiness::compute(std::slice::from_ref(&task), &config);
    assert!(
        ColumnId::Priority.auto_significant(&rows(&[("1c", &task)]), &config, &readiness, false),
        "priority column must be significant when at least one task has a value"
    );
}

#[test]
fn priority_not_significant_when_unset() {
    let task = bare_task("t");
    let config = Config::default();
    let readiness = Readiness::compute(std::slice::from_ref(&task), &config);
    assert!(
        !ColumnId::Priority.auto_significant(&rows(&[("1c", &task)]), &config, &readiness, false),
        "priority column must not be significant when no task has a value"
    );
}

#[test]
fn file_not_significant_under_built_in_order() {
    // The crux of the opt-in design: `file`'s cell is never blank, so
    // the ordinary "non-empty means significant" rule cannot apply to it
    // — it would pin the column permanently visible. Significance is
    // instead `explicit_order` alone.
    let task = bare_task("t");
    let config = Config::default();
    let readiness = Readiness::compute(std::slice::from_ref(&task), &config);
    assert!(
        !ColumnId::File.auto_significant(&rows(&[("1c", &task)]), &config, &readiness, false),
        "file column must not be significant under the built-in (non-explicit) order"
    );
}

#[test]
fn file_significant_when_order_explicit() {
    let task = bare_task("t");
    let config = Config::default();
    let readiness = Readiness::compute(std::slice::from_ref(&task), &config);
    assert!(
        ColumnId::File.auto_significant(&rows(&[("1c", &task)]), &config, &readiness, true),
        "file column must be significant once the render order is explicitly named"
    );
}

#[test]
fn due_today_is_not_overdue() {
    let mut task = bare_task("t");
    task.due = Some(today_iso());
    let config = Config::default();
    let cell = due_cell(&task, &config);
    assert!(
        cell.color.is_none(),
        "due today must not be rendered as overdue"
    );
}

#[test]
fn due_yesterday_is_overdue() {
    let mut task = bare_task("t");
    // Actually yesterday, not a hardcoded date two decades back: the
    // boundary this pins is `due < today`, and only a date adjacent to
    // today exercises it. `days_ago_iso` reads the same pinnable clock
    // `due_cell` compares against, so the pair cannot drift apart.
    task.due = Some(mindtape::days_ago_iso(1));
    let config = Config::default();
    let cell = due_cell(&task, &config);
    assert!(
        cell.color.is_some(),
        "due in the past must be rendered as overdue"
    );
}

#[test]
fn file_cell_renders_the_task_file_when_context_unset() {
    // No test in this binary ever calls `file_path::init`, so the cell
    // falls back to the root-relative `task.file` unchanged.
    let mut task = bare_task("t");
    task.file = std::path::PathBuf::from("truth/tasks/t.typ");
    let cell = ColumnId::File.cell("1c", &task, &Config::default(), &Readiness::default());
    assert_eq!(cell.text, "truth/tasks/t.typ");
}

#[test]
fn bare_ls_hides_the_file_column() {
    let task = bare_task("t");
    let config = Config::default();
    let readiness = Readiness::compute(std::slice::from_ref(&task), &config);
    let order = resolve_order(&config, &rows(&[("1c", &task)]), &readiness);
    assert!(
        !order.contains(&ColumnId::File),
        "file must stay hidden under a bare mt ls with no explicit columns"
    );
}

#[test]
fn explicit_columns_order_shows_the_file_column() {
    let task = bare_task("t");
    let mut config = Config::default();
    config.ls.columns_order = vec!["title".to_owned(), "file".to_owned()];
    let readiness = Readiness::compute(std::slice::from_ref(&task), &config);
    let order = resolve_order(&config, &rows(&[("1c", &task)]), &readiness);
    assert!(
        order.contains(&ColumnId::File),
        "file must show once named in [ls] columns-order"
    );
}

#[test]
fn visibility_visible_forces_file_on_under_built_in_order() {
    let task = bare_task("t");
    let mut config = Config::default();
    config.ls.columns.insert(
        "file".to_owned(),
        mindtape::ColumnConfig {
            visibility: Visibility::Visible,
        },
    );
    let readiness = Readiness::compute(std::slice::from_ref(&task), &config);
    let order = resolve_order(&config, &rows(&[("1c", &task)]), &readiness);
    assert!(
        order.contains(&ColumnId::File),
        "visibility = visible must force file on even under the built-in order"
    );
}

#[test]
fn row_hyperlink_targets_uses_default_template_when_unset() {
    let task = bare_task("t");
    let config = Config::default();
    let uris = row_hyperlink_targets(&rows(&[("1c", &task)]), &config);
    assert_eq!(uris, vec![format!("file://{}", task.file.display())]);
}

#[test]
fn row_hyperlink_targets_honours_a_custom_template() {
    let task = bare_task("t");
    let mut config = Config::default();
    config.ls.hyperlink_target = Some("editor://open?f={rel}".to_owned());
    let uris = row_hyperlink_targets(&rows(&[("1c", &task)]), &config);
    assert_eq!(
        uris,
        vec![format!("editor://open?f={}", task.file.display())]
    );
}

#[test]
fn visibility_hidden_wins_even_when_explicitly_listed() {
    let task = bare_task("t");
    let mut config = Config::default();
    config.ls.columns_order = vec!["title".to_owned(), "file".to_owned()];
    config.ls.columns.insert(
        "file".to_owned(),
        mindtape::ColumnConfig {
            visibility: Visibility::Hidden,
        },
    );
    let readiness = Readiness::compute(std::slice::from_ref(&task), &config);
    let order = resolve_order(&config, &rows(&[("1c", &task)]), &readiness);
    assert!(
        !order.contains(&ColumnId::File),
        "visibility = hidden must win even when file is explicitly listed"
    );
}
