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
use super::file_path;

/// The fixed set of `ls` columns. Slice order is the built-in render order and
/// matches [`mindtape::LS_COLUMNS`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColumnId {
    Handle,
    Status,
    Ready,
    Priority,
    Due,
    Assignee,
    Title,
    File,
}

/// Built-in column order, used when `[ls] columns-order` is unset. `File` is
/// last: it never renders unless named explicitly (see
/// [`ColumnId::auto_significant`]), so appending it after `Title` cannot
/// disturb the built-in look of an unconfigured `mt ls`.
///
/// `Assignee` sits with the other short scalars, before the flex `Title`,
/// rather than being tacked on at the end like `File`: when it renders at all
/// it is because the rows disagree about it, and that is metadata to read
/// alongside status and priority, not a trailing path.
pub const ORDER: [ColumnId; 8] = [
    ColumnId::Handle,
    ColumnId::Status,
    ColumnId::Ready,
    ColumnId::Priority,
    ColumnId::Due,
    ColumnId::Assignee,
    ColumnId::Title,
    ColumnId::File,
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
            ColumnId::Assignee => "assignee",
            ColumnId::Title => "title",
            ColumnId::File => "file",
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
            ColumnId::Assignee => "ASSIGNEE",
            ColumnId::Title => "TITLE",
            ColumnId::File => "FILE",
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
            ColumnId::Assignee => plain(task.assignee.clone().unwrap_or_default()),
            ColumnId::Title => plain(task.title.clone()),
            ColumnId::File => plain(file_path::cell_text(&task.file)),
        }
    }

    /// Whether the column carries something worth a column under `auto`
    /// visibility. `#` and `title` are never blank; `ready` shows only when a
    /// task is contested; `file` is never blank either but that must *not*
    /// make it significant — its cell always has text, so the ordinary rule
    /// would pin it permanently visible and there would be no opt-in at all.
    /// Its significance is instead `explicit_order`: whether the render order
    /// was explicitly named (`--columns`, a view's `columns`, an
    /// `[[ls.rules]]` columns override) rather than the built-in default.
    /// `assignee` is significant only when the rows disagree about it (see
    /// [`assignee_contested`]) — a listing already narrowed to one actor
    /// learns nothing from a column repeating that name. The rest show when
    /// any cell has text.
    #[must_use]
    pub fn auto_significant(
        self,
        rows: &[(String, &Task)],
        config: &Config,
        readiness: &Readiness,
        explicit_order: bool,
    ) -> bool {
        match self {
            ColumnId::Handle | ColumnId::Title => true,
            ColumnId::File => explicit_order,
            ColumnId::Ready => rows.iter().any(|(_, task)| readiness.is_blocked(task)),
            ColumnId::Assignee => assignee_contested(rows),
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
    // Whether the render order was explicitly named — `ls.rs` assigns
    // `columns_order` uniformly for `--columns`, a view's `columns`, and an
    // `[[ls.rules]]` columns override, so this one check covers every
    // explicit path and doubles as the `file` column's `auto` significance.
    let explicit_order = !config.ls.columns_order.is_empty();
    let order: Vec<ColumnId> = if explicit_order {
        config
            .ls
            .columns_order
            .iter()
            .filter_map(|key| ColumnId::from_key(key))
            .collect()
    } else {
        ORDER.to_vec()
    };
    order
        .into_iter()
        .filter(|col| {
            let significant = col.auto_significant(handles, config, readiness, explicit_order);
            col.visible(config, significant)
        })
        .collect()
}

/// Per-row hyperlink target URIs, shared by every column OSC 8 wraps (`#` and
/// `file` — see `render/link.rs`). The target is a property of the row (the
/// task's file), not of any one column, so this is built once per render and
/// the same list is attached to both columns rather than recomputed per
/// column. Built from `[ls] hyperlink-target`, defaulting to
/// [`file_path::DEFAULT_HYPERLINK_TARGET`]; `table.rs` decides whether to
/// actually apply it (`--hyperlinks`).
#[must_use]
pub fn row_hyperlink_targets(handles: &[(String, &Task)], config: &Config) -> Vec<String> {
    let template = config
        .ls
        .hyperlink_target
        .as_deref()
        .unwrap_or(file_path::DEFAULT_HYPERLINK_TARGET);
    handles
        .iter()
        .map(|(_, task)| file_path::hyperlink_target(&task.file, template))
        .collect()
}

/// Whether the listed rows disagree about their assignee — the `auto`
/// significance of the `assignee` column.
///
/// An absent assignee counts as its own value rather than being skipped, so a
/// half-assigned listing is contested (the blanks are exactly what you want to
/// see) while a wholly unassigned one is not. Short-circuits on the first row
/// that differs from the first.
#[must_use]
fn assignee_contested(rows: &[(String, &Task)]) -> bool {
    let mut values = rows
        .iter()
        .map(|(_, task)| task.assignee.as_deref().unwrap_or_default());
    let Some(first) = values.next() else {
        return false;
    };
    values.any(|value| value != first)
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
#[path = "columns_tests.rs"]
mod tests;
