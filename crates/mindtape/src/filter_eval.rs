//! The walk over a parsed filter's `evalexpr` tree, split out of `filter.rs`
//! (file-size gate): the boolean operators, the function table, the AST
//! helpers those read, and `in_view` recursion. [`Ctx`] and `field_value`
//! stay in the parent; [`eval_bool`] is re-exported there so the sibling walk
//! modules keep their `crate::filter::eval_bool` path.
//!
//! Maximum `in_view` nesting depth evaluated before bailing. Config load
//! rejects cyclic view references ([`crate::config_views::validate_views`]),
//! so this only guards an unvalidated config (a hand-built test one) against
//! a stack overflow.
const MAX_VIEW_DEPTH: usize = 64;

use evalexpr::{DefaultNumericTypes, Node, Operator};

use crate::expr::{extract_identifier, extract_literal, restore_name};
use crate::filter::{Ctx, Filter, FilterError};
use crate::filter_predicates::{contains_call, is_stale, present};

/// Every bare word the language reads as a predicate or a function name:
/// [`nullary`] and [`function`] dispatch on exactly these.
pub(crate) const WORDS: [&str; 17] = [
    "ready", "blocked", "terminal", "exists", "in_view", "has", "miss", "has_tag", "links",
    "contains", "touches", "matches", "in_dict", "word", "changed", "count", "stale",
];

pub(crate) fn eval_bool(node: &Node<DefaultNumericTypes>, ctx: &Ctx) -> Result<bool, FilterError> {
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
        | Operator::Geq => crate::filter_compare::comparison(node, ctx),
        Operator::FunctionIdentifier { identifier } => {
            function(&restore_name(identifier), node, ctx)
        }
        Operator::VariableIdentifierRead { identifier } => nullary(&restore_name(identifier), ctx),
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
        "exists" => crate::filter_predicates::exists(ctx),
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

fn function(name: &str, node: &Node<DefaultNumericTypes>, ctx: &Ctx) -> Result<bool, FilterError> {
    let children = node.children();
    let (task, config) = (ctx.task, ctx.config);
    match name {
        "ready" => Ok(ctx.readiness.is_ready(task)),
        "blocked" => Ok(ctx.readiness.is_blocked(task)),
        "terminal" => Ok(task.is_terminal(&config.tasks.status.terminal())),
        "exists" => crate::filter_predicates::exists(ctx),
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
        "contains" => contains_call(children, task, config),
        "touches" => crate::filter_predicates::touches(children, ctx),
        "matches" => crate::filter_strings::matches_call(children, task, config),
        "in_dict" => crate::filter_strings::in_dict_call(children, task, config),
        "word" => Err(crate::filter_strings::word_is_not_boolean()),
        "changed" => Err(FilterError::Invalid(
            "changed() is a count; compare it: changed(\"worktree\") <= estimate.files".to_owned(),
        )),
        "count" => Err(FilterError::Invalid(
            "count() is a number, not a predicate; compare it: `count(...) <= 1`".to_owned(),
        )),
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
