#![allow(clippy::unwrap_used)]

//! `resolve_order`: which columns render, and why. The `file` column's opt-in,
//! the `visibility` key, `[[ls.columns.<name>.rules]]` against the rows, and
//! the four listing predicates a rule can read.
//!
//! Sibling of `columns_tests.rs`, which covers the cell builders and the
//! per-column `auto` significance in isolation. Split from it to keep both
//! under the size gate, and because these tests answer a different question:
//! not "what does `auto` mean for this column" but "what does the resolver
//! decide once the config and the conditions have had their say".

use mindtape::test_support::{bare_task, dep_edge};
use mindtape::{ColumnConfig, ColumnRule, Config, Readiness, Task, Visibility, parse_config};

use super::super::TestCtx;
use super::{ColumnId, resolve_order};

/// `(handle, task)` rows in listing order.
fn rows(tasks: &[Task]) -> Vec<(String, &Task)> {
    tasks
        .iter()
        .enumerate()
        .map(|(i, task)| (format!("{}a", i + 1), task))
        .collect()
}

/// The columns `config` resolves for `tasks`, under an environment carrying
/// `tty` and `format` — the two ambient facts the shipped floor rule reads.
fn order(config: Config, tasks: &[Task], tty: bool, format: &str) -> Vec<ColumnId> {
    let mut fixture = TestCtx {
        readiness: Readiness::compute(tasks, &config),
        config,
        ..TestCtx::default()
    };
    fixture.env.tty = tty;
    fixture.env.format = format.to_owned();
    resolve_order(&rows(tasks), fixture.ctx()).unwrap()
}

/// One column's presence in the resolved set for a captured (`!tty`) table.
fn captured(config: Config, tasks: &[Task], column: ColumnId) -> bool {
    order(config, tasks, false, "table").contains(&column)
}

/// A config declaring one rule on `column`, over the column's own `auto`.
fn ruled(column: &str, condition: &str, visibility: Visibility) -> Config {
    ruled_over(column, condition, visibility, Visibility::Auto)
}

/// [`ruled`] with the column's unconditional `visibility` pinned to `base`.
///
/// A test isolating one predicate wants `base` to be the opposite of what the
/// rule imposes: then the column's presence reports whether the rule fired,
/// rather than whether `auto` happened to agree with it.
fn ruled_over(column: &str, condition: &str, visibility: Visibility, base: Visibility) -> Config {
    let mut config = Config::default();
    config.ls.columns.insert(
        column.to_owned(),
        ColumnConfig {
            visibility: base,
            rules: vec![ColumnRule {
                condition: Some(condition.to_owned()),
                visibility,
            }],
        },
    );
    config
}

/// A task with a title and no priority, due date or assignee.
fn titled(id: &str, title: &str) -> Task {
    let mut task = bare_task(id);
    task.title = title.to_owned();
    task
}

/// A config pinning one column's unconditional `visibility`, no rules.
fn pinned(column: &str, visibility: Visibility) -> Config {
    let mut config = Config::default();
    config.ls.columns.insert(
        column.to_owned(),
        ColumnConfig {
            visibility,
            ..ColumnConfig::default()
        },
    );
    config
}

// --- the `file` column's opt-in ---------------------------------------------

#[test]
fn bare_ls_hides_the_file_column() {
    assert!(
        !captured(Config::default(), &[titled("t", "T")], ColumnId::File),
        "file must stay hidden under a bare mt ls with no explicit columns"
    );
}

#[test]
fn explicit_columns_order_shows_the_file_column() {
    let mut config = Config::default();
    config.ls.columns_order = vec!["title".to_owned(), "file".to_owned()];
    assert!(
        captured(config, &[titled("t", "T")], ColumnId::File),
        "file must show once named in [ls] columns-order"
    );
}

#[test]
fn visibility_visible_forces_file_on_under_built_in_order() {
    assert!(
        captured(
            pinned("file", Visibility::Visible),
            &[titled("t", "T")],
            ColumnId::File
        ),
        "visibility = visible must force file on even under the built-in order"
    );
}

#[test]
fn visibility_hidden_wins_even_when_explicitly_listed() {
    let mut config = pinned("file", Visibility::Hidden);
    config.ls.columns_order = vec!["title".to_owned(), "file".to_owned()];
    assert!(
        !captured(config, &[titled("t", "T")], ColumnId::File),
        "visibility = hidden must win even when file is explicitly listed"
    );
}

// --- the shipped floor rule -------------------------------------------------
//
// These read the real embedded floor rather than a fixture: the shipped rule
// is the thing under test, and a fixture restating it would pass while the
// floor said anything.

#[test]
fn the_floor_shows_file_in_a_captured_table() {
    let tasks = [titled("t", "T")];
    assert!(captured(parse_config("").unwrap(), &tasks, ColumnId::File));
}

#[test]
fn the_floor_leaves_file_opt_in_on_a_terminal() {
    let tasks = [titled("t", "T")];
    let order = order(parse_config("").unwrap(), &tasks, true, "table");
    assert!(!order.contains(&ColumnId::File));
}

/// CSV resolves the same column set as the table, so the floor rule has to
/// exclude it by name or a script parsing `mt ls --format csv` silently gains
/// a column.
#[test]
fn the_floor_leaves_csv_alone_even_when_captured() {
    let tasks = [titled("t", "T")];
    let order = order(parse_config("").unwrap(), &tasks, false, "csv");
    assert!(!order.contains(&ColumnId::File));
}

/// The documented opt-out: an array replaces rather than merges across config
/// layers, so an empty list drops the inherited floor rule outright.
#[test]
fn an_empty_rules_list_opts_out_of_the_floor_rule() {
    let config = parse_config("[ls.columns.file]\nrules = []\n").unwrap();
    let tasks = [titled("t", "T")];
    assert!(!captured(config, &tasks, ColumnId::File));
}

// --- rule precedence --------------------------------------------------------

/// A rule beats the column's own `visibility` key: rules ARE the override
/// mechanism, so the other direction would make them unwritable.
#[test]
fn a_matching_rule_beats_the_unconditional_visibility() {
    let mut config = ruled("status", "!tty()", Visibility::Visible);
    config.ls.columns.get_mut("status").unwrap().visibility = Visibility::Hidden;
    let tasks = [titled("t", "T")];
    assert!(captured(config, &tasks, ColumnId::Status));
}

/// ... and when no rule matches, the unconditional key is what is left.
#[test]
fn a_non_matching_rule_leaves_the_unconditional_visibility() {
    let mut config = ruled("status", "!tty()", Visibility::Visible);
    config.ls.columns.get_mut("status").unwrap().visibility = Visibility::Hidden;
    let tasks = [titled("t", "T")];
    assert!(!order(config, &tasks, true, "table").contains(&ColumnId::Status));
}

/// First match wins, mirroring `[[ls.rules]]`.
#[test]
fn the_first_matching_rule_wins() {
    let mut config = Config::default();
    config.ls.columns.insert(
        "due".to_owned(),
        ColumnConfig {
            visibility: Visibility::Auto,
            rules: vec![
                ColumnRule {
                    condition: Some("!tty()".to_owned()),
                    visibility: Visibility::Hidden,
                },
                ColumnRule {
                    condition: Some("!tty()".to_owned()),
                    visibility: Visibility::Visible,
                },
            ],
        },
    );
    let tasks = [titled("t", "T")];
    assert!(!captured(config, &tasks, ColumnId::Due));
}

/// Config load validates every condition, so reaching this needs a config
/// built in memory — but it must still report rather than silently not match.
#[test]
fn an_unevaluable_condition_names_the_column_and_index() {
    let config = ruled("title", "ttty()", Visibility::Hidden);
    let tasks = [titled("t", "T")];
    let fixture = TestCtx::with_config(config);
    let err = resolve_order(&rows(&tasks), fixture.ctx()).unwrap_err();
    assert!(
        format!("{err:#}").contains("ls.columns.title.rules[0].condition"),
        "{err:#}"
    );
}

// --- the listing predicates, against real rows ------------------------------

/// `nonempty()` reads this column's own cells. Base `hidden` against a rule
/// that shows, so the column's presence is the rule firing and nothing else.
#[test]
fn nonempty_reads_this_column_s_cells() {
    let rule = || {
        ruled_over(
            "priority",
            "!nonempty()",
            Visibility::Visible,
            Visibility::Hidden,
        )
    };
    assert!(captured(rule(), &[titled("a", "A")], ColumnId::Priority));
    let mut scored = titled("b", "B");
    scored.priority = Some(mindtape::numeric::NumValue::Num(5.0));
    assert!(!captured(rule(), &[scored], ColumnId::Priority));
}

/// `contested()` reads whether the rows disagree — the fact `assignee`'s
/// `auto` already uses, now readable for any column.
#[test]
fn contested_reads_whether_the_rows_disagree() {
    let hidden = ruled("title", "contested()", Visibility::Hidden);
    let mixed = [titled("a", "A"), titled("b", "B")];
    assert!(!captured(hidden.clone(), &mixed, ColumnId::Title));
    let agreed = [titled("a", "Same"), titled("b", "Same")];
    assert!(captured(hidden, &agreed, ColumnId::Title));
}

/// `named()` reads the REQUEST — whether the caller asked for this column —
/// never the outcome. Same rule, same rows, two different column requests.
#[test]
fn named_reads_the_requested_order() {
    let tasks = [titled("t", "T")];
    let rule = || ruled_over("status", "named()", Visibility::Hidden, Visibility::Visible);
    let mut asked = rule();
    asked.ls.columns_order = vec!["status".to_owned(), "title".to_owned()];
    assert!(!captured(asked, &tasks, ColumnId::Status));
    // Unasked: the built-in order, so the rule does not fire and the column's
    // own `visible` is what is left.
    assert!(captured(rule(), &tasks, ColumnId::Status));
}

/// `blocked()` reads the listing, so a column with nothing in it can still be
/// forced on by what the rows say.
#[test]
fn blocked_reads_the_listing_not_the_column() {
    let visible = ruled("due", "blocked()", Visibility::Visible);
    let clear = [titled("a", "A")];
    assert!(!captured(visible.clone(), &clear, ColumnId::Due));

    let mut waiting = titled("b", "B");
    waiting.links = vec![dep_edge("a.typ")];
    let stuck = [titled("a", "A"), waiting];
    assert!(captured(visible, &stuck, ColumnId::Due));
}

/// The reason `ready` needed a fourth predicate rather than reusing
/// `nonempty()`: on an all-ready listing every `ready` cell carries a label,
/// so `nonempty()` is true while `blocked()` — the column's actual `auto`
/// significance — is false. The two do not coincide, and `ready` stays hidden.
#[test]
fn ready_significance_is_blocked_not_nonempty() {
    let tasks = [titled("a", "A"), titled("b", "B")];
    let config = Config::default();
    let readiness = Readiness::compute(&tasks, &config);
    let facts = ColumnId::Ready.facts(&rows(&tasks), &config, &readiness, false);
    assert!(
        facts.nonempty,
        "a ready task renders a label, so the cells are not blank"
    );
    assert!(!facts.blocked, "nothing here is blocked");
    assert!(!ColumnId::Ready.auto_significant(facts));
}

/// Each column is resolved against its own facts: one run, one condition text,
/// two different answers, because `nonempty()` means this column's cells.
#[test]
fn facts_do_not_leak_between_columns() {
    let mut config = Config::default();
    for column in ["priority", "title"] {
        config.ls.columns.insert(
            column.to_owned(),
            ColumnConfig {
                visibility: Visibility::Hidden,
                rules: vec![ColumnRule {
                    condition: Some("nonempty()".to_owned()),
                    visibility: Visibility::Visible,
                }],
            },
        );
    }
    // `title` carries text and `priority` does not, so the same condition must
    // fire for one and not the other.
    let order = order(config, &[titled("a", "A")], false, "table");
    assert!(order.contains(&ColumnId::Title));
    assert!(!order.contains(&ColumnId::Priority));
}
