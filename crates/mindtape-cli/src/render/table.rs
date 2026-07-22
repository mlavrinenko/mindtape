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
    apply_flex(&mut columns, terminal_width());
    columns
}

fn col_width(header: &str, cells: impl Iterator<Item = usize>) -> usize {
    cells.fold(display_width(header), usize::max)
}

/// Shrink the flex column (`title`) to the remaining terminal width: total
/// minus every other column and the inter-column gaps, clamped to
/// `[TITLE_MIN, natural]`. No flex column or no known terminal width: leave
/// natural widths untouched.
fn apply_flex(columns: &mut [Column], term: Option<usize>) {
    let Some(term) = term else {
        return;
    };
    flex_columns(columns, term);
}

/// The core flex computation, split from [`apply_flex`] so it is testable
/// without mocking the terminal.
fn flex_columns(columns: &mut [Column], term: usize) {
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
    let fitted = truncate_to_width(&content, width);
    match spec.color {
        Some(style) => painter.paint(&fitted, style),
        None => fitted,
    }
}

#[cfg(test)]
#[path = "table_tests.rs"]
mod tests;
#[cfg(test)]
#[path = "table_tests_layout.rs"]
mod tests_layout;
#[cfg(test)]
#[path = "table_tests_ready.rs"]
mod tests_ready;
