//! The filter-expression query language, evaluated over live tasks.
//!
//! `evalexpr` is used only as a parser: its AST is walked and evaluated against
//! each [`Task`] in memory (never against a database).
//!
//! # Syntax
//!
//! Comparable fields: `status`, `priority`, `difficulty`, `kind`, `title`,
//! `id`, `due`, `start`, `reviewed`, `file`. `priority` and `difficulty` are
//! numeric, so they compare numerically against a number literal (e.g.
//! `priority >= 7`). The opaque `estimate` dict supports presence only
//! (`has(estimate)` / `miss(estimate)`), not comparison.
//!
//! Operators: `==`, `!=`, `<`, `>`, `<=`, `>=`, `&&`, `||`, `!`, parentheses.
//!
//! Functions: `has(field)`, `miss(field)`, `has_tag("name")`,
//! `contains(field, "substr")`, `search("query")`, `stale(N)`,
//! `in_view("name")` (also written `@name`, which desugars to it): matches when
//! the named view's filter matches.
//!
//! `now()` evaluates to today's ISO date string (`YYYY-MM-DD`) and may appear
//! on the right-hand side of a comparison: `start <= now()`, `due < now()`.
//!
//! Dependency-aware predicates `ready` and `blocked` answer "what can I start
//! now" over the whole loaded set (see [`Readiness`]). They take no argument and
//! may be written bare (`ready`) or as a call (`ready()`). The `terminal`
//! predicate (likewise bare or `terminal()`) matches a task whose status is
//! flagged terminal in config.

use evalexpr::{DefaultNumericTypes, Node, Operator, Value, build_operator_tree};
use thiserror::Error;

use crate::config::{Config, PriorityConfig};
use crate::filter_views::{collect_view_refs, desugar_view_refs};
use crate::model::Task;
use crate::numeric::{NumValue, format_num};
use crate::ready::Readiness;

/// Maximum `in_view` nesting depth evaluated before bailing. Config load rejects
/// cyclic view references ([`crate::config_views::validate_views`]), so this only
/// guards an unvalidated config (e.g. a hand-built test one) against a stack
/// overflow.
const MAX_VIEW_DEPTH: usize = 64;

/// A parsed, reusable filter expression. `None` is the match-all filter (an
/// empty or whitespace-only expression, e.g. `mt ls ''`).
#[derive(Debug, Clone)]
pub struct Filter {
    tree: Option<Node<DefaultNumericTypes>>,
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
    /// An `in_view` / `@` reference naming no declared view.
    #[error("unknown view in filter: {0}")]
    UnknownView(String),
    /// A structurally invalid expression.
    #[error("invalid filter: {0}")]
    Invalid(String),
}

/// A resolved field value during evaluation. String fields compare
/// lexicographically (ISO-correct for dates); numeric fields compare as numbers
/// against a numeric literal and fall back to their display string otherwise.
enum FieldValue {
    Str(String),
    Num(f64),
    Missing,
}

impl Filter {
    /// Parse a filter expression. `@name` view sugar is desugared to
    /// `in_view("name")` first; an empty or whitespace-only expression is the
    /// match-all filter.
    ///
    /// # Errors
    /// Returns [`FilterError::Parse`] on a syntax error.
    pub fn parse(expr: &str) -> Result<Self, FilterError> {
        let desugared = desugar_view_refs(expr);
        if desugared.trim().is_empty() {
            return Ok(Self { tree: None });
        }
        let tree = build_operator_tree::<DefaultNumericTypes>(&desugared)
            .map_err(|err| FilterError::Parse(err.to_string()))?;
        Ok(Self { tree: Some(tree) })
    }

    /// The view names this filter references through `in_view` / `@`, in
    /// traversal order (may contain duplicates). Empty for the match-all filter.
    #[must_use]
    pub fn view_refs(&self) -> Vec<String> {
        let mut out = Vec::new();
        if let Some(tree) = &self.tree {
            collect_view_refs(tree, &mut out);
        }
        out
    }

    /// Whether `task` satisfies the filter, resolving framework scores against
    /// `config`. The `ready` / `blocked` predicates are unavailable here (no
    /// task-set context); use [`matches_with`](Self::matches_with) for those.
    ///
    /// # Errors
    /// Returns [`FilterError`] on an unknown field or unsupported operator.
    pub fn matches(&self, task: &Task, config: &Config) -> Result<bool, FilterError> {
        self.matches_with(task, config, &Readiness::empty())
    }

    /// Whether `task` satisfies the filter, with `readiness` answering the
    /// `ready` / `blocked` predicates. `readiness` must be computed over the full
    /// loaded set (see [`Readiness::compute`]).
    ///
    /// # Errors
    /// Returns [`FilterError`] on an unknown field or unsupported operator.
    pub fn matches_with(
        &self,
        task: &Task,
        config: &Config,
        readiness: &Readiness,
    ) -> Result<bool, FilterError> {
        match &self.tree {
            None => Ok(true),
            Some(tree) => eval_bool(
                tree,
                &Ctx {
                    task,
                    config,
                    readiness,
                    depth: 0,
                },
            ),
        }
    }
}

/// Shared context threaded through the filter walk: the task under test, the
/// config (framework scores, terminal set, view lookup), the readiness oracle
/// (`ready` / `blocked`), and the current `in_view` recursion depth.
struct Ctx<'a> {
    task: &'a Task,
    config: &'a Config,
    readiness: &'a Readiness,
    depth: usize,
}

impl<'a> Ctx<'a> {
    /// The same context one `in_view` level deeper.
    fn deeper(&self) -> Ctx<'a> {
        Ctx {
            task: self.task,
            config: self.config,
            readiness: self.readiness,
            depth: self.depth + 1,
        }
    }
}

fn eval_bool(node: &Node<DefaultNumericTypes>, ctx: &Ctx) -> Result<bool, FilterError> {
    match node.operator() {
        Operator::RootNode => match node.children() {
            [child] => eval_bool(child, ctx),
            _ => Err(FilterError::Invalid("root needs one child".to_owned())),
        },
        Operator::And => binary(node, ctx, |left, right| left && right),
        Operator::Or => binary(node, ctx, |left, right| left || right),
        Operator::Not => match node.children() {
            [child] => Ok(!eval_bool(child, ctx)?),
            _ => Err(FilterError::Invalid("`!` needs one operand".to_owned())),
        },
        Operator::Eq
        | Operator::Neq
        | Operator::Lt
        | Operator::Gt
        | Operator::Leq
        | Operator::Geq => comparison(node, ctx),
        Operator::FunctionIdentifier { identifier } => function(identifier, node, ctx),
        Operator::VariableIdentifierRead { identifier } => nullary(identifier, ctx),
        other => Err(FilterError::Unsupported(format!("{other:?}"))),
    }
}

/// A bare identifier in boolean position: the predicates `ready` / `blocked` /
/// `terminal` are valid; any other field is a type error (use a comparison).
fn nullary(identifier: &str, ctx: &Ctx) -> Result<bool, FilterError> {
    match identifier {
        "ready" => Ok(ctx.readiness.is_ready(ctx.task)),
        "blocked" => Ok(ctx.readiness.is_blocked(ctx.task)),
        "terminal" => Ok(ctx.task.is_terminal(&ctx.config.tasks.status.terminal())),
        other => Err(FilterError::Invalid(format!(
            "bare field `{other}` is not boolean; use a comparison or function"
        ))),
    }
}

fn binary(
    node: &Node<DefaultNumericTypes>,
    ctx: &Ctx,
    combine: impl Fn(bool, bool) -> bool,
) -> Result<bool, FilterError> {
    match node.children() {
        [left, right] => Ok(combine(eval_bool(left, ctx)?, eval_bool(right, ctx)?)),
        _ => Err(FilterError::Invalid(
            "binary op needs two operands".to_owned(),
        )),
    }
}

fn comparison(node: &Node<DefaultNumericTypes>, ctx: &Ctx) -> Result<bool, FilterError> {
    let [lhs, rhs] = node.children() else {
        return Err(FilterError::Invalid(
            "comparison needs two operands".to_owned(),
        ));
    };
    let field = extract_identifier(lhs)?;
    let literal = eval_rhs(rhs)?;
    let value = field_value(ctx.task, &field, ctx.config)?;
    Ok(compare(node.operator(), &value, &literal))
}

fn compare(op: &Operator<DefaultNumericTypes>, value: &FieldValue, literal: &str) -> bool {
    match value {
        FieldValue::Missing => matches!(op, Operator::Neq),
        FieldValue::Str(text) => compare_ordering(op, Some(text.as_str().cmp(literal))),
        FieldValue::Num(num) => match literal.parse::<f64>() {
            Ok(rhs) => compare_ordering(op, num.partial_cmp(&rhs)),
            Err(_) => compare_ordering(op, Some(format_num(*num).as_str().cmp(literal))),
        },
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

fn function(name: &str, node: &Node<DefaultNumericTypes>, ctx: &Ctx) -> Result<bool, FilterError> {
    let children = node.children();
    let (task, config) = (ctx.task, ctx.config);
    match name {
        "ready" => Ok(ctx.readiness.is_ready(task)),
        "blocked" => Ok(ctx.readiness.is_blocked(task)),
        "terminal" => Ok(task.is_terminal(&config.tasks.status.terminal())),
        "in_view" => in_view(&single_literal(children, "in_view")?, ctx),
        "has" => Ok(present(task, &single_identifier(children, "has")?, config)?),
        "miss" => Ok(!present(
            task,
            &single_identifier(children, "miss")?,
            config,
        )?),
        "has_tag" => {
            let tag = single_literal(children, "has_tag")?;
            Ok(task
                .tags
                .iter()
                .chain(&task.derived_tags)
                .any(|owned| owned == &tag))
        }
        "contains" => {
            let (field, needle) = two_args(children, "contains")?;
            let name = extract_identifier(field)?;
            let needle = extract_literal(needle)?;
            Ok(contains_ci(&field_value(task, &name, config)?, &needle))
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

/// `in_view("name")`: does the task match the named view's filter? Resolves the
/// view from config, then evaluates its (desugared) filter recursively. The
/// depth in `ctx` guards against a cycle in an unvalidated config.
fn in_view(name: &str, ctx: &Ctx) -> Result<bool, FilterError> {
    if ctx.depth >= MAX_VIEW_DEPTH {
        return Err(FilterError::Invalid(format!(
            "view reference nested too deep at {name:?} (cycle?)"
        )));
    }
    let spec = ctx
        .config
        .views
        .get(name)
        .ok_or_else(|| FilterError::UnknownView(name.to_owned()))?;
    match &Filter::parse(&spec.filter)?.tree {
        None => Ok(true),
        Some(tree) => eval_bool(tree, &ctx.deeper()),
    }
}

fn present(task: &Task, name: &str, config: &Config) -> Result<bool, FilterError> {
    match name {
        "tags" => Ok(!task.tags.is_empty() || !task.derived_tags.is_empty()),
        "depends_on" => Ok(!task.depends_on.is_empty()),
        "relates_to" => Ok(!task.relates_to.is_empty()),
        "supersedes" => Ok(!task.supersedes.is_empty()),
        "duplicates" => Ok(!task.duplicates.is_empty()),
        "estimate" => Ok(!task.estimate.is_empty()),
        other => Ok(!matches!(
            field_value(task, other, config)?,
            FieldValue::Missing
        )),
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
        FieldValue::Num(num) => format_num(*num),
        FieldValue::Missing => return false,
    };
    haystack.to_lowercase().contains(&needle.to_lowercase())
}

fn field_value(task: &Task, name: &str, config: &Config) -> Result<FieldValue, FilterError> {
    let value = match name {
        "status" => opt_str(task.status.as_ref().map(|status| status.name.clone())),
        "priority" => num_field(task.priority.as_ref(), Some(&config.tasks.priority)),
        "difficulty" => num_field(task.difficulty.as_ref(), None),
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

/// A numeric field as a comparable value: a raw number compares numerically; a
/// label falls back to a string compare; a framework value resolves its score
/// through `priority` (numeric) when given the priority config.
fn num_field(value: Option<&NumValue>, priority: Option<&PriorityConfig>) -> FieldValue {
    match value {
        Some(NumValue::Num(num)) => FieldValue::Num(*num),
        Some(NumValue::Label(label)) => FieldValue::Str(label.clone()),
        Some(framework @ NumValue::Framework { .. }) => priority
            .and_then(|config| config.resolve(framework))
            .map_or(FieldValue::Missing, FieldValue::Num),
        None => FieldValue::Missing,
    }
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

pub(crate) fn extract_literal(node: &Node<DefaultNumericTypes>) -> Result<String, FilterError> {
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

/// Right-hand side of a comparison: either a literal or `now()`.
fn eval_rhs(node: &Node<DefaultNumericTypes>) -> Result<String, FilterError> {
    let node = unwrap_root(node);
    if let Operator::FunctionIdentifier { identifier } = node.operator() {
        return match identifier.as_str() {
            "now" => Ok(crate::date::today_iso()),
            other => Err(FilterError::Invalid(format!(
                "function `{other}` cannot be used as a comparison value; only `now()` is allowed here"
            ))),
        };
    }
    extract_literal(node)
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
