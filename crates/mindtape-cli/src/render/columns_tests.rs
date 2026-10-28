#![allow(clippy::unwrap_used)]
use mindtape::config::Config;
use mindtape::model::numeric::NumValue;
use mindtape::test_support::{bare_task, floor_config};
use mindtape::{ConditionEnv, Readiness, today_iso};

use super::super::TestCtx;
use super::*;

fn rows<'a>(data: &[(&'a str, &'a Task)]) -> Vec<(String, &'a Task)> {
    data.iter().map(|(h, t)| (h.to_string(), *t)).collect()
}

/// Whether the shipped floor's `when` for `col` resolves true over `rows`.
///
/// The per-column policy used to be a Rust match (`auto_significant`) and is
/// now one `when` expression per column in `defaults.toml`, so these tests ask
/// the floor rather than a fixture restating it: a fixture would pass while the
/// floor said anything. `named` is the request fact `resolve_order` derives
/// from `[ls] columns-order`; the environment is otherwise blank, which is the
/// captured-but-formatless shape (`!tty()`, no `format()`).
fn shows(col: ColumnId, rows: &[(String, &Task)], readiness: &Readiness, named: bool) -> bool {
    let fixture = TestCtx {
        readiness: readiness.clone(),
        ..TestCtx::default()
    };
    let env = ConditionEnv {
        column: col.facts(rows, fixture.ctx(), named),
        ..ConditionEnv::default()
    };
    col.visible(&fixture.config, &env).unwrap()
}

/// The render order `config` resolves for `rows`. Every fixture here carries a
/// parseable `when`, so the resolution cannot fail.
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
fn slug_is_not_flex() {
    assert!(!ColumnId::StemHash.is_flex());
}

#[test]
fn status_is_not_flex() {
    assert!(!ColumnId::Status.is_flex());
}

#[test]
fn slug_always_renders() {
    assert!(shows(ColumnId::StemHash, &[], &Readiness::default(), false));
}

#[test]
fn title_always_renders() {
    assert!(shows(ColumnId::Title, &[], &Readiness::default(), false));
}

#[test]
fn ready_hidden_when_nothing_is_blocked() {
    let task = bare_task("t");
    let readiness = Readiness::compute(std::slice::from_ref(&task), &Config::default());
    assert!(
        !shows(ColumnId::Ready, &rows(&[("1c", &task)]), &readiness, false),
        "ready column must not render when no task is blocked"
    );
}

/// The reason `ready` reads `blocked()` rather than `nonempty()`: on an
/// all-ready listing every cell carries a label, so the two disagree, and the
/// column that says "something is stuck" must follow the one that means it.
#[test]
fn ready_reads_blocked_not_nonempty() {
    let task = bare_task("t");
    let config = floor_config();
    let readiness = Readiness::compute(std::slice::from_ref(&task), &config);
    let fixture = TestCtx {
        config,
        readiness,
        ..TestCtx::default()
    };
    let facts = ColumnId::Ready.facts(&rows(&[("1c", &task)]), fixture.ctx(), false);
    assert!(
        facts.nonempty,
        "a ready task renders a label, so the cells are not blank"
    );
    assert!(!facts.blocked, "nothing here is blocked");
}

#[test]
fn priority_renders_when_set() {
    let mut task = bare_task("t");
    task.priority = Some(NumValue::Num(5.0));
    let readiness = Readiness::compute(std::slice::from_ref(&task), &Config::default());
    assert!(
        shows(
            ColumnId::Priority,
            &rows(&[("1c", &task)]),
            &readiness,
            false
        ),
        "priority column must render when at least one task has a value"
    );
}

#[test]
fn priority_hidden_when_unset() {
    let task = bare_task("t");
    let readiness = Readiness::compute(std::slice::from_ref(&task), &Config::default());
    assert!(
        !shows(
            ColumnId::Priority,
            &rows(&[("1c", &task)]),
            &readiness,
            false
        ),
        "priority column must not render when no task has a value"
    );
}

#[test]
fn file_hidden_under_built_in_order() {
    // The crux of the opt-in design: `file`'s cell is never blank, so the
    // ordinary `nonempty()` rule cannot apply to it — it would pin the column
    // permanently visible. The floor gives it `named()` instead.
    let task = bare_task("t");
    assert!(
        !shows(
            ColumnId::File,
            &rows(&[("1c", &task)]),
            &Readiness::default(),
            false
        ),
        "file column must not render under the built-in (non-explicit) order"
    );
}

#[test]
fn file_renders_when_order_explicit() {
    let task = bare_task("t");
    assert!(
        shows(
            ColumnId::File,
            &rows(&[("1c", &task)]),
            &Readiness::default(),
            true
        ),
        "file column must render once the render order is explicitly named"
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
    // The default fixture has no root, so the cell falls back to the
    // root-relative `task.file` unchanged.
    let mut task = bare_task("t");
    task.file = std::path::PathBuf::from("truth/tasks/t.typ");
    let cell = ColumnId::File.cell("1c", &task, TestCtx::default().ctx());
    assert_eq!(cell.text, mindtape::display_path("truth/tasks/t.typ"));
}

#[test]
fn row_hyperlink_targets_uses_default_template_when_unset() {
    let task = bare_task("t");
    let fixture = TestCtx::with_config(Config::default());
    let uris = row_hyperlink_targets(&rows(&[("1c", &task)]), fixture.ctx());
    assert_eq!(uris, vec![format!("file://{}", task.file.display())]);
}

#[test]
fn row_hyperlink_targets_honours_a_custom_template() {
    let task = bare_task("t");
    let mut config = Config::default();
    config.ls.hyperlink_target = Some("editor://open?f={rel}".to_owned());
    let fixture = TestCtx::with_config(config);
    let uris = row_hyperlink_targets(&rows(&[("1c", &task)]), fixture.ctx());
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

/// Whether the floor renders the `assignee` column across a two-row listing
/// with these assignees, under the built-in (non-explicit) order.
fn assignee_shows(a: Option<&str>, b: Option<&str>) -> bool {
    let (a, b) = (assigned("a", a), assigned("b", b));
    shows(
        ColumnId::Assignee,
        &rows(&[("1a", &a), ("2b", &b)]),
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

/// The floor with `[ls.columns.assignee] when` replaced. A scalar replaces
/// wholesale across config layers, so this is exactly what a project writes.
fn assignee_when(when: &str) -> Config {
    let mut config = floor_config();
    config.ls.columns.insert(
        "assignee".to_owned(),
        mindtape::ColumnConfig {
            when: Some(when.to_owned()),
        },
    );
    config
}

#[test]
fn assignee_cell_is_blank_when_unset() {
    let task = assigned("t", None);
    let cell = ColumnId::Assignee.cell("1c", &task, TestCtx::default().ctx());
    assert_eq!(cell.text, "");
}

#[test]
fn assignee_cell_renders_the_actor_name() {
    let task = assigned("t", Some("agent"));
    let cell = ColumnId::Assignee.cell("1c", &task, TestCtx::default().ctx());
    assert_eq!(cell.text, "agent");
}

#[test]
fn assignee_hidden_when_every_row_agrees() {
    // The crux of the contested design: `mt ls @agent` is already narrowed to
    // one actor, so a column repeating that name down every row says nothing
    // the filter did not. The ordinary "any cell has text" rule would show it.
    assert!(
        !assignee_shows(Some("agent"), Some("agent")),
        "assignee must not render when every row carries the same name"
    );
}

#[test]
fn assignee_renders_when_rows_disagree() {
    assert!(
        assignee_shows(Some("agent"), Some("max")),
        "assignee must render once a listing mixes two actors"
    );
}

#[test]
fn assignee_hidden_when_nobody_is_assigned() {
    assert!(
        !assignee_shows(None, None),
        "assignee must not render when no task carries one"
    );
}

#[test]
fn assignee_renders_when_only_some_rows_are_assigned() {
    // An absent assignee counts as its own value, so the blanks are what makes
    // this listing contested — reading which rows are unassigned is the point.
    assert!(
        assignee_shows(Some("agent"), None),
        "a half-assigned listing must be contested"
    );
    assert!(
        assignee_shows(None, Some("agent")),
        "the blank arriving first must be contested too"
    );
}

#[test]
fn assignee_hidden_on_a_single_row() {
    let task = assigned("t", Some("agent"));
    assert!(
        !shows(
            ColumnId::Assignee,
            &rows(&[("1c", &task)]),
            &Readiness::default(),
            false
        ),
        "one row can never disagree with itself"
    );
}

#[test]
fn bare_ls_shows_assignee_on_a_mixed_listing() {
    assert!(
        assignee_rendered(&floor_config(), "agent", "max"),
        "assignee must appear under the built-in order once the rows disagree"
    );
}

#[test]
fn bare_ls_hides_assignee_on_a_single_actor_listing() {
    assert!(
        !assignee_rendered(&floor_config(), "agent", "agent"),
        "mt ls @agent must not grow a column repeating one name on every row"
    );
}

#[test]
fn assignee_renders_between_due_and_title() {
    let order = assignee_order(&floor_config(), "agent", "max");
    let at = |id: ColumnId| order.iter().position(|c| *c == id);
    assert!(
        at(ColumnId::Assignee) < at(ColumnId::Title),
        "assignee is a short scalar and belongs before the flex title column"
    );
}

#[test]
fn explicit_columns_order_still_defers_to_the_contested_rule() {
    // Naming a column does not override its `when` — the same way an
    // explicitly listed `priority` stays hidden when no task scores one.
    // `[ls.columns.assignee] when = 'true'` is the override.
    let mut config = floor_config();
    config.ls.columns_order = vec!["title".to_owned(), "assignee".to_owned()];
    assert!(
        !assignee_rendered(&config, "agent", "agent"),
        "an uncontested assignee stays hidden even when named in columns-order"
    );
}

#[test]
fn when_true_forces_assignee_on_an_uncontested_listing() {
    assert!(
        assignee_rendered(&assignee_when("true"), "agent", "agent"),
        "when = 'true' must force assignee on despite an uncontested listing"
    );
}

#[test]
fn when_false_hides_assignee_on_a_mixed_listing() {
    assert!(
        !assignee_rendered(&assignee_when("false"), "agent", "max"),
        "when = 'false' must win even on a contested listing"
    );
}

// --- children_text ---
//
// The column is blank for a leaf and the count for a parent: both sides of
// `== 0` decide output, so a table with one fixture each pins the operator and
// the whole body (a stub return of `String::new()` or a constant fails the
// parent case; a stub of `"xyzzy"` fails the leaf case).

#[test]
fn children_text_is_blank_for_a_leaf() {
    let mut task = bare_task("t");
    task.children = 0;
    assert_eq!(children_text(&task), "");
}

#[test]
fn children_text_renders_the_count_for_a_parent() {
    let mut task = bare_task("t");
    task.children = 3;
    assert_eq!(children_text(&task), "3");
}

// --- a set's PROJECT column and joined columns -------------------------------

#[test]
fn union_order_slots_a_new_column_after_the_one_it_follows() {
    use ColumnId::{Priority, Status, StemHash, Title};
    let joined = union_order(&[
        vec![StemHash, Title],
        vec![StemHash, Status, Priority, Title],
        vec![Title, Status],
    ]);
    assert_eq!(joined, [StemHash, Status, Priority, Title]);
    assert!(union_order(&[]).is_empty());
}

#[test]
fn the_project_column_leads_only_a_set_listing() {
    let task = bare_task("x");
    let fixture = TestCtx::default();
    let rows = rows(&[("a1", &task)]);
    let alone = resolve_order(&rows, fixture.ctx()).unwrap();
    assert!(!alone.contains(&ColumnId::Project), "{alone:?}");
    let ctx = RenderCtx {
        project: Some("home"),
        ..fixture.ctx()
    };
    let in_set = resolve_order(&rows, ctx).unwrap();
    assert_eq!(in_set.first(), Some(&ColumnId::Project), "{in_set:?}");
    assert_eq!(ColumnId::Project.cell("a1", &task, ctx).text, "home");
    assert_eq!(ColumnId::Project.key(), "project");
    assert_eq!(ColumnId::Project.header(), "PROJECT");
}
