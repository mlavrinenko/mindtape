//! The condition mini-language for `[[ls.rules]]` and `[ls.columns.<name>]
//! when` (`truth/tasks/conditional-ls-rules.typ`).
//!
//! Same `evalexpr` parser and operator set as [`crate::filter`] (`==`, `!=`,
//! `<`, `>`, `<=`, `>=`, `&&`, `||`, `!`, parens, bare-or-call predicates) —
//! walked, never evaluated by `evalexpr` itself — but a separate evaluation
//! domain: ambient environment facts, never a task. A task field name (or any
//! other bare identifier) is invalid here; the condition grammar has no
//! fields, only calls.
//!
//! Predicates / values:
//! - `on_branch("glob")` -> bool. Glob-matches the current git branch (the
//!   same [`globset`] matcher [`crate::loader::derive`] uses for `derive-tags`
//!   globs); `false` when not a git repo / detached `HEAD`.
//! - `tty()` -> bool. Whether stdout is a terminal, i.e. whether a human is
//!   reading this output directly. Boolean like `on_branch`, so it is used
//!   bare (`!tty()`) rather than in a comparison.
//! - `env("NAME")` -> string (`""` when unset).
//! - `weekday()` -> int `1..7` (Mon..Sun), derived from [`ConditionEnv::today`].
//! - `now()` -> [`ConditionEnv::today`] verbatim (an ISO date string).
//! - `format()` -> string. The active `--format` value (`table`, `csv`,
//!   `json`, `path`, `path0`), so a rule can scope itself to the presentation
//!   surfaces without the distinction being hardcoded in Rust.
//! - `nonempty()`, `contested()`, `named()`, `blocked()` -> bool. The four
//!   listing facts a column's `[ls.columns.<name>] when` is decided from, read
//!   off [`ConditionEnv::column`]; see [`ColumnFacts`] for what each means and
//!   what they answer for a caller resolving no column.
//! - `true`, `false` -> bool constants, usable bare or composed (`!false`,
//!   `true && !tty()`). Not valid on either side of a comparison — `env(...)`
//!   and friends only ever yield a string or number, so a bool there is a
//!   mistake to reject rather than a case to define.
//!
//! A comparison accepts a value (`env(...)`, `now()`, `weekday()`,
//! `format()`) or a literal on either side — more general than `filter/mod.rs`,
//! which only ever compares a field (always the left side) against a literal.
//!
//! Purity: this module never reads the clock or the filesystem itself.
//! [`ConditionEnv`] carries every fact it needs (mirrors
//! [`crate::worktrees`]'s injected-fs pattern); the CLI populates it from
//! `std::env`, [`crate::current_branch`], and [`crate::today_iso`], and the
//! render layer supplies [`ColumnFacts`] per column.

use evalexpr::{DefaultNumericTypes, Node, Operator, Value, build_operator_tree};
use thiserror::Error;

use crate::filter::condition_env::{ColumnFacts, ConditionEnv};
use crate::filter::expr::{compare_ordering, connective, extract_literal, unwrap_root};
use crate::model::numeric::format_num;

/// A parsed, reusable condition expression. `None` is the always-matching
/// condition (an absent or empty/whitespace-only `condition`, e.g. an
/// `[[ls.rules]]` entry with no `condition` key).
#[derive(Debug, Clone)]
pub struct Condition {
    tree: Option<Node<DefaultNumericTypes>>,
}

/// Errors from parsing or evaluating a condition expression.
#[derive(Debug, Error)]
pub enum ConditionError {
    /// The expression failed to parse.
    #[error("failed to parse condition: {0}")]
    Parse(String),
    /// An operator the condition language does not support.
    #[error("unsupported in condition: {0}")]
    Unsupported(String),
    /// A bare identifier — the condition grammar has no fields, only calls,
    /// so this is almost always a task field name carried over from filter
    /// syntax by mistake.
    #[error("field `{0}` not available in a condition")]
    TaskField(String),
    /// A function name outside `on_branch` / `tty` / `env` / `weekday` /
    /// `now` / `format` / `nonempty` / `contested` / `named` / `blocked`.
    #[error("unknown predicate in condition: {0}")]
    UnknownPredicate(String),
    /// A structurally invalid expression (wrong arity, bad glob, ...).
    #[error("invalid condition: {0}")]
    Invalid(String),
}

impl Condition {
    /// Parse a condition expression. An empty or whitespace-only expression
    /// is the always-matching condition. Structural problems that would only
    /// otherwise surface at evaluation time — a task-field identifier, an
    /// unknown predicate, a malformed `on_branch` glob — are caught here too,
    /// by dry-running the parsed tree against [`ConditionEnv::probe`], so a
    /// bad `[[ls.rules]]` condition is a load-time error.
    ///
    /// # Errors
    /// Returns [`ConditionError`] on a syntax error, or on a structural
    /// problem caught by the dry-run.
    pub fn parse(expr: &str) -> Result<Self, ConditionError> {
        let trimmed = expr.trim();
        if trimmed.is_empty() {
            return Ok(Self { tree: None });
        }
        let tree = build_operator_tree::<DefaultNumericTypes>(trimmed)
            .map_err(|err| ConditionError::Parse(err.to_string()))?;
        let condition = Self { tree: Some(tree) };
        condition.matches(&ConditionEnv::probe())?;
        Ok(condition)
    }

    /// Whether `env` satisfies the condition.
    ///
    /// # Errors
    /// Returns [`ConditionError`] on a task-field identifier, an unknown
    /// predicate, or a malformed `on_branch` glob — all of which
    /// [`Condition::parse`] already rejects, so this only recurs for a
    /// hand-built tree bypassing `parse`.
    pub fn matches(&self, env: &ConditionEnv) -> Result<bool, ConditionError> {
        match &self.tree {
            None => Ok(true),
            Some(tree) => eval_bool(tree, env),
        }
    }
}

fn eval_bool(node: &Node<DefaultNumericTypes>, env: &ConditionEnv) -> Result<bool, ConditionError> {
    if let Some(result) = connective(
        node,
        &|child| eval_bool(child, env),
        ConditionError::Invalid,
    ) {
        return result;
    }
    match node.operator() {
        Operator::Eq
        | Operator::Neq
        | Operator::Lt
        | Operator::Gt
        | Operator::Leq
        | Operator::Geq => comparison(node, env),
        Operator::FunctionIdentifier { identifier } => predicate(identifier, node, env),
        Operator::VariableIdentifierRead { identifier } => {
            Err(ConditionError::TaskField(identifier.clone()))
        }
        Operator::Const {
            value: Value::Boolean(flag),
        } => Ok(*flag),
        other => Err(ConditionError::Unsupported(format!("{other:?}"))),
    }
}

/// A function call in boolean position: `on_branch(...)`, `tty()` and the four
/// [`ColumnFacts`] predicates are boolean; `env` / `weekday` / `now` /
/// `format` are value-producers and belong in a comparison.
fn predicate(
    name: &str,
    node: &Node<DefaultNumericTypes>,
    env: &ConditionEnv,
) -> Result<bool, ConditionError> {
    match name {
        "on_branch" => {
            let pattern = single_literal(node.children(), "on_branch")?;
            on_branch(env, &pattern)
        }
        "tty" => {
            no_args(node.children(), "tty")?;
            Ok(env.tty)
        }
        "nonempty" | "contested" | "named" | "blocked" => {
            no_args(node.children(), name)?;
            Ok(column_fact(name, env.column))
        }
        "env" | "weekday" | "now" | "format" => Err(ConditionError::Invalid(format!(
            "`{name}(...)` is not boolean; use it in a comparison"
        ))),
        other => Err(ConditionError::UnknownPredicate(other.to_owned())),
    }
}

/// Read one [`ColumnFacts`] field by predicate name. `name` is already known
/// to be one of the four — [`predicate`]'s match arm is the only caller — so
/// the fallback arm is unreachable in practice and reads `false`, which is
/// what an unknown fact means everywhere else in this domain.
fn column_fact(name: &str, facts: ColumnFacts) -> bool {
    match name {
        "nonempty" => facts.nonempty,
        "contested" => facts.contested,
        "named" => facts.named,
        _ => facts.blocked,
    }
}

/// `on_branch("glob")`: glob-match `env.branch`. The glob is always compiled
/// (even with no branch to match against), so a malformed pattern surfaces at
/// [`Condition::parse`]'s dry-run rather than only when a branch exists.
fn on_branch(env: &ConditionEnv, pattern: &str) -> Result<bool, ConditionError> {
    let matcher = globset::Glob::new(pattern)
        .map_err(|err| {
            ConditionError::Invalid(format!("on_branch: invalid glob {pattern:?}: {err}"))
        })?
        .compile_matcher();
    Ok(env
        .branch
        .as_deref()
        .is_some_and(|branch| matcher.is_match(branch)))
}

fn comparison(
    node: &Node<DefaultNumericTypes>,
    env: &ConditionEnv,
) -> Result<bool, ConditionError> {
    let [lhs, rhs] = node.children() else {
        return Err(ConditionError::Invalid(
            "comparison needs two operands".to_owned(),
        ));
    };
    let left = resolve_value(lhs, env)?;
    let right = resolve_value(rhs, env)?;
    Ok(compare_values(node.operator(), &left, &right))
}

/// A resolved comparison operand: numeric when the source was a number (a
/// literal or `weekday()`), text otherwise (`env(...)`, `now()`, a quoted
/// literal).
enum ConditionValue {
    Str(String),
    Num(f64),
}

impl ConditionValue {
    /// Canonical display form, used for a cross-type comparison fallback.
    fn display(&self) -> String {
        match self {
            Self::Str(text) => text.clone(),
            Self::Num(num) => format_num(*num),
        }
    }
}

fn compare_values(
    op: &Operator<DefaultNumericTypes>,
    left: &ConditionValue,
    right: &ConditionValue,
) -> bool {
    if let (ConditionValue::Num(left), ConditionValue::Num(right)) = (left, right) {
        return compare_ordering(op, left.partial_cmp(right));
    }
    compare_ordering(
        op,
        Some(left.display().as_str().cmp(right.display().as_str())),
    )
}

/// Resolve one side of a comparison: a value-producing call (`env`, `now`,
/// `weekday`, `format`) or a literal. The boolean predicates and a bare
/// identifier are rejected here — the former are boolean-only, the latter is
/// always a task-field mistake in this grammar.
fn resolve_value(
    node: &Node<DefaultNumericTypes>,
    env: &ConditionEnv,
) -> Result<ConditionValue, ConditionError> {
    let node = unwrap_root(node);
    match node.operator() {
        Operator::FunctionIdentifier { identifier } => resolve_function(identifier, node, env),
        Operator::VariableIdentifierRead { identifier } => {
            Err(ConditionError::TaskField(identifier.clone()))
        }
        Operator::Const {
            value: Value::Boolean(_),
        } => Err(ConditionError::Invalid(
            "a boolean literal cannot appear in a comparison".to_owned(),
        )),
        Operator::Const { .. } | Operator::Neg => literal_value(node),
        other => Err(ConditionError::Invalid(format!(
            "expected a value, got {other:?}"
        ))),
    }
}

fn literal_value(node: &Node<DefaultNumericTypes>) -> Result<ConditionValue, ConditionError> {
    let text = extract_literal(node).map_err(ConditionError::Invalid)?;
    Ok(match text.parse::<f64>() {
        Ok(num) => ConditionValue::Num(num),
        Err(_) => ConditionValue::Str(text),
    })
}

fn resolve_function(
    name: &str,
    node: &Node<DefaultNumericTypes>,
    env: &ConditionEnv,
) -> Result<ConditionValue, ConditionError> {
    match name {
        "env" => {
            let key = single_literal(node.children(), "env")?;
            Ok(ConditionValue::Str(
                env.env.get(&key).cloned().unwrap_or_default(),
            ))
        }
        "now" => {
            no_args(node.children(), "now")?;
            Ok(ConditionValue::Str(env.today.clone()))
        }
        "weekday" => {
            no_args(node.children(), "weekday")?;
            let day = weekday_number(&env.today).ok_or_else(|| {
                ConditionError::Invalid(format!("weekday(): malformed date {:?}", env.today))
            })?;
            Ok(ConditionValue::Num(f64::from(day)))
        }
        "format" => {
            no_args(node.children(), "format")?;
            Ok(ConditionValue::Str(env.format.clone()))
        }
        "on_branch" | "tty" | "nonempty" | "contested" | "named" | "blocked" => {
            Err(ConditionError::Invalid(format!(
                "{name}(...) is boolean; it cannot appear in a comparison"
            )))
        }
        other => Err(ConditionError::UnknownPredicate(other.to_owned())),
    }
}

/// `env.today` (`YYYY-MM-DD`) as an ISO weekday number, Monday = 1 .. Sunday =
/// 7. `None` when `today` does not parse as a calendar date.
fn weekday_number(today: &str) -> Option<u32> {
    let (year, month, day) = crate::date::parse_ymd(today)?;
    let date = chrono::NaiveDate::from_ymd_opt(year, u32::from(month), u32::from(day))?;
    Some(chrono::Datelike::weekday(&date).num_days_from_monday() + 1)
}

fn single_literal(
    children: &[Node<DefaultNumericTypes>],
    name: &str,
) -> Result<String, ConditionError> {
    match children {
        [arg] => extract_literal(arg).map_err(ConditionError::Invalid),
        _ => Err(ConditionError::Invalid(format!(
            "{name}() needs one argument"
        ))),
    }
}

/// A zero-argument call (`weekday()`, `now()`): `evalexpr` always wraps a
/// function's argument list in one `RootNode` child, so a genuinely empty
/// list is `[RootNode { 0 children }]`, not `[]`. Any real argument unwraps
/// past that `RootNode` to a non-`RootNode` leaf, so checking the unwrapped
/// operator (rather than a length) tells the two apart.
fn no_args(children: &[Node<DefaultNumericTypes>], name: &str) -> Result<(), ConditionError> {
    match children {
        [arg] if matches!(unwrap_root(arg).operator(), Operator::RootNode) => Ok(()),
        _ => Err(ConditionError::Invalid(format!(
            "{name}() takes no arguments"
        ))),
    }
}

#[cfg(test)]
#[path = "condition_tests.rs"]
mod tests;
