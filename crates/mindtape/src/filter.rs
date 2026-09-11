//! The filter-expression query language, evaluated over live tasks.
//!
//! `evalexpr` is used only as a parser: its AST is walked and evaluated against
//! each [`Task`] in memory (never against a database).
//!
//! # Syntax
//!
//! Comparable fields: [`crate::fields::FIELDS`] (`status`, `priority`,
//! `difficulty`, `kind`, `title`, `id`, `due`, `start`, `reviewed`, `path`,
//! `assignee`, `depth`) plus `role` (a virtual, comparison-only field with no
//! `Task` field behind it: the literal resolves against the assignee's
//! declared roles from the `[[actors]]` registry at read time, so
//! `role == "reviewer"` matches a task whose assignee holds that role; any
//! other comparison follows the missing-field rule, making
//! `role != "reviewer"` its exact complement). `priority` and `difficulty` are
//! numeric, so they compare numerically against a number literal (e.g.
//! `priority >= 7`). The opaque `estimate` dict supports presence only
//! (`has(estimate)` / `miss(estimate)`), not comparison. The same field set
//! backs `--sort` — see [`crate::sort::SORT_FIELDS`].
//!
//! Operators: `==`, `!=`, `<`, `>`, `<=`, `>=`, `&&`, `||`, `!`, parentheses.
//!
//! Functions: `has(field)`, `miss(field)`, `has_tag("name")`,
//! `contains("substr")` / `contains("substr", field, ...)` — case-insensitive
//! substring, needle first; fields optional, defaulting to `title`, `id`,
//! `body` (an explicit field draws from
//! [`crate::fields::is_contains_field`]). The old `contains(field, "substr")`
//! order and `search(…)` both hard-error, naming the replacement. `stale(N)`,
//! `links("kind")` (does the task carry a `links:` edge of that kind — built-in
//! or a project's custom `[[tasks.links.kinds]]`), `in_view("name")` (also
//! written `@name`, which desugars to it): matches when the named view's
//! filter matches. `has(depends_on)` / `has(supersedes)` / `has(duplicates)`
//! are legacy sugar for `links("depends-on")` etc — see
//! [`crate::filter_links::sugar_kind`].
//!
//! Three functions read a string rather than compare one, and may take the
//! result of another call as an argument (see [`crate::filter_strings`]):
//! `word(field, n)` (the n-th whitespace-separated word, counting from 1,
//! punctuation-trimmed), `matches(field, "regex")`, and `in_dict("name", <string
//! expr>)` — membership in a `[dicts.<name>]` set or a built-in one, under that
//! dictionary's `case`. Canonically: `in_dict("verbs", word(title, 1))`.
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

use crate::config::Config;
use crate::expr::{compare_ordering, extract_identifier, extract_literal, resolve_now_or_literal};
use crate::fields::{self, FieldValue};
use crate::filter_predicates::{contains_call, is_stale, present};
use crate::filter_views::{collect_view_refs, desugar_view_refs};
use crate::model::Task;
use crate::numeric::format_num;
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
    /// A field name the filter language does not know. A name the registry
    /// retired (`file`, now `path`) appends
    /// [`crate::fields::rename_hint`]'s clause naming the replacement, so the
    /// reader who copied a name out of the `file` column is told what to type
    /// rather than only that they are wrong.
    #[error("unknown field in filter: {}{}", .0, fields::rename_hint(.0))]
    UnknownField(String),
    /// An `in_view` / `@` reference naming no declared view.
    #[error("unknown view in filter: {0}")]
    UnknownView(String),
    /// An `in_dict` reference naming no compiled dictionary — neither a
    /// `[dicts.<name>]` section nor a built-in. `hint` carries the nearest
    /// match and the full enumeration ([`crate::suggest::hint`]), because a
    /// dictionary name is invisible in the listing a wrong one produces.
    #[error("unknown dict in filter: {name}{hint}")]
    UnknownDict {
        /// The name as written in the expression.
        name: String,
        /// The trailing suggestion clause, or empty.
        hint: String,
    },
    /// A structurally invalid expression.
    #[error("invalid filter: {0}")]
    Invalid(String),
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

/// What makes one saved filter expression unusable, if anything: a syntax
/// error, or a structural problem a dry-run against `probe` surfaces — an
/// unknown field, an unknown view, an unknown dictionary, an invalid regex.
///
/// The load-time half of every config section that stores an expression
/// (`required`, `[[tasks.limits]]`, `[[tasks.rules]]`). Each of those calls it
/// once per expression from [`crate::config::Config::validate`] and prefixes
/// the answer with its own dotted TOML path, so a malformed expression is one
/// config error at load rather than a surprise per artifact — or, for a rule,
/// one violation per artifact.
#[must_use]
pub(crate) fn expr_problem(source: &str, probe: &Task, config: &Config) -> Option<String> {
    match Filter::parse(source) {
        Err(err) => Some(err.to_string()),
        Ok(filter) => filter
            .matches(probe, config)
            .err()
            .map(|err| err.to_string()),
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
        "contains" => contains_call(children, task, config),
        "matches" => crate::filter_strings::matches_call(children, task, config),
        "in_dict" => crate::filter_strings::in_dict_call(children, task, config),
        "word" => Err(crate::filter_strings::word_is_not_boolean()),
        "search" => Err(FilterError::Invalid(
            "search(...) is gone; use contains(\"needle\") — needle first, fields optional \
             (contains(\"needle\", field, ...))"
                .to_owned(),
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

/// Resolve a field name to its value on `task`, via the shared
/// [`crate::fields`] registry — the single place every generically comparable
/// field is wired, so a field usable here is usable in `--sort` too. `pub(crate)`
/// so [`crate::filter_predicates`]'s `present`/`contains_value` can share it.
pub(crate) fn field_value(
    task: &Task,
    name: &str,
    config: &Config,
) -> Result<FieldValue, FilterError> {
    fields::get(name)
        .map(|def| (def.get)(task, config))
        .ok_or_else(|| FilterError::UnknownField(name.to_owned()))
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

#[cfg(test)]
#[path = "filter_tests.rs"]
mod tests;
