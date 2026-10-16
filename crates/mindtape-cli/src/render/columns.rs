//! Column descriptors for the `ls` table.
//!
//! One place maps each [`mindtape::LS_COLUMNS`] name to its header label and
//! its per-task [`CellSpec`] builder. `table.rs` drives rendering off a
//! resolved list of these, so column order and visibility are data, not
//! hardcoded control flow.
//!
//! # Why visibility resolves here
//!
//! Whether a column renders is `[ls.columns.<name>] when`, and that expression
//! reads facts about the rows — whether any cell carries text, whether the rows
//! disagree, whether anything is blocked (`nonempty()`, `contested()`,
//! `blocked()`) — plus whether the caller named the column (`named()`). This
//! module is where the rows are, so this is where the expression is evaluated,
//! against the [`mindtape::ColumnFacts`] built here per column.
//!
//! No visibility policy lives in this file. The per-column defaults are lines
//! of `crates/mindtape/src/config/defaults.toml`, and a column with no `when` at all
//! does not render.

use crate::diag::Result;
use mindtape::{
    ColumnFacts, Condition, ConditionEnv, Config, ReadyState, StemHashes, Task, format_iso,
    today_iso, waits_on,
};

use super::RenderCtx;
use super::color::parse_color;
use super::field::{CellSpec, Field};
use super::fields::{Status as StatusField, priority_cell_spec};
use super::file_path;
use crate::diag::Diag;

/// The fixed set of `ls` columns. Slice order is the built-in render order and
/// matches [`mindtape::LS_COLUMNS`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColumnId {
    StemHash,
    Status,
    Ready,
    Priority,
    Due,
    Assignee,
    Title,
    Why,
    Children,
    Progress,
    File,
}

/// Built-in column order, used when `[ls] columns-order` is unset. `File` is
/// last: under the shipped floor it renders only when named explicitly or when
/// the output is a captured table, so appending it after `Title` cannot disturb
/// the built-in look of an unconfigured `mt ls` on a terminal.
///
/// `Assignee` sits with the other short scalars, before the flex `Title`,
/// rather than being tacked on at the end like `File`: when it renders at all
/// it is because the rows disagree about it, and that is metadata to read
/// alongside status and priority, not a trailing path.
pub const ORDER: [ColumnId; 11] = [
    ColumnId::StemHash,
    ColumnId::Status,
    ColumnId::Ready,
    ColumnId::Priority,
    ColumnId::Due,
    ColumnId::Assignee,
    ColumnId::Title,
    ColumnId::Why,
    ColumnId::Children,
    ColumnId::Progress,
    ColumnId::File,
];

impl ColumnId {
    /// Config / `columns-order` name (matches [`mindtape::LS_COLUMNS`]).
    #[must_use]
    pub fn key(self) -> &'static str {
        match self {
            ColumnId::StemHash => "slug#",
            ColumnId::Status => "status",
            ColumnId::Ready => "ready",
            ColumnId::Priority => "priority",
            ColumnId::Due => "due",
            ColumnId::Assignee => "assignee",
            ColumnId::Title => "title",
            ColumnId::Why => "why",
            ColumnId::Children => "children",
            ColumnId::Progress => "progress",
            ColumnId::File => "file",
        }
    }

    /// Header label printed in the table's first line.
    #[must_use]
    pub fn header(self) -> &'static str {
        match self {
            ColumnId::StemHash => "SLUG#",
            ColumnId::Status => "STATUS",
            ColumnId::Ready => "READY",
            ColumnId::Priority => "PRIORITY",
            ColumnId::Due => "DUE",
            ColumnId::Assignee => "ASSIGNEE",
            ColumnId::Title => "TITLE",
            ColumnId::Why => "WHY",
            ColumnId::Children => "CHILDREN",
            ColumnId::Progress => "PROGRESS",
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

    /// Build this column's cell for one task. `hash` is the task's stem hash,
    /// the `slug#` column's text.
    #[must_use]
    pub fn cell(self, hash: &str, task: &Task, ctx: RenderCtx<'_>) -> CellSpec {
        let (config, readiness) = (ctx.config, ctx.readiness);
        match self {
            ColumnId::StemHash => plain(hash.to_owned()),
            ColumnId::Status => status_cell(task, config),
            ColumnId::Ready => ready_cell(readiness.state(task), config),
            ColumnId::Priority => priority_cell(task, config),
            ColumnId::Due => due_cell(task, config),
            ColumnId::Assignee => plain(task.assignee.clone().unwrap_or_default()),
            ColumnId::Title => plain(task.title.clone()),
            ColumnId::Why => plain(waits_on(readiness.blockers(task))),
            ColumnId::Children => plain(children_text(task)),
            ColumnId::Progress => plain(progress_text(task)),
            ColumnId::File => plain(file_path::cell_text(ctx.root, &task.file)),
        }
    }

    /// The listing facts this column's `when` expression is decided against —
    /// the four it reads as `nonempty()`, `contested()`, `named()` and
    /// `blocked()`.
    ///
    /// `nonempty` and `contested` come from one pass over the rows, which
    /// stops as soon as both are settled: a column whose first two cells
    /// differ and carry text never touches row three. A blank counts as its
    /// own value for `contested`, so a half-filled column is contested — the
    /// blanks are exactly what a reader wants to see there.
    #[must_use]
    pub fn facts(self, rows: &[(String, &Task)], ctx: RenderCtx<'_>, named: bool) -> ColumnFacts {
        let mut facts = ColumnFacts {
            named,
            blocked: rows.iter().any(|(_, task)| ctx.readiness.is_blocked(task)),
            ..ColumnFacts::default()
        };
        let mut first: Option<String> = None;
        for (hash, task) in rows {
            let text = self.cell(hash, task, ctx).text;
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

    /// Resolve this column's visibility for one run: evaluate
    /// `[ls.columns.<key>] when` against the facts `env` carries.
    ///
    /// A column the config does not describe — no entry, or an entry with no
    /// `when` — does not render. The embedded floor carries one per column, so
    /// reaching that case means a config layer deleted the column's answer, not
    /// that a built-in one is waiting.
    ///
    /// # Errors
    /// Returns an error naming the column if the expression fails to parse or
    /// evaluate. Config load already validates every one of them, so this only
    /// recurs for a config built in memory rather than parsed.
    fn visible(self, config: &Config, env: &ConditionEnv) -> Result<bool> {
        let key = self.key();
        let Some(expr) = config
            .ls
            .columns
            .get(key)
            .and_then(|column| column.when.as_deref())
        else {
            return Ok(false);
        };
        Condition::parse(expr)
            .and_then(|condition| condition.matches(env))
            .map_err(|err| {
                Diag::from(err)
                    .rule(format!("ls.columns.{key}.when"))
                    .value(expr)
            })
    }
}

/// Each task paired with its stem hash from `hashes`, in listing order: the
/// rows every renderer walks.
#[must_use]
pub fn rows<'t>(tasks: &'t [Task], hashes: &StemHashes) -> Vec<(String, &'t Task)> {
    tasks
        .iter()
        .map(|task| {
            (
                hashes.hash_of(&task.id).unwrap_or_default().to_owned(),
                task,
            )
        })
        .collect()
}

/// The render order (built-in or `[ls] columns-order`), filtered to the
/// columns that resolve visible for this task list.
///
/// Shared by every `ls` renderer (`table`, `csv`) so they agree on which
/// columns render and in what order — only their per-cell formatting differs.
///
/// Each column is resolved against `ctx.env` with its own [`ColumnFacts`]
/// swapped in, so a `when` asking `nonempty()` gets this column's answer and
/// the next column's `when` gets its own.
///
/// # Errors
/// Returns an error if a column's `when` fails to parse or evaluate (see
/// [`ColumnId::visible`]).
pub fn resolve_order(rows: &[(String, &Task)], ctx: RenderCtx<'_>) -> Result<Vec<ColumnId>> {
    // Whether the render order was explicitly named — `cmd/ls/mod.rs` assigns
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
        env.column = col.facts(rows, ctx, named);
        if col.visible(ctx.config, &env)? {
            visible.push(col);
        }
    }
    Ok(visible)
}

/// Per-row hyperlink target URIs, shared by every column OSC 8 wraps (`slug#` and
/// `file` — see `render/link.rs`). The target is a property of the row (the
/// task's file), not of any one column, so this is built once per render and
/// the same list is attached to both columns rather than recomputed per
/// column. Built from `[ls] hyperlink-target`, defaulting to
/// [`file_path::DEFAULT_HYPERLINK_TARGET`]; `table.rs` decides whether to
/// actually apply it (`--hyperlinks`).
#[must_use]
pub fn row_hyperlink_targets(rows: &[(String, &Task)], ctx: RenderCtx<'_>) -> Vec<String> {
    let template = ctx
        .config
        .ls
        .hyperlink_target
        .as_deref()
        .unwrap_or(file_path::DEFAULT_HYPERLINK_TARGET);
    rows.iter()
        .map(|(_, task)| file_path::hyperlink_target(ctx.root, &task.file, template))
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

/// The child count, blank for a task with none — a leaf is not a parent with
/// zero children to a reader scanning the column.
fn children_text(task: &Task) -> String {
    if task.children == 0 {
        String::new()
    } else {
        task.children.to_string()
    }
}

/// Progress as `done/children` (`3/5`), blank with no children. The table
/// shows the counts a reader can check; the filter language, `--json` and CSV
/// carry the same value as the `0..1` fraction (`progress == 1`).
fn progress_text(task: &Task) -> String {
    if task.children == 0 {
        String::new()
    } else {
        format!("{}/{}", task.children_done, task.children)
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
