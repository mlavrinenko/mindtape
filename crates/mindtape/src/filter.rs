//! The filter-expression query language, evaluated over live tasks.
//!
//! `evalexpr` is used only as a parser: its AST is walked and evaluated against
//! each [`Task`] in memory (never against a database).
//!
//! # Syntax
//!
//! Comparable fields: `status`, `priority`, `grade`, `kind`, `title`, `id`,
//! `due`, `start`, `reviewed`, `file`. The opaque `estimate` dict supports
//! presence only (`has(estimate)` / `miss(estimate)`), not comparison.
//!
//! Operators: `==`, `!=`, `<`, `>`, `<=`, `>=`, `&&`, `||`, `!`, parentheses.
//!
//! Functions: `has(field)`, `miss(field)`, `has_tag("name")`,
//! `contains(field, "substr")`, `search("query")`, `stale(N)`.

use evalexpr::{DefaultNumericTypes, Node, Operator, Value, build_operator_tree};
use thiserror::Error;

use crate::model::Task;

/// A parsed, reusable filter expression.
#[derive(Debug, Clone)]
pub struct Filter {
    tree: Node<DefaultNumericTypes>,
}

/// Errors from parsing or evaluating a filter expression.
#[derive(Debug, Error)]
pub enum FilterError {
    /// The expression failed to parse.
    #[error("failed to parse filter: {0}")]
    Parse(String),
    /// An operator that the filter language does not support.
    #[error("unsupported in filter: {0}")]
    Unsupported(String),
    /// A field name the filter language does not know.
    #[error("unknown field in filter: {0}")]
    UnknownField(String),
    /// A structurally invalid expression.
    #[error("invalid filter: {0}")]
    Invalid(String),
}

/// A resolved field value during evaluation. Every comparable field is a
/// string (dates compare lexicographically, which is ISO-correct).
enum FieldValue {
    Str(String),
    Missing,
}

impl Filter {
    /// Parse a filter expression.
    ///
    /// # Errors
    /// Returns [`FilterError::Parse`] on a syntax error.
    pub fn parse(expr: &str) -> Result<Self, FilterError> {
        let tree = build_operator_tree::<DefaultNumericTypes>(expr)
            .map_err(|err| FilterError::Parse(err.to_string()))?;
        Ok(Self { tree })
    }

    /// Whether `task` satisfies the filter.
    ///
    /// # Errors
    /// Returns [`FilterError`] on an unknown field or unsupported operator.
    pub fn matches(&self, task: &Task) -> Result<bool, FilterError> {
        eval_bool(&self.tree, task)
    }
}

fn eval_bool(node: &Node<DefaultNumericTypes>, task: &Task) -> Result<bool, FilterError> {
    match node.operator() {
        Operator::RootNode => match node.children() {
            [child] => eval_bool(child, task),
            _ => Err(FilterError::Invalid("root needs one child".to_owned())),
        },
        Operator::And => binary(node, task, |left, right| left && right),
        Operator::Or => binary(node, task, |left, right| left || right),
        Operator::Not => match node.children() {
            [child] => Ok(!eval_bool(child, task)?),
            _ => Err(FilterError::Invalid("`!` needs one operand".to_owned())),
        },
        Operator::Eq
        | Operator::Neq
        | Operator::Lt
        | Operator::Gt
        | Operator::Leq
        | Operator::Geq => comparison(node, task),
        Operator::FunctionIdentifier { identifier } => function(identifier, node, task),
        Operator::VariableIdentifierRead { identifier } => Err(FilterError::Invalid(format!(
            "bare field `{identifier}` is not boolean; use a comparison or function"
        ))),
        other => Err(FilterError::Unsupported(format!("{other:?}"))),
    }
}

fn binary(
    node: &Node<DefaultNumericTypes>,
    task: &Task,
    combine: impl Fn(bool, bool) -> bool,
) -> Result<bool, FilterError> {
    match node.children() {
        [left, right] => Ok(combine(eval_bool(left, task)?, eval_bool(right, task)?)),
        _ => Err(FilterError::Invalid(
            "binary op needs two operands".to_owned(),
        )),
    }
}

fn comparison(node: &Node<DefaultNumericTypes>, task: &Task) -> Result<bool, FilterError> {
    let [lhs, rhs] = node.children() else {
        return Err(FilterError::Invalid(
            "comparison needs two operands".to_owned(),
        ));
    };
    let field = extract_identifier(lhs)?;
    let literal = extract_literal(rhs)?;
    let value = field_value(task, &field)?;
    Ok(compare(node.operator(), &value, &literal))
}

fn compare(op: &Operator<DefaultNumericTypes>, value: &FieldValue, literal: &str) -> bool {
    match value {
        FieldValue::Missing => matches!(op, Operator::Neq),
        FieldValue::Str(text) => compare_ordering(op, Some(text.as_str().cmp(literal))),
    }
}

fn compare_ordering(
    op: &Operator<DefaultNumericTypes>,
    ordering: Option<std::cmp::Ordering>,
) -> bool {
    use std::cmp::Ordering::{Equal, Greater, Less};
    let Some(ordering) = ordering else {
        return matches!(op, Operator::Neq);
    };
    match op {
        Operator::Eq => ordering == Equal,
        Operator::Neq => ordering != Equal,
        Operator::Lt => ordering == Less,
        Operator::Gt => ordering == Greater,
        Operator::Leq => ordering != Greater,
        Operator::Geq => ordering != Less,
        _ => false,
    }
}

fn function(
    name: &str,
    node: &Node<DefaultNumericTypes>,
    task: &Task,
) -> Result<bool, FilterError> {
    let children = node.children();
    match name {
        "has" => Ok(present(task, &single_identifier(children, "has")?)?),
        "miss" => Ok(!present(task, &single_identifier(children, "miss")?)?),
        "has_tag" => {
            let tag = single_literal(children, "has_tag")?;
            Ok(task.tags.iter().any(|owned| owned == &tag))
        }
        "contains" => {
            let (field, needle) = two_args(children, "contains")?;
            let name = extract_identifier(field)?;
            let needle = extract_literal(needle)?;
            Ok(contains_ci(&field_value(task, &name)?, &needle))
        }
        "search" => {
            let needle = single_literal(children, "search")?.to_lowercase();
            Ok(task.title.to_lowercase().contains(&needle)
                || task.id.to_lowercase().contains(&needle))
        }
        "stale" => {
            let count_str = single_literal(children, "stale")?;
            let count: u64 = count_str.parse().map_err(|_| {
                FilterError::Invalid(format!(
                    "stale() expects a non-negative integer, got {count_str:?}"
                ))
            })?;
            Ok(is_stale(task, count))
        }
        other => Err(FilterError::Invalid(format!("unknown function: {other}"))),
    }
}

fn present(task: &Task, name: &str) -> Result<bool, FilterError> {
    match name {
        "tags" => Ok(!task.tags.is_empty()),
        "depends_on" => Ok(!task.depends_on.is_empty()),
        "relates_to" => Ok(!task.relates_to.is_empty()),
        "supersedes" => Ok(!task.supersedes.is_empty()),
        "duplicates" => Ok(!task.duplicates.is_empty()),
        "estimate" => Ok(!task.estimate.is_empty()),
        other => Ok(!matches!(field_value(task, other)?, FieldValue::Missing)),
    }
}

fn is_stale(task: &Task, days: u64) -> bool {
    let threshold = crate::date::days_ago_iso(days);
    if let Some(rev) = &task.reviewed {
        rev.as_str() < threshold.as_str()
    } else if let Some(ref_date) = task.status.as_ref().and_then(|s| s.date.as_deref()) {
        ref_date < threshold.as_str()
    } else {
        true
    }
}

fn contains_ci(value: &FieldValue, needle: &str) -> bool {
    let haystack = match value {
        FieldValue::Str(text) => text.clone(),
        FieldValue::Missing => return false,
    };
    haystack.to_lowercase().contains(&needle.to_lowercase())
}

fn field_value(task: &Task, name: &str) -> Result<FieldValue, FilterError> {
    let value = match name {
        "status" => opt_str(task.status.as_ref().map(|status| status.name.clone())),
        "priority" => opt_str(task.priority.clone()),
        "grade" => opt_str(task.grade.clone()),
        "kind" => opt_str(task.kind.clone()),
        "title" => FieldValue::Str(task.title.clone()),
        "id" => FieldValue::Str(task.id.clone()),
        "due" => opt_str(task.due.clone()),
        "start" => opt_str(task.start.clone()),
        "reviewed" => opt_str(task.reviewed.clone()),
        "file" => FieldValue::Str(task.file.display().to_string()),
        other => return Err(FilterError::UnknownField(other.to_owned())),
    };
    Ok(value)
}

fn opt_str(value: Option<String>) -> FieldValue {
    value.map_or(FieldValue::Missing, FieldValue::Str)
}

// --- AST helpers (ported from the SQL translator) ---

fn unwrap_root(node: &Node<DefaultNumericTypes>) -> &Node<DefaultNumericTypes> {
    if let Operator::RootNode = node.operator()
        && let [child] = node.children()
    {
        return unwrap_root(child);
    }
    node
}

fn extract_identifier(node: &Node<DefaultNumericTypes>) -> Result<String, FilterError> {
    match unwrap_root(node).operator() {
        Operator::VariableIdentifierRead { identifier } => Ok(identifier.clone()),
        other => Err(FilterError::Invalid(format!(
            "expected a field, got {other:?}"
        ))),
    }
}

fn extract_literal(node: &Node<DefaultNumericTypes>) -> Result<String, FilterError> {
    let node = unwrap_root(node);
    match node.operator() {
        Operator::Const { value } => literal_to_string(value),
        Operator::Neg => match node.children() {
            [child] => Ok(format!("-{}", extract_literal(child)?)),
            _ => Err(FilterError::Invalid("malformed negation".to_owned())),
        },
        other => Err(FilterError::Invalid(format!(
            "expected a literal, got {other:?}"
        ))),
    }
}

fn literal_to_string(value: &Value<DefaultNumericTypes>) -> Result<String, FilterError> {
    match value {
        Value::String(text) => Ok(text.clone()),
        Value::Int(num) => Ok(num.to_string()),
        Value::Float(num) => Ok(num.to_string()),
        Value::Boolean(flag) => Ok(if *flag { "true" } else { "false" }.to_owned()),
        _ => Err(FilterError::Invalid("unsupported literal".to_owned())),
    }
}

fn single_identifier(
    children: &[Node<DefaultNumericTypes>],
    name: &str,
) -> Result<String, FilterError> {
    match children {
        [arg] => extract_identifier(arg),
        _ => Err(FilterError::Invalid(format!("{name}() needs one argument"))),
    }
}

fn single_literal(
    children: &[Node<DefaultNumericTypes>],
    name: &str,
) -> Result<String, FilterError> {
    match children {
        [arg] => extract_literal(arg),
        _ => Err(FilterError::Invalid(format!("{name}() needs one argument"))),
    }
}

fn two_args<'a>(
    children: &'a [Node<DefaultNumericTypes>],
    name: &str,
) -> Result<(&'a Node<DefaultNumericTypes>, &'a Node<DefaultNumericTypes>), FilterError> {
    if let [child] = children
        && let Operator::Tuple = unwrap_root(child).operator()
        && let [first, second] = unwrap_root(child).children()
    {
        return Ok((first, second));
    }
    Err(FilterError::Invalid(format!(
        "{name}() needs two arguments"
    )))
}

#[cfg(test)]
#[path = "filter_tests.rs"]
mod tests;
