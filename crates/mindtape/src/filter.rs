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
//! `x-<ns>` extension slot reads whether or not `[tasks.fields]` declares it
//! (see [`crate::filter_predicates::extra_value`]): a scalar compares like a
//! core field, a list equals and orders against nothing, and a dict supports
//! presence only. The same vocabulary backs `--sort` — see
//! [`crate::sort::is_sort_field`], `estimate.<unit>` aside.
//!
//! Names follow Typst's identifier rule: a `-` between name characters is part
//! of the name (`x-goals`, `estimate.src-files`), see [`crate::expr::join_kebab_names`].
//!
//! Operators: `==`, `!=`, `<`, `>`, `<=`, `>=`, `&&`, `||`, `!`, parentheses.
//!
//! Functions: `has(field)`, `miss(field)`, `has(field, value)` (a list holds
//! the element, a scalar equals it, a dict has the key), `has_tag("name")`,
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
//! `touches("<path>")` matches a task named by the `[commits] trailer` of a
//! commit that changed the repository-relative path (see [`crate::commit_refs`]).
//!
//! `count(<expr>)` is a count too: how many tasks of the whole loaded set
//! match `<expr>`, each read as the task in context (see
//! [`crate::filter_aggregate`]). `count(status == "wip") <= 1`.
//!
//! `now()` evaluates to today's ISO date string (`YYYY-MM-DD`) and may appear
//! on the right-hand side of a comparison: `start <= now()`, `due < now()`.
//!
//! Dependency-aware predicates `ready` and `blocked` answer "what can I start
//! now" over the whole loaded set (see [`Readiness`]). They take no argument and
//! may be written bare (`ready`) or as a call (`ready()`). The `terminal`
//! predicate (likewise bare or `terminal()`) matches a task whose status is
//! flagged terminal in config.

use evalexpr::{DefaultNumericTypes, Node, build_operator_tree};
use thiserror::Error;

use crate::config::Config;
use crate::expr::{join_kebab_names, restore_name};
use crate::fields::{self, FieldValue};
use crate::filter_views::{collect_view_refs, desugar_view_refs};
use crate::model::Task;
use crate::ready::Readiness;

/// Maximum `in_view` nesting depth evaluated before bailing. Config load rejects
/// cyclic view references ([`crate::config_views::validate_views`]), so this only
/// guards an unvalidated config (e.g. a hand-built test one) against a stack
/// overflow.
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
    /// `changed()` or `touches()` asked git and git could not answer —
    /// outside a repository, or git missing. A `[[tasks.rules]]` entry
    /// reading it is skipped rather than failed ([`crate::config_rules`]).
    #[error("{call}() cannot read git: {reason}")]
    Git {
        /// The function that asked.
        call: &'static str,
        /// Why git could not answer.
        reason: String,
    },
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

    /// Whether `word` names a filter term on its own: a predicate or function
    /// of the language, a task field, or a view, dictionary, field or status
    /// the config declares. A caller that reads a bare token as either a task
    /// reference or a filter asks this first, so a term never addresses a task
    /// whose stem hash happens to spell it.
    #[must_use]
    pub fn names_term(word: &str, config: &Config) -> bool {
        eval::is_word(word)
            || fields::get(word).is_some()
            || config.views.contains_key(word)
            || config.dicts.contains_key(word)
            || config.tasks.fields.contains_key(word)
            || config
                .tasks
                .status
                .entries
                .iter()
                .any(|entry| entry.value == word)
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

    /// Every bare name the filter reads — a field, or a predicate written bare
    /// such as `ready` — as written. Empty for the match-all filter.
    #[must_use]
    pub fn names(&self) -> std::collections::BTreeSet<String> {
        self.tree
            .iter()
            .flat_map(Node::iter_variable_identifiers)
            .map(restore_name)
            .collect()
    }

    /// Whether the filter reads the whole task set rather than one task: it
    /// calls `count(...)`, or names a field a task takes from the others
    /// ([`SET_FIELDS`]). Such an expression can answer differently for the
    /// same task as the set around it changes.
    #[must_use]
    pub fn reads_set(&self) -> bool {
        self.names()
            .iter()
            .any(|name| SET_FIELDS.contains(&name.as_str()))
            || self.tree.as_ref().is_some_and(calls_count)
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
                    exists: None,
                },
            ),
        }
    }
}

/// The names a task takes from the tasks around it: the `links:` graph through
/// `ready`, `blocked`, `blocks` and `topo`, and its children's `children` and
/// `progress`.
const SET_FIELDS: [&str; 6] = ["ready", "blocked", "blocks", "topo", "children", "progress"];

/// Whether `node` or any node under it calls `count(...)`.
fn calls_count(node: &Node<DefaultNumericTypes>) -> bool {
    matches!(node.operator(), evalexpr::Operator::FunctionIdentifier { identifier } if identifier == "count")
        || node.children().iter().any(calls_count)
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

/// The `x-` names `source` reads that `[tasks.fields]` does not declare, each
/// as a warning with the nearest declared name. Empty while the config declares
/// no `x-` field at all, and for an expression that does not parse: a misspelt
/// name is a likely typo only beside declared ones, and reading an undeclared
/// name stays legal.
#[must_use]
pub(crate) fn undeclared_extras(source: &str, config: &Config) -> Vec<String> {
    let declared: Vec<String> = config
        .tasks
        .fields
        .keys()
        .filter(|name| fields::is_extra(name))
        .cloned()
        .collect();
    let Ok(filter) = Filter::parse(source) else {
        return Vec::new();
    };
    if declared.is_empty() {
        return Vec::new();
    }
    filter
        .names()
        .into_iter()
        .filter(|name| fields::is_extra(name) && !declared.contains(name))
        .map(|name| {
            format!(
                "`{name}` is not declared in [tasks.fields]{}",
                crate::suggest::hint("field", &name, &declared)
            )
        })
        .collect()
}

/// [`expr_problem`] for an expression evaluated over the loaded set — a
/// `[[tasks.rules]]` entry, an `[[on.flip]]` hook — so `count(...)` passes the
/// dry run: the probe is its own one-task set. The run is dry, so loading
/// never reads git for `changed()` or `touches()`.
#[must_use]
pub(crate) fn expr_problem_in_set(source: &str, probe: &Task, config: &Config) -> Option<String> {
    let set = Readiness::compute(std::slice::from_ref(probe), config);
    match Filter::parse(source) {
        Err(err) => Some(err.to_string()),
        Ok(filter) => filter
            .eval(probe, config, &set, true)
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
    pub(crate) readiness: &'a Readiness,
    depth: usize,
    pub(crate) dry: bool,
    /// Whether the task exists in the commit state being read — `Some` only
    /// under a task-scope commit rule's `head(...)` / `staged(...)` /
    /// `worktree(...)`, where `exists` means something.
    pub(crate) exists: Option<bool>,
}

impl<'a> Ctx<'a> {
    /// A context reading `task` as one commit state holds it
    /// ([`crate::commit_task_expr`]): `exists` answers whether the state holds
    /// the file at all, `readiness` is computed over that state's task set.
    pub(crate) fn in_state(
        task: &'a Task,
        config: &'a Config,
        readiness: &'a Readiness,
        exists: bool,
        dry: bool,
    ) -> Self {
        Ctx {
            task,
            config,
            readiness,
            depth: 0,
            dry,
            exists: Some(exists),
        }
    }

    /// The same context evaluating `task` instead — what `count` hands its
    /// argument for each task it counts.
    pub(crate) fn over<'b>(&self, task: &'b Task) -> Ctx<'b>
    where
        'a: 'b,
    {
        Ctx {
            task,
            config: self.config,
            readiness: self.readiness,
            depth: self.depth,
            dry: self.dry,
            // Every task of a state's set is present in it.
            exists: self.exists.map(|_| true),
        }
    }

    /// The same context one `in_view` level deeper.
    fn deeper(&self) -> Ctx<'a> {
        Ctx {
            task: self.task,
            config: self.config,
            readiness: self.readiness,
            depth: self.depth + 1,
            dry: self.dry,
            exists: self.exists,
        }
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
    if fields::is_extra(name) {
        return crate::filter_predicates::extra_value(task, name);
    }
    fields::get(name)
        .map(|def| (def.get)(task, config))
        .ok_or_else(|| FilterError::UnknownField(name.to_owned()))
}

/// The tree walk; split out to keep this file under its line budget. See
/// [`eval_bool`].
#[path = "filter_eval.rs"]
mod eval;
pub(crate) use eval::eval_bool;
pub use eval::{Arity, FilterFunction};

/// The registry of the language's predicates and functions.
#[must_use]
pub const fn functions() -> &'static [FilterFunction] {
    &eval::FUNCTIONS
}

#[cfg(test)]
#[path = "filter_tests.rs"]
mod tests;
