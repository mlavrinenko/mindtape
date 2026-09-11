//! Shared `evalexpr`-AST helpers, used purely as a parser by both
//! [`crate::filter`] (task-field comparisons) and [`crate::condition`]
//! (environment-predicate comparisons). `evalexpr` never evaluates these
//! trees itself — both callers walk the parsed [`Node`] by hand.
//!
//! Error messages are returned as plain `String`s here; each caller wraps
//! them into its own error type (`FilterError`, `ConditionError`) so the two
//! languages keep independent error surfaces while sharing the walk logic.

use evalexpr::{DefaultNumericTypes, Node, Operator, Value};

/// Recurse through nested [`Operator::RootNode`] wrappers down to the real
/// node underneath.
pub(crate) fn unwrap_root(node: &Node<DefaultNumericTypes>) -> &Node<DefaultNumericTypes> {
    if let Operator::RootNode = node.operator()
        && let [child] = node.children()
    {
        return unwrap_root(child);
    }
    node
}

/// Extract a bare identifier (a field or predicate name in comparison
/// position).
///
/// # Errors
/// Returns a message if `node` is not a variable-identifier read.
pub(crate) fn extract_identifier(node: &Node<DefaultNumericTypes>) -> Result<String, String> {
    match unwrap_root(node).operator() {
        Operator::VariableIdentifierRead { identifier } => Ok(identifier.clone()),
        other => Err(format!("expected a field, got {other:?}")),
    }
}

/// Extract a literal value (string / int / float / bool, or a negated
/// numeric literal) as its string form.
///
/// # Errors
/// Returns a message if `node` is not a literal (or a negation of one).
pub(crate) fn extract_literal(node: &Node<DefaultNumericTypes>) -> Result<String, String> {
    let node = unwrap_root(node);
    match node.operator() {
        Operator::Const { value } => literal_to_string(value),
        Operator::Neg => match node.children() {
            [child] => Ok(format!("-{}", extract_literal(child)?)),
            _ => Err("malformed negation".to_owned()),
        },
        other => Err(format!("expected a literal, got {other:?}")),
    }
}

fn literal_to_string(value: &Value<DefaultNumericTypes>) -> Result<String, String> {
    match value {
        Value::String(text) => Ok(text.clone()),
        Value::Int(num) => Ok(num.to_string()),
        Value::Float(num) => Ok(num.to_string()),
        Value::Boolean(flag) => Ok(if *flag { "true" } else { "false" }.to_owned()),
        _ => Err("unsupported literal".to_owned()),
    }
}

/// All arguments of a call, left to right. `evalexpr` wraps a call's argument
/// list in one child: zero args unwrap to an empty `RootNode`, one arg is the
/// bare node, two or more is a `Tuple`.
pub(crate) fn call_args(children: &[Node<DefaultNumericTypes>]) -> Vec<&Node<DefaultNumericTypes>> {
    let [wrapper] = children else {
        return Vec::new();
    };
    match unwrap_root(wrapper).operator() {
        Operator::RootNode => Vec::new(),
        Operator::Tuple => unwrap_root(wrapper).children().iter().collect(),
        _ => vec![wrapper],
    }
}

/// Apply a comparison operator to a resolved ordering. `None` (incomparable
/// operands) only satisfies `!=`, matching how a missing field or a
/// type-mismatched comparison behaves elsewhere in both languages.
pub(crate) fn compare_ordering(
    op: &Operator<DefaultNumericTypes>,
    ordering: Option<std::cmp::Ordering>,
) -> bool {
    use std::cmp::Ordering::{Equal, Greater, Less};
    let Some(ordering) = ordering else {
        return matches!(op, Operator::Neq);
    };
    match op {
        Operator::Eq => ordering == Equal,
        Operator::Neq => ordering != Equal,
        Operator::Lt => ordering == Less,
        Operator::Gt => ordering == Greater,
        Operator::Leq => ordering != Greater,
        Operator::Geq => ordering != Less,
        _ => false,
    }
}

/// Resolve a comparison-side node that may be `now()` (today's ISO date) or a
/// literal. Any other function name is rejected at this position — this is
/// the filter language's narrower "field == `literal-or-now()`" grammar; the
/// condition language resolves its comparison sides differently (either side
/// may be a value-producing call), so it does not use this helper.
///
/// # Errors
/// Returns a message if `node` is neither `now()` nor a literal.
pub(crate) fn resolve_now_or_literal(node: &Node<DefaultNumericTypes>) -> Result<String, String> {
    let node = unwrap_root(node);
    if let Operator::FunctionIdentifier { identifier } = node.operator() {
        return match identifier.as_str() {
            "now" => Ok(crate::date::today_iso()),
            other => Err(format!(
                "function `{other}` cannot be used as a comparison value; only `now()` is allowed here"
            )),
        };
    }
    extract_literal(node)
}

#[cfg(test)]
#[path = "expr_tests.rs"]
mod tests;
