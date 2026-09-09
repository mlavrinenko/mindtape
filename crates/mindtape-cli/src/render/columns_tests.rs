#![allow(clippy::unwrap_used)]
use mindtape::config::Config;
use mindtape::numeric::NumValue;
use mindtape::test_support::bare_task;
use mindtape::{Readiness, today_iso};

use super::super::TestCtx;
use super::*;

fn rows<'a>(data: &[(&'a str, &'a Task)]) -> Vec<(String, &'a Task)> {
    data.iter().map(|(h, t)| (h.to_string(), *t)).collect()
}

/// The `auto` outcome for one column over `rows` — the [`ColumnId::facts`] /
/// [`ColumnId::auto_significant`] pair `resolve_order` runs, kept together so
/// a test asserts on the decision rather than on either half of it.
fn significant(
    col: ColumnId,
    rows: &[(String, &Task)],
    config: &Config,
    readiness: &Readiness,
    named: bool,
) -> bool {
    col.auto_significant(col.facts(rows, config, readiness, named))
}

/// The render order `config` resolves for `rows`. No fixture here declares a
/// column rule, so the resolution cannot fail.
fn order_for(config: &Config, rows: &[(String, &Task)], readiness: Readiness) -> Vec<ColumnId> {
    let fixture = TestCtx {
        config: config.clone(),
        readiness,
        ..TestCtx::default()
    };
    resolve_order(rows, fixture.ctx()).unwrap()
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
    assert!(significant(
        ColumnId::Handle,
        &[],
        &Config::default(),
        &Readiness::default(),
        false
    ));
}

#[test]
fn title_always_significant() {
    assert!(significant(
        ColumnId::Title,
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
        !significant(
            ColumnId::Ready,
            &rows(&[("1c", &task)]),
            &config,
            &readiness,
            false
        ),
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
        significant(
            ColumnId::Priority,
            &rows(&[("1c", &task)]),
            &config,
            &readiness,
            false
        ),
        "priority column must be significant when at least one task has a value"
    );
}

#[test]
fn priority_not_significant_when_unset() {
    let task = bare_task("t");
    let config = Config::default();
    let readiness = Readiness::compute(std::slice::from_ref(&task), &config);
    assert!(
        !significant(
            ColumnId::Priority,
            &rows(&[("1c", &task)]),
            &config,
            &readiness,
            false
        ),
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
        !significant(
            ColumnId::File,
            &rows(&[("1c", &task)]),
            &config,
            &readiness,
            false
        ),
        "file column must not be significant under the built-in (non-explicit) order"
    );
}

#[test]
fn file_significant_when_order_explicit() {
    let task = bare_task("t");
    let config = Config::default();
    let readiness = Readiness::compute(std::slice::from_ref(&task), &config);
    assert!(
        significant(
            ColumnId::File,
            &rows(&[("1c", &task)]),
            &config,
            &readiness,
            true
        ),
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
fn assignee_is_not_flex() {
    assert!(!ColumnId::Assignee.is_flex());
}

/// A task carrying `assignee` (`None` leaves it unset).
fn assigned(id: &str, who: Option<&str>) -> Task {
    let mut task = bare_task(id);
    task.assignee = who.map(str::to_owned);
    task
}

/// Whether the `assignee` column is significant across a two-row listing with
/// these assignees, under the built-in (non-explicit) order.
fn assignee_significant(a: Option<&str>, b: Option<&str>) -> bool {
    let (a, b) = (assigned("a", a), assigned("b", b));
    significant(
        ColumnId::Assignee,
        &rows(&[("1a", &a), ("2b", &b)]),
        &Config::default(),
        &Readiness::default(),
        false,
    )
}

/// The render order `config` resolves for a two-row listing with these
/// assignees.
fn assignee_order(config: &Config, a: &str, b: &str) -> Vec<ColumnId> {
    let (a, b) = (assigned("a", Some(a)), assigned("b", Some(b)));
    let tasks = [a.clone(), b.clone()];
    let readiness = Readiness::compute(&tasks, config);
    order_for(config, &rows(&[("1a", &a), ("2b", &b)]), readiness)
}

/// Whether that order keeps the `assignee` column.
fn assignee_rendered(config: &Config, a: &str, b: &str) -> bool {
    assignee_order(config, a, b).contains(&ColumnId::Assignee)
}

/// A config pinning `[ls.columns.assignee] visibility`.
fn assignee_visibility(visibility: Visibility) -> Config {
    let mut config = Config::default();
    config.ls.columns.insert(
        "assignee".to_owned(),
        mindtape::ColumnConfig {
            visibility,
            ..mindtape::ColumnConfig::default()
        },
    );
    config
}

#[test]
fn assignee_cell_is_blank_when_unset() {
    let task = assigned("t", None);
    let cell = ColumnId::Assignee.cell("1c", &task, &Config::default(), &Readiness::default());
    assert_eq!(cell.text, "");
}

#[test]
fn assignee_cell_renders_the_actor_name() {
    let task = assigned("t", Some("agent"));
    let cell = ColumnId::Assignee.cell("1c", &task, &Config::default(), &Readiness::default());
    assert_eq!(cell.text, "agent");
}

#[test]
fn assignee_not_significant_when_every_row_agrees() {
    // The crux of the contested design: `mt ls @agent` is already narrowed to
    // one actor, so a column repeating that name down every row says nothing
    // the filter did not. The ordinary "any cell has text" rule would show it.
    assert!(
        !assignee_significant(Some("agent"), Some("agent")),
        "assignee must not be significant when every row carries the same name"
    );
}

#[test]
fn assignee_significant_when_rows_disagree() {
    assert!(
        assignee_significant(Some("agent"), Some("max")),
        "assignee must be significant once a listing mixes two actors"
    );
}

#[test]
fn assignee_not_significant_when_nobody_is_assigned() {
    assert!(
        !assignee_significant(None, None),
        "assignee must not be significant when no task carries one"
    );
}

#[test]
fn assignee_significant_when_only_some_rows_are_assigned() {
    // An absent assignee counts as its own value, so the blanks are what makes
    // this listing contested — reading which rows are unassigned is the point.
    assert!(
        assignee_significant(Some("agent"), None),
        "a half-assigned listing must be contested"
    );
    assert!(
        assignee_significant(None, Some("agent")),
        "the blank arriving first must be contested too"
    );
}

#[test]
fn assignee_not_significant_on_a_single_row() {
    let task = assigned("t", Some("agent"));
    assert!(
        !significant(
            ColumnId::Assignee,
            &rows(&[("1c", &task)]),
            &Config::default(),
            &Readiness::default(),
            false
        ),
        "one row can never disagree with itself"
    );
}

#[test]
fn bare_ls_shows_assignee_on_a_mixed_listing() {
    assert!(
        assignee_rendered(&Config::default(), "agent", "max"),
        "assignee must appear under the built-in order once the rows disagree"
    );
}

#[test]
fn bare_ls_hides_assignee_on_a_single_actor_listing() {
    assert!(
        !assignee_rendered(&Config::default(), "agent", "agent"),
        "mt ls @agent must not grow a column repeating one name on every row"
    );
}

#[test]
fn assignee_renders_between_due_and_title() {
    let order = assignee_order(&Config::default(), "agent", "max");
    let at = |id: ColumnId| order.iter().position(|c| *c == id);
    assert!(
        at(ColumnId::Assignee) < at(ColumnId::Title),
        "assignee is a short scalar and belongs before the flex title column"
    );
}

#[test]
fn explicit_columns_order_still_defers_to_the_contested_rule() {
    // Naming a column does not override its `auto` significance — the same
    // way an explicitly listed `priority` stays hidden when no task scores
    // one. `[ls.columns.assignee] visibility = "visible"` is the override.
    let mut config = Config::default();
    config.ls.columns_order = vec!["title".to_owned(), "assignee".to_owned()];
    assert!(
        !assignee_rendered(&config, "agent", "agent"),
        "an uncontested assignee stays hidden even when named in columns-order"
    );
}

#[test]
fn visibility_visible_forces_assignee_on_an_uncontested_listing() {
    assert!(
        assignee_rendered(&assignee_visibility(Visibility::Visible), "agent", "agent"),
        "visibility = visible must force assignee on despite an uncontested listing"
    );
}

#[test]
fn visibility_hidden_wins_for_assignee_on_a_mixed_listing() {
    assert!(
        !assignee_rendered(&assignee_visibility(Visibility::Hidden), "agent", "max"),
        "visibility = hidden must win even on a contested listing"
    );
}
