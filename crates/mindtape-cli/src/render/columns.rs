//! Column descriptors for the `ls` table.
//!
//! One place maps each [`mindtape::LS_COLUMNS`] name to its header label, its
//! per-task [`CellSpec`] builder, and its auto-significance test. `table.rs`
//! drives rendering off a resolved list of these, so column order and
//! visibility are data, not hardcoded control flow.
//!
//! # Why visibility resolves here
//!
//! A column's `auto` significance is a fact about the rows — whether any cell
//! carries text, whether the rows disagree, whether anything is blocked — and a
//! `[[ls.columns.<name>.rules]]` condition can now ask for exactly those facts
//! (`nonempty()`, `contested()`, `named()`, `blocked()`). So the two cannot be
//! resolved in different places any more: this module is where the rows are, so
//! this is where the conditions are evaluated, against the same
//! [`mindtape::ColumnFacts`] `auto` itself reads.

use anyhow::{Context, Result};
use mindtape::{
    ColumnConfig, ColumnFacts, Condition, ConditionEnv, Config, Readiness, ReadyState, Task,
    Visibility, format_iso, today_iso,
};

use super::RenderCtx;
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

    /// The listing facts this column's visibility is decided from — the same
    /// four a `[[ls.columns.<name>.rules]]` condition reads as `nonempty()`,
    /// `contested()`, `named()` and `blocked()`.
    ///
    /// `nonempty` and `contested` come from one pass over the rows, which
    /// stops as soon as both are settled: a column whose first two cells
    /// differ and carry text never touches row three. A blank counts as its
    /// own value for `contested`, so a half-filled column is contested — the
    /// blanks are exactly what a reader wants to see there.
    #[must_use]
    pub fn facts(
        self,
        rows: &[(String, &Task)],
        config: &Config,
        readiness: &Readiness,
        named: bool,
    ) -> ColumnFacts {
        let mut facts = ColumnFacts {
            named,
            blocked: rows.iter().any(|(_, task)| readiness.is_blocked(task)),
            ..ColumnFacts::default()
        };
        let mut first: Option<String> = None;
        for (handle, task) in rows {
            let text = self.cell(handle, task, config, readiness).text;
            facts.nonempty |= !text.is_empty();
            match &first {
                None => first = Some(text),
                Some(seen) => facts.contested |= *seen != text,
            }
            if facts.nonempty && facts.contested {
                break;
            }
        }
        facts
    }

    /// Whether the column carries something worth a column under `auto`
    /// visibility — the built-in policy, one [`ColumnFacts`] field per arm.
    ///
    /// `#` and `title` are never blank; `ready` shows only when something is
    /// blocked, which is narrower than `nonempty` (a listing where every task
    /// is ready has a label in every cell and nothing to report). `file` is
    /// never blank either but that must *not* make it significant — its cell
    /// always has text, so the ordinary rule would pin it permanently visible
    /// and there would be no opt-in at all; its significance is `named`
    /// instead. `assignee` is significant only when the rows disagree — a
    /// listing already narrowed to one actor learns nothing from a column
    /// repeating that name. The rest show when any cell has text.
    #[must_use]
    pub fn auto_significant(self, facts: ColumnFacts) -> bool {
        match self {
            ColumnId::Handle | ColumnId::Title => true,
            ColumnId::File => facts.named,
            ColumnId::Ready => facts.blocked,
            ColumnId::Assignee => facts.contested,
            _ => facts.nonempty,
        }
    }

    /// Resolve this column's visibility for one run: the first matching
    /// `rules` entry, else `[ls.columns.<key>] visibility`; `visible` always
    /// renders, `hidden` never, `auto` defers to [`ColumnId::auto_significant`]
    /// over the facts `env` carries.
    fn visible(self, config: &Config, env: &ConditionEnv) -> Result<bool> {
        let visibility = rule_visibility(self.key(), config.ls.columns.get(self.key()), env)?;
        Ok(match visibility {
            Visibility::Visible => true,
            Visibility::Hidden => false,
            Visibility::Auto => self.auto_significant(env.column),
        })
    }
}

/// The visibility a column's `rules` resolve to: the first entry whose
/// condition holds (or that omits one) wins and beats the column's own
/// `visibility` key — rules ARE the override mechanism, so the other direction
/// would make them unwritable. No column config at all is the `auto` default.
///
/// # Errors
/// Returns an error naming the column and rule index if a condition fails to
/// parse or evaluate. Config load already validates every one of them, so this
/// only recurs for a config built in memory rather than parsed.
fn rule_visibility(
    key: &str,
    column: Option<&ColumnConfig>,
    env: &ConditionEnv,
) -> Result<Visibility> {
    let Some(column) = column else {
        return Ok(Visibility::default());
    };
    for (index, rule) in column.rules.iter().enumerate() {
        let holds = match &rule.condition {
            None => true,
            Some(expr) => Condition::parse(expr)
                .and_then(|condition| condition.matches(env))
                .with_context(|| format!("ls.columns.{key}.rules[{index}].condition {expr:?}"))?,
        };
        if holds {
            return Ok(rule.visibility);
        }
    }
    Ok(column.visibility)
}

/// The render order (built-in or `[ls] columns-order`), filtered to the
/// columns that resolve visible for this task list.
///
/// Shared by every `ls` renderer (`table`, `csv`) so they agree on which
/// columns render and in what order — only their per-cell formatting differs.
///
/// Each column is resolved against `ctx.env` with its own [`ColumnFacts`]
/// swapped in, so a condition asking `nonempty()` gets this column's answer
/// and the next column's condition gets its own.
///
/// # Errors
/// Returns an error if a column's `rules` condition fails to parse or
/// evaluate (see [`rule_visibility`]).
pub fn resolve_order(handles: &[(String, &Task)], ctx: RenderCtx<'_>) -> Result<Vec<ColumnId>> {
    // Whether the render order was explicitly named — `ls.rs` assigns
    // `columns_order` uniformly for `--columns`, a view's `columns`, and an
    // `[[ls.rules]]` columns override, so this one check covers every explicit
    // path. It is per-column `named()` too, and not by approximation: when the
    // order is explicit it IS the requested list, so every column reaching the
    // loop below was named by the caller.
    let named = !ctx.config.ls.columns_order.is_empty();
    let order: Vec<ColumnId> = if named {
        ctx.config
            .ls
            .columns_order
            .iter()
            .filter_map(|key| ColumnId::from_key(key))
            .collect()
    } else {
        ORDER.to_vec()
    };
    let mut env = ctx.env.clone();
    let mut visible = Vec::with_capacity(order.len());
    for col in order {
        env.column = col.facts(handles, ctx.config, ctx.readiness, named);
        if col.visible(ctx.config, &env)? {
            visible.push(col);
        }
    }
    Ok(visible)
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
#[cfg(test)]
#[path = "columns_tests_rules.rs"]
mod tests_rules;
