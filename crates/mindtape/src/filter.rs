//! The filter-expression query language, evaluated over live tasks.
//!
//! `evalexpr` is used only as a parser: its AST is walked and evaluated against
//! each [`Task`] in memory (never against a database).
//!
//! # Syntax
//!
//! Comparable fields: `status`, `priority`, `difficulty`, `kind`, `title`,
//! `id`, `due`, `start`, `reviewed`, `file`, `assignee` (the actor name, a
//! string compare), and `role` (a virtual, comparison-only field: the literal
//! resolves against the assignee's declared roles from the `[[actors]]`
//! registry at read time, so `role == "reviewer"` matches a task whose
//! assignee holds that role; any other comparison follows the missing-field
//! rule, making `role != "reviewer"` its exact complement). `priority` and
//! `difficulty` are numeric, so they compare numerically against a number
//! literal (e.g. `priority >= 7`). The opaque `estimate` dict supports presence
//! only (`has(estimate)` / `miss(estimate)`), not comparison.
//!
//! Operators: `==`, `!=`, `<`, `>`, `<=`, `>=`, `&&`, `||`, `!`, parentheses.
//!
//! Functions: `has(field)`, `miss(field)`, `has_tag("name")`,
//! `contains(field, "substr")`, `search("query")`, `stale(N)`,
//! `links("kind")` (does the task carry a `links:` edge of that kind — built-in
//! or a project's custom `[[tasks.links.kinds]]`), `in_view("name")` (also
//! written `@name`, which desugars to it): matches when the named view's
//! filter matches. `has(depends_on)` / `has(supersedes)` / `has(duplicates)`
//! are legacy sugar for `links("depends-on")` etc — see
//! [`crate::filter_links::sugar_kind`].
//!
//! `now()` evaluates to today's ISO date string (`YYYY-MM-DD`) and may appear
//! on the right-hand side of a comparison: `start <= now()`, `due < now()`.
//!
//! Dependency-aware predicates `ready` and `blocked` answer "what can I start
//! now" over the whole loaded set (see [`Readiness`]). They take no argument and
//! may be written bare (`ready`) or as a call (`ready()`). The `terminal`
//! predicate (likewise bare or `terminal()`) matches a task whose status is
//! flagged terminal in config.

use evalexpr::{DefaultNumericTypes, Node, Operator, build_operator_tree};
use thiserror::Error;

use crate::config::{Config, PriorityConfig};
use crate::expr::{
    compare_ordering, extract_identifier, extract_literal, resolve_now_or_literal, unwrap_root,
};
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
    let field = extract_identifier(lhs).map_err(FilterError::Invalid)?;
    let literal = resolve_now_or_literal(rhs).map_err(FilterError::Invalid)?;
    let value = if field == "role" {
        role_field(ctx.task, ctx.config, &literal)
    } else {
        field_value(ctx.task, &field, ctx.config)?
    };
    Ok(compare(node.operator(), &value, &literal))
}

/// `role` is a virtual, comparison-only field resolved from the actors
/// registry at read time: a task matches a role literal when its assignee is a
/// declared actor holding that role. Every other comparison resolves to
/// [`FieldValue::Missing`], so the missing-field rule applies uniformly and
/// `role == "reviewer"` / `role != "reviewer"` partition the whole set with no
/// third case.
fn role_field(task: &Task, config: &Config, literal: &str) -> FieldValue {
    match task
        .assignee
        .as_deref()
        .and_then(|name| crate::config_actors::roles_of(name, config))
    {
        Some(roles) if roles.iter().any(|role| role == literal) => {
            FieldValue::Str(literal.to_owned())
        }
        _ => FieldValue::Missing,
    }
}

/// Compare one resolved field value against a literal.
///
/// Missing-field rule: a field the task does not carry is unequal to every
/// literal and orders against none — `!=` holds, `==` / `<` / `>` / `<=` / `>=`
/// are all false. So `kind != "umbrella"` matches a task with no `kind` at all,
/// and the pair `kind == "umbrella"` / `kind != "umbrella"` partitions the whole
/// set with no third case. `[[tasks.limits]]`
/// ([`crate::config_limits`]) rests on that: one limit over each half of such a
/// pair caps every task by construction, including the ones leaving the field
/// unset. Pinned by `missing_field_is_unequal_to_every_literal` in
/// `filter_tests.rs` — do not relax it without revisiting that rule.
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
        "links" => {
            let kind = single_literal(children, "links")?;
            Ok(crate::filter_links::has_kind(task, &kind))
        }
        "contains" => {
            let (field, needle) = two_args(children, "contains")?;
            let name = extract_identifier(field).map_err(FilterError::Invalid)?;
            let needle = extract_literal(needle).map_err(FilterError::Invalid)?;
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
    if let Some(kind) = crate::filter_links::sugar_kind(name) {
        return Ok(crate::filter_links::has_kind(task, kind));
    }
    match name {
        "tags" => Ok(!task.tags.is_empty() || !task.derived_tags.is_empty()),
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
        "file" => FieldValue::Str(task.file_key()),
        "assignee" => opt_str(task.assignee.clone()),
        "depth" => FieldValue::Num(task.depth.into()),
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

// --- AST helpers (ported from the SQL translator; shared walk primitives now
// live in `crate::expr`, consumed by both this module and `condition.rs`) ---

fn single_identifier(
    children: &[Node<DefaultNumericTypes>],
    name: &str,
) -> Result<String, FilterError> {
    match children {
        [arg] => extract_identifier(arg).map_err(FilterError::Invalid),
        _ => Err(FilterError::Invalid(format!("{name}() needs one argument"))),
    }
}

fn single_literal(
    children: &[Node<DefaultNumericTypes>],
    name: &str,
) -> Result<String, FilterError> {
    match children {
        [arg] => extract_literal(arg).map_err(FilterError::Invalid),
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
