//! Translate `evalexpr` expression strings into SQL WHERE fragments.
//!
//! Uses `evalexpr` **only as a parser** — the AST is walked to produce SQL,
//! never evaluated in Rust.  This keeps filtering inside `SQLite`.
//!
//! # Supported syntax
//!
//! **Fields:** `done`, `due`, `start`, `rank`, `title`, `milestone`, `file`, `id`
//!
//! **Operators:** `==`, `!=`, `<`, `>`, `<=`, `>=`, `&&`, `||`, `!`, parentheses
//!
//! **Functions:**
//! - `has(field)` — presence check
//! - `miss(field)` — absence check (equivalent to `!has(field)`)
//! - `has_tag("name")` — specific tag
//! - `search("query")` — FTS5 full-text search
//! - `contains(field, "substr")` — case-insensitive substring

use evalexpr::{DefaultNumericTypes, Node, Operator, Value, build_operator_tree};

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// SQL fragment produced by translating one AST node.
pub(crate) struct SqlFragment {
    /// SQL WHERE clause fragment (e.g. `"(t.due IS NOT NULL AND t.due < ?)"`)
    pub condition: String,
    /// Positional parameter values to bind.
    pub params: Vec<String>,
    /// Whether `search()` was used (requires FTS JOIN).
    pub needs_fts_join: bool,
}

/// Errors from filter-expression translation.
#[derive(Debug, thiserror::Error)]
pub enum FilterExprError {
    #[error("failed to parse filter expression: {0}")]
    Parse(String),
    #[error("unsupported operator in filter expression: {0}")]
    UnsupportedOperator(String),
    #[error("unknown field in filter expression: {0}")]
    UnknownField(String),
    #[error("invalid filter expression: {0}")]
    Invalid(String),
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

/// Parse an expression and translate it into a SQL WHERE fragment.
///
/// # Errors
///
/// Returns `FilterExprError` on syntax errors, unknown fields, or
/// unsupported operators.
pub(crate) fn translate_expr(expr: &str) -> Result<SqlFragment, FilterExprError> {
    let tree = build_operator_tree::<DefaultNumericTypes>(expr)
        .map_err(|e| FilterExprError::Parse(e.to_string()))?;
    translate_node(&tree)
}

// ---------------------------------------------------------------------------
// Recursive translator
// ---------------------------------------------------------------------------

fn translate_node(node: &Node<DefaultNumericTypes>) -> Result<SqlFragment, FilterExprError> {
    match node.operator() {
        Operator::RootNode => translate_root(node),
        Operator::And => translate_binary(node, "AND"),
        Operator::Or => translate_binary(node, "OR"),
        Operator::Not => translate_not(node),
        Operator::Eq
        | Operator::Neq
        | Operator::Lt
        | Operator::Gt
        | Operator::Leq
        | Operator::Geq => translate_comparison(node),
        Operator::VariableIdentifierRead { identifier } => translate_bare_variable(identifier),
        Operator::FunctionIdentifier { identifier } => translate_function(identifier, node),
        other => Err(FilterExprError::UnsupportedOperator(format!("{other:?}"))),
    }
}

fn translate_root(node: &Node<DefaultNumericTypes>) -> Result<SqlFragment, FilterExprError> {
    match node.children() {
        [child] => translate_node(child),
        _ => Err(FilterExprError::Invalid(
            "root node must have exactly one child".into(),
        )),
    }
}

// ---------------------------------------------------------------------------
// Boolean combinators
// ---------------------------------------------------------------------------

fn translate_binary(
    node: &Node<DefaultNumericTypes>,
    sql_op: &str,
) -> Result<SqlFragment, FilterExprError> {
    match node.children() {
        [left, right] => {
            let left = translate_node(left)?;
            let right = translate_node(right)?;
            Ok(SqlFragment::combine(left, right, sql_op))
        }
        _ => Err(FilterExprError::Invalid(format!(
            "{sql_op} requires exactly 2 operands"
        ))),
    }
}

fn translate_not(node: &Node<DefaultNumericTypes>) -> Result<SqlFragment, FilterExprError> {
    match node.children() {
        [child] => {
            let inner = translate_node(child)?;
            Ok(SqlFragment {
                condition: format!("NOT ({})", inner.condition),
                params: inner.params,
                needs_fts_join: inner.needs_fts_join,
            })
        }
        _ => Err(FilterExprError::Invalid(
            "NOT requires exactly 1 operand".into(),
        )),
    }
}

// ---------------------------------------------------------------------------
// Comparisons
// ---------------------------------------------------------------------------

fn translate_comparison(node: &Node<DefaultNumericTypes>) -> Result<SqlFragment, FilterExprError> {
    match node.children() {
        [lhs, rhs] => build_comparison(node.operator(), lhs, rhs),
        _ => Err(FilterExprError::Invalid(
            "comparison requires exactly 2 operands".into(),
        )),
    }
}

fn build_comparison(
    op: &Operator<DefaultNumericTypes>,
    lhs: &Node<DefaultNumericTypes>,
    rhs: &Node<DefaultNumericTypes>,
) -> Result<SqlFragment, FilterExprError> {
    let field_name = extract_identifier(lhs)?;
    let literal = extract_literal(rhs)?;
    let col = resolve_field(&field_name)?;
    let sql_op = operator_to_sql(op)?;

    // Boolean field `done` uses 0/1.
    if field_name == "done" {
        let int_val = bool_literal_to_int(&literal)?;
        return Ok(SqlFragment {
            condition: format!("{col} = ?"),
            params: vec![int_val],
            needs_fts_join: false,
        });
    }

    // Nullable fields need IS NOT NULL guard for inequalities.
    if is_nullable_field(&field_name) && is_inequality(op) {
        Ok(SqlFragment {
            condition: format!("({col} IS NOT NULL AND {col} {sql_op} ?)"),
            params: vec![literal],
            needs_fts_join: false,
        })
    } else {
        Ok(SqlFragment {
            condition: format!("{col} {sql_op} ?"),
            params: vec![literal],
            needs_fts_join: false,
        })
    }
}

// ---------------------------------------------------------------------------
// Bare variables (e.g. `done` or `!done`)
// ---------------------------------------------------------------------------

fn translate_bare_variable(identifier: &str) -> Result<SqlFragment, FilterExprError> {
    if identifier == "done" {
        return Ok(SqlFragment {
            condition: "t.is_done = 1".into(),
            params: vec![],
            needs_fts_join: false,
        });
    }
    Err(FilterExprError::Invalid(format!(
        "bare field \"{identifier}\" is not boolean; use a comparison or function"
    )))
}

// ---------------------------------------------------------------------------
// Functions
// ---------------------------------------------------------------------------

fn translate_function(
    name: &str,
    node: &Node<DefaultNumericTypes>,
) -> Result<SqlFragment, FilterExprError> {
    let children = node.children();
    match name {
        "has" => translate_fn_has(children),
        "miss" => translate_fn_miss(children),
        "has_tag" => translate_fn_has_tag(children),
        "search" => translate_fn_search(children),
        "contains" => translate_fn_contains(children),
        _ => Err(FilterExprError::Invalid(format!(
            "unknown function: {name}"
        ))),
    }
}

/// `has(field)` — presence check.
fn translate_fn_has(
    children: &[Node<DefaultNumericTypes>],
) -> Result<SqlFragment, FilterExprError> {
    let arg = single_arg(children, "has")?;
    let field_name = extract_identifier(arg)?;
    match field_name.as_str() {
        "due" => Ok(sql_no_params("t.due IS NOT NULL")),
        "start" => Ok(sql_no_params("t.start IS NOT NULL")),
        "rank" => Ok(sql_no_params("t.rank IS NOT NULL")),
        "id" => Ok(sql_no_params("t.task_id IS NOT NULL")),
        "tag" => Ok(sql_no_params(
            "EXISTS (SELECT 1 FROM task_properties tp \
             WHERE tp.task_id = t.id AND tp.kind = 'mindtape.tag')",
        )),
        other => Err(FilterExprError::UnknownField(format!(
            "has({other}): unknown property"
        ))),
    }
}

/// `miss(field)` — absence check (negation of `has`).
fn translate_fn_miss(
    children: &[Node<DefaultNumericTypes>],
) -> Result<SqlFragment, FilterExprError> {
    let arg = single_arg(children, "miss")?;
    let field_name = extract_identifier(arg)?;
    match field_name.as_str() {
        "due" => Ok(sql_no_params("t.due IS NULL")),
        "start" => Ok(sql_no_params("t.start IS NULL")),
        "rank" => Ok(sql_no_params("t.rank IS NULL")),
        "id" => Ok(sql_no_params("t.task_id IS NULL")),
        "tag" => Ok(sql_no_params(
            "NOT EXISTS (SELECT 1 FROM task_properties tp \
             WHERE tp.task_id = t.id AND tp.kind = 'mindtape.tag')",
        )),
        other => Err(FilterExprError::UnknownField(format!(
            "miss({other}): unknown property"
        ))),
    }
}

/// `has_tag("name")` — specific tag check.
fn translate_fn_has_tag(
    children: &[Node<DefaultNumericTypes>],
) -> Result<SqlFragment, FilterExprError> {
    let arg = single_arg(children, "has_tag")?;
    let tag_name = extract_literal(arg)?;
    Ok(SqlFragment {
        condition: "EXISTS (SELECT 1 FROM task_properties tp \
                    WHERE tp.task_id = t.id AND tp.kind = 'mindtape.tag' AND tp.value = ?)"
            .into(),
        params: vec![tag_name],
        needs_fts_join: false,
    })
}

/// `search("query")` — FTS5 full-text search.
fn translate_fn_search(
    children: &[Node<DefaultNumericTypes>],
) -> Result<SqlFragment, FilterExprError> {
    let arg = single_arg(children, "search")?;
    let query = extract_literal(arg)?;
    Ok(SqlFragment {
        condition: "tasks_fts MATCH ?".into(),
        params: vec![query],
        needs_fts_join: true,
    })
}

/// `contains(field, "substr")` — case-insensitive substring match.
fn translate_fn_contains(
    children: &[Node<DefaultNumericTypes>],
) -> Result<SqlFragment, FilterExprError> {
    let (field_node, value_node) = two_args(children, "contains")?;
    let field_name = extract_identifier(field_node)?;
    let substr = extract_literal(value_node)?;
    let col = resolve_field(&field_name)?;
    Ok(SqlFragment {
        condition: format!("{col} LIKE '%' || ? || '%' COLLATE NOCASE"),
        params: vec![substr],
        needs_fts_join: false,
    })
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

impl SqlFragment {
    fn combine(left: Self, right: Self, op: &str) -> Self {
        let mut params = left.params;
        params.extend(right.params);
        Self {
            condition: format!("({} {op} {})", left.condition, right.condition),
            params,
            needs_fts_join: left.needs_fts_join || right.needs_fts_join,
        }
    }
}

fn sql_no_params(condition: &str) -> SqlFragment {
    SqlFragment {
        condition: condition.into(),
        params: vec![],
        needs_fts_join: false,
    }
}

/// Resolve a user-facing field name to a SQL column expression.
fn resolve_field(name: &str) -> Result<&'static str, FilterExprError> {
    match name {
        "done" => Ok("t.is_done"),
        "due" => Ok("t.due"),
        "start" => Ok("t.start"),
        "rank" => Ok("t.rank"),
        "title" => Ok("t.title"),
        "milestone" => Ok("t.milestone"),
        "file" => Ok("tf.file_path"),
        "id" => Ok("t.task_id"),
        other => Err(FilterExprError::UnknownField(other.to_string())),
    }
}

fn is_nullable_field(name: &str) -> bool {
    matches!(name, "due" | "start" | "rank" | "id" | "milestone")
}

fn is_inequality(op: &Operator<DefaultNumericTypes>) -> bool {
    matches!(
        op,
        Operator::Lt | Operator::Gt | Operator::Leq | Operator::Geq
    )
}

fn operator_to_sql(op: &Operator<DefaultNumericTypes>) -> Result<&'static str, FilterExprError> {
    match op {
        Operator::Eq => Ok("="),
        Operator::Neq => Ok("!="),
        Operator::Lt => Ok("<"),
        Operator::Gt => Ok(">"),
        Operator::Leq => Ok("<="),
        Operator::Geq => Ok(">="),
        other => Err(FilterExprError::UnsupportedOperator(format!("{other:?}"))),
    }
}

fn bool_literal_to_int(literal: &str) -> Result<String, FilterExprError> {
    match literal {
        "1" | "true" => Ok("1".into()),
        "0" | "false" => Ok("0".into()),
        other => Err(FilterExprError::Invalid(format!(
            "done field requires a boolean value, got: {other}"
        ))),
    }
}

/// Unwrap transparent `RootNode` wrappers that evalexpr inserts around
/// function arguments and parenthesised sub-expressions.
fn unwrap_root(node: &Node<DefaultNumericTypes>) -> &Node<DefaultNumericTypes> {
    if let Operator::RootNode = node.operator() {
        match node.children() {
            [child] => unwrap_root(child),
            _ => node,
        }
    } else {
        node
    }
}

/// Extract a variable identifier from a node.
fn extract_identifier(node: &Node<DefaultNumericTypes>) -> Result<String, FilterExprError> {
    let node = unwrap_root(node);
    match node.operator() {
        Operator::VariableIdentifierRead { identifier } => Ok(identifier.clone()),
        other => Err(FilterExprError::Invalid(format!(
            "expected a field name, got: {other:?}"
        ))),
    }
}

/// Extract a literal value from a `Const` node as a string.
fn extract_literal(node: &Node<DefaultNumericTypes>) -> Result<String, FilterExprError> {
    let node = unwrap_root(node);
    match node.operator() {
        Operator::Const { value } => value_to_string(value),
        // evalexpr parses negative numbers as Neg(Const(positive)).
        Operator::Neg => match node.children() {
            [child] => {
                let inner = extract_literal(child)?;
                Ok(format!("-{inner}"))
            }
            _ => Err(FilterExprError::Invalid("malformed negation".into())),
        },
        other => Err(FilterExprError::Invalid(format!(
            "expected a literal value, got: {other:?}"
        ))),
    }
}

fn value_to_string(value: &Value<DefaultNumericTypes>) -> Result<String, FilterExprError> {
    match value {
        Value::String(str_val) => Ok(str_val.clone()),
        Value::Int(int_val) => Ok(int_val.to_string()),
        Value::Float(float_val) => Ok(float_val.to_string()),
        Value::Boolean(bool_val) => Ok(if *bool_val { "1" } else { "0" }.into()),
        _ => Err(FilterExprError::Invalid(
            "unsupported literal type (tuple or empty)".into(),
        )),
    }
}

/// Get the single argument node for a 1-arg function.
///
/// evalexpr stores function args as child nodes of the `FunctionIdentifier`.
fn single_arg<'a>(
    children: &'a [Node<DefaultNumericTypes>],
    fn_name: &str,
) -> Result<&'a Node<DefaultNumericTypes>, FilterExprError> {
    match children {
        [arg] => Ok(arg),
        _ => Err(FilterExprError::Invalid(format!(
            "{fn_name}() requires exactly 1 argument, got {}",
            children.len()
        ))),
    }
}

/// Get two argument nodes for a 2-arg function.
///
/// evalexpr wraps multi-arg function calls in `RootNode(Tuple(...))`, so
/// `contains(title, "foo")` has a single child: `RootNode -> Tuple(2)`.
fn two_args<'a>(
    children: &'a [Node<DefaultNumericTypes>],
    fn_name: &str,
) -> Result<(&'a Node<DefaultNumericTypes>, &'a Node<DefaultNumericTypes>), FilterExprError> {
    if let [child] = children {
        let unwrapped = unwrap_root(child);
        if let Operator::Tuple = unwrapped.operator()
            && let [first, second] = unwrapped.children()
        {
            return Ok((first, second));
        }
    }
    Err(FilterExprError::Invalid(format!(
        "{fn_name}() requires exactly 2 arguments"
    )))
}

#[cfg(test)]
#[path = "filter_expr_tests.rs"]
mod tests;
