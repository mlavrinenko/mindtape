//! The expression language of a `scope = "commit"` rule: the filter
//! language's operators over a [`CommitRecord`] instead of a task.
//!
//! Same `evalexpr` parse and the same operators as [`crate::filter`] (`==`,
//! `!=`, `<`, `>`, `<=`, `>=`, `&&`, `||`, `!`, parentheses), walked by hand.
//! The fields are the commit's:
//!
//! - `subject` — the message's first line; compares against a literal.
//! - `refs` — the stems the task trailer names.
//! - `files` — every path the commit changes, repository-relative.
//! - `trailers` — every trailer line of the message as `Key: value`, whatever
//!   its key: `matches(trailers, "^Changelog: none$")`.
//! - `tasks_only` — a boolean, bare or called: the commit changes only task
//!   files.
//!
//! Functions: `has(field)` / `miss(field)` (a non-empty subject, at least one
//! ref or file) and `matches(field, "regex")` — over a list, whether ANY entry
//! matches, so `!matches(files, "^services/")` reads "touches nothing under
//! `services/`".
//!
//! `count(files, "glob"…)` is a number, so it only sits on the left of a
//! comparison: how many of the commit's files the globs admit, with the same
//! `"!glob"` excludes as `changed()` ([`PathFilter`]) —
//! `count(files, "!docs/**") <= 1`. A rename counts once, at its destination,
//! as in `changed()`, and an `"@name"` glob reads the `[dicts.<name>]` path
//! set, so the two calls share one list. It reads the commit's own files, so it
//! also works under `mt check --commits`, where `changed()` is skipped.
//!
//! A `scope = "task"` rule reads the same fields, plus the task's states:
//! `head(...)`, `staged(...)`, `worktree(...)` and `named`, evaluated by
//! [`crate::commit_task_expr`] against the [`TaskScope`] it runs under.
//!
//! A task field is refused by name: a commit rule sees the commit, not a
//! task, and a bare `status` that silently read nothing would be a rule that
//! never fires. Load dry-runs every expression against [`CommitRecord::default`],
//! so the refusal is one config error.

use evalexpr::{DefaultNumericTypes, Node, Operator, build_operator_tree};
use regex::Regex;

use crate::commit_msg::CommitRecord;
use crate::commit_task_expr::{STATES, TaskScope};
use crate::config::Config;
use crate::expr::{
    call_args, connective, extract_identifier, extract_literal, restore_name, unwrap_root,
};
use crate::fields::FieldValue;
use crate::filter::FilterError;
use crate::git_state::PathFilter;

/// The field names a commit expression reads, for the refusal message.
const COMMIT_FIELDS: &str = "subject, refs, files, trailers, tasks_only";

/// A parsed commit-scope expression.
#[derive(Debug, Clone)]
pub struct CommitExpr {
    tree: Option<Node<DefaultNumericTypes>>,
}

/// What an expression is evaluated over: the commit, and under a task-scope
/// rule the task it runs for.
#[derive(Clone, Copy)]
struct Env<'a> {
    commit: &'a CommitRecord,
    config: &'a Config,
    task: Option<&'a TaskScope<'a>>,
}

impl<'a> Env<'a> {
    /// The task scope, or the refusal naming what needs one.
    fn task(&self, what: &str) -> Result<&'a TaskScope<'a>, FilterError> {
        self.task.ok_or_else(|| {
            FilterError::Invalid(format!(
                "{what} reads a task; only a scope = \"task\" commit rule has one"
            ))
        })
    }
}

impl CommitExpr {
    /// Parse `expr`; an empty one always holds.
    ///
    /// # Errors
    /// [`FilterError::Parse`] on a syntax error.
    pub fn parse(expr: &str) -> Result<Self, FilterError> {
        let joined = crate::expr::join_kebab_names(expr);
        if joined.trim().is_empty() {
            return Ok(Self { tree: None });
        }
        let tree = build_operator_tree::<DefaultNumericTypes>(&joined)
            .map_err(|err| FilterError::Parse(restore_name(&err.to_string())))?;
        Ok(Self { tree: Some(tree) })
    }

    /// Whether `commit` satisfies the expression.
    ///
    /// # Errors
    /// [`FilterError`] on a task field, an unknown function, or a malformed
    /// call.
    pub fn matches(&self, commit: &CommitRecord, config: &Config) -> Result<bool, FilterError> {
        self.eval(Env {
            commit,
            config,
            task: None,
        })
    }

    /// Whether `commit` satisfies the expression for the task `scope` names —
    /// a `scope = "task"` rule's evaluation.
    ///
    /// # Errors
    /// [`FilterError`] on a bare task field, an unknown function, a malformed
    /// call, or a `worktree(...)` where there is no working tree.
    pub fn matches_task(
        &self,
        commit: &CommitRecord,
        scope: &TaskScope<'_>,
    ) -> Result<bool, FilterError> {
        self.eval(Env {
            commit,
            config: scope.config,
            task: Some(scope),
        })
    }

    /// Whether the expression calls the function `name` anywhere, nested
    /// calls included.
    #[must_use]
    pub fn calls(&self, name: &str) -> bool {
        self.tree.as_ref().is_some_and(|tree| {
            tree.iter_function_identifiers()
                .any(|identifier| restore_name(identifier) == name)
        })
    }

    fn eval(&self, env: Env<'_>) -> Result<bool, FilterError> {
        self.tree
            .as_ref()
            .map_or(Ok(true), |tree| eval_bool(tree, env))
    }
}

/// What makes one saved commit expression unusable, if anything: a syntax
/// error or a problem a dry run against an empty commit surfaces.
#[must_use]
pub(crate) fn commit_expr_problem(source: &str, config: &Config) -> Option<String> {
    CommitExpr::parse(source)
        .and_then(|expr| expr.matches(&CommitRecord::default(), config))
        .err()
        .map(|err| err.to_string())
}

fn eval_bool(node: &Node<DefaultNumericTypes>, env: Env<'_>) -> Result<bool, FilterError> {
    if let Some(result) = connective(node, &|child| eval_bool(child, env), FilterError::Invalid) {
        return result;
    }
    match node.operator() {
        Operator::Eq
        | Operator::Neq
        | Operator::Lt
        | Operator::Gt
        | Operator::Leq
        | Operator::Geq => comparison(node, env),
        Operator::FunctionIdentifier { identifier } => {
            function(&restore_name(identifier), node.children(), env)
        }
        Operator::VariableIdentifierRead { identifier } => {
            match restore_name(identifier).as_str() {
                "tasks_only" => Ok(env.commit.tasks_only),
                "named" => Ok(env.task("`named`")?.named(env.commit)),
                other => Err(not_boolean(other, env)),
            }
        }
        other => Err(FilterError::Unsupported(format!("{other:?}"))),
    }
}

/// `subject <op> "literal"`, or a state wrapper's field against a literal or
/// another wrapper: `worktree(status) != staged(status)`.
fn comparison(node: &Node<DefaultNumericTypes>, env: Env<'_>) -> Result<bool, FilterError> {
    let [lhs, rhs] = node.children() else {
        return Err(FilterError::Invalid(
            "comparison needs two operands".to_owned(),
        ));
    };
    let op = node.operator();
    let left = side(lhs, env)?
        .ok_or_else(|| FilterError::Invalid(extract_identifier(lhs).err().unwrap_or_default()))?;
    if let Some(right) = side(rhs, env)? {
        return Ok(crate::filter_compare::compare_values(op, &left, &right));
    }
    let literal = extract_literal(rhs).map_err(FilterError::Invalid)?;
    Ok(crate::filter_compare::compare(op, &left, &literal))
}

/// A comparison side that resolves rather than being written out: `subject`,
/// or a state wrapper's value. `None` for a literal.
fn side(node: &Node<DefaultNumericTypes>, env: Env<'_>) -> Result<Option<FieldValue>, FilterError> {
    let node = unwrap_root(node);
    match node.operator() {
        Operator::FunctionIdentifier { identifier } if identifier == "count" => {
            count_files(node.children(), env).map(Some)
        }
        Operator::FunctionIdentifier { identifier } if STATES.contains(&identifier.as_str()) => {
            let scope = env.task(&format!("{identifier}()"))?;
            scope.value(identifier, node.children()).map(Some)
        }
        Operator::VariableIdentifierRead { identifier } => {
            match restore_name(identifier).as_str() {
                "subject" => Ok(Some(FieldValue::Str(env.commit.subject.clone()))),
                field @ ("refs" | "files" | "trailers") => Err(FilterError::Invalid(format!(
                    "`{field}` is a list; test it with has({field}) or matches({field}, \"regex\")"
                ))),
                other => Err(unknown_field(other, env)),
            }
        }
        _ => Ok(None),
    }
}

/// `count(files, "glob"…)`: how many of the commit's files the globs admit.
fn count_files(
    children: &[Node<DefaultNumericTypes>],
    env: Env<'_>,
) -> Result<FieldValue, FilterError> {
    let invalid = |err: String| FilterError::Invalid(format!("count(): {err}"));
    let args = call_args(children);
    let Some((field, glob_nodes)) = args.split_first() else {
        return Err(invalid(
            "takes files and globs: count(files, \"!docs/**\")".to_owned(),
        ));
    };
    let field = extract_identifier(unwrap_root(field)).map_err(invalid)?;
    if field != "files" {
        return Err(invalid(format!("counts `files`, not `{field}`")));
    }
    let globs = glob_nodes
        .iter()
        .map(|node| extract_literal(node).map_err(invalid))
        .collect::<Result<Vec<_>, _>>()?;
    let admitted = PathFilter::resolve(&globs, env.config.compiled_dicts()).map_err(invalid)?;
    let files = env.commit.files.iter().map(String::as_str);
    Ok(FieldValue::Num(
        admitted.count(files, &env.commit.renamed_from) as f64,
    ))
}

fn function(
    name: &str,
    children: &[Node<DefaultNumericTypes>],
    env: Env<'_>,
) -> Result<bool, FilterError> {
    match name {
        "tasks_only" => Ok(env.commit.tasks_only),
        "named" => Ok(env.task("named()")?.named(env.commit)),
        "head" | "staged" | "worktree" => env.task(&format!("{name}()"))?.holds(name, children),
        "has" => present(&single_field(children, "has")?, env),
        "miss" => Ok(!present(&single_field(children, "miss")?, env)?),
        "matches" => matches_call(children, env),
        other => Err(FilterError::Invalid(format!(
            "unknown function in a commit rule: {other}"
        ))),
    }
}

/// The strings `field` holds on the commit: one for `subject`, any number
/// for the list fields.
fn values<'a>(field: &str, env: Env<'a>) -> Result<Vec<&'a str>, FilterError> {
    let commit = env.commit;
    match field {
        "subject" => Ok(vec![commit.subject.as_str()]),
        "refs" => Ok(commit.refs.iter().map(String::as_str).collect()),
        "files" => Ok(commit.files.iter().map(String::as_str).collect()),
        "trailers" => Ok(commit.trailers.iter().map(String::as_str).collect()),
        other => Err(unknown_field(other, env)),
    }
}

fn present(field: &str, env: Env<'_>) -> Result<bool, FilterError> {
    Ok(values(field, env)?.iter().any(|value| !value.is_empty()))
}

/// `matches(field, "regex")`: any of the field's strings matches.
fn matches_call(children: &[Node<DefaultNumericTypes>], env: Env<'_>) -> Result<bool, FilterError> {
    let args = call_args(children);
    let [field, pattern] = args.as_slice() else {
        return Err(FilterError::Invalid(
            "matches() takes two arguments: matches(files, \"^docs/\")".to_owned(),
        ));
    };
    let field = extract_identifier(field).map_err(FilterError::Invalid)?;
    let pattern = extract_literal(pattern).map_err(FilterError::Invalid)?;
    let regex = Regex::new(&pattern).map_err(|err| {
        FilterError::Invalid(format!("matches(): invalid regex {pattern:?}: {err}"))
    })?;
    Ok(values(&field, env)?
        .iter()
        .any(|value| regex.is_match(value)))
}

fn single_field(children: &[Node<DefaultNumericTypes>], name: &str) -> Result<String, FilterError> {
    match call_args(children).as_slice() {
        [arg] => extract_identifier(unwrap_root(arg)).map_err(FilterError::Invalid),
        _ => Err(FilterError::Invalid(format!("{name}() needs one argument"))),
    }
}

/// A name no commit field answers to — a task field named by mistake, most
/// likely, so say what a commit rule does read and where a task field goes.
fn unknown_field(name: &str, env: Env<'_>) -> FilterError {
    let task_field = if env.task.is_some() {
        format!(
            "read a task field through a state: staged({name}), head({name}) or worktree({name})"
        )
    } else {
        "a task field needs scope = \"task\" and a state: staged(...), head(...) or worktree(...)"
            .to_owned()
    };
    FilterError::UnknownField(format!(
        "{name} (a commit rule reads {COMMIT_FIELDS}; {task_field})"
    ))
}

fn not_boolean(name: &str, env: Env<'_>) -> FilterError {
    if matches!(name, "subject" | "refs" | "files" | "trailers") {
        FilterError::Invalid(format!(
            "bare field `{name}` is not boolean; use has({name}) or a comparison"
        ))
    } else {
        unknown_field(name, env)
    }
}

#[cfg(test)]
#[path = "commit_expr_tests.rs"]
mod tests;
