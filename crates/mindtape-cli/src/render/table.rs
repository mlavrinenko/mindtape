//! Borderless, copy-friendly list rendering.
//!
//! Replaces the box-drawing table with space-aligned columns: one line per task
//! plus a single header line — greppable, diffable, paste-friendly. Colour and
//! glyphs activate only on a TTY; piped output is plain.

use std::io::IsTerminal;

use anstyle::{Effects, Style};
use mindtape::{Config, Readiness, Task};

use super::color::{Painter, is_color_active};
use super::columns::{ColumnId, resolve_order};
use super::field::{CellSpec, DensityMode, cell_content, content_width, render_cell};
use super::glyph::GlyphMode;
use super::width::{display_width, pad_to_width, truncate_to_width};
use crate::handle::{DEFAULT_CHECK_LEN, make_handles};

const GAP: &str = "  ";
const TITLE_MIN: usize = 16;

/// A resolved visible column: its id, per-row cells, and display width.
struct Column {
    id: ColumnId,
    cells: Vec<CellSpec>,
    width: usize,
}

/// Render `tasks` as a borderless, space-aligned table.
///
/// The column set, order, and visibility come from `[ls.columns]` /
/// `[ls] columns-order`. Under the default `auto` visibility a column is
/// dropped when it carries nothing — so a backlog with no priorities, no due
/// dates, or no contested dependencies stays uncluttered.
#[must_use]
pub fn render(tasks: &[Task], config: &Config, readiness: &Readiness) -> String {
    render_inner(tasks, config, readiness, &[])
}

/// Render `tasks` with the interactive `#`-column highlight applied.
///
/// `highlighted[i]` marks whether the handle cell of display row `i` renders
/// reverse-video; a shorter slice leaves the remaining rows un-highlighted. The
/// invert effect rides the table's painter, so `--color never` drops it.
#[must_use]
pub fn render_highlighted(
    tasks: &[Task],
    config: &Config,
    readiness: &Readiness,
    highlighted: &[bool],
) -> String {
    render_inner(tasks, config, readiness, highlighted)
}

fn render_inner(
    tasks: &[Task],
    config: &Config,
    readiness: &Readiness,
    highlighted: &[bool],
) -> String {
    let glyph_mode = glyph_mode();
    let painter = Painter::new(is_color_active());
    let handles: Vec<(String, &Task)> = make_handles(tasks, DEFAULT_CHECK_LEN);
    let visible = resolve_order(config, &handles, readiness);
    let mut columns = build_columns(&visible, &handles, config, readiness, glyph_mode);
    apply_highlight(&mut columns, highlighted);
    render_grid(&columns, glyph_mode, painter)
}

/// Paint the `#` column's cells reverse-video for the highlighted rows. A no-op
/// when no row is flagged or the handle column is not visible.
fn apply_highlight(columns: &mut [Column], highlighted: &[bool]) {
    if highlighted.iter().all(|on| !on) {
        return;
    }
    let invert = Style::new().effects(Effects::INVERT);
    if let Some(col) = columns.iter_mut().find(|col| col.id == ColumnId::Handle) {
        for (cell, on) in col.cells.iter_mut().zip(highlighted) {
            if *on {
                cell.color = Some(invert);
            }
        }
    }
}

/// Emoji on a TTY; no glyphs when piped (keeps pipe output plain).
fn glyph_mode() -> GlyphMode {
    if std::io::stdout().is_terminal() {
        GlyphMode::Emoji
    } else {
        GlyphMode::None
    }
}

/// Build each visible column's cells and width. The flex column (`title`)
/// absorbs the remaining terminal width; the rest size to their content.
fn build_columns(
    visible: &[ColumnId],
    handles: &[(String, &Task)],
    config: &Config,
    readiness: &Readiness,
    glyph_mode: GlyphMode,
) -> Vec<Column> {
    let full = DensityMode::Full;
    let mut columns: Vec<Column> = visible
        .iter()
        .map(|&id| {
            let cells: Vec<CellSpec> = handles
                .iter()
                .map(|(handle, task)| id.cell(handle, task, config, readiness))
                .collect();
            let width = col_width(
                id.header(),
                cells
                    .iter()
                    .map(|cell| content_width(cell, glyph_mode, full)),
            );
            Column { id, cells, width }
        })
        .collect();
    apply_flex(&mut columns);
    columns
}

fn col_width(header: &str, cells: impl Iterator<Item = usize>) -> usize {
    cells.fold(display_width(header), usize::max)
}

/// Shrink the flex column (`title`) to the remaining terminal width: total
/// minus every other column and the inter-column gaps, clamped to
/// `[TITLE_MIN, natural]`. No flex column or no known terminal width: leave
/// natural widths untouched.
fn apply_flex(columns: &mut [Column]) {
    let Some(term) = terminal_width() else {
        return;
    };
    let others: usize = columns
        .iter()
        .filter(|col| !col.id.is_flex())
        .map(|col| col.width)
        .sum();
    let gaps = columns.len().saturating_sub(1);
    if let Some(col) = columns.iter_mut().find(|col| col.id.is_flex()) {
        let avail = term.saturating_sub(others + GAP.len() * gaps);
        col.width = col.width.min(avail).max(TITLE_MIN);
    }
}

fn terminal_width() -> Option<usize> {
    terminal_size::terminal_size().map(|(width, _)| width.0 as usize)
}

/// Lay out the header and one line per task. Every column is padded to its
/// width except the last, which is truncated-not-padded so no trailing run
/// survives (keeps output greppable / paste-friendly).
fn render_grid(columns: &[Column], glyph_mode: GlyphMode, painter: Painter) -> String {
    if columns.is_empty() {
        return String::new();
    }
    let last = columns.len().saturating_sub(1);
    let mut out = String::new();
    let header: Vec<String> = columns
        .iter()
        .enumerate()
        .map(|(idx, col)| {
            if idx == last {
                col.id.header().to_owned()
            } else {
                pad_to_width(col.id.header(), col.width)
            }
        })
        .collect();
    out.push_str(&header.join(GAP));
    out.push('\n');
    let rows = columns.first().map_or(0, |col| col.cells.len());
    for row in 0..rows {
        let cells: Vec<String> = columns
            .iter()
            .enumerate()
            .filter_map(|(idx, col)| {
                col.cells.get(row).map(|cell| {
                    if idx == last {
                        render_unpadded(cell, glyph_mode, painter, col.width)
                    } else {
                        render_cell(cell, glyph_mode, DensityMode::Full, painter, col.width)
                    }
                })
            })
            .collect();
        out.push_str(&cells.join(GAP));
        out.push('\n');
    }
    out
}

/// The last column: truncate to width but do not pad (no trailing run).
fn render_unpadded(
    spec: &CellSpec,
    glyph_mode: GlyphMode,
    painter: Painter,
    width: usize,
) -> String {
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
    use mindtape::{ColumnConfig, Config, Readiness, Status, Task, Visibility};

    use super::super::columns::due_cell;
    use super::{render, render_highlighted};

    fn with_column(key: &str, visibility: Visibility) -> Config {
        let mut config = Config::default();
        config
            .ls
            .columns
            .insert(key.to_owned(), ColumnConfig { visibility });
        config
    }

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
    fn highlight_empty_mask_equals_plain() {
        // No row flagged: the highlighted render is byte-identical to the plain
        // one (the `#` column is untouched).
        let tasks = [task("a", "A", "wip"), task("b", "B", "wip")];
        let config = Config::default();
        let plain = render(&tasks, &config, &Readiness::empty());
        let highlighted = render_highlighted(&tasks, &config, &Readiness::empty(), &[false, false]);
        assert_eq!(plain, highlighted);
    }

    #[test]
    fn highlight_preserves_shape() {
        // Flagging a row keeps the header + one-line-per-task layout intact
        // (colour is off in tests, so the invert effect itself is not emitted).
        let tasks = [task("a", "A", "wip"), task("b", "B", "wip")];
        let out = render_highlighted(
            &tasks,
            &Config::default(),
            &Readiness::empty(),
            &[true, false],
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
    fn empty_columns_hidden_by_default() {
        // A task with status but no priority and no due date: PRIORITY and DUE
        // carry nothing, so `auto` drops them; STATUS and TITLE stay.
        let mut t = task("a", "A", "wip");
        t.priority = None;
        let out = render(&[t], &Config::default(), &Readiness::empty());
        let header = out.lines().next().unwrap();
        assert!(!header.contains("PRIORITY"), "empty PRIORITY must hide");
        assert!(!header.contains("DUE"), "empty DUE must hide");
        assert!(header.contains("STATUS"));
        assert!(header.contains("TITLE"));
    }

    #[test]
    fn visibility_visible_forces_blank_column() {
        let mut t = task("a", "A", "wip");
        t.priority = None;
        let config = with_column("priority", Visibility::Visible);
        let out = render(&[t], &config, &Readiness::empty());
        assert!(
            out.lines().next().unwrap().contains("PRIORITY"),
            "visible must render even when blank"
        );
    }

    #[test]
    fn visibility_hidden_drops_populated_column() {
        let config = with_column("status", Visibility::Hidden);
        let out = render(&[task("a", "A", "wip")], &config, &Readiness::empty());
        assert!(!out.contains("STATUS"), "hidden column must not render");
        assert!(
            !out.contains("wip"),
            "hidden column's cells must not render"
        );
    }

    #[test]
    fn columns_order_selects_and_orders() {
        let mut config = Config::default();
        config.ls.columns_order = vec!["title".to_owned(), "status".to_owned()];
        let out = render(&[task("a", "Hello", "wip")], &config, &Readiness::empty());
        let header = out.lines().next().unwrap();
        let title_at = header.find("TITLE").unwrap();
        let status_at = header.find("STATUS").unwrap();
        assert!(title_at < status_at, "columns-order must reorder");
        assert!(!header.contains('#'), "unlisted handle column excluded");
        assert!(!header.contains("PRIORITY"), "unlisted column excluded");
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
