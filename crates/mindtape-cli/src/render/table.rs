//! Borderless, copy-friendly list rendering.
//!
//! Replaces the box-drawing table with space-aligned columns: one line per task
//! plus a single header line — greppable, diffable, paste-friendly. Colour and
//! glyphs activate only on a TTY; piped output is plain.

use std::io::IsTerminal;

use mindtape::{Config, Task};

use super::color::{Painter, is_color_active};
use super::field::{CellSpec, DensityMode, Field, cell_content, content_width, render_cell};
use super::fields::Status as StatusField;
use super::glyph::GlyphMode;
use super::width::{display_width, pad_to_width, truncate_to_width};
use crate::handle::{DEFAULT_CHECK_LEN, make_handles};

const GAP: &str = "  ";
const TITLE_MIN: usize = 16;

/// The rendered cells of one task row.
struct Row {
    handle: CellSpec,
    status: CellSpec,
    priority: CellSpec,
    due: CellSpec,
    title: CellSpec,
}

/// Per-column display widths.
struct Widths {
    handle: usize,
    status: usize,
    priority: usize,
    due: usize,
    title: usize,
}

/// Render `tasks` as a borderless, space-aligned table.
#[must_use]
pub fn render(tasks: &[Task], config: &Config) -> String {
    let glyph_mode = glyph_mode();
    let painter = Painter::new(is_color_active());
    let rows = build_rows(tasks, config);
    let cols = widths(&rows, glyph_mode);
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

fn build_rows(tasks: &[Task], config: &Config) -> Vec<Row> {
    make_handles(tasks, DEFAULT_CHECK_LEN)
        .into_iter()
        .map(|(handle, task)| Row {
            handle: plain(handle),
            status: status_cell(task, config),
            priority: plain(task.priority.clone().unwrap_or_default()),
            due: plain(task.due.clone().unwrap_or_default()),
            title: plain(task.title.clone()),
        })
        .collect()
}

fn status_cell(task: &Task, config: &Config) -> CellSpec {
    match StatusField::values(task).first() {
        Some(name) => StatusField::cell_spec(name, config),
        None => plain(String::new()),
    }
}

fn widths(rows: &[Row], glyph_mode: GlyphMode) -> Widths {
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
    let title = title_width(rows, handle + status + priority + due);
    Widths {
        handle,
        status,
        priority,
        due,
        title,
    }
}

fn col_width(header: &str, cells: impl Iterator<Item = usize>) -> usize {
    cells.fold(display_width(header), usize::max)
}

fn title_width(rows: &[Row], others: usize) -> usize {
    let natural = col_width(
        "TITLE",
        rows.iter()
            .map(|row| content_width(&row.title, GlyphMode::None, DensityMode::Full)),
    );
    match terminal_width() {
        Some(term) => {
            let avail = term.saturating_sub(others + GAP.len() * 4);
            natural.min(avail).max(TITLE_MIN)
        }
        None => natural,
    }
}

fn terminal_width() -> Option<usize> {
    terminal_size::terminal_size().map(|(width, _)| width.0 as usize)
}

fn push_header(out: &mut String, cols: &Widths) {
    let cells = [
        pad_to_width("#", cols.handle),
        pad_to_width("STATUS", cols.status),
        pad_to_width("PRIORITY", cols.priority),
        pad_to_width("DUE", cols.due),
        "TITLE".to_owned(),
    ];
    out.push_str(&cells.join(GAP));
    out.push('\n');
}

fn push_row(out: &mut String, row: &Row, cols: &Widths, glyph_mode: GlyphMode, painter: Painter) {
    let full = DensityMode::Full;
    let cells = [
        render_cell(&row.handle, glyph_mode, full, painter, cols.handle),
        render_cell(&row.status, glyph_mode, full, painter, cols.status),
        render_cell(&row.priority, glyph_mode, full, painter, cols.priority),
        render_cell(&row.due, glyph_mode, full, painter, cols.due),
        render_title(&row.title, glyph_mode, painter, cols.title),
    ];
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
    use std::collections::BTreeMap;

    use mindtape::{Config, Status, Task};

    use super::render;

    fn task(id: &str, title: &str, status: &str) -> Task {
        Task {
            id: id.to_owned(),
            file: format!("tasks/{id}.typ").into(),
            title: title.to_owned(),
            status: Some(Status {
                name: status.to_owned(),
                date: None,
                note: None,
            }),
            priority: Some("normal".to_owned()),
            grade: None,
            kind: None,
            estimate: BTreeMap::new(),
            depends_on: Vec::new(),
            relates_to: Vec::new(),
            supersedes: Vec::new(),
            duplicates: Vec::new(),
            tags: Vec::new(),
            due: None,
            start: None,
            reviewed: None,
            extras: BTreeMap::new(),
        }
    }

    #[test]
    fn one_line_per_task_plus_header() {
        let out = render(
            &[task("a", "A", "wip"), task("b", "B", "done")],
            &Config::default(),
        );
        assert_eq!(out.lines().count(), 3);
    }

    #[test]
    fn no_box_drawing_chars() {
        let out = render(&[task("a", "A", "wip")], &Config::default());
        for ch in ['│', '┆', '─', '┌', '└', '╞'] {
            assert!(!out.contains(ch), "unexpected box char {ch:?}");
        }
    }

    #[test]
    fn header_and_content_present() {
        let out = render(&[task("auth", "Rewrite auth", "wip")], &Config::default());
        assert!(out.contains("TITLE"));
        assert!(out.contains("Rewrite auth"));
        assert!(out.contains("wip"));
        assert!(out.lines().next().unwrap().contains('#'));
    }
}
