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
            ColumnId::Ready => ready_cell(readiness.state(task)),
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

/// A readiness cell: a glyph and short label, coloured green (ready) or red
/// (blocked). Settled (terminal) tasks render blank.
fn ready_cell(state: ReadyState) -> CellSpec {
    match state {
        ReadyState::Ready => CellSpec {
            text: "ready".to_owned(),
            glyph: Some("▶".to_owned()),
            color: parse_color("green"),
        },
        ReadyState::Blocked => CellSpec {
            text: "blocked".to_owned(),
            glyph: Some("🔒".to_owned()),
            color: parse_color("red"),
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

/// Due cell: red when the task is overdue (past due, non-terminal), else plain.
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
    CellSpec {
        text,
        glyph: None,
        color: if overdue { parse_color("red") } else { None },
    }
}
