//! The board: one row per project, one cell per `[[board.columns]]` entry.
//!
//! A column is a `filter`, whose cell counts the project's matching tasks, or
//! a `top`, whose cell is the first matching task in the project's default
//! order. Each runs under the project's own config, so `@view` reads the
//! project's view, and `fallback` stands in when the project lacks one.
//! `hide` and `sort` read the cells by column name.
//!
//! The shipped columns are `board.toml`. `[board]` overrides them key by key,
//! and `[at.<name>.board]` overrides that for one set. A layer that sets
//! `columns` starts with no `sort` or `hide` but its own.

use std::cmp::Ordering;
use std::collections::BTreeSet;
use std::path::PathBuf;

use evalexpr::{DefaultNumericTypes, Node, Operator, Value, build_operator_tree};
use serde::Deserialize;

use crate::config::Config;
use crate::filter::expr::{compare_ordering, connective, describe, restore_name, unwrap_root};
use crate::filter::{Filter, FilterError};
use crate::graph::ready::Readiness;
use crate::model::Task;
use crate::sort::default_order_with;

/// The shipped board, the bottom layer under `[board]`.
const SHIPPED: &str = include_str!("board.toml");

/// One board layer as written: each key overrides the layer below it.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BoardSpec {
    /// The columns, in display order.
    pub columns: Option<Vec<BoardColumn>>,
    /// Row order: `column`, `column:asc` or `column:desc`, ties broken by
    /// the next entry and then by set order.
    pub sort: Option<Vec<String>>,
    /// An expression over the cells; a row it holds for is hidden. Empty hides
    /// nothing.
    pub hide: Option<String>,
}

/// One `[[board.columns]]` entry.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BoardColumn {
    /// The column's name, which `sort` and `hide` read it by.
    pub name: String,
    /// A filter whose matching tasks the cell counts.
    pub filter: Option<String>,
    /// A filter whose first task, in the project's default order, is the cell.
    pub top: Option<String>,
    /// The filter to use instead when the project lacks a view the column's
    /// filter names.
    pub fallback: Option<String>,
}

impl BoardColumn {
    /// The column's own filter, `filter` or `top`.
    fn expr(&self) -> &str {
        self.filter.as_deref().or(self.top.as_deref()).unwrap_or("")
    }
}

/// A resolved, checked board.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Board {
    columns: Vec<BoardColumn>,
    sort: Vec<String>,
    hide: Option<String>,
}

/// One cell of a project's row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Cell {
    /// A `filter` column: how many tasks matched.
    Count(usize),
    /// A `top` column: the first matching task, if any matched.
    Top(Option<TopTask>),
    /// The column's filter failed; the message names the column.
    Failed(String),
}

/// The task a `top` cell shows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TopTask {
    /// Its stem.
    pub id: String,
    /// Its title.
    pub title: String,
    /// Its file, as the project loaded it.
    pub file: PathBuf,
}

/// A value `hide` compares.
enum HideValue {
    Number(f64),
    Text(String),
}

impl Board {
    /// Layer `specs` from the bottom up over the shipped board, then check
    /// the result.
    ///
    /// # Errors
    /// A message naming the first malformed column, sort spec or `hide`.
    pub(crate) fn resolve(specs: &[Option<&BoardSpec>]) -> Result<Self, String> {
        let shipped: BoardSpec = toml::from_str(SHIPPED).map_err(|err| err.to_string())?;
        let mut board = Self {
            columns: shipped.columns.unwrap_or_default(),
            sort: shipped.sort.unwrap_or_default(),
            hide: shipped.hide,
        };
        for spec in specs.iter().flatten() {
            // New columns drop the sort and hide below, which read the old ones.
            if let Some(columns) = &spec.columns {
                board.columns.clone_from(columns);
                board.sort.clear();
                board.hide = None;
            }
            if let Some(sort) = &spec.sort {
                board.sort.clone_from(sort);
            }
            if let Some(hide) = &spec.hide {
                board.hide = Some(hide.clone()).filter(|hide| !hide.trim().is_empty());
            }
        }
        board.check()?;
        Ok(board)
    }

    /// The columns, in display order.
    #[must_use]
    pub fn columns(&self) -> &[BoardColumn] {
        &self.columns
    }

    /// The row order, as written.
    #[must_use]
    pub fn sort(&self) -> &[String] {
        &self.sort
    }

    /// The `hide` expression, or `None` when no row hides.
    #[must_use]
    pub fn hide(&self) -> Option<&str> {
        self.hide.as_deref()
    }

    fn check(&self) -> Result<(), String> {
        let mut seen = BTreeSet::new();
        for column in &self.columns {
            let name = &column.name;
            if name.is_empty() {
                return Err("a column needs a `name`".to_owned());
            }
            if !seen.insert(name.as_str()) {
                return Err(format!("column '{name}' appears twice"));
            }
            if column.filter.is_some() == column.top.is_some() {
                return Err(format!("column '{name}' needs one of `filter` and `top`"));
            }
            for expr in [Some(column.expr()), column.fallback.as_deref()]
                .into_iter()
                .flatten()
            {
                Filter::parse(expr).map_err(|err| format!("column '{name}': {err}"))?;
            }
        }
        for spec in &self.sort {
            self.sort_key(spec)?;
        }
        let dry: Vec<Cell> = self
            .columns
            .iter()
            .map(|column| match column.top {
                Some(_) => Cell::Top(None),
                None => Cell::Count(0),
            })
            .collect();
        self.eval_hide(&dry).map(drop)
    }

    /// `spec`'s column index and whether it sorts descending.
    fn sort_key(&self, spec: &str) -> Result<(usize, bool), String> {
        let (name, direction) = spec.split_once(':').unwrap_or((spec, "asc"));
        let descending = match direction {
            "asc" => false,
            "desc" => true,
            other => {
                return Err(format!(
                    "sort '{spec}': direction '{other}' is not asc or desc"
                ));
            }
        };
        let index = self
            .columns
            .iter()
            .position(|column| column.name == name)
            .ok_or_else(|| format!("sort '{spec}': no column '{name}'"))?;
        Ok((index, descending))
    }

    /// One project's cells, one per column, under its own `config`.
    #[must_use]
    pub fn row(&self, tasks: &[Task], config: &Config) -> Vec<Cell> {
        let readiness = Readiness::compute(tasks, config);
        self.columns
            .iter()
            .map(|column| {
                cell(column, tasks, config, &readiness)
                    .unwrap_or_else(|err| Cell::Failed(format!("column '{}': {err}", column.name)))
            })
            .collect()
    }

    /// Whether `hide` hides a row with these cells. A row whose `hide` reads
    /// a failed cell stays, so its failure shows.
    #[must_use]
    pub fn hides(&self, cells: &[Cell]) -> bool {
        self.eval_hide(cells).unwrap_or(false)
    }

    /// Sort `rows` by `sort`, reading each row's cells through `cells`. A
    /// failed or missing cell sorts last either way; ties keep their order.
    pub fn sort_rows<T>(&self, rows: &mut [T], cells: impl Fn(&T) -> &[Cell]) {
        let keys: Vec<(usize, bool)> = self
            .sort
            .iter()
            .filter_map(|spec| self.sort_key(spec).ok())
            .collect();
        rows.sort_by(|left, right| {
            let (left, right) = (cells(left), cells(right));
            keys.iter()
                .fold(Ordering::Equal, |order, &(index, descending)| {
                    order.then_with(|| compare_cells(left.get(index), right.get(index), descending))
                })
        });
    }

    fn eval_hide(&self, cells: &[Cell]) -> Result<bool, String> {
        let Some(hide) = &self.hide else {
            return Ok(false);
        };
        let tree = build_operator_tree::<DefaultNumericTypes>(hide)
            .map_err(|err| format!("hide: {err}"))?;
        self.eval_node(&tree, cells)
            .map_err(|err| format!("hide: {err}"))
    }

    fn eval_node(&self, node: &Node<DefaultNumericTypes>, cells: &[Cell]) -> Result<bool, String> {
        if let Some(result) = connective(node, &|child| self.eval_node(child, cells), |msg| msg) {
            return result;
        }
        let op = node.operator();
        let [left, right] = node.children() else {
            return Err(format!("expected a comparison, got {}", describe(node)));
        };
        if !matches!(
            op,
            Operator::Eq
                | Operator::Neq
                | Operator::Lt
                | Operator::Gt
                | Operator::Leq
                | Operator::Geq
        ) {
            return Err(format!("expected a comparison, got {}", describe(node)));
        }
        let (left, right) = (self.value(left, cells)?, self.value(right, cells)?);
        let ordering = match (left, right) {
            (HideValue::Number(left), HideValue::Number(right)) => left.partial_cmp(&right),
            (HideValue::Text(left), HideValue::Text(right)) => Some(left.cmp(&right)),
            _ => None,
        };
        Ok(compare_ordering(op, ordering))
    }

    /// A comparison side: a column's cell or a literal.
    fn value(&self, node: &Node<DefaultNumericTypes>, cells: &[Cell]) -> Result<HideValue, String> {
        match unwrap_root(node).operator() {
            Operator::VariableIdentifierRead { identifier } => {
                let name = restore_name(identifier);
                let index = self
                    .columns
                    .iter()
                    .position(|column| column.name == name)
                    .ok_or_else(|| format!("no column '{name}'"))?;
                match cells.get(index) {
                    Some(Cell::Count(count)) => Ok(HideValue::Number(count_f64(*count))),
                    Some(Cell::Top(top)) => Ok(HideValue::Text(
                        top.as_ref()
                            .map(|top| top.title.clone())
                            .unwrap_or_default(),
                    )),
                    Some(Cell::Failed(message)) => Err(message.clone()),
                    None => Err(format!("the row has no cell '{name}'")),
                }
            }
            Operator::Const {
                value: Value::Int(number),
            } => Ok(HideValue::Number(int_f64(*number))),
            Operator::Const {
                value: Value::Float(number),
            } => Ok(HideValue::Number(*number)),
            Operator::Const {
                value: Value::String(text),
            } => Ok(HideValue::Text(text.clone())),
            _ => Err(format!(
                "expected a column or a value, got {}",
                describe(node)
            )),
        }
    }
}

/// One cell: the column's filter, or its fallback when the project lacks a
/// view the filter names.
fn cell(
    column: &BoardColumn,
    tasks: &[Task],
    config: &Config,
    readiness: &Readiness,
) -> Result<Cell, FilterError> {
    let mut filter = Filter::parse(column.expr())?;
    if let Some(fallback) = &column.fallback
        && filter
            .view_refs()
            .iter()
            .any(|view| !config.views.contains_key(view))
    {
        filter = Filter::parse(fallback)?;
    }
    filter.check_statuses(&config.tasks.status)?;
    let mut matched = Vec::new();
    for task in tasks {
        if filter.matches_with(task, config, readiness)? {
            matched.push(task);
        }
    }
    if column.top.is_none() {
        return Ok(Cell::Count(matched.len()));
    }
    let mut matched: Vec<Task> = matched.into_iter().cloned().collect();
    default_order_with(&mut matched, config, readiness);
    Ok(Cell::Top(matched.into_iter().next().map(|task| TopTask {
        id: task.id,
        title: task.title,
        file: task.file,
    })))
}

/// Order two cells of one column: counts by number, top cells by title with
/// an empty one first, and a failed or missing cell last in either direction.
fn compare_cells(left: Option<&Cell>, right: Option<&Cell>, descending: bool) -> Ordering {
    let order = match (left, right) {
        (Some(Cell::Count(left)), Some(Cell::Count(right))) => left.cmp(right),
        (Some(Cell::Top(left)), Some(Cell::Top(right))) => {
            let title = |top: &Option<TopTask>| top.as_ref().map(|top| top.title.clone());
            title(left).cmp(&title(right))
        }
        (Some(Cell::Failed(_)) | None, Some(Cell::Failed(_)) | None) => return Ordering::Equal,
        (Some(Cell::Failed(_)) | None, _) => return Ordering::Greater,
        (_, Some(Cell::Failed(_)) | None) => return Ordering::Less,
        _ => Ordering::Equal,
    };
    if descending { order.reverse() } else { order }
}

#[allow(clippy::cast_precision_loss)]
const fn count_f64(count: usize) -> f64 {
    count as f64
}

#[allow(clippy::cast_precision_loss)]
const fn int_f64(number: i64) -> f64 {
    number as f64
}

#[cfg(test)]
#[path = "board_tests.rs"]
mod tests;
