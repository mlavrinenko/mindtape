//! Shared `evalexpr`-AST helpers, used purely as a parser by both
//! [`crate::filter`] (task-field comparisons) and [`crate::filter::condition`]
//! (environment-predicate comparisons). `evalexpr` never evaluates these
//! trees itself — both callers walk the parsed [`Node`] by hand.
//!
//! Error messages are returned as plain `String`s here; each caller wraps
//! them into its own error type (`FilterError`, `ConditionError`) so the two
//! languages keep independent error surfaces while sharing the walk logic.

use evalexpr::{DefaultNumericTypes, Node, Operator, Value};

/// The stand-in a kebab-case name's `-` travels through `evalexpr` as.
/// `evalexpr` lexes `-` as minus, so [`crate::filter`] rewrites a `-` inside a
/// name to this private-use character before parsing, and every place that
/// reads a name back out of the tree restores it with [`restore_name`].
pub(crate) const NAME_HYPHEN: char = '\u{E000}';

/// A name as written: [`NAME_HYPHEN`] back to `-`.
#[must_use]
pub(crate) fn restore_name(name: &str) -> String {
    name.replace(NAME_HYPHEN, "-")
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
pub(crate) fn join_kebab_names(expr: &str) -> String {
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
        Operator::VariableIdentifierRead { identifier } => Ok(restore_name(identifier)),
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

/// The boolean connectives every expression walker shares — a root wrapper,
/// `&&`, `||` and `!` — evaluating each child through `eval`. Both sides of a
/// binary connective are evaluated, never short-circuited, so a load-time dry
/// run reaches every call. `None` for any other node, which the caller
/// evaluates itself.
pub(crate) fn connective<E>(
    node: &Node<DefaultNumericTypes>,
    eval: &dyn Fn(&Node<DefaultNumericTypes>) -> Result<bool, E>,
    invalid: fn(String) -> E,
) -> Option<Result<bool, E>> {
    let result = match (node.operator(), node.children()) {
        (Operator::RootNode | Operator::Not, [child]) => {
            let negate = matches!(node.operator(), Operator::Not);
            eval(child).map(|value| value != negate)
        }
        (Operator::RootNode, _) => Err(invalid("root needs one child".to_owned())),
        (Operator::Not, _) => Err(invalid("`!` needs one operand".to_owned())),
        (Operator::And | Operator::Or, [left, right]) => {
            let is_and = matches!(node.operator(), Operator::And);
            eval(left).and_then(|left| {
                let right = eval(right)?;
                Ok(if is_and { left && right } else { left || right })
            })
        }
        (Operator::And | Operator::Or, _) => {
            Err(invalid("binary op needs two operands".to_owned()))
        }
        _ => return None,
    };
    Some(result)
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
