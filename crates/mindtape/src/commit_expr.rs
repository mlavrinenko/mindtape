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
//! - `tasks_only` — a boolean, bare or called: the commit changes only task
//!   files.
//!
//! Functions: `has(field)` / `miss(field)` (a non-empty subject, at least one
//! ref or file) and `matches(field, "regex")` — over a list, whether ANY entry
//! matches, so `!matches(files, "^services/")` reads "touches nothing under
//! `services/`".
//!
//! A task field is refused by name: a commit rule sees the commit, not a
//! task, and a bare `status` that silently read nothing would be a rule that
//! never fires. Load dry-runs every expression against [`CommitRecord::default`],
//! so the refusal is one config error.

use evalexpr::{DefaultNumericTypes, Node, Operator, build_operator_tree};
use regex::Regex;

use crate::commit_msg::CommitRecord;
use crate::expr::{
    call_args, compare_ordering, extract_identifier, extract_literal, restore_name, unwrap_root,
};
use crate::filter::FilterError;

/// The field names a commit expression reads, for the refusal message.
const COMMIT_FIELDS: &str = "subject, refs, files, tasks_only";

/// A parsed commit-scope expression.
#[derive(Debug, Clone)]
pub struct CommitExpr {
    tree: Option<Node<DefaultNumericTypes>>,
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
    pub fn matches(&self, commit: &CommitRecord) -> Result<bool, FilterError> {
        self.tree
            .as_ref()
            .map_or(Ok(true), |tree| eval_bool(tree, commit))
    }
}

/// What makes one saved commit expression unusable, if anything: a syntax
/// error or a problem a dry run against an empty commit surfaces.
#[must_use]
pub(crate) fn commit_expr_problem(source: &str) -> Option<String> {
    CommitExpr::parse(source)
        .and_then(|expr| expr.matches(&CommitRecord::default()))
        .err()
        .map(|err| err.to_string())
}

fn eval_bool(node: &Node<DefaultNumericTypes>, commit: &CommitRecord) -> Result<bool, FilterError> {
    match node.operator() {
        Operator::RootNode => match node.children() {
            [child] => eval_bool(child, commit),
            _ => Err(FilterError::Invalid("root needs one child".to_owned())),
        },
        Operator::And => binary(node, commit, |left, right| left && right),
        Operator::Or => binary(node, commit, |left, right| left || right),
        Operator::Not => match node.children() {
            [child] => Ok(!eval_bool(child, commit)?),
            _ => Err(FilterError::Invalid("`!` needs one operand".to_owned())),
        },
        Operator::Eq
        | Operator::Neq
        | Operator::Lt
        | Operator::Gt
        | Operator::Leq
        | Operator::Geq => comparison(node, commit),
        Operator::FunctionIdentifier { identifier } => {
            function(&restore_name(identifier), node.children(), commit)
        }
        Operator::VariableIdentifierRead { identifier } => {
            match restore_name(identifier).as_str() {
                "tasks_only" => Ok(commit.tasks_only),
                other => Err(not_boolean(other)),
            }
        }
        other => Err(FilterError::Unsupported(format!("{other:?}"))),
    }
}

fn binary(
    node: &Node<DefaultNumericTypes>,
    commit: &CommitRecord,
    combine: impl Fn(bool, bool) -> bool,
) -> Result<bool, FilterError> {
    match node.children() {
        [left, right] => Ok(combine(eval_bool(left, commit)?, eval_bool(right, commit)?)),
        _ => Err(FilterError::Invalid(
            "binary op needs two operands".to_owned(),
        )),
    }
}

/// `subject <op> "literal"` — the one scalar field a comparison reads.
fn comparison(
    node: &Node<DefaultNumericTypes>,
    commit: &CommitRecord,
) -> Result<bool, FilterError> {
    let [lhs, rhs] = node.children() else {
        return Err(FilterError::Invalid(
            "comparison needs two operands".to_owned(),
        ));
    };
    let field = extract_identifier(lhs).map_err(FilterError::Invalid)?;
    let literal = extract_literal(rhs).map_err(FilterError::Invalid)?;
    match field.as_str() {
        "subject" => Ok(compare_ordering(
            node.operator(),
            Some(commit.subject.as_str().cmp(literal.as_str())),
        )),
        "refs" | "files" => Err(FilterError::Invalid(format!(
            "`{field}` is a list; test it with has({field}) or matches({field}, \"regex\")"
        ))),
        other => Err(unknown_field(other)),
    }
}

fn function(
    name: &str,
    children: &[Node<DefaultNumericTypes>],
    commit: &CommitRecord,
) -> Result<bool, FilterError> {
    match name {
        "tasks_only" => Ok(commit.tasks_only),
        "has" => present(&single_field(children, "has")?, commit),
        "miss" => Ok(!present(&single_field(children, "miss")?, commit)?),
        "matches" => matches_call(children, commit),
        other => Err(FilterError::Invalid(format!(
            "unknown function in a commit rule: {other}"
        ))),
    }
}

/// The strings `field` holds on `commit`: one for `subject`, any number for
/// the list fields.
fn values<'a>(field: &str, commit: &'a CommitRecord) -> Result<Vec<&'a str>, FilterError> {
    match field {
        "subject" => Ok(vec![commit.subject.as_str()]),
        "refs" => Ok(commit.refs.iter().map(String::as_str).collect()),
        "files" => Ok(commit.files.iter().map(String::as_str).collect()),
        other => Err(unknown_field(other)),
    }
}

fn present(field: &str, commit: &CommitRecord) -> Result<bool, FilterError> {
    Ok(values(field, commit)?.iter().any(|value| !value.is_empty()))
}

/// `matches(field, "regex")`: any of the field's strings matches.
fn matches_call(
    children: &[Node<DefaultNumericTypes>],
    commit: &CommitRecord,
) -> Result<bool, FilterError> {
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
    Ok(values(&field, commit)?
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
/// likely, so say what a commit rule does read.
fn unknown_field(name: &str) -> FilterError {
    FilterError::UnknownField(format!(
        "{name} (a commit rule reads {COMMIT_FIELDS}, not task fields)"
    ))
}

fn not_boolean(name: &str) -> FilterError {
    if matches!(name, "subject" | "refs" | "files") {
        FilterError::Invalid(format!(
            "bare field `{name}` is not boolean; use has({name}) or a comparison"
        ))
    } else {
        unknown_field(name)
    }
}

#[cfg(test)]
#[path = "commit_expr_tests.rs"]
mod tests;
