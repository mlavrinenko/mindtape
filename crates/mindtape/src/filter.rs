//! The filter-expression query language, evaluated over live tasks.
//!
//! `evalexpr` is used only as a parser: its AST is walked and evaluated against
//! each [`Task`] in memory (never against a database).
//!
//! # Syntax
//!
//! Comparable fields: every [`crate::fields::FIELDS`] entry, including the
//! dotted properties of a nested value (`status.name`, `status.date`,
//! `status.note`, `status.index`), plus `role` (a virtual, comparison-only
//! field with no `Task` field behind it: the literal resolves against the
//! assignee's declared roles from the `[[actors]]` registry at read time, so
//! `role == "reviewer"` matches a task whose assignee holds that role; any
//! other comparison follows the missing-field rule, making
//! `role != "reviewer"` its exact complement). `priority` and `difficulty` are
//! numeric, so they compare numerically against a number literal (e.g.
//! `priority >= 7`). The `estimate` dict itself supports presence only
//! (`has(estimate)` / `miss(estimate)`); one unit of it reads as a number,
//! `estimate.<unit>` (see [`crate::filter_predicates::estimate_unit`]). An
//! `x-<ns>` extension slot supports presence only. The same vocabulary backs
//! `--sort` — see [`crate::sort::is_sort_field`], `estimate.<unit>` aside.
//!
//! Names follow Typst's identifier rule: a `-` between name characters is part
//! of the name (`x-goals`, `estimate.src-files`), see [`join_kebab_names`].
//!
//! Operators: `==`, `!=`, `<`, `>`, `<=`, `>=`, `&&`, `||`, `!`, parentheses.
//!
//! Functions: `has(field)`, `miss(field)`, `has_tag("name")`,
//! `contains("substr")` / `contains("substr", field, ...)` — case-insensitive
//! substring, needle first; fields optional, defaulting to `title`, `id`,
//! `body` (an explicit field draws from
//! [`crate::fields::is_contains_field`]). `stale(N)`,
//! `links("kind")` (does the task carry a `links:` edge of that kind — built-in
//! or a project's custom `[[tasks.links.kinds]]`), `in_view("name")` (also
//! written `@name`, which desugars to it): matches when the named view's
//! filter matches.
//!
//! Three functions read a string rather than compare one, and may take the
//! result of another call as an argument (see [`crate::filter_strings`]):
//! `word(field, n)` (the n-th whitespace-separated word, counting from 1,
//! punctuation-trimmed), `matches(field, "regex")`, and `in_dict("name", <string
//! expr>)` — membership in a `[dicts.<name>]` set or a built-in one, under that
//! dictionary's `case`. Canonically: `in_dict("verbs", word(title, 1))`.
//!
//! `changed("<states>", "<glob>"…)` counts the distinct paths git reports
//! changed in `staged`, `worktree`, or both joined with `+`, narrowed by
//! include globs and `!`-prefixed exclude globs (see [`crate::git_state`]).
//! It is a count, so it only appears in a comparison — against a literal or a
//! field: `changed("staged", "src/**") <= estimate.files`. Either side of a
//! comparison may name a field when the other is a field or a count.
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
use crate::expr::{NAME_HYPHEN, extract_identifier, extract_literal, restore_name};
use crate::fields::{self, FieldValue};
use crate::filter_predicates::{contains_call, is_stale, present};
use crate::filter_views::{collect_view_refs, desugar_view_refs};
use crate::model::Task;
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
    /// `changed()` asked git and git could not answer — outside a
    /// repository, or git missing. A `[[tasks.rules]]` entry reading it is
    /// skipped rather than failed ([`crate::config_rules`]).
    #[error("changed() cannot read git: {0}")]
    Git(String),
}

impl Filter {
    /// Parse a filter expression. `@name` view sugar is desugared to
    /// `in_view("name")` first; an empty or whitespace-only expression is the
    /// match-all filter.
    ///
    /// # Errors
    /// Returns [`FilterError::Parse`] on a syntax error.
    pub fn parse(expr: &str) -> Result<Self, FilterError> {
        let desugared = join_kebab_names(&desugar_view_refs(expr));
        if desugared.trim().is_empty() {
            return Ok(Self { tree: None });
        }
        let tree = build_operator_tree::<DefaultNumericTypes>(&desugared)
            .map_err(|err| FilterError::Parse(restore_name(&err.to_string())))?;
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
        self.eval(task, config, readiness, false)
    }

    /// Evaluate against `task`; `dry` checks every `changed()` call's
    /// arguments without reading git (see [`expr_problem`]).
    fn eval(
        &self,
        task: &Task,
        config: &Config,
        readiness: &Readiness,
        dry: bool,
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
                    dry,
                },
            ),
        }
    }
}

/// The pre-lex pass that keeps a kebab-case name one token: a `-` between two
/// name characters, inside a name that began with a letter or `_`, becomes
/// [`NAME_HYPHEN`], which `evalexpr` lexes as a name character. Typst's
/// identifier rule, so a filter spells a name the way the task file does.
///
/// A `-` anywhere else stays an operator: a leading one (`> -1`), one after a
/// number (`1e-5`), one beside a space (`a - b`). String literals are copied
/// verbatim, escapes included.
#[must_use]
fn join_kebab_names(expr: &str) -> String {
    let mut out = String::with_capacity(expr.len());
    let mut chars = expr.chars().peekable();
    let (mut in_string, mut escaped) = (false, false);
    // Whether the current run of name characters began as a name.
    let mut in_name = false;
    // Whether the previous character belongs to a run of name characters.
    let mut in_run = false;
    while let Some(ch) = chars.next() {
        if in_string {
            in_string = escaped || ch != '"';
            escaped = !escaped && ch == '\\';
            out.push(ch);
            continue;
        }
        let name_char = ch.is_alphanumeric() || ch == '_' || ch == '.';
        let joins = ch == '-'
            && in_name
            && chars
                .peek()
                .is_some_and(|next| next.is_alphanumeric() || *next == '_');
        if joins {
            out.push(NAME_HYPHEN);
            continue;
        }
        if !in_run && name_char {
            in_name = ch.is_alphabetic() || ch == '_';
        }
        in_run = name_char;
        in_name &= name_char;
        in_string = ch == '"';
        out.push(ch);
    }
    out
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
///
/// The dry run never reads git: a `changed()` call has its states and globs
/// checked and counts zero, so loading a config outside a repository is not
/// an error and loading one inside it costs no `git status`.
#[must_use]
pub(crate) fn expr_problem(source: &str, probe: &Task, config: &Config) -> Option<String> {
    match Filter::parse(source) {
        Err(err) => Some(err.to_string()),
        Ok(filter) => filter
            .eval(probe, config, &Readiness::empty(), true)
            .err()
            .map(|err| err.to_string()),
    }
}

/// Shared context threaded through the filter walk: the task under test, the
/// config (framework scores, terminal set, view lookup), the readiness oracle
/// (`ready` / `blocked`), the current `in_view` recursion depth, and whether
/// this is the load-time dry run (see [`expr_problem`]).
pub(crate) struct Ctx<'a> {
    pub(crate) task: &'a Task,
    pub(crate) config: &'a Config,
    readiness: &'a Readiness,
    depth: usize,
    pub(crate) dry: bool,
}

impl<'a> Ctx<'a> {
    /// The same context one `in_view` level deeper.
    fn deeper(&self) -> Ctx<'a> {
        Ctx {
            task: self.task,
            config: self.config,
            readiness: self.readiness,
            depth: self.depth + 1,
            dry: self.dry,
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
        "changed" => Err(FilterError::Invalid(
            "changed() is a count; compare it: changed(\"worktree\") <= estimate.files".to_owned(),
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
    if let Some(value) = crate::filter_predicates::estimate_unit(task, name, config) {
        return value;
    }
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
