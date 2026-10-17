//! Borderless, copy-friendly list rendering.
//!
//! Replaces the box-drawing table with space-aligned columns: one line per task
//! plus a single header line — greppable, diffable, paste-friendly. Colour and
//! glyphs activate only on a TTY; piped output is plain.

use std::io::IsTerminal;

use crate::diag::Result;
use anstyle::{Effects, Style};
use mindtape::Task;

use super::RenderCtx;
use super::color::{Painter, is_color_active};
use super::columns::{ColumnId, joined_rows, resolve_order, row_hyperlink_targets, rows};
use super::field::{CellSpec, DensityMode, cell_content, content_width, render_cell};
use super::glyph::GlyphMode;
use super::link::Linker;
use super::width::{display_width, pad_to_width, truncate_to_width};

const GAP: &str = "  ";
const TITLE_MIN: usize = 16;

/// A resolved visible column: its id, per-row cells, and display width.
struct Column {
    id: ColumnId,
    cells: Vec<CellSpec>,
    width: usize,
    /// Per-row hyperlink target URI, populated for the `slug#` and `file`
    /// columns — the only two OSC 8 wraps. Both share the same per-row list
    /// (the target is a property of the row, not the column); `None` for
    /// every other column, so `render_grid` skips the wrap there regardless
    /// of whether hyperlinks are active.
    link_uris: Option<Vec<String>>,
}

/// Render `tasks` as a borderless, space-aligned table.
///
/// The column set, order, and visibility come from `[ls.columns]` /
/// `[ls] columns-order`. Under the shipped floor's `when` expressions a column
/// is dropped when it carries nothing — so a backlog with no priorities, no due
/// dates, or no contested dependencies stays uncluttered.
///
/// # Errors
/// Returns an error if a column's `when` fails to evaluate
/// ([`resolve_order`]).
pub fn render(tasks: &[Task], ctx: RenderCtx<'_>) -> Result<String> {
    render_inner(tasks, ctx, &[])
}

/// Render `tasks` with the interactive `slug#`-column highlight applied.
///
/// `highlighted[i]` marks whether the `slug#` cell of display row `i` renders
/// reverse-video; a shorter slice leaves the remaining rows un-highlighted. The
/// invert effect rides the table's painter, so `--color never` drops it.
///
/// # Errors
/// Same as [`render`].
pub fn render_highlighted(
    tasks: &[Task],
    ctx: RenderCtx<'_>,
    highlighted: &[bool],
) -> Result<String> {
    render_inner(tasks, ctx, highlighted)
}

fn render_inner(tasks: &[Task], ctx: RenderCtx<'_>, highlighted: &[bool]) -> Result<String> {
    let glyph_mode = glyph_mode();
    let painter = Painter::new(is_color_active());
    let linker = Linker::new(ctx.hyperlinks);
    let rows = rows(tasks, ctx.hashes);
    let visible = resolve_order(&rows, ctx)?;
    let mut columns = build_columns(&visible, &rows, ctx, glyph_mode);
    apply_highlight(&mut columns, highlighted);
    Ok(render_grid(&columns, glyph_mode, painter, linker))
}

/// Render several listings as one table, each listing's rows in a block, in
/// order: a set's projects, each with its own [`RenderCtx`]. The columns are
/// every column any listing resolves visible ([`union_order`]), and each cell
/// is built by its own listing's context.
///
/// A listing whose context turns hyperlinks off gets no OSC 8 wrap on its
/// rows, whatever the others decide.
///
/// # Errors
/// Returns an error if a column's `when` fails to evaluate
/// ([`resolve_order`]).
pub fn render_groups(groups: &[(&[Task], RenderCtx<'_>)]) -> Result<String> {
    let glyph_mode = glyph_mode();
    let (rows, visible) = joined_rows(groups)?;
    let mut joined: Vec<Column> = Vec::new();
    for ((_, ctx), rows) in groups.iter().zip(&rows) {
        let columns = group_columns(&visible, rows, *ctx, glyph_mode);
        join_columns(&mut joined, columns, ctx.hyperlinks);
    }
    apply_flex(&mut joined, terminal_width());
    let painter = Painter::new(is_color_active());
    let linker = Linker::new(groups.iter().any(|(_, ctx)| ctx.hyperlinks));
    Ok(render_grid(&joined, glyph_mode, painter, linker))
}

/// Append one listing's `columns` to `joined`, column by column: the cells
/// and link targets follow the rows already there, and the width grows to fit.
/// Without `hyperlinks`, the listing's targets are blank, which
/// [`render_grid`] leaves unwrapped.
fn join_columns(joined: &mut Vec<Column>, columns: Vec<Column>, hyperlinks: bool) {
    if joined.is_empty() {
        joined.extend(columns.into_iter().map(|col| Column {
            link_uris: col.link_uris.map(|uris| blank_unless(uris, hyperlinks)),
            ..col
        }));
        return;
    }
    for (into, col) in joined.iter_mut().zip(columns) {
        into.width = into.width.max(col.width);
        into.cells.extend(col.cells);
        if let (Some(uris), Some(more)) = (into.link_uris.as_mut(), col.link_uris) {
            uris.extend(blank_unless(more, hyperlinks));
        }
    }
}

/// `uris` when `keep`, else as many blank targets.
fn blank_unless(uris: Vec<String>, keep: bool) -> Vec<String> {
    if keep {
        uris
    } else {
        vec![String::new(); uris.len()]
    }
}

/// Paint the `slug#` column's cells reverse-video for the highlighted rows. A
/// no-op when no row is flagged or the column is not visible.
fn apply_highlight(columns: &mut [Column], highlighted: &[bool]) {
    if highlighted.iter().all(|on| !on) {
        return;
    }
    let invert = Style::new().effects(Effects::INVERT);
    if let Some(col) = columns.iter_mut().find(|col| col.id == ColumnId::StemHash) {
        for (cell, on) in col.cells.iter_mut().zip(highlighted) {
            if *on {
                cell.color = Some(invert);
            }
        }
    }
}

/// Emoji on a TTY; no glyphs when piped (keeps pipe output plain).
fn glyph_mode() -> GlyphMode {
    glyph_mode_for(std::io::stdout().is_terminal())
}

/// [`glyph_mode`] with the terminal answer supplied, so both arms are reachable
/// in a test regardless of how the suite was invoked.
fn glyph_mode_for(tty: bool) -> GlyphMode {
    if tty {
        GlyphMode::Emoji
    } else {
        GlyphMode::None
    }
}

/// Build each visible column's cells and width. The flex column (`title`)
/// absorbs the remaining terminal width; the rest size to their content.
fn build_columns(
    visible: &[ColumnId],
    rows: &[(String, &Task)],
    ctx: RenderCtx<'_>,
    glyph_mode: GlyphMode,
) -> Vec<Column> {
    let mut columns = group_columns(visible, rows, ctx, glyph_mode);
    apply_flex(&mut columns, terminal_width());
    columns
}

/// [`build_columns`] at natural widths, before the flex column is fitted.
fn group_columns(
    visible: &[ColumnId],
    rows: &[(String, &Task)],
    ctx: RenderCtx<'_>,
    glyph_mode: GlyphMode,
) -> Vec<Column> {
    let full = DensityMode::Full;
    // `slug#` and `file` both link to the same per-row target (the task's own
    // file), so it is built once here — only when one of them is actually
    // visible — and cloned into each, rather than recomputed per column.
    // Recomputing per column would be both wasted work and a divergence
    // risk: two calls could disagree if `[ls] hyperlink-target` or the
    // rows ever drifted between them.
    let row_uris = visible
        .iter()
        .any(|&id| is_linked_column(id))
        .then(|| row_hyperlink_targets(rows, ctx));
    visible
        .iter()
        .map(|&id| {
            let cells: Vec<CellSpec> = rows
                .iter()
                .map(|(hash, task)| id.cell(hash, task, ctx))
                .collect();
            let width = col_width(
                id.header(),
                cells
                    .iter()
                    .map(|cell| content_width(cell, glyph_mode, full)),
            );
            let link_uris = if is_linked_column(id) {
                row_uris.clone()
            } else {
                None
            };
            Column {
                id,
                cells,
                width,
                link_uris,
            }
        })
        .collect()
}

/// Whether `id` is one of the two columns OSC 8 wraps. Fixed by design (see
/// `truth/tasks/hyperlink-the-handle-column.typ`) — there is no per-column
/// `link` key to consult.
fn is_linked_column(id: ColumnId) -> bool {
    matches!(id, ColumnId::StemHash | ColumnId::File)
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
/// survives (keeps output greppable / paste-friendly). `linker` wraps a data
/// cell in its OSC 8 sequence, if any, strictly after width fitting and
/// colour — same as colour, it never disturbs the padding math.
fn render_grid(
    columns: &[Column],
    glyph_mode: GlyphMode,
    painter: Painter,
    linker: Linker,
) -> String {
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
                    let rendered = if idx == last {
                        render_unpadded(cell, glyph_mode, painter, col.width)
                    } else {
                        render_cell(cell, glyph_mode, DensityMode::Full, painter, col.width)
                    };
                    match col.link_uris.as_ref().and_then(|uris| uris.get(row)) {
                        Some(uri) if !uri.is_empty() => linker.link(&rendered, uri),
                        _ => rendered,
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
#[path = "table_tests_hyperlinks.rs"]
mod tests_hyperlinks;
#[cfg(test)]
#[path = "table_tests_layout.rs"]
mod tests_layout;
#[cfg(test)]
#[path = "table_tests_ready.rs"]
mod tests_ready;
