//! Borderless, copy-friendly list rendering.
//!
//! Replaces the box-drawing table with space-aligned columns: one line per task
//! plus a single header line — greppable, diffable, paste-friendly. Colour and
//! glyphs activate only on a TTY; piped output is plain.

use std::io::IsTerminal;

use mindtape::{Config, Readiness, ReadyState, Task, format_iso, today_iso};

use super::color::{Painter, is_color_active, parse_color};
use super::field::{CellSpec, DensityMode, Field, cell_content, content_width, render_cell};
use super::fields::{Status as StatusField, priority_cell_spec};
use super::glyph::GlyphMode;
use super::width::{display_width, pad_to_width, truncate_to_width};
use crate::handle::{DEFAULT_CHECK_LEN, make_handles};

const GAP: &str = "  ";
const TITLE_MIN: usize = 16;

/// The rendered cells of one task row.
struct Row {
    handle: CellSpec,
    status: CellSpec,
    ready: CellSpec,
    priority: CellSpec,
    due: CellSpec,
    title: CellSpec,
}

/// Per-column display widths.
struct Widths {
    handle: usize,
    status: usize,
    ready: usize,
    priority: usize,
    due: usize,
    title: usize,
}

/// Render `tasks` as a borderless, space-aligned table. The dependency-readiness
/// column appears only when some task is blocked, so a backlog with no contested
/// dependencies stays uncluttered.
#[must_use]
pub fn render(tasks: &[Task], config: &Config, readiness: &Readiness) -> String {
    let glyph_mode = glyph_mode();
    let painter = Painter::new(is_color_active());
    let show_ready = tasks.iter().any(|task| readiness.is_blocked(task));
    let rows = build_rows(tasks, config, readiness);
    let cols = widths(&rows, glyph_mode, show_ready);
    let mut out = String::new();
    push_header(&mut out, &cols);
    for row in &rows {
        push_row(&mut out, row, &cols, glyph_mode, painter);
    }
    out
}

/// Emoji on a TTY; no glyphs when piped (keeps pipe output plain).
fn glyph_mode() -> GlyphMode {
    if std::io::stdout().is_terminal() {
        GlyphMode::Emoji
    } else {
        GlyphMode::None
    }
}

fn plain(text: String) -> CellSpec {
    CellSpec {
        text,
        glyph: None,
        color: None,
    }
}

fn build_rows(tasks: &[Task], config: &Config, readiness: &Readiness) -> Vec<Row> {
    make_handles(tasks, DEFAULT_CHECK_LEN)
        .into_iter()
        .map(|(handle, task)| Row {
            handle: plain(handle),
            status: status_cell(task, config),
            ready: ready_cell(readiness.state(task)),
            priority: priority_cell(task, config),
            due: due_cell(task, config),
            title: plain(task.title.clone()),
        })
        .collect()
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
fn due_cell(task: &Task, config: &Config) -> CellSpec {
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

fn widths(rows: &[Row], glyph_mode: GlyphMode, show_ready: bool) -> Widths {
    let full = DensityMode::Full;
    let handle = col_width(
        "#",
        rows.iter()
            .map(|row| content_width(&row.handle, glyph_mode, full)),
    );
    let status = col_width(
        "STATUS",
        rows.iter()
            .map(|row| content_width(&row.status, glyph_mode, full)),
    );
    let ready = if show_ready {
        col_width(
            "READY",
            rows.iter()
                .map(|row| content_width(&row.ready, glyph_mode, full)),
        )
    } else {
        0
    };
    let priority = col_width(
        "PRIORITY",
        rows.iter()
            .map(|row| content_width(&row.priority, glyph_mode, full)),
    );
    let due = col_width(
        "DUE",
        rows.iter()
            .map(|row| content_width(&row.due, glyph_mode, full)),
    );
    // Gaps before the title column: one per preceding column boundary.
    let gaps = if show_ready { 5 } else { 4 };
    let title = title_width(rows, handle + status + ready + priority + due, gaps);
    Widths {
        handle,
        status,
        ready,
        priority,
        due,
        title,
    }
}

fn col_width(header: &str, cells: impl Iterator<Item = usize>) -> usize {
    cells.fold(display_width(header), usize::max)
}

fn title_width(rows: &[Row], others: usize, gaps: usize) -> usize {
    let natural = col_width(
        "TITLE",
        rows.iter()
            .map(|row| content_width(&row.title, GlyphMode::None, DensityMode::Full)),
    );
    match terminal_width() {
        Some(term) => {
            let avail = term.saturating_sub(others + GAP.len() * gaps);
            natural.min(avail).max(TITLE_MIN)
        }
        None => natural,
    }
}

fn terminal_width() -> Option<usize> {
    terminal_size::terminal_size().map(|(width, _)| width.0 as usize)
}

fn push_header(out: &mut String, cols: &Widths) {
    let mut cells = vec![
        pad_to_width("#", cols.handle),
        pad_to_width("STATUS", cols.status),
    ];
    if cols.ready != 0 {
        cells.push(pad_to_width("READY", cols.ready));
    }
    cells.push(pad_to_width("PRIORITY", cols.priority));
    cells.push(pad_to_width("DUE", cols.due));
    cells.push("TITLE".to_owned());
    out.push_str(&cells.join(GAP));
    out.push('\n');
}

fn push_row(out: &mut String, row: &Row, cols: &Widths, glyph_mode: GlyphMode, painter: Painter) {
    let full = DensityMode::Full;
    let mut cells = vec![
        render_cell(&row.handle, glyph_mode, full, painter, cols.handle),
        render_cell(&row.status, glyph_mode, full, painter, cols.status),
    ];
    // A zero-width readiness column means it was gated off (no blocked task).
    if cols.ready != 0 {
        cells.push(render_cell(
            &row.ready, glyph_mode, full, painter, cols.ready,
        ));
    }
    cells.push(render_cell(
        &row.priority,
        glyph_mode,
        full,
        painter,
        cols.priority,
    ));
    cells.push(render_cell(&row.due, glyph_mode, full, painter, cols.due));
    cells.push(render_title(&row.title, glyph_mode, painter, cols.title));
    out.push_str(&cells.join(GAP));
    out.push('\n');
}

/// Title is the last column: truncate to width but do not pad (no trailing run).
fn render_title(spec: &CellSpec, glyph_mode: GlyphMode, painter: Painter, width: usize) -> String {
    let content = cell_content(spec, glyph_mode, DensityMode::Full);
    let fitted = if display_width(&content) > width {
        truncate_to_width(&content, width)
    } else {
        content
    };
    match spec.color {
        Some(style) => painter.paint(&fitted, style),
        None => fitted,
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use mindtape::config::{Legend, LegendEntry};
    use mindtape::numeric::NumValue;
    use mindtape::test_support::bare_task;
    use mindtape::{Config, Readiness, Status, Task};

    use super::{due_cell, render};

    fn task(id: &str, title: &str, status: &str) -> Task {
        let mut t = bare_task(id);
        t.title = title.to_owned();
        t.status = Some(Status {
            name: status.to_owned(),
            date: None,
            note: None,
        });
        t.priority = Some(NumValue::Num(5.0));
        t
    }

    fn task_with_due(id: &str, status: &str, due: &str) -> Task {
        let mut t = task(id, id, status);
        t.due = Some(due.to_owned());
        t
    }

    fn config_with_terminal(terminal_statuses: &[&str]) -> Config {
        let mut config = Config::default();
        config.tasks.status = Legend {
            entries: terminal_statuses
                .iter()
                .map(|name| LegendEntry {
                    value: (*name).to_owned(),
                    terminal: true,
                    ..LegendEntry::default()
                })
                .collect(),
        };
        config
    }

    #[test]
    fn one_line_per_task_plus_header() {
        let out = render(
            &[task("a", "A", "wip"), task("b", "B", "done")],
            &Config::default(),
            &Readiness::empty(),
        );
        assert_eq!(out.lines().count(), 3);
    }

    #[test]
    fn no_box_drawing_chars() {
        let out = render(
            &[task("a", "A", "wip")],
            &Config::default(),
            &Readiness::empty(),
        );
        for ch in ['│', '┆', '─', '┌', '└', '╞'] {
            assert!(!out.contains(ch), "unexpected box char {ch:?}");
        }
    }

    #[test]
    fn header_and_content_present() {
        let out = render(
            &[task("auth", "Rewrite auth", "wip")],
            &Config::default(),
            &Readiness::empty(),
        );
        assert!(out.contains("TITLE"));
        assert!(out.contains("Rewrite auth"));
        assert!(out.contains("wip"));
        assert!(out.lines().next().unwrap().contains('#'));
    }

    #[test]
    fn readiness_column_appears_only_when_blocked() {
        // No blocked task: column stays hidden.
        let clean = render(
            &[task("a", "A", "wip")],
            &Config::default(),
            &Readiness::empty(),
        );
        assert!(!clean.contains("READY"));

        // `b` depends on the non-terminal `a`, so `b` is blocked and the column
        // appears with its label (glyphs are off in the test's non-TTY output).
        let mut blocked = task("b", "B", "wip");
        blocked.depends_on = vec!["a.typ".to_owned()];
        let tasks = vec![task("a", "A", "wip"), blocked];
        let readiness = Readiness::compute(&tasks, &Config::default());
        let out = render(&tasks, &Config::default(), &readiness);
        assert!(out.contains("READY"));
        assert!(out.contains("blocked"));
    }

    #[test]
    fn due_cell_overdue_non_terminal_is_red() {
        let task = task_with_due("t", "wip", "2020-01-01");
        let config = config_with_terminal(&["done"]);
        let cell = due_cell(&task, &config);
        assert!(cell.color.is_some(), "expected red color for overdue task");
    }

    #[test]
    fn due_cell_overdue_terminal_is_plain() {
        let task = task_with_due("t", "done", "2020-01-01");
        let config = config_with_terminal(&["done"]);
        let cell = due_cell(&task, &config);
        assert!(cell.color.is_none(), "terminal task must not be accented");
    }

    #[test]
    fn due_cell_future_due_is_plain() {
        let task = task_with_due("t", "wip", "2999-12-31");
        let config = config_with_terminal(&["done"]);
        let cell = due_cell(&task, &config);
        assert!(cell.color.is_none(), "future due must not be accented");
    }

    #[test]
    fn due_cell_applies_date_format() {
        let task = task_with_due("t", "wip", "2026-06-27");
        let mut config = config_with_terminal(&["done"]);
        config.ls.date_format = Some("DD.MM.YYYY".to_owned());
        let cell = due_cell(&task, &config);
        assert_eq!(
            cell.text, "27.06.2026",
            "date format must rewrite displayed text"
        );
    }

    #[test]
    fn due_cell_overdue_formatted_text_still_red() {
        // Formatting must not interfere with the overdue colour.
        let task = task_with_due("t", "wip", "2020-01-01");
        let mut config = config_with_terminal(&["done"]);
        config.ls.date_format = Some("DD.MM.YYYY".to_owned());
        let cell = due_cell(&task, &config);
        assert!(
            cell.color.is_some(),
            "overdue must still be red after formatting"
        );
        assert_eq!(cell.text, "01.01.2020");
    }
}
