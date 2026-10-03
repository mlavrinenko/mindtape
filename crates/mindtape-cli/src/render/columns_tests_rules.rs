#![allow(clippy::unwrap_used)]

//! `resolve_order`: which columns render, and why. The shipped floor's `when`
//! for `file`, what an absent `when` means, and the four listing predicates a
//! `when` can read.
//!
//! Sibling of `columns_tests.rs`, which covers the cell builders and each
//! column's own floored `when` in isolation. Split from it to keep both under
//! the size gate, and because these tests answer a different question: not
//! "what does the floor say about this column" but "what does the resolver
//! decide once the config and the expressions have had their say".

use mindtape::test_support::{bare_task, dep_edge, floor_config};
use mindtape::{ColumnConfig, Config, Readiness, Task, parse_config};

use super::super::TestCtx;
use super::{ColumnId, resolve_order};

/// `(hash, task)` rows in listing order.
fn rows(tasks: &[Task]) -> Vec<(String, &Task)> {
    tasks
        .iter()
        .enumerate()
        .map(|(i, task)| (format!("{}a", i + 1), task))
        .collect()
}

/// The columns `config` resolves for `tasks`, under an environment carrying
/// `tty` and `format` — the two ambient facts the shipped floor's `file`
/// expression reads.
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

/// The floor with one column's `when` replaced — a scalar replaces wholesale
/// across config layers, so this is exactly the one line a project writes.
fn when(column: &str, expr: &str) -> Config {
    let mut config = floor_config();
    config.ls.columns.insert(
        column.to_owned(),
        ColumnConfig {
            when: Some(expr.to_owned()),
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

// --- the shipped floor ------------------------------------------------------
//
// These read the real embedded floor rather than a fixture: the shipped
// expression is the thing under test, and a fixture restating it would pass
// while the floor said anything.

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

/// CSV resolves the same column set as the table, so the floor's expression has
/// to exclude it by name or a script parsing `mt ls --format csv` silently
/// gains a column.
#[test]
fn the_floor_leaves_csv_alone_even_when_captured() {
    let tasks = [titled("t", "T")];
    let order = order(parse_config("").unwrap(), &tasks, false, "csv");
    assert!(!order.contains(&ColumnId::File));
}

/// The `named()` half of the same expression: on a terminal, where the other
/// half is false, asking for the column is what turns it on.
#[test]
fn the_floor_shows_file_on_a_terminal_once_it_is_named() {
    let mut config = parse_config("").unwrap();
    config.ls.columns_order = vec!["title".to_owned(), "file".to_owned()];
    let order = order(config, &[titled("t", "T")], true, "table");
    assert!(order.contains(&ColumnId::File));
}

/// The opt-out, and the whole reason one expression replaced a `visibility`
/// key plus a rules list: a scalar replaces wholesale across config layers, so
/// turning the floored column off is one line with no precedence to know.
#[test]
fn a_project_when_replaces_the_floor_s() {
    let config = parse_config("[ls.columns.file]\nwhen = 'false'\n").unwrap();
    assert!(!captured(config, &[titled("t", "T")], ColumnId::File));
}

// --- what an absent `when` means --------------------------------------------

/// A column the config does not describe never renders: nothing falls back to
/// a built-in policy in Rust, so a config that never merged the floor resolves
/// to no columns at all rather than to a hidden second opinion.
#[test]
fn a_column_the_config_does_not_describe_never_renders() {
    let resolved = order(Config::default(), &[titled("t", "T")], false, "table");
    assert!(resolved.is_empty(), "{resolved:?}");
}

/// An entry with no `when` is the same answer as no entry: a project drops a
/// floored column by writing over it, and an empty table is what it gets.
#[test]
fn an_entry_carrying_no_when_never_renders() {
    let mut config = floor_config();
    config
        .ls
        .columns
        .insert("title".to_owned(), ColumnConfig::default());
    assert!(!captured(config, &[titled("t", "T")], ColumnId::Title));
}

/// Config load validates every `when`, so reaching this needs a config built
/// in memory — but it must still report rather than silently not match.
#[test]
fn an_unevaluable_when_names_the_column() {
    let config = when("title", "ttty()");
    let fixture = TestCtx::with_config(config);
    let err = resolve_order(&rows(&[titled("t", "T")]), fixture.ctx()).unwrap_err();
    assert!(
        format!("{err:#}").contains("ls.columns.title.when"),
        "{err:#}"
    );
}

// --- the listing predicates, against real rows ------------------------------

/// `nonempty()` reads this column's own cells.
#[test]
fn nonempty_reads_this_column_s_cells() {
    let config = || when("priority", "!nonempty()");
    assert!(captured(config(), &[titled("a", "A")], ColumnId::Priority));
    let mut scored = titled("b", "B");
    scored.priority = Some(mindtape::numeric::NumValue::Num(5.0));
    assert!(!captured(config(), &[scored], ColumnId::Priority));
}

/// `contested()` reads whether the rows disagree.
#[test]
fn contested_reads_whether_the_rows_disagree() {
    let config = || when("title", "!contested()");
    let mixed = [titled("a", "A"), titled("b", "B")];
    assert!(!captured(config(), &mixed, ColumnId::Title));
    let agreed = [titled("a", "Same"), titled("b", "Same")];
    assert!(captured(config(), &agreed, ColumnId::Title));
}

/// `named()` reads the REQUEST — whether the caller asked for this column —
/// never the outcome. Same expression, same rows, two different column
/// requests.
#[test]
fn named_reads_the_requested_order() {
    let tasks = [titled("t", "T")];
    let config = || when("status", "!named()");
    let mut asked = config();
    asked.ls.columns_order = vec!["status".to_owned(), "title".to_owned()];
    assert!(!captured(asked, &tasks, ColumnId::Status));
    // Unasked: the built-in order, so `named()` is false and the column shows.
    assert!(captured(config(), &tasks, ColumnId::Status));
}

/// `blocked()` reads the listing, so a column with nothing in it can still be
/// forced on by what the rows say.
#[test]
fn blocked_reads_the_listing_not_the_column() {
    let config = || when("due", "blocked()");
    let clear = [titled("a", "A")];
    assert!(!captured(config(), &clear, ColumnId::Due));

    let mut waiting = titled("b", "B");
    waiting.links = vec![dep_edge("a.typ")];
    let stuck = [titled("a", "A"), waiting];
    assert!(captured(config(), &stuck, ColumnId::Due));
}

/// Each column is resolved against its own facts: one run, one expression text,
/// two different answers, because `nonempty()` means this column's cells.
#[test]
fn facts_do_not_leak_between_columns() {
    let mut config = floor_config();
    for column in ["priority", "title"] {
        config.ls.columns.insert(
            column.to_owned(),
            ColumnConfig {
                when: Some("nonempty()".to_owned()),
            },
        );
    }
    // `title` carries text and `priority` does not, so the same expression must
    // hold for one and not the other.
    let order = order(config, &[titled("a", "A")], false, "table");
    assert!(order.contains(&ColumnId::Title));
    assert!(!order.contains(&ColumnId::Priority));
}
