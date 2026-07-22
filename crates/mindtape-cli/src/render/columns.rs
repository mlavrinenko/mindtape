//! Column descriptors for the `ls` table.
//!
//! One place maps each [`mindtape::LS_COLUMNS`] name to its header label, its
//! per-task [`CellSpec`] builder, and its auto-significance test. `table.rs`
//! drives rendering off a resolved list of these, so column order and
//! visibility are data, not hardcoded control flow.

use mindtape::{Config, Readiness, ReadyState, Task, Visibility, format_iso, today_iso};

use super::color::parse_color;
use super::field::{CellSpec, Field};
use super::fields::{Status as StatusField, priority_cell_spec};

/// The fixed set of `ls` columns. Slice order is the built-in render order and
/// matches [`mindtape::LS_COLUMNS`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColumnId {
    Handle,
    Status,
    Ready,
    Priority,
    Due,
    Title,
}

/// Built-in column order, used when `[ls] columns-order` is unset.
pub const ORDER: [ColumnId; 6] = [
    ColumnId::Handle,
    ColumnId::Status,
    ColumnId::Ready,
    ColumnId::Priority,
    ColumnId::Due,
    ColumnId::Title,
];

impl ColumnId {
    /// Config / `columns-order` name (matches [`mindtape::LS_COLUMNS`]).
    #[must_use]
    pub fn key(self) -> &'static str {
        match self {
            ColumnId::Handle => "#",
            ColumnId::Status => "status",
            ColumnId::Ready => "ready",
            ColumnId::Priority => "priority",
            ColumnId::Due => "due",
            ColumnId::Title => "title",
        }
    }

    /// Header label printed in the table's first line.
    #[must_use]
    pub fn header(self) -> &'static str {
        match self {
            ColumnId::Handle => "#",
            ColumnId::Status => "STATUS",
            ColumnId::Ready => "READY",
            ColumnId::Priority => "PRIORITY",
            ColumnId::Due => "DUE",
            ColumnId::Title => "TITLE",
        }
    }

    /// Resolve a `columns-order` name to its column. Config validation already
    /// rejects unknown names, so callers can treat `None` as a no-op.
    #[must_use]
    pub fn from_key(key: &str) -> Option<Self> {
        ORDER.into_iter().find(|col| col.key() == key)
    }

    /// The flex column absorbs remaining terminal width; the rest size to their
    /// natural content. Only `title` flexes today.
    #[must_use]
    pub fn is_flex(self) -> bool {
        matches!(self, ColumnId::Title)
    }

    /// Build this column's cell for one task. `handle` is the precomputed,
    /// abbreviated identifier shared with the `#` column.
    #[must_use]
    pub fn cell(
        self,
        handle: &str,
        task: &Task,
        config: &Config,
        readiness: &Readiness,
    ) -> CellSpec {
        match self {
            ColumnId::Handle => plain(handle.to_owned()),
            ColumnId::Status => status_cell(task, config),
            ColumnId::Ready => ready_cell(readiness.state(task), config),
            ColumnId::Priority => priority_cell(task, config),
            ColumnId::Due => due_cell(task, config),
            ColumnId::Title => plain(task.title.clone()),
        }
    }

    /// Whether the column carries something worth a column under `auto`
    /// visibility. `#` and `title` are never blank; `ready` shows only when a
    /// task is contested; the rest show when any cell has text.
    #[must_use]
    pub fn auto_significant(
        self,
        rows: &[(String, &Task)],
        config: &Config,
        readiness: &Readiness,
    ) -> bool {
        match self {
            ColumnId::Handle | ColumnId::Title => true,
            ColumnId::Ready => rows.iter().any(|(_, task)| readiness.is_blocked(task)),
            _ => rows
                .iter()
                .any(|(handle, task)| !self.cell(handle, task, config, readiness).text.is_empty()),
        }
    }

    /// Resolve `[ls.columns.<key>] visibility` against `significant` (the
    /// `auto` outcome): `visible` always renders, `hidden` never, `auto` defers.
    #[must_use]
    pub fn visible(self, config: &Config, significant: bool) -> bool {
        let visibility = config
            .ls
            .columns
            .get(self.key())
            .map_or(Visibility::default(), |col| col.visibility);
        match visibility {
            Visibility::Visible => true,
            Visibility::Hidden => false,
            Visibility::Auto => significant,
        }
    }
}

/// The render order (built-in or `[ls] columns-order`), filtered to the
/// columns that resolve visible for this task list.
///
/// Shared by every `ls` renderer (`table`, `csv`) so they agree on which
/// columns render and in what order — only their per-cell formatting differs.
#[must_use]
pub fn resolve_order(
    config: &Config,
    handles: &[(String, &Task)],
    readiness: &Readiness,
) -> Vec<ColumnId> {
    let order: Vec<ColumnId> = if config.ls.columns_order.is_empty() {
        ORDER.to_vec()
    } else {
        config
            .ls
            .columns_order
            .iter()
            .filter_map(|key| ColumnId::from_key(key))
            .collect()
    };
    order
        .into_iter()
        .filter(|col| {
            let significant = col.auto_significant(handles, config, readiness);
            col.visible(config, significant)
        })
        .collect()
}

fn plain(text: String) -> CellSpec {
    CellSpec {
        text,
        glyph: None,
        color: None,
    }
}

/// A readiness cell: a glyph and short label, coloured per `[ls.ready]`
/// (green/ready, red/blocked by embedded-floor default). Settled (terminal)
/// tasks render blank.
#[must_use]
pub fn ready_cell(state: ReadyState, config: &Config) -> CellSpec {
    let ready = &config.ls.ready;
    match state {
        ReadyState::Ready => CellSpec {
            text: ready
                .ready_label
                .clone()
                .unwrap_or_else(|| "ready".to_owned()),
            glyph: Some(ready.ready_glyph.clone().unwrap_or_else(|| "▶".to_owned())),
            color: parse_color(ready.ready_color.as_deref().unwrap_or("green")),
        },
        ReadyState::Blocked => CellSpec {
            text: ready
                .blocked_label
                .clone()
                .unwrap_or_else(|| "blocked".to_owned()),
            glyph: Some(
                ready
                    .blocked_glyph
                    .clone()
                    .unwrap_or_else(|| "🔒".to_owned()),
            ),
            color: parse_color(ready.blocked_color.as_deref().unwrap_or("red")),
        },
        ReadyState::Settled => plain(String::new()),
    }
}

fn status_cell(task: &Task, config: &Config) -> CellSpec {
    match StatusField::values(task).first() {
        Some(name) => StatusField::cell_spec(name, config),
        None => plain(String::new()),
    }
}

fn priority_cell(task: &Task, config: &Config) -> CellSpec {
    match &task.priority {
        Some(value) => priority_cell_spec(value, config),
        None => plain(String::new()),
    }
}

/// Due cell: accented with `[ls] overdue-color` (red by embedded-floor
/// default) when the task is overdue (past due, non-terminal), else plain.
///
/// The overdue check compares the raw ISO value; only the displayed text is
/// reformatted via the configured `date_format` (or the default `YYYY-MM-DD`).
#[must_use]
pub fn due_cell(task: &Task, config: &Config) -> CellSpec {
    let raw = task.due.as_deref().unwrap_or_default();
    let overdue = task
        .due
        .as_deref()
        .is_some_and(|due| due < today_iso().as_str())
        && !task.is_terminal(&config.tasks.status.terminal());
    let fmt = config.ls.date_format.as_deref().unwrap_or("YYYY-MM-DD");
    let text = if raw.is_empty() {
        String::new()
    } else {
        format_iso(raw, fmt)
    };
    let overdue_color = config.ls.overdue_color.as_deref().unwrap_or("red");
    CellSpec {
        text,
        glyph: None,
        color: if overdue {
            parse_color(overdue_color)
        } else {
            None
        },
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
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
        assert!(ColumnId::Handle.auto_significant(&[], &Config::default(), &Readiness::default()));
    }

    #[test]
    fn title_always_significant() {
        assert!(ColumnId::Title.auto_significant(&[], &Config::default(), &Readiness::default()));
    }

    #[test]
    fn ready_not_significant_for_uncontested() {
        let task = bare_task("t");
        let config = Config::default();
        let readiness = Readiness::compute(std::slice::from_ref(&task), &config);
        assert!(
            !ColumnId::Ready.auto_significant(&rows(&[("1c", &task)]), &config, &readiness),
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
            ColumnId::Priority.auto_significant(&rows(&[("1c", &task)]), &config, &readiness),
            "priority column must be significant when at least one task has a value"
        );
    }

    #[test]
    fn priority_not_significant_when_unset() {
        let task = bare_task("t");
        let config = Config::default();
        let readiness = Readiness::compute(std::slice::from_ref(&task), &config);
        assert!(
            !ColumnId::Priority.auto_significant(&rows(&[("1c", &task)]), &config, &readiness),
            "priority column must not be significant when no task has a value"
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
}
