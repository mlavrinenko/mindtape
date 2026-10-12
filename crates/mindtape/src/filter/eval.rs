//! The walk over a parsed filter's `evalexpr` tree, split out of `filter/mod.rs`
//! (file-size gate): the boolean operators, the function table, the AST
//! helpers those read, and `in_view` recursion. [`Ctx`] and `field_value`
//! stay in the parent; [`eval_bool`] is re-exported there so the sibling walk
//! modules keep their `crate::filter::eval_bool` path.
//!
//! Maximum `in_view` nesting depth evaluated before bailing. Config load
//! rejects cyclic view references ([`crate::config::views::validate_views`]),
//! so this only guards an unvalidated config (a hand-built test one) against
//! a stack overflow.
const MAX_VIEW_DEPTH: usize = 64;

use evalexpr::{DefaultNumericTypes, Node, Operator};

use crate::filter::expr::{describe, extract_identifier, extract_literal, restore_name};
use crate::filter::predicates::{contains_call, is_stale, present};
use crate::filter::{Ctx, Filter, FilterError};

type Run = fn(&[Node<DefaultNumericTypes>], &Ctx) -> Result<bool, FilterError>;

/// How many arguments a filter word takes: `max` is `None` when it is
/// variadic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Arity {
    pub min: u8,
    pub max: Option<u8>,
}

const fn exactly(count: u8) -> Arity {
    Arity {
        min: count,
        max: Some(count),
    }
}

/// One predicate or function of the filter language: its name, its arity and
/// the code that evaluates it.
#[derive(Clone, Copy)]
pub struct FilterFunction {
    pub name: &'static str,
    pub arity: Arity,
    run: Run,
}

const fn word(name: &'static str, arity: Arity, run: Run) -> FilterFunction {
    FilterFunction { name, arity, run }
}

/// The one registry of the language's predicates and functions. [`nullary`]
/// and [`function`] dispatch through it, [`Filter::names_term`] reads its
/// names, and `mt _cli --json` carries it so the docs gate can check that
/// each name is documented.
pub(crate) const FUNCTIONS: [FilterFunction; 17] = [
    word("ready", exactly(0), |_, ctx| {
        Ok(ctx.readiness.is_ready(ctx.task))
    }),
    word("blocked", exactly(0), |_, ctx| {
        Ok(ctx.readiness.is_blocked(ctx.task))
    }),
    word("terminal", exactly(0), |_, ctx| {
        Ok(ctx.task.is_terminal(&ctx.config.tasks.status.terminal()))
    }),
    word("exists", exactly(0), |_, ctx| {
        crate::filter::predicates::exists(ctx)
    }),
    word("in_view", exactly(1), |args, ctx| {
        in_view(&single_literal(args, "in_view")?, ctx)
    }),
    word(
        "has",
        Arity {
            min: 1,
            max: Some(2),
        },
        |args, ctx| crate::filter::predicates::has_call(args, ctx.task, ctx.config),
    ),
    word("miss", exactly(1), |args, ctx| {
        Ok(!present(
            ctx.task,
            &single_identifier(args, "miss")?,
            ctx.config,
        )?)
    }),
    word("has_tag", exactly(1), |args, ctx| {
        let tag = single_literal(args, "has_tag")?;
        Ok(ctx
            .task
            .tags
            .iter()
            .chain(&ctx.task.derived_tags)
            .any(|owned| owned == &tag))
    }),
    word("links", exactly(1), |args, ctx| {
        let kind = single_literal(args, "links")?;
        Ok(crate::filter::links::has_kind(ctx.task, &kind))
    }),
    word("contains", Arity { min: 1, max: None }, |args, ctx| {
        contains_call(args, ctx.task, ctx.config)
    }),
    word("touches", exactly(1), crate::filter::predicates::touches),
    word("matches", exactly(2), |args, ctx| {
        crate::filter::strings::matches_call(args, ctx.task, ctx.config)
    }),
    word("in_dict", exactly(2), |args, ctx| {
        crate::filter::strings::in_dict_call(args, ctx.task, ctx.config)
    }),
    word("word", exactly(2), |_, _| {
        Err(crate::filter::strings::word_is_not_boolean())
    }),
    word("changed", exactly(1), |_, _| {
        Err(FilterError::call(
            "changed() is a count, not a condition",
            "changed(\"worktree\") <= estimate.files",
        ))
    }),
    word("count", Arity { min: 1, max: None }, |_, _| {
        Err(FilterError::call(
            "count() is a number, not a condition",
            "count(status == \"wip\") <= 1",
        ))
    }),
    word("stale", exactly(1), |args, ctx| {
        let count_str = single_literal(args, "stale")?;
        let count: u64 = count_str.parse().map_err(|_| {
            FilterError::call(
                format!("stale() takes a whole number of days, not '{count_str}'"),
                "stale(30)",
            )
        })?;
        Ok(is_stale(ctx.task, count))
    }),
];

/// An operator with a side missing: `status ==`, `&& wip`, `!`.
fn incomplete() -> FilterError {
    FilterError::Invalid("the filter is incomplete".to_owned())
}

/// A call to a name that is no function of the language, with the nearest
/// function when one is close.
fn unknown_function(name: &str) -> FilterError {
    let known: Vec<&str> = FUNCTIONS.iter().map(|entry| entry.name).collect();
    FilterError::UnknownFunction {
        name: name.to_owned(),
        nearest: crate::suggest::nearest_owned(name, &known),
    }
}

fn lookup(name: &str) -> Option<&'static FilterFunction> {
    FUNCTIONS.iter().find(|entry| entry.name == name)
}

/// Whether `name` is a predicate or function of the language.
pub(crate) fn is_word(name: &str) -> bool {
    lookup(name).is_some()
}

pub(crate) fn eval_bool(node: &Node<DefaultNumericTypes>, ctx: &Ctx) -> Result<bool, FilterError> {
    match node.operator() {
        Operator::RootNode => match node.children() {
            [child] => eval_bool(child, ctx),
            _ => Err(incomplete()),
        },
        Operator::And => binary(node, ctx, |left, right| left && right),
        Operator::Or => binary(node, ctx, |left, right| left || right),
        Operator::Not => match node.children() {
            [child] => Ok(!eval_bool(child, ctx)?),
            _ => Err(incomplete()),
        },
        Operator::Eq
        | Operator::Neq
        | Operator::Lt
        | Operator::Gt
        | Operator::Leq
        | Operator::Geq => crate::filter::compare::comparison(node, ctx),
        Operator::FunctionIdentifier { identifier } => {
            function(&restore_name(identifier), node, ctx)
        }
        Operator::VariableIdentifierRead { identifier } => nullary(&restore_name(identifier), ctx),
        Operator::Const { .. } => Err(FilterError::Unsupported(format!(
            "{} on its own isn't a condition",
            describe(node)
        ))),
        _ => Err(FilterError::Unsupported(format!(
            "{} isn't an operator the filter language has",
            describe(node)
        ))),
    }
}

/// A bare identifier in boolean position: the registry's predicates of arity
/// zero are valid; any other field is a type error (use a comparison).
fn nullary(identifier: &str, ctx: &Ctx) -> Result<bool, FilterError> {
    match lookup(identifier) {
        Some(entry) if entry.arity.max == Some(0) => (entry.run)(&[], ctx),
        Some(_) => Err(FilterError::Invalid(format!(
            "'{identifier}' is a function that needs arguments, not a condition on its own"
        ))),
        None => Err(FilterError::call(
            format!("'{identifier}' is a field, not a condition"),
            "status == \"wip\"",
        )),
    }
}

fn binary(
    node: &Node<DefaultNumericTypes>,
    ctx: &Ctx,
    combine: impl Fn(bool, bool) -> bool,
) -> Result<bool, FilterError> {
    match node.children() {
        [left, right] => Ok(combine(eval_bool(left, ctx)?, eval_bool(right, ctx)?)),
        _ => Err(incomplete()),
    }
}

fn function(name: &str, node: &Node<DefaultNumericTypes>, ctx: &Ctx) -> Result<bool, FilterError> {
    let entry = lookup(name).ok_or_else(|| unknown_function(name))?;
    (entry.run)(node.children(), ctx)
}

/// `in_view("name")`: does the task match the named view's filter? Resolves the
/// view from config, then evaluates its (desugared) filter recursively. The
/// depth in `ctx` guards against a cycle in an unvalidated config.
fn in_view(name: &str, ctx: &Ctx) -> Result<bool, FilterError> {
    if ctx.depth >= MAX_VIEW_DEPTH {
        return Err(FilterError::Invalid(format!(
            "view '{name}' is nested too deep; check the views for a cycle"
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
// live in `crate::filter::expr`, consumed by both this module and `condition.rs`) ---

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
