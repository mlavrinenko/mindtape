//! Framework formula evaluation: scalar scoring, category predicates, and
//! interval analysis of a score over its declared input bounds.
//!
//! `evalexpr` is reused as both parser and evaluator here (unlike the filter
//! language, which only walks the parsed tree). The score formula and category
//! `when` predicates are evaluated against a context built from a task's typed
//! inputs. [`score_interval`] instead walks the parsed tree with interval
//! arithmetic so config load can prove a framework's achievable range fits the
//! priority scale — rejecting (never clamping) any framework that overflows.

use std::collections::BTreeMap;

use evalexpr::{
    ContextWithMutableVariables, DefaultNumericTypes, HashMapContext, Node, Operator, Value,
    build_operator_tree,
};

use crate::framework::{Input, InputKind};
use crate::model::numeric::InputValue;

/// A closed numeric interval `[lo, hi]`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Interval {
    /// Lower bound.
    pub lo: f64,
    /// Upper bound.
    pub hi: f64,
}

impl Interval {
    fn point(value: f64) -> Self {
        Self {
            lo: value,
            hi: value,
        }
    }
}

type Tree = Node<DefaultNumericTypes>;

/// Build an evaluation context binding each input name to its typed value.
fn context_of(inputs: &BTreeMap<String, InputValue>) -> HashMapContext {
    let mut ctx = HashMapContext::<DefaultNumericTypes>::new();
    for (name, value) in inputs {
        let value = match value {
            InputValue::Num(num) => Value::Float(*num),
            InputValue::Bool(flag) => Value::Boolean(*flag),
            InputValue::Text(text) => Value::String(text.clone()),
        };
        // Fresh keys never clash, so a set error cannot occur here.
        let _ = ctx.set_value(name.clone(), value);
    }
    ctx
}

/// Evaluate a scalar `score` formula against `inputs`, returning the number.
///
/// # Errors
/// Returns a message if the formula fails to parse or evaluate.
pub fn eval_score(formula: &str, inputs: &BTreeMap<String, InputValue>) -> Result<f64, String> {
    let tree = build_operator_tree::<DefaultNumericTypes>(formula)
        .map_err(|err| format!("invalid score formula '{formula}': {err}"))?;
    let ctx = context_of(inputs);
    tree.eval_number_with_context(&ctx)
        .map_err(|err| format!("score '{formula}' failed: {err}"))
}

/// Evaluate a category `when` predicate against `inputs`.
///
/// # Errors
/// Returns a message if the predicate fails to parse or evaluate to a boolean.
pub fn eval_predicate(when: &str, inputs: &BTreeMap<String, InputValue>) -> Result<bool, String> {
    let tree = build_operator_tree::<DefaultNumericTypes>(when)
        .map_err(|err| format!("invalid predicate '{when}': {err}"))?;
    let ctx = context_of(inputs);
    tree.eval_boolean_with_context(&ctx)
        .map_err(|err| format!("predicate '{when}' failed: {err}"))
}

/// Parse-check a formula or predicate without evaluating it.
///
/// # Errors
/// Returns a message if the expression fails to parse.
pub fn parse_check(expr: &str) -> Result<(), String> {
    build_operator_tree::<DefaultNumericTypes>(expr)
        .map(|_| ())
        .map_err(|err| format!("invalid expression '{expr}': {err}"))
}

/// Interval-evaluate a `score` formula over the declared input bounds.
///
/// # Errors
/// Returns a message if the formula references an unbounded/non-numeric input or
/// uses an operator interval arithmetic does not support (a safe over-rejection).
pub fn score_interval(formula: &str, inputs: &BTreeMap<String, Input>) -> Result<Interval, String> {
    let tree = build_operator_tree::<DefaultNumericTypes>(formula)
        .map_err(|err| format!("invalid score formula '{formula}': {err}"))?;
    let vars = input_intervals(inputs);
    interval_of(&tree, &vars)
}

/// The numeric interval of each input that can feed a scalar formula: numbers
/// span their `[min, max]`, booleans span `[0, 1]`; enums are omitted (they only
/// appear in categorical predicates, never in arithmetic).
fn input_intervals(inputs: &BTreeMap<String, Input>) -> BTreeMap<String, Interval> {
    let mut vars = BTreeMap::new();
    for (name, input) in inputs {
        match input.kind {
            InputKind::Number => {
                if let (Some(lo), Some(hi)) = (input.min, input.max) {
                    vars.insert(name.clone(), Interval { lo, hi });
                }
            }
            InputKind::Bool => {
                vars.insert(name.clone(), Interval { lo: 0.0, hi: 1.0 });
            }
            InputKind::Enum => {}
        }
    }
    vars
}

fn interval_of(node: &Tree, vars: &BTreeMap<String, Interval>) -> Result<Interval, String> {
    match node.operator() {
        Operator::RootNode => match node.children() {
            [child] => interval_of(child, vars),
            _ => Err("the score formula is incomplete".to_owned()),
        },
        Operator::Const { value } => const_interval(value),
        Operator::VariableIdentifierRead { identifier } => vars
            .get(identifier)
            .copied()
            .ok_or_else(|| format!("score references unbounded input '{identifier}'")),
        Operator::Neg => {
            let inner = unary(node, vars)?;
            Ok(Interval {
                lo: -inner.hi,
                hi: -inner.lo,
            })
        }
        Operator::Add | Operator::Sub | Operator::Mul | Operator::Div => arithmetic(node, vars),
        Operator::FunctionIdentifier { identifier } => function_interval(identifier, node, vars),
        other => Err(crate::filter::expr::operator_symbol(other).map_or_else(
            || "the score formula has a part a score can't use".to_owned(),
            |symbol| {
                format!("a score can't use '{symbol}'; it takes + - * / and round, floor or ceil")
            },
        )),
    }
}

fn const_interval(value: &Value<DefaultNumericTypes>) -> Result<Interval, String> {
    match value {
        Value::Int(int) => Ok(Interval::point(*int as f64)),
        Value::Float(float) => Ok(Interval::point(*float)),
        other => Err(format!("score uses the non-numeric constant '{other}'")),
    }
}

fn unary(node: &Tree, vars: &BTreeMap<String, Interval>) -> Result<Interval, String> {
    match node.children() {
        [child] => interval_of(child, vars),
        _ => Err("the score formula is incomplete".to_owned()),
    }
}

fn binary(node: &Tree, vars: &BTreeMap<String, Interval>) -> Result<(Interval, Interval), String> {
    match node.children() {
        [left, right] => Ok((interval_of(left, vars)?, interval_of(right, vars)?)),
        _ => Err("the score formula is incomplete".to_owned()),
    }
}

fn arithmetic(node: &Tree, vars: &BTreeMap<String, Interval>) -> Result<Interval, String> {
    let (left, right) = binary(node, vars)?;
    match node.operator() {
        Operator::Add => Ok(Interval {
            lo: left.lo + right.lo,
            hi: left.hi + right.hi,
        }),
        Operator::Sub => Ok(Interval {
            lo: left.lo - right.hi,
            hi: left.hi - right.lo,
        }),
        Operator::Mul => Ok(mul(left, right)),
        Operator::Div => div(left, right),
        _ => Err("not an arithmetic operator".to_owned()),
    }
}

fn mul(left: Interval, right: Interval) -> Interval {
    let corners = [
        left.lo * right.lo,
        left.lo * right.hi,
        left.hi * right.lo,
        left.hi * right.hi,
    ];
    span(&corners)
}

fn div(left: Interval, right: Interval) -> Result<Interval, String> {
    if right.lo <= 0.0 && right.hi >= 0.0 {
        return Err("score divides by an input range that spans zero".to_owned());
    }
    // Reciprocal of a zero-free interval is monotone.
    let recip = Interval {
        lo: 1.0 / right.hi,
        hi: 1.0 / right.lo,
    };
    Ok(mul(left, recip))
}

fn function_interval(
    name: &str,
    node: &Tree,
    vars: &BTreeMap<String, Interval>,
) -> Result<Interval, String> {
    let inner = unary(node, vars)?;
    match name {
        "round" => Ok(Interval {
            lo: inner.lo.round(),
            hi: inner.hi.round(),
        }),
        "floor" => Ok(Interval {
            lo: inner.lo.floor(),
            hi: inner.hi.floor(),
        }),
        "ceil" => Ok(Interval {
            lo: inner.lo.ceil(),
            hi: inner.hi.ceil(),
        }),
        other => Err(format!(
            "a score can't call '{other}'; it takes round, floor or ceil"
        )),
    }
}

fn span(values: &[f64]) -> Interval {
    let mut lo = f64::INFINITY;
    let mut hi = f64::NEG_INFINITY;
    for &value in values {
        lo = lo.min(value);
        hi = hi.max(value);
    }
    Interval { lo, hi }
}

#[cfg(test)]
#[path = "eval_tests.rs"]
mod tests;
