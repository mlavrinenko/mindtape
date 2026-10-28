//! `mt board`: one row per project, one cell per `[board]` column.
//!
//! On a held set (`mt @home board`, or a bare `mt @home`) every member loads in
//! parallel through [`open_set_warned`], each under its own config; otherwise
//! the board is the current project's one row. [`mindtape::Board`] computes the
//! cells, hides and sorts the rows; this module prints them. A member whose
//! config fails is a `⚠` row naming the error, and the command exits 1 after
//! the others print. A failed cell is `?`, with a note under the table.

use std::path::PathBuf;

use mindtape::{Board, Cell, StemHashes};

use crate::cmd::at::load_at;
use crate::diag::{Diag, Kind, Result};
use crate::project_set::{held_set, open_set_warned};
use crate::render::width::{display_width, pad_to_width, truncate_to_width};
use crate::util::{self, Project};

/// Arguments for `mt board`.
#[derive(Debug, clap::Args)]
pub struct BoardArgs {
    /// Show the rows the board's `hide` hides, too.
    #[arg(long)]
    pub all: bool,
    /// Print every row as JSON, hidden ones included.
    #[arg(long)]
    pub json: bool,
}

/// One project's row: its cells, or why its config would not load.
struct Row {
    name: String,
    root: PathBuf,
    body: std::result::Result<Loaded, String>,
}

/// A loaded project's cells and the stem hashes its `top` cells print.
struct Loaded {
    cells: Vec<Cell>,
    hashes: StemHashes,
    hidden: bool,
}

impl Row {
    fn cells(&self) -> &[Cell] {
        self.body.as_ref().map_or(&[], |loaded| &loaded.cells)
    }

    fn hidden(&self) -> bool {
        self.body.as_ref().is_ok_and(|loaded| loaded.hidden)
    }
}

/// Run `mt board`.
///
/// # Errors
/// Returns an error if `at.toml` or the current project fails to load, or,
/// after the board prints, when a member of the set failed to load.
pub fn run(args: &BoardArgs) -> Result<()> {
    let (at, _) = load_at()?;
    let held = held_set();
    // No entry is named "", so it reads `[board]` over the shipped board.
    let board = at.board(held.map_or("", |held| held.name.as_str()));
    let mut rows = if let Some(held) = held {
        open_set_warned(&held.members)
            .into_iter()
            .zip(&held.members)
            .map(|(member, (_, root))| row(board, member.name, root.clone(), member.project))
            .collect()
    } else {
        let project = util::open_warned()?;
        let name = project_name(&project);
        vec![row(board, name, project.root.clone(), Ok(project))]
    };
    board.sort_rows(&mut rows, Row::cells);
    if args.json {
        print_json(board, &rows);
    } else {
        let scope = held.map(|held| held.name.as_str());
        print_board(board, &rows, scope, args.all);
    }
    let failed = rows.iter().filter(|row| row.body.is_err()).count();
    if failed > 0 {
        let message = format!("{} failed to load", mindtape::count(failed, "project"));
        return Err(Diag::verbatim(Kind::Failed, message));
    }
    Ok(())
}

/// The name the current project goes by: its directory's.
fn project_name(project: &Project) -> String {
    project.root.file_name().map_or_else(
        || mindtape::display_path(&project.root),
        |name| name.to_string_lossy().into_owned(),
    )
}

fn row(board: &Board, name: String, root: PathBuf, project: Result<Project>) -> Row {
    let body = project
        .map(|project| {
            let tasks = &project.outcome.tasks;
            let cells = board.row(tasks, &project.config);
            Loaded {
                hidden: board.hides(&cells),
                hashes: util::stem_hashes(tasks, &project.config),
                cells,
            }
        })
        .map_err(|diag| diag.to_string());
    Row { name, root, body }
}

/// The table, the failed cells' notes and the footer.
fn print_board(board: &Board, rows: &[Row], scope: Option<&str>, all: bool) {
    let shown: Vec<&Row> = rows.iter().filter(|row| all || !row.hidden()).collect();
    let term = terminal_size::terminal_size().map(|(width, _)| usize::from(width.0));
    for line in table(board, &shown) {
        println!(
            "{}",
            term.map_or_else(|| line.clone(), |w| truncate_to_width(&line, w))
        );
    }
    let notes: Vec<String> = shown
        .iter()
        .flat_map(|row| {
            row.cells().iter().filter_map(|cell| match cell {
                Cell::Failed(message) => Some(format!("note: {}: {message}", row.name)),
                _ => None,
            })
        })
        .collect();
    if !notes.is_empty() {
        println!();
        for note in notes {
            println!("{note}");
        }
    }
    println!();
    println!("{}", footer(rows.len(), rows.len() - shown.len(), scope));
}

/// `@home · 27 projects · 19 hidden · mt @home board --all`.
fn footer(total: usize, hidden: usize, scope: Option<&str>) -> String {
    let mut parts: Vec<String> = scope.map(|name| format!("@{name}")).into_iter().collect();
    parts.push(mindtape::count(total, "project"));
    if hidden > 0 {
        parts.push(format!("{hidden} hidden"));
        parts.push(match scope {
            Some(name) => format!("mt @{name} board --all"),
            None => "mt board --all".to_owned(),
        });
    }
    parts.join(" · ")
}

/// The header and one line per row: counts right-aligned, every column but
/// the last padded, columns two spaces apart.
fn table(board: &Board, rows: &[&Row]) -> Vec<String> {
    let columns = board.columns();
    let mut header = vec!["PROJECT".to_owned()];
    header.extend(columns.iter().map(|column| column.name.to_uppercase()));
    let right: Vec<bool> = std::iter::once(false)
        .chain(columns.iter().map(|column| column.top.is_none()))
        .collect();
    let lines: Vec<Vec<String>> = std::iter::once(header)
        .chain(rows.iter().map(|row| row_texts(row)))
        .collect();
    let mut widths = vec![0; right.len()];
    for line in &lines {
        // A failed project's message spans the cells, so it widens only the
        // name's column.
        let cells = if line.len() == widths.len() {
            line.len()
        } else {
            1
        };
        for (width, text) in widths.iter_mut().zip(line).take(cells) {
            *width = (*width).max(display_width(text));
        }
    }
    lines
        .iter()
        .map(|line| layout(line, &widths, &right))
        .collect()
}

fn layout(line: &[String], widths: &[usize], right: &[bool]) -> String {
    let last = line.len().saturating_sub(1);
    let texts: Vec<String> = line
        .iter()
        .enumerate()
        .map(|(index, text)| {
            let width = widths.get(index).copied().unwrap_or(0);
            if right.get(index).copied().unwrap_or(false) && line.len() == widths.len() {
                format!(
                    "{}{text}",
                    " ".repeat(width.saturating_sub(display_width(text)))
                )
            } else if index == last {
                text.clone()
            } else {
                pad_to_width(text, width)
            }
        })
        .collect();
    texts.join("  ").trim_end().to_owned()
}

/// A row's texts: the name, then each cell, or `⚠ name` and the error.
fn row_texts(row: &Row) -> Vec<String> {
    match &row.body {
        Err(message) => vec![format!("⚠ {}", row.name), message.clone()],
        Ok(loaded) => std::iter::once(row.name.clone())
            .chain(
                loaded
                    .cells
                    .iter()
                    .map(|cell| cell_text(cell, &loaded.hashes)),
            )
            .collect(),
    }
}

fn cell_text(cell: &Cell, hashes: &StemHashes) -> String {
    match cell {
        Cell::Count(count) => count.to_string(),
        Cell::Top(Some(top)) => match hashes.hash_of(&top.id) {
            Some(hash) => format!("{hash} {}", top.title),
            None => top.title.clone(),
        },
        Cell::Top(None) => "-".to_owned(),
        Cell::Failed(_) => "?".to_owned(),
    }
}

/// Every row as JSON: `project`, `path`, `hidden` and `cells` by column name,
/// or `error` for a project that failed to load.
fn print_json(board: &Board, rows: &[Row]) {
    let items: Vec<serde_json::Value> = rows
        .iter()
        .map(|row| {
            let (project, path) = (&row.name, mindtape::slash_path(&row.root));
            match &row.body {
                Err(message) => serde_json::json!({
                    "project": project, "path": path, "error": message,
                }),
                Ok(loaded) => {
                    let cells: serde_json::Map<String, serde_json::Value> = board
                        .columns()
                        .iter()
                        .zip(&loaded.cells)
                        .map(|(column, cell)| (column.name.clone(), cell_json(cell)))
                        .collect();
                    serde_json::json!({
                        "project": project, "path": path, "hidden": loaded.hidden, "cells": cells,
                    })
                }
            }
        })
        .collect();
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::Value::Array(items)).unwrap_or_default()
    );
}

fn cell_json(cell: &Cell) -> serde_json::Value {
    match cell {
        Cell::Count(count) => (*count).into(),
        Cell::Top(Some(top)) => serde_json::json!({
            "id": top.id, "title": top.title, "file": mindtape::slash_path(&top.file),
        }),
        Cell::Top(None) => serde_json::Value::Null,
        Cell::Failed(message) => serde_json::json!({ "error": message }),
    }
}

#[cfg(test)]
#[path = "board_tests.rs"]
mod tests;
