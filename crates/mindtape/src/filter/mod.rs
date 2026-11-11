//! The filter-expression query language, evaluated over live tasks.
//!
//! `evalexpr` is used only as a parser: its AST is walked and evaluated against
//! each [`Task`] in memory (never against a database).
//!
//! # Syntax
//!
//! Comparable fields: every [`crate::model::fields::FIELDS`] entry, including the
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
//! `estimate.<unit>` (see [`crate::filter::predicates::estimate_unit`]). An
//! `x-<ns>` extension slot reads whether or not `[tasks.fields]` declares it
//! (see [`crate::filter::predicates::extra_value`]): a scalar compares like a
//! core field, a list equals and orders against nothing, and a dict supports
//! presence only. The same vocabulary backs `--sort` — see
//! [`crate::sort::is_sort_field`], `estimate.<unit>` aside.
//!
//! Names follow Typst's identifier rule: a `-` between name characters is part
//! of the name (`x-goals`, `estimate.src-files`), see [`crate::filter::expr::join_kebab_names`].
//!
//! Operators: `==`, `!=`, `<`, `>`, `<=`, `>=`, `&&`, `||`, `!`, parentheses.
//!
//! Functions: `has(field)`, `miss(field)`, `has(field, value)` (a list holds
//! the element, a scalar equals it, a dict has the key), `has_tag("name")`,
//! `contains("substr")` / `contains("substr", field, ...)` — case-insensitive
//! substring, needle first; fields optional, defaulting to `title`, `id`,
//! `body` (an explicit field draws from
//! [`crate::model::fields::is_contains_field`]). `stale(N)`,
//! `links("kind")` (does the task carry a `links:` edge of that kind — built-in
//! or a project's custom `[[tasks.links.kinds]]`), `in_view("name")` (also
//! written `@name`, which desugars to it): matches when the named view's
//! filter matches.
//!
//! Three functions read a string rather than compare one, and may take the
//! result of another call as an argument (see [`crate::filter::strings`]):
//! `word(field, n)` (the n-th whitespace-separated word, counting from 1,
//! punctuation-trimmed), `matches(field, "regex")`, and `in_dict("name", <string
//! expr>)` — membership in a `[dicts.<name>]` set or a built-in one, under that
//! dictionary's `case`. Canonically: `in_dict("verbs", word(title, 1))`.
//! `old("field")` also gives text there, so `matches(old("title"), "^Add")`
//! reads the draft before the hook's actions ran.
//!
//! `changed("<states>", "<glob>"…)` counts the distinct paths git reports
//! changed in `staged`, `worktree`, or both joined with `+`, narrowed by
//! include globs and `!`-prefixed exclude globs (see [`crate::git::state`]).
//! It is a count, so it only appears in a comparison — against a literal or a
//! field: `changed("staged", "src/**") <= estimate.files`. Either side of a
//! comparison may name a field when the other is a field or a count.
//!
//! `touches("<path>")` matches a task named by the `[commits] trailer` of a
//! commit that changed the repository-relative path (see [`crate::commit::refs`]).
//!
//! `count(<expr>)` is a count too: how many tasks of the whole loaded set
//! match `<expr>`, each read as the task in context (see
//! [`crate::filter::aggregate`]). `count(status == "wip") <= 1`.
//!
//! `now()` evaluates to today's ISO date string (`YYYY-MM-DD`) and may appear
//! on the right-hand side of a comparison: `start <= now()`, `due < now()`.
//!
//! Dependency-aware predicates `ready` and `blocked` answer "what can I start
//! now" over the whole loaded set (see [`Readiness`]). They take no argument and
//! may be written bare (`ready`) or as a call (`ready()`). The `terminal`
//! predicate (likewise bare or `terminal()`) matches a task whose status is
//! flagged terminal in config.

//!
//! # Files
//!
//! - `eval.rs`: the tree walk and the registry of functions.
//! - `compare.rs`: comparison of a field value against a literal.
//! - `aggregate.rs`: `count`, `all` and `any` over the loaded set.
//! - `links.rs`: the `links` predicate and edge-kind lookup.
//! - `predicates.rs`: `has`, `miss`, `stale`, `changed`, `touches` and the field readers.
//! - `prefix.rs`: `starts_with` and its one-field prefix check.
//! - `paths.rs`: the fields a task's file location supplies (`path`, `dir`).
//! - `set.rs`: the fields a filter takes from the whole task set.
//! - `strings.rs`: `contains` and the other string predicates.
//! - `views.rs`: `in_view` and the `@name` sugar.
//! - `expr.rs`: the AST helpers and the kebab-name rewrite.
//! - `parse_error.rs`: a syntax error in plain words.
//! - `problems.rs`: what makes a saved expression unusable at config load.
//! - `condition.rs`: a config-held filter expression, parsed once.
//! - `condition_env.rs`: the environment a condition reads at evaluation.
//! - `error.rs`: `FilterError`.

pub(crate) mod aggregate;
pub(crate) mod compare;
mod context;
pub(crate) use context::{Ctx, field_value};
pub mod condition;
pub mod condition_env;
mod error;
pub use error::FilterError;
pub(crate) mod expr;
pub(crate) mod links;
pub(crate) mod parse_error;
mod problems;
pub(crate) use problems::{
    expr_problem, expr_problem_add, expr_problem_in_set, problem_of, undeclared_extras,
    unknown_derived,
};
pub mod paths;
pub use paths::path_dependent_fields;
pub(crate) mod predicates;
pub(crate) mod prefix;
pub(crate) mod strings;
pub use strings::DictMiss;
pub(crate) mod views;

use std::collections::BTreeSet;

use evalexpr::{DefaultNumericTypes, Node, Operator, build_operator_tree};

use crate::config::Config;
use crate::filter::expr::{
    call_args, extract_identifier, extract_literal, join_kebab_names, restore_name, unwrap_root,
};
use crate::filter::views::{collect_view_refs, desugar_view_refs};
use crate::graph::ready::Readiness;
use crate::model::Task;
use crate::model::fields::{self as fields};

/// The inputs one filter evaluation reads, grouped to keep the internal
/// `eval_with_old` seam under the argument limit.
#[derive(Clone, Copy)]
pub(crate) struct Eval<'a> {
    pub(crate) task: &'a Task,
    pub(crate) old: Option<&'a Task>,
    pub(crate) config: &'a Config,
    pub(crate) readiness: &'a Readiness,
    pub(crate) dry: bool,
}

/// A parsed filter expression, or the match-all filter for an empty or
/// whitespace-only expression, e.g. `mt ls ''`.
#[derive(Debug, Clone)]
pub struct Filter {
    tree: Option<Node<DefaultNumericTypes>>,
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
        parse_error::refuse_stray_symbol(&desugared)?;
        let tree = build_operator_tree::<DefaultNumericTypes>(&desugared)
            .map_err(|err| FilterError::Parse(parse_error::plain_parse_error(&err)))?;
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
            || (fields::is_derived(word) && fields::derived_type(config, word).is_some())
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

    /// Refuse a string compared with `status` that the legend does not
    /// declare. An empty legend accepts any status.
    ///
    /// # Errors
    /// [`FilterError::UnknownStatus`] for the first such comparison.
    pub(crate) fn check_statuses(&self, legend: &crate::config::Legend) -> Result<(), FilterError> {
        self.tree
            .as_ref()
            .map_or(Ok(()), |tree| compare::check_statuses(tree, legend))
    }

    /// The view names this filter references through `in_view` / `@`, in
    /// traversal order (may contain duplicates). Empty for the match-all filter.
    #[must_use]
    pub fn view_refs(&self) -> Vec<String> {
        self.collect_with(collect_view_refs)
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

    /// Every function the filter calls, as written. Empty for the match-all
    /// filter. A call is not a bare name, so [`Self::names`] misses it.
    #[must_use]
    pub fn calls(&self) -> BTreeSet<String> {
        let mut calls = BTreeSet::new();
        for name in self.tree.iter().flat_map(Node::iter_function_identifiers) {
            calls.insert(restore_name(name));
        }
        calls
    }

    /// Every tag the filter reads through `has_tag("name")` or
    /// `has(tags, "name")`, however either call is spaced, in sorted order.
    /// Empty for the match-all filter.
    #[must_use]
    pub fn tag_reads(&self) -> BTreeSet<String> {
        self.collect_with(collect_tag_reads)
    }

    /// Fill a fresh collection with whatever `walk` gathers from this filter's
    /// parsed tree. Empty for the match-all filter.
    fn collect_with<T: Default>(&self, walk: fn(&Node<DefaultNumericTypes>, &mut T)) -> T {
        let mut out = T::default();
        if let Some(tree) = &self.tree {
            walk(tree, &mut out);
        }
        out
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
            || self
                .calls()
                .iter()
                .any(|name| crate::filter::is_set_function(name))
    }

    /// Whether the filter reads where the task's file lives (`path` or `dir`,
    /// [`path_dependent_fields`]). A task with no file yet, such as a filled
    /// template example, has no value for either.
    #[must_use]
    pub fn reads_path(&self) -> bool {
        let dependent = path_dependent_fields();
        self.names()
            .iter()
            .any(|name| dependent.iter().any(|field| *field == name))
    }

    /// [`Self::reads_set`], following `in_view` into the named views. A view
    /// that calls `count(...)` makes the filter reading it set-aware too.
    #[must_use]
    pub fn reads_set_in(&self, config: &Config) -> bool {
        let mut seen = BTreeSet::new();
        self.any_through(config, &mut seen, Filter::reads_set)
    }

    /// Whether this filter or any view it reaches, directly or through another
    /// view, satisfies `holds`. `seen` breaks a view cycle.
    fn any_through(
        &self,
        config: &Config,
        seen: &mut BTreeSet<String>,
        holds: fn(&Filter) -> bool,
    ) -> bool {
        if holds(self) {
            return true;
        }
        for view in self.view_refs() {
            if !seen.insert(view.clone()) {
                continue;
            }
            if let Some(spec) = config.views.get(&view)
                && Filter::parse(&spec.filter)
                    .is_ok_and(|filter| filter.any_through(config, seen, holds))
            {
                return true;
            }
        }
        false
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

    /// Whether `task` satisfies the filter, with `old` answering the `old()`
    /// predicate: the task before an `[[on.add]]` hook's actions ran. Only an
    /// add hook has such a draft, so every other caller uses
    /// [`Self::matches_with`].
    ///
    /// # Errors
    /// Returns [`FilterError`] on an unknown field or unsupported operator.
    pub fn matches_add(
        &self,
        task: &Task,
        old: &Task,
        config: &Config,
        readiness: &Readiness,
    ) -> Result<bool, FilterError> {
        self.eval_with_old(Eval {
            task,
            old: Some(old),
            config,
            readiness,
            dry: false,
        })
    }

    /// Why the filter read false for `task`: the first `in_dict()` call
    /// outside any `!` whose dictionary lacks its word (see
    /// [`strings::dict_miss`]). `None` when no such call missed.
    #[must_use]
    pub fn dict_miss(&self, task: &Task, config: &Config) -> Option<DictMiss> {
        strings::dict_miss(self.tree.as_ref()?, task, config)
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
        self.eval_with_old(Eval {
            task,
            old: None,
            config,
            readiness,
            dry,
        })
    }

    /// [`Self::eval`] with the pre-hook draft `old()` reads, when one exists.
    pub(crate) fn eval_with_old(&self, eval: Eval<'_>) -> Result<bool, FilterError> {
        match &self.tree {
            None => Ok(true),
            Some(tree) => eval_bool(
                tree,
                &Ctx {
                    task: eval.task,
                    config: eval.config,
                    readiness: eval.readiness,
                    depth: 0,
                    dry: eval.dry,
                    exists: None,
                    old: eval.old,
                },
            ),
        }
    }
}

/// Every bare name a saved expression reads, following `in_view` into the
/// named views. Empty for an expression that does not parse.
#[must_use]
pub(crate) fn names_in(source: &str, config: &Config) -> BTreeSet<String> {
    set::reads_in(source, config, Filter::names)
}

/// The names a filter takes from the whole task set; split out to keep this
/// file under its line budget. See [`Filter::reads_set`].
mod set;
use set::SET_FIELDS;
pub(crate) use set::calls_in;

/// Collect every tag `node` reads through `has_tag("name")` or
/// `has(tags, "name")` into `tags`. A call whose tag argument is not a
/// literal counts for nothing, and so does `has()` on any other field.
fn collect_tag_reads(node: &Node<DefaultNumericTypes>, tags: &mut BTreeSet<String>) {
    let node = unwrap_root(node);
    if let Operator::FunctionIdentifier { identifier } = node.operator() {
        let args = call_args(node.children());
        match (identifier.as_str(), args.as_slice()) {
            ("has_tag", [tag]) => push_tag(tag, tags),
            ("has", [field, tag]) if names_tags(field) => push_tag(tag, tags),
            _ => {}
        }
    }
    for child in node.children() {
        collect_tag_reads(child, tags);
    }
}

/// Whether `field` is the `tags` field `has(field, value)` tests.
fn names_tags(field: &Node<DefaultNumericTypes>) -> bool {
    extract_identifier(field).is_ok_and(|name| name == "tags")
}

fn push_tag(tag: &Node<DefaultNumericTypes>, tags: &mut BTreeSet<String>) {
    if let Ok(tag) = extract_literal(tag) {
        tags.insert(tag);
    }
}

/// The tree walk; split out to keep this file under its line budget. See
/// [`eval_bool`].
mod eval;
pub(crate) use eval::eval_bool;
pub use eval::{Arity, FilterFunction};

/// The registry of the language's predicates and functions.
#[must_use]
pub const fn functions() -> &'static [FilterFunction] {
    &eval::FUNCTIONS
}

/// Whether `name` calls a function whose answer depends on the whole task set.
#[must_use]
pub(crate) fn is_set_function(name: &str) -> bool {
    set::is_set_function(name)
}

#[cfg(test)]
mod tests;

#[cfg(test)]
#[path = "derived_tests.rs"]
mod derived_tests;
